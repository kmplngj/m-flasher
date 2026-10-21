//! Strict, one-shot host profile for the source-defined FMR1 reader.
//!
//! This module owns no device access, loader bootstrap, reset or writer. The
//! injected transport must validate complete BOT framing before returning data.
//! The fixed qualification consumes two fresh 256-byte commands at address zero;
//! it does not establish original-stock return or hardware qualification.

use std::fmt;

pub const IDENTITY: u16 = 0xFC40;
pub const JEDEC: u16 = 0xFC0B;
pub const READ: u16 = 0xFD05;
pub const ABI: u8 = 1;
pub const BUILD_ID: [u8; 4] = [0x38, 0x30, 0xFC, 0x94];
pub const JEDEC_ID: [u8; 3] = [0x85, 0x60, 0x14];
pub const CAPACITY: u32 = 0x0010_0000;
pub const READ_SIZE: usize = 256;
pub const METADATA_SIZE: usize = 16;

/// One complete command, with fresh data owned by the caller on success.
///
/// Implementations must send exactly one canonical 31-byte CBW (16-byte CDB,
/// IN direction, LUN zero), enforce a finite transaction deadline, and validate
/// exact CBW/data/13-byte CSW transfer counts, CSW signature, matching fresh tag,
/// zero residue and successful status. Errors must be terminal: no automatic
/// retry, reset, fallback, sense query or other implicit device command.
/// Returning a data buffer alone cannot prove these properties; they require
/// independent adapter tests and selected physical evidence.
pub trait Transport {
    fn transaction(&mut self, cdb: &[u8; 16], expected_len: usize) -> Result<Vec<u8>, String>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Terminal,
    Transport(String),
    UnexpectedLength {
        opcode: u16,
        expected: usize,
        actual: usize,
    },
    Identity(&'static str),
    Chip(&'static str),
    ReadMismatch {
        offset: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Terminal => write!(
                f,
                "FMR1 session is terminal; no further commands are permitted"
            ),
            Self::Transport(reason) => write!(f, "FMR1 transport failed: {reason}"),
            Self::UnexpectedLength {
                opcode,
                expected,
                actual,
            } => write!(
                f,
                "FMR1 command {opcode:04X}: expected {expected} bytes, received {actual}"
            ),
            Self::Identity(reason) => write!(f, "FMR1 identity rejected: {reason}"),
            Self::Chip(reason) => write!(f, "FMR1 chip metadata rejected: {reason}"),
            Self::ReadMismatch { offset } => {
                write!(f, "FMR1 fresh reads differ at offset {offset}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Two equal, independently requested blocks. This is local read evidence only;
/// stock return and persistence receipts belong to separate operations.
#[derive(Debug, PartialEq, Eq)]
pub struct Qualification {
    pub first: [u8; READ_SIZE],
    pub second: [u8; READ_SIZE],
    pub build_id: [u8; 4],
    pub jedec: [u8; 3],
    pub capacity: u32,
    pub completed_dma: u32,
}

pub struct Reader<T: Transport> {
    transport: T,
    expected_build_id: [u8; 4],
    terminal: bool,
}

impl<T: Transport> Reader<T> {
    /// The build ID must come from the exact previously verified upload image.
    /// For the recorded offline candidate that value is [`BUILD_ID`].
    /// Construction issues no commands and does not start a loader.
    pub fn new(transport: T, expected_build_id: [u8; 4]) -> Self {
        Self {
            transport,
            expected_build_id,
            terminal: false,
        }
    }

    /// Consume this session's only qualification attempt, including on success.
    /// Every failure leaves the session terminal before any caller can retry it.
    pub fn read_twice(&mut self) -> Result<Qualification, Error> {
        if self.terminal {
            return Err(Error::Terminal);
        }
        self.terminal = true;
        self.identity(0)?;
        self.chip()?;
        let first = self.read_zero()?;
        self.identity(1)?;
        let second = self.read_zero()?;
        if let Some(offset) = first.iter().zip(&second).position(|(a, b)| a != b) {
            return Err(Error::ReadMismatch { offset });
        }
        self.identity(2)?;
        Ok(Qualification {
            first,
            second,
            build_id: self.expected_build_id,
            jedec: JEDEC_ID,
            capacity: CAPACITY,
            completed_dma: 2,
        })
    }

    fn command(&mut self, opcode: u16, expected_len: usize) -> Result<Vec<u8>, Error> {
        let mut cdb = [0u8; 16];
        cdb[..2].copy_from_slice(&opcode.to_be_bytes());
        if opcode == READ {
            // Fixed physical address zero; no caller-controlled range or length.
            cdb[6..8].copy_from_slice(&(READ_SIZE as u16).to_be_bytes());
        }
        let data = self
            .transport
            .transaction(&cdb, expected_len)
            .map_err(Error::Transport)?;
        if data.len() != expected_len {
            return Err(Error::UnexpectedLength {
                opcode,
                expected: expected_len,
                actual: data.len(),
            });
        }
        Ok(data)
    }

    fn identity(&mut self, completed_dma: u32) -> Result<(), Error> {
        let data = self.command(IDENTITY, METADATA_SIZE)?;
        if data[..2] != IDENTITY.to_be_bytes() {
            return Err(Error::Identity("opcode"));
        }
        if &data[2..6] != b"FMR1" {
            return Err(Error::Identity("magic"));
        }
        if data[6] != ABI {
            return Err(Error::Identity("ABI"));
        }
        if data[7] != 0 {
            return Err(Error::Identity("reader status"));
        }
        if data[8..12] != self.expected_build_id {
            return Err(Error::Identity("build ID"));
        }
        if data[12..16] != completed_dma.to_be_bytes() {
            return Err(Error::Identity("completed DMA count"));
        }
        Ok(())
    }

    fn chip(&mut self) -> Result<(), Error> {
        let data = self.command(JEDEC, METADATA_SIZE)?;
        if data[..2] != JEDEC.to_be_bytes() {
            return Err(Error::Chip("opcode"));
        }
        if data[2..5] != JEDEC_ID {
            return Err(Error::Chip("JEDEC ID"));
        }
        if data[5] != 0 {
            return Err(Error::Chip("reader status"));
        }
        if data[6..10] != CAPACITY.to_be_bytes() {
            return Err(Error::Chip("capacity"));
        }
        if data[10..12] != (READ_SIZE as u16).to_be_bytes() {
            return Err(Error::Chip("read size"));
        }
        if data[12..16] != [0; 4] {
            return Err(Error::Chip("reserved bytes"));
        }
        Ok(())
    }

    fn read_zero(&mut self) -> Result<[u8; READ_SIZE], Error> {
        let data = self.command(READ, READ_SIZE)?;
        let mut block = [0u8; READ_SIZE];
        block.copy_from_slice(&data);
        Ok(block)
    }
}
