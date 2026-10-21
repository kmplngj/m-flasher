> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Next safe step toward the owner's first stock USB-C backup

Date: **2026-10-15**, Europe/Berlin. FM1-033-D2 continuation on `init`, after
the separately authorized [V15 identity observation](26-native-midi-identity.md).
**This continuation used files, source research and isolated compiler/fake tests
only. No FM-1 inventory, connection, message, mode change or flash operation
was performed. A full device backup has not been created.**

**Later exact-machine audit:** [report 31](31-loader-callpath-audit.md) now
resolves the previously missing startup/read/return paths and records concrete
conditional status-write attempts in startup and ordinary `FC0A`. The default
lifecycle remains unqualified; use that report's updated work order before any
hardware continuation. The build evidence below is unchanged.

## Completed build gap

The unchanged `native-usb` code now has actual **Linux arm64** compilation
evidence in addition to the earlier macOS build. Its `native-usb` feature was
typechecked, never executed. The public factory still returns
`QualificationBlocked`; its private qualification permit has no variants.
No CLI dependency, transport allowlist, profile, firmware or activation gate
changed.

An immutable 11-file snapshot from implementation commit `da489f0` contained the
binding manifest/lock/policy, five Rust sources, runner/guard and shared source
audit. Its canonical input-map SHA-256 is
`[digest omitted from public archive]`.
Original and copied inputs were hash-compared after validation.

The compiler ran in the official `rust:1.99.0-bookworm` image, pinned to
`sha256:[digest omitted from public archive]`,
resolved as `linux/arm64`. Recorded tools were Rust 1.99.0
(`b940084d7`, LLVM 23.1.1), Cargo 1.99.0 (`5f94df478`) and Python 3.11.2.
The container had no network, privilege, device passthrough or Docker socket;
all capabilities were dropped and no-new-privileges was enabled. It used UID
501/GID 20, a read-only root, read-only input/registry mounts and disposable
tmpfs work. No running service or existing image was modified. The container
removed itself; the newly pulled image was removed after validation.

Results on that snapshot:

| Check | Result |
|---|---|
| Source/default-feature/closed-factory/dependency audit | Passed: five Rust sources, 34 exact registry pins |
| File-only mutation/guard tests | 12 passed |
| Default-feature-free debug injected tests | 26 passed; none ignored or filtered |
| Default-feature-free optimized injected tests | 26 passed; none ignored or filtered |
| Actual Linux `native-usb` API compilation | `cargo check --offline --locked --lib --features native-usb` passed |
| Source formatting | Host `cargo fmt --check` passed on the same source hashes |

The canonical guarded host suite on macOS separately passed **306 tests in
193.740 s**, without skips. It installed its no-hardware guard before importing
the tests; the simulated writer checks did not access a physical device.

The first container attempt stopped at the missing `rustfmt` component before
tests. No component was installed to resolve that: formatting was checked on
the host, then the remaining checks ran in a fresh isolated container.
This environment failure and the subsequent successful run are both retained.

The ignored local evidence directory is
`[local artifact location omitted]`, containing the input snapshot, input
map, both logs and a structured result. Successful log SHA-256:
`[digest omitted from public archive]`.
Result-receipt SHA-256:
`[digest omitted from public archive]`.
These records contain build evidence, not device bytes or USB authority.

No x86-64 Linux, Windows, hosted CI, selected-device open/claim, cancellation,
release or live backup acceptance is inferred from this arm64 container run.
Linux work is parallel product validation; it is not an extra prerequisite
that a macOS-only first backup must wait for.

## Owner-device readiness

| Requirement | Current evidence / remaining work |
|---|---|
| Reported firmware identity | One explicit macOS USB-MIDI query returned valid `FM-1_015`; advisory V15, not firmware integrity or board/chip authentication |
| Unchanged installed stock firmware | Required throughout this task; no rollback/upgrade, OTA, preset write or recovery-stub installation |
| Exact stock-to-ROM entry | Transporter's V15 observation is a source reference; direct CoreMIDI packet framing/effects and exact route remain to be qualified |
| Exact RAM-loader identity | Encoded/decoded hashes established; [source-effects follow-up](27-loader-source-effects-followup.md) refines provenance and remaining analysis |
| No persistent effects along the read route | Startup, SPI status/protection/quad paths, `FD05`, idle/watchdog and normal return still require transitive instruction/control-flow evidence |
| macOS selected-object access | Actual code compiles; freshness, exclusive open, ordinary claim, cancellation and release effects remain qualification gates |
| Capture and persistence | Two-read/two-copy offline storage exists; actual native provenance and held-session continuity through fsync/reopen/independent copy still need integration |
| Full capacity and original stock return | Must be established in the eventual authorized read session; model capacity is not a device measurement |

## Next work within FM1-033-D2

Use the exact loader hashes and the pinned vendor-distribution references in
report 27 as the next analysis inputs. Recover instruction boundaries and
control flow for entry/initialization and `FD05` separately, including every
indirect/ROM callee and possible persistent side effect. Matching SDK symbols,
diagnostics, recovered library IR and a vendor-hosted blob narrow the
investigation; they cannot replace a source-to-binary linkage or prove which
loader path executes. The IR supplies specific status-write flag/command
branches to compare against the decoded loader. Unknown instructions/callees
keep activation closed.

Then implement the native capture/storage provenance boundary using injected
transport and storage failures first. Keep one serialized session owner through
both full reads, persistence, synchronization, reopened verification and the
second independent copy. A deadline or cached descriptor cannot prove the
loader is still running. Do not introduce a keepalive, reset, retry or automatic
return command before auditing its exact effects.

Only after the concrete entry/loader/platform gates pass may the already
authorized hardware-backup task perform a selected-device read session. Full
acceptance means two fresh identical reads of independently established complete
flash capacity, chip/layout checks, two durable reopened/hash-verified copies
and a documented return to the original stock application. A firmware-only or
preset export does not satisfy that milestone. Flash writing remains disabled.
