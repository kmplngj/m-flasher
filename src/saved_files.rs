//! Bounded, read-only regular-file input. Errors deliberately omit local paths.
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::Path;
// Host ABI constants are kept here so the canonical std-only harness need not
// resolve/link libc. The Cargo tests assert they match the supported host libc.
#[cfg(target_os = "linux")]
const READ_FLAGS: i32 = 0x20000 | 0x800; // O_NOFOLLOW | O_NONBLOCK
#[cfg(target_os = "macos")]
const READ_FLAGS: i32 = 0x100 | 0x4; // O_NOFOLLOW | O_NONBLOCK

pub fn read(path: &Path, limit: usize) -> Result<Vec<u8>, &'static str> {
    read_with_hook(path, limit, || {})
}

fn read_with_hook(path: &Path, limit: usize, before_open: impl FnOnce()) -> Result<Vec<u8>, &'static str> {
    let before = fs::symlink_metadata(path).map_err(|_| "input-unavailable")?;
    if !before.is_file() || before.file_type().is_symlink() { return Err("input-must-be-regular-file"); }
    if before.len() == 0 || before.len() > limit as u64 { return Err("invalid-input-size"); }
    before_open();
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(any(target_os = "linux", target_os = "macos"))] {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(READ_FLAGS);
    }
    #[cfg(windows)] {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x00200000); // FILE_FLAG_OPEN_REPARSE_POINT
        options.share_mode(1); // FILE_SHARE_READ: deny concurrent write/delete
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    return Err("unsupported-file-platform");
    let file = options.open(path).map_err(|_| "input-unavailable")?;
    let opened = file.metadata().map_err(|_| "input-unavailable")?;
    if !opened.is_file() || opened.file_type().is_symlink() || opened.len() != before.len() { return Err("input-changed"); }
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
        File::create(&path).unwrap().set_len(1024).unwrap();
        assert_eq!(read(&path, 100), Err("invalid-input-size"));
        fs::write(&path, []).unwrap();
        assert_eq!(read(&path, 100), Err("invalid-input-size"));
        fs::remove_file(path).unwrap(); fs::remove_dir(directory).unwrap();
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod replacement_tests {
    use super::*;
    #[test]
    fn substitution_before_open_refuses_symlink_and_fifo_without_following_or_blocking() {
        let directory = std::env::temp_dir().join(format!("saved-substitute-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&directory).unwrap();
        let input = directory.join("input");
        let original = directory.join("original");
        let secret = directory.join("secret");
        fs::write(&secret, b"secret").unwrap();
        for fifo in [false, true] {
            fs::write(&input, b"public").unwrap();
            let started = std::time::Instant::now();
            let result = read_with_hook(&input, 100, || {
                fs::rename(&input, &original).unwrap();
                if fifo {
                    // Creates a local filesystem FIFO, never a device. Without
                    // O_NONBLOCK a read-only open here would wait for a writer.
                    assert!(std::process::Command::new("mkfifo").arg(&input).status().unwrap().success());
                } else {
                    std::os::unix::fs::symlink(&secret, &input).unwrap();
                }
            });
            assert!(result.is_err());
            assert!(started.elapsed() < std::time::Duration::from_secs(5));
            assert_eq!(fs::read(&secret).unwrap(), b"secret");
            fs::remove_file(&input).unwrap(); fs::remove_file(&original).unwrap();
        }
        fs::remove_file(secret).unwrap(); fs::remove_dir(directory).unwrap();
    }
}
