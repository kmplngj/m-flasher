//! Independent literal FMR1 wire fixtures; no platform adapter is linked.

use crate::fmr_reader::{Error, Reader, Transport};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

const BUILD: [u8; 4] = [0x38, 0x30, 0xfc, 0x94];
const IDENTIFY_CDB: [u8; 16] = [0xfc, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const CHIP_CDB: [u8; 16] = [0xfc, 0x0b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const READ_CDB: [u8; 16] = [0xfd, 0x05, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const IDENTITY: [u8; 16] = [
    0xfc, 0x40, b'F', b'M', b'R', b'1', 1, 0, 0x38, 0x30, 0xfc, 0x94, 0, 0, 0, 0,
];
const CHIP: [u8; 16] = [
    0xfc, 0x0b, 0x85, 0x60, 0x14, 0, 0, 0x10, 0, 0, 1, 0, 0, 0, 0, 0,
];

type Trace = Rc<RefCell<Vec<([u8; 16], usize)>>>;
struct Fake {
    replies: VecDeque<Result<Vec<u8>, String>>,
    trace: Trace,
}
impl Transport for Fake {
    fn transaction(&mut self, cdb: &[u8; 16], expected_len: usize) -> Result<Vec<u8>, String> {
        self.trace.borrow_mut().push((*cdb, expected_len));
        self.replies.pop_front().expect("unexpected extra command")
    }
}
fn identity(count: u8) -> Vec<u8> {
    let mut out = IDENTITY.to_vec();
    out[15] = count;
    out
}
fn block() -> Vec<u8> {
    (0..=255).collect()
}
fn success() -> Vec<Result<Vec<u8>, String>> {
    vec![
        Ok(identity(0)),
        Ok(CHIP.to_vec()),
        Ok(block()),
        Ok(identity(1)),
        Ok(block()),
        Ok(identity(2)),
    ]
}
fn reader(replies: Vec<Result<Vec<u8>, String>>) -> (Reader<Fake>, Trace) {
    let trace = Rc::new(RefCell::new(vec![]));
    let transport = Fake {
        replies: replies.into(),
        trace: trace.clone(),
    };
    (Reader::new(transport, BUILD), trace)
}
fn terminal(r: &mut Reader<Fake>, trace: &Trace, expected_calls: usize) {
    assert_eq!(trace.borrow().len(), expected_calls);
    assert_eq!(r.read_twice(), Err(Error::Terminal));
    assert_eq!(
        trace.borrow().len(),
        expected_calls,
        "terminal session emitted a command"
    );
}

#[test]
fn exact_literal_commands_consume_two_fresh_blocks_once() {
    let (mut r, trace) = reader(success());
    let result = r.read_twice().unwrap();
    assert_eq!(result.first.as_slice(), block());
    assert_eq!(result.second.as_slice(), block());
    assert_eq!(result.build_id, BUILD);
    assert_eq!(result.jedec, [0x85, 0x60, 0x14]);
    assert_eq!(result.capacity, 1_048_576);
    assert_eq!(result.completed_dma, 2);
    assert_eq!(
        *trace.borrow(),
        vec![
            (IDENTIFY_CDB, 16),
            (CHIP_CDB, 16),
            (READ_CDB, 256),
            (IDENTIFY_CDB, 16),
            (READ_CDB, 256),
            (IDENTIFY_CDB, 16)
        ]
    );
    terminal(&mut r, &trace, 6);
}

#[test]
fn transport_failure_at_every_phase_stops_without_retry_or_followup() {
    for phase in 0..6 {
        let mut replies = success();
        replies[phase] = Err("injected short CSW / timeout / transfer failure".into());
        let (mut r, trace) = reader(replies);
        assert!(matches!(r.read_twice(), Err(Error::Transport(_))));
        terminal(&mut r, &trace, phase + 1);
    }
}

#[test]
fn short_or_long_payload_at_every_phase_is_terminal() {
    for phase in 0..6 {
        for delta in [-1, 1] {
            let mut replies = success();
            let value = replies[phase].as_mut().unwrap();
            if delta < 0 {
                value.pop();
            } else {
                value.push(0);
            }
            let (mut r, trace) = reader(replies);
            assert!(matches!(
                r.read_twice(),
                Err(Error::UnexpectedLength { .. })
            ));
            terminal(&mut r, &trace, phase + 1);
        }
    }
}

#[test]
fn every_identity_field_is_checked_at_every_identity_phase() {
    for phase in [0, 3, 5] {
        for byte in 0..16 {
            let mut replies = success();
            replies[phase].as_mut().unwrap()[byte] ^= 1;
            let (mut r, trace) = reader(replies);
            assert!(
                matches!(r.read_twice(), Err(Error::Identity(_))),
                "phase {phase}, byte {byte}"
            );
            terminal(&mut r, &trace, phase + 1);
        }
    }
}

#[test]
fn every_chip_metadata_field_and_reserved_byte_is_checked() {
    for byte in 0..16 {
        let mut replies = success();
        replies[1].as_mut().unwrap()[byte] ^= 1;
        let (mut r, trace) = reader(replies);
        assert!(matches!(r.read_twice(), Err(Error::Chip(_))), "byte {byte}");
        terminal(&mut r, &trace, 2);
    }
}

#[test]
fn unknown_status_values_and_stale_dma_counts_are_terminal() {
    for phase in [0, 3, 5] {
        for status in 1..=255 {
            let mut replies = success();
            replies[phase].as_mut().unwrap()[7] = status;
            let (mut r, trace) = reader(replies);
            assert_eq!(r.read_twice(), Err(Error::Identity("reader status")));
            terminal(&mut r, &trace, phase + 1);
        }
    }
    for phase in [3, 5] {
        let mut replies = success();
        replies[phase].as_mut().unwrap()[15] = 0;
        let (mut r, trace) = reader(replies);
        assert_eq!(r.read_twice(), Err(Error::Identity("completed DMA count")));
        terminal(&mut r, &trace, phase + 1);
    }
}

#[test]
fn every_read_byte_is_compared_before_any_further_command() {
    for offset in 0..256 {
        let mut replies = success();
        replies[4].as_mut().unwrap()[offset] ^= 1;
        let (mut r, trace) = reader(replies);
        assert_eq!(r.read_twice(), Err(Error::ReadMismatch { offset }));
        terminal(&mut r, &trace, 5);
    }
}

#[test]
fn selected_build_identity_cannot_silently_fall_back() {
    let trace = Rc::new(RefCell::new(vec![]));
    let fake = Fake {
        replies: success().into(),
        trace: trace.clone(),
    };
    let mut r = Reader::new(fake, [0; 4]);
    assert_eq!(r.read_twice(), Err(Error::Identity("build ID")));
    terminal(&mut r, &trace, 1);
}
