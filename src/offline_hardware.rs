//! Compile-time replacement for every native adapter in offline tests.
//! This module never imports an OS hardware API or constructs a native handle.

use super::{Data, Scsi, ScsiError};

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Inventory,
    Open,
    Ioctl,
    Midi,
    Usb,
}

/// Fail before an adapter can perform any OS operation. New hardware routes
/// must use the same boundary, or remain unavailable in an offline build.
pub fn deny(operation: Operation) -> Result<(), ScsiError> {
    Err(ScsiError::Failed(format!(
        "offline hardware access forbidden: {operation:?}"
    )))
}

pub struct Device {
    _private: (),
}

impl Device {
    pub fn open(_path: &str) -> Result<Device, ScsiError> {
        deny(Operation::Open)?;
        unreachable!("the offline guard always refuses hardware access")
    }
}

impl Scsi for Device {
    fn execute(&mut self, _cdb: &[u8], _data: Data) -> Result<usize, ScsiError> {
        deny(Operation::Ioctl)?;
        unreachable!("the offline guard always refuses hardware access")
    }
}

pub fn find_devices() -> Vec<(String, String)> {
    panic!("{}", deny(Operation::Inventory).unwrap_err());
}

#[cfg(test)]
mod offline_guard_tests {
    use super::*;

    #[test]
    fn every_forbidden_operation_is_refused() {
        for operation in [
            Operation::Inventory,
            Operation::Open,
            Operation::Ioctl,
            Operation::Midi,
            Operation::Usb,
        ] {
            let message = deny(operation).unwrap_err().to_string();
            assert!(message.starts_with("offline hardware access forbidden:"));
            assert!(message.contains(&format!("{operation:?}")));
        }
    }

    #[test]
    #[should_panic(expected = "offline hardware access forbidden: Inventory")]
    fn actual_inventory_entrypoint_fails_loudly() {
        find_devices();
    }

    #[test]
    fn actual_open_entrypoint_fails_before_using_path() {
        // Deliberately invalid, with no filesystem existence check.
        let result = Device::open("offline://must-never-be-opened");
        assert!(matches!(result, Err(ScsiError::Failed(message)) if message.ends_with("Open")));
    }

    #[test]
    fn actual_command_entrypoint_preserves_the_caller_buffer() {
        let mut device = Device { _private: () };
        let mut bytes = [0xA5; 16];
        let result = device.execute(&[], Data::In(&mut bytes));
        assert!(matches!(result, Err(ScsiError::Failed(message)) if message.ends_with("Ioctl")));
        assert_eq!(bytes, [0xA5; 16]);
    }
}
