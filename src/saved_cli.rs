//! Offline commands dispatch before any legacy hardware setup.
use crate::boards::{CHIP_KEY, FLASH_SIZE};
use crate::saved_analysis::{MAX_INPUT, RecoveryState, SavedAnalysis};
use crate::saved_options::{self, Options};
use crate::saved_receipt::{MAX_RECEIPT, Receipt};
use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

pub fn run(args: &[String]) -> i32 {
    if args.len() == 2 && matches!(args[1].as_str(), "--help" | "-h") {
        println!("{}", saved_options::HELP); return 0;
    }
    match saved_options::parse(args).and_then(|options| execute(&options)) {
        Ok((output, matches)) => { println!("{output}"); if matches { 0 } else { 1 } }
        Err(code) => { eprintln!("{{\"schema\":\"saved-diagnostics-v1\",\"error\":\"{code}\"}}"); 1 }
    }
}

pub fn execute(options: &Options) -> Result<(String, bool), &'static str> {
    let bytes = crate::saved_files::read(&options.input, MAX_INPUT)?;
    let sha256 = sha256(&bytes);
    let (saved, representation) = if options.package {
        (SavedAnalysis::package(&bytes)?.1, "package")
    } else { (SavedAnalysis::raw(&bytes)?, "raw") };
    let mut matches = true;
    let mut comparison = "not-requested";
    let mut receipt_result = "not-supplied";
    let mut compatibility = "unknown";
    let mut candidate_sectors = None;
    let mut dry_plan = "not-requested";
    if let Some(path) = &options.compare {
        let other_bytes = crate::saved_files::read(path, MAX_INPUT)?;
        if options.command == "dry-plan" {
            let (_, other) = SavedAnalysis::package(&other_bytes)?;
            compatibility = saved.compatibility(&other);
            // A dry comparison is never an executable transition authorization.
            if bytes.len() != FLASH_SIZE || saved.analysis.board.is_none()
                || !saved.layout_valid || other.recovery != RecoveryState::IntactApp {
                dry_plan = "blocked-unknown-or-corrupt-layout"; matches = false;
            } else if compatibility != "matching-declarations-only" {
                dry_plan = "blocked-unqualified-data-compatibility"; matches = false;
            } else {
                let plan = crate::ops::plan_firmware_saved(&bytes, &saved.analysis,
                    saved.analysis.board.unwrap(), CHIP_KEY, &other_bytes, false, false)
                    .map_err(|_| "dry-plan-refused-by-shared-core")?;
                candidate_sectors = Some(plan.sectors.len());
                dry_plan = "advisory-only-no-execution";
            }
        } else {
            let other = if options.package { SavedAnalysis::package(&other_bytes)?.1 }
                else { SavedAnalysis::raw(&other_bytes)? };
            compatibility = saved.compatibility(&other);
            comparison = if bytes == other_bytes { "equal-saved-bytes" } else { matches = false; "different-saved-bytes" };
        }
    }
    if let Some(path) = &options.receipt {
        let receipt = Receipt::parse(&crate::saved_files::read(path, MAX_RECEIPT)?)?;
        receipt_result = if receipt.bound(bytes.len(), &sha256) { receipt.stage }
            else { matches = false; "receipt-mismatch" };
    }
    if options.expected_size.is_some_and(|size| size != bytes.len()) { matches = false; }
    if options.sha256.as_ref().is_some_and(|hash| hash != &sha256) { matches = false; }
    let integrity = if !matches { "reference-or-policy-check-failed" }
        else if options.sha256.is_some() || options.receipt.is_some() || options.compare.is_some() && options.command != "dry-plan" {
            "local-reference-matched"
        } else { "computed-only-no-authenticated-reference" };
    let mut output = saved.support_json(bytes.len(), representation, comparison, receipt_result, integrity, compatibility);
    output.pop();
    output.push_str(&format!(",\"expected_size_bytes\":{},\"declared_capacity_bytes\":null,\"dry_plan\":\"{dry_plan}\",\"candidate_sectors\":{}",
        options.expected_size.map(|n| n.to_string()).unwrap_or("null".into()),
        candidate_sectors.map(|n| n.to_string()).unwrap_or("null".into())));
    if !options.support { output.push_str(&format!(",\"private_local_sha256\":\"{sha256}\"")); }
    output.push('}');
    Ok((output, matches))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    struct Files(std::path::PathBuf);
    impl Files {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!("saved-cli-{}-{}", std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            fs::create_dir(&dir).unwrap(); Self(dir)
        }
        fn put(&self, name: &str, bytes: &[u8]) -> String {
            let path = self.0.join(name); fs::write(&path, bytes).unwrap(); path.to_str().unwrap().into()
        }
        fn options(&self, command: &str, file: &str, flags: &[&str]) -> Options {
            let mut args = vec![command.into(), file.into()];
            args.extend(flags.iter().map(|s| s.to_string())); saved_options::parse(&args).unwrap()
        }
    }
    impl Drop for Files { fn drop(&mut self) { fs::remove_dir_all(&self.0).unwrap(); } }
    #[test]
    fn actual_dispatcher_inspect_verify_package_and_dry_plan_never_discover_or_open() {
        let files = Files::new();
        let flash = crate::saved_analysis_tests::full();
        let raw = files.put("PRIVATE-SELECTOR.raw", &flash);
        let pkg = files.put("PRIVATE-PRESET.fwsc", include_bytes!("../tests/fixtures/saved/valid.fwsc"));
        for args in [vec!["inspect", &raw, "--support"], vec!["verify", &raw, "--compare", &raw, "--support"],
            vec!["inspect", &pkg, "--package", "--support"], vec!["dry-plan", &raw, &pkg, "--support"]] {
            assert_eq!(crate::cli::run(args.iter().map(|s| s.to_string()).collect()), 0);
        }
        let dry = execute(&files.options("dry-plan", &raw, &[&pkg, "--support"])).unwrap().0;
        assert!(dry.contains("\"candidate_sectors\":0"));
        assert!(dry.contains("advisory-only-no-execution"));
        // Full offline guarded tests compile native adapters out. Their inventory
        // entrypoint panics: reaching it here fails this test immediately.
    }
    #[test]
    fn integrity_references_and_sanitized_failure_results_are_exact() {
        let files = Files::new();
        let mut flash = crate::saved_analysis_tests::full();
        let raw = files.put("PRIVATE-SELECTOR.raw", &flash);
        let hash = sha256(&flash);
        let receipt_text = crate::saved_analysis_tests::receipt(&hash);
        let receipt = files.put("PRIVATE-RECEIPT.json", receipt_text.as_bytes());
        let opts = files.options("verify", &raw, &["--receipt", &receipt, "--sha256", &hash, "--expected-size", "1048576", "--support"]);
        let (report, success) = execute(&opts).unwrap();
        assert!(success);
        assert!(report.contains("held-session-receipt-bound-locally"));
        for secret in ["PRIVATE", "sha256", hash.as_str(), "private key", "provenance", "serial", "offset-oracle"] { assert!(!report.contains(secret)); }
        flash[0xFF001] ^= 1; let other = files.put("other.raw", &flash);
        let (report, success) = execute(&files.options("verify", &other, &["--receipt", &receipt, "--compare", &raw, "--support"])).unwrap();
        assert!(!success); assert!(report.contains("different-saved-bytes")); assert!(report.contains("receipt-mismatch"));
        assert_eq!(crate::cli::run(vec!["verify".into(), other, "--sha256".into(), hash, "--support".into()]), 1);
        let (private, _) = execute(&files.options("inspect", &raw, &[])).unwrap();
        assert!(private.contains("private_local_sha256"));
    }
    #[test]
    fn corrupt_acquisition_can_match_its_reference_without_claiming_bootability() {
        let files = Files::new(); let mut flash = crate::saved_analysis_tests::full(); flash[0x4240] ^= 1;
        let raw = files.put("corrupt.raw", &flash);
        let (report, success) = execute(&files.options("verify", &raw, &["--compare", &raw, "--support"])).unwrap();
        assert!(success); assert!(report.contains("corrupt-or-incomplete-app")); assert!(report.contains("not-demonstrated"));
        let (_, size_ok) = execute(&files.options("inspect", &raw, &["--expected-size", "20480", "--support"])).unwrap(); assert!(!size_ok);
    }
    #[test]
    fn incompatible_and_unknown_dry_plans_stay_blocked() {
        let files = Files::new(); let raw = files.put("saved.raw", &crate::saved_analysis_tests::full());
        let pkg = files.put("incompatible.fwsc", include_bytes!("../tests/fixtures/saved/incompatible.fwsc"));
        let (report, success) = execute(&files.options("dry-plan", &raw, &[&pkg, "--support"])).unwrap();
        assert!(!success); assert!(report.contains("incompatible-data-layout")); assert!(report.contains("blocked-unqualified-data-compatibility"));
        let unknown = files.put("unknown.raw", &vec![0xFF; FLASH_SIZE]);
        let (report, success) = execute(&files.options("dry-plan", &unknown, &[&pkg, "--support"])).unwrap();
        assert!(!success); assert!(report.contains("blocked-unknown-or-corrupt-layout"));
    }
    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn regular_file_open_flags_match_platform_abi() {
        #[cfg(target_os = "linux")] let flags = 0x20000 | 0x800;
        #[cfg(target_os = "macos")] let flags = 0x100 | 0x4;
        assert_eq!(flags, libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[test]
    fn errors_omit_paths_contents_hashes_and_invalid_flags() {
        let files = Files::new(); let bad = files.put("PRIVATE-PATH.fwsc", b"PRIVATE-KEY");
        assert_eq!(execute(&files.options("inspect", &bad, &["--package", "--support"])).unwrap_err(), "invalid-package");
        assert_eq!(crate::cli::run(vec!["inspect".into(), bad, "--device".into(), "SECRET-SELECTOR".into(), "--support".into()]), 1);
    }
}
