//! Fixed-profile, two-pass raw capture over one retained, injected transport.
//!
//! No adapter, bootstrap, recovery or flash writer is available here. Successful
//! capture proves the checked transcript and equal returned bytes, not physical
//! freshness when an adapter lies, durable storage, or original-stock return.

use crate::fmr_reader::{self, Transport};
use sha2::{Digest, Sha256};
use std::time::Duration;

pub const READ_BUDGET: Duration = Duration::from_secs(10 * 60);
pub const TOTAL_BUDGET: Duration = Duration::from_secs(15 * 60);
pub const EXPECTED_DMA: u32 = 8194;
const CHECKPOINT: u32 = 64 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub struct Capture {
    pub first: Vec<u8>,
    pub second: Vec<u8>,
    pub sha256: String,
    pub completed_dma: u32,
}

/// The clock covers the enclosing job, including loader entry and persistence.
/// It must be monotonic; callbacks must themselves be bounded, since this
/// synchronous core cannot interrupt a blocked callback or transport call.
pub trait Control {
    fn elapsed(&self) -> Duration;
    fn cancelled(&self) -> bool;
    /// Bytes are verified checkpoint coverage within pass 1 or 2.
    fn progress(&mut self, pass: u8, bytes: u32) -> Result<(), String>;
}

struct Session<T, C> {
    transport: T,
    control: C,
    completed_dma: u32,
    last_elapsed: Duration,
}

impl<T: Transport, C: Control> Session<T, C> {
    fn check(&mut self, reading: bool) -> Result<(), String> {
        if self.control.cancelled() {
            return Err("FMR1 full capture cancelled; session is terminal".into());
        }
        let elapsed = self.control.elapsed();
        if elapsed < self.last_elapsed {
            return Err("FMR1 job clock moved backwards; session is terminal".into());
        }
        self.last_elapsed = elapsed;
        if elapsed >= TOTAL_BUDGET {
            return Err("FMR1 total job deadline expired".into());
        }
        if reading && elapsed >= READ_BUDGET {
            return Err("FMR1 read deadline expired".into());
        }
        Ok(())
    }

    fn command(&mut self, opcode: u16, address: u32, reading: bool) -> Result<Vec<u8>, String> {
        self.check(reading)?;
        let mut cdb = [0u8; 16];
        cdb[..2].copy_from_slice(&opcode.to_be_bytes());
        let expected = if opcode == fmr_reader::READ {
            // Only internal, aligned ranges from the fixed established profile.
            if address % fmr_reader::READ_SIZE as u32 != 0
                || address
                    .checked_add(fmr_reader::READ_SIZE as u32)
                    .is_none_or(|end| end > fmr_reader::CAPACITY)
            {
                return Err("FMR1 internal read range rejected".into());
            }
            cdb[2..6].copy_from_slice(&address.to_be_bytes());
            cdb[6..8].copy_from_slice(&(fmr_reader::READ_SIZE as u16).to_be_bytes());
            fmr_reader::READ_SIZE
        } else {
            fmr_reader::METADATA_SIZE
        };
        let response = self.transport.transaction(&cdb, expected);
        self.check(reading)?;
        let data = response.map_err(|reason| format!("FMR1 transport failed: {reason}"))?;
        if data.len() != expected {
            return Err(format!(
                "FMR1 command {opcode:04X}: expected {expected} bytes, received {}",
                data.len()
            ));
        }
        if opcode == fmr_reader::READ {
            self.completed_dma = self
                .completed_dma
                .checked_add(1)
                .ok_or("FMR1 DMA count overflow")?;
        }
        Ok(data)
    }

    fn identity(&mut self, reading: bool) -> Result<(), String> {
        let data = self.command(fmr_reader::IDENTITY, 0, reading)?;
        let mut expected = [0u8; 16];
        expected[..2].copy_from_slice(&fmr_reader::IDENTITY.to_be_bytes());
        expected[2..6].copy_from_slice(b"FMR1");
        expected[6] = fmr_reader::ABI;
        expected[8..12].copy_from_slice(&fmr_reader::BUILD_ID);
        expected[12..16].copy_from_slice(&self.completed_dma.to_be_bytes());
        if data != expected {
            return Err("FMR1 identity/profile/status/DMA checkpoint rejected".into());
        }
        Ok(())
    }

