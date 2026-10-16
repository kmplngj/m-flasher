//! File- and device-inert arguments for the fixed two-read FMR1 check.

use std::path::PathBuf;

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
    pub output: PathBuf,
}

/// Parse arguments after `reader-check`; this never accesses paths or devices.
pub fn parse(args: &[String]) -> Result<Command, String> {
    let (mut mode, mut reader, mut device, mut location, mut serial, mut output) =
        (None, None, None, None, None, None);
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        if matches!(flag.as_str(), "--prepare" | "--run") {
            if mode.replace(flag == "--run").is_some() {
                return Err("choose exactly one of --prepare or --run".into());
            }
            continue;
        }
        let slot = match flag.as_str() {
            "--reader" => &mut reader,
            "--device" => &mut device,
            "--location" => &mut location,
            "--serial" => &mut serial,
            "--output" => &mut output,
            _ => return Err(format!("unknown reader-check argument: {flag}")),
        };
        if slot.is_some() {
            return Err(format!("duplicate argument: {flag}"));
        }
        let value = args
            .next()
            .filter(|v| !v.is_empty() && !v.starts_with("--"))
            .ok_or_else(|| format!("{flag} needs a nonempty value"))?;
        *slot = Some(value.clone());
    }
    let run = mode.ok_or("choose exactly one of --prepare or --run")?;
    let reader = PathBuf::from(reader.ok_or("missing --reader")?);
    let device = device.ok_or("missing --device")?;
    let canonical_u8 = |part: &str| part.parse::<u8>().is_ok_and(|n| n.to_string() == part);
    let valid_device = device
        .strip_prefix("usb:")
        .and_then(|s| s.split_once('-'))
        .is_some_and(|(bus, address)| canonical_u8(bus) && canonical_u8(address));
    if !valid_device {
        return Err("--device must be canonical usb:<u8>-<u8>".into());
    }
    let location = location.ok_or("missing --location")?;
    if location.len() != 8 || !location.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("--location must be eight hexadecimal digits".into());
    }
    let location = u32::from_str_radix(&location, 16).map_err(|_| "invalid --location")?;
    if location == 0 {
        return Err("--location must be nonzero".into());
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
        output: PathBuf::from(output.ok_or("missing --output")?),
    };
    Ok(if run {
        Command::Run(options)
    } else {
        Command::Prepare(options)
    })
}
