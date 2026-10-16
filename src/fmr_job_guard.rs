//! A terminal job guard around injected endpoint operations, including bootstrap.
//! The caller supplies cancellation/deadline policy; no hardware API is linked.

use crate::fmr_bot::Bulk;
use std::time::Duration;

pub struct Guarded<B, F> {
    bulk: B,
    check_job: F,
    terminal: bool,
}

impl<B, F: FnMut() -> Result<(), String>> Guarded<B, F> {
    /// Construction performs no I/O or callback. The job owner can call `check`
    /// before native open, and each endpoint operation checks again itself.
    pub fn new(bulk: B, check_job: F) -> Self {
        Self {
            bulk,
            check_job,
            terminal: false,
        }
    }

    pub fn check(&mut self) -> Result<(), String> {
        if self.terminal {
            return Err("FMR1 guarded endpoint session is terminal".into());
        }
        // Also poison on callback panic/unwind; no later call can resume it.
        self.terminal = true;
        let result = (self.check_job)();
        if result.is_ok() {
            self.terminal = false;
        }
        result
    }

    fn finish<R>(&mut self, endpoint: Result<R, String>) -> Result<R, String> {
        // Run the post-operation check even when the endpoint itself failed.
        // A late/cancelled result is never accepted as a successful transfer.
        (self.check_job)()?;
        let value = endpoint?;
        self.terminal = false;
        Ok(value)
    }
}

impl<B: Bulk, F: FnMut() -> Result<(), String>> Bulk for Guarded<B, F> {
    fn write(&mut self, bytes: &[u8], timeout: Duration) -> Result<usize, String> {
        self.check()?;
        self.terminal = true;
        let result = self.bulk.write(bytes, timeout);
        self.finish(result)
    }

    fn read(&mut self, bytes: &mut [u8], timeout: Duration) -> Result<usize, String> {
        self.check()?;
        self.terminal = true;
        let result = self.bulk.read(bytes, timeout);
        self.finish(result)
    }

