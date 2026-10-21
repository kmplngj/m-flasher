//! One selected nusb handle graph for the bounded FMR1 BOT route.
//! Registry/sysfs discovery reads metadata; only the selected object is opened.
//! macOS performs one explicit selected driver-ownership capture before open.
//! No device reset, halt clearing, configuration change or transaction retry.

#![cfg(all(
    feature = "reader-qualification",
    not(any(test, feature = "offline-tests")),
    any(target_os = "macos", target_os = "linux")
))]

use crate::fmr_bot::Bulk;
use nusb::descriptors::{ConfigurationDescriptor, TransferType};
use nusb::transfer::{Buffer, Bulk as UsbBulk, Completion, Direction, EndpointDirection, In, Out};
use nusb::{Device, DeviceInfo, Endpoint, Interface, MaybeFuture};
use std::time::{Duration, Instant};

const VID: u16 = 0x4c4a;
const PID: u16 = 0x8057;
const EP_IN: u8 = 0x81;
const EP_OUT: u8 = 0x01;
const PACKET: usize = 64;
const CANCEL_DRAIN: Duration = Duration::from_millis(100);

pub struct Selection {
    pub device: String,
    pub location: u32,
    /// Caller-supplied stock label; never assumed to be a ROM USB serial.
    pub serial: String,
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub device: String,
    pub location: u32,
    pub registry_entry_id: Option<u64>,
    pub stock_serial_label: String,
    pub rom_serial: Option<String>,
    pub selected_id: String,
    pub bus: u8,
    pub address: u8,
    pub ports: Vec<u8>,
    pub configuration: u8,
    pub interface: u8,
    pub endpoint_in: u8,
    pub endpoint_out: u8,
    pub descriptor: String,
    pub configuration_descriptor: String,
    pub initial_capture: bool,
}

pub struct Native {
    // Endpoints drop before interface/device; pending transfers retain ownership
    // through their callbacks. A fully completed normal drop issues no cancel.
    input: Endpoint<UsbBulk, In>,
    output: Endpoint<UsbBulk, Out>,
    interface: Interface,
    device: Device,
    _selected: DeviceInfo,
    descriptor: Vec<u8>,
    binding: Binding,
    started: Instant,
    terminal: bool,
    // Last field: release only the capture COM reference after all IO handles.
    #[cfg(target_os = "macos")]
    _capture: Option<Capture>,
}

#[cfg(target_os = "macos")]
impl Drop for Native {
    fn drop(&mut self) {
        if self.input.pending() != 0 || self.output.pending() != 0 {
            // Pending nusb callbacks can retain interface/device handles beyond
            // this struct. Keep the capture COM lease until this one-shot
            // process exits; do not release ownership while callbacks remain.
            if let Some(capture) = self._capture.take() {
                std::mem::forget(capture);
            }
        }
    }
}

#[cfg(target_os = "macos")]
struct Capture(std::ptr::NonNull<std::ffi::c_void>);

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn fmr_capture_selected(
        registry_id: u64,
        location: u32,
        address: u8,
        lease_out: *mut *mut std::ffi::c_void,
    ) -> i32;
    fn fmr_release_capture_object(lease: *mut std::ffi::c_void);
}

#[cfg(target_os = "macos")]
impl Capture {
    fn selected(info: &DeviceInfo) -> Result<Self, String> {
        let mut lease = std::ptr::null_mut();
        // SAFETY: this is a live output word. The C routine checks root, exact
        // registry ID/type/VID/PID/location/address before the single capture.
        // A successful lease is owned here and dropped after selected handles.
        let code = unsafe {
            fmr_capture_selected(
                info.registry_entry_id(),
                info.location_id(),
                info.device_address(),
                &mut lease,
            )
        };
        if code != 0 {
            return Err(format!(
                "FMR1 initial selected capture failed: 0x{:08X}; no RAM upload or retry",
                code as u32
            ));
        }
        std::ptr::NonNull::new(lease)
            .map(Self)
            .ok_or_else(|| "FMR1 capture returned no ownership lease; no RAM upload".into())
    }
}

#[cfg(target_os = "macos")]
impl Drop for Capture {
    fn drop(&mut self) {
        // SAFETY: exactly the successful owned COM pointer from the C routine.
        // This only releases a reference, never the capture state/reset mask.
        unsafe {
            fmr_release_capture_object(self.0.as_ptr());
        }
    }
}

