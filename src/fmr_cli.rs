//! Separate experimental entrypoint: never constructs the legacy flash session.

use crate::fmr_bootstrap::{self, IMAGE_LEN, IMAGE_SHA256, Image};
use crate::fmr_options::{self, Command, Options};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub const USAGE: &str = "m-flasher reader-check (--prepare | --run) --reader IMAGE.bin \
    --device usb:BUS-ADDRESS --location 03110000 --serial STOCK_SERIAL --output NEW_DIRECTORY\n\
Exactly two fresh 256-byte reads at physical address 0.\n\
--prepare validates files and arguments without any device access.\n\
--run needs the reader-qualification build feature and an explicitly selected fresh ROM session.\n\
No flash/status writes, automatic reset/retry, loader fallback or software return command.\n\
Stock return requires actual manual cold power-off/on and a fresh identity observation.";

pub fn run(args: &[String]) -> i32 {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        println!("{USAGE}");
        return 0;
    }
    match execute(args) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("Reader check stopped: {error}");
            1
        }
    }
}

fn execute(args: &[String]) -> Result<(), String> {
    let command = fmr_options::parse(args)?;
    let (mut options, run) = match command {
        Command::Prepare(options) => (options, false),
        Command::Run(options) => (options, true),
    };
    options.output = resolve_output(
        &options.output,
        &std::env::current_dir().map_err(|e| format!("output working directory: {e}"))?,
    )?;
    let image = load_image(&options.reader)?;
    crate::fmr_storage::PrivateOutputDirectory::preflight(&options.output)?;
    if !run {
        println!(
            "Prepared exact FMR1 image {IMAGE_SHA256}; selected {}, location {:08X}. No devices accessed.",
            options.device, options.location
        );
        return Ok(());
    }
    run_selected(options, image)
}

fn resolve_output(path: &Path, working_directory: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        working_directory.join(path)
    };
    let mut resolved = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::ParentDir => {
                return Err("output path cannot contain parent components".into());
            }
            Component::CurDir => {}
            component => resolved.push(component.as_os_str()),
        }
    }
    // No canonicalization: a symlink ancestor remains visible to the secure
    // storage walk, which applies the same no-follow policy in both modes.
    Ok(resolved)
}

pub(crate) fn load_image(path: &Path) -> Result<Image, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| format!("reader image: {e}"))?;
    if !metadata.is_file() {
        return Err("reader must be a regular file, not a symlink or special file".into());
    }
    let mut open = OpenOptions::new();
    open.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        open.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = open.open(path).map_err(|e| format!("reader image: {e}"))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("reader must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take((IMAGE_LEN + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    Image::verify(bytes)
}

#[cfg(all(
    feature = "reader-qualification",
    not(any(test, feature = "offline-tests")),
    any(target_os = "macos", all(target_os = "linux", feature = "usb"))
))]
fn run_selected(options: Options, image: Image) -> Result<(), String> {
    use crate::fmr_native::Selection;

    let mut output = crate::fmr_storage::PrivateOutputDirectory::prepare(&options.output)?;
    let base = format!(
        "\"schema\":1,\"kind\":\"bounded-reader-check\",\"reader_sha256\":{},\"stock_serial_label\":{},\"device\":{},\"location\":{},\"stock_return\":\"pending-manual-cold-return-and-fresh-identity\",\"full_backup\":false",
        quote(IMAGE_SHA256),
        quote(&options.serial),
        quote(&options.device),
        quote(&format!("{:08X}", options.location))
    );
    output.write_private(
        "started.json",
        format!("{{{base},\"state\":\"starting\"}}\n").as_bytes(),
    )?;
    // Persist and revalidate private identifiers before native device access.
    output.check()?;
    let result: Result<(String, String), String> = (|| {
        let selection = Selection {
            device: options.device.clone(),
            location: options.location,
            serial: options.serial.clone(),
        };
        #[cfg(target_os = "macos")]
        let read = crate::scsi::macos::reader_check(&selection, image)?;
        #[cfg(not(target_os = "macos"))]
        let read = physical_selected(selection, image)?;
        let binding = read.binding;
        let digest: String = Sha256::digest(read.first)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        output.write_private("first-256.bin", &read.first)?;
        output.write_private("second-256.bin", &read.second)?;
        Ok((digest, binding))
    })();
    let (state, extra) = match &result {
        Ok((digest, binding)) => (
            "two-equal-samples",
            format!(
                ",\"bytes_per_sample\":256,\"physical_address\":0,\"sample_sha256\":{},\"completed_dma\":2,\"binding\":{}",
                quote(digest),
                quote(binding)
            ),
        ),
        Err(error) => ("stopped", format!(",\"error\":{}", quote(error))),
    };
    output.write_private(
        "result.json",
        format!("{{{base},\"state\":{}{extra}}}\n", quote(state)).as_bytes(),
    )?;
    result?;
    println!(
        "Two fresh 256-byte samples match. Receipt: {}",
        options.output.join("result.json").display()
    );
    println!(
        "Stock return is pending: switch the FM-1 fully off and on, then verify fresh Stock identity. This is not a full backup."
    );
    Ok(())
}

