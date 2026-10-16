//! Pure argument tests: these deliberately name nonexistent paths.

use crate::fmr_options::{self, Command};
use std::path::PathBuf;

fn valid() -> Vec<String> {
    [
        "--prepare",
        "--reader",
        "/nonexistent/reader image.bin",
        "--device",
        "usb:171-127",
        "--location",
        "aBcD0001",
        "--serial",
        "aBcD0123456789ef",
        "--output",
        "/nonexistent/output directory",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn with(flag: &str, value: &str) -> Vec<String> {
    let mut args = valid();
    let at = args.iter().position(|v| v == flag).unwrap();
    args[at + 1] = value.into();
    args
}

#[test]
fn prepare_and_run_parse_without_opening_paths() {
    let mut args = valid();
    let Command::Prepare(options) = fmr_options::parse(&args).unwrap() else {
        panic!("wrong mode")
    };
    assert_eq!(
        options.reader,
        PathBuf::from("/nonexistent/reader image.bin")
    );
    assert_eq!(
        options.output,
        PathBuf::from("/nonexistent/output directory")
    );
    assert_eq!(options.device, "usb:171-127");
    assert_eq!(options.location, 0xABCD0001);
    assert_eq!(options.serial, "ABCD0123456789EF");
    args[0] = "--run".into();
    assert_eq!(fmr_options::parse(&args), Ok(Command::Run(options)));
}

#[test]
fn exactly_one_mode_and_each_required_value_are_mandatory() {
    assert!(fmr_options::parse(&[]).is_err());
    let mut args = valid();
    args.remove(0);
    assert!(fmr_options::parse(&args).is_err());
    for flag in ["--prepare", "--run"] {
        let mut args = valid();
        args.push(flag.into());
        assert!(fmr_options::parse(&args).is_err());
    }
    for flag in ["--reader", "--device", "--location", "--serial", "--output"] {
        let mut args = valid();
        let at = args.iter().position(|v| v == flag).unwrap();
        args.drain(at..at + 2);
        assert!(fmr_options::parse(&args).is_err(), "missing {flag}");
        assert!(fmr_options::parse(&with(flag, "")).is_err(), "empty {flag}");
        let mut args = valid();
        let at = args.iter().position(|v| v == flag).unwrap();
        args.remove(at + 1);
        assert!(fmr_options::parse(&args).is_err(), "no value {flag}");
        let mut args = valid();
        args.extend([flag.into(), "duplicate".into()]);
        assert!(fmr_options::parse(&args).is_err(), "duplicate {flag}");
    }
}

#[test]
fn usb_selector_boundaries_match_preflight_in_both_modes() {
    // Explicit selector/location pairs are independent of the production parser.
    for mode in ["--prepare", "--run"] {
        for (device, location, expected_location) in [
            ("usb:0-1", "00000001", 0x00000001),
            ("usb:0-127", "00010000", 0x00010000),
            ("usb:255-1", "ff000001", 0xFF000001),
            ("usb:255-127", "FF010000", 0xFF010000),
        ] {
            let mut args = with("--device", device);
            args[0] = mode.into();
            let at = args.iter().position(|v| v == "--location").unwrap();
            args[at + 1] = location.into();
            let command = fmr_options::parse(&args).unwrap();
            let options = match (mode, command) {
                ("--prepare", Command::Prepare(options)) | ("--run", Command::Run(options)) => {
                    options
                }
                _ => panic!("wrong mode"),
            };
            assert_eq!(options.device, device);
            assert_eq!(options.location, expected_location);
        }
        for device in ["usb:171-0", "usb:171-128", "usb:171-200", "usb:171-255"] {
            let mut args = with("--device", device);
            args[0] = mode.into();
            assert!(fmr_options::parse(&args).is_err(), "{mode} {device}");
        }
        let mut args = with("--location", "ACCD0001");
        args[0] = mode.into();
        assert_eq!(
            fmr_options::parse(&args),
            Err("--location must match selected USB bus".into()),
            "{mode}"
        );
    }
}

#[test]
fn malformed_device_location_and_serial_are_refused() {
    for value in [
        "usb:01-2",
        "usb:1-02",
        "usb:+1-2",
        "usb:1-+2",
        "usb:256-2",
        "usb:1-256",
        "usb:1-2-3",
        "usb:1",
        "USB:1-2",
        "usb:-2",
        "1-2",
        "usb: 1-2",
    ] {
        assert!(
            fmr_options::parse(&with("--device", value)).is_err(),
            "{value}"
        );
    }
    for value in [
        "00000000",
        "1234567",
        "123456789",
        "0x12345678",
        "0000000g",
        " 0000001",
        "000000１",
    ] {
        assert!(
            fmr_options::parse(&with("--location", value)).is_err(),
            "{value}"
        );
    }
    for value in [
        "123456789abcdef",
        "123456789abcdef01",
        "0x123456789abcde",
        "123456789abcdegg",
        "123456789abcdeé",
    ] {
        assert!(
            fmr_options::parse(&with("--serial", value)).is_err(),
            "{value}"
        );
    }
}

#[test]
fn extra_arguments_and_other_operation_flags_are_refused() {
    for value in [
        "--help",
        "-h",
        "--yes",
        "--address",
        "--count",
        "--cdb",
        "--firmware",
        "--reader=foo",
        "restore",
        "extra.bin",
    ] {
        let mut args = valid();
        args.push(value.into());
        assert!(fmr_options::parse(&args).is_err(), "{value}");
    }
}
