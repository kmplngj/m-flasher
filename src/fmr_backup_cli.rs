//! Complete raw acquisition through the pinned read-only reader, outside the
//! legacy loader/writer session. File preparation precedes all native access.

use crate::fmr_backup_options::{self, Command, Options};
use crate::fmr_bootstrap::{IMAGE_SHA256, Image};
use crate::fmr_cli::{load_image, quote};
use crate::fmr_full::{Capture, EXPECTED_DMA};
use crate::fmr_storage::{Policy, PreparedPair};

const USAGE: &str = "m-flasher raw-backup (--prepare | --run) --reader ABSOLUTE_IMAGE.bin \
    --device usb:BUS-ADDRESS --location XXXXXXXX --serial STOCK_SERIAL \
    --copy-a ABSOLUTE_NEW_DIRECTORY --copy-b ABSOLUTE_NEW_DIRECTORY [--allow-same-filesystem]\n\
Two fresh complete raw 1 MiB passes, SHA-256 and byte comparison, protected reopened copies.\n\
--prepare creates/preflights private empty storage only; no devices are accessed.\n\
--run requires reader-qualification and an explicitly selected fresh ROM session.\n\
Different filesystems are required unless --allow-same-filesystem explicitly records reduced redundancy.\n\
No flash/status writes, FC0A, automatic reset/retry or fallback. Manual cold return remains separate.";

pub fn run(args: &[String]) -> i32 {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!("{USAGE}");
        return 0;
    }
    match execute(args) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("Raw backup stopped: {error}");
            1
        }
    }
}

fn execute(args: &[String]) -> Result<(), String> {
    let (options, run) = match fmr_backup_options::parse(args)? {
        Command::Prepare(options) => (options, false),
        Command::Run(options) => (options, true),
    };
    let image = load_image(&options.reader)?;
    // All output descriptors, identities and policy exist before a helper can
    // start. Prepare intentionally reserves new names; run uses fresh names.
    let mut pair = PreparedPair::prepare(
        &options.copy_a,
        &options.copy_b,
        if options.allow_same_filesystem {
            Policy::SeparateDirectories
        } else {
            Policy::DifferentFilesystems
        },
    )?;
    if !run {
        println!(
            "Storage prepared at {} and {}. No devices accessed; --run needs two new directories.",
            options.copy_a.display(),
            options.copy_b.display()
        );
        return Ok(());
    }
    run_selected(options, image, &mut pair)
}

fn provenance(options: &Options, capture: &Capture, binding: &str) -> String {
    format!(
        "{{\"schema\":1,\"route\":\"pinned-FMR1-USB-C\",\"reader_sha256\":{},\"stock_serial_label\":{},\"device\":{},\"location\":{},\"binding\":{},\"capacity\":1048576,\"capacity_basis\":\"exact-JEDEC-856014-profile\",\"passes\":2,\"bytes_per_pass\":1048576,\"coverage\":\"[0,1048576)\",\"block_bytes\":256,\"completed_dma\":{},\"tail_samples\":2,\"sha256\":{},\"raw_ciphertext_preserved\":true,\"flash_write_commands\":false,\"status_register_write_commands\":false,\"stock_return\":\"pending-manual-cold-return-and-fresh-identity\"}}",
        quote(IMAGE_SHA256),
        quote(&options.serial),
        quote(&options.device),
        quote(&format!("{:08X}", options.location)),
        quote(binding),
        capture.completed_dma,
        quote(&capture.sha256)
    )
}

#[cfg(all(
    feature = "reader-qualification",
    not(any(test, feature = "offline-tests")),
    target_os = "macos"
))]
fn run_selected(options: Options, image: Image, pair: &mut PreparedPair) -> Result<(), String> {
    let selection = crate::fmr_native::Selection {
        device: options.device.clone(),
        location: options.location,
        serial: options.serial.clone(),
    };
    let mut stored_digest = None;
    let result = crate::scsi::macos::reader_backup(&selection, image, |capture, binding| {
        eprintln!("Both complete passes match. Syncing and reopening the two protected copies…");
        let stored = pair.store(
            &capture.first,
            &capture.second,
            &provenance(&options, capture, binding),
        )?;
        if stored.sha256 != capture.sha256 {
            return Err("storage digest differs from fresh capture".into());
        }
        stored_digest = Some(stored.sha256);
        Ok(())
    });
    let (capture, binding) = match result {
        Ok(result) => result,
        Err(error) => {
            pair.cancel();
            return Err(error);
        }
    };
    if stored_digest.as_ref() != Some(&capture.sha256) || capture.completed_dma != EXPECTED_DMA {
        pair.cancel();
        return Err("final capture is not bound to this storage attempt".into());
    }
    pair.accept(&provenance(&options, &capture, &binding))?;
    println!(
        "Two fresh complete 1 MiB passes and both reopened copies match SHA-256 {}.",
        capture.sha256
    );
    println!(
        "Copy A: {}\nCopy B: {}",
        options.copy_a.display(),
        options.copy_b.display()
    );
    println!(
        "Held-session continuity verified after storage. Manual cold return and a fresh Stock identity check are still required."
    );
    Ok(())
}

