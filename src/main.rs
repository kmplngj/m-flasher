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
    // macOS: the root step the app runs through the authorization dialog
    #[cfg(target_os = "macos")]
    if args.len() == 2 && args[0] == "--take-unit" {
        std::process::exit(match scsi::take_from_disk_driver(&args[1]) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{e}");
                1
            }
        });
    }
    if !args.is_empty() {
        #[cfg(windows)]
        unsafe {
            // a windows-subsystem app has no console: use the one it was started from
            windows_sys::Win32::System::Console::AttachConsole(windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS);
        }
        std::process::exit(cli::run(args));
    }
    if let Err(e) = ui::run() {
        eprintln!("cannot open the window: {e}");
        std::process::exit(1);
    }
}
