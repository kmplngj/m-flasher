//! Two-phase acquisition/storage exchange over bounded local IPC. No devices.

use crate::fmr_backup_ipc as ipc;
use crate::fmr_backup_pipe::Pipe;
use crate::fmr_full::{Capture, EXPECTED_DMA};
use sha2::{Digest, Sha256};

fn digest(first: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&Sha256::digest(first));
    hash
}

pub fn receive<F: FnOnce(&Capture, &str) -> Result<(), String>>(
    pipe: &mut Pipe,
    persist: F,
) -> Result<(Capture, String), String> {
    let (status, payload) = pipe.read_frame(ipc::MAX_CAPTURED_LEN)?;
    if status == ipc::FAILED {
        return Err(String::from_utf8_lossy(&payload).into_owned());
    }
    if status != ipc::CAPTURED {
        return Err("FMR1 expected captured phase".into());
    }
    let acquired = ipc::decode_captured(&payload)?;
    let hash = digest(&acquired.first);
    let capture = Capture {
        first: acquired.first,
        second: acquired.second,
        sha256: hash.iter().map(|b| format!("{b:02x}")).collect(),
        completed_dma: EXPECTED_DMA,
    };
    persist(&capture, &acquired.binding)?;
    pipe.write_all(&ipc::encode_ack(&hash))?;
    let (status, payload) = pipe.read_frame(ipc::MAX_BINDING_LEN)?;
    if status == ipc::FAILED {
        return Err(String::from_utf8_lossy(&payload).into_owned());
    }
    if status != ipc::FINISHED || payload.as_slice() != hash {
        return Err("FMR1 final session receipt does not match stored capture".into());
    }
    Ok((capture, acquired.binding))
}

/// Called only inside the held-owner persist callback. Completion here is a
/// storage acknowledgement; the caller still owes final device checkpoints.
pub fn publish(pipe: &mut Pipe, capture: &Capture, binding: &str) -> Result<(), String> {
    let payload = ipc::encode_captured(&ipc::Captured {
        first: capture.first.clone(),
        second: capture.second.clone(),
        binding: binding.into(),
    })?;
    pipe.write_frame(ipc::CAPTURED, &payload, ipc::MAX_CAPTURED_LEN)?;
    let mut ack = [0u8; ipc::ACK_LEN];
    pipe.read_exact(&mut ack)?;
    if ipc::decode_ack(&ack)? != digest(&capture.first) {
        return Err("FMR1 storage acknowledgement has a foreign digest".into());
    }
    Ok(())
}
