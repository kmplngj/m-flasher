// SPDX-License-Identifier: MIT
//! Felucca device-pull OTA protocol. Immutable input; injected I/O and clock.
//! Protocol reference: kmplngj/Felucca firmware/src/ota.c and loader/ldr_core.c.
use crate::formats::{Package, crc16, enc};
use std::time::Duration;

pub type Result<T> = std::result::Result<T, String>;
pub const QUERY: &[u8] = &[0xf0, 0, 0x32, 0x45, 0, 0, 0, 0x40, 0x7f, 0xf7];
pub const START: &[u8] = &[0xf0, 0x22, 0x24, 0x35, 0x7f, 0xf7];
pub const CHECK_DONE: u32 = 0xe0000000;
pub const WRITE_DONE: u32 = 0xf0000000;
pub const MAX_PACKAGE: usize = 2 * 1024 * 1024;

fn le16(b: &[u8], p: usize) -> u16 {
    u16::from_le_bytes(b[p..p + 2].try_into().unwrap())
}
fn le32(b: &[u8], p: usize) -> u32 {
    u32::from_le_bytes(b[p..p + 4].try_into().unwrap())
}

pub struct Image {
    pub identity: String,
    bytes: Vec<u8>,
}
impl Image {
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_PACKAGE {
            return Err("package exceeds 2 MiB".into());
        }
        let package = Package::parse(raw).map_err(|e| e.to_string())?;
        if !valid_identity(&package.identity) || !package.identity.starts_with("FM-1_") {
            return Err("MIDI OTA accepts FM-1 Felucca packages only".into());
        }
        let mut bytes = Vec::with_capacity(raw.len() - 20);
        for block in raw[..20 * 48].chunks_exact(48) {
            bytes.extend_from_slice(&block[..47]);
        }
        bytes.extend_from_slice(&raw[20 * 48..]);
        let header = enc(&bytes[..64], 0xffff);
        let count = le16(&header, 8) as usize;
        if !(1..=11).contains(&count)
            || bytes.len() < 1024
            || le32(&header, 4) as usize != bytes.len()
            || crc16(&header[2..]) != le16(&header, 0)
        {
            return Err("unsupported OTA header/marker layout".into());
        }
        let mut types = std::collections::BTreeSet::new();
        for entry in bytes[64..64 + 80 * count].chunks_exact(80) {
            let decoded = enc(entry, 0xffff);
            if !types.insert(le16(&decoded, 0)) {
                return Err("duplicate OTA file type".into());
            }
        }
        if package.flash.len() != 0x93000 {
            return Err("FM-1 OTA flash.bin must be exactly 0x93000 bytes".into());
        }
        let loader = package
            .files
            .get("ota.bin")
            .filter(|(t, _, _)| *t == 100)
            .map(|(_, _, b)| b)
            .ok_or("missing type-100 ota.bin")?;
        if loader.len() < 512 {
            return Err("truncated OTA loader".into());
        }
        let body = le32(loader, 8) as usize;
        if body > 0x4e00
            || body < 32
            || loader.len() != body + 32
            || le32(loader, 4) != 32
            || loader[12] != 0x41
            || crc16(&loader[2..32]) != le16(loader, 0)
            || crc16(&loader[34..64]) != le16(loader, 32)
            || le32(loader, 40) != 0x01c0a800
            || crc16(&loader[32..]) != le16(loader, 2)
            || le32(loader, 36) > 0x14000
            || !loader[0x48..0x48 + 64]
                .windows(16)
                .any(|w| w == b"FELUCCA-LOADER-1")
        {
            return Err("unsupported or damaged Felucca OTA loader".into());
        }
        Ok(Self {
            identity: package.identity,
            bytes,
        })
    }

    /// `parse` admits only FELUCCA-LOADER-1 with flash.bin [0, 0x93000).
    /// Its source-defined USB handshake is exactly ota-FM-1_900. The update
    /// record's ota-FM-1_015 text is not that handshake or compatibility proof.
    /// A numeric family (including 9xx) does not establish write semantics.
    fn require_loader(&self, id: &Identity) -> Result<()> {
        if id.0 != "ota-FM-1_900" {
            return Err(format!(
                "incompatible MIDI loader {}: FELUCCA-LOADER-1 with 0x93000-byte flash.bin requires ota-FM-1_900; stopped before loader START/data",
                id.0
            ));
        }
        Ok(())
    }
}

