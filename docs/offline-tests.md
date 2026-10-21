# Offline host tests

Run the canonical file-only entrypoint with an existing Python 3 and Rust
2024-capable compiler:

```sh
python3 tools/run_offline_tests.py
```

The runner uses `rustc --test`, the actual shared format/board/sector/protocol
source, and a compile-time denying adapter. It does not resolve Cargo packages,
install a hardware backend, enumerate devices, open a device or execute/emulate
firmware. Four deliberate forbidden-call tests pass before the shared
and synthetic fixture tests run. Inventory panics explicitly; open and command
access return explicit errors. The operation-denial test also covers the MIDI
and USB categories, which have no native implementation in this harness.

Both `cfg(test)` and the `offline-tests` feature exclude every project native
SCSI/USB/macOS/MIDI adapter, including the macOS helper entrypoint. This happens at
compilation, before the test harness can invoke anything. The std-only harness
also forbids unsafe Rust. Injected `Scsi` and `Flash` fakes remain available.
This is a project adapter boundary, not an OS sandbox for arbitrary code added
to a test. New device paths must preserve that boundary and bring their own
independent denial test.

Compiler/target override environment variables and unknown runner arguments
are rejected. Each compiler/test subprocess has a 20-minute limit; CI jobs have
a 25-minute limit. Test output goes under `target/offline-tests/`. Normal application
build defaults, dependency versions, bundled artifacts and platform behavior
are unchanged.

`offline-tests` is a test-only feature. Application builds that enable it,
including `cargo build --all-features`, fail with an explicit compile error
instead of producing a flasher with disabled adapters. Use the normal default
or selected production features for application builds.

## Full existing suite

With all locked dependencies already cached:

```sh
python3 tools/run_offline_tests.py --cargo
```

This proves the guard first, then runs `cargo test --offline --locked
--no-default-features --features offline-tests`. A missing cache is a failure,
not a successful skip or permission to download/install anything. Cargo can
still compile existing dependencies that are unused by the excluded adapters
(for example the mandatory macOS libusb dependency); the project native
adapters remain absent from the test executable. The isolated Linux CI job
prefetches locked crate sources, then uses this offline command; default Linux
dependencies exclude optional libusb. CI adds no OS driver/backend bootstrap
and never runs the application.

After the full suite, the runner also requires a production `cargo check` with
the test-only feature to fail for that exact compile-error reason. A missing
dependency or unrelated compiler error cannot satisfy this negative check.

Twelve existing tests require a private firmware corpus and are built-in
`#[ignore]` tests. Their output is honestly counted as ignored. To request them
explicitly from an already cached source tree:

```sh
FLASHER_TEST_DATA=/private/fixture-directory cargo test --offline --locked --no-default-features --features offline-tests -- --ignored
```

An explicitly requested corpus test fails when an input is missing. Do not
publish the corpus. These tests currently do not validate a hash manifest;
a successful invocation must not be presented as hash-bound corpus evidence.

## Coverage and remaining work

The synthetic literals use independent Python `binascii.crc_hqx` and a separate
shift/XOR ENC reference. They cover standard CRC vectors, an APP file at logical
`0x240` relative to physical `0x4000`, a BANKCB payload at nondefault `0x20`,
truncated lengths, damaged table/payload CRCs and missing APP bytes. They contain
no real firmware. They check existing representation/offset semantics, not a
new directory-derived package extraction algorithm.

This is the first isolation/fixture slice. It does not complete strict transport
framing, overlap/overflow validation, partial-transfer fault traces, matching-CRC
unstable reads, mixed old/new sectors, stale-tail changes, recovery behavior, or
injected storage/clock coverage. Existing fakes do not establish physical reads,
bootability, persistence durability or writer qualification. New paths need
injected transport/storage/clock seams and independent deterministic fault tests
before those claims can be made.

## Saved-file diagnostics slice (#11)

The std-only harness now also includes saved analysis, strict file-only option
parsing, the protected-copy receipt contract and bounded regular-file reads.
`tests/fixtures/saved/generate.py` independently generates checked-in raw/package
and malformed-table oracles with Python `binascii.crc_hqx` and a separate ENC
reference. No private firmware is included. App CRC damage, unknown layout,
truncation, overlap/overflow, auxiliary package payload CRCs, duplicate package
names, receipt ambiguity, incompatible declarations and incident-shaped mixed
sectors are exercised. Support output is checked against secret canaries and
must retain unknown USB/bootability/fresh-acquisition limits. The incident oracle
has a multi-sector valid APP before mutation; mixed old/new APP sectors retain
an intact boot container plus inert staged-loader/update markers, and diagnose
app corruption without claiming a demonstrated resume path. The Unix filesystem
substitution test uses the system `mkfifo` utility; it installs nothing.

The full guarded Cargo pass also exercises the actual offline CLI dispatch,
reference mismatch exit codes, protected writer/reader receipt interoperability,
and the pure planner shared with the existing session path. Native adapters stay
compiled out. This adds bounded parser/incident fixtures toward #2; it does not
close the remaining transport, storage/clock or physical failure-model work.

## Recreated upstream MIDI adapter

The GUI/CLI download-mode soft key in `src/midi.rs` remains upstream behavior.
Its native implementation is compiled out in test/offline builds. Felucca OTA
is handled by its Python updater, outside this repository. The canonical runner includes the upstream pure port-name
and ALSA-card parser tests and proves that soft-key inventory and sending are
refused before the shared suite. The upstream real-sequencer loopback test is
excluded from the offline suite; device access is never enabled by `--ignored`.
