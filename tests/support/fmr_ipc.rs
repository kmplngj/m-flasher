//! Independent fixed helper frames; these tests contain no I/O or device code.
use crate::fmr_ipc::{self, Request, Response};

fn request() -> Request {
    Request {
        location: 0x0311_0000,
        device: "usb:3-7".into(),
        serial: "0123456789ABCDEF".into(),
        image: (0..1456).map(|i| (i % 256) as u8).collect(),
    }
}
fn literal_request() -> Vec<u8> {
    let mut bytes = vec![5, 3, 0x11, 0, 0, 3, 7];
    bytes.extend_from_slice(b"0123456789ABCDEF");
    bytes.extend((0..1456).map(|i| (i % 256) as u8));
    bytes
}
fn response() -> Response {
    Response {
        binding: "bound".into(),
        first: [0xa5; 256],
        second: [0xa5; 256],
    }
}
fn literal_response() -> Vec<u8> {
    let mut bytes = vec![5, 0, b'b', b'o', b'u', b'n', b'd'];
    bytes.extend([0xa5; 512]);
    bytes
}

#[test]
fn fixed_request_and_response_match_independent_literals() {
    assert_eq!(fmr_ipc::REQUEST_LEN, 1479);
    assert_eq!(
        fmr_ipc::encode_request(&request()).unwrap(),
        literal_request()
    );
    assert_eq!(
        fmr_ipc::decode_request(&literal_request()).unwrap(),
        request()
    );
    assert_eq!(
        fmr_ipc::encode_response(&response()).unwrap(),
        literal_response()
    );
    assert_eq!(
        fmr_ipc::decode_response(&literal_response()).unwrap(),
        response()
    );
}

#[test]
fn every_request_truncation_trailer_and_foreign_operation_is_rejected() {
    let valid = literal_request();
    for length in 0..valid.len() {
        assert!(
            fmr_ipc::decode_request(&valid[..length]).is_err(),
            "length{length}"
        );
    }
    for extra in [1, 2, 256, 8192] {
        let mut bytes = valid.clone();
        bytes.resize(valid.len() + extra, 0);
        assert!(fmr_ipc::decode_request(&bytes).is_err());
    }
    for op in 0..=255 {
        if op == 5 {
            continue;
        }
        let mut bytes = valid.clone();
        bytes[0] = op;
        assert!(fmr_ipc::decode_request(&bytes).is_err(), "opcode{op}");
    }
}

#[test]
fn all_invalid_request_identity_bytes_are_rejected() {
    for address in [0].into_iter().chain(128..=255) {
        let mut bytes = literal_request();
        bytes[6] = address;
        assert!(fmr_ipc::decode_request(&bytes).is_err());
    }
    for bus in 0..=255 {
        if bus == 3 {
            continue;
        }
        let mut bytes = literal_request();
        bytes[5] = bus;
        assert!(fmr_ipc::decode_request(&bytes).is_err());
        let mut bytes = literal_request();
        bytes[1] = bus;
        assert!(fmr_ipc::decode_request(&bytes).is_err());
    }
    let mut zero_location = literal_request();
    zero_location[1..5].fill(0);
    zero_location[5] = 0;
    assert!(fmr_ipc::decode_request(&zero_location).is_err());
    for offset in 7..23 {
        for value in 0..=255u8 {
            if value.is_ascii_hexdigit() {
                continue;
            }
            let mut bytes = literal_request();
            bytes[offset] = value;
            assert!(
                fmr_ipc::decode_request(&bytes).is_err(),
                "serialbyte{offset}value{value}"
            );
        }
    }
}

