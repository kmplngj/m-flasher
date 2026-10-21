//! Independent bootstrap transcript for synthetic bytes, never a loadable image.
//! This is a cfg(test) child of fmr_bootstrap so only local fakes can bypass the
//! production Image SHA-256 constructor. Expected bytes were generated using
//! Python binascii.crc_hqx, not the Rust upload/cipher implementations.
use super::{Image, start};
use crate::fmr_bot::{Bot, Bulk};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const ENCRYPTED_512: &str = "2fa583cf4ac45bede6abbcad2587daf81395f9f2db9f9c03a5ffc707cce55824bd20cf70b00ac786efbc3ba6fad2841290d980aeb59a72eb97b344676d85d9c0daa29fbaf50e1905f87582628e6b3637873ce1562ec209e4a290a3bbc97e74a271871c55e6943d412561231192479575aa11852604d43acc793173cd239777bc60b093207c65f7f68d32270127b22721ff190faf98296a47b82d73035ff44cba744b69aee863938fbb13456256ca30109cfe9b264eaaf9c62098b38251ad12dcda266299254d8a37b7d6fab5791da62c335dbe21e99812565cf9db6aa13eebaff7bd6ed1ce8c851fb38fee18569b4359f04f7f18985f7168af82d93a6f0235fbfd308f8fcec457b17c626a2c1a6b87f9fee0f4297b3c1cbb73c890d3503389516f03ab08f5ddf7daaa8d29fc00513f786f4903f93ecf9b1ada6cd5334a5aa475570490152c6cab7cb3b514c2447b72fba238d37e1232297fc5e4d9a56028624c0df5a3f4d26cbcd220fb36e04c34375ef0910ba9f0b9c40efb64b3a4ff3f9a0a8b66329b361ec153edf167264d7831fa05331dafeb38698861e9eb11c6e8c200067e3acd2ddfd1babbe52537fa73bd3afe30706f1fb80516c8f10565db966040202f2ba4e7e0d850a66082cfebd99108f6c2a72958d223ab3832b628c828b9781105d86a521340305d899193693dd599b694708d446430ed3b2927721f00c8af";
const CRC_512: [u8; 2] = [0xdb, 0xcb];
const CRC_432: [u8; 2] = [0x78, 0xfc];

