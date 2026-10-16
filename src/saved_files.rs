//! Bounded, read-only regular-file input. Errors deliberately omit local paths.
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
pub fn read(path: &Path, limit: usize) -> Result<Vec<u8>, &'static str> {
    let before = fs::symlink_metadata(path).map_err(|_| "input-unavailable")?;
    if !before.is_file() || before.file_type().is_symlink() { return Err("input-must-be-regular-file"); }
    if before.len() == 0 || before.len() > limit as u64 { return Err("invalid-input-size"); }
    let file = File::open(path).map_err(|_| "input-unavailable")?;
    let opened = file.metadata().map_err(|_| "input-unavailable")?;
    if !opened.is_file() || opened.len() != before.len() { return Err("input-changed"); }
    #[cfg(unix)] {
        use std::os::unix::fs::MetadataExt;
        if (before.dev(), before.ino()) != (opened.dev(), opened.ino()) { return Err("input-changed"); }
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes).map_err(|_| "input-read-failed")?;
    if bytes.len() as u64 != before.len() || bytes.len() > limit { return Err("input-changed"); }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_only_bounded_regular_files_and_never_creates_output() {
        let directory = std::env::temp_dir().join(format!("saved-read-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("private-input");
        fs::write(&path, b"private").unwrap();
        assert_eq!(read(&path, 7).unwrap(), b"private");
        assert_eq!(read(&path, 6), Err("invalid-input-size"));
        assert_eq!(read(&directory, 100), Err("input-must-be-regular-file"));
        assert_eq!(read(&directory.join("missing-private-selector"), 100), Err("input-unavailable"));
        #[cfg(unix)] {
            let link = directory.join("link");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert_eq!(read(&link, 100), Err("input-must-be-regular-file"));
            fs::remove_file(link).unwrap();
        }
        File::create(&path).unwrap().set_len(1u64 << 40).unwrap();
        assert_eq!(read(&path, 100), Err("invalid-input-size"));
        fs::write(&path, []).unwrap();
        assert_eq!(read(&path, 100), Err("invalid-input-size"));
        fs::remove_file(path).unwrap(); fs::remove_dir(directory).unwrap();
    }
}
