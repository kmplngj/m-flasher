# Native MIDI OTA (implementation slice of #15)

`m-flasher midi firmware PACKAGE.fwsc` is an explicitly selected FM-1 Felucca
OTA route. It does not enter ROM USB or silently switch to it. The running
firmware and the package's loader pull immutable requested ranges over SysEx.
No Python, mido, python-rtmidi, libusb, root helper or ROM download mode is needed
for this route. The optional native adapter uses CoreMIDI, WinMM or ALSA.

## Install and select

Successful **native MIDI CLI** Actions runs provide `m-flasher-midi-macos`
(universal Apple silicon/Intel), `m-flasher-midi-linux-x86_64` and
`m-flasher-midi-windows-x86_64`. These development artifacts are not releases.
Retain `source.json` with the executable; it records the source commit and SHA-256.
On macOS/Linux restore the downloaded executable's mode with `chmod +x m-flasher`.
Alternatively, with Rust installed, build the small CLI:

```sh
cargo build --release --locked -p m-flasher-midi
# Linux build prerequisite: libasound2-dev (or the distribution's ALSA headers).
target/release/m-flasher-midi --capabilities  # offline, no MIDI initialization
```

Cargo builds/installs the small CLI as `m-flasher-midi`, keeping it distinct from
the root GUI/ROM binary. The distributed artifact calls it `m-flasher`; it has
only MIDI operations. Prefix the commands below with `target/release/m-flasher-midi`
when running a source build.
The full GUI/ROM executable can include the same route with `--features midi`.
Default GUI/ROM builds remain unchanged; capabilities report `midi_ota: false`
when that adapter is absent. No application is run by the offline tests.

```sh
m-flasher midi info --port FM-1
m-flasher midi firmware build/felucca.fwsc --port FM-1
# If input/output names cannot be paired uniquely:
m-flasher midi firmware build/felucca.fwsc --in-port 'FM-1 input' --out-port 'FM-1 output'
```

Only one connected FM-1 may match. Other MIDI gear is never selected by default.
The exact port constraints remain in force during loader/reboot re-enumeration;
use a shared substring such as `FM-1` if names change. Ambiguous pairs are refused.
`--yes` skips the install confirmation; it does not skip file/device checks.

### macOS endpoint refresh

The native CLI services the CoreFoundation event loop on its MIDI client thread
before enumerating ports. CoreMIDI's process-local source/destination lists can
otherwise retain the old firmware endpoints through the loader handoff, even
when USB has already enumerated `Felucca Update`. Sleeping and creating another
MIDI client do not refresh those lists. The same refresh covers the final reboot.

An opt-in software hotplug diagnostic exercises the production refresh helper:

```sh
mkdir -p target/midi-hotplug
rustc --edition=2024 tools/macos_midi_hotplug_probe.rs -o target/midi-hotplug/probe
target/midi-hotplug/probe --without-fix  # expected failure: new source stays invisible
target/midi-hotplug/probe                # expected success: refresh reveals the new source
```

This diagnostic is excluded from the offline test runners. Run it only in a
separately authorized macOS diagnostic task: it enumerates CoreMIDI sources and
creates/removes a temporary virtual source. It opens no physical endpoint, sends
no MIDI and performs no update. A passing result establishes cache refresh, not
a completed FM-1 installation. Firmware/loader handoff remains a separate device
check with the exact CLI and firmware artifacts.

## Loader/package compatibility (#119)

The native CLI accepts one source-defined package/loader pair. Application
products follow the build's explicit `FM-1_[A-Z0-9._-]{1,15}` contract (20 bytes
maximum); the full product must match after reboot. A configured application
product does not change the loader's fixed identity.

| Selected package | Observed state | Native behavior |
| --- | --- | --- |
| Valid `FELUCCA-LOADER-1`, `flash.bin` exactly `0x93000` bytes | Running FM-1 application | Stage the package loader; query the new endpoint; require exactly `ota-FM-1_900` before loader START/data |
| Same custom package | Already running `ota-FM-1_900` | Resume directly; require the package's full non-loader application identity after reboot |
| Same custom package | `ota-FM-1_015`, any other loader ID, or wrong model | Refuse before loader START/data; a failed post-stage check does not undo staging |
| Official V15 or another loader/layout | Any state | Refuse package validation before constructing the native adapter |

