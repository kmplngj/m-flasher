//! The download mode over plain USB (libusb), for systems that do not pass
//! SCSI commands through to a USB disk (macOS): the app takes the interface
//! from the system's mass-storage driver and speaks USB mass-storage
//! Bulk-Only Transport itself. macOS lets only root take a device from its
//! driver; it gets it back when the app lets go.

use super::{Data, ScsiError, cmd_name};
use rusb::{Direction, GlobalContext, TransferType};
use std::time::Duration;

const VID: u16 = 0x4C4A;
const PID: u16 = 0x8057;
const TIMEOUT: Duration = Duration::from_secs(10);
const CBW_SIGNATURE: u32 = 0x4342_5355; // "USBC"
const CSW_SIGNATURE: u32 = 0x5342_5355; // "USBS"

pub struct Device {
    handle: rusb::DeviceHandle<GlobalContext>,
    iface: u8,
    ep_in: u8,
    ep_out: u8,
    tag: u32,
}

fn failed(what: &str, e: rusb::Error) -> ScsiError {
    ScsiError::Failed(format!("{what}: {e}"))
}

fn needs_root(e: rusb::Error) -> ScsiError {
    ScsiError::NeedsAdmin(format!(
        "no permission to take the unit from the system's USB disk driver ({e}): {}",
        if cfg!(target_os = "macos") { "run M-Flasher with sudo" } else { "run as root or install the udev rule" }
    ))
}

fn path_of(d: &rusb::Device<GlobalContext>) -> String {
    format!("usb:{}-{}", d.bus_number(), d.address())
}

impl Device {
    pub fn open(path: &str) -> Result<Device, ScsiError> {
        let devices = rusb::devices().map_err(|e| failed("cannot list the USB devices", e))?;
        let dev = devices
            .iter()
            .find(|d| path_of(d) == path)
            .ok_or_else(|| ScsiError::Failed(format!("{path}: the unit is no longer connected")))?;
        // the mass-storage interface and its two bulk endpoints
        let config = dev.active_config_descriptor().map_err(|e| failed("cannot read the USB configuration", e))?;
        let mut found = None;
        for iface in config.interfaces() {
            for alt in iface.descriptors().filter(|a| a.class_code() == 8) {
                let bulk = |dir| {
                    alt.endpoint_descriptors()
                        .find(|e| e.transfer_type() == TransferType::Bulk && e.direction() == dir)
                        .map(|e| e.address())
                };
                if let (Some(i), Some(o)) = (bulk(Direction::In), bulk(Direction::Out)) {
                    found = Some((alt.interface_number(), i, o));
                }
            }
        }
        let (iface, ep_in, ep_out) =
            found.ok_or_else(|| ScsiError::Failed(format!("{path}: no USB mass-storage interface")))?;
        let handle = dev.open().map_err(|e| match e {
            rusb::Error::Access => needs_root(e),
            e => failed("cannot open the unit", e),
        })?;
        let _ = handle.set_auto_detach_kernel_driver(true);
        handle.claim_interface(iface).map_err(|e| match e {
            rusb::Error::Access | rusb::Error::Busy => needs_root(e),
            e => failed("cannot take the unit's USB interface", e),
        })?;
        Ok(Device { handle, iface, ep_in, ep_out, tag: 0 })
    }

    /// Bulk-Only Mass Storage Reset, then clear both endpoints' halts.
    fn reset_recovery(&mut self) {
        let _ = self.handle.write_control(0x21, 0xFF, 0, self.iface as u16, &[], TIMEOUT);
        let _ = self.handle.clear_halt(self.ep_in);
        let _ = self.handle.clear_halt(self.ep_out);
    }

    fn read_csw(&mut self) -> Result<[u8; 13], rusb::Error> {
        let mut csw = [0u8; 13];
        match self.handle.read_bulk(self.ep_in, &mut csw, TIMEOUT) {
            Ok(13) => Ok(csw),
            Ok(_) => Err(rusb::Error::Io),
            Err(rusb::Error::Pipe) => {
                // a stalled endpoint: clear it and try once more
                let _ = self.handle.clear_halt(self.ep_in);
                match self.handle.read_bulk(self.ep_in, &mut csw, TIMEOUT)? {
                    13 => Ok(csw),
                    _ => Err(rusb::Error::Io),
                }
            }
            Err(e) => Err(e),
        }
    }

