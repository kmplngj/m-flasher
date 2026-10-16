//! File-only persistence for one backup and its JSON sidecar.
//!
//! Exclusive creation prevents timestamp collisions from overwriting either
//! existing final name. Both names are reserved before writing any bytes. An
//! error retains partial new files for diagnosis and makes no success claim;
//! callers must not record the backup as saved. This is ordinary local-file
//! storage, not private/ACL storage, hostile-directory protection or two copies.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

pub fn save_pair(raw_path: &Path, raw: &[u8], json: &[u8]) -> io::Result<()> {
    save_with(&mut Filesystem, raw_path, raw, json)
}

trait Io {
    fn create_new(&mut self, path: &Path) -> io::Result<File>;
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> io::Result<()>;
    fn sync(&mut self, file: &File) -> io::Result<()>;
    fn verify(&mut self, path: &Path, expected: &[u8]) -> io::Result<()>;
    fn sync_directory(&mut self, path: &Path) -> io::Result<()>;
}

struct Filesystem;

impl Io for Filesystem {
    fn create_new(&mut self, path: &Path) -> io::Result<File> {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> io::Result<()> {
        file.write_all(bytes)
    }
    fn sync(&mut self, file: &File) -> io::Result<()> {
        file.sync_all()
    }
    fn verify(&mut self, path: &Path, expected: &[u8]) -> io::Result<()> {
        // Bound reads even if the ordinary destination file was changed.
        let mut actual = Vec::with_capacity(expected.len());
        File::open(path)?
            .take(expected.len() as u64 + 1)
            .read_to_end(&mut actual)?;
        if actual != expected {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "reopened bytes differ",
            ));
        }
        Ok(())
    }
    fn sync_directory(&mut self, path: &Path) -> io::Result<()> {
        // std can open/sync a directory on Unix. Windows has no portable std
        // equivalent; sync_all above still requests flushing both file handles.
        #[cfg(unix)]
        File::open(path)?.sync_all()?;
        #[cfg(not(unix))]
        let _ = path;
        Ok(())
    }
}

fn context(error: io::Error, operation: &str, path: &Path) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("{operation} {}: {error}", path.display()),
    )
}

