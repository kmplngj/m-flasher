// SPDX-License-Identifier: MIT
//! Small MIDI-only distribution; no ROM adapter or GUI is linked.
#![allow(dead_code)]
#[cfg(all(feature = "offline-tests", not(test)))]
compile_error!("offline-tests is test-only");
mod formats;
mod midi_ota;
mod midi_cli;
#[cfg(all(feature = "midi", not(any(test, feature = "offline-tests"))))]
mod midi_native;
fn main() {
    std::process::exit(midi_cli::run(&std::env::args().skip(1).collect::<Vec<_>>()));
}