    fn transfer(&mut self, cdb: &[u8], data: Data) -> Result<(), ScsiError> {
        self.tag = self.tag.wrapping_add(1);
        let (len, flags) = match &data {
            Data::None => (0, 0),
            Data::In(b) => (b.len(), 0x80),
            Data::Out(b) => (b.len(), 0),
        };
        let mut cbw = [0u8; 31];
        cbw[0..4].copy_from_slice(&CBW_SIGNATURE.to_le_bytes());
        cbw[4..8].copy_from_slice(&self.tag.to_le_bytes());
        cbw[8..12].copy_from_slice(&(len as u32).to_le_bytes());
        cbw[12] = flags;
        cbw[14] = cdb.len() as u8;
        cbw[15..15 + cdb.len()].copy_from_slice(cdb);
        let name = cmd_name(cdb);
        self.handle.write_bulk(self.ep_out, &cbw, TIMEOUT).map_err(|e| failed(&format!("sending command {name}"), e))?;
        // libusb ends a read at a short packet: the device may send less
        let phase = match data {
            Data::None => Ok(()),
            Data::In(buf) => match self.handle.read_bulk(self.ep_in, buf, TIMEOUT) {
                Ok(_) => Ok(()),
                Err(rusb::Error::Pipe) => Err(self.ep_in),
                Err(e) => return Err(failed(&format!("command {name}: reading its data"), e)),
            },
            Data::Out(buf) => match self.handle.write_bulk(self.ep_out, buf, TIMEOUT) {
                Ok(n) if n == buf.len() => Ok(()),
                Ok(n) => return Err(ScsiError::Failed(format!("command {name}: {n} of {} bytes sent", buf.len()))),
                Err(rusb::Error::Pipe) => Err(self.ep_out),
                Err(e) => return Err(failed(&format!("command {name}: sending its data"), e)),
            },
        };
        if let Err(ep) = phase {
            // the device stalled the data phase: its status follows
            let _ = self.handle.clear_halt(ep);
        }
        let csw = self.read_csw().map_err(|e| failed(&format!("command {name}: reading its status"), e))?;
        if u32::from_le_bytes(csw[0..4].try_into().unwrap()) != CSW_SIGNATURE
            || u32::from_le_bytes(csw[4..8].try_into().unwrap()) != self.tag
        {
            return Err(ScsiError::Failed(format!("command {name}: an invalid status block")));
        }
        match csw[12] {
            0 => Ok(()),
            1 => Err(ScsiError::Failed(format!("command {name} failed (status 1)"))),
            s => Err(ScsiError::Failed(format!("command {name}: phase error (status {s})"))),
        }
    }
}

impl super::Scsi for Device {
    fn execute(&mut self, cdb: &[u8], data: Data) -> Result<(), ScsiError> {
        let r = self.transfer(cdb, data);
        // a failed command (status 1) leaves the transport in order; anything
        // else is reset, for the next command (or its retry) to start clean
        if matches!(&r, Err(ScsiError::Failed(s)) if !s.ends_with("(status 1)")) {
            self.reset_recovery();
        }
        r
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // the system's driver gets the interface back (auto-detach re-attaches it)
        let _ = self.handle.release_interface(self.iface);
    }
}

/// (path, description) of the download-mode units on USB.
pub fn find_devices() -> Vec<(String, String)> {
    let Ok(devices) = rusb::devices() else { return vec![] };
    devices
        .iter()
        .filter(|d| d.device_descriptor().is_ok_and(|dd| dd.vendor_id() == VID && dd.product_id() == PID))
        .map(|d| {
            let p = path_of(&d);
            let desc = format!("WL82 UBOOT1.00 (USB bus {}, device {})", d.bus_number(), d.address());
            (p, desc)
        })
        .collect()
}
