//! Address/pass-sensitive offline oracle. It owns two independent raw models,
//! validates every CDB, and never links a native adapter.

use crate::fmr_full::{Capture, Control, EXPECTED_DMA, READ_BUDGET, TOTAL_BUDGET, capture_full};
use crate::fmr_reader::Transport;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const CAPACITY: usize = 1_048_576;
const BLOCKS: usize = 4096;
const DMA: usize = 8194;
const COMMANDS: usize = 8265;
const ID: [u8; 16] = [
    0xfc, 0x40, b'F', b'M', b'R', b'1', 1, 0, 0x38, 0x30, 0xfc, 0x94, 0, 0, 0, 0,
];
const CHIP: [u8; 16] = [
    0xfc, 0x0b, 0x85, 0x60, 0x14, 0, 0, 0x10, 0, 0, 1, 0, 0, 0, 0, 0,
];

#[derive(Default)]
struct State {
    trace: Vec<([u8; 16], usize)>,
    progress: Vec<(u8, u32)>,
    reads: usize,
    elapsed: Duration,
    cancelled: bool,
    dropped: bool,
    fail_command: Option<usize>,
    mutate: Option<(usize, usize)>,
    short: Option<usize>,
    long: Option<usize>,
    after_command: Option<(usize, Duration, bool)>,
    progress_failure: Option<usize>,
    progress_time: Option<(usize, Duration, bool)>,
}
type Shared = Rc<RefCell<State>>;
struct Model {
    state: Shared,
    raw: Vec<u8>,
}
impl Drop for Model {
    fn drop(&mut self) {
        self.state.borrow_mut().dropped = true;
    }
}
impl Transport for Model {
    fn transaction(&mut self, cdb: &[u8; 16], length: usize) -> Result<Vec<u8>, String> {
        let mut state = self.state.borrow_mut();
        state.trace.push((*cdb, length));
        let call = state.trace.len();
        if let Some((at, elapsed, cancelled)) = state.after_command {
            if at == call {
                state.elapsed = elapsed;
                state.cancelled = cancelled;
            }
        }
        if state.fail_command == Some(call) {
            return Err("injected transaction failure".into());
        }
        let mut canonical = [0u8; 16];
        canonical[..2].copy_from_slice(&cdb[..2]);
        let mut reply = match &cdb[..2] {
            [0xfc, 0x40] => {
                assert_eq!(length, 16);
                let mut reply = ID;
                reply[12..16].copy_from_slice(&(state.reads as u32).to_be_bytes());
                reply.to_vec()
            }
            [0xfc, 0x0b] => {
                assert_eq!(length, 16);
                CHIP.to_vec()
            }
            [0xfd, 0x05] => {
                assert_eq!(length, 256);
                let (pass, address) = if state.reads < 2 {
                    (0, CAPACITY - 256)
                } else {
                    let ordinal = state.reads - 2;
                    (ordinal / BLOCKS, (ordinal % BLOCKS) * 256)
                };
                assert!(pass < 2, "extra pass/read");
                canonical[2..6].copy_from_slice(&(address as u32).to_be_bytes());
                canonical[6..8].copy_from_slice(&256u16.to_be_bytes());
                state.reads += 1;
                self.raw[pass * CAPACITY + address..pass * CAPACITY + address + 256].to_vec()
            }
            _ => panic!("forbidden/background command: {cdb:x?}"),
        };
        assert_eq!(*cdb, canonical, "CDB address/order/reserved-byte oracle");
        if let Some((at, byte)) = state.mutate {
            if at == call {
                reply[byte] ^= 1;
            }
        }
        if state.short == Some(call) {
            reply.pop();
        }
        if state.long == Some(call) {
            reply.push(0);
        }
        Ok(reply)
    }
}
struct Clock(Shared);
impl Control for Clock {
    fn elapsed(&self) -> Duration {
        self.0.borrow().elapsed
    }
    fn cancelled(&self) -> bool {
        self.0.borrow().cancelled
    }
    fn progress(&mut self, pass: u8, bytes: u32) -> Result<(), String> {
        let mut state = self.0.borrow_mut();
        state.progress.push((pass, bytes));
        let n = state.progress.len();
        if let Some((at, elapsed, cancelled)) = state.progress_time {
            if n == at {
                state.elapsed = elapsed;
                state.cancelled = cancelled;
            }
        }
        if state.progress_failure == Some(n) {
            Err("injected progress failure".into())
        } else {
            Ok(())
        }
    }
}
fn setup() -> (Model, Clock, Shared) {
    let state = Rc::new(RefCell::new(State::default()));
    // Independent 2MiB model: address-sensitive raw bytes, including all 256
    // possible values; no fixture is derived from capture output.
    let raw = (0..2 * CAPACITY)
        .map(|index| {
            let address = index % CAPACITY;
            ((address.wrapping_mul(37) ^ (address >> 8) ^ (address >> 16)) & 255) as u8
        })
        .collect();
    (
        Model {
            state: state.clone(),
            raw,
        },
        Clock(state.clone()),
        state,
    )
}
fn forbidden_persist(_: &Capture) -> Result<(), String> {
    panic!("failed capture reached storage")
}

