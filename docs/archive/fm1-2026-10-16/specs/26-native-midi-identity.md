> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# FM1-034: read-only firmware identity in the Rust CLI

Date: **2026-10-15**. Implementation on `init`; development and tests use
local files, pure models and injected callbacks only. **During implementation,
no hardware inventory, client creation, connection, query, loader or flash
operation was executed.** A later separately authorized identity-only hardware
observation is recorded at the end; it is not part of development tests.

## Request and outcome

The owner reports that fm1-editor.com displays **V15** on their stock FM-1,
using macOS 27 on Apple Silicon. This is an owner-reported editor result; no
raw reply or firmware image was captured by our CLI. It replaces “version
unknown” as the owner's reported observation, without authenticating the image.

The owner requested the same identity operation in the CLI. It is independent
of the unresolved ROM/RAM-loader backup route. This increment adds an explicit,
opt-in macOS identity path to the **same `fm1-flasher` executable**, while the
default build remains hardware-free. It does not enable any backup, ROM entry,
loader execution, preset/settings change, programming or erase operation.
The request authorizes implementation; it does not turn development/tests into
a live hardware session or relax the owner's no-flash-write constraint.

## Acceptance and implemented boundary

- `devices midi --json` explicitly lists MIDI endpoint IDs and display names.
  It creates CoreMIDI clients and reads metadata only; it opens no endpoint
  connection and sends no identity request. It is never run at startup/help/status.
- `identify --input-id <id> --output-id <id> [--timeout-ms <100..5000>] [--json]`
  requires both exact current opaque endpoint IDs. No name substring, port index,
  first-device choice, broadcast, fallback or retry. Signed IDs are valid;
  missing metadata ID `0`, duplicate matches and excessive inventory refuse.
  The caller chooses the pair; endpoint IDs do not authenticate a physical FM-1.
- Input is connected before output. The only send statement transmits exactly
  `F0 00 32 45 00 00 00 40 7F F7` once. No arbitrary-message interface exists.
  A local receive fence precedes the send, so fast replies during send are kept;
  a failed or late send still refuses the collected reply.
- The receive state accepts fragmented replies and interleaved realtime bytes,
  rejects malformed, excessive, multiple or late replies, and observes the whole
  receive window to detect duplicates. There are no sleep-based tests.
- Strict decoding reuses the existing codec: 41 wire bytes, canonical 7-bit
  packing/padding, 34 decoded bytes, `00 59 11`, body length 27, complement
  checksum and zero-padded ASCII `FM-1_NNN`. OTA identities refuse. No checksum
  exception, version-range firmware-family guess or update compatibility follows.
- `identify --reply-hex <hex> [--json]` decodes a supplied reply offline. It
  cannot fall back to hardware or interpret its argument as a file path.
- Success labels the result as reported firmware identity, with full-backup,
  firmware-integrity, raw-verification and physical write claims false. JSON
  includes the accepted strict raw reply for independent offline review. Invalid
  raw replies are not persisted in this slice; errors use fixed sanitized codes.

