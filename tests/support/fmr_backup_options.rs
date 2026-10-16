//! Pure CLI fixtures; all named paths are deliberately nonexistent.
use crate::fmr_backup_options::{self, Command};

fn valid() -> Vec<String> {
    [
        "--prepare",
        "--reader",
        "/nonexistent/reader.bin",
        "--device",
        "usb:3-7",
        "--location",
        "03110000",
        "--serial",
        "0123456789abcdef",
        "--copy-a",
        "/nonexistent/a",
        "--copy-b",
        "/nonexistent/b",
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
fn modes_and_explicit_same_filesystem_policy_are_pure() {
    let Command::Prepare(options) = fmr_backup_options::parse(&valid()).unwrap() else {
        panic!()
    };
    assert_eq!(options.location, 0x03110000);
    assert_eq!(options.serial, "0123456789ABCDEF");
    assert!(!options.allow_same_filesystem);
    let mut args = valid();
    args[0] = "--run".into();
    assert_eq!(fmr_backup_options::parse(&args), Ok(Command::Run(options)));
    args.push("--allow-same-filesystem".into());
    let Command::Run(options) = fmr_backup_options::parse(&args).unwrap() else {
        panic!()
    };
    assert!(options.allow_same_filesystem);
    args.push("--allow-same-filesystem".into());
    assert!(fmr_backup_options::parse(&args).is_err());
}
#[test]
fn missing_duplicate_unknown_and_alias_flags_are_refused() {
    assert!(fmr_backup_options::parse(&[]).is_err());
    for flag in [
        "--reader",
        "--device",
        "--location",
        "--serial",
        "--copy-a",
        "--copy-b",
    ] {
        let mut args = valid();
        let at = args.iter().position(|v| v == flag).unwrap();
        args.drain(at..at + 2);
        assert!(fmr_backup_options::parse(&args).is_err(), "missing {flag}");
        assert!(
            fmr_backup_options::parse(&with(flag, "")).is_err(),
            "empty {flag}"
        );
        let mut args = valid();
        args.extend([flag.into(), "duplicate".into()]);
        assert!(
            fmr_backup_options::parse(&args).is_err(),
            "duplicate {flag}"
        );
    }
    let mut args = valid();
    args.remove(0);
    assert!(fmr_backup_options::parse(&args).is_err());
    for flag in [
        "--prepare",
        "--run",
        "--yes",
        "--output",
        "--firmware",
        "--copy-a=x",
        "--allow-same-filesystem=true",
        "raw-backup",
        "extra",
    ] {
        let mut args = valid();
        args.push(flag.into());
        assert!(fmr_backup_options::parse(&args).is_err(), "{flag}");
    }
}
#[test]
fn selectors_require_canonical_matching_identity() {
    for value in [
        "usb:03-7",
        "usb:3-07",
        "usb:+3-7",
        "usb:3-+7",
        "usb:3-0",
        "usb:3-128",
        "usb:256-7",
        "usb:3-7-8",
        "/dev/disk7",
        "usb:３-7",
    ] {
        assert!(
            fmr_backup_options::parse(&with("--device", value)).is_err(),
            "{value}"
        );
    }
    for value in [
        "00000000",
        "04110000",
        "0311000",
        "031100000",
        "0x03110000",
        "0311000g",
    ] {
        assert!(
            fmr_backup_options::parse(&with("--location", value)).is_err(),
            "{value}"
        );
    }
    for value in [
        "0123456789abcde",
        "0123456789abcdef0",
        "0123456789abcdeg",
        "0123456789abcdeé",
    ] {
        assert!(
            fmr_backup_options::parse(&with("--serial", value)).is_err(),
            "{value}"
        );
    }
    for (device, location) in [("usb:0-1", "00110000"), ("usb:255-127", "ff110000")] {
        let mut args = with("--device", device);
        let at = args.iter().position(|v| v == "--location").unwrap();
        args[at + 1] = location.into();
        assert!(fmr_backup_options::parse(&args).is_ok());
    }
}
#[test]
fn path_spelling_and_lexical_aliases_are_refused_without_io() {
    for flag in ["--reader", "--copy-a", "--copy-b"] {
        for value in [
            "relative",
            ".",
            "/",
            "/nonexistent/",
            "/nonexistent/.",
            "/nonexistent/../a",
            "/nonexistent/./a",
            "/nonexistent/a\0x",
        ] {
            assert!(
                fmr_backup_options::parse(&with(flag, value)).is_err(),
                "{flag}: {value:?}"
            );
        }
    }
    for (flag, value) in [
        ("--copy-b", "/nonexistent/a"),
        ("--copy-a", "/nonexistent/reader.bin"),
        ("--copy-b", "/nonexistent/reader.bin"),
        ("--copy-b", "/nonexistent//a"),
    ] {
        assert!(fmr_backup_options::parse(&with(flag, value)).is_err());
    }
}
