//! Exact, one-use RAM upload for the reproduced FMR1 image. No flash commands.

use crate::fmr_bot::{Bot, Bulk};
use crate::formats::crc16;
use crate::jl::crc_cipher;
use sha2::{Digest, Sha256};

pub const IMAGE_SHA256: &str = "63c12a9ded28b53fa991c8261cbc8065ef93f8e71a805f53553999175422618a";
pub const IMAGE_LEN: usize = 1456;
pub const LOAD_ADDRESS: u32 = 0x01C2_0000;

pub struct Image(Vec<u8>);

impl Image {
    #[cfg(all(
        feature = "reader-qualification",
        target_os = "macos",
        not(any(test, feature = "offline-tests"))
    ))]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn verify(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() != IMAGE_LEN
            || Sha256::digest(&bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
                != IMAGE_SHA256
        {
            return Err(
                "reader image does not match the reproduced 1456-byte FMR1 artifact".into(),
            );
        }
        Ok(Self(bytes))
    }
}

/// Consumes the checked image; three fixed RAM writes and one invocation only.
/// A failed or ambiguous command is never retried or followed by another image.
pub fn start<B: Bulk>(bot: &mut Bot<B>, image: Image) -> Result<(), String> {
    bot.claim_bootstrap()?;
    let result = (|| {
        for (index, plain) in image.0.chunks(512).enumerate() {
            let data = crc_cipher(plain, u32::MAX);
            let mut cdb = [0xff; 16];
            cdb[..2].copy_from_slice(&0xFB06u16.to_be_bytes());
            cdb[2..6].copy_from_slice(&(LOAD_ADDRESS + index as u32 * 512).to_be_bytes());
            cdb[6..8].copy_from_slice(&(data.len() as u16).to_be_bytes());
            cdb[8] = 0;
            cdb[9..11].copy_from_slice(&crc16(&data).to_le_bytes());
            bot.write(&cdb, &data).map_err(|error| format!("RAM upload block {index} at {:08X}: {error}", LOAD_ADDRESS + index as u32 * 512))?;
        }
        let jump = [
            0xFB, 0x08, 0x01, 0xC2, 0, 0, 0, 1, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        let reply = bot.read(&jump, 16).map_err(|error| format!("RAM entry FB08: {error}"))?;
        if reply[..2] != [0xFB, 0x08] {
            return Err("RAM entry reply does not echo FB08; stop without fallback".into());
        }
        Ok(())
    })();
    if result.is_err() {
        bot.poison();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrong_image_is_rejected_before_transport_exists() {
        for length in [0, 1455, 1456, 1457, 32768] {
            assert!(Image::verify(vec![0; length]).is_err());
        }
    }
}

#[cfg(test)]
#[path = "../tests/support/fmr_bootstrap.rs"]
mod qualification_tests;