The ten-byte vendor question and normal-mode reply are documented separately
from the mode-changing update command. A published stock-V15 macOS capture
supports the identity route, not this new native adapter's physical acceptance.
[Protocol](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/11-ota-protocol.md#L24-L36),
[macOS capture](https://github.com/ip2k/lunar-modulator/blob/bfa042446f36c8647efddc6d76b06753d2a5b612/notes/2026-09-06-bench.md),
[prior version research](25-stock-version-identification.md).

## Feature and development isolation

The `midi-identity` feature has empty defaults and an optional macOS-only
`midir = 0.11.1` dependency with default features disabled. Actual calls live
only in the feature/OS-gated executable module `macos_midi.rs`. Library dispatch
is pure even when compiled with the feature; only executable dispatch can
construct the backend, after complete argument validation. A normal build or
unsupported OS returns an actionable refusal for live commands.

Canonical tests force `--no-default-features`. The source audit checks the
manifest/OS/feature shape, registry pins, module gate, fixed query and the sole
send/two connection sites, while retaining other filesystem/network/FFI/device
restrictions. `--build-midi` audits and compiles the executable; it returns
without running it. The Ubuntu/macOS/Windows workflow continues to test the
default core and adds a macOS compile-only feature step. Hosted runs are pending.

The existing seven registry pins remain unchanged. `midi-dependency-policy.json`
adds 94 exact package/version/checksum pins; the complete lock contains 101
registry pairs, including other target-conditional packages. Only the observed
macOS graph is compiled for this feature. Nine additional macOS dependency
notices are retained in MIDI notices (historical reference `../native/MIDI_THIRD_PARTY_NOTICES.md`; omitted from public archive).
Source preparation installs no OS driver or Python hardware backend.

Context7 did not resolve midir; its published API and pinned upstream source
were used instead. Port IDs are opaque and may be signed. Upstream's CoreMIDI
ID accessor falls back to `0` on error, so that value is refused here.
[midir API](https://docs.rs/midir/0.11.1/midir/struct.MidiInput.html),
[CoreMIDI implementation](https://github.com/Boddlnagg/midir/blob/v0.11.1/src/backend/coremidi/mod.rs).

## Concrete limits

The protocol has no nonce. Post-fence arrival provides advisory correlation,
not authenticated transaction freshness. Close the editor/DAW/other clients
before a later explicit capture, and select the intended input/output pair.
The resulting string cannot grant entry, backup, writer or firmware authority.

CoreMIDI client creation, enumeration, opening, sending and teardown are
synchronous OS calls. Local checks before/after them and a bounded receive
wait cannot preempt those calls or guarantee a total wall-clock bound.
midir also accumulates segmented SysEx in its own Vec before our callback;
our 41-byte state and callback/activity limits do not bound that upstream
allocation. This is a reviewed regression boundary, not a malicious-device or
malicious-source sandbox. Review noted additional lexical hardening opportunities
for commented cfg/shadowed constant spellings; current code has neither pattern.

## Reproducible commands and first later use

Source preparation is separate from offline build/test execution:

```sh
rtk proxy python3.12 native/run_tests.py --audit-only
rtk proxy cargo fetch --manifest-path native/Cargo.toml --locked --target aarch64-apple-darwin
rtk proxy python3.12 host/run_tests.py
rtk proxy python3.12 native/run_tests.py
rtk proxy python3.12 native/run_tests.py --release --test midi_identity_conformance
rtk proxy python3.12 native/run_tests.py --release --test lib --filter midi_identity
rtk proxy python3.12 native/run_tests.py --build-midi --release
rtk proxy mkdir -p [local artifact location omitted]
rtk proxy cp [local artifact location omitted] [local artifact location omitted]
```

The feature build is copied locally to `[local artifact location omitted]`
so subsequent default builds cannot silently replace the intended executable.
The following are **manual live commands for a separate hardware session**;
they were not run during development:

```sh
rtk proxy [local artifact location omitted] devices midi --json
rtk proxy [local artifact location omitted] identify --input-id 'INPUT_ID' --output-id 'OUTPUT_ID' --json
```

Replace both placeholders with the intended exact IDs from the inventory.
Silence, ambiguity, a disappearing endpoint or a malformed reply stops the
operation; there is no automatic retry, mode switch or update fallback.

## Validation record

- Full canonical native debug: **207 passed** (165 unit, 42 integration),
  no ignored/filtered tests; unit suite **519.06 s**. All 14 new identity/CLI
  cases are included. The seven protocol/state tests and seven CLI conformance
  tests also passed optimized, with 158 deliberately filtered old unit cases.
- Canonical Python 3.12.15 no-hardware runner: **306 passed in 215.729 s**,
  no skips; includes four new pure audit regressions. The guard is installed
  before test discovery/import as before.
- The separate native-usb default injected suite: **26 passed in 0.07 s** after
  the shared lexical-audit update; its hardware factory remains closed.
- Source/API/dependency audit passed for **40 Rust files**; formatting and diff
  checks passed. The exact 57-file native input-map SHA-256 is
  `[digest omitted from public archive]`.
- Actual macOS identity-feature release build passed. Existing unchanged core
  code emits three warnings (deprecated atomic method and two unused methods);
  no new adapter warning/error occurred. The feature executable was not run.
- The default executable's status and offline decoding of an independently
  specified V15 wire response passed; malformed supplied bytes refused. Its
  dynamic-library inspection shows no CoreMIDI dependency. These commands
  used no device access.
- Local arm64 feature artifact: **355,120 bytes**, SHA-256
  `[digest omitted from public archive]`.
  Static file/link inspection confirms Mach-O arm64 and system CoreMIDI,
  CoreAudio, Foundation, CoreFoundation, Objective-C and system libraries.
  It embeds no Python and requires no Python interpreter to run. This is a
  local build, not a release/install or physical-device qualification.

Logs, input map and copied executable are ignored under `native/target`.
GPT-6.1 Sol high implemented/reviewed the protocol/backend; medium implemented
CLI/conformance and notices. Review corrected signed endpoint ID handling and
the unavailable-ID sentinel; root integrated audit/dependencies/CI/docs and ran
validation. At implementation close, no live query, hosted CI result, push or
owner-backup acceptance was claimed. Actual MIDI CLI acceptance remained a
separate hardware check; the browser's V15 report could not substitute for it.

## First separately authorized CLI hardware observation

On **2026-10-15**, after implementation commit
`750385b9494b6a93400a03d5a02a0e36fa3587c5`, the owner explicitly requested the
USB-MIDI read test through the CLI. The copied artifact's SHA-256 was rechecked
against the build record above before hardware access. Host platform is the
owner's reported macOS 27 / Apple Silicon; no additional OS/USB probe was used.

1. One `devices midi --json` invocation exited 0 and reported exactly one input
   and one output, both named `FM-1`. It reported no identity query sent.
2. One `identify --input-id … --output-id … --json` invocation used those exact
   current endpoint IDs, the default 1,500 ms window and no retry. It exited 0
   with `identity_query_sent: true`, `identity: FM-1_015`, `reported_version: 15`
   and `source: selected-midi-pair`.

The accepted reply was **41 wire bytes / 34 decoded bytes**. An independent
Python stdlib byte check confirmed canonical 7-bit padding,
`00 59 11`, body length 27, ASCII `FM-1_015` plus 19 zero bytes and the complement
checksum. No additional hardware request was used for this verification.

Exact endpoint IDs and the captured JSON/raw reply remain in a private local
mode-0600 receipt under ignored `[local artifact location omitted]`. They are
historical observations, not a saved selector or authority for another session.
Public reporting omits endpoint/location/serial identifiers.

This demonstrates the explicit identity-only CLI route on this one owner unit
and reported host platform. It does not qualify another firmware/OS/device,
prove firmware image integrity, or establish a raw flash backup. No ROM/OTA
entry, loader upload/execution, preset/settings message, flash erase/program,
reset, reconnect retry or post-test device probe was performed. Result flags
for firmware integrity, raw verification, full backup and real writes stayed
false. Normal return after ROM/loader execution was not applicable or tested.