fn save_with(io: &mut impl Io, raw_path: &Path, raw: &[u8], json: &[u8]) -> io::Result<()> {
    let json_path = raw_path.with_extension("json");
    if raw_path == json_path {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "raw and sidecar paths must differ",
        ));
    }
    let mut raw_file = io
        .create_new(raw_path)
        .map_err(|e| context(e, "reserve raw backup", raw_path))?;
    let mut json_file = io
        .create_new(&json_path)
        .map_err(|e| context(e, "reserve JSON sidecar", &json_path))?;
    // Keep both reservations on any error. Deleting by pathname could remove
    // somebody else's replacement, and a retry must not reuse partial output.
    io.write(&mut raw_file, raw)
        .map_err(|e| context(e, "write raw backup", raw_path))?;
    io.sync(&raw_file)
        .map_err(|e| context(e, "sync raw backup", raw_path))?;
    io.write(&mut json_file, json)
        .map_err(|e| context(e, "write JSON sidecar", &json_path))?;
    io.sync(&json_file)
        .map_err(|e| context(e, "sync JSON sidecar", &json_path))?;
    let parent = raw_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    io.sync_directory(parent)
        .map_err(|e| context(e, "sync backup directory", parent))?;
    io.verify(raw_path, raw)
        .map_err(|e| context(e, "verify raw backup", raw_path))?;
    io.verify(&json_path, json)
        .map_err(|e| context(e, "verify JSON sidecar", &json_path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "backup-pair-{}-{nonce}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn raw(&self) -> PathBuf {
            self.0.join("fm1-20261016-123456.bin")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const RAW: &[u8] = b"\x00\x81\xffindependent raw fixture\0";
    const JSON: &[u8] = b"{\"fixture\":\"associated sidecar\"}\n";

    #[test]
    fn same_second_collision_preserves_both_original_files() {
        let fixture = Fixture::new();
        let path = fixture.raw();
        save_pair(&path, RAW, JSON).unwrap();
        let error = save_pair(&path, b"other raw", b"other JSON").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert!(error.to_string().contains("reserve raw backup"));
        assert_eq!(std::fs::read(&path).unwrap(), RAW);
        assert_eq!(std::fs::read(path.with_extension("json")).unwrap(), JSON);
    }

    #[test]
    fn either_existing_counterpart_refuses_pair_before_writing_bytes() {
        for existing_raw in [true, false] {
            let fixture = Fixture::new();
            let raw = fixture.raw();
            let json = raw.with_extension("json");
            let prior = if existing_raw { &raw } else { &json };
            std::fs::write(prior, b"preexisting sentinel").unwrap();
            let error = save_pair(&raw, RAW, JSON).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
            assert_eq!(std::fs::read(prior).unwrap(), b"preexisting sentinel");
            if existing_raw {
                assert!(!json.exists());
            } else {
                assert!(error.to_string().contains("reserve JSON sidecar"));
                assert_eq!(std::fs::metadata(&raw).unwrap().len(), 0);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn existing_symlink_is_not_followed_or_overwritten() {
        let fixture = Fixture::new();
        let target = fixture.0.join("prior");
        std::fs::write(&target, b"sentinel").unwrap();
        std::os::unix::fs::symlink(&target, fixture.raw()).unwrap();
        assert!(save_pair(&fixture.raw(), RAW, JSON).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"sentinel");
        assert!(!fixture.raw().with_extension("json").exists());
    }

    struct Faults {
        fail: usize,
        step: usize,
        trace: Vec<&'static str>,
    }
    impl Faults {
        fn boundary(&mut self, name: &'static str) -> io::Result<()> {
            self.trace.push(name);
            let step = self.step;
            self.step += 1;
            if step == self.fail {
                return Err(io::Error::other("injected filesystem failure"));
            }
            Ok(())
        }
    }
    impl Io for Faults {
        fn create_new(&mut self, path: &Path) -> io::Result<File> {
            self.boundary("reserve")?;
            Filesystem.create_new(path)
        }
        fn write(&mut self, file: &mut File, bytes: &[u8]) -> io::Result<()> {
            // A failed write may have saved a prefix. The caller must retain
            // the reservation and never treat a partial result as successful.
            if self.step == self.fail {
                file.write_all(&bytes[..bytes.len() / 2])?;
            }
            self.boundary("write")?;
            Filesystem.write(file, bytes)
        }
        fn sync(&mut self, file: &File) -> io::Result<()> {
            self.boundary("sync")?;
            Filesystem.sync(file)
        }
        fn verify(&mut self, path: &Path, expected: &[u8]) -> io::Result<()> {
            self.boundary("verify")?;
            Filesystem.verify(path, expected)
        }
        fn sync_directory(&mut self, path: &Path) -> io::Result<()> {
            self.boundary("directory sync")?;
            Filesystem.sync_directory(path)
        }
    }

    #[test]
    fn every_io_boundary_fails_without_acceptance_or_reusing_partial_files() {
        for fail in 0..9 {
            let fixture = Fixture::new();
            let path = fixture.raw();
            let mut io = Faults {
                fail,
                step: 0,
                trace: vec![],
            };
            let error = save_with(&mut io, &path, RAW, JSON).unwrap_err();
            assert!(error.to_string().contains("injected filesystem failure"));
            assert_eq!(io.step, fail + 1);
            if path.exists() {
                let raw_before = std::fs::read(&path).unwrap();
                let json_before = std::fs::read(path.with_extension("json")).ok();
                assert!(save_pair(&path, b"retry", b"retry JSON").is_err());
                assert_eq!(std::fs::read(&path).unwrap(), raw_before);
                assert_eq!(std::fs::read(path.with_extension("json")).ok(), json_before);
            }
            if fail < 2 {
                assert!(
                    !io.trace.contains(&"write"),
                    "both reservations must precede bytes"
                );
            }
        }
    }

    #[test]
    fn successful_io_order_reserves_both_before_writes_and_verifies_after_sync() {
        let fixture = Fixture::new();
        let mut io = Faults {
            fail: usize::MAX,
            step: 0,
            trace: vec![],
        };
        save_with(&mut io, &fixture.raw(), RAW, JSON).unwrap();
        assert_eq!(
            io.trace,
            [
                "reserve",
                "reserve",
                "write",
                "sync",
                "write",
                "sync",
                "directory sync",
                "verify",
                "verify"
            ]
        );
        assert_eq!(std::fs::read(fixture.raw()).unwrap(), RAW);
        assert_eq!(
            std::fs::read(fixture.raw().with_extension("json")).unwrap(),
            JSON
        );
    }

    #[test]
    fn reopen_verification_rejects_short_changed_and_extra_bytes() {
        let fixture = Fixture::new();
        for bytes in [
            &RAW[..RAW.len() - 1],
            b"wrong bytes".as_slice(),
            &[RAW, b"extra"].concat(),
        ] {
            std::fs::write(fixture.raw(), bytes).unwrap();
            assert_eq!(
                Filesystem.verify(&fixture.raw(), RAW).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
        }
    }
}
