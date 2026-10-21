// SPDX-License-Identifier: MIT
use super::*;
use std::collections::VecDeque;
#[path = "../fixtures/midi/fixture.rs"]
mod fixture;
use fixture::*;
fn raw() -> Vec<u8> {
    [PREFIX, &vec![255; 0x93000], LOADER].concat()
}
fn image() -> Image {
    Image::parse(&raw()).unwrap()
}

#[derive(Default)]
struct Fake {
    tick: Duration,
    phase: usize,
    queue: VecDeque<Vec<u8>>,
    sent: Vec<Vec<u8>>,
    starts: usize,
    inject: Option<Vec<u8>>,
    silent: bool,
    wrong_identity: bool,
    fail_send: bool,
    disconnected: bool,
    completion_at: Option<Duration>,
    dropped_early: bool,
}
impl Io for Fake {
    fn now(&self) -> Duration {
        self.tick
    }
    fn pause(&mut self, d: Duration) {
        self.tick += d;
    }
    fn connect(&mut self, _: &Selection) -> Result<()> {
        Ok(())
    }
    fn disconnect(&mut self) {
        if let Some(when) = self.completion_at.take() {
            self.dropped_early |= self.tick.saturating_sub(when) < seconds(3);
        }
        self.queue.clear();
    }
    fn send(&mut self, wire: &[u8]) -> Result<()> {
        if self.fail_send {
            return Err("injected send failure".into());
        }
        self.sent.push(wire.to_vec());
        if wire == QUERY {
            self.queue.push_back(
                match self.phase {
                    0 => ID_RUNNING,
                    1 => ID_LOADER,
                    _ if self.wrong_identity => ID_RUNNING,
                    _ => ID_FINAL,
                }
                .to_vec(),
            );
        }
        if wire == START {
            self.starts += 1;
            if !self.silent {
                if let Some(bad) = self.inject.take() {
                    self.queue.push_back(bad);
                } else {
                    // Device-pull accepts duplicate and unordered ranges unchanged.
                    self.queue.extend([
                        vec![0xf0, 0x7f, 0xf7],
                        READ_LATE.to_vec(),
                        READ.to_vec(),
                        READ.to_vec(),
                        if self.phase == 0 {
                            DONE_CHECK.to_vec()
                        } else {
                            DONE_WRITE.to_vec()
                        },
                    ]);
                }
            }
        }
        if frame(wire, 0x30, 23).is_some_and(|d| le32(&d, 7) == CHECK_DONE) {
            self.phase = 1;
            self.completion_at = Some(self.tick);
        }
        if frame(wire, 0x30, 23).is_some_and(|d| le32(&d, 7) == WRITE_DONE) {
            self.phase = 2;
            self.completion_at = Some(self.tick);
        }
        Ok(())
    }
    fn receive(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>> {
        self.tick += Duration::from_millis(1);
        if self.disconnected {
            return Err("injected disconnect".into());
        }
        if let Some(data) = self.queue.pop_front() {
            Ok(Some(data))
        } else {
            self.tick += timeout;
            Ok(None)
        }
    }
}
#[test]
fn independent_package_and_crc_damage_oracles() {
    assert_eq!(image().identity, "FM-1_906");
    let valid = raw();
    for offset in [0, 50, PREFIX.len() + 12, valid.len() - 1] {
        let mut broken = valid.clone();
        broken[offset] ^= 1;
        assert!(
            Image::parse(&broken).is_err(),
            "accepted corruption at {offset}"
        );
    }
    assert!(Image::parse(&valid[..100]).is_err());
    assert!(Image::parse(&vec![0; MAX_PACKAGE + 1]).is_err());
}
#[test]
fn wire_literals_and_malformed_frames() {
    assert_eq!(Identity::parse(ID_RUNNING).unwrap().0, "FM-1_905");
    assert!(Identity::parse(ID_LOADER).unwrap().loader());
    let image = image();
    assert_eq!(answer(7, 3, &image.bytes[3..8]), REPLY);
    let mut bad = READ.to_vec();
    bad[4] ^= 1;
    assert!(frame(&bad, 0x30, 15).is_none());
    let mut bad = ID_FINAL.to_vec();
    bad[1] = 0x80;
    assert!(Identity::parse(&bad).is_none());
    let mut bad = READ.to_vec();
    bad.insert(bad.len() - 1, 0);
    assert!(frame(&bad, 0x30, 15).is_none());
    for n in 0..READ.len() {
        assert!(frame(&READ[..n], 0x30, 15).is_none());
    }
}
#[test]
fn explicitly_pairs_endpoints_and_refuses_ambiguity() {
    let names = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert!(
        pair(
            &names(&["Other keyboard"]),
            &names(&["Other keyboard"]),
            &Selection::default()
        )
        .is_err()
    );
    assert!(
        pair(
            &names(&["FM-1", "Felucca"]),
            &names(&["FM-1", "Felucca"]),
            &Selection::default()
        )
        .is_err()
    );
    assert!(
        pair(
            &names(&["FM-1"]),
            &names(&["Different"]),
            &Selection::default()
        )
        .is_err()
    );
    assert_eq!(
        pair(
            &names(&["FM-1 1"]),
            &names(&["FM-1 2"]),
            &Selection::default()
        )
        .unwrap()
        .output,
        "FM-1 2"
    );
    assert!(pair(&names(&["FM-1"]), &names(&["FM-2"]), &Selection::default()).is_err());
    assert_eq!(
        pair(
            &names(&["FM-1"]),
            &names(&["FM-1", "FM-2"]),
            &Selection::default()
        )
        .unwrap()
        .output,
        "FM-1"
    );
    let shared = Selection {
        port: Some("FM-1".into()),
        ..Selection::default()
    };
    assert_eq!(
        pair(&names(&["FM-1 Input"]), &names(&["FM-1 Output"]), &shared)
            .unwrap()
            .output,
        "FM-1 Output"
    );
    assert!(
        pair(
            &names(&["FM-1 Input"]),
            &names(&["FM-1 Output", "FM-1 Other"]),
            &shared
        )
        .is_err()
    );
    for other in ["FM-10", "FM-123", "SomeFM-1", "FeluccaLike"] {
        assert!(pair(&names(&[other]), &names(&[other]), &Selection::default()).is_err());
    }
    assert_eq!(
        pair(
            &names(&["FM-1", "FM-10"]),
            &names(&["FM-1", "FM-10"]),
            &Selection::default()
        )
        .unwrap()
        .input,
        "FM-1"
    );
    let explicit = Selection {
        input: Some("Selected in".into()),
        output: Some("Selected out".into()),
        port: None,
    };
    assert!(
        pair(
            &names(&["Selected in"]),
            &names(&["Selected out"]),
            &explicit
        )
        .is_ok()
    );
}
#[test]
fn complete_pull_transcript_preserves_duplicates_and_checks_reboot() {
    let mut io = Fake::default();
    let image = image();
    let id = open(&mut io, &Selection::default()).unwrap();
    install(&mut io, &Selection::default(), &image, &id, &mut |_| {}).unwrap();
    assert_eq!(io.starts, 2);
    assert_eq!(io.phase, 2);
    assert!(
        !io.dropped_early,
        "completion reply dropped before the settling delay"
    );
    assert_eq!(io.sent.iter().filter(|p| p.as_slice() == REPLY).count(), 4);
}
#[test]
fn existing_loader_resumes_without_restarting_running_stage() {
    let mut io = Fake {
        phase: 1,
        ..Fake::default()
    };
    install(
        &mut io,
        &Selection::default(),
        &image(),
        &Identity("ota-FM-1_015".into()),
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(io.starts, 1);
    assert!(!io.dropped_early);
}
#[test]
fn bad_ranges_phase_send_disconnect_and_idle_never_report_success() {
    for request in [BAD_RANGE, BIG_READ, ZERO_READ, DONE_WRITE] {
        let mut io = Fake {
            inject: Some(request.to_vec()),
            ..Fake::default()
        };
        assert!(
            install(
                &mut io,
                &Selection::default(),
                &image(),
                &Identity("FM-1_905".into()),
                &mut |_| {}
            )
            .is_err()
        );
        assert_eq!(io.starts, 1);
    }
    for mut io in [
        Fake {
            silent: true,
            ..Fake::default()
        },
        Fake {
            fail_send: true,
            ..Fake::default()
        },
        Fake {
            disconnected: true,
            ..Fake::default()
        },
        Fake {
            wrong_identity: true,
            ..Fake::default()
        },
    ] {
        assert!(
            install(
                &mut io,
                &Selection::default(),
                &image(),
                &Identity("FM-1_905".into()),
                &mut |_| {}
            )
            .is_err()
        );
        assert!(io.tick < seconds(90));
    }
}
#[test]
fn wrong_model_never_sends_upgrade() {
    let mut io = Fake::default();
    assert!(
        install(
            &mut io,
            &Selection::default(),
            &image(),
            &Identity("SMK-37_015".into()),
            &mut |_| {}
        )
        .is_err()
    );
    assert!(io.sent.is_empty());
}

#[test]
fn same_version_exchange_does_not_claim_confirmed_replacement() {
    let mut io = Fake::default();
    let mut reports = Vec::new();
    install(
        &mut io,
        &Selection::default(),
        &image(),
        &Identity("FM-1_906".into()),
        &mut |message| reports.push(message.to_string()),
    )
    .unwrap();
    assert!(
        reports
            .last()
            .unwrap()
            .contains("Identity cannot confirm replacement")
    );
    assert!(!reports.last().unwrap().starts_with("done:"));
    assert_eq!(io.starts, 2);
}
