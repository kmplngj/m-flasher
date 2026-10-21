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
    loader_sent: Vec<Vec<u8>>,
    loader_identity: Option<&'static [u8]>,
    final_identity: Option<&'static [u8]>,
    selections: Vec<Selection>,
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
    fn connect(&mut self, selection: &Selection) -> Result<()> {
        self.selections.push(selection.clone());
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
        if self.phase == 1 {
            self.loader_sent.push(wire.to_vec());
        }
        if wire == QUERY {
            self.queue.push_back(
                match self.phase {
                    0 => ID_RUNNING,
                    1 => self.loader_identity.unwrap_or(ID_LOADER),
                    _ if self.wrong_identity => ID_RUNNING,
                    _ => self.final_identity.unwrap_or(ID_FINAL),
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
fn crc_valid_incompatible_package_layout_and_loader_are_refused() {
    let layout = [WRONG_LAYOUT_PREFIX, &vec![255; 0x94000], LOADER].concat();
    assert!(
        Image::parse(&layout)
            .err()
            .unwrap()
            .contains("flash.bin must be exactly 0x93000")
    );
    let loader = [WRONG_LOADER_PREFIX, &vec![255; 0x93000], WRONG_LOADER].concat();
    assert!(
        Image::parse(&loader)
            .err()
            .unwrap()
            .contains("unsupported or damaged Felucca OTA loader")
    );
}
#[test]
fn configured_application_product_keeps_fixed_loader_policy_and_exact_final_check() {
    let package = [CONFIGURED_PREFIX, &vec![255; 0x93000], LOADER].concat();
    let image = Image::parse(&package).unwrap();
    assert_eq!(image.identity, "FM-1_DEV_2.0-PR7");
    assert_eq!(Identity::parse(ID_CONFIGURED).unwrap().0, image.identity);
    for phase in [0, 1] {
        let mut io = Fake {
            phase,
            final_identity: Some(ID_CONFIGURED),
            ..Fake::default()
        };
        let id = open(&mut io, &Selection::default()).unwrap();
        install(&mut io, &Selection::default(), &image, &id, &mut |_| {}).unwrap();
        assert_eq!(io.starts, 2 - phase);
        let mut io = Fake {
            phase,
            ..Fake::default()
        };
        let id = open(&mut io, &Selection::default()).unwrap();
        assert_eq!(
            install(&mut io, &Selection::default(), &image, &id, &mut |_| {}).unwrap_err(),
            "device returned FM-1_906, expected FM-1_DEV_2.0-PR7"
        );
    }
    for invalid in [
        "FM-1_",
        "FM-1_lower",
        "FM-1_A/B",
        "FM-1_ABCDEFGHIJKLMNOP",
        "FM-1_É",
    ] {
        assert!(!valid_identity(invalid), "accepted invalid product {invalid}");
    }
    assert!(valid_identity("FM-1_ABCDEFGHIJKLMNO"));
    assert!(!valid_identity("ota-FM-1_DEV"));
}
#[test]
fn wire_literals_and_malformed_frames() {
    assert_eq!(Identity::parse(ID_RUNNING).unwrap().0, "FM-1_905");
    assert_eq!(Identity::parse(ID_LOADER).unwrap().0, "ota-FM-1_900");
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
    let id = open(&mut io, &Selection::default()).unwrap();
    install(
        &mut io,
        &Selection::default(),
        &image(),
        &id,
        &mut |_| {},
    )
    .unwrap();
    assert_eq!(io.starts, 1);
    assert_eq!(io.phase, 2);
    assert!(!io.dropped_early);
}
#[test]
fn incompatible_running_loaders_send_no_start_or_package_data() {
    for wire in [ID_OFFICIAL_LOADER, ID_UNKNOWN_LOADER] {
        let mut io = Fake {
            phase: 1,
            ..Fake::default()
        };
        let id = Identity::parse(wire).unwrap();
        let error = install(
            &mut io,
            &Selection::default(),
            &image(),
            &id,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(error.contains(&format!("incompatible MIDI loader {}", id.0)));
        assert!(error.contains("requires ota-FM-1_900"));
        assert!(io.sent.is_empty());
        assert_eq!(io.starts, 0);
    }
}
#[test]
fn incompatible_discovered_loaders_send_no_phase_two_start_or_data() {
    for wire in [ID_OFFICIAL_LOADER, ID_UNKNOWN_LOADER] {
        let mut io = Fake {
            loader_identity: Some(wire),
            ..Fake::default()
        };
        let error = install(
            &mut io,
            &Selection::default(),
            &image(),
            &Identity("FM-1_905".into()),
            &mut |_| {},
        )
        .unwrap_err();
        assert!(error.contains("incompatible MIDI loader"));
        assert!(error.contains("stopped before loader START/data"));
        assert_eq!(io.phase, 1, "only staging completed");
        assert_eq!(io.starts, 1);
        assert_eq!(io.loader_sent, vec![QUERY.to_vec()]);
        assert!(!io.dropped_early);
    }
}
#[test]
fn fresh_and_resumed_updates_require_exact_returned_application_identity() {
    for phase in [0, 1] {
        let mut io = Fake {
            phase,
            wrong_identity: true,
            ..Fake::default()
        };
        let mut reports = Vec::new();
        let id = open(&mut io, &Selection::default()).unwrap();
        let error = install(
            &mut io,
            &Selection::default(),
            &image(),
            &id,
            &mut |message| reports.push(message.to_string()),
        )
        .unwrap_err();
        assert_eq!(error, "device returned FM-1_905, expected FM-1_906");
        assert_eq!(io.starts, 2 - phase);
        assert_eq!(io.phase, 2);
        assert!(
            !reports
                .iter()
                .any(|s| s.starts_with("done:") || s.starts_with("OTA exchange completed"))
        );
    }
}
#[test]
fn exact_endpoint_constraints_survive_both_successful_handoffs() {
    let selection = Selection {
        port: None,
        input: Some("selected input".into()),
        output: Some("selected output".into()),
    };
    let mut io = Fake::default();
    let id = open(&mut io, &selection).unwrap();
    install(&mut io, &selection, &image(), &id, &mut |_| {}).unwrap();
    assert_eq!(io.selections.len(), 3);
    for seen in io.selections {
        assert_eq!(seen.port, selection.port);
        assert_eq!(seen.input, selection.input);
        assert_eq!(seen.output, selection.output);
    }
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
    for id in ["SMK-37_015", "ota-SMK-37_900"] {
        let mut io = Fake::default();
        assert!(
            install(
                &mut io,
                &Selection::default(),
                &image(),
                &Identity(id.into()),
                &mut |_| {}
            )
            .is_err()
        );
        assert!(io.sent.is_empty());
    }
}
#[test]
fn wrong_model_after_staging_never_receives_phase_two_start_or_data() {
    let mut io = Fake {
        loader_identity: Some(ID_WRONG_MODEL_LOADER),
        ..Fake::default()
    };
    assert!(
        install(
            &mut io,
            &Selection::default(),
            &image(),
            &Identity("FM-1_905".into()),
            &mut |_| {},
        )
        .unwrap_err()
        .contains("update loader did not appear")
    );
    assert_eq!(io.starts, 1);
    assert_eq!(io.phase, 1);
    assert!(io.loader_sent.iter().all(|wire| wire.as_slice() == QUERY));
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