    fn chip(&mut self, reading: bool) -> Result<(), String> {
        let data = self.command(fmr_reader::JEDEC, 0, reading)?;
        let mut expected = [0u8; 16];
        expected[..2].copy_from_slice(&fmr_reader::JEDEC.to_be_bytes());
        expected[2..5].copy_from_slice(&fmr_reader::JEDEC_ID);
        expected[6..10].copy_from_slice(&fmr_reader::CAPACITY.to_be_bytes());
        expected[10..12].copy_from_slice(&(fmr_reader::READ_SIZE as u16).to_be_bytes());
        if data != expected {
            return Err("FMR1 chip/profile/status checkpoint rejected".into());
        }
        Ok(())
    }

    fn checkpoint(&mut self, reading: bool) -> Result<(), String> {
        self.identity(reading)?;
        self.chip(reading)
    }

    fn progress(&mut self, pass: u8, bytes: u32) -> Result<(), String> {
        self.check(true)?;
        let result = self.control.progress(pass, bytes);
        self.check(true)?;
        result.map_err(|reason| format!("FMR1 progress callback failed: {reason}"))
    }

    fn pass(&mut self, pass: u8) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::with_capacity(fmr_reader::CAPACITY as usize);
        self.progress(pass, 0)?;
        for address in (0..fmr_reader::CAPACITY).step_by(fmr_reader::READ_SIZE) {
            let data = self.command(fmr_reader::READ, address, true)?;
            bytes.extend_from_slice(&data);
            let covered = address + fmr_reader::READ_SIZE as u32;
            if covered % CHECKPOINT == 0 {
                self.checkpoint(true)?;
                self.progress(pass, covered)?;
            }
        }
        Ok(bytes)
    }
}

/// Consume the sole session owner. Any failure drops it without issuing cleanup,
/// retry, reset or keepalive traffic. `persist` runs only for two fresh complete
/// equal passes; it must finish within the total budget and use bounded I/O.
/// The owner remains held until final FC40/FC0B continuity checks after storage.
pub fn capture_full<T: Transport, C: Control, F: FnOnce(&Capture) -> Result<(), String>>(
    transport: T,
    control: C,
    persist: F,
) -> Result<Capture, String> {
    let mut session = Session {
        transport,
        control,
        completed_dma: 0,
        last_elapsed: Duration::ZERO,
    };
    session.checkpoint(true)?;
    // Close the selected end-of-flash small-read gate before full acquisition.
    let tail_address = fmr_reader::CAPACITY - fmr_reader::READ_SIZE as u32;
    let tail_first = session.command(fmr_reader::READ, tail_address, true)?;
    session.identity(true)?;
    let tail_second = session.command(fmr_reader::READ, tail_address, true)?;
    if tail_first != tail_second {
        return Err("FMR1 fresh tail samples differ".into());
    }
    session.checkpoint(true)?;
    let first = session.pass(1)?;
    let second = session.pass(2)?;
    session.check(true)?;
    let first_hash = Sha256::digest(&first);
    let second_hash = Sha256::digest(&second);
    if first.len() != fmr_reader::CAPACITY as usize
        || second.len() != first.len()
        || first_hash != second_hash
        || first != second
    {
        return Err("FMR1 fresh complete raw passes differ in length, SHA-256 or bytes".into());
    }
    if session.completed_dma != EXPECTED_DMA {
        return Err("FMR1 final DMA coverage rejected".into());
    }
    let capture = Capture {
        first,
        second,
        sha256: first_hash
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        completed_dma: session.completed_dma,
    };
    session.check(true)?;
    let stored = persist(&capture);
    session.check(false)?;
    stored.map_err(|reason| format!("FMR1 persistence failed: {reason}"))?;
    session.checkpoint(false)?;
    Ok(capture)
}