#[cfg(not(all(
    feature = "reader-qualification",
    not(any(test, feature = "offline-tests")),
    target_os = "macos"
)))]
fn run_selected(_options: Options, _image: Image, _pair: &mut PreparedPair) -> Result<(), String> {
    Err("full native backup activation requires a separately authorized macOS qualification task; this build has no active full-backup adapter".into())
}

#[cfg(all(
    feature = "reader-qualification",
    not(any(test, feature = "offline-tests")),
    target_os = "macos"
))]
pub(crate) fn physical_full_selected<F: FnOnce(&Capture, &str) -> Result<(), String>>(
    selection: crate::fmr_native::Selection,
    image: Image,
    started: std::time::Instant,
    cancellation_fd: Option<std::os::fd::RawFd>,
    persist: F,
) -> Result<(Capture, String), String> {
    use crate::fmr_full::Control;
    struct Live {
        started: std::time::Instant,
        cancellation_fd: Option<std::os::fd::RawFd>,
    }
    impl Control for Live {
        fn elapsed(&self) -> std::time::Duration {
            self.started.elapsed()
        }
        fn cancelled(&self) -> bool {
            self.cancellation_fd
                .is_some_and(crate::fmr_backup_pipe::disconnected)
        }
        fn progress(&mut self, pass: u8, bytes: u32) -> Result<(), String> {
            if bytes % (256 * 1024) == 0 {
                eprintln!("Raw backup pass {pass}/2: {}%", 100 * bytes / 1048576);
            }
            Ok(())
        }
    }
    fn check(
        started: std::time::Instant,
        fd: Option<std::os::fd::RawFd>,
        reading: bool,
    ) -> Result<(), String> {
        if fd.is_some_and(crate::fmr_backup_pipe::disconnected) {
            return Err("FMR1 backup client disconnected or cancelled".into());
        }
        let limit = if reading {
            crate::fmr_full::READ_BUDGET
        } else {
            crate::fmr_full::TOTAL_BUDGET
        };
        if started.elapsed() >= limit {
            return Err("FMR1 backup job deadline expired".into());
        }
        Ok(())
    }
    check(started, cancellation_fd, true)?;
    let native = crate::fmr_native::open_selected(&selection)?;
    check(started, cancellation_fd, true)?;
    let binding = format!("{:?}", native.binding());
    let reading = std::rc::Rc::new(std::cell::Cell::new(true));
    let endpoint_reading = reading.clone();
    let guarded = crate::fmr_job_guard::Guarded::new(native, move || {
        check(started, cancellation_fd, endpoint_reading.get())
    });
    let mut bot = crate::fmr_bot::Bot::new(guarded, std::time::Duration::from_secs(3))?;
    crate::fmr_bootstrap::start(&mut bot, image)?;
    check(started, cancellation_fd, true)?;
    let capture = crate::fmr_full::capture_full(
        bot,
        Live {
            started,
            cancellation_fd,
        },
        |capture| {
            reading.set(false);
            persist(capture, &binding)
        },
    )?;
    Ok((capture, binding))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn help_is_file_and_device_inert() {
        assert_eq!(run(&["--help".into()]), 0);
    }
    #[test]
    fn provenance_keeps_capture_separate_from_stock_return() {
        let options = Options {
            reader: "/local/reader.bin".into(),
            device: "usb:1-1".into(),
            location: 0x01010000,
            serial: "0123456789ABCDEF".into(),
            copy_a: "/local/a".into(),
            copy_b: "/local/b".into(),
            allow_same_filesystem: true,
        };
        let capture = Capture {
            first: vec![],
            second: vec![],
            sha256: "fixture".into(),
            completed_dma: 8194,
        };
        let json = provenance(&options, &capture, "local \"binding\"");
        assert!(json.contains("pending-manual-cold-return"));
        assert!(json.contains("\\\"binding\\\""));
        assert!(!json.contains("stock_return\":true"));
    }
}
