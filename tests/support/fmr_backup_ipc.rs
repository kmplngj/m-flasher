//! Independent literals exercise only bounded in-memory codecs.
use crate::fmr_backup_ipc::{self as ipc, Captured, Request};
fn literal_request() -> Vec<u8> {
    let mut bytes = vec![6, 3, 0x11, 0, 0, 3, 7];
    bytes.extend_from_slice(b"0123456789ABCDEF");
    bytes.extend((0..1456).map(|v| (v % 256) as u8));
    bytes
}
fn request() -> Request {
    Request {
        location: 0x03110000,
        device: "usb:3-7".into(),
        serial: "0123456789ABCDEF".into(),
        image: (0..1456).map(|v| (v % 256) as u8).collect(),
    }
}
fn literal_capture(binding: &[u8]) -> Vec<u8> {
    let mut bytes = (binding.len() as u16).to_le_bytes().to_vec();
    bytes.extend_from_slice(binding);
    bytes.extend((0..2).flat_map(|_| (0..1048576).map(|v| (v % 251) as u8)));
    bytes
}
#[test]
fn full_request_has_separate_opcode_and_strict_existing_identity_codec() {
    assert_eq!(ipc::RAW_BACKUP, 6);
    assert_eq!(ipc::REQUEST_LEN, 1479);
    assert_eq!(ipc::IMAGE_LEN, 1456);
    assert_eq!(ipc::encode_request(&request()).unwrap(), literal_request());
    assert_eq!(ipc::decode_request(&literal_request()).unwrap(), request());
    let valid = literal_request();
    for n in 0..valid.len() {
        assert!(ipc::decode_request(&valid[..n]).is_err());
    }
    for opcode in 0..=255 {
        if opcode == 6 {
            continue;
        }
        let mut frame = valid.clone();
        frame[0] = opcode;
        assert!(ipc::decode_request(&frame).is_err());
    }
    let mut extra = valid.clone();
    extra.push(0);
    assert!(ipc::decode_request(&extra).is_err());
    for (offset, byte) in [(1, 4), (6, 0), (6, 128), (7, b'g'), (7, 0xff)] {
        let mut frame = valid.clone();
        frame[offset] = byte;
        assert!(ipc::decode_request(&frame).is_err());
    }
    for device in [
        "usb:03-7",
        "usb:3-07",
        "usb:+3-7",
        "usb:3-0",
        "usb:3-128",
        "/dev/disk7",
    ] {
        let mut request = request();
        request.device = device.into();
        assert!(ipc::encode_request(&request).is_err());
    }
}
#[test]
fn captured_literal_boundaries_and_utf8_round_trip() {
    assert_eq!(ipc::RAW_LEN, 1048576);
    assert_eq!(ipc::MAX_CAPTURED_LEN, 2 + 8192 + 2097152);
    assert_eq!((ipc::FAILED, ipc::CAPTURED, ipc::FINISHED), (1, 10, 11));
    for binding in ["", "bound ü🙂", &"x".repeat(8192)] {
        let frame = literal_capture(binding.as_bytes());
        let captured = ipc::decode_captured(&frame).unwrap();
        assert_eq!(captured.binding, binding);
        assert_eq!(captured.first.len(), 1048576);
        assert_eq!(captured.second, captured.first);
        assert_eq!(ipc::encode_captured(&captured).unwrap(), frame);
    }
}
#[test]
fn captured_invalid_lengths_utf8_and_equality_are_refused() {
    let valid = literal_capture(b"bound");
    for n in [0, 1, 2, 6, 7, 1048576, valid.len() - 1] {
        assert!(ipc::decode_captured(&valid[..n]).is_err(), "length {n}");
    }
    let mut frame = valid.clone();
    frame.push(0);
    assert!(ipc::decode_captured(&frame).is_err());
    for n in [0u16, 4, 6, 8193, 65535] {
        let mut frame = valid.clone();
        frame[..2].copy_from_slice(&n.to_le_bytes());
        assert!(ipc::decode_captured(&frame).is_err());
    }
    assert!(ipc::decode_captured(&literal_capture(&vec![b'x'; 8193])).is_err());
    for binding in [&[0xff][..], &[0xc0, 0x80][..], &[0xed, 0xa0, 0x80][..]] {
        assert!(ipc::decode_captured(&literal_capture(binding)).is_err());
    }
    for pass in 0..2 {
        for offset in [0, 255, 1048575] {
            let mut frame = valid.clone();
            frame[7 + pass * 1048576 + offset] ^= 1;
            assert!(ipc::decode_captured(&frame).is_err());
        }
    }
    let good = ipc::decode_captured(&valid).unwrap();
    for length in [0, 1, 1048575, 1048577] {
        let bad = Captured {
            first: vec![0; length],
            second: vec![0; length],
            binding: String::new(),
        };
        assert!(ipc::encode_captured(&bad).is_err());
    }
    let mut bad = good;
    bad.second[1048575] ^= 1;
    assert!(ipc::encode_captured(&bad).is_err());
    bad.second[1048575] ^= 1;
    bad.binding = "ü".repeat(4097);
    assert!(ipc::encode_captured(&bad).is_err());
}
#[test]
fn storage_ack_is_exactly_one_digest_and_never_an_arbitrary_command() {
    let digest = [0xa5; 32];
    let mut literal = vec![0xa6];
    literal.extend([0xa5; 32]);
    assert_eq!(ipc::encode_ack(&digest).as_slice(), literal);
    assert_eq!(ipc::decode_ack(&literal).unwrap(), digest);
    for n in 0..33 {
        assert!(ipc::decode_ack(&literal[..n]).is_err());
    }
    let mut extra = literal.clone();
    extra.push(0);
    assert!(ipc::decode_ack(&extra).is_err());
    for opcode in 0..=255 {
        if opcode == 0xa6 {
            continue;
        }
        let mut frame = literal.clone();
        frame[0] = opcode;
        assert!(ipc::decode_ack(&frame).is_err());
    }
}