pub fn pack(data: &[u8]) -> Vec<u8> {
    let (mut bits, mut n) = (0u32, 0);
    let mut out = vec![0xf0];
    for &b in data {
        bits |= (b as u32) << n;
        n += 8;
        while n >= 7 {
            out.push((bits & 127) as u8);
            bits >>= 7;
            n -= 7;
        }
    }
    if n > 0 {
        out.push(bits as u8);
    }
    out.push(0xf7);
    out
}
fn unpack(wire: &[u8]) -> Option<Vec<u8>> {
    if wire.len() < 3 || wire.len() > 640 || wire[0] != 0xf0 || *wire.last()? != 0xf7 {
        return None;
    }
    let (mut bits, mut n) = (0u32, 0);
    let mut data = Vec::new();
    for &b in &wire[1..wire.len() - 1] {
        if b > 127 {
            return None;
        }
        bits |= (b as u32) << n;
        n += 7;
        if n >= 8 {
            data.push((bits & 255) as u8);
            bits >>= 8;
            n -= 8;
        }
    }
    // Canonical encoding rejects extra zero groups and nonzero padding.
    if bits != 0 || pack(&data) != wire {
        return None;
    }
    Some(data)
}
fn frame(wire: &[u8], command: u8, len: usize) -> Option<Vec<u8>> {
    let d = unpack(wire)?;
    if d.len() != len
        || d[..3] != [0, 0x59, command]
        || (d[3] as usize | (d[4] as usize) << 8 | (d[5] as usize) << 16) + 7 != len
        || d[6..].iter().fold(0u8, |a, b| a.wrapping_add(*b)) != 255
    {
        return None;
    }
    Some(d)
}
fn valid_identity(s: &str) -> bool {
    // Match the build's explicit application product contract. The loader has
    // its own fixed identity and is never classified by this app suffix.
    if let Some(version) = s.strip_prefix("FM-1_") {
        return (1..=15).contains(&version.len())
            && version.bytes().all(|c| {
                c.is_ascii_uppercase() || c.is_ascii_digit() || b"._-".contains(&c)
            })
            && !s
                .bytes()
                .enumerate()
                .any(|(i, c)| c.wrapping_add(i as u8 + 1) == 0x7d);
    }
    s.rsplit_once('_').is_some_and(|(m, v)| {
        !m.is_empty()
            && !v.is_empty()
            && m.bytes().all(|c| c.is_ascii_graphic() && c != b'_')
            && v.bytes().all(|c| c.is_ascii_digit())
    })
}
#[derive(Clone, Debug, PartialEq)]
pub struct Identity(pub String);
impl Identity {
    pub fn parse(wire: &[u8]) -> Option<Self> {
        let data = frame(wire, 0x11, 34)?;
        let text = std::str::from_utf8(&data[6..33])
            .ok()?
            .trim_end_matches('\0');
        valid_identity(text).then(|| Self(text.into()))
    }
    pub fn loader(&self) -> bool {
        self.0.starts_with("ota-")
    }
    pub fn fm1(&self) -> bool {
        self.0.starts_with("FM-1_") || self.0.starts_with("ota-FM-1_")
    }
}
fn answer(flags: u8, address: u32, payload: &[u8]) -> Vec<u8> {
    let mut d = vec![
        0,
        0x59,
        0x30,
        (payload.len() + 8) as u8,
        ((payload.len() + 8) >> 8) as u8,
        0,
        flags,
    ];
    d.extend_from_slice(&address.to_le_bytes());
    d.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    d.push(0);
    d.extend_from_slice(payload);
    let sum = d[6..].iter().fold(0u8, |a, b| a.wrapping_add(*b));
    d.push(!sum);
    pack(&d)
}

#[derive(Clone, Default)]
pub struct Selection {
    pub port: Option<String>,
    pub input: Option<String>,
    pub output: Option<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Pair {
    pub input: String,
    pub output: String,
}
fn base(name: &str) -> &str {
    let name = name.trim_end();
    match name.rsplit_once(char::is_whitespace) {
        Some((stem, instance))
            if !instance.is_empty() && instance.bytes().all(|b| b.is_ascii_digit()) =>
        {
            stem.trim_end()
        }
        _ => name,
    }
}
fn token(name: &str, model: &str) -> bool {
    name.match_indices(model).any(|(at, _)| {
        name[..at]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric())
            && name[at + model.len()..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric())
    })
}
/// Select exactly one explicit pair. Never pick the first of multiple units.
pub fn pair(inputs: &[String], outputs: &[String], s: &Selection) -> Result<Pair> {
    let candidates: Vec<_> = inputs
        .iter()
        .filter(|n| {
            if let Some(input) = &s.input {
                return *n == input;
            }
            if let Some(port) = &s.port {
                return n.to_lowercase().contains(&port.to_lowercase());
            }
            let lower = n.to_lowercase();
            token(&lower, "fm-1") || token(&lower, "felucca")
        })
        .collect();
    if candidates.len() != 1 {
        return Err("need one FM-1 MIDI input; use --port or --in-port and --out-port".into());
    }
    let input = candidates[0];
    let exact_output = outputs.iter().any(|n| n == input);
    let outs: Vec<_> = outputs
        .iter()
        .filter(|n| match &s.output {
            Some(output) => *n == output,
            None if s.port.is_some() => n
                .to_lowercase()
                .contains(&s.port.as_ref().unwrap().to_lowercase()),
            None if exact_output => *n == input,
            None => base(n) == base(input),
        })
        .collect();
    if outs.len() != 1 {
        return Err("cannot pair MIDI endpoints uniquely; use --in-port and --out-port".into());
    }
    Ok(Pair {
        input: input.clone(),
        output: outs[0].clone(),
    })
}

