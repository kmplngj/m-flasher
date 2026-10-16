//! Validate the exact payload of a flash read without changing other IN commands.
//!
//! Standalone, device-free tests:
//! `rustc --edition=2024 --test src/usb_data_in.rs -o target/usb-data-in-tests`
//! `target/usb-data-in-tests`

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ReadError<E> {
    Transport(E),
    Incomplete { expected: usize, actual: usize },
}

/// One data-phase read only. FD05 requests an exact flash chunk; a successful
/// short USB transfer must not turn its untouched buffer tail into flash data.
/// Other commands retain their existing short-IN behavior. Never read again
/// to fill a short payload: the next transfer can be the command's status.
pub(crate) fn read<E>(
    cdb: &[u8],
    buffer: &mut [u8],
    receive: impl FnOnce(&mut [u8]) -> Result<usize, E>,
) -> Result<(), ReadError<E>> {
    let expected = buffer.len();
    let actual = receive(buffer).map_err(ReadError::Transport)?;
    if cdb.starts_with(&[0xfd, 0x05]) && actual != expected {
        return Err(ReadError::Incomplete { expected, actual });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ReadError, read};

    // Independent protocol literals: a 256-byte flash read at physical 0x1200,
    // a 36-byte INQUIRY allocation, and the 16-byte loader-identify response.
    const FLASH: [u8; 16] = [
        0xfd, 0x05, 0x00, 0x00, 0x12, 0x00, 0x01, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff,
    ];
    const INQUIRY: [u8; 6] = [0x12, 0x00, 0x00, 0x00, 0x24, 0x00];
    const IDENTIFY: [u8; 16] = [
        0xfc, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff,
    ];

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Error {
        Stall,
        Timeout,
    }

    #[derive(Debug, PartialEq, Eq)]
    enum Call {
        Data(usize),
        Status,
    }

    struct Fake {
        result: Result<usize, Error>,
        calls: Vec<Call>,
    }

    impl Fake {
        fn transfer(&mut self, cdb: &[u8], buffer: &mut [u8]) -> Result<(), ReadError<Error>> {
            read(cdb, buffer, |bytes| {
                self.calls.push(Call::Data(bytes.len()));
                let count = self.result?;
                for (index, byte) in bytes.iter_mut().take(count).enumerate() {
                    *byte = index as u8 ^ 0x5a;
                }
                Ok(count)
            })?;
            self.calls.push(Call::Status);
            Ok(())
        }
    }

    #[test]
    fn every_short_flash_payload_is_rejected_without_filling_or_continuing() {
        for count in 0..256 {
            let mut fake = Fake {
                result: Ok(count),
                calls: vec![],
            };
            let mut buffer = [0xa5; 256];
            assert_eq!(
                fake.transfer(&FLASH, &mut buffer),
                Err(ReadError::Incomplete {
                    expected: 256,
                    actual: count,
                })
            );
            assert_eq!(fake.calls, [Call::Data(256)]);
            assert!(buffer[count..].iter().all(|byte| *byte == 0xa5));
        }
    }

    #[test]
    fn complete_flash_payload_allows_status_and_preserves_all_returned_bytes() {
        let mut fake = Fake {
            result: Ok(256),
            calls: vec![],
        };
        let mut buffer = [0xa5; 256];
        assert_eq!(fake.transfer(&FLASH, &mut buffer), Ok(()));
        assert_eq!(fake.calls, [Call::Data(256), Call::Status]);
        assert_eq!(buffer, std::array::from_fn(|index| index as u8 ^ 0x5a));
    }

    #[test]
    fn flash_chunk_boundaries_include_partial_final_and_both_loader_sizes() {
        for (length, encoded) in [
            (1, [0x00, 0x01]),
            (255, [0x00, 0xff]),
            (512, [0x02, 0x00]),
            (4096, [0x10, 0x00]),
            (16384, [0x40, 0x00]),
        ] {
            let mut cdb = FLASH;
            cdb[6..8].copy_from_slice(&encoded);
            for actual in [length - 1, length] {
                let mut fake = Fake {
                    result: Ok(actual),
                    calls: vec![],
                };
                let mut buffer = vec![0xa5; length];
                let result = fake.transfer(&cdb, &mut buffer);
                if actual == length {
                    assert_eq!(result, Ok(()));
                    assert_eq!(fake.calls, [Call::Data(length), Call::Status]);
                } else {
                    assert_eq!(
                        result,
                        Err(ReadError::Incomplete {
                            expected: length,
                            actual
                        })
                    );
                    assert_eq!(fake.calls, [Call::Data(length)]);
                }
            }
        }
    }

    #[test]
    fn inquiry_metadata_and_other_opcodes_keep_their_short_in_behavior() {
        for (cdb, length) in [
            (INQUIRY.as_slice(), 36),
            (IDENTIFY.as_slice(), 16),
            (&[0xfd, 0x04], 256),
            (&[0xfb, 0x05], 256),
            (&[], 16),
        ] {
            for count in [0, length - 1, length] {
                let mut fake = Fake {
                    result: Ok(count),
                    calls: vec![],
                };
                assert_eq!(fake.transfer(cdb, &mut vec![0xa5; length]), Ok(()));
                assert_eq!(fake.calls, [Call::Data(length), Call::Status]);
            }
        }
    }

    #[test]
    fn transport_errors_are_preserved_without_retry_or_further_data_reads() {
        for error in [Error::Stall, Error::Timeout] {
            let mut fake = Fake {
                result: Err(error),
                calls: vec![],
            };
            assert_eq!(
                fake.transfer(&FLASH, &mut [0xa5; 256]),
                Err(ReadError::Transport(error))
            );
            assert_eq!(fake.calls, [Call::Data(256)]);
        }
    }
}
