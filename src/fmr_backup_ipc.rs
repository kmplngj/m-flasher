//! Bounded full-capture frames for the dedicated reader helper. Pure codecs:
//! no file/device access, and no generic privileged commands.

pub use crate::fmr_ipc::Request;
pub const RAW_BACKUP: u8 = 6;
pub const REQUEST_LEN: usize = crate::fmr_ipc::REQUEST_LEN;
pub const IMAGE_LEN: usize = crate::fmr_ipc::IMAGE_LEN;
pub const MAX_BINDING_LEN: usize = 8192;
pub const RAW_LEN: usize = 1024 * 1024;
pub const MAX_CAPTURED_LEN: usize = 2 + MAX_BINDING_LEN + 2 * RAW_LEN;
pub const CAPTURED: u8 = 10;
pub const FINISHED: u8 = 11;
pub const FAILED: u8 = 1;
pub const ACK: u8 = 0xa6;
pub const ACK_LEN: usize = 33;

#[derive(Debug, PartialEq, Eq)]
pub struct Captured {
    pub first: Vec<u8>,
    pub second: Vec<u8>,
    pub binding: String,
}

pub fn encode_request(request: &Request) -> Result<Vec<u8>, String> {
    let mut bytes = crate::fmr_ipc::encode_request(request)?;
    bytes[0] = RAW_BACKUP;
    Ok(bytes)
}

pub fn decode_request(bytes: &[u8]) -> Result<Request, String> {
    if bytes.len() != REQUEST_LEN || bytes[0] != RAW_BACKUP {
        return Err("full backup IPC requires exactly one fixed raw-backup frame".into());
    }
    let mut frame = bytes.to_vec();
    frame[0] = crate::fmr_ipc::FMR_CHECK;
    crate::fmr_ipc::decode_request(&frame)
}

pub fn encode_captured(captured: &Captured) -> Result<Vec<u8>, String> {
    if captured.binding.len() > MAX_BINDING_LEN {
        return Err("full backup IPC binding exceeds the response bound".into());
    }
    if captured.first.len() != RAW_LEN || captured.second.len() != RAW_LEN {
        return Err("full backup IPC requires two complete 1 MiB passes".into());
    }
    if captured.first != captured.second {
        return Err("full backup IPC raw passes differ".into());
    }
    let mut bytes = Vec::with_capacity(2 + captured.binding.len() + 2 * RAW_LEN);
    bytes.extend_from_slice(&(captured.binding.len() as u16).to_le_bytes());
    bytes.extend_from_slice(captured.binding.as_bytes());
    bytes.extend_from_slice(&captured.first);
    bytes.extend_from_slice(&captured.second);
    Ok(bytes)
}

pub fn decode_captured(bytes: &[u8]) -> Result<Captured, String> {
    let prefix = bytes
        .get(..2)
        .ok_or("full backup IPC truncated captured length")?;
    let binding_len = u16::from_le_bytes(prefix.try_into().unwrap()) as usize;
    if binding_len > MAX_BINDING_LEN || bytes.len() != 2 + binding_len + 2 * RAW_LEN {
        return Err("full backup IPC captured length mismatch".into());
    }
    let binding = std::str::from_utf8(&bytes[2..2 + binding_len])
        .map_err(|_| "full backup IPC binding is not UTF-8")?;
    let first = &bytes[2 + binding_len..2 + binding_len + RAW_LEN];
    let second = &bytes[2 + binding_len + RAW_LEN..];
    // Validate the full frame and equality before allocating acquired data.
    if first != second {
        return Err("full backup IPC raw passes differ".into());
    }
    Ok(Captured {
        first: first.to_vec(),
        second: second.to_vec(),
        binding: binding.into(),
    })
}

/// A storage acknowledgement binds only the exact acquired raw SHA-256.
/// The helper's FINISHED payload echoes this digest after final acceptance.
pub fn encode_ack(digest: &[u8; 32]) -> [u8; ACK_LEN] {
    let mut bytes = [0u8; ACK_LEN];
    bytes[0] = ACK;
    bytes[1..].copy_from_slice(digest);
    bytes
}

pub fn decode_ack(bytes: &[u8]) -> Result<[u8; 32], String> {
    if bytes.len() != ACK_LEN || bytes[0] != ACK {
        return Err("full backup IPC requires one exact storage acknowledgement".into());
    }
    Ok(bytes[1..].try_into().unwrap())
}