fn access_error(operation: &str, error: impl std::fmt::Display) -> String {
    format!(
        "FMR1 {operation}: {error}; selected object/interface access failed; no retry or recovery was attempted"
    )
}

fn parse_path(path: &str) -> Result<(u8, u8), String> {
    let (bus, address) = path
        .strip_prefix("usb:")
        .and_then(|s| s.split_once('-'))
        .ok_or("selected ROM device must have canonical usb:BUS-ADDRESS form")?;
    let bus: u8 = bus.parse().map_err(|_| "invalid selected USB bus")?;
    let address: u8 = address
        .parse()
        .map_err(|_| "invalid selected USB address")?;
    if address == 0 || address > 127 || path != format!("usb:{bus}-{address}") {
        return Err("selected ROM device path is not canonical".into());
    }
    Ok((bus, address))
}

fn matches_location(info: &DeviceInfo, selection: &Selection, bus: u8) -> bool {
    #[cfg(target_os = "macos")]
    {
        selection.location != 0
            && info.location_id() == selection.location
            && (selection.location >> 24) as u8 == bus
    }
    #[cfg(target_os = "linux")]
    {
        selection.location == 0 && info.busnum() == bus
    }
}

fn validate_configuration(config: &ConfigurationDescriptor<'_>) -> Result<(), String> {
    if config.configuration_value() != 1 || config.num_interfaces() != 1 {
        return Err("selected ROM must have configuration 1 and one BOT interface".into());
    }
    let interfaces: Vec<_> = config.interface_alt_settings().collect();
    if interfaces.len() != 1 {
        return Err("selected ROM interface/alternate setting is ambiguous".into());
    }
    let interface = &interfaces[0];
    if (
        interface.interface_number(),
        interface.alternate_setting(),
        interface.class(),
        interface.subclass(),
        interface.protocol(),
        interface.num_endpoints(),
    ) != (0, 0, 8, 6, 0x50, 2)
    {
        return Err("selected ROM interface must be 0/alt0, class 08/06/50, two endpoints".into());
    }
    let endpoints: Vec<_> = interface.endpoints().collect();
    if endpoints.len() != 2 {
        return Err("selected ROM BOT endpoint count is not two".into());
    }
    for (address, direction) in [(EP_IN, Direction::In), (EP_OUT, Direction::Out)] {
        let matches: Vec<_> = endpoints
            .iter()
            .filter(|ep| ep.address() == address)
            .collect();
        if matches.len() != 1
            || matches[0].direction() != direction
            || matches[0].transfer_type() != TransferType::Bulk
            || matches[0].max_packet_size() != PACKET
        {
            return Err("selected ROM BOT endpoints must be unique 64-byte bulk 01/81".into());
        }
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn open_selected(selection: &Selection) -> Result<Native, String> {
    let (bus, address) = parse_path(&selection.device)?;
    #[cfg(target_os = "linux")]
    if selection.location != 0 {
        return Err("macOS location binding is unsupported on Linux".into());
    }
    // nusb's macOS enumeration reads IORegistry properties; Linux reads sysfs.
    // Opening resolves the selected saved registry ID/path, never a first VID/PID.
    let devices = nusb::list_devices()
        .wait()
        .map_err(|e| access_error("registry/sysfs lookup", e))?;
    let mut matches = devices
        .filter(|info| info.device_address() == address && matches_location(info, selection, bus));
    let selected = matches
        .next()
        .ok_or("selected ROM location/address is absent")?;
    if matches.next().is_some() {
        return Err("selected ROM location/address is ambiguous".into());
    }
    if selected.vendor_id() != VID
        || selected.product_id() != PID
        || selected.port_chain().is_empty()
    {
        return Err("selected ROM object must be 4c4a:8057 with a physical port chain".into());
    }
    #[cfg(target_os = "macos")]
    let registry_entry_id = Some(selected.registry_entry_id());
    #[cfg(target_os = "linux")]
    let registry_entry_id = None;
    if registry_entry_id == Some(0) {
        return Err("selected ROM registry ID is invalid".into());
    }
    #[cfg(target_os = "macos")]
    let capture = Capture::selected(&selected)?;
    #[cfg(target_os = "macos")]
    let selected = {
        // Capture is an explicit initial driver-ownership operation, never an
        // error fallback. It is expected to preserve this ROM registry object.
        // Reconcile metadata before opening/uploading; never follow a new ID.
        let devices = nusb::list_devices()
            .wait()
            .map_err(|e| access_error("post-capture registry metadata", e))?;
        let mut matches = devices.filter(|info| {
            info.device_address() == address && matches_location(info, selection, bus)
        });
        let current = matches
            .next()
            .ok_or("selected ROM absent after capture; no RAM upload")?;
        if matches.next().is_some()
            || current.id() != selected.id()
            || current.vendor_id() != selected.vendor_id()
            || current.product_id() != selected.product_id()
            || current.device_version() != selected.device_version()
            || current.usb_version() != selected.usb_version()
            || current.port_chain() != selected.port_chain()
        {
            return Err(
                "selected ROM object/metadata drifted during capture; no RAM upload".into(),
            );
        }
        current
    };
    let device = selected
        .open()
        .wait()
        .map_err(|e| access_error("open selected ROM object", e))?;
    let descriptor = device.device_descriptor();
    if descriptor.vendor_id() != VID
        || descriptor.product_id() != PID
        || descriptor.device_version() != selected.device_version()
        || descriptor.usb_version() != selected.usb_version()
        || descriptor.num_configurations() != 1
        || device.configurations().count() != 1
    {
        return Err(
            "selected ROM cached device descriptor differs from selection/required geometry".into(),
        );
    }
    let configuration = device
        .active_configuration()
        .map_err(|e| access_error("initial cached configuration", e))?;
    validate_configuration(&configuration)?;
    let descriptor_bytes = descriptor.as_bytes().to_vec();
    let binding = Binding {
        device: selection.device.clone(),
        location: selection.location,
        registry_entry_id,
        stock_serial_label: selection.serial.clone(),
        // Cached OS metadata only: no USB string request and no stock equality assumption.
        rom_serial: selected.serial_number().map(str::to_owned),
        selected_id: format!("{:?}", selected.id()),
        bus,
        address,
        ports: selected.port_chain().to_vec(),
        configuration: configuration.configuration_value(),
        interface: 0,
        endpoint_in: EP_IN,
        endpoint_out: EP_OUT,
        descriptor: hex(&descriptor_bytes),
        configuration_descriptor: hex(configuration.as_bytes()),
        initial_capture: cfg!(target_os = "macos"),
    };
    // Exactly one ordinary claim; busy/driver-owned objects fail closed.
    let interface = device
        .claim_interface(0)
        .wait()
        .map_err(|e| access_error("claim selected ROM interface", e))?;
    if interface.interface_number() != 0 || interface.get_alt_setting() != 0 {
        return Err("selected ROM claimed interface/alternate setting changed".into());
    }
    let actual = interface
        .descriptor()
        .ok_or("selected ROM claimed interface descriptor absent")?;
    let expected = configuration
        .interface_alt_settings()
        .next()
        .ok_or("selected ROM interface absent")?;
    if actual.as_bytes() != expected.as_bytes() {
        return Err("selected ROM claimed interface descriptor changed".into());
    }
    let input = interface
        .endpoint::<UsbBulk, In>(EP_IN)
        .map_err(|e| access_error("selected bulk IN", e))?;
    let output = interface
        .endpoint::<UsbBulk, Out>(EP_OUT)
        .map_err(|e| access_error("selected bulk OUT", e))?;
    let mut native = Native {
        input,
        output,
        interface,
        device,
        _selected: selected,
        descriptor: descriptor_bytes,
        binding,
        started: Instant::now(),
        terminal: false,
        #[cfg(target_os = "macos")]
        _capture: Some(capture),
    };
    native.check_binding()?;
    Ok(native)
}

/// One submission, finite wait, one explicit cancellation request for an expired
/// pending transfer. Drop also cancels if it remains pending. Never use
/// nusb::transfer_blocking: its cancel drain loops.
fn transfer<D: EndpointDirection>(
    endpoint: &mut Endpoint<UsbBulk, D>,
    buffer: Buffer,
    timeout: Duration,
) -> Result<Completion, String> {
    if timeout.is_zero() || endpoint.pending() != 0 {
        return Err("FMR1 endpoint timeout is zero or a previous transfer is pending".into());
    }
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or("FMR1 endpoint deadline overflow")?;
    endpoint.submit(buffer);
    if let Some(remaining) = deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
    {
        if let Some(completion) = endpoint.wait_next_complete(remaining) {
            if Instant::now() >= deadline {
                return Err(format!(
                    "FMR1 endpoint deadline expired; late status {:?}, actual_len {}",
                    completion.status, completion.actual_len
                ));
            }
            return Ok(completion);
        }
    }
    // Cancellation is terminal cleanup, not retry/recovery. macOS AbortPipe
    // clears a controller halted bit; it does not issue device ClearFeature.
    // Its synchronous OS call/final callback lifetime is not wall-clock bounded.
    if endpoint.pending() != 0 {
        endpoint.cancel_all();
        if let Some(completion) = endpoint.wait_next_complete(CANCEL_DRAIN) {
            return Err(format!(
                "FMR1 endpoint deadline expired; drained status {:?}, actual_len {}",
                completion.status, completion.actual_len
            ));
        }
    }
    Err("FMR1 endpoint deadline expired; cancellation remains pending; no further transaction permitted".into())
}

fn completed(completion: Completion, expected: usize) -> Result<Buffer, String> {
    if let Err(error) = completion.status {
        return Err(format!(
            "FMR1 native transfer failed: {error}; actual_len {}, expected {expected}; session terminal",
            completion.actual_len
        ));
    }
    if completion.actual_len != expected || completion.buffer.len() != expected {
        return Err(format!(
            "FMR1 native count mismatch: actual_len {}, initialized {}, expected {expected}; session terminal",
            completion.actual_len,
            completion.buffer.len()
        ));
    }
    Ok(completion.buffer)
}

impl Native {
    pub fn binding(&self) -> &Binding {
        &self.binding
    }

    fn check_binding(&mut self) -> Result<(), String> {
        if self.terminal {
            return Err("FMR1 native session is terminal".into());
        }
        self.terminal = true;
        // Same owned Device/Interface/Endpoint graph throughout the session.
        // These methods inspect cached data; no lookup, reopen, string request,
        // device/configuration control request or replacement object is used.
        if self.device.device_descriptor().as_bytes() != self.descriptor.as_slice()
            || self.interface.interface_number() != 0
            || self.interface.get_alt_setting() != 0
            || self.input.endpoint_address() != EP_IN
            || self.output.endpoint_address() != EP_OUT
            || self.input.max_packet_size() != PACKET
            || self.output.max_packet_size() != PACKET
            || self.input.pending() != 0
            || self.output.pending() != 0
        {
            return Err("FMR1 retained handle/descriptor/endpoint state changed".into());
        }
        self.terminal = false;
        Ok(())
    }
    fn begin_phase(&mut self, size: usize, timeout: Duration) -> Result<(), String> {
        self.check_binding()?;
        self.terminal = true;
        if size == 0 || size > 512 || timeout.is_zero() {
            return Err("FMR1 endpoint size/timeout is outside the bounded profile".into());
        }
        Ok(())
    }
}

impl Bulk for Native {
    fn write(&mut self, bytes: &[u8], timeout: Duration) -> Result<usize, String> {
        self.begin_phase(bytes.len(), timeout)?;
        // Buffer::from(Vec) retains exactly len bytes for OUT, with no padding
        // or appended zero-length transfer. In particular the CBW stays 31 bytes.
        let completion = transfer(&mut self.output, bytes.to_vec().into(), timeout)?;
        completed(completion, bytes.len())?;
        self.terminal = false;
        Ok(bytes.len())
    }
    fn read(&mut self, bytes: &mut [u8], timeout: Duration) -> Result<usize, String> {
        self.begin_phase(bytes.len(), timeout)?;
        // nusb requires packet-aligned requested length. Preserve actual_len:
        // a 16-byte response to a logical 13-byte request is rejected, not sliced.
        let requested = bytes.len().div_ceil(PACKET) * PACKET;
        let completion = transfer(&mut self.input, Buffer::new(requested), timeout)?;
        let received = completed(completion, bytes.len())?;
        bytes.copy_from_slice(&received[..]);
        self.terminal = false;
        Ok(bytes.len())
    }
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}
