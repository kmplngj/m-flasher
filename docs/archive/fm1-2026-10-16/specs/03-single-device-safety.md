> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Single-device development safety and recovery preparation

Date: 2026-10-11; backup authorization updated 2026-10-15. This is an enforceable implementation policy, not a guarantee that single-bank flashing can never fail.

## Owner constraint

There is exactly one FM-1. The owner reports no backup or recovery dongle yet. A separately authorized CLI identity query on 2026-10-15 returned `FM-1_015`, corroborating the browser's V15 report; firmware integrity, board revision and flash-chip identity remain unknown. See [report 26](26-native-midi-identity.md). The owner is willing to build a recovery kit and authorized the FM-1 Transporter fork and sequential offline implementation.

**Authorization update, 2026-10-15:** the owner permits hardware access for full-backup work and explicitly prohibits flash writes. FM1-033 starts with D0 offline implementation; all development/tests remain hardware-free. Reviewed identity/read operations may use this permission once an exact target and evidenced read path exist. Do not probe guessed commands or infer entry/loader safety from permission. The separate identity-only hardware observation did not enter ROM, execute a loader or capture raw flash; no full-backup session has been performed. The repository's offline-development instructions continue to govern tests and simulations.

**No experimental writes on that unit.** Do not intentionally brick it, pull a cable while writing, interrupt power, alter the SPL/header, probe guessed commands, write test patterns, run arbitrary RAM payloads, or install an unqualified firmware to see what happens. Do not reproduce an upstream destructive experiment locally. A backup and a dongle do not relax this rule.

If “never brick during testing/development” is absolute, keep the device outside the write-test loop. Use a simulator, virtual endpoints, recordings and a spare/external qualification unit. If no spare or external qualification is available, real writing remains blocked; there is no honest software shortcut around that gate.

The [agent-driven development goal](08-agent-firmware-validation.md) inherits this restriction. Its future physical flash/boot/health-check loop belongs on a separately enrolled spare/external bench, never the owner's only FM-1. A controlled boot is a separate, explicitly requested and qualified stage after full preboot raw verification; transfer completion or a failed health check cannot trigger an automatic reset, reflash or rollback. Documenting this goal grants no hardware permission.

## Why a host tool cannot promise zero bricks

The layout is single-bank. Interrupted programming may leave a mixture of old/new application sectors. A maintainer reported precisely this on 2026-10-07: 33 new sectors, 110 old ones, intact head and staged loader, and no USB enumeration after cold boot. A Transporter recovered that unit on 2026-10-09. The SPL/flash record was not a demonstrated cold resume path. [Incident and dump](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/INSTALL-COMPAT.md).

ChoralRoot later added a boot stub/commit marker and reports a successful same-stub cable-pull rescue on one device. Its cut-point model still has 16 uncovered points when the stub sector is first installed or changed. Earlier text within its boot-safety document is stale relative to the later unit-test section; use the final results without treating them as a universal guarantee. The host cannot install a recovery stub with zero power-loss risk, and adopting one would change the firmware rather than merely improve the host. [Boot safety and device results](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md).

Immutable ROM recovery survives corrupt application bytes; it still needs reachable hardware, correct entry, an appropriate loader, reliable power, a supported flash chip and a trustworthy restore image. A physically defective device, damaged USB/power hardware or an unsupported boot head is outside the product's unbrick claim.

## Optional emergency recovery kit

This accessory supports external entry when a broken application no longer enumerates over USB. The owner prefers normal installation with one USB-C data cable; the RP2040 is not a routine-install requirement. USB-only installation still needs exact qualified protocol, backup and verification evidence.

