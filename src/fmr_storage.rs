//! Protected file-only persistence for two independently acquired raw passes.
//!
//! This module never accesses a device. A storage receipt proves local copy
//! checks only. The caller owns acquisition freshness, held-session continuity
//! and the separate human-observed return to stock. Native durability remains
//! a platform qualification gate; a successful fsync is not physical evidence.

use std::path::{Path, PathBuf};

pub const RAW_LEN: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    DifferentFilesystems,
    /// Explicitly accepts separate private directories on one filesystem.
    /// Filesystem identity is recorded separately from physical independence.
    SeparateDirectories,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredCopies {
    pub sha256: String,
    /// Metadata cannot establish physical-media independence.
    pub independent_media: bool,
    pub different_filesystems: bool,
    pub first_path: PathBuf,
    pub second_path: PathBuf,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
mod native {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::ffi::{CStr, CString};
    use std::fs::{File, Metadata};
    use std::io::{Read, Write};
    use std::os::fd::{AsRawFd, FromRawFd, RawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::MetadataExt;
    use std::time::{Duration, Instant};

    const RAW_NAME: &str = "backup.raw";
    const STORAGE_NAME: &str = "storage-only.json";
    const CAPTURE_NAME: &str = "capture-receipt.json";
    const BUDGET: Duration = Duration::from_secs(600);

    #[cfg(target_os = "macos")]
    mod darwin_acl {
        use super::*;
        use std::ffi::c_void;
        const EXTENDED: i32 = 0x100; // SDK sys/acl.h: ACL_TYPE_EXTENDED
        unsafe extern "C" {
            fn acl_init(count: i32) -> *mut c_void;
            fn acl_free(acl: *mut c_void) -> i32;
            fn acl_valid(acl: *mut c_void) -> i32;
            fn acl_get_fd_np(fd: i32, kind: i32) -> *mut c_void;
            fn acl_set_fd_np(fd: i32, acl: *mut c_void, kind: i32) -> i32;
            fn acl_get_entry(acl: *mut c_void, entry: i32, out: *mut *mut c_void) -> i32;
        }
        struct Acl(*mut c_void);
        impl Drop for Acl {
            fn drop(&mut self) {
                unsafe {
                    acl_free(self.0);
                }
            }
        }
        pub(super) fn empty(file: &File) -> Result<(), String> {
            // SAFETY: all calls operate on a held local file fd or an owned ACL
            // allocation. acl_free releases exactly that allocation once.
            let acl = Acl(unsafe { acl_get_fd_np(file.as_raw_fd(), EXTENDED) });
            if acl.0.is_null() {
                // Darwin returns ENOENT for the absent extended-ACL attribute
                // on an existing held fd after installing an empty ACL.
                if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                    file.metadata()
                        .map_err(|e| format!("ACL fd metadata: {e}"))?;
                    return Ok(());
                }
                return Err(error("read storage extended ACL"));
            }
            if unsafe { acl_valid(acl.0) } != 0 {
                return Err(error("invalid storage ACL"));
            }
            let mut entry = std::ptr::null_mut();
            // Darwin returns 0 for an entry and -1/EINVAL at the end, unlike
            // Linux POSIX ACL iteration. Validate the ACL before this query.
            let result = unsafe { acl_get_entry(acl.0, 0, &mut entry) };
            if result == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINVAL)
            {
                Ok(())
            } else if result == 0 {
                Err("storage extended ACL must be empty".into())
            } else {
                Err(error("check storage extended ACL entries"))
            }
        }
        pub(super) fn clear_new(file: &File) -> Result<(), String> {
            let acl = Acl(unsafe { acl_init(0) });
            if acl.0.is_null() {
                return Err(error("allocate empty storage ACL"));
            }
            if unsafe { acl_set_fd_np(file.as_raw_fd(), acl.0, EXTENDED) } != 0 {
                return Err(error("clear inherited ACL on new storage object"));
            }
            empty(file)
        }
        #[cfg(test)]
        pub(super) fn add_fixture_grant(file: &File) {
            unsafe extern "C" {
                fn acl_create_entry(acl: *mut *mut c_void, entry: *mut *mut c_void) -> i32;
                fn acl_set_tag_type(entry: *mut c_void, tag: i32) -> i32;
                fn acl_set_qualifier(entry: *mut c_void, qualifier: *const c_void) -> i32;
                fn acl_set_permset_mask_np(entry: *mut c_void, mask: u64) -> i32;
                fn acl_get_flagset_np(entry: *mut c_void, flags: *mut *mut c_void) -> i32;
                fn acl_add_flag_np(flags: *mut c_void, flag: i32) -> i32;
            }
            let mut acl = Acl(unsafe { acl_init(1) });
            assert!(!acl.0.is_null());
            let mut entry = std::ptr::null_mut();
            assert_eq!(unsafe { acl_create_entry(&mut acl.0, &mut entry) }, 0);
            assert_eq!(unsafe { acl_set_tag_type(entry, 1) }, 0); // ACL_EXTENDED_ALLOW
            // Darwin's well-known everyone UUID; only synthetic temp fixtures
            // receive grants. Existing user parents are never changed.
            let everyone: [u8; 16] = [
                0xab, 0xcd, 0xef, 0xab, 0xcd, 0xef, 0xab, 0xcd, 0xef, 0xab, 0xcd, 0xef, 0, 0, 0,
                0x0c,
            ];
            assert_eq!(
                unsafe { acl_set_qualifier(entry, everyone.as_ptr().cast()) },
                0
            );
            assert_eq!(
                unsafe { acl_set_permset_mask_np(entry, (1 << 1) | (1 << 2) | (1 << 3)) },
                0
            );
            let mut flags = std::ptr::null_mut();
            assert_eq!(unsafe { acl_get_flagset_np(entry, &mut flags) }, 0);
            assert_eq!(unsafe { acl_add_flag_np(flags, 1 << 5) }, 0); // FILE_INHERIT
            assert_eq!(unsafe { acl_add_flag_np(flags, 1 << 6) }, 0); // DIRECTORY_INHERIT
            assert_eq!(
                unsafe { acl_set_fd_np(file.as_raw_fd(), acl.0, EXTENDED) },
                0,
                "{}",
                error("fixture ACL install")
            );
        }
    }

    fn empty_acl(file: &File) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        darwin_acl::empty(file)?;
        #[cfg(not(target_os = "macos"))]
        let _ = file;
        Ok(())
    }
    fn clear_new_acl(file: &File) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        darwin_acl::clear_new(file)?;
        #[cfg(not(target_os = "macos"))]
        let _ = file;
        Ok(())
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Identity {
        dev: u64,
        ino: u64,
    }
    impl Identity {
        fn of(m: &Metadata) -> Self {
            Self {
                dev: m.dev(),
                ino: m.ino(),
            }
        }
    }

    struct Destination {
        path: PathBuf,
        directory: File,
        identity: Identity,
        raw: File,
        raw_identity: Identity,
    }

    struct Receipt {
        file: File,
        identity: Identity,
        name: &'static str,
        bytes: Vec<u8>,
    }

    impl Receipt {
        fn verify(&self, directory: &File) -> Result<(), String> {
            regular_private(&self.file, self.identity)?;
            let mut reopened = open_at(
                directory.as_raw_fd(),
                &literal(self.name),
                libc::O_RDONLY,
                0,
            )?;
            if regular_private(&reopened, self.identity)?.len() != self.bytes.len() as u64 {
                return Err("storage receipt identity or length changed".into());
            }
            let mut actual = Vec::new();
            (&mut reopened)
                .take(self.bytes.len() as u64 + 1)
                .read_to_end(&mut actual)
                .map_err(|e| e.to_string())?;
            if actual != self.bytes {
                return Err("storage receipt bytes changed".into());
            }
            regular_private(&reopened, self.identity)?;
            Ok(())
        }
    }

    /// A pair of new, exclusively reserved destinations. Errors are terminal;
    /// partial files are retained privately for diagnosis, never auto-reused.
    pub struct PreparedPair {
        first: Destination,
        second: Destination,
        policy: Policy,
        different_filesystems: bool,
        started: Instant,
        attempted: bool,
        acceptance_attempted: bool,
        terminal: bool,
        stored: Option<StoredCopies>,
        receipts: Option<(Receipt, Receipt)>,
        #[cfg(test)]
        fault: Option<Fault>,
    }

    #[cfg(test)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Fault {
        Write,
        Sync,
        Reopen,
        CorruptReopen,
        Receipt,
        Accept,
    }

    fn error(context: &str) -> String {
        format!("{context}: {}", std::io::Error::last_os_error())
    }
    fn c_name(value: &std::ffi::OsStr) -> Result<CString, String> {
        CString::new(value.as_bytes()).map_err(|_| "storage path contains NUL".into())
    }
    fn literal(value: &str) -> CString {
        CString::new(value).unwrap()
    }
    fn open_at(parent: RawFd, name: &CStr, flags: i32, mode: libc::mode_t) -> Result<File, String> {
        // All names are one component and each open forbids following symlinks.
        let fd = unsafe {
            libc::openat(
                parent,
                name.as_ptr(),
                flags | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
                mode as libc::c_uint,
            )
        };
        if fd < 0 {
            Err(error("secure storage open"))
        } else {
            Ok(unsafe { File::from_raw_fd(fd) })
        }
    }
    fn open_directory(path: &Path) -> Result<File, String> {
        if !path.is_absolute() {
            return Err("storage destinations must be absolute".into());
        }
        let mut directory = open_at(
            libc::AT_FDCWD,
            &literal("/"),
            libc::O_RDONLY | libc::O_DIRECTORY,
            0,
        )?;
        for component in path.components() {
            use std::path::Component;
            match component {
                Component::RootDir => {}
                Component::Normal(name) => {
                    directory = open_at(
                        directory.as_raw_fd(),
                        &c_name(name)?,
                        libc::O_RDONLY | libc::O_DIRECTORY,
                        0,
                    )?
                }
                _ => return Err("storage paths cannot contain relative components".into()),
            }
        }
        Ok(directory)
    }
    fn private_directory(file: &File) -> Result<Metadata, String> {
        let m = file
            .metadata()
            .map_err(|e| format!("storage directory metadata: {e}"))?;
        if !m.is_dir() || m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o7777 != 0o700 {
            return Err("storage destination must be an owner-private 0700 directory".into());
        }
        empty_acl(file)?;
        Ok(m)
    }
    fn regular_private(file: &File, identity: Identity) -> Result<Metadata, String> {
        let m = file
            .metadata()
            .map_err(|e| format!("storage file metadata: {e}"))?;
        if !m.is_file()
            || m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o7777 != 0o600
            || m.nlink() != 1
            || Identity::of(&m) != identity
        {
            return Err(
                "storage file identity, ownership, private mode or link count changed".into(),
            );
        }
        empty_acl(file)?;
        Ok(m)
    }
    fn create_private(directory: &File, name: &str) -> Result<File, String> {
        if !matches!(
            Path::new(name).components().collect::<Vec<_>>().as_slice(),
            [std::path::Component::Normal(_)]
        ) {
            return Err("private output filename must be one normal component".into());
        }
        let name = c_name(std::ffi::OsStr::new(name))?;
        let file = open_at(
            directory.as_raw_fd(),
            &name,
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL,
            0o600,
        )?;
        clear_new_acl(&file)?;
        // A restrictive umask is allowed; normalize only this newly created fd.
        if unsafe { libc::fchmod(file.as_raw_fd(), 0o600) } != 0 {
            return Err(error("private file permissions"));
        }
        let identity = Identity::of(&file.metadata().map_err(|e| e.to_string())?);
        regular_private(&file, identity)?;
        Ok(file)
    }
    fn sync_file(file: &File) -> Result<(), String> {
        file.sync_all().map_err(|e| format!("storage fsync: {e}"))?;
        #[cfg(target_os = "macos")]
        if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_FULLFSYNC) } != 0 {
            return Err(error("storage full fsync"));
        }
        Ok(())
    }
    fn sync_directory(directory: &File) -> Result<(), String> {
        directory
            .sync_all()
            .map_err(|e| format!("storage directory fsync: {e}"))
    }
    fn digest(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
    fn json_string(value: &str) -> String {
        let mut out = String::from("\"");
        for c in value.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }

    fn prepare_directory(path: &Path) -> Result<(File, Identity), String> {
        if !path.is_absolute() {
            return Err("storage destinations must be absolute".into());
        }
        let parent_path = path
            .parent()
            .ok_or("storage destination needs an existing parent")?;
        let name = path
            .file_name()
            .ok_or("storage destination must be a new named directory")?;
        let parent = open_directory(parent_path)?;
        let name = c_name(name)?;
        if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
            return Err(error("exclusive storage directory creation"));
        }
        let directory = open_at(
            parent.as_raw_fd(),
            &name,
            libc::O_RDONLY | libc::O_DIRECTORY,
            0,
        )?;
        clear_new_acl(&directory)?;
        if unsafe { libc::fchmod(directory.as_raw_fd(), 0o700) } != 0 {
            return Err(error("private directory permissions"));
        }
        let identity = Identity::of(&private_directory(&directory)?);
        sync_directory(&parent)?;
        check_directory(path, &directory, identity)?;
        Ok((directory, identity))
    }

    fn check_directory(path: &Path, directory: &File, identity: Identity) -> Result<(), String> {
        if Identity::of(&private_directory(directory)?) != identity {
            return Err("storage directory identity changed".into());
        }
        let reopened = open_directory(path)?;
        if Identity::of(&private_directory(&reopened)?) != identity {
            return Err("storage directory pathname changed".into());
        }
        Ok(())
    }

    /// New local output directory for small private receipts and samples.
    /// Uses the same descriptor-relative creation and ACL checks as backups.
    /// Write failures are terminal; retained partial files are never overwritten.
    pub struct PrivateOutputDirectory {
        path: PathBuf,
        directory: File,
        identity: Identity,
        files: Vec<Receipt>,
        terminal: bool,
        #[cfg(test)]
        before_write: Option<fn(&File, &Path)>,
    }

    impl PrivateOutputDirectory {
        /// File-only validation shared by CLI prepare and run. Reserves nothing;
        /// prepare repeats the checks and exclusively creates the directory.
        pub fn preflight(path: &Path) -> Result<(), String> {
            if !path.is_absolute() {
                return Err("storage destinations must be absolute".into());
            }
            let parent = open_directory(path.parent().ok_or("output parent missing")?)?;
            let name = c_name(path.file_name().ok_or("output needs a named directory")?)?;
            let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
            // SAFETY: fstatat only queries this named local filesystem entry;
            // the output storage is valid and need not be read afterward.
            let result = unsafe {
                libc::fstatat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    metadata.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if result == 0 {
                return Err("output directory must be new".into());
            }
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::NotFound {
                return Err(format!("private output preflight: {error}"));
            }
            Ok(())
        }

        pub fn prepare(path: &Path) -> Result<Self, String> {
            let (directory, identity) = prepare_directory(path)?;
            let output = Self {
                path: path.to_owned(),
                directory,
                identity,
                files: Vec::new(),
                terminal: false,
                #[cfg(test)]
                before_write: None,
            };
            output.check()?;
            Ok(output)
        }

        pub fn check(&self) -> Result<(), String> {
            if self.terminal {
                return Err("private output cannot be reused after failure".into());
            }
            check_directory(&self.path, &self.directory, self.identity)?;
            for file in &self.files {
                file.verify(&self.directory)?;
            }
            check_directory(&self.path, &self.directory, self.identity)
        }

        pub fn write_private(&mut self, name: &'static str, bytes: &[u8]) -> Result<(), String> {
            let result = self.write_inner(name, bytes);
            if result.is_err() {
                self.terminal = true;
            }
            result
        }

        fn write_inner(&mut self, name: &'static str, bytes: &[u8]) -> Result<(), String> {
            self.check()?;
            let mut file = create_private(&self.directory, name)?;
            let identity = Identity::of(&file.metadata().map_err(|e| e.to_string())?);
            #[cfg(test)]
            if let Some(before_write) = self.before_write.take() {
                before_write(&file, &self.path.join(name));
            }
            // Recheck the directory and new pathname before any private bytes.
            // ACL clearing only touches newly created, still-empty descriptors.
            self.check()?;
            let reopened = open_at(
                self.directory.as_raw_fd(),
                &literal(name),
                libc::O_RDONLY,
                0,
            )?;
            regular_private(&reopened, identity)?;
            regular_private(&file, identity)?;
            file.write_all(bytes)
                .map_err(|e| format!("private output write: {e}"))?;
            sync_file(&file)?;
            sync_directory(&self.directory)?;
            let saved = Receipt {
                file,
                identity,
                name,
                bytes: bytes.to_vec(),
            };
            saved.verify(&self.directory)?;
            self.files.push(saved);
            self.check()
        }
    }

    impl Destination {
        fn prepare(path: &Path) -> Result<Self, String> {
            let (directory, identity) = prepare_directory(path)?;
            let raw = create_private(&directory, RAW_NAME)?;
            let raw_identity = Identity::of(&raw.metadata().map_err(|e| e.to_string())?);
            let destination = Self {
                path: path.to_owned(),
                directory,
                identity,
                raw,
                raw_identity,
            };
            sync_file(&destination.raw)?;
            sync_directory(&destination.directory)?;
            destination.check()?;
            Ok(destination)
        }
        fn check(&self) -> Result<(), String> {
            check_directory(&self.path, &self.directory, self.identity)?;
            regular_private(&self.raw, self.raw_identity)?;
            let reopened = open_at(
                self.directory.as_raw_fd(),
                &literal(RAW_NAME),
                libc::O_RDONLY,
                0,
            )?;
            regular_private(&reopened, self.raw_identity)?;
            Ok(())
        }
        fn verify(&self, expected: &[u8], hash: &str) -> Result<(), String> {
            self.check()?;
            let mut reopened = open_at(
                self.directory.as_raw_fd(),
                &literal(RAW_NAME),
                libc::O_RDONLY,
                0,
            )?;
            if regular_private(&reopened, self.raw_identity)?.len() != RAW_LEN as u64 {
                return Err("reopened storage length mismatch".into());
            }
            let mut bytes = Vec::with_capacity(RAW_LEN);
            (&mut reopened)
                .take((RAW_LEN + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|e| format!("storage reopen read: {e}"))?;
            if bytes != expected || digest(&bytes) != hash {
                return Err("reopened storage hash or bytes mismatch".into());
            }
            regular_private(&reopened, self.raw_identity)?;
            self.check()
        }
        fn verify_hash(&self, hash: &str) -> Result<(), String> {
            self.check()?;
            let mut reopened = open_at(
                self.directory.as_raw_fd(),
                &literal(RAW_NAME),
                libc::O_RDONLY,
                0,
            )?;
            if regular_private(&reopened, self.raw_identity)?.len() != RAW_LEN as u64 {
                return Err("stored copy length changed before acceptance".into());
            }
            let mut bytes = Vec::with_capacity(RAW_LEN);
            (&mut reopened)
                .take((RAW_LEN + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() != RAW_LEN || digest(&bytes) != hash {
                return Err("stored copy changed before acceptance".into());
            }
            regular_private(&reopened, self.raw_identity)?;
            self.check()
        }
        fn receipt(&self, name: &'static str, bytes: &[u8]) -> Result<Receipt, String> {
            self.check()?;
            let mut receipt = create_private(&self.directory, name)?;
            let identity = Identity::of(&receipt.metadata().map_err(|e| e.to_string())?);
            receipt
                .write_all(bytes)
                .map_err(|e| format!("storage receipt write: {e}"))?;
            sync_file(&receipt)?;
            sync_directory(&self.directory)?;
            regular_private(&receipt, identity)?;
            let mut reopened = open_at(
                self.directory.as_raw_fd(),
                &literal(name),
                libc::O_RDONLY,
                0,
            )?;
            if regular_private(&reopened, identity)?.len() != bytes.len() as u64 {
                return Err("storage receipt length changed".into());
            }
            let mut actual = Vec::new();
            (&mut reopened)
                .take(bytes.len() as u64 + 1)
                .read_to_end(&mut actual)
                .map_err(|e| e.to_string())?;
            if actual != bytes {
                return Err("storage receipt changed after reopen".into());
            }
            regular_private(&reopened, identity)?;
            self.check()?;
            let saved = Receipt {
                file: receipt,
                identity,
                name,
                bytes: bytes.to_owned(),
            };
            saved.verify(&self.directory)?;
            Ok(saved)
        }
    }

    impl PreparedPair {
        pub fn prepare(first: &Path, second: &Path, policy: Policy) -> Result<Self, String> {
            let started = Instant::now();
            if first == second {
                return Err("backup destinations must be separate directories".into());
            }
            let first = Destination::prepare(first)?;
            let second = Destination::prepare(second)?;
            if first.identity == second.identity {
                return Err("backup destinations refer to the same directory".into());
            }
            let different_filesystems = first.identity.dev != second.identity.dev;
            if policy == Policy::DifferentFilesystems && !different_filesystems {
                return Err(
                    "independent-media policy requires different filesystem devices".into(),
                );
            }
            let pair = Self {
                first,
                second,
                policy,
                different_filesystems,
                started,
                attempted: false,
                acceptance_attempted: false,
                terminal: false,
                stored: None,
                receipts: None,
                #[cfg(test)]
                fault: None,
            };
            pair.check()?;
            Ok(pair)
        }
        fn check(&self) -> Result<(), String> {
            if self.terminal {
                return Err("storage pair is terminal or cancelled".into());
            }
            if self.started.elapsed() >= BUDGET {
                return Err("storage lifecycle exceeded its ten-minute budget".into());
            }
            self.first.check()?;
            self.second.check()?;
            if let Some((first, second)) = &self.receipts {
                first.verify(&self.first.directory)?;
                second.verify(&self.second.directory)?;
            }
            if self.started.elapsed() >= BUDGET {
                return Err("storage lifecycle exceeded its ten-minute budget".into());
            }
            Ok(())
        }
        /// Cancellation does not remove partial originals or publish receipts.
        pub fn cancel(&mut self) {
            self.terminal = true;
        }
        pub fn store(
            &mut self,
            first: &[u8],
            second: &[u8],
            provenance_json: &str,
        ) -> Result<StoredCopies, String> {
            if self.attempted {
                return Err("storage pair cannot be reused".into());
            }
            self.attempted = true;
            let result = self.store_inner(first, second, provenance_json);
            if result.is_err() {
                self.terminal = true;
            }
            result
        }
        fn store_inner(
            &mut self,
            first: &[u8],
            second: &[u8],
            provenance: &str,
        ) -> Result<StoredCopies, String> {
            self.check()?;
            if first.len() != RAW_LEN || second.len() != RAW_LEN {
                return Err("both raw passes must be exactly 1 MiB".into());
            }
            if provenance.len() > crate::saved_receipt::MAX_PROVENANCE {
                return Err("storage provenance exceeds the bounded receipt size".into());
            }
            if self.first.raw.metadata().map_err(|e| e.to_string())?.len() != 0
                || self.second.raw.metadata().map_err(|e| e.to_string())?.len() != 0
            {
                return Err("reserved raw files changed before their sole write".into());
            }
            let hash = digest(first);
            if digest(second) != hash || first != second {
                return Err("raw passes differ in hash or bytes".into());
            }
            #[cfg(test)]
            if self.fault == Some(Fault::Write) {
                return Err("injected disk-full write failure".into());
            }
            self.first
                .raw
                .write_all(first)
                .map_err(|e| format!("first raw write: {e}"))?;
            self.check()?;
            self.second
                .raw
                .write_all(second)
                .map_err(|e| format!("second raw write: {e}"))?;
            self.check()?;
            #[cfg(test)]
            if self.fault == Some(Fault::Sync) {
                return Err("injected sync failure".into());
            }
            sync_file(&self.first.raw)?;
            self.check()?;
            sync_file(&self.second.raw)?;
            self.check()?;
            sync_directory(&self.first.directory)?;
            self.check()?;
            sync_directory(&self.second.directory)?;
            self.check()?;
            #[cfg(test)]
            if self.fault == Some(Fault::Reopen) {
                return Err("injected reopen failure".into());
            }
            #[cfg(test)]
            if self.fault == Some(Fault::CorruptReopen) {
                use std::os::unix::fs::FileExt;
                self.second
                    .raw
                    .write_at(&[second[0] ^ 1], 0)
                    .map_err(|e| e.to_string())?;
            }
            self.first.verify(first, &hash)?;
            self.check()?;
            self.second.verify(second, &hash)?;
            self.check()?;
            let copies = StoredCopies {
                sha256: hash,
                independent_media: false,
                different_filesystems: self.different_filesystems,
                first_path: self.first.path.join(RAW_NAME),
                second_path: self.second.path.join(RAW_NAME),
            };
            #[cfg(test)]
            if self.fault == Some(Fault::Receipt) {
                return Err("injected receipt failure".into());
            }
            let receipt = self.receipt(&copies, "storage-only", provenance);
            let first_receipt = self.first.receipt(STORAGE_NAME, receipt.as_bytes())?;
            self.check()?;
            let second_receipt = self.second.receipt(STORAGE_NAME, receipt.as_bytes())?;
            self.receipts = Some((first_receipt, second_receipt));
            self.check()?;
            self.stored = Some(copies.clone());
            Ok(copies)
        }
        fn receipt(&self, copies: &StoredCopies, stage: &str, provenance: &str) -> String {
            let policy = match self.policy {
                Policy::DifferentFilesystems => "different-filesystems",
                Policy::SeparateDirectories => "separate-directories",
            };
            format!(
                "{{\"schema\":{},\"stage\":{},\"length\":{},\"sha256\":{},\"copies\":2,\"independent_media\":false,\"different_filesystems\":{},\"policy\":{},\"first_filesystem\":{},\"second_filesystem\":{},\"first_directory_inode\":{},\"second_directory_inode\":{},\"physical_media_independence_verified\":false,\"returned_to_stock\":false,\"provenance_json\":{}}}\n",
                json_string(crate::saved_receipt::SCHEMA),
                json_string(stage),
                RAW_LEN,
                json_string(&copies.sha256),
                copies.different_filesystems,
                json_string(policy),
                self.first.identity.dev,
                self.second.identity.dev,
                self.first.identity.ino,
                self.second.identity.ino,
                json_string(provenance)
            )
        }
        /// Call only after the owner's final held-session continuity checkpoint.
        /// This records that checkpoint's provenance; it never proves stock return.
        /// Both matching receipts are required. A partial publication on an I/O
        /// failure is not an accepted pair and this object cannot retry it.
        pub fn accept(&mut self, provenance_json: &str) -> Result<(), String> {
            if self.acceptance_attempted {
                return Err("capture receipt cannot be reused".into());
            }
            self.acceptance_attempted = true;
            let result = self.accept_inner(provenance_json);
            self.terminal = true;
            result
        }
        fn accept_inner(&self, provenance: &str) -> Result<(), String> {
            self.check()?;
            if provenance.len() > crate::saved_receipt::MAX_PROVENANCE {
                return Err("capture provenance exceeds the bounded receipt size".into());
            }
            let copies = self
                .stored
                .as_ref()
                .ok_or("capture receipt requires completed storage")?;
            #[cfg(test)]
            if self.fault == Some(Fault::Accept) {
                return Err("injected final receipt failure".into());
            }
            self.first.verify_hash(&copies.sha256)?;
            self.check()?;
            self.second.verify_hash(&copies.sha256)?;
            self.check()?;
            let receipt = self.receipt(copies, "final-held-session-checkpoint", provenance);
            let first = self.first.receipt(CAPTURE_NAME, receipt.as_bytes())?;
            self.check()?;
            let second = self.second.receipt(CAPTURE_NAME, receipt.as_bytes())?;
            self.check()?;
            first.verify(&self.first.directory)?;
            second.verify(&self.second.directory)?;
            self.check()?;
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::os::unix::fs::{PermissionsExt, symlink};
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        struct Fixture(PathBuf);
        impl Fixture {
            fn new() -> Self {
                let path = std::env::temp_dir().join(format!(
                    "fmr-storage-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                std::fs::create_dir(&path).unwrap();
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
                Self(path.canonicalize().unwrap())
            }
            fn pair(&self) -> PreparedPair {
                PreparedPair::prepare(
                    &self.0.join("a"),
                    &self.0.join("b"),
                    Policy::SeparateDirectories,
                )
                .unwrap()
            }
        }
        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        fn data() -> Vec<u8> {
            (0..RAW_LEN)
                .map(|i| (i.wrapping_mul(71).wrapping_add(i / 257)) as u8)
                .collect()
        }
        fn no_capture(pair: &PreparedPair) {
            assert!(!pair.first.path.join(CAPTURE_NAME).exists());
            assert!(!pair.second.path.join(CAPTURE_NAME).exists());
        }
        #[test]
        fn private_output_reopens_all_four_files_and_never_overwrites() {
            let fixture = Fixture::new();
            let path = fixture.0.join("small-check");
            let mut output = PrivateOutputDirectory::prepare(&path).unwrap();
            for (name, bytes) in [
                (
                    "started.json",
                    b"{\"serial\":\"private-fixture\"}".as_slice(),
                ),
                ("first-256.bin", [0x19; 256].as_slice()),
                ("second-256.bin", [0x19; 256].as_slice()),
                (
                    "result.json",
                    b"{\"state\":\"two-equal-samples\"}".as_slice(),
                ),
            ] {
                output.write_private(name, bytes).unwrap();
                assert_eq!(std::fs::read(path.join(name)).unwrap(), bytes);
                let metadata = std::fs::metadata(path.join(name)).unwrap();
                assert_eq!(metadata.mode() & 0o7777, 0o600);
                assert_eq!(metadata.nlink(), 1);
            }
            output.check().unwrap();
            assert_eq!(std::fs::metadata(&path).unwrap().mode() & 0o7777, 0o700);
            assert!(PrivateOutputDirectory::prepare(&path).is_err());
            assert!(
                output
                    .write_private("started.json", b"replacement")
                    .is_err()
            );
            assert!(output.write_private("later.json", b"private").is_err());
            assert!(!path.join("later.json").exists());
        }

        #[test]
        fn private_output_refuses_unsafe_names_and_symlink_paths() {
            let fixture = Fixture::new();
            symlink(&fixture.0, fixture.0.join("alias")).unwrap();
            assert!(PrivateOutputDirectory::prepare(&fixture.0.join("alias/output")).is_err());
            for (i, name) in ["../escape", "/absolute", "sub/name", "", ".", "bad\0name"]
                .into_iter()
                .enumerate()
            {
                let mut output =
                    PrivateOutputDirectory::prepare(&fixture.0.join(format!("out-{i}"))).unwrap();
                assert!(output.write_private(name, b"private").is_err(), "{name:?}");
                assert_eq!(std::fs::read_dir(&output.path).unwrap().count(), 0);
            }
            assert!(!fixture.0.join("escape").exists());
        }

        #[test]
        fn private_output_refuses_directory_and_saved_file_substitution_before_next_bytes() {
            for directory in [true, false] {
                let fixture = Fixture::new();
                let path = fixture.0.join("output");
                let mut output = PrivateOutputDirectory::prepare(&path).unwrap();
                output
                    .write_private("started.json", b"private fixture")
                    .unwrap();
                if directory {
                    std::fs::rename(&path, fixture.0.join("original")).unwrap();
                    std::fs::create_dir(&path).unwrap();
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
                        .unwrap();
                } else {
                    let saved = path.join("started.json");
                    std::fs::rename(&saved, path.join("original.json")).unwrap();
                    symlink(path.join("original.json"), &saved).unwrap();
                }
                assert!(output.write_private("first-256.bin", &[0x31; 256]).is_err());
                assert!(!path.join("first-256.bin").exists());
                assert!(!fixture.0.join("original/first-256.bin").exists());
            }
        }

        #[test]
        fn private_output_refuses_new_file_substitution_before_writing() {
            let fixture = Fixture::new();
            let path = fixture.0.join("output");
            let mut output = PrivateOutputDirectory::prepare(&path).unwrap();
            output.before_write = Some(|_, path| {
                std::fs::rename(path, path.with_extension("held")).unwrap();
                std::fs::write(path, []).unwrap();
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
            });
            assert!(
                output
                    .write_private("started.json", b"private identifier")
                    .is_err()
            );
            assert_eq!(
                std::fs::metadata(path.join("started.held")).unwrap().len(),
                0
            );
            assert_eq!(
                std::fs::metadata(path.join("started.json")).unwrap().len(),
                0
            );
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn private_output_clears_inherited_everyone_acl_and_preserves_parent_acl() {
            let fixture = Fixture::new();
            let parent = open_directory(&fixture.0).unwrap();
            darwin_acl::add_fixture_grant(&parent);
            let mut output = PrivateOutputDirectory::prepare(&fixture.0.join("output")).unwrap();
            for name in [
                "started.json",
                "first-256.bin",
                "second-256.bin",
                "result.json",
            ] {
                output.write_private(name, b"private fixture").unwrap();
            }
            output.check().unwrap();
            empty_acl(&output.directory).unwrap();
            for saved in &output.files {
                empty_acl(&saved.file).unwrap();
            }
            assert!(empty_acl(&parent).is_err());
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn private_output_refuses_postcreation_acls_before_private_bytes() {
            for target in ["directory", "saved", "new"] {
                let fixture = Fixture::new();
                let mut output =
                    PrivateOutputDirectory::prepare(&fixture.0.join("output")).unwrap();
                match target {
                    "directory" => darwin_acl::add_fixture_grant(&output.directory),
                    "saved" => {
                        output
                            .write_private("started.json", b"private fixture")
                            .unwrap();
                        darwin_acl::add_fixture_grant(&output.files[0].file);
                    }
                    "new" => {
                        output.before_write = Some(|file, _| darwin_acl::add_fixture_grant(file))
                    }
                    _ => unreachable!(),
                }
                assert!(
                    output.write_private("first-256.bin", &[0x71; 256]).is_err(),
                    "{target}"
                );
                if target == "new" {
                    assert_eq!(
                        std::fs::metadata(output.path.join("first-256.bin"))
                            .unwrap()
                            .len(),
                        0
                    );
                } else {
                    assert!(!output.path.join("first-256.bin").exists());
                }
                assert!(output.write_private("result.json", b"private").is_err());
                assert!(!output.path.join("result.json").exists());
            }
        }
        #[test]
        fn private_two_passes_reopen_and_separate_acceptance() {
            let fixture = Fixture::new();
            let mut pair = fixture.pair();
            let bytes = data();
            let copies = pair
                .store(&bytes, &bytes.clone(), "{\"capture\":\"fixture\"}")
                .unwrap();
            assert!(!copies.independent_media);
            assert!(!copies.different_filesystems);
            assert_eq!(copies.sha256, digest(&bytes));
            no_capture(&pair);
            for path in [&copies.first_path, &copies.second_path] {
                let m = std::fs::metadata(path).unwrap();
                assert_eq!(m.mode() & 0o777, 0o600);
                assert_eq!(m.nlink(), 1);
                assert_eq!(std::fs::read(path).unwrap(), bytes);
            }
            pair.accept("{\"final_identity\":true}").unwrap();
            let receipt = std::fs::read_to_string(pair.first.path.join(CAPTURE_NAME)).unwrap();
            assert!(receipt.contains("\"returned_to_stock\":false"));
            let parsed = crate::saved_receipt::Receipt::parse(receipt.as_bytes()).unwrap();
            assert_eq!(parsed.stage, "held-session-receipt-bound-locally");
            assert!(receipt.contains("\"independent_media\":false"));
            assert!(pair.accept("{}").is_err());
            assert!(pair.store(&bytes, &bytes, "{}").is_err());
        }
        #[test]
        fn different_filesystem_receipt_never_claims_physical_media_independence() {
            let fixture = Fixture::new();
            let mut pair = fixture.pair();
            let bytes = data();
            let mut copies = pair.store(&bytes, &bytes, "{}").unwrap();
            // Inject the other filesystem case without inspecting any devices.
            copies.different_filesystems = true;
            let receipt = pair.receipt(&copies, "storage-only", "{}");
            assert!(receipt.contains("\"different_filesystems\":true"));
            assert!(receipt.contains("\"independent_media\":false"));
            assert!(receipt.contains("\"physical_media_independence_verified\":false"));
        }
        #[test]
        fn no_overwrite_and_policy_are_preflight_checks() {
            let fixture = Fixture::new();
            let pair = fixture.pair();
            assert!(
                PreparedPair::prepare(
                    &pair.first.path,
                    &fixture.0.join("c"),
                    Policy::SeparateDirectories
                )
                .is_err()
            );
            assert!(
                PreparedPair::prepare(
                    &fixture.0.join("d"),
                    &fixture.0.join("e"),
                    Policy::DifferentFilesystems
                )
                .is_err()
            );
            assert!(
                PreparedPair::prepare(
                    Path::new("relative"),
                    &fixture.0.join("f"),
                    Policy::SeparateDirectories
                )
                .is_err()
            );
            assert_eq!(
                std::fs::metadata(pair.first.path.join(RAW_NAME))
                    .unwrap()
                    .len(),
                0
            );
        }
        #[test]
        fn symlink_ancestors_and_existing_leaf_refused() {
            let fixture = Fixture::new();
            symlink(&fixture.0, fixture.0.join("alias")).unwrap();
            assert!(
                PreparedPair::prepare(
                    &fixture.0.join("alias/a"),
                    &fixture.0.join("b"),
                    Policy::SeparateDirectories
                )
                .is_err()
            );
            symlink(&fixture.0, fixture.0.join("leaf")).unwrap();
            assert!(
                PreparedPair::prepare(
                    &fixture.0.join("leaf"),
                    &fixture.0.join("c"),
                    Policy::SeparateDirectories
                )
                .is_err()
            );
        }
        #[test]
        fn mismatched_and_short_passes_are_terminal() {
            for short in [true, false] {
                let fixture = Fixture::new();
                let mut pair = fixture.pair();
                let bytes = data();
                let mut second = bytes.clone();
                if short {
                    second.pop();
                } else {
                    second[RAW_LEN - 1] ^= 1;
                }
                assert!(pair.store(&bytes, &second, "{}").is_err());
                assert_eq!(pair.first.raw.metadata().unwrap().len(), 0);
                no_capture(&pair);
                assert!(pair.store(&bytes, &bytes, "{}").is_err());
                assert!(pair.accept("{}").is_err());
            }
        }
        #[test]
        fn injected_io_failures_never_publish_capture() {
            for fault in [
                Fault::Write,
                Fault::Sync,
                Fault::Reopen,
                Fault::CorruptReopen,
                Fault::Receipt,
            ] {
                let fixture = Fixture::new();
                let mut pair = fixture.pair();
                pair.fault = Some(fault);
                let bytes = data();
                assert!(pair.store(&bytes, &bytes, "{}").is_err(), "{fault:?}");
                no_capture(&pair);
                assert!(pair.accept("{}").is_err());
            }
        }
        #[test]
        fn hardlink_and_path_substitution_refused() {
            for rename_dir in [true, false] {
                let fixture = Fixture::new();
                let mut pair = fixture.pair();
                let bytes = data();
                if rename_dir {
                    std::fs::rename(&pair.first.path, fixture.0.join("moved")).unwrap();
                    std::fs::create_dir(&pair.first.path).unwrap();
                } else {
                    std::fs::hard_link(pair.first.path.join(RAW_NAME), fixture.0.join("link"))
                        .unwrap();
                }
                assert!(pair.store(&bytes, &bytes, "{}").is_err());
                no_capture(&pair);
            }
        }
        #[test]
        fn raw_symlink_and_regular_substitution_refused() {
            for link in [true, false] {
                let fixture = Fixture::new();
                let mut pair = fixture.pair();
                let bytes = data();
                std::fs::rename(pair.second.path.join(RAW_NAME), fixture.0.join("original"))
                    .unwrap();
                if link {
                    symlink(fixture.0.join("original"), pair.second.path.join(RAW_NAME)).unwrap();
                } else {
                    std::fs::write(pair.second.path.join(RAW_NAME), []).unwrap();
                }
                assert!(pair.store(&bytes, &bytes, "{}").is_err());
                no_capture(&pair);
            }
        }
        #[test]
        fn cancel_timeout_foreign_receipt_and_final_corruption_refused() {
            let bytes = data();
            for scenario in 0..5 {
                let fixture = Fixture::new();
                let mut pair = fixture.pair();
                if scenario == 0 {
                    pair.cancel();
                    assert!(pair.store(&bytes, &bytes, "{}").is_err());
                } else if scenario == 1 {
                    pair.started = Instant::now() - BUDGET;
                    assert!(pair.store(&bytes, &bytes, "{}").is_err());
                } else {
                    pair.store(&bytes, &bytes, "{}").unwrap();
                    if scenario == 2 {
                        std::fs::write(pair.first.path.join(CAPTURE_NAME), b"foreign receipt")
                            .unwrap();
                    }
                    if scenario == 3 {
                        use std::os::unix::fs::FileExt;
                        pair.second.raw.write_at(&[99], 0).unwrap();
                    }
                    if scenario == 4 {
                        pair.fault = Some(Fault::Accept);
                    }
                    assert!(pair.accept("{}").is_err());
                    assert!(!pair.second.path.join(CAPTURE_NAME).exists());
                }
                assert!(pair.accept("{}").is_err());
            }
        }
        #[cfg(target_os = "macos")]
        #[test]
        fn inherited_acl_is_cleared_only_on_new_objects_and_later_grants_refused() {
            let fixture = Fixture::new();
            let parent = open_directory(&fixture.0).unwrap();
            darwin_acl::add_fixture_grant(&parent);
            assert!(empty_acl(&parent).is_err());
            let mut pair = fixture.pair();
            // The explicit existing parent stays untouched.
            assert!(empty_acl(&parent).is_err());
            empty_acl(&pair.first.directory).unwrap();
            empty_acl(&pair.second.raw).unwrap();
            let bytes = data();
            pair.store(&bytes, &bytes, "{}").unwrap();
            darwin_acl::add_fixture_grant(&pair.first.raw);
            assert!(pair.accept("{}").is_err());
            no_capture(&pair);
        }
        #[test]
        fn fifo_substitution_without_a_writer_is_refused_without_blocking() {
            let bytes = data();
            for receipt in [false, true] {
                let fixture = Fixture::new();
                let mut pair = fixture.pair();
                if receipt {
                    pair.store(&bytes, &bytes, "{}").unwrap();
                }
                let path = pair
                    .first
                    .path
                    .join(if receipt { STORAGE_NAME } else { RAW_NAME });
                std::fs::rename(&path, fixture.0.join("held-original")).unwrap();
                let name = c_name(path.as_os_str()).unwrap();
                // A local filesystem FIFO fixture; no device is opened.
                assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
                if receipt {
                    assert!(pair.accept("{}").is_err());
                } else {
                    assert!(pair.store(&bytes, &bytes, "{}").is_err());
                }
                no_capture(&pair);
            }
        }
        #[test]
        fn storage_receipt_mutation_replacement_and_links_block_acceptance() {
            let bytes = data();
            for scenario in 0..3 {
                let fixture = Fixture::new();
                let mut pair = fixture.pair();
                pair.store(&bytes, &bytes, "{}").unwrap();
                let path = pair.first.path.join(STORAGE_NAME);
                if scenario == 0 {
                    std::fs::write(&path, b"foreign storage receipt").unwrap();
                } else if scenario == 1 {
                    std::fs::rename(&path, fixture.0.join("saved-receipt")).unwrap();
                    std::fs::write(&path, b"replacement").unwrap();
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                        .unwrap();
                } else {
                    std::fs::hard_link(&path, fixture.0.join("linked-receipt")).unwrap();
                }
                assert!(pair.accept("{}").is_err());
                no_capture(&pair);
            }
        }
        #[test]
        fn prewritten_reserved_raw_is_not_overwritten() {
            let fixture = Fixture::new();
            let mut pair = fixture.pair();
            let path = pair.first.path.join(RAW_NAME);
            std::fs::write(&path, b"unexpected prior content").unwrap();
            let bytes = data();
            assert!(pair.store(&bytes, &bytes, "{}").is_err());
            assert_eq!(std::fs::read(path).unwrap(), b"unexpected prior content");
            assert_eq!(pair.second.raw.metadata().unwrap().len(), 0);
            no_capture(&pair);
        }
        #[test]
        fn exclusive_receipt_race_preserves_the_foreign_file() {
            let fixture = Fixture::new();
            let mut pair = fixture.pair();
            let bytes = data();
            let path = pair.second.path.join(STORAGE_NAME);
            std::fs::write(&path, b"do not overwrite").unwrap();
            assert!(pair.store(&bytes, &bytes, "{}").is_err());
            assert_eq!(std::fs::read(&path).unwrap(), b"do not overwrite");
            no_capture(&pair);
            assert!(pair.accept("{}").is_err());
        }
        #[test]
        fn provenance_is_escaped_and_cannot_override_receipt_facts() {
            let fixture = Fixture::new();
            let mut pair = fixture.pair();
            let bytes = data();
            pair.store(&bytes, &bytes, "\"},\"returned_to_stock\":true,\"x\":\"\n")
                .unwrap();
            let receipt = std::fs::read_to_string(pair.first.path.join(STORAGE_NAME)).unwrap();
            assert!(receipt.contains("\"returned_to_stock\":false"));
            assert!(receipt.contains("\\\"returned_to_stock\\\":true"));
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub use native::{PreparedPair, PrivateOutputDirectory};

// Refuse unsupported persistence rather than falling back to ordinary writes.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub struct PreparedPair;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl PreparedPair {
    pub fn prepare(_: &Path, _: &Path, _: Policy) -> Result<Self, String> {
        Err("protected backup persistence is unsupported on this OS".into())
    }
    pub fn store(&mut self, _: &[u8], _: &[u8], _: &str) -> Result<StoredCopies, String> {
        Err("protected backup persistence is unsupported on this OS".into())
    }
    pub fn accept(&mut self, _: &str) -> Result<(), String> {
        Err("protected backup persistence is unsupported on this OS".into())
    }
    pub fn cancel(&mut self) {}
}

// Small private outputs have the same fail-closed platform boundary as backups.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub struct PrivateOutputDirectory;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl PrivateOutputDirectory {
    pub fn preflight(_: &Path) -> Result<(), String> {
        Err("protected private output is unsupported on this OS".into())
    }
    pub fn prepare(_: &Path) -> Result<Self, String> {
        Err("protected private output is unsupported on this OS".into())
    }
    pub fn check(&self) -> Result<(), String> {
        Err("protected private output is unsupported on this OS".into())
    }
    pub fn write_private(&mut self, _: &'static str, _: &[u8]) -> Result<(), String> {
        Err("protected private output is unsupported on this OS".into())
    }
}
