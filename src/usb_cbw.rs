//! Send one complete Bulk-Only Transport command wrapper before its data phase.

#[derive(Debug, PartialEq)]
pub(crate) enum WriteError<E> {
    Transport(E),
    Incomplete(usize),
}

pub(crate) fn write<E>(
    cbw: &[u8; 31],
    send: impl FnOnce(&[u8]) -> Result<usize, E>,
) -> Result<(), WriteError<E>> {
    let count = send(cbw).map_err(WriteError::Transport)?;
    if count != cbw.len() {
        return Err(WriteError::Incomplete(count));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{WriteError, write};

    // Independent literal fixture: tag 0x12345678, 36-byte INQUIRY data IN,
    // LUN 0 and a six-byte CDB, padded to the fixed 31-byte BOT wrapper.
    const INQUIRY: [u8; 31] = [
        0x55, 0x53, 0x42, 0x43, 0x78, 0x56, 0x34, 0x12, 0x24, 0x00, 0x00, 0x00, 0x80, 0x00, 0x06,
        0x12, 0x00, 0x00, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum FakeError {
        Stall,
        Timeout,
    }

    struct Fake {
        result: Result<usize, FakeError>,
        writes: Vec<Vec<u8>>,
        data_reads: usize,
        status_reads: usize,
    }

    impl Fake {
        fn new(result: Result<usize, FakeError>) -> Self {
            Self {
                result,
                writes: Vec::new(),
                data_reads: 0,
                status_reads: 0,
            }
        }

        fn send(&mut self, bytes: &[u8]) -> Result<usize, FakeError> {
            self.writes.push(bytes.to_vec());
            std::mem::replace(&mut self.result, Err(FakeError::Timeout))
        }

        fn transfer(&mut self) -> Result<(), WriteError<FakeError>> {
            write(&INQUIRY, |bytes| self.send(bytes))?;
            self.data_reads += 1;
            self.status_reads += 1;
            Ok(())
        }

        fn assert_command_sent_once(&self) {
            assert_eq!(self.writes, [INQUIRY.to_vec()]);
        }
    }

    #[test]
    fn every_short_cbw_stops_before_data_or_status_without_resending() {
        for count in 0..31 {
            let mut fake = Fake::new(Ok(count));
            assert_eq!(fake.transfer(), Err(WriteError::Incomplete(count)));
            fake.assert_command_sent_once();
            assert_eq!((fake.data_reads, fake.status_reads), (0, 0));
        }
    }

    #[test]
    fn complete_cbw_allows_the_data_and_status_phases() {
        let mut fake = Fake::new(Ok(31));
        assert_eq!(fake.transfer(), Ok(()));
        fake.assert_command_sent_once();
        assert_eq!((fake.data_reads, fake.status_reads), (1, 1));
    }

    #[test]
    fn transport_error_is_preserved_without_resending_or_continuing() {
        for error in [FakeError::Stall, FakeError::Timeout] {
            let mut fake = Fake::new(Err(error));
            assert_eq!(fake.transfer(), Err(WriteError::Transport(error)));
            fake.assert_command_sent_once();
            assert_eq!((fake.data_reads, fake.status_reads), (0, 0));
        }
    }
}
