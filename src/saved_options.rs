//! Strict file-only command grammar. Shared by the canonical offline harness.
use std::path::PathBuf;
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub command: String,
    pub input: PathBuf,
    pub package: bool,
    pub compare: Option<PathBuf>,
    pub receipt: Option<PathBuf>,
    pub expected_size: Option<usize>,
    pub sha256: Option<String>,
    pub support: bool,
}
pub const HELP: &str = "File-only (no device access):
  inspect FILE [--package] [--compare FILE] [--receipt FILE]
  verify FILE [--package] [--compare FILE] [--receipt FILE]
  dry-plan SAVED_RAW PACKAGE.fwsc
Options: --expected-size BYTES --sha256 HEX --support
verify requires --sha256, --receipt, or --compare; mismatches exit 1.
--support emits only sanitized JSON. Default also prints the private local SHA-256.
A dry plan is advisory; it cannot be executed and proves no device state.";
pub fn parse(args: &[String]) -> Result<Options, &'static str> {
    let mut it = args.iter();
    let command = it.next().ok_or("missing-command")?.clone();
    if !matches!(command.as_str(), "inspect" | "verify" | "dry-plan") { return Err("invalid-command"); }
    let input = it.next().filter(|s| !s.starts_with('-')).ok_or("missing-input")?.into();
    let mut options = Options { command, input, package: false, compare: None, receipt: None,
        expected_size: None, sha256: None, support: false };
    if options.command == "dry-plan" {
        options.compare = Some(it.next().filter(|s| !s.starts_with('-')).ok_or("missing-package")?.into());
    }
    // Parse a second iterator to keep values and duplicate handling explicit.
    let start = if options.command == "dry-plan" { 3 } else { 2 };
    let mut index = start;
    let mut seen = std::collections::BTreeSet::new();
    while index < args.len() {
        let flag = args[index].as_str();
        if !seen.insert(flag) { return Err("duplicate-option"); }
        index += 1;
        if flag == "--support" { options.support = true; continue; }
        if options.command == "dry-plan" { return Err("invalid-dry-plan-option"); }
        if flag == "--package" { options.package = true; continue; }
        let value = args.get(index).filter(|s| !s.starts_with('-')).ok_or("missing-option-value")?;
        index += 1;
        match flag {
            "--compare" => options.compare = Some(value.into()),
            "--receipt" => options.receipt = Some(value.into()),
            "--expected-size" => {
                let n = value.parse::<usize>().map_err(|_| "invalid-expected-size")?;
                if n == 0 || n > crate::saved_analysis::MAX_INPUT { return Err("invalid-expected-size"); }
                options.expected_size = Some(n);
            }
            "--sha256" => {
                if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) { return Err("invalid-sha256"); }
                options.sha256 = Some(value.to_ascii_lowercase());
            }
            _ => return Err("unknown-option"),
        }
    }
    if options.package && options.receipt.is_some() { return Err("raw-receipt-requires-raw"); }
    if options.command == "verify" && options.compare.is_none() && options.receipt.is_none() && options.sha256.is_none() {
        return Err("verification-needs-reference");
    }
    Ok(options)
}
