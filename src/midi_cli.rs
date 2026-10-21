// SPDX-License-Identifier: MIT
//! Explicit MIDI dispatch. Help/capability/file validation never opens an adapter.
use crate::midi_ota::{self as midi, Image, Selection};
use std::io::Read;
#[cfg(all(feature = "midi", not(any(test, feature = "offline-tests"))))]
use std::io::Write;

const USAGE: &str = "m-flasher --capabilities\nm-flasher midi check PACKAGE.fwsc (offline)\nm-flasher midi info [--port NAME | --in-port INPUT --out-port OUTPUT]\nm-flasher midi firmware PACKAGE.fwsc [--port NAME | --in-port INPUT --out-port OUTPUT] [--yes]\nMIDI OTA uses the running FM-1 and its package loader, not ROM USB. No raw backup or rollback checkpoint is created.";

fn options(args: &[String]) -> midi::Result<(String, Option<String>, Selection, bool)> {
    let mut s = Selection::default();
    let mut yes = false;
    let mut pos = vec![];
    let mut args = args.iter();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--port" | "--in-port" | "--out-port" => {
                let value = args
                    .next()
                    .filter(|v| !v.is_empty() && !v.starts_with('-'))
                    .ok_or("port option requires a nonempty name")?
                    .clone();
                let dest = match a.as_str() {
                    "--port" => &mut s.port,
                    "--in-port" => &mut s.input,
                    _ => &mut s.output,
                };
                if dest.replace(value).is_some() {
                    return Err("duplicate port option".into());
                }
            }
            "--yes" => yes = true,
            a if a.starts_with('-') => return Err(format!("unknown MIDI option {a}")),
            _ => pos.push(a.clone()),
        }
    }
    if s.port.is_some() && (s.input.is_some() || s.output.is_some())
        || s.input.is_some() != s.output.is_some()
    {
        return Err("select --port OR both --in-port and --out-port".into());
    }
    match pos.as_slice() {
        [m, command] if m == "midi" && command == "info" => Ok((command.clone(), None, s, yes)),
        [m, command, file] if m == "midi" && matches!(command.as_str(), "firmware" | "check") => {
            Ok((command.clone(), Some(file.clone()), s, yes))
        }
        _ => Err(USAGE.into()),
    }
}
fn image(path: &str) -> midi::Result<Image> {
    let file = std::fs::File::open(path).map_err(|e| format!("cannot open package: {e}"))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("package must be a regular file".into());
    }
    let mut raw = vec![];
    file.take(midi::MAX_PACKAGE as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    Image::parse(&raw)
}
fn execute(args: &[String]) -> midi::Result<()> {
    let (command, file, selection, yes) = options(args)?;
    // Package validation deliberately precedes any native backend construction.
    let package = file.as_deref().map(image).transpose()?;
    if command == "check" {
        println!(
            "verified MIDI OTA package: {}",
            package.as_ref().ok_or("missing package")?.identity
        );
        return Ok(());
    }
    #[cfg(all(feature = "midi", not(any(test, feature = "offline-tests"))))]
    {
        use midi::Io;
        let mut io = crate::midi_native::Native::new();
        let result = (|| {
            let id = midi::open(&mut io, &selection)?;
            println!("device: {}", id.0);
            if let Some(package) = package {
                println!(
                    "package: {}\nMIDI OTA stages an update loader and writes firmware; no raw backup is created.",
                    package.identity
                );
                if !yes {
                    print!("Install? Do not unplug while writing. [y/N] ");
                    std::io::stdout().flush().map_err(|e| e.to_string())?;
                    let mut answer = String::new();
                    std::io::stdin()
                        .read_line(&mut answer)
                        .map_err(|e| e.to_string())?;
                    if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
                        return Err("cancelled; update not started".into());
                    }
                }
                midi::install(&mut io, &selection, &package, &id, &mut |s| println!("{s}"))?;
            }
            Ok(())
        })();
        io.disconnect();
        result
    }
    #[cfg(not(all(feature = "midi", not(any(test, feature = "offline-tests")))))]
    {
        let _ = (package, selection, yes);
        Err("native MIDI unavailable in this build (tests/offline builds deny access)".into())
    }
}
pub fn run(args: &[String]) -> i32 {
    if args == ["--capabilities"] {
        println!(
            "{{\"schema\":1,\"midi_ota\":{},\"protocol\":\"felucca-device-pull-v1\"}}",
            cfg!(all(
                feature = "midi",
                not(any(test, feature = "offline-tests"))
            ))
        );
        return 0;
    }
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return 0;
    }
    match execute(args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn midi_native_entrypoint_is_denied_offline() {
        assert!(
            execute(&["midi".into(), "info".into()])
                .unwrap_err()
                .contains("native MIDI unavailable")
        );
    }
    #[test]
    fn malformed_options_and_packages_fail_before_adapter_access() {
        for args in [
            vec!["midi", "info", "--port"],
            vec!["midi", "info", "--in-port", "FM-1"],
            vec!["midi", "firmware", "absent.fwsc"],
            vec!["midi", "info", "--with-bootloader"],
        ] {
            assert!(execute(&args.into_iter().map(str::to_owned).collect::<Vec<_>>()).is_err());
        }
    }
}