Choose **standard Seeed Studio XIAO RP2040, SKU 102010428**. [Official product](https://www.seeedstudio.com/XIAO-RP2040-v1-0-p-5026.html), [official pin documentation](https://wiki.seeedstudio.com/XIAO-RP2040/).

Use the exact board qualified by [FM-1 Transporter](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/README.md). The current shop also offers RP2040 Plus and other XIAO variants; this research has not qualified those substitutions. Upstream explicitly marks other RP2040 boards untested and the Waveshare RP2350-USB-A unsupported.

Parts:

- One standard XIAO RP2040; headers are optional depending on soldering preference.
- A USB data cable connecting the computer to the XIAO.
- A USB breakout/cable adapter with clearly labeled **D+, D− and GND** and a data connection to the FM-1's USB-C socket. USB-A breakout plus an appropriate USB-A-to-C data cable is one documented arrangement; direct USB-C breakout needs independently checked pin routing.
- Three insulated wires, soldering equipment or suitable connectors, strain relief, and a multimeter for continuity/short checks.
- A fully charged FM-1 battery; a second independent location for backups.

The documented signal mapping is XIAO **D6/GP0 → D+**, **D7/GP1 → D−**, **GND → GND**. **Do not connect VBUS/5 V between the XIAO and FM-1.** The FM-1 uses its battery; the computer powers the XIAO. This mapping is a design reference, not a command to assemble/connect hardware during this research. Validate the exact adapter's routing and isolation before use. [Recovery parts/wiring guide](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/TRANSPORTER-GUIDE.md).

**USB_KEY source discrepancy, 2026-10-16:** the dedicated key notes assign D+ to
clock and D− to data; the older tool guide reverses these roles in its prose,
despite its own waveform. [Recorded comparison](43-full-raw-dumper-product.md#usb_key-documentation-conflict)
retains both sources. Do not infer signal roles from board-pin labels or try
alternate wiring/signals on the owner's FM-1. Bind the exact activator source,
pin routing and matching waveform/ACK evidence during kit qualification; isolated
activator/external-bench measurements may resolve missing evidence. This adds no
dongle prerequisite to a qualified direct-USB software-entry backup.

## Capability levels

| Level | Allowed work | Gate |
|---|---|---|
| D0: offline | Research, package parsing, simulation, tests, static analysis, builds without hardware | Ready now |
| D1: passive inventory | OS device listing only; no MIDI/serial/SCSI opens or commands | Owner permitted backup hardware access on 2026-10-15; separate from tests |
| D2: bounded inspection | Reviewed identity query; known bridge/ROM entry and volatile loader upload; raw flash reads only | Backup permission exists; exact target, audited entry/image and known protocol still required; no guessed probes |
| D3: backup acceptance | Two identical full reads, stable metadata, durable files and a second copy; offline expected-image planning | Backup permission exists; exact identity/layout and evidenced read path still required; no erases/programming |
| Q: external qualification | Writer, rollback, interruption and migration tests on a spare/external unit | Never the owner's only FM-1 |
| R: qualified production write | A released exact profile/image on one explicit device | Q evidence + all checks + a separate owner decision about residual risk |

Even D2 is not passive: ROM entry changes the running mode and a loader upload executes code in RAM. Label it honestly. The 2026-10-15 permission supersedes the earlier D0-only access restriction for backup work; it does not authorize flash writes or unreviewed payloads. This specification does not grant additional permission by itself.

## Enforced software invariants

1. **No real hardware in tests/simulation.** Tests use injected adapters. The simulation command neither enumerates nor imports/open real device backends. CI has no attached FM-1. Fail the suite if a test calls an OS hardware API.
2. **No boot-head erase/program.** Every host erase/write is sector-aligned and contained in an exact operation allowlist; `[0,0x4000)` is always forbidden. Keep the bridge's independent `[0x4000,0x93000)` gate. No `erasechip`, arbitrary address, hidden override or full-chip restore command.
3. **No default device.** Select one explicit FM-1/Transporter pair and verify protocol/chip identity; reject ambiguous ports/disks and re-identify after reconnect. A name, VID/PID or shared firmware number alone is insufficient. Never send probes to all MIDI instruments or generic serial ports.
4. **Immutable inputs.** Verify package/profile/loader digests before entry and before execution; execute from the checked bytes. A modified file, changed source snapshot or changed device invalidates the plan.
5. **Backup before destructive action.** A double-read full snapshot must already exist durably, be reopened/hash-verified and have a second copy. Never overwrite the original. A preset/editor backup is not a full raw recovery backup. No supported raw read path means no write under this policy.
6. **Bounded writes and whole-image verification.** Compute the expected full image from the snapshot. Program only differing approved sectors; verify each, then the full image before restart. Protect head, key/mac/calibration and all untouched data. Any failed comparison remains a failed operation.
7. **No hidden data reset/migration.** Preservation may still be incompatible with a target; then refuse. A future migration needs a reversible offline transform, exact region policy and separately reviewed acceptance. Do not expose a generic wipe.
8. **Loader audit applies to MIDI.** The host does not control a target OTA loader's physical writes. Stage code and loader digests must prove the declared write ranges; the expected image must include all staging/record/data side effects. Full raw readback needs a qualified held-session checkpoint before the app boots, which ordinary existing OTA has not demonstrated. Thus MIDI writing remains disabled initially. Official-stock loader rollback is excluded from the strict head-never-written route.
9. **Honest progress and failure.** Bytes served/request count, loader completion, raw readback and postboot functionality are distinct. Do not print “safe” or “done” because 100% of bytes were sent. Failures keep logs and identify the next reachable recovery path.
10. **No blind automation.** Do not power-cycle, change firmware target, clear data or retry an entire operation automatically after an ambiguous failure. Resume only an exact qualified job/device/image. Keep a reachable ROM/bridge session open while diagnosing a failed write; if it is already gone, re-establish entry through the documented external route.

The existing Transporter application-only gate is narrower than the unbricker's low-level flash-capacity gate. If reusing the latter, enforce both lower and upper operation bounds at every entry point, including retries and rollback.

## Offline tests that protect these invariants

- Golden `.fwsc` → logical → entry → raw-region fixtures and independent cross-checks; regression for the `0x400` extraction shift; malformed/truncated/overlapping entries and CRC/hash failures.
- SysEx packing/unpacking and byte-exact framing, partial/odd lengths, re-ordered/duplicate requests, wrong checksums, out-of-range requests and stale completion markers.
- NOR model with real erase-before-program and 1→0 programming semantics; torn pages/sector erase, full flash and metadata persistence across virtual power loss.
- Crash/fault injection before and after every destructive operation: host exit, timeout, disconnected transport, queue overflow, changing identity, corrupt read/write, dead cell, journal/disk-full/fsync failure.
- Assert no protected byte changes on success or failure. Assert backup/plan failures produce zero erase/program calls. Assert unchanged sectors are not erased.
- Stateful/random traces alongside deterministic all-cut-point tests. Capture a reproducible seed/trace for every failure; no probabilistic result substitutes for exhaustive bounds checks.
- Separate models for app-dependent MIDI and application-independent ROM/bridge entry. A simplistic mock that always returns ROM after a failed update would conceal the observed no-USB failure.
- Replay on macOS/Linux/Windows fake/virtual ports; test busy handles, reconnect and device selection. Virtual-port success is software evidence only.

## Direct USB-C stock-backup milestone

The owner authorized backup hardware access on 2026-10-15. FM1-033 starts offline
and prefers a direct computer USB-C connection with installed stock unchanged.
An actual session needs exact identity, reviewed stock-to-ROM entry where needed,
an audited volatile loader and supported selected-device host access. Two fresh
equal full reads and durable independent copies precede acceptance, followed by
the documented normal return to original stock. The [stock-route research](16-stock-usbc-backup.md)
records the remaining evidence; no RP2040 or firmware installation is an implicit
fallback. No additional permission question is needed for the already authorized
bounded reads once their concrete requirements are met. Entry/RAM execution is
mode-changing and cannot be described as passive or inherently free of risk.

## First recovery-kit milestone

Record the current firmware/USB identity without changing firmware. Prepare and verify the exact XIAO image with the FM-1 disconnected; inspect wiring/continuity and power isolation. Use an audited upstream read-only workflow, not the new writer. Enter the documented recovery transport only within the agreed scope, capture chip/flash identities, make two full dumps, compare them, save hashes/metadata and a second copy. End the known read session using its documented exit procedure and confirm the original firmware still boots.

This milestone demonstrates entry/read/normal return on the owner's device. It **does not prove erase, programming or restoration**, and must not be labeled “recovery fully proven.” Those gates require spare/external evidence. Do not manufacture a brick merely to prove the dongle.

## Release checklist

An enabled transition needs exact source/target/hardware/data profiles, offline invariant evidence, externally qualified writer/recovery evidence, reviewed loader behavior, OS/transport evidence, verified artifact provenance and a documented residual-risk statement. Hardware qualification cannot be waived by an override. If the owner continues to require absolute zero risk, release the read-only/offline tool and leave write capabilities disabled.
