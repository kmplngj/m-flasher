// SPDX-License-Identifier: MIT
//! Service CoreMIDI's process-local endpoint cache before re-enumeration.
use std::ffi::c_void;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFRunLoopDefaultMode: *const c_void;
    fn CFRunLoopRunInMode(mode: *const c_void, seconds: f64, return_after_source: u8) -> i32;
}

pub(super) fn refresh_endpoints() {
    // CoreMIDI delivers endpoint changes on the thread that created its first
    // client. This synchronous CLI creates and reconnects on that same thread.
    // Sleeping or creating another midir client does not refresh the cache.
    // Give pending changes a bounded window before reading sources/destinations.
    unsafe {
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.01, 0);
    }
}
