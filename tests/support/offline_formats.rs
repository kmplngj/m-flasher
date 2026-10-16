//! Synthetic byte oracles. These literals were computed outside the Rust
//! implementation with Python binascii.crc_hqx and an independent shift/XOR
//! ENC reference. No real flash image or package is embedded here.

use crate::formats::{self, AppStatus, Entry, Package};

const ENTRY: [u8; 32] = [
    0x93, 0xD5, 0x13, 0xFD, 0x40, 0x02, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x82, 0x00, 0x00, 0x00,
    0x61, 0x70, 0x70, 0x2E, 0x62, 0x69, 0x6E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const ENC_ENTRY: [u8; 32] = [
    0x9C, 0xEA, 0x6D, 0x01, 0x99, 0xB0, 0x64, 0xC8, 0xA0, 0x62, 0xC4, 0x88, 0x92, 0x01, 0x23, 0x46,
    0xCC, 0x2A, 0xE5, 0x25, 0x74, 0x45, 0x17, 0xD3, 0xA6, 0x6D, 0xFB, 0xF6, 0xEC, 0xF9, 0xD3, 0xA6,
];
const ENC_PAYLOAD: [u8; 32] = [
    0xF0, 0x79, 0x58, 0x0F, 0xBC, 0xC6, 0x49, 0xA7, 0xC3, 0x22, 0xE5, 0x60, 0x5C, 0x5F, 0xF7, 0xBE,
    0x24, 0x09, 0x33, 0x47, 0x8E, 0x3D, 0x5B, 0xB6, 0x4D, 0xBB, 0x57, 0xAE, 0x7D, 0xFA, 0xD5, 0xAA,
];
const BANKCB: [u8; 37] = [
    0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x1A, 0xAE, 0x54, 0xF8,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x42, 0x41, 0x4E, 0x4B, 0x21,
];

fn synthetic_flash() -> Vec<u8> {
    let mut flash = vec![0xFF; 0x5000];
    flash[0x4000..0x4020].copy_from_slice(&ENC_ENTRY);
    // Physical APP base 0x4000 plus this entry's logical 0x240. This is
    // deliberately different from the commonly observed 0x400 entry offset.
    flash[0x4240..0x4260].copy_from_slice(&ENC_PAYLOAD);
    flash
}

#[test]
fn literal_crc_and_cipher_oracles() {
    assert_eq!(formats::crc16(b"123456789"), 0x31C3);
    assert_eq!(formats::crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(formats::enc(&ENC_ENTRY, 0x980F), ENTRY);
    let entry = Entry::parse(&ENTRY);
    assert!(entry.valid);
    assert_eq!((entry.addr, entry.len, entry.data_crc), (0x240, 17, 0xFD13));
    assert_eq!(entry.name, "app.bin");
}

#[test]
fn logical_app_offset_is_relative_to_physical_base() {
    let flash = synthetic_flash();
    let table = formats::app_table(&flash, 0x980F);
    assert_eq!(table.len(), 1);
    assert_eq!(table[0].addr, 0x240);
    assert_eq!(
        formats::app_read(&flash, 0x4000, 0x240, 17, 0x980F),
        b"offset-oracle-240"
    );
    assert_ne!(
        formats::app_read(&flash, 0x4000, 0x400, 17, 0x980F),
        b"offset-oracle-240"
    );
    assert_eq!(formats::app_files_status(&flash, 0x980F), AppStatus::Ok);
}

#[test]
fn independent_header_and_payload_crc_mutations_are_detected() {
    let mut entry = ENTRY;
    entry[8] ^= 1;
    assert!(!Entry::parse(&entry).valid);
    let mut flash = synthetic_flash();
    flash[0x4240] ^= 1;
    assert_eq!(formats::app_files_status(&flash, 0x980F), AppStatus::BadCrc);
    flash[0x4000] ^= 1;
    assert!(formats::app_table(&flash, 0x980F).is_empty());
}

#[test]
fn literal_container_uses_declared_nondefault_payload_offset() {
    assert_eq!(formats::bankcb_check(&BANKCB).unwrap(), b"BANK!");
    for size in 0..BANKCB.len() {
        assert!(
            formats::bankcb_check(&BANKCB[..size]).is_err(),
            "truncated length {size}"
        );
    }
    for position in [0, 2, 8, 12, 14, 32, 36] {
        let mut broken = BANKCB;
        broken[position] ^= 1;
        assert!(
            formats::bankcb_check(&broken).is_err(),
            "mutated byte {position}"
        );
    }
}

#[test]
fn malformed_package_and_short_app_lengths_fail_closed() {
    for size in [0, 1, 31, 32, 63, 64, 4095] {
        assert!(Package::parse(&vec![0x7D; size]).is_err());
    }
    assert!(Package::parse(&vec![0x7D; 4096]).is_err());
    assert!(Package::parse(&vec![0; 4096]).is_err());
    for size in [0, 1, 0x3FFF, 0x4000, 0x401F] {
        assert!(formats::app_table(&vec![0xFF; size], 0x980F).is_empty());
    }
    let flash = synthetic_flash();
    // A short saved image may produce only bytes that are actually present.
    assert!(formats::app_read(&flash[..0x4240], 0x4000, 0x240, 17, 0x980F).is_empty());
}