#[derive(Clone, Debug, PartialEq, Eq)]
enum Call {
    Write(Vec<u8>),
    Read(usize),
}
type Trace = Rc<RefCell<Vec<Call>>>;
#[derive(Clone, Copy)]
enum Fault {
    None,
    Error(usize),
    Short(usize),
    Deadline(usize),
    BadEcho,
}
struct Fake {
    trace: Trace,
    fault: Fault,
    elapsed: Duration,
}
impl Fake {
    fn result(&mut self, len: usize) -> Result<usize, String> {
        let index = self.trace.borrow().len() - 1;
        match self.fault {
            Fault::Error(at) if at == index => Err("injected transport error".into()),
            Fault::Short(at) if at == index => Ok(len - 1),
            Fault::Deadline(at) if at == index => {
                self.elapsed += Duration::from_secs(1);
                Ok(len)
            }
            _ => Ok(len),
        }
    }
}
impl Bulk for Fake {
    fn write(&mut self, bytes: &[u8], timeout: Duration) -> Result<usize, String> {
        assert!(!timeout.is_zero());
        self.trace.borrow_mut().push(Call::Write(bytes.to_vec()));
        self.result(bytes.len())
    }
    fn read(&mut self, bytes: &mut [u8], timeout: Duration) -> Result<usize, String> {
        assert!(!timeout.is_zero());
        self.trace.borrow_mut().push(Call::Read(bytes.len()));
        let index = self.trace.borrow().len() - 1;
        if index == 10 {
            assert_eq!(bytes.len(), 16);
            bytes.fill(0);
            bytes[..2].copy_from_slice(if matches!(self.fault, Fault::BadEcho) {
                &[0xfc, 0x40]
            } else {
                &[0xfb, 0x08]
            });
        } else {
            let tag = match index {
                2 => 1,
                5 => 2,
                8 => 3,
                11 => 4,
                _ => panic!("unexpected status read"),
            };
            assert_eq!(bytes.len(), 13);
            bytes.copy_from_slice(&[b'U', b'S', b'B', b'S', tag, 0, 0, 0, 0, 0, 0, 0, 0]);
        }
        self.result(bytes.len())
    }
    fn elapsed(&self) -> Duration {
        self.elapsed
    }
}
fn setup(fault: Fault) -> (Bot<Fake>, Trace) {
    let trace = Rc::new(RefCell::new(vec![]));
    let fake = Fake {
        trace: trace.clone(),
        fault,
        elapsed: Duration::ZERO,
    };
    (Bot::new(fake, Duration::from_secs(1)).unwrap(), trace)
}
fn synthetic_image() -> Image {
    Image((0..1456).map(|i| (i % 256) as u8).collect())
}
fn encrypted() -> Vec<u8> {
    ENCRYPTED_512
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn write_cbw(index: u8) -> Vec<u8> {
    // Independent literal CBW/CDB templates: RAM addresses20000/20200/20400,
    // fresh tags1/2/3, fixed512/512/432 lengths, cipher CRC serialized LE.
    let (length_low, length_high, address, crc) = match index {
        1 => (0, 2, 0, CRC_512),
        2 => (0, 2, 2, CRC_512),
        3 => (0xb0, 1, 4, CRC_432),
        _ => panic!("index"),
    };
    vec![
        b'U',
        b'S',
        b'B',
        b'C',
        index,
        0,
        0,
        0,
        length_low,
        length_high,
        0,
        0,
        0,
        0,
        16,
        0xfb,
        0x06,
        0x01,
        0xc2,
        address,
        0,
        length_high,
        length_low,
        0,
        crc[0],
        crc[1],
        0xff,
        0xff,
        0xff,
        0xff,
        0xff,
    ]
}
fn terminal(bot: &mut Bot<Fake>, trace: &Trace, expected_calls: usize) {
    assert_eq!(trace.borrow().len(), expected_calls);
    assert!(start(bot, synthetic_image()).is_err());
    assert!(bot.read(&[0; 16], 16).is_err());
    assert_eq!(
        trace.borrow().len(),
        expected_calls,
        "terminal/repeated bootstrap sent traffic"
    );
}

#[test]
fn independent_encrypted_three_block_and_jump_transcript() {
    let (mut bot, trace) = setup(Fault::None);
    start(&mut bot, synthetic_image()).unwrap();
    let encrypted = encrypted();
    let jump = vec![
        b'U', b'S', b'B', b'C', 4, 0, 0, 0, 16, 0, 0, 0, 0x80, 0, 16, 0xfb, 0x08, 0x01, 0xc2, 0, 0,
        0, 1, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    assert_eq!(
        *trace.borrow(),
        vec![
            Call::Write(write_cbw(1)),
            Call::Write(encrypted.clone()),
            Call::Read(13),
            Call::Write(write_cbw(2)),
            Call::Write(encrypted.clone()),
            Call::Read(13),
            Call::Write(write_cbw(3)),
            Call::Write(encrypted[..432].to_vec()),
            Call::Read(13),
            Call::Write(jump),
            Call::Read(16),
            Call::Read(13)
        ]
    );
    terminal(&mut bot, &trace, 12);
}

#[test]
fn every_upload_and_jump_phase_failure_stops_without_retry() {
    for phase in 0..12 {
        for fault in [
            Fault::Error(phase),
            Fault::Short(phase),
            Fault::Deadline(phase),
        ] {
            let (mut bot, trace) = setup(fault);
            assert!(start(&mut bot, synthetic_image()).is_err(), "phase{phase}");
            terminal(&mut bot, &trace, phase + 1);
        }
    }
}

#[test]
fn successful_bot_framing_with_wrong_jump_echo_is_terminal() {
    let (mut bot, trace) = setup(Fault::BadEcho);
    assert!(
        start(&mut bot, synthetic_image())
            .unwrap_err()
            .contains("echo FB08")
    );
    terminal(&mut bot, &trace, 12);
}

#[test]
fn synthetic_bytes_cannot_enter_production_image_constructor() {
    let plain: Vec<_> = (0..1456).map(|i| (i % 256) as u8).collect();
    assert!(Image::verify(plain).is_err());
}

#[test]
fn bootstrap_emits_only_ram_write_and_one_jump_opcodes() {
    let (mut bot, trace) = setup(Fault::None);
    start(&mut bot, synthetic_image()).unwrap();
    let opcodes: Vec<_> = trace
        .borrow()
        .iter()
        .filter_map(|call| match call {
            Call::Write(bytes) if bytes.len() == 31 && &bytes[..4] == b"USBC" => {
                Some([bytes[15], bytes[16]])
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        opcodes,
        vec![[0xfb, 0x06], [0xfb, 0x06], [0xfb, 0x06], [0xfb, 0x08]]
    );
}
