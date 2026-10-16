//! File-only integration of the actual bounded helper exchange and capture core.

use crate::fmr_backup_exchange::{publish, receive};
use crate::fmr_backup_ipc as ipc;
use crate::fmr_backup_pipe::Pipe;
use crate::fmr_full::{Capture, Control, capture_full};
use crate::fmr_reader::Transport;
use crate::fmr_storage::{Policy, PreparedPair};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::net::Shutdown;
use std::os::fd::OwnedFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const BEFORE_FINAL: usize = 8263;
const RAW_LEN: usize = 1_048_576;
const DMA: usize = 8194;

// Production IPC has a long qualification budget. A fixture watchdog closes
// both local endpoints if a regression forgets a frame/ACK. Backdating the
// public pipe start also limits each production poll deadline to five seconds.
// Successful tests stop the watchdog immediately.
struct Watchdog(
    mpsc::Sender<()>,
    Option<JoinHandle<()>>,
    UnixStream,
    UnixStream,
);
impl Watchdog {
    fn close_client(&self) {
        self.2.shutdown(Shutdown::Both).unwrap();
    }
    fn close_server(&self) {
        self.3.shutdown(Shutdown::Both).unwrap();
    }
}
impl Drop for Watchdog {
    fn drop(&mut self) {
        let _ = self.0.send(());
        self.1.take().unwrap().join().unwrap();
    }
}
fn pipes() -> (Pipe, Pipe, Watchdog) {
    let (a, b) = UnixStream::pair().unwrap();
    let (stop, stopped) = mpsc::channel();
    let shutdown_a = a.try_clone().unwrap();
    let shutdown_b = b.try_clone().unwrap();
    let close_a = a.try_clone().unwrap();
    let close_b = b.try_clone().unwrap();
    let watchdog = thread::spawn(move || {
        if stopped.recv_timeout(Duration::from_secs(10)).is_err() {
            let _ = shutdown_a.shutdown(Shutdown::Both);
            let _ = shutdown_b.shutdown(Shutdown::Both);
        }
    });
    let mut client = Pipe::duplex(File::from(OwnedFd::from(a))).unwrap();
    let mut server = Pipe::duplex(File::from(OwnedFd::from(b))).unwrap();
    client.started = Instant::now() - Duration::from_secs(895);
    server.started = client.started;
    (
        client,
        server,
        Watchdog(stop, Some(watchdog), close_a, close_b),
    )
}

#[derive(Default)]
struct Trace {
    commands: Vec<u16>,
    reads: usize,
    publishing: bool,
    acknowledged: bool,
    dropped: bool,
    fail_final: Option<u16>,
}
type Shared = Arc<Mutex<Trace>>;
struct Model(Shared);
impl Drop for Model {
    fn drop(&mut self) {
        self.0.lock().unwrap().dropped = true;
    }
}
fn raw_byte(address: usize) -> u8 {
    ((address.wrapping_mul(37) ^ (address >> 8) ^ (address >> 16)) & 255) as u8
}
impl Transport for Model {
    fn transaction(&mut self, cdb: &[u8; 16], length: usize) -> Result<Vec<u8>, String> {
        let mut trace = self.0.lock().unwrap();
        let opcode = u16::from_be_bytes([cdb[0], cdb[1]]);
        trace.commands.push(opcode);
        if trace.publishing {
            assert!(trace.acknowledged, "device command before storage ACK");
        }
        if trace.acknowledged && trace.fail_final == Some(opcode) {
            return Err("injected final checkpoint failure".into());
        }
        let mut canonical = [0u8; 16];
        canonical[..2].copy_from_slice(&cdb[..2]);
        let reply = match opcode {
            0xfc40 => {
                assert_eq!(length, 16);
                let mut reply = [
                    0xfc, 0x40, b'F', b'M', b'R', b'1', 1, 0, 0x38, 0x30, 0xfc, 0x94, 0, 0, 0, 0,
                ];
                reply[12..].copy_from_slice(&(trace.reads as u32).to_be_bytes());
                reply.to_vec()
            }
            0xfc0b => {
                assert_eq!(length, 16);
                vec![
                    0xfc, 0x0b, 0x85, 0x60, 0x14, 0, 0, 0x10, 0, 0, 1, 0, 0, 0, 0, 0,
                ]
            }
            0xfd05 => {
                assert_eq!(length, 256);
                assert!(trace.reads < DMA);
                let address = if trace.reads < 2 {
                    RAW_LEN - 256
                } else {
                    ((trace.reads - 2) % 4096) * 256
                };
                canonical[2..6].copy_from_slice(&(address as u32).to_be_bytes());
                canonical[6..8].copy_from_slice(&[1, 0]);
                trace.reads += 1;
                (address..address + 256).map(raw_byte).collect()
            }
            _ => panic!("forbidden fake transport command {opcode:04x}"),
        };
        assert_eq!(*cdb, canonical);
        Ok(reply)
    }
}
struct Clock;
impl Control for Clock {
    fn elapsed(&self) -> Duration {
        Duration::ZERO
    }
    fn cancelled(&self) -> bool {
        false
    }
    fn progress(&mut self, _: u8, _: u32) -> Result<(), String> {
        Ok(())
    }
}

