//! Literal BOT fixtures over local memory; no native endpoint implementation.
use crate::fmr_bot::{Bot, Bulk};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;

const CDB: [u8; 16] = [0xfc, 0x40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const CBW_IN: [u8; 31] = [
    b'U', b'S', b'B', b'C', 1, 0, 0, 0, 16, 0, 0, 0, 0x80, 0, 16, 0xfc, 0x40, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0,
];
const CSW: [u8; 13] = [b'U', b'S', b'B', b'S', 1, 0, 0, 0, 0, 0, 0, 0, 0];
#[derive(Debug, PartialEq, Eq)]
enum Call {
    Write(Vec<u8>, Duration),
    Read(usize, Duration),
}
type Trace = Rc<RefCell<Vec<Call>>>;
struct Step {
    data: Vec<u8>,
    count: Result<usize, String>,
    advance: Duration,
}
struct Fake {
    script: VecDeque<Step>,
    clock: Rc<Cell<Duration>>,
    trace: Trace,
}
impl Fake {
    fn step(&mut self) -> Step {
        let step = self
            .script
            .pop_front()
            .expect("unexpected extra endpoint operation");
        self.clock.set(self.clock.get() + step.advance);
        step
    }
}
impl Bulk for Fake {
    fn write(&mut self, bytes: &[u8], timeout: Duration) -> Result<usize, String> {
        assert!(!bytes.is_empty(), "zero-length endpoint write");
        assert!(!timeout.is_zero(), "zero timeout");
        self.trace
            .borrow_mut()
            .push(Call::Write(bytes.to_vec(), timeout));
        self.step().count
    }
    fn read(&mut self, bytes: &mut [u8], timeout: Duration) -> Result<usize, String> {
        assert!(!bytes.is_empty(), "zero-length endpoint read");
        assert!(!timeout.is_zero(), "zero timeout");
        self.trace
            .borrow_mut()
            .push(Call::Read(bytes.len(), timeout));
        let step = self.step();
        let n = bytes.len().min(step.data.len());
        bytes[..n].copy_from_slice(&step.data[..n]);
        step.count
    }
    fn elapsed(&self) -> Duration {
        self.clock.get()
    }
}
fn step(data: &[u8], count: usize) -> Step {
    Step {
        data: data.to_vec(),
        count: Ok(count),
        advance: Duration::ZERO,
    }
}
fn success() -> Vec<Step> {
    vec![step(&[], 31), step(&[0xa5; 16], 16), step(&CSW, 13)]
}
fn bot(script: Vec<Step>, timeout: Duration) -> (Bot<Fake>, Trace) {
    let trace = Rc::new(RefCell::new(vec![]));
    let fake = Fake {
        script: script.into(),
        clock: Rc::new(Cell::new(Duration::ZERO)),
        trace: trace.clone(),
    };
    (Bot::new(fake, timeout).unwrap(), trace)
}
fn terminal(bot: &mut Bot<Fake>, trace: &Trace, calls: usize) {
    assert_eq!(trace.borrow().len(), calls);
    assert_eq!(bot.read(&CDB, 16), Err("BOT session is terminal".into()));
    assert_eq!(bot.write(&CDB, &[1]), Err("BOT session is terminal".into()));
    assert_eq!(
        trace.borrow().len(),
        calls,
        "terminal transport emitted endpoint operation"
    );
}

#[test]
fn canonical_literal_cbw_and_csw_and_fresh_tags() {
    let mut script = success();
    let mut second = success();
    second[2].data[4] = 2;
    script.extend(second);
    let (mut bot, trace) = bot(script, Duration::from_millis(10));
    assert_eq!(bot.read(&CDB, 16).unwrap(), vec![0xa5; 16]);
    assert_eq!(bot.read(&CDB, 16).unwrap(), vec![0xa5; 16]);
    let mut second_cbw = CBW_IN.to_vec();
    second_cbw[4] = 2;
    assert_eq!(
        *trace.borrow(),
        vec![
            Call::Write(CBW_IN.to_vec(), Duration::from_millis(10)),
            Call::Read(16, Duration::from_millis(10)),
            Call::Read(13, Duration::from_millis(10)),
            Call::Write(second_cbw, Duration::from_millis(10)),
            Call::Read(16, Duration::from_millis(10)),
            Call::Read(13, Duration::from_millis(10))
        ]
    );
}

#[test]
fn every_short_and_excess_count_is_terminal_in_each_in_phase() {
    for (phase, expected) in [(0, 31), (1, 16), (2, 13)] {
        for count in (0..expected).chain([expected + 1, usize::MAX]) {
            let mut script = success();
            script[phase].count = Ok(count);
            let (mut bot, trace) = bot(script, Duration::from_secs(1));
            assert!(bot.read(&CDB, 16).is_err());
            terminal(&mut bot, &trace, phase + 1);
        }
    }
}

#[test]
fn each_endpoint_error_is_terminal_without_recovery() {
    for phase in 0..3 {
        let mut script = success();
        script[phase].count = Err("injected endpoint error".into());
        let (mut bot, trace) = bot(script, Duration::from_secs(1));
        assert_eq!(bot.read(&CDB, 16), Err("injected endpoint error".into()));
        terminal(&mut bot, &trace, phase + 1);
    }
}

#[test]
fn every_csw_field_and_all_failed_statuses_are_rejected() {
    for byte in 0..13 {
        let mut script = success();
        script[2].data[byte] ^= 1;
        let (mut bot, trace) = bot(script, Duration::from_secs(1));
        assert!(bot.read(&CDB, 16).is_err(), "CSW byte {byte}");
        terminal(&mut bot, &trace, 3);
    }
    for status in 1..=255 {
        let mut script = success();
        script[2].data[12] = status;
        let (mut bot, trace) = bot(script, Duration::from_secs(1));
        assert!(bot.read(&CDB, 16).is_err());
        terminal(&mut bot, &trace, 3);
    }
}

#[test]
fn whole_deadline_is_not_restarted_and_positive_timeouts_round_up() {
    let mut script = success();
    script[0].advance = Duration::from_micros(900);
    script[1].advance = Duration::from_micros(900);
    script[2].advance = Duration::from_micros(100);
    let (mut bot, trace) = bot(script, Duration::from_micros(2500));
    bot.read(&CDB, 16).unwrap();
    assert_eq!(
        *trace.borrow(),
        vec![
            Call::Write(CBW_IN.to_vec(), Duration::from_millis(3)),
            Call::Read(16, Duration::from_millis(2)),
            Call::Read(13, Duration::from_millis(1))
        ]
    );
}

#[test]
fn expiry_after_any_phase_stops_even_after_successful_csw() {
    for phase in 0..3 {
        let mut script = success();
        script[phase].advance = Duration::from_millis(5);
        let (mut bot, trace) = bot(script, Duration::from_millis(5));
        assert_eq!(
            bot.read(&CDB, 16),
            Err("BOT transaction deadline expired".into())
        );
        terminal(&mut bot, &trace, phase + 1);
    }
    let mut script = success();
    script[0].advance = Duration::from_millis(3);
    script[1].advance = Duration::from_millis(2);
    let (mut bot, trace) = bot(script, Duration::from_millis(5));
    assert!(bot.read(&CDB, 16).is_err());
    terminal(&mut bot, &trace, 2);
}

#[test]
fn literal_out_cbw_data_and_csw_are_exact() {
    let cdb = [
        0xfb, 0x06, 1, 0xc2, 0, 0, 0, 3, 0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    let expected = vec![
        b'U', b'S', b'B', b'C', 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 16, 0xfb, 0x06, 1, 0xc2, 0, 0, 0, 3,
        0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    let script = vec![step(&[], 31), step(&[], 3), step(&CSW, 13)];
    let (mut bot, trace) = bot(script, Duration::from_secs(1));
    bot.write(&cdb, &[0x31, 0x4d, 0x52]).unwrap();
    assert_eq!(
        *trace.borrow(),
        vec![
            Call::Write(expected, Duration::from_secs(1)),
            Call::Write(vec![0x31, 0x4d, 0x52], Duration::from_secs(1)),
            Call::Read(13, Duration::from_secs(1))
        ]
    );
}

#[test]
fn out_phase_errors_counts_and_deadlines_are_terminal() {
    for phase in 0..3 {
        let expected = [31, 3, 13][phase];
        for count in (0..expected).chain([expected + 1]) {
            let mut script = vec![step(&[], 31), step(&[], 3), step(&CSW, 13)];
            script[phase].count = Ok(count);
            let (mut bot, trace) = bot(script, Duration::from_secs(1));
            assert!(bot.write(&CDB, &[1, 2, 3]).is_err());
            terminal(&mut bot, &trace, phase + 1);
        }
        for error in [true, false] {
            let mut script = vec![step(&[], 31), step(&[], 3), step(&CSW, 13)];
            if error {
                script[phase].count = Err("injected".into());
            } else {
                script[phase].advance = Duration::from_secs(1);
            }
            let (mut bot, trace) = bot(script, Duration::from_secs(1));
            assert!(bot.write(&CDB, &[1, 2, 3]).is_err());
            terminal(&mut bot, &trace, phase + 1);
        }
    }
}

#[test]
fn zero_data_phase_has_no_endpoint_operation_and_zero_timeout_is_rejected() {
    let (mut bot, trace) = bot(vec![step(&[], 31), step(&CSW, 13)], Duration::from_secs(1));
    bot.write(&CDB, &[]).unwrap();
    assert_eq!(trace.borrow().len(), 2);
    assert!(matches!(trace.borrow()[1], Call::Read(13, _)));
    let fake = Fake {
        script: VecDeque::new(),
        clock: Rc::new(Cell::new(Duration::ZERO)),
        trace,
    };
    assert!(Bot::new(fake, Duration::ZERO).is_err());
}

#[test]
fn oversized_transfer_and_deadline_overflow_stop_before_endpoints() {
    if usize::BITS > 32 {
        let (mut bot, trace) = bot(vec![], Duration::from_secs(1));
        assert!(bot.read(&CDB, (u32::MAX as usize) + 1).is_err());
        terminal(&mut bot, &trace, 0);
    }
    let trace = Rc::new(RefCell::new(vec![]));
    let fake = Fake {
        script: VecDeque::new(),
        clock: Rc::new(Cell::new(Duration::MAX)),
        trace: trace.clone(),
    };
    let mut bot = Bot::new(fake, Duration::from_secs(1)).unwrap();
    assert_eq!(
        bot.read(&CDB, 16),
        Err("BOT transaction deadline overflow".into())
    );
    terminal(&mut bot, &trace, 0);
}
