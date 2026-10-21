//! Pure arguments for the fixed full raw capture; parsing performs no I/O.

use std::path::{Component, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Prepare(Options),
    Run(Options),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub reader: PathBuf,
    pub device: String,
    pub location: u32,
    pub serial: String,
    pub copy_a: PathBuf,
    pub copy_b: PathBuf,
    pub allow_same_filesystem: bool,
}

fn absolute_path(value: String, flag: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(&value);
    // Inspect spelling too: Path::components normalizes interior `.` and a
    // terminal separator, which would hide ambiguous destination names.
    if !path.is_absolute()
        || value.contains('\0')
        || value.ends_with(['/', '\\'])
        || value
            .split(['/', '\\'])
            .any(|part| matches!(part, "." | ".."))
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
        || path.file_name().is_none()
    {
        return Err(format!(
            "{flag} requires an absolute path with a plain terminal name and no dot components"
        ));
    }
    Ok(path)
}

/// Parse arguments after `raw-backup`. File existence and filesystem identity
/// are checked by storage preflight, never by this parser.
pub fn parse(args: &[String]) -> Result<Command, String> {
    let (mut mode, mut reader, mut device, mut location, mut serial, mut copy_a, mut copy_b) =
        (None, None, None, None, None, None, None);
    let mut allow_same_filesystem = false;
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        if matches!(flag.as_str(), "--prepare" | "--run") {
            if mode.replace(flag == "--run").is_some() {
                return Err("choose exactly one of --prepare or --run".into());
            }
            continue;
        }
        if flag == "--allow-same-filesystem" {
            if allow_same_filesystem {
                return Err("duplicate argument: --allow-same-filesystem".into());
            }
            allow_same_filesystem = true;
            continue;
        }
        let slot = match flag.as_str() {
            "--reader" => &mut reader,
            "--device" => &mut device,
            "--location" => &mut location,
            "--serial" => &mut serial,
            "--copy-a" => &mut copy_a,
            "--copy-b" => &mut copy_b,
            _ => return Err(format!("unknown raw-backup argument: {flag}")),
        };
        if slot.is_some() {
            return Err(format!("duplicate argument: {flag}"));
        }
        *slot = Some(
            args.next()
                .filter(|v| !v.is_empty() && !v.starts_with("--"))
                .ok_or_else(|| format!("{flag} needs a nonempty value"))?
                .clone(),
        );
    }
    let run = mode.ok_or("choose exactly one of --prepare or --run")?;
    let reader = absolute_path(reader.ok_or("missing --reader")?, "--reader")?;
    let copy_a = absolute_path(copy_a.ok_or("missing --copy-a")?, "--copy-a")?;
    let copy_b = absolute_path(copy_b.ok_or("missing --copy-b")?, "--copy-b")?;
    if copy_a == copy_b || copy_a == reader || copy_b == reader {
        return Err("backup destinations and reader must have distinct paths".into());
    }
    let device = device.ok_or("missing --device")?;
    let (bus, address) = device
        .strip_prefix("usb:")
        .and_then(|v| v.split_once('-'))
        .ok_or("--device must be canonical usb:BUS-ADDRESS")?;
    let bus: u8 = bus.parse().map_err(|_| "invalid USB bus")?;
    let address: u8 = address.parse().map_err(|_| "invalid USB address")?;
    if address == 0 || address > 127 || device != format!("usb:{bus}-{address}") {
        return Err("--device must be canonical usb:BUS-ADDRESS with address 1..127".into());
    }
    let location = location.ok_or("missing --location")?;
    if location.len() != 8 || !location.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("--location must be eight hexadecimal digits".into());
    }
    let location = u32::from_str_radix(&location, 16).map_err(|_| "invalid --location")?;
    if location == 0 || (location >> 24) as u8 != bus {
        return Err("--location must be nonzero and match the selected USB bus".into());
    }
    let serial = serial.ok_or("missing --serial")?;
    if serial.len() != 16 || !serial.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("--serial must be sixteen hexadecimal digits".into());
    }
    let options = Options {
        reader,
        device,
        location,
        serial: serial.to_ascii_uppercase(),
        copy_a,
        copy_b,
        allow_same_filesystem,
    };
    Ok(if run {
        Command::Run(options)
    } else {
        Command::Prepare(options)
    })
}