#[cfg(all(
    feature = "reader-qualification",
    not(any(test, feature = "offline-tests")),
    any(target_os = "macos", all(target_os = "linux", feature = "usb"))
))]
pub(crate) struct HardwareResult {
    pub first: [u8; 256],
    pub second: [u8; 256],
    pub binding: String,
}

#[cfg(all(
    feature = "reader-qualification",
    not(any(test, feature = "offline-tests")),
    any(target_os = "macos", all(target_os = "linux", feature = "usb"))
))]
pub(crate) fn physical_selected(
    selection: crate::fmr_native::Selection,
    image: Image,
) -> Result<HardwareResult, String> {
    let native = crate::fmr_native::open_selected(&selection)?;
    let binding = format!("{:?}", native.binding());
    let mut bot = crate::fmr_bot::Bot::new(native, std::time::Duration::from_secs(3))?;
    fmr_bootstrap::start(&mut bot, image)?;
    let mut reader = crate::fmr_reader::Reader::new(bot, crate::fmr_reader::BUILD_ID);
    let read = reader.read_twice().map_err(|e| e.to_string())?;
    // All transactions completed before the retained native graph is released.
    drop(reader);
    Ok(HardwareResult {
        first: read.first,
        second: read.second,
        binding,
    })
}

#[cfg(not(all(
    feature = "reader-qualification",
    not(any(test, feature = "offline-tests")),
    any(target_os = "macos", all(target_os = "linux", feature = "usb"))
)))]
fn run_selected(_options: Options, _image: Image) -> Result<(), String> {
    Err("native reader is unavailable in this build; tests/offline-tests never contain a real reader adapter".into())
}