This policy comes from Felucca source at
[`e971e8f97ed66abff20441fccdf43a23c4e9ac13`](https://github.com/kmplngj/Felucca/tree/e971e8f97ed66abff20441fccdf43a23c4e9ac13):
[`firmware/loader/loader.c`](https://github.com/kmplngj/Felucca/blob/e971e8f97ed66abff20441fccdf43a23c4e9ac13/firmware/loader/loader.c)
defines the actual handshake `ota-FM-1_900`, while
[`firmware/loader/ldr_core.c`](https://github.com/kmplngj/Felucca/blob/e971e8f97ed66abff20441fccdf43a23c4e9ac13/firmware/loader/ldr_core.c)
checks the `0x93000` layout, chip key and package CRC before writing the application.
[`firmware/src/ota.c`](https://github.com/kmplngj/Felucca/blob/e971e8f97ed66abff20441fccdf43a23c4e9ac13/firmware/src/ota.c)
uses `ota-FM-1_015` in the staging record for both loader choices. That record
text does not change the custom loader's USB handshake. The official loader
has separate whitelisted checks and writes the head too; native custom-package
validation does not establish that pairing. No `9xx` family inference is made.

Official V15 start/return remains the Felucca Python installer's and maintained
[browser installer's](https://github.com/hugelton/Felucca-WebApp) separate path;
the native CLI has no stock-package or force override. Loader identity is an
advisory compatibility check, not authentication or proof of exact loader bytes.
Both fresh and resumed native installs retain range/time limits, exact endpoint
constraints, and mandatory final application identity verification.

## Scope and hardware validation

Only FM-1 packages with the Felucca loader, its exact 0x93000-byte flash image,
valid FWSC/UFW tables/payload CRCs and valid loader headers/body are accepted.
There is no force option, package identity rewrite, stock detour, bootloader
operation, settings wipe or automatic ROM fallback. Firmware that refuses the
same numeric version times out honestly; Felucca versions permitting the same
version may install it. The post-reboot product/version reply cannot distinguish
PR commits sharing one version. For a same-identity exchange, the CLI explicitly
reports that identity cannot confirm replacement; retain the artifact provenance and inspect the
FM-1 ABOUT build commit.

This is ordinary OTA, not #15's fully qualified checkpoint-backed writer. It
creates **no raw backup or preboot full-image verification checkpoint**. The
running Felucca stages a loader at 0xE0000..0xE4FFF and an update record at
0xE4F00, then resets. The selected Felucca loader checks the chip key and writes
its application area 0x4000..0x92FFF; it may clear its update records. The host
does not prove physical readback, data preservation, rollback or bootability.
Make the backups required by your firmware/data transition separately. Do not
mistake a staging timeout for proof that nothing was written. After an interrupted
update, inspect the device state; rerunning the same package can resume only an
accepted `ota-FM-1_900` loader. No automatic retry/recovery writes are initiated
by the CLI, and an intact staging record alone does not prove cold resume.

Host evidence: independent synthetic package/transcript literals, injected MIDI
I/O and clock, range/framing/phase/send/disconnect/timeout/identity failures, and
compile-time denial of native MIDI in tests. Independent literals cover custom
fresh/resumed installs, official/unknown/wrong-model loader refusal before
phase-two START/data, CRC-valid incompatible package layout/loader, configured
application products, and final identity mismatch after both entry paths.
These tests exercise no device and
execute no firmware. #15's checkpoint/transition gates and #16's external writer
qualification remain open.

On 2026-10-19 the owner reported a successful FM-1 OTA exchange on macOS 27.0.1
arm64 with this endpoint-refresh change, using Felucca PR #477's automatic merge
build `a188b65b5d604224c1116fbed6abd5131d6e98bc` (Actions run `39257158106`).
The locally built release CLI was based on `f81a0a4` plus the endpoint-refresh
change and had SHA-256
`ca37c44b65cc05ba50c1f2db18a05d10bca4e2478161fcc19edfd1d9dc4c31c2`.
Its log progressed through loader handoff and at least 2,304 package requests, then
reconnected to running `FM-1_900`. The owner confirmed that the update worked.
This establishes that ordinary exchange on that setup; the shared product
identity does not establish the installed ABOUT commit, sound/controls, data
preservation or the separate external writer/recovery qualifications.

For a separately authorized ordinary hardware test: connect one FM-1 via USB data
cable while Felucca is running; stop transport and notes, close other MIDI apps,
use the exact PR FWSC and invoke the command above. Confirm the printed identity
and package before answering `yes`. Expect loader handoff, writing, reboot and
matching product/version; hold HOME to open MENU, release, select SYSTEM with
ALGORITHM, ABOUT with PRESETS, then tap OCT+ and confirm the PR build commit.
Report the CLI source commit, firmware provenance, host OS, input/output names,
and observed result. Cancel at the prompt to check that no upgrade starts.
Do not use the owner's sole unit for injected writer-failure or recovery cycles.
