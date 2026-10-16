//! Strict BOT framing over injected bulk endpoints. No recovery or device access.

use std::time::Duration;

/// Exactly one endpoint operation. Implementations must honor the supplied
/// positive timeout and must not retry, reset, clear a halt or send commands.
/// `elapsed` is a monotonic clock shared by every operation on this endpoint.
pub trait Bulk {
    fn write(&mut self, bytes: &[u8], timeout: Duration) -> Result<usize, String>;
    fn read(&mut self, bytes: &mut [u8], timeout: Duration) -> Result<usize, String>;
    fn elapsed(&self) -> Duration;
}

pub struct Bot<B: Bulk> {
    bulk: B,
    timeout: Duration,
    last_tag: u32,
    terminal: bool,
    bootstrap_claimed: bool,
}

impl<B: Bulk> Bot<B> {
    pub fn new(bulk: B, timeout: Duration) -> Result<Self, String> {
        if timeout.is_zero() {
            return Err("BOT transaction timeout must be positive".into());
        }
        Ok(Self {
            bulk,
            timeout,
            last_tag: 0,
            terminal: false,
            bootstrap_claimed: false,
        })
    }

    /// Reserve this retained session's sole bootstrap before sending any byte.
    pub(crate) fn claim_bootstrap(&mut self) -> Result<(), String> {
        if self.terminal || self.bootstrap_claimed {
            self.terminal = true;
            return Err("BOT bootstrap is already consumed or session is terminal".into());
        }
        self.bootstrap_claimed = true;
        Ok(())
    }

    pub(crate) fn poison(&mut self) {
        self.terminal = true;
    }

    // Crate-private: the selected profile/bootstrap owns the command allowlist.
    // No public generic command interface or recovery operation is exposed.
    pub(crate) fn read(&mut self, cdb: &[u8; 16], len: usize) -> Result<Vec<u8>, String> {
        self.begin()?;
        let result = self.read_inner(cdb, len);
        if result.is_ok() {
            self.terminal = false;
        }
        result
    }

    pub(crate) fn write(&mut self, cdb: &[u8; 16], data: &[u8]) -> Result<(), String> {
        self.begin()?;
        let result = self.write_inner(cdb, data);
        if result.is_ok() {
            self.terminal = false;
        }
        result
    }

    fn begin(&mut self) -> Result<(), String> {
        if self.terminal {
            return Err("BOT session is terminal".into());
        }
        // Poison before starting, including panic/unwind from an adapter.
        self.terminal = true;
        Ok(())
    }

    fn prepare(
        &mut self,
        cdb: &[u8; 16],
        len: usize,
        direction: u8,
    ) -> Result<(Duration, u32, [u8; 31]), String> {
        let transfer = u32::try_from(len).map_err(|_| "BOT transfer length exceeds u32")?;
        let deadline = self
            .bulk
            .elapsed()
            .checked_add(self.timeout)
            .ok_or("BOT transaction deadline overflow")?;
        let tag = self
            .last_tag
            .checked_add(1)
            .ok_or("BOT tag space exhausted")?;
        self.last_tag = tag;
        let mut cbw = [0u8; 31];
        cbw[..4].copy_from_slice(b"USBC");
        cbw[4..8].copy_from_slice(&tag.to_le_bytes());
        cbw[8..12].copy_from_slice(&transfer.to_le_bytes());
        cbw[12] = direction;
        cbw[14] = 16;
        cbw[15..].copy_from_slice(cdb);
        Ok((deadline, tag, cbw))
    }

    fn remaining(&self, deadline: Duration) -> Result<Duration, String> {
        let remaining = deadline
            .checked_sub(self.bulk.elapsed())
            .filter(|value| !value.is_zero())
            .ok_or("BOT transaction deadline expired")?;
        // Native APIs use milliseconds: truncation would create a zero/infinite
        // timeout. The deadline is checked again after the bounded endpoint call.
        let millis = remaining
            .as_millis()
            .checked_add(u128::from(remaining.subsec_nanos() % 1_000_000 != 0))
            .ok_or("BOT timeout conversion overflow")?;
        let millis = u64::try_from(millis).map_err(|_| "BOT timeout exceeds milliseconds range")?;
        Ok(Duration::from_millis(millis))
    }

    fn send(&mut self, bytes: &[u8], deadline: Duration) -> Result<(), String> {
        let timeout = self.remaining(deadline)?;
        let actual = self.bulk.write(bytes, timeout)?;
        self.remaining(deadline)?;
        if actual != bytes.len() {
            return Err(format!(
                "BOT OUT count: expected {}, received {actual}",
                bytes.len()
            ));
        }
        Ok(())
    }

    fn receive(&mut self, bytes: &mut [u8], deadline: Duration) -> Result<(), String> {
        let timeout = self.remaining(deadline)?;
        let actual = self.bulk.read(bytes, timeout)?;
        self.remaining(deadline)?;
        if actual != bytes.len() {
            return Err(format!(
                "BOT IN count: expected {}, received {actual}",
                bytes.len()
            ));
        }
        Ok(())
    }

    fn status(&mut self, tag: u32, deadline: Duration) -> Result<(), String> {
        let mut csw = [0u8; 13];
        self.receive(&mut csw, deadline)?;
        if &csw[..4] != b"USBS" {
            return Err("BOT CSW signature mismatch".into());
        }
        if csw[4..8] != tag.to_le_bytes() {
            return Err("BOT CSW tag mismatch".into());
        }
        if csw[8..12] != [0; 4] {
            return Err("BOT CSW residue is nonzero".into());
        }
        if csw[12] != 0 {
            return Err("BOT CSW status is not success".into());
        }
        Ok(())
    }

    fn read_inner(&mut self, cdb: &[u8; 16], len: usize) -> Result<Vec<u8>, String> {
        let (deadline, tag, cbw) = self.prepare(cdb, len, 0x80)?;
        self.send(&cbw, deadline)?;
        let mut data = vec![0u8; len];
        if !data.is_empty() {
            self.receive(&mut data, deadline)?;
        }
        self.status(tag, deadline)?;
        Ok(data)
    }

    fn write_inner(&mut self, cdb: &[u8; 16], data: &[u8]) -> Result<(), String> {
        let (deadline, tag, cbw) = self.prepare(cdb, data.len(), 0)?;
        self.send(&cbw, deadline)?;
        if !data.is_empty() {
            self.send(data, deadline)?;
        }
        self.status(tag, deadline)
    }
}

impl<B: Bulk> crate::fmr_reader::Transport for Bot<B> {
    fn transaction(&mut self, cdb: &[u8; 16], expected_len: usize) -> Result<Vec<u8>, String> {
        self.read(cdb, expected_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Never;
    impl Bulk for Never {
        fn write(&mut self, _: &[u8], _: Duration) -> Result<usize, String> {
            panic!("unexpected write")
        }
        fn read(&mut self, _: &mut [u8], _: Duration) -> Result<usize, String> {
            panic!("unexpected read")
        }
        fn elapsed(&self) -> Duration {
            Duration::ZERO
        }
    }
    #[test]
    fn exhausted_tags_poison_before_any_endpoint_operation() {
        let mut bot = Bot::new(Never, Duration::from_secs(1)).unwrap();
        bot.last_tag = u32::MAX;
        assert_eq!(
            bot.read(&[0; 16], 16),
            Err("BOT tag space exhausted".into())
        );
        assert_eq!(
            bot.read(&[0; 16], 16),
            Err("BOT session is terminal".into())
        );
    }
}