pub(crate) fn quote(value: &str) -> String {
    let mut quoted = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            character if character <= '\u{1f}' => {
                quoted.push_str(&format!("\\u{:04x}", character as u32))
            }
            character => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn help_does_not_require_reader_files() {
        assert_eq!(run(&["--help".into()]), 0);
    }

    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "fmr-cli-{}-{nonce}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            // Plain file-preflight fixtures also run on platforms where
            // protected native output is intentionally unsupported.
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn refused_without_output(&self, reader: &Path, expected_error: &str) {
            for mode in ["--prepare", "--run"] {
                let output = self.0.join("must-not-be-created");
                let args = [
                    mode.to_owned(),
                    "--reader".into(),
                    reader.to_str().unwrap().into(),
                    "--device".into(),
                    "usb:3-7".into(),
                    "--location".into(),
                    "03110000".into(),
                    "--serial".into(),
                    "0123456789ABCDEF".into(),
                    "--output".into(),
                    output.to_str().unwrap().into(),
                ];
                let error = execute(&args).unwrap_err();
                assert!(error.contains(expected_error), "{mode}: {error}");
                assert!(!output.exists(), "failed preflight created output");
            }
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn missing_directory_and_oversized_image_fail_file_preflight() {
        let scratch = Scratch::new();
        scratch.refused_without_output(&scratch.0.join("missing.bin"), "reader image:");
        scratch.refused_without_output(&scratch.0, "regular file");
        let oversized = scratch.0.join("oversized.bin");
        File::create(&oversized)
            .unwrap()
            .set_len(1024 * 1024 * 1024)
            .unwrap();
        scratch.refused_without_output(&oversized, "reproduced 1456-byte FMR1 artifact");
        // These specific preflight errors precede the unavailable-native stub;
        // they do not claim to exercise or prove the real native factory.
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_and_unopened_fifo_fail_regular_file_preflight() {
        use std::os::unix::fs::symlink;
        let scratch = Scratch::new();
        let target = scratch.0.join("target.bin");
        fs::write(&target, b"local fixture").unwrap();
        let link = scratch.0.join("link.bin");
        symlink(&target, &link).unwrap();
        scratch.refused_without_output(&link, "not a symlink or special file");
        let fifo = scratch.0.join("fixture.fifo");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        // No FIFO writer is opened. Opening this FIFO for blocking read would
        // hang, so successful rejection also checks the pre-open boundary.
        scratch.refused_without_output(&fifo, "not a symlink or special file");
    }

    #[test]
    fn receipt_strings_escape_quotes_and_controls() {
        assert_eq!(quote("a\n\"\\"), "\"a\\u000a\\\"\\\\\"");
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn documented_relative_output_passes_preflight_and_protected_creation_without_native() {
        let scratch = Scratch::new();
        // Canonicalize only the synthetic fixture's base, never user output.
        let base = scratch.0.canonicalize().unwrap();
        fs::create_dir(base.join("target")).unwrap();
        let expected = base.join("target/private-read-result");
        let resolved = resolve_output(Path::new("./target/private-read-result"), &base).unwrap();
        assert_eq!(resolved, expected);
        assert_eq!(resolve_output(&expected, &base).unwrap(), expected);
        crate::fmr_storage::PrivateOutputDirectory::preflight(&resolved).unwrap();
        assert!(!resolved.exists(), "preflight must not create output");
        let mut output = crate::fmr_storage::PrivateOutputDirectory::prepare(&resolved).unwrap();
        output
            .write_private("started.json", b"private fixture")
            .unwrap();
        output.check().unwrap();
        assert_eq!(
            fs::read(expected.join("started.json")).unwrap(),
            b"private fixture"
        );
    }

    #[test]
    fn parent_output_components_are_refused_in_both_modes_before_file_loading() {
        for mode in ["--prepare", "--run"] {
            for output in ["target/../private-read-result", "../private-read-result"] {
                let args: Vec<String> = [
                    mode,
                    "--reader",
                    "missing-fixture-reader.bin",
                    "--device",
                    "usb:3-7",
                    "--location",
                    "03110000",
                    "--serial",
                    "0123456789ABCDEF",
                    "--output",
                    output,
                ]
                .into_iter()
                .map(String::from)
                .collect();
                let error = execute(&args).unwrap_err();
                assert!(
                    error.contains("output path cannot contain parent components"),
                    "{mode}: {error}"
                );
            }
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn shared_output_preflight_refuses_symlink_ancestors_without_creating_output() {
        use std::os::unix::fs::symlink;
        let scratch = Scratch::new();
        let base = scratch.0.canonicalize().unwrap();
        fs::create_dir(base.join("real")).unwrap();
        symlink(base.join("real"), base.join("alias")).unwrap();
        let resolved = resolve_output(Path::new("alias/private-read-result"), &base).unwrap();
        assert_eq!(resolved, base.join("alias/private-read-result"));
        assert!(crate::fmr_storage::PrivateOutputDirectory::preflight(&resolved).is_err());
        assert!(!base.join("real/private-read-result").exists());
    }
}
