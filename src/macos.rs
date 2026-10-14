//! macOS: the window runs as root. Only root may take the unit from macOS's
//! disk driver, and after that only root may open it.
//!
//! Opened as a normal app, it asks for the password in the system's
//! authorization dialog and runs one root step, which starts the window again
//! as root. macOS lets that window use the unit only when it descends from a
//! process of the user's (M-Flasher here, Terminal under sudo): then it asks the
//! user, once, to let M-Flasher access the removable volume (the unit). A root
//! process started any other way (osascript's administrator privileges, a
//! launchd job, or one "responsible for itself") is refused without asking.
//! AuthorizationExecuteWithPrivileges runs the root step as a child of this
//! process; it is deprecated, but nothing newer does that.

use std::ffi::{CString, c_char, c_void};
use std::os::unix::ffi::OsStrExt;

/// The root step: `M-Flasher --window-as-root UID GID USER HOME`.
pub const WINDOW_AS_ROOT: &str = "--window-as-root";

/// Not root: run the root step through the authorization dialog. True: this
/// process is done (also when the dialog was cancelled).
pub fn relaunch_as_root() -> bool {
    // SAFETY: plain libc calls without arguments
    let (euid, uid, gid) = unsafe { (libc::geteuid(), libc::getuid(), libc::getgid()) };
    if euid == 0 {
        return false;
    }
    let Ok(exe) = std::env::current_exe() else { return false };
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    let args: Vec<CString> = [WINDOW_AS_ROOT.to_string(), uid.to_string(), gid.to_string(), env("USER"), env("HOME")]
        .into_iter()
        .filter_map(|a| CString::new(a).ok())
        .collect();
    let Ok(path) = CString::new(exe.as_os_str().as_bytes()) else { return false };
    let mut argv: Vec<*const c_char> = args.iter().map(|a| a.as_ptr()).collect();
    argv.push(std::ptr::null());
    let prompt = c"M-Flasher needs administrator rights to reach the unit's USB download mode.";
    // SAFETY: Security framework calls with initialised items; every pointer
    // outlives the call it is passed to.
    unsafe {
        let mut auth = std::ptr::null_mut();
        if AuthorizationCreate(std::ptr::null(), std::ptr::null(), 0, &mut auth) != 0 {
            return true;
        }
        let mut right = AuthorizationItem {
            name: c"system.privilege.admin".as_ptr(),
            value_length: 0,
            value: std::ptr::null_mut(),
            flags: 0,
        };
        let rights = AuthorizationItemSet { count: 1, items: &mut right };
        let mut prompt_item = AuthorizationItem {
            name: c"prompt".as_ptr(),
            value_length: prompt.to_bytes().len(),
            value: prompt.as_ptr() as *mut c_void,
            flags: 0,
        };
        let environment = AuthorizationItemSet { count: 1, items: &mut prompt_item };
        // interaction allowed, extend rights, preauthorize
        let flags = 1 | 2 | 16;
        if AuthorizationCopyRights(auth, &rights, &environment, flags, std::ptr::null_mut()) == 0 {
            let mut pipe: *mut libc::FILE = std::ptr::null_mut();
            if AuthorizationExecuteWithPrivileges(auth, path.as_ptr(), 0, argv.as_ptr(), &mut pipe) == 0
                && !pipe.is_null()
            {
                // the root step ends as soon as the window has started
                let mut buf = [0u8; 256];
                while libc::fread(buf.as_mut_ptr() as *mut c_void, 1, buf.len(), pipe) > 0 {}
                libc::fclose(pipe);
            }
        }
        AuthorizationFree(auth, 0);
    }
    true
}

#[repr(C)]
struct AuthorizationItem {
    name: *const c_char,
    value_length: usize,
    value: *mut c_void,
    flags: u32,
}

#[repr(C)]
struct AuthorizationItemSet {
    count: u32,
    items: *mut AuthorizationItem,
}

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    fn AuthorizationCreate(rights: *const AuthorizationItemSet, environment: *const AuthorizationItemSet,
                           flags: u32, authorization: *mut *mut c_void) -> i32;
    fn AuthorizationCopyRights(authorization: *mut c_void, rights: *const AuthorizationItemSet,
                               environment: *const AuthorizationItemSet, flags: u32,
                               authorized: *mut *mut AuthorizationItemSet) -> i32;
    fn AuthorizationExecuteWithPrivileges(authorization: *mut c_void, path: *const c_char, options: u32,
                                          arguments: *const *const c_char, pipe: *mut *mut libc::FILE) -> i32;
    fn AuthorizationFree(authorization: *mut c_void, flags: u32) -> i32;
}

/// As root: start the window (this executable, no arguments) as root, in its
/// own session, with the user's home and
/// SUDO_UID/GID/USER (so the files it creates belong to the user); returns at
/// once. The exit code for the root step.
pub fn start_window(args: &[String]) -> i32 {
    let [uid, gid, user, home] = args else { return 2 };
    let Ok(exe) = std::env::current_exe() else { return 1 };
    let ours = ["HOME", "USER", "SUDO_UID", "SUDO_GID", "SUDO_USER"];
    let mut env: Vec<CString> = std::env::vars()
        .filter(|(k, _)| !ours.contains(&k.as_str()))
        .filter_map(|(k, v)| CString::new(format!("{k}={v}")).ok())
        .collect();
    for (k, v) in ours.iter().zip([home, user, uid, gid, user]) {
        env.extend(CString::new(format!("{k}={v}")));
    }
    let Ok(path) = CString::new(exe.as_os_str().as_bytes()) else { return 1 };
    let argv = [path.as_ptr() as *mut c_char, std::ptr::null_mut()];
    let mut envp: Vec<*mut c_char> = env.iter().map(|e| e.as_ptr() as *mut c_char).collect();
    envp.push(std::ptr::null_mut());
    // SAFETY: the attributes and file actions are initialised before use and
    // destroyed after; every pointer handed to posix_spawn outlives the call.
    unsafe {
        let mut attr: libc::posix_spawnattr_t = std::ptr::null_mut();
        libc::posix_spawnattr_init(&mut attr);
        libc::posix_spawnattr_setflags(&mut attr, POSIX_SPAWN_SETSID);
        let mut files: libc::posix_spawn_file_actions_t = std::ptr::null_mut();
        libc::posix_spawn_file_actions_init(&mut files);
        for fd in 0..3 {
            let mode = if fd == 0 { libc::O_RDONLY } else { libc::O_WRONLY };
            libc::posix_spawn_file_actions_addopen(&mut files, fd, c"/dev/null".as_ptr(), mode, 0);
        }
        let mut pid = 0;
        let r = libc::posix_spawn(&mut pid, path.as_ptr(), &files, &attr, argv.as_ptr(), envp.as_ptr());
        libc::posix_spawn_file_actions_destroy(&mut files);
        libc::posix_spawnattr_destroy(&mut attr);
        if r == 0 { 0 } else { 1 }
    }
}

/// posix_spawn: start the child in a new session (<spawn.h>).
const POSIX_SPAWN_SETSID: libc::c_short = 0x0400;