    fn elapsed(&self) -> Duration {
        self.bulk.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    struct State {
        cancelled: bool,
        elapsed: Duration,
        checks: usize,
        operations: Vec<(bool, usize, Duration)>,
        cancel_after: Option<usize>,
        expire_after: Option<usize>,
        fail_after: Option<usize>,
        panic_after: Option<usize>,
    }
    type Shared = Rc<RefCell<State>>;
    struct Fake(Shared);
    const LIMIT: Duration = Duration::from_secs(600);
    const TIMEOUT: Duration = Duration::from_millis(371);
    impl Fake {
        fn operation(
            &self,
            write: bool,
            length: usize,
            timeout: Duration,
        ) -> Result<usize, String> {
            let mut state = self.0.borrow_mut();
            state.operations.push((write, length, timeout));
            let n = state.operations.len();
            if state.cancel_after == Some(n) {
                state.cancelled = true;
            }
            if state.expire_after == Some(n) {
                state.elapsed = LIMIT;
            }
            assert_ne!(state.panic_after, Some(n), "injected endpoint panic");
            if state.fail_after == Some(n) {
                Err("injected endpoint failure".into())
            } else {
                Ok(length)
            }
        }
    }
    impl Bulk for Fake {
        fn write(&mut self, bytes: &[u8], timeout: Duration) -> Result<usize, String> {
            self.operation(true, bytes.len(), timeout)
        }
        fn read(&mut self, bytes: &mut [u8], timeout: Duration) -> Result<usize, String> {
            bytes.fill(0xa5);
            self.operation(false, bytes.len(), timeout)
        }
        fn elapsed(&self) -> Duration {
            self.0.borrow().elapsed
        }
    }
    fn fixture() -> (Guarded<Fake, impl FnMut() -> Result<(), String>>, Shared) {
        let state = Rc::new(RefCell::new(State::default()));
        let job = state.clone();
        let check = move || {
            let mut state = job.borrow_mut();
            state.checks += 1;
            if state.cancelled {
                return Err("injected job cancellation".into());
            }
            if state.elapsed >= LIMIT {
                return Err("injected job deadline".into());
            }
            Ok(())
        };
        (Guarded::new(Fake(state.clone()), check), state)
    }
    fn phase<B: Bulk>(bulk: &mut B, phase: usize) -> Result<usize, String> {
        if phase == 0 {
            bulk.write(&[7; 31], TIMEOUT)
        } else {
            bulk.read(&mut [0; 16], TIMEOUT)
        }
    }

    #[test]
    fn inert_construction_and_exact_endpoint_arguments_with_two_checks_each() {
        let (mut guard, state) = fixture();
        assert_eq!(state.borrow().checks, 0);
        assert!(state.borrow().operations.is_empty());
        assert_eq!(guard.write(&[0; 31], TIMEOUT), Ok(31));
        let mut bytes = [0; 16];
        assert_eq!(guard.read(&mut bytes, TIMEOUT), Ok(16));
        assert_eq!(bytes, [0xa5; 16]);
        assert_eq!(
            state.borrow().operations,
            [(true, 31, TIMEOUT), (false, 16, TIMEOUT)]
        );
        assert_eq!(state.borrow().checks, 4);
        state.borrow_mut().elapsed = Duration::from_secs(17);
        assert_eq!(guard.elapsed(), Duration::from_secs(17));
        assert_eq!(state.borrow().checks, 4);
    }

    #[test]
    fn cancellation_or_deadline_before_every_phase_prevents_that_operation() {
        for before_phase in 0..3 {
            for cancelled in [false, true] {
                let (mut guard, state) = fixture();
                for prior in 0..before_phase {
                    phase(&mut guard, prior).unwrap();
                }
                if cancelled {
                    state.borrow_mut().cancelled = true;
                } else {
                    state.borrow_mut().elapsed = LIMIT;
                }
                assert!(phase(&mut guard, before_phase).is_err());
                assert_eq!(state.borrow().operations.len(), before_phase);
                state.borrow_mut().cancelled = false;
                state.borrow_mut().elapsed = Duration::ZERO;
                assert!(
                    guard
                        .write(&[0; 31], TIMEOUT)
                        .unwrap_err()
                        .contains("terminal")
                );
                assert!(
                    guard
                        .read(&mut [0; 16], TIMEOUT)
                        .unwrap_err()
                        .contains("terminal")
                );
                assert_eq!(state.borrow().operations.len(), before_phase);
            }
        }
    }

    #[test]
    fn cancellation_or_deadline_after_each_phase_rejects_its_success_and_stays_terminal() {
        for after_phase in 1..=3 {
            for cancelled in [false, true] {
                let (mut guard, state) = fixture();
                if cancelled {
                    state.borrow_mut().cancel_after = Some(after_phase);
                } else {
                    state.borrow_mut().expire_after = Some(after_phase);
                }
                for prior in 0..after_phase - 1 {
                    phase(&mut guard, prior).unwrap();
                }
                let error = phase(&mut guard, after_phase - 1).unwrap_err();
                assert!(error.contains(if cancelled {
                    "cancellation"
                } else {
                    "deadline"
                }));
                assert_eq!(state.borrow().operations.len(), after_phase);
                assert!(guard.check().unwrap_err().contains("terminal"));
                assert!(phase(&mut guard, 0).is_err());
                assert_eq!(state.borrow().operations.len(), after_phase);
            }
        }
    }

    #[test]
    fn endpoint_error_is_preserved_post_checked_and_terminal() {
        for operation in 0..2 {
            let (mut guard, state) = fixture();
            state.borrow_mut().fail_after = Some(1);
            assert_eq!(
                phase(&mut guard, operation),
                Err("injected endpoint failure".into())
            );
            assert_eq!(state.borrow().checks, 2);
            assert!(
                phase(&mut guard, operation)
                    .unwrap_err()
                    .contains("terminal")
            );
            assert_eq!(state.borrow().operations.len(), 1);
        }
    }

    #[test]
    fn failed_endpoint_also_runs_post_cancellation_check() {
        let (mut guard, state) = fixture();
        state.borrow_mut().fail_after = Some(1);
        state.borrow_mut().cancel_after = Some(1);
        assert_eq!(
            guard.write(&[0; 31], TIMEOUT),
            Err("injected job cancellation".into())
        );
        assert_eq!(state.borrow().checks, 2);
        assert!(guard.read(&mut [0; 16], TIMEOUT).is_err());
        assert_eq!(state.borrow().operations.len(), 1);
    }

    #[test]
    fn endpoint_and_check_panics_poison_before_unwind() {
        let (mut guard, state) = fixture();
        state.borrow_mut().panic_after = Some(1);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            guard.write(&[0; 31], TIMEOUT)
        }));
        assert!(result.is_err());
        assert!(
            guard
                .read(&mut [0; 16], TIMEOUT)
                .unwrap_err()
                .contains("terminal")
        );
        assert_eq!(state.borrow().operations.len(), 1);
        let checks = Rc::new(RefCell::new(0));
        let observed = checks.clone();
        let mut guard = Guarded::new(Fake(state.clone()), move || {
            *observed.borrow_mut() += 1;
            panic!("injected check panic")
        });
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| guard.check())).is_err());
        assert!(guard.check().unwrap_err().contains("terminal"));
        assert_eq!(*checks.borrow(), 1);
        assert_eq!(state.borrow().operations.len(), 1);
    }
}