#[test]
fn encoder_rejects_paths_arbitrary_device_syntax_and_invalid_fields() {
    for device in [
        "",
        "usb:03-7",
        "usb:3-07",
        "usb:+3-7",
        "usb:3-+7",
        "usb:3-0",
        "usb:3-128",
        "usb:256-7",
        "usb:3-256",
        "usb:3-7-1",
        "usb:3-7\n",
        "usb:3- 7",
        "/dev/disk7",
        "--device usb:3-7",
        "usb:３-7",
    ] {
        let mut value = request();
        value.device = device.into();
        assert!(fmr_ipc::encode_request(&value).is_err(), "{device:?}");
    }
    for serial in [
        "",
        "0123456789ABCDE",
        "0123456789ABCDEFG",
        "0123456789ABCDEG",
        "0123456789ABCDE\0",
        "０１２３４５６７",
        "0123456789ABCDE\n",
    ] {
        let mut value = request();
        value.serial = serial.into();
        assert!(fmr_ipc::encode_request(&value).is_err());
    }
    for length in [0, 1, 1455, 1457, 8192] {
        let mut value = request();
        value.image.resize(length, 0);
        assert!(fmr_ipc::encode_request(&value).is_err());
    }
    for location in [0, 0x04110000] {
        let mut value = request();
        value.location = location;
        assert!(fmr_ipc::encode_request(&value).is_err());
    }
}

#[test]
fn canonical_identity_boundaries_and_lowercase_serial_are_normalized() {
    for (device, location) in [("usb:0-1", 0x00110000), ("usb:255-127", 0xff110000)] {
        let mut value = request();
        value.device = device.into();
        value.location = location;
        let frame = fmr_ipc::encode_request(&value).unwrap();
        assert_eq!(fmr_ipc::decode_request(&frame).unwrap(), value);
    }
    let mut value = request();
    value.serial = "0123456789abcdef".into();
    assert_eq!(fmr_ipc::encode_request(&value).unwrap(), literal_request());
    let mut frame = literal_request();
    frame[7..23].copy_from_slice(b"0123456789abcdef");
    assert_eq!(fmr_ipc::decode_request(&frame).unwrap(), request());
}

#[test]
fn every_response_truncation_trailer_and_length_mismatch_is_rejected() {
    let valid = literal_response();
    for length in 0..valid.len() {
        assert!(
            fmr_ipc::decode_response(&valid[..length]).is_err(),
            "length{length}"
        );
    }
    for extra in [1, 256, 8192] {
        let mut bytes = valid.clone();
        bytes.resize(valid.len() + extra, 0);
        assert!(fmr_ipc::decode_response(&bytes).is_err());
    }
    for length in [0u16, 1, 4, 6, 8192, 8193, 65535] {
        let mut bytes = valid.clone();
        bytes[..2].copy_from_slice(&length.to_le_bytes());
        assert!(fmr_ipc::decode_response(&bytes).is_err());
    }
}

#[test]
fn binding_byte_bounds_and_utf8_are_enforced() {
    for binding in [String::new(), "ü🙂".into(), "x".repeat(8192)] {
        let value = Response {
            binding,
            ..response()
        };
        let frame = fmr_ipc::encode_response(&value).unwrap();
        assert_eq!(fmr_ipc::decode_response(&frame).unwrap(), value);
    }
    let value = Response {
        binding: "x".repeat(8193),
        ..response()
    };
    assert!(fmr_ipc::encode_response(&value).is_err());
    let mut frame = vec![1, 32];
    frame.extend(vec![b'x'; 8193]);
    frame.extend([0xa5; 512]);
    assert!(fmr_ipc::decode_response(&frame).is_err());
    for bytes in [
        vec![0x80],
        vec![0xff],
        vec![0xc0, 0x80],
        vec![0xed, 0xa0, 0x80],
        vec![0xf0, 0x90, 0x80],
    ] {
        let mut frame = (bytes.len() as u16).to_le_bytes().to_vec();
        frame.extend(bytes);
        frame.extend([0xa5; 512]);
        assert!(fmr_ipc::decode_response(&frame).is_err());
    }
}

#[test]
fn every_sample_byte_difference_is_rejected_in_both_directions() {
    for offset in 0..256 {
        for sample in [0, 1] {
            let mut value = response();
            if sample == 0 {
                value.first[offset] ^= 1;
            } else {
                value.second[offset] ^= 1;
            }
            assert!(fmr_ipc::encode_response(&value).is_err());
            let mut frame = literal_response();
            frame[7 + sample * 256 + offset] ^= 1;
            assert!(fmr_ipc::decode_response(&frame).is_err());
        }
    }
}