fn serve(mut pipe: Pipe, state: Shared) -> JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        let result = capture_full(Model(state.clone()), Clock, |capture| {
            {
                let mut trace = state.lock().unwrap();
                assert_eq!(trace.commands.len(), BEFORE_FINAL);
                assert_eq!(trace.reads, DMA);
                assert!(!trace.dropped);
                trace.publishing = true;
            }
            publish(&mut pipe, capture, "saved-offline-binding")?;
            state.lock().unwrap().acknowledged = true;
            Ok(())
        });
        match result {
            Ok(capture) => pipe.write_frame(
                ipc::FINISHED,
                &Sha256::digest(&capture.first),
                ipc::MAX_BINDING_LEN,
            ),
            Err(error) => {
                // A disconnected client cannot receive an error. Only final
                // checkpoint failures after ACK have a live receiving client.
                if state.lock().unwrap().acknowledged {
                    let _ = pipe.write_frame(ipc::FAILED, error.as_bytes(), ipc::MAX_BINDING_LEN);
                }
                Err(error)
            }
        }
    })
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "fmr-exchange-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self(path.canonicalize().unwrap())
    }
    fn pair(&self) -> PreparedPair {
        PreparedPair::prepare(
            &self.0.join("a"),
            &self.0.join("b"),
            Policy::SeparateDirectories,
        )
        .unwrap()
    }
    fn no_acceptance(&self) {
        for name in ["a", "b"] {
            assert!(!self.0.join(name).join("capture-receipt.json").exists());
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn full_capture_retains_owner_through_real_reopened_storage_then_final_checkpoints_and_acceptance()
{
    let fixture = Fixture::new();
    let mut pair = fixture.pair();
    let state = Shared::default();
    let (mut client, server, _watchdog) = pipes();
    let worker = serve(server, state.clone());
    let (capture, binding) = receive(&mut client, |capture, binding| {
        assert_eq!(binding, "saved-offline-binding");
        {
            let trace = state.lock().unwrap();
            assert!(!trace.dropped);
            assert!(!trace.acknowledged);
            assert_eq!(trace.commands.len(), BEFORE_FINAL);
        }
        let copies = pair.store(&capture.first, &capture.second, "offline acquisition")?;
        assert_eq!(
            copies.sha256,
            "06b668491c0fc6fa31ddd03be8c71feb34b7faf533b9b4133b9ec8a243c3103c"
        );
        for path in [&copies.first_path, &copies.second_path] {
            assert_eq!(
                std::fs::read(path).unwrap(),
                (0..RAW_LEN).map(raw_byte).collect::<Vec<_>>()
            );
        }
        fixture.no_acceptance();
        assert!(!state.lock().unwrap().dropped);
        Ok(())
    })
    .unwrap();
    assert_eq!(binding, "saved-offline-binding");
    assert_eq!(capture.completed_dma, 8194);
    worker.join().unwrap().unwrap();
    let trace = state.lock().unwrap();
    assert!(trace.dropped);
    assert!(trace.acknowledged);
    assert_eq!(trace.commands.len(), BEFORE_FINAL + 2);
    assert_eq!(&trace.commands[BEFORE_FINAL..], &[0xfc40, 0xfc0b]);
    fixture.no_acceptance();
    pair.accept("offline final held-session checkpoint")
        .unwrap();
    for name in ["a", "b"] {
        let receipt =
            std::fs::read_to_string(fixture.0.join(name).join("capture-receipt.json")).unwrap();
        assert!(receipt.contains("final-held-session-checkpoint"));
        assert!(receipt.contains("\"returned_to_stock\":false"));
    }
}

#[test]
fn failed_real_storage_sends_no_ack_and_no_final_device_commands_or_acceptance() {
    let fixture = Fixture::new();
    let mut pair = fixture.pair();
    // Foreign output is preserved; failure occurs after acquired bytes arrive.
    std::fs::write(fixture.0.join("b/storage-only.json"), b"foreign").unwrap();
    let state = Shared::default();
    let (mut client, server, watchdog) = pipes();
    let worker = serve(server, state.clone());
    assert!(
        receive(&mut client, |capture, _| pair
            .store(&capture.first, &capture.second, "fixture")
            .map(|_| ()))
        .is_err()
    );
    watchdog.close_client();
    drop(client); // Real caller unwinds and closes the helper channel.
    assert!(worker.join().unwrap().is_err());
    let trace = state.lock().unwrap();
    assert!(!trace.acknowledged);
    assert!(trace.dropped);
    assert_eq!(trace.commands.len(), BEFORE_FINAL);
    assert_eq!(
        std::fs::read(fixture.0.join("b/storage-only.json")).unwrap(),
        b"foreign"
    );
    assert!(pair.accept("must fail").is_err());
    fixture.no_acceptance();
}

#[test]
fn closing_client_after_capture_without_ack_drops_owner_without_final_commands() {
    let state = Shared::default();
    let (mut client, server, watchdog) = pipes();
    let worker = serve(server, state.clone());
    let (status, bytes) = client.read_frame(ipc::MAX_CAPTURED_LEN).unwrap();
    assert_eq!(status, ipc::CAPTURED);
    assert_eq!(ipc::decode_captured(&bytes).unwrap().first.len(), RAW_LEN);
    assert!(!state.lock().unwrap().dropped);
    watchdog.close_client();
    drop(client);
    assert!(worker.join().unwrap().is_err());
    let trace = state.lock().unwrap();
    assert!(trace.dropped);
    assert!(!trace.acknowledged);
    assert_eq!(trace.commands.len(), BEFORE_FINAL);
}

#[test]
fn final_identity_or_chip_failure_preserves_storage_only_and_cannot_accept() {
    for opcode in [0xfc40, 0xfc0b] {
        let fixture = Fixture::new();
        let mut pair = fixture.pair();
        let state = Shared::default();
        state.lock().unwrap().fail_final = Some(opcode);
        let (mut client, server, _watchdog) = pipes();
        let worker = serve(server, state.clone());
        let result = receive(&mut client, |capture, _| {
            pair.store(&capture.first, &capture.second, "fixture")
                .map(|_| ())
        });
        assert!(result.unwrap_err().contains("final checkpoint failure"));
        pair.cancel(); // Production caller cancels when receive lacks FINISHED.
        assert!(worker.join().unwrap().is_err());
        assert!(pair.accept("must fail").is_err());
        fixture.no_acceptance();
        let trace = state.lock().unwrap();
        assert!(trace.dropped);
        assert!(trace.acknowledged);
        assert_eq!(
            trace.commands.len(),
            BEFORE_FINAL + if opcode == 0xfc40 { 1 } else { 2 }
        );
    }
}

fn synthetic() -> Capture {
    let first: Vec<_> = (0..RAW_LEN).map(raw_byte).collect();
    Capture {
        second: first.clone(),
        sha256: String::new(),
        first,
        completed_dma: 8194,
    }
}

#[test]
fn foreign_ack_is_rejected_and_foreign_finished_digest_cannot_accept() {
    let (mut client, mut server, _watchdog) = pipes();
    let worker = thread::spawn(move || publish(&mut server, &synthetic(), "fixture"));
    assert_eq!(
        client.read_frame(ipc::MAX_CAPTURED_LEN).unwrap().0,
        ipc::CAPTURED
    );
    client.write_all(&ipc::encode_ack(&[0; 32])).unwrap();
    assert!(
        worker
            .join()
            .unwrap()
            .unwrap_err()
            .contains("foreign digest")
    );

    let fixture = Fixture::new();
    let mut pair = fixture.pair();
    let (mut client, mut server, _watchdog) = pipes();
    let worker = thread::spawn(move || {
        publish(&mut server, &synthetic(), "fixture").unwrap();
        server
            .write_frame(ipc::FINISHED, &[0; 32], ipc::MAX_BINDING_LEN)
            .unwrap();
    });
    assert!(
        receive(&mut client, |capture, _| pair
            .store(&capture.first, &capture.second, "fixture")
            .map(|_| ()))
        .unwrap_err()
        .contains("does not match stored capture")
    );
    pair.cancel();
    worker.join().unwrap();
    assert!(pair.accept("must fail").is_err());
    fixture.no_acceptance();
}

#[test]
fn closed_server_and_wrong_phase_never_invoke_persistence() {
    let (mut client, server, watchdog) = pipes();
    watchdog.close_server();
    drop(server);
    assert!(receive(&mut client, |_, _| panic!("closed peer reached storage")).is_err());
    let (mut client, mut server, _watchdog) = pipes();
    server
        .write_frame(ipc::FINISHED, &[0; 32], ipc::MAX_BINDING_LEN)
        .unwrap();
    assert!(
        receive(&mut client, |_, _| panic!("wrong phase reached storage"))
            .unwrap_err()
            .contains("expected captured phase")
    );
}
