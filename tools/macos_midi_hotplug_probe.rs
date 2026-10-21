// SPDX-License-Identifier: MIT
//! Opt-in macOS diagnostic, excluded from all offline test runners.
//! Enumerates CoreMIDI sources and creates one temporary virtual source. Never
//! opens physical endpoints or sends MIDI. Run only in an authorized diagnostic
//! task; see docs/midi-ota.md. Uses the production endpoint-cache refresh helper.
#![allow(non_upper_case_globals)]

#[path = "../src/midi_macos.rs"]
mod macos;

use std::ffi::{c_char, c_void};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const UTF8: u32 = 0x0800_0100;
const NAME: &str = "CoreMIDI investigation virtual hotplug";

#[link(name = "CoreMIDI", kind = "framework")]
unsafe extern "C" {
    fn MIDIClientCreate(
        name: *const c_void,
        notify: *const c_void,
        context: *mut c_void,
        client: *mut u32,
    ) -> i32;
    fn MIDIClientDispose(client: u32) -> i32;
    fn MIDISourceCreate(client: u32, name: *const c_void, source: *mut u32) -> i32;
    fn MIDIEndpointDispose(source: u32) -> i32;
    fn MIDIGetNumberOfSources() -> usize;
    fn MIDIGetSource(index: usize) -> u32;
    fn MIDIObjectGetStringProperty(
        object: u32,
        property: *const c_void,
        value: *mut *const c_void,
    ) -> i32;
    static kMIDIPropertyName: *const c_void;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        allocator: *const c_void,
        value: *const c_char,
        encoding: u32,
    ) -> *const c_void;
    fn CFStringGetCString(
        value: *const c_void,
        buffer: *mut c_char,
        size: isize,
        encoding: u32,
    ) -> u8;
    fn CFRelease(value: *const c_void);
}

fn cf_string(value: &str) -> *const c_void {
    let value = std::ffi::CString::new(value).unwrap();
    let result = unsafe { CFStringCreateWithCString(std::ptr::null(), value.as_ptr(), UTF8) };
    assert!(!result.is_null());
    result
}

struct Client(u32);
impl Client {
    fn new() -> Self {
        let name = cf_string("Read-only CoreMIDI hotplug diagnostic");
        let mut client = 0;
        let status =
            unsafe { MIDIClientCreate(name, std::ptr::null(), std::ptr::null_mut(), &mut client) };
        unsafe { CFRelease(name) };
        assert_eq!(status, 0, "MIDIClientCreate failed");
        Self(client)
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        unsafe { MIDIClientDispose(self.0) };
    }
}

struct Publisher(Child);
impl Drop for Publisher {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn visible() -> bool {
    for index in 0..unsafe { MIDIGetNumberOfSources() } {
        let mut name = std::ptr::null();
        let status = unsafe {
            MIDIObjectGetStringProperty(MIDIGetSource(index), kMIDIPropertyName, &mut name)
        };
        if status != 0 || name.is_null() {
            continue;
        }
        let mut buffer = [0u8; 256];
        let copied = unsafe {
            CFStringGetCString(
                name,
                buffer.as_mut_ptr().cast(),
                buffer.len() as isize,
                UTF8,
            )
        };
        unsafe { CFRelease(name) };
        if copied != 0 && buffer.split(|b| *b == 0).next() == Some(NAME.as_bytes()) {
            return true;
        }
    }
    false
}

fn main() {
    let client = Client::new();
    if std::env::args().any(|a| a == "--publisher") {
        let name = cf_string(NAME);
        let mut source = 0;
        let status = unsafe { MIDISourceCreate(client.0, name, &mut source) };
        unsafe { CFRelease(name) };
        assert_eq!(status, 0, "MIDISourceCreate failed");
        println!("ready");
        std::io::stdout().flush().unwrap();
        let mut done = String::new();
        std::io::stdin().read_line(&mut done).unwrap();
        unsafe { MIDIEndpointDispose(source) };
        return;
    }
    assert!(!visible(), "another probe is already running");
    let mut publisher = Publisher(
        Command::new(std::env::current_exe().unwrap())
            .arg("--publisher")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut ready = String::new();
    BufReader::new(publisher.0.stdout.take().unwrap())
        .read_line(&mut ready)
        .unwrap();
    assert_eq!(ready.trim(), "ready");
    std::thread::sleep(Duration::from_millis(200));
    // The native reconnect closes both old clients before looking for the loader.
    drop(client);
    // Recreate clients exactly as reconnect does, without running the event loop.
    let mut before = false;
    for _ in 0..3 {
        let _another = Client::new();
        before |= visible();
        std::thread::sleep(Duration::from_millis(50));
    }
    println!("visible after polling/recreating clients: {before}");
    assert!(!before, "baseline no longer reproduces stale discovery");
    if !std::env::args().any(|a| a == "--without-fix") {
        macos::refresh_endpoints();
    }
    let after = visible();
    println!("visible after production refresh: {after}");
    writeln!(publisher.0.stdin.take().unwrap(), "done").unwrap();
    assert!(publisher.0.wait().unwrap().success());
    macos::refresh_endpoints();
    assert!(!visible(), "virtual endpoint cleanup failed");
    assert!(
        after,
        "new endpoint stayed invisible; loader reconnect would time out"
    );
}
