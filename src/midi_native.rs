// SPDX-License-Identifier: MIT
//! Native MIDI adapter. Never compiled into test/offline executables.
use crate::midi_ota::{self as midi, Io, Result, Selection};
use midir_ota::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver},
};
use std::time::{Duration, Instant};

#[cfg(target_os = "macos")]
#[path = "midi_macos.rs"]
mod macos;

pub struct Native {
    epoch: Instant,
    input: Option<MidiInputConnection<()>>,
    output: Option<MidiOutputConnection>,
    queue: Option<Receiver<Vec<u8>>>,
    overflow: Arc<AtomicBool>,
}
impl Native {
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
            input: None,
            output: None,
            queue: None,
            overflow: Arc::new(AtomicBool::new(false)),
        }
    }
}
impl Io for Native {
    fn now(&self) -> Duration {
        self.epoch.elapsed()
    }
    fn pause(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
    fn disconnect(&mut self) {
        self.input.take();
        self.output.take();
        self.queue.take();
    }
    fn connect(&mut self, selection: &Selection) -> Result<()> {
        self.disconnect();
        #[cfg(target_os = "macos")]
        macos::refresh_endpoints();
        let mut input = MidiInput::new("M-Flasher OTA input").map_err(|e| e.to_string())?;
        input.ignore(Ignore::None);
        let output = MidiOutput::new("M-Flasher OTA output").map_err(|e| e.to_string())?;
        let ins = input.ports();
        let outs = output.ports();
        let names_in: Vec<_> = ins
            .iter()
            .map(|p| input.port_name(p).unwrap_or_default())
            .collect();
        let names_out: Vec<_> = outs
            .iter()
            .map(|p| output.port_name(p).unwrap_or_default())
            .collect();
        let selected = midi::pair(&names_in, &names_out, selection)?;
        let ip = &ins[names_in
            .iter()
            .position(|s| s == &selected.input)
            .ok_or("input disappeared")?];
        let op = &outs[names_out
            .iter()
            .position(|s| s == &selected.output)
            .ok_or("output disappeared")?];
        let (sender, receiver) = mpsc::sync_channel(128);
        self.overflow = Arc::new(AtomicBool::new(false));
        let overflow = self.overflow.clone();
        let connection = input
            .connect(
                ip,
                "M-Flasher OTA",
                move |_, data, _| {
                    if data.first() == Some(&0xf0)
                        && (data.len() > 640 || sender.try_send(data.to_vec()).is_err())
                    {
                        overflow.store(true, Ordering::Relaxed);
                    }
                },
                (),
            )
            .map_err(|e| e.to_string())?;
        self.output = Some(
            output
                .connect(op, "M-Flasher OTA")
                .map_err(|e| e.to_string())?,
        );
        self.input = Some(connection);
        self.queue = Some(receiver);
        Ok(())
    }
    fn send(&mut self, wire: &[u8]) -> Result<()> {
        self.output
            .as_mut()
            .ok_or("MIDI output closed")?
            .send(wire)
            .map_err(|e| format!("MIDI send failed: {e}"))
    }
    fn receive(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>> {
        if self.overflow.load(Ordering::Relaxed) {
            return Err("MIDI receive queue overflow; stopped".into());
        }
        match self
            .queue
            .as_ref()
            .ok_or("MIDI input closed")?
            .recv_timeout(timeout)
        {
            Ok(data) => Ok(Some(data)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(_) => Err(
                "MIDI input disconnected; rerun the same package to finish an interrupted update"
                    .into(),
            ),
        }
    }
}