#[test]
fn two_complete_independent_passes_have_exact_literal_transcript_and_retained_owner() {
    let (model, clock, state) = setup();
    let expected = model.raw[..CAPACITY].to_vec();
    let capture = capture_full(model, clock, |capture| {
        let state = state.borrow();
        assert!(!state.dropped);
        assert_eq!(state.reads, DMA);
        assert_eq!(state.trace.len(), COMMANDS - 2);
        assert_eq!(capture.first, expected);
        assert_eq!(capture.second, expected);
        assert_ne!(capture.first.as_ptr(), capture.second.as_ptr());
        assert_eq!(capture.completed_dma, 8194);
        // Independently computed with Python hashlib for the literal formula.
        assert_eq!(
            capture.sha256,
            "06b668491c0fc6fa31ddd03be8c71feb34b7faf533b9b4133b9ec8a243c3103c"
        );
        Ok(())
    })
    .unwrap();
    assert_eq!(capture.completed_dma, EXPECTED_DMA);
    let state = state.borrow();
    assert!(state.dropped);
    assert_eq!(state.trace.len(), COMMANDS);
    assert_eq!(state.reads, DMA);
    let mut expected_trace = vec![
        (ID_CDB, 16),
        (CHIP_CDB, 16),
        (read_cdb(0xfff00), 256),
        (ID_CDB, 16),
        (read_cdb(0xfff00), 256),
        (ID_CDB, 16),
        (CHIP_CDB, 16),
    ];
    for _pass in 0..2 {
        for block in 0..4096 {
            expected_trace.push((read_cdb(block * 256), 256));
            if (block + 1) % 256 == 0 {
                expected_trace.push((ID_CDB, 16));
                expected_trace.push((CHIP_CDB, 16));
            }
        }
    }
    expected_trace.push((ID_CDB, 16));
    expected_trace.push((CHIP_CDB, 16));
    assert_eq!(state.trace, expected_trace);
    let expected_progress: Vec<_> = (1..=2)
        .flat_map(|pass| (0..=16).map(move |n| (pass, n * 65536)))
        .collect();
    assert_eq!(state.progress, expected_progress);
}
const ID_CDB: [u8; 16] = [0xfc, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const CHIP_CDB: [u8; 16] = [0xfc, 0x0b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
fn read_cdb(address: u32) -> [u8; 16] {
    let mut cdb = [0; 16];
    cdb[..2].copy_from_slice(&[0xfd, 0x05]);
    cdb[2..6].copy_from_slice(&address.to_be_bytes());
    cdb[6..8].copy_from_slice(&[1, 0]);
    cdb
}

#[test]
fn changed_second_pass_with_successful_transport_is_rejected() {
    for offset in [0, 255, 65536, CAPACITY - 1] {
        let (mut model, clock, state) = setup();
        model.raw[CAPACITY + offset] ^= 1;
        assert!(
            capture_full(model, clock, forbidden_persist)
                .unwrap_err()
                .contains("passes differ")
        );
        assert_eq!(state.borrow().reads, DMA);
        assert_eq!(state.borrow().trace.len(), COMMANDS - 2);
    }
}

#[test]
fn changed_tail_is_rejected_before_full_capture() {
    let (model, clock, state) = setup();
    state.borrow_mut().mutate = Some((5, 255));
    assert!(
        capture_full(model, clock, forbidden_persist)
            .unwrap_err()
            .contains("tail samples")
    );
    assert_eq!(state.borrow().trace.len(), 5);
}

#[test]
fn every_metadata_byte_is_checked_initially_at_pass_checkpoint_and_after_storage() {
    // Initial identity/chip, tail DMA checkpoints, 64KiB identity/chip, final identity/chip.
    for call in [1, 2, 4, 6, 7, 264, 265, COMMANDS - 1, COMMANDS] {
        for byte in 0..16 {
            let (model, clock, state) = setup();
            state.borrow_mut().mutate = Some((call, byte));
            let stored = Rc::new(RefCell::new(false));
            let result = capture_full(model, clock, |_| {
                *stored.borrow_mut() = true;
                Ok(())
            });
            assert!(result.is_err(), "call {call}, byte {byte}");
            assert_eq!(state.borrow().trace.len(), call);
            assert_eq!(*stored.borrow(), call >= COMMANDS - 1);
        }
    }
}

#[test]
fn transport_or_payload_failure_is_terminal_without_any_cleanup_command() {
    for call in [
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
        264,
        265,
        COMMANDS - 2,
        COMMANDS - 1,
        COMMANDS,
    ] {
        for fault in 0..3 {
            let (model, clock, state) = setup();
            match fault {
                0 => state.borrow_mut().fail_command = Some(call),
                1 => state.borrow_mut().short = Some(call),
                _ => state.borrow_mut().long = Some(call),
            }
            assert!(capture_full(model, clock, |_| Ok(())).is_err());
            assert_eq!(state.borrow().trace.len(), call);
            assert!(state.borrow().dropped);
        }
    }
}

#[test]
fn initial_cancel_and_deadlines_prevent_all_commands() {
    for (elapsed, cancelled) in [
        (Duration::ZERO, true),
        (READ_BUDGET, false),
        (TOTAL_BUDGET, false),
    ] {
        let (model, clock, state) = setup();
        state.borrow_mut().elapsed = elapsed;
        state.borrow_mut().cancelled = cancelled;
        assert!(capture_full(model, clock, forbidden_persist).is_err());
        assert!(state.borrow().trace.is_empty());
    }
}

#[test]
fn transaction_deadline_and_cancel_are_checked_immediately_after_return() {
    for call in [1, 3, 8, 264, COMMANDS - 2] {
        for (elapsed, cancelled) in [(READ_BUDGET, false), (Duration::ZERO, true)] {
            let (model, clock, state) = setup();
            state.borrow_mut().after_command = Some((call, elapsed, cancelled));
            assert!(capture_full(model, clock, forbidden_persist).is_err());
            assert_eq!(state.borrow().trace.len(), call);
        }
    }
}

#[test]
fn progress_failure_timeout_or_cancel_issues_no_next_command() {
    for callback in [1, 2, 17, 18, 34] {
        for fault in 0..3 {
            let (model, clock, state) = setup();
            match fault {
                0 => state.borrow_mut().progress_failure = Some(callback),
                1 => state.borrow_mut().progress_time = Some((callback, READ_BUDGET, false)),
                _ => state.borrow_mut().progress_time = Some((callback, Duration::ZERO, true)),
            }
            assert!(capture_full(model, clock, forbidden_persist).is_err());
            assert_eq!(state.borrow().progress.len(), callback);
            assert!(state.borrow().dropped);
        }
    }
}

#[test]
fn storage_error_cancel_timeout_or_clock_reversal_emit_no_final_command() {
    for fault in 0..4 {
        let (model, clock, state) = setup();
        if fault == 3 {
            state.borrow_mut().elapsed = Duration::from_secs(1);
        }
        let result = capture_full(model, clock, |_| {
            assert!(!state.borrow().dropped);
            match fault {
                0 => return Err("disk full".into()),
                1 => state.borrow_mut().cancelled = true,
                2 => state.borrow_mut().elapsed = TOTAL_BUDGET,
                _ => state.borrow_mut().elapsed = Duration::ZERO,
            }
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(state.borrow().trace.len(), COMMANDS - 2);
        assert!(state.borrow().dropped);
    }
}

#[test]
fn storage_may_cross_read_budget_but_final_continuity_remains_total_bounded() {
    let (model, clock, state) = setup();
    let result = capture_full(model, clock, |_| {
        state.borrow_mut().elapsed = READ_BUDGET;
        Ok(())
    });
    assert!(result.is_ok());
    assert_eq!(state.borrow().trace.len(), COMMANDS);
    let (model, clock, state) = setup();
    state.borrow_mut().after_command = Some((COMMANDS - 1, TOTAL_BUDGET, false));
    assert!(capture_full(model, clock, |_| Ok(())).is_err());
    assert_eq!(state.borrow().trace.len(), COMMANDS - 1);
}
