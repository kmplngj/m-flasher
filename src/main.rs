//! M-Flasher: back up, restore and reinstall the firmware of the M-VAVE FM-1
//! and SMK-37 Pro / Elite through the chip's USB download mode, and install the
//! open bootloader. Without arguments: the window; with arguments: the
//! command line.

#![cfg_attr(windows, windows_subsystem = "windows")]

mod boards;
mod cli;
mod controller;
mod formats;
mod jl;
mod ops;
mod scsi;
mod sectormap;
#[cfg(test)]
mod tests;
mod ui;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        #[cfg(windows)]
        unsafe {
            // a windows-subsystem app has no console: use the one it was started from
            windows_sys::Win32::System::Console::AttachConsole(windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS);
        }
        std::process::exit(cli::run(args));
    }
    #[cfg(target_os = "macos")]
    if relaunch_as_root() {
        return;
    }
    if let Err(e) = ui::run() {
        eprintln!("cannot open the window: {e}");
        std::process::exit(1);
    }
}

/// macOS lets only root take the unit from its disk driver. Opened as a normal
/// app, the window starts again as root through sudo, with the password asked
/// in a system dialog. Started by this process (not through an authorization
/// helper), the root window stays M-Flasher's own, so macOS can ask the user
/// to allow it the USB device. sudo keeps the home folder and sets SUDO_UID,
/// so the files it creates belong to the user. True: this process is done.
#[cfg(target_os = "macos")]
fn relaunch_as_root() -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};
    // SAFETY: a plain libc call without arguments
    if unsafe { libc::geteuid() } == 0 {
        return false;
    }
    let Ok(exe) = std::env::current_exe() else { return false };
    let mut prompt = "M-Flasher needs administrator rights to reach the unit's USB download mode.\\n\\nEnter your password to continue.";
    loop {
        let dialog = format!(
            "display dialog \"{prompt}\" default answer \"\" with hidden answer with title \"M-Flasher\" \
             buttons {{\"Quit\", \"OK\"}} default button \"OK\" cancel button \"Quit\""
        );
        let Ok(out) = Command::new("osascript").args(["-e", &dialog, "-e", "text returned of result"]).output() else {
            return true;
        };
        if !out.status.success() {
            return true; // Quit
        }
        let mut password = out.stdout;
        if password.last() == Some(&b'\n') {
            password.pop();
        }
        let Ok(mut child) = Command::new("/usr/bin/sudo")
            .args(["-S", "-k", "-p", ""])
            .arg(&exe)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            return true;
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(&password);
            let _ = stdin.write_all(b"\n");
        }
        password.fill(0);
        // a wrong password ends sudo within a few seconds; the window keeps running
        for _ in 0..40 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if let Ok(Some(_)) = child.try_wait() {
                break;
            }
        }
        match child.try_wait() {
            Ok(None) => return true,
            _ => prompt = "That password did not work. Enter your password to continue.",
        }
    }
}