pub trait Io {
    fn now(&self) -> Duration;
    fn pause(&mut self, duration: Duration);
    fn connect(&mut self, selection: &Selection) -> Result<()>;
    fn disconnect(&mut self);
    fn send(&mut self, wire: &[u8]) -> Result<()>;
    fn receive(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>>;
}
fn seconds(s: u64) -> Duration {
    Duration::from_secs(s)
}
fn identify(io: &mut impl Io) -> Result<Identity> {
    io.pause(Duration::from_millis(300));
    for _ in 0..2 {
        io.send(QUERY)?;
        let deadline = io.now() + seconds(1);
        while io.now() < deadline {
            if let Some(wire) = io.receive(deadline.saturating_sub(io.now()))? {
                if let Some(id) = Identity::parse(&wire) {
                    return Ok(id);
                }
            }
        }
    }
    Err("FM-1 did not answer the MIDI identity query".into())
}
pub fn open(io: &mut impl Io, s: &Selection) -> Result<Identity> {
    io.connect(s)?;
    let id = identify(io)?;
    if !id.fm1() {
        return Err("MIDI device is not an FM-1".into());
    }
    Ok(id)
}
fn reconnect(io: &mut impl Io, s: &Selection, loader: bool, timeout: u64) -> Result<Identity> {
    let deadline = io.now() + seconds(timeout);
    // Keep explicit endpoint constraints during re-enumeration as well.
    while io.now() < deadline {
        io.disconnect();
        if let Ok(id) = open(io, s) {
            if id.loader() == loader {
                return Ok(id);
            }
        }
        io.disconnect();
        io.pause(seconds(1));
    }
    Err(if loader {
        "update loader did not appear; rerun to finish the update"
    } else {
        "firmware did not return after update; inspect the device before retrying"
    }
    .into())
}
fn transfer(
    io: &mut impl Io,
    image: &Image,
    finish: u32,
    idle: u64,
    report: &mut impl FnMut(&str),
) -> Result<()> {
    io.send(START)?;
    io.pause(seconds(2));
    let (mut active, total) = (io.now(), io.now() + seconds(1200));
    let mut requests = 0;
    while io.now() < total && io.now().saturating_sub(active) < seconds(idle) {
        let Some(wire) = io.receive(seconds(1))? else {
            continue;
        };
        let Some(d) = frame(&wire, 0x30, 15) else {
            continue;
        };
        let address = le32(&d, 7);
        let len = d[11] as usize | (d[12] as usize) << 8 | (d[13] as usize) << 16;
        if address == CHECK_DONE || address == WRITE_DONE {
            if address != finish {
                return Err("unexpected OTA phase completion; stopped".into());
            }
            io.send(&answer(0, address, b"success\0"))?;
            return Ok(());
        }
        let start = address as usize;
        if len == 0
            || len > 512
            || start
                .checked_add(len)
                .is_none_or(|end| end > image.bytes.len())
        {
            return Err("device requested bytes outside the immutable package".into());
        }
        io.pause(Duration::from_millis(10));
        io.send(&answer(d[6], address, &image.bytes[start..start + len]))?;
        active = io.now();
        requests += 1;
        if requests % 128 == 0 {
            report(&format!("served {requests} package requests"));
        }
    }
    Err("OTA transfer timed out; loader staging or writing may have begun; rerun the same package to finish".into())
}
/// Caller obtains confirmation before invoking this method. No backup is implied.
pub fn install(
    io: &mut impl Io,
    s: &Selection,
    image: &Image,
    id: &Identity,
    report: &mut impl FnMut(&str),
) -> Result<()> {
    if !id.fm1() {
        return Err("MIDI device is not an FM-1".into());
    }
    // A device can refuse an unchanged numeric product version. Do not rewrite
    // identities, detour through stock, or claim successful replacement.
    // Felucca permits same-version packages; its display identity alone cannot
    // distinguish PR commits, so the final identity check is deliberately limited.
    let loader_id = if id.loader() {
        id.clone()
    } else {
        report("checking package and staging the update loader");
        transfer(io, image, CHECK_DONE, 8, report)?;
        // CoreMIDI queues sends: keep both ports alive until the final reply drains.
        io.pause(seconds(3));
        io.disconnect();
        reconnect(io, s, true, 30)?
    };
    // Gate both already-running and newly staged loaders before START or any
    // package response. Staging may already have written the update records.
    image.require_loader(&loader_id)?;
    report("writing firmware through its MIDI loader");
    transfer(io, image, WRITE_DONE, 180, report)?;
    io.pause(seconds(3));
    io.disconnect();
    let final_id = reconnect(io, s, false, 40)?;
    if final_id.0 != image.identity {
        return Err(format!(
            "device returned {}, expected {}",
            final_id.0, image.identity
        ));
    }
    if id.0 == image.identity {
        report(&format!(
            "OTA exchange completed; device reports {} as before. Identity cannot confirm replacement; verify the ABOUT build commit against the package provenance.",
            final_id.0
        ));
    } else {
        report(&format!(
            "done: {} (retain the package provenance to identify the PR commit)",
            final_id.0
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../tests/support/midi.rs"]
mod tests;
