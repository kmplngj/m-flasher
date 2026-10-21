> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Actual USB binding code with activation closed

Date: **2026-10-15**. FM1-033-D2 continuation on `init`. This increment adds
actual macOS/Linux nusb API calls in the isolated native-usb package (historical reference `../native-usb/README.md`; omitted from public archive).
The original local macOS native feature compiles; Linux arm64 compilation was
subsequently added in [report 28](28-first-backup-readiness.md). Other Linux
architectures and all platform hardware qualification remain pending.
**No hardware operation was executed. The public factory always refuses.**
This is implementation/build evidence, not a working stock-device backup.

## Package and activation boundary

The default CLI and existing `native/` library do not depend on this package.
Its default features are empty. Only the opt-in `native-usb` feature on
macOS/Linux compiles the actual binding module. Our Rust code forbids unsafe.
The dependency has its own native bindings; that fact does not qualify them.

`open_selected` returns `QualificationBlocked` before any native operation.
The private real implementation requires a private empty enum permit. There
is no variant, constructor, call site, environment switch or test bypass.
Even enabling the feature does not enable connection. Capability reporting
keeps real open, hardware qualification, storage integration and writes false.

The actual body typechecks selected-object open, ordinary interface claim,
endpoint creation, queued transfers, completion mapping and handle ownership.
This advances beyond the injected callbacks in [report 21](21-injected-native-read-driver.md).
Actual provenance is not converted into fictional `InjectedReadIo`, schema-2
storage records or writer authority. A future storage bridge needs its own
provenance contract and held-session checks throughout persistence.

## Dependency choice and source review

nusb is pinned to **0.2.8**, registry checksum
`[digest omitted from public archive]`,
source revision `c80218d342740321190436bc45304ebbbf16bef0`.
The previously researched 0.2.3 candidate is superseded in this new package.
Upstream 0.2.4 fixed a `wait_next_complete` deadlock involving out-of-order
callbacks; later 0.2.8 also fixes a macOS/Linux control-transfer future leak.
The binding does not use control transfers. These fixes justify the newer
exact source pin without asserting physical safety.
[Official releases](https://github.com/kevinmehall/nusb/releases),
[pinned device API](https://github.com/kevinmehall/nusb/blob/c80218d342740321190436bc45304ebbbf16bef0/src/device.rs).

`Cargo.lock` and `dependency-policy.json` record **34** registry packages,
including target-conditional Windows/WebAssembly dependencies. Their versions,
checksums, SPDX licenses and build-script presence were read from fetched source.
The notices (historical reference `../native-usb/THIRD_PARTY_NOTICES.md`; omitted from public archive) retain upstream license text.
Source preparation fetched Cargo packages; no OS driver was installed. Cargo
build scripts are compilation inputs, so this guard is not a sandbox against
malicious dependencies or toolchains. Release artifact/license qualification
and supply-chain review remain separate from this source increment.

## Exact selected-device path

The caller must supply one `DeviceInfo` and saved device/configuration/INQUIRY
bytes. The constructor reads only that object's cached metadata; it performs
no inventory or descriptor request. The binding records OS object identity,
bus/port chain, address, VID/PID/revision/USB version, optional serial and the
OS-specific location/sysfs path. Private Debug output omits identifiers/bytes.

The accepted consistency shape is deliberately narrow: VID/PID `4C4A:8057`,
nonempty bus/port path, nonzero address, matching device descriptor, one
saved configuration (value 1), one interface at alternate 0, class/subclass/protocol
`08/06/50`, and exactly two 64-byte bulk endpoints `0x81` and `0x01`.
These saved values do not authenticate stock firmware or a running loader.

The unreachable implementation checks selected metadata before open, compares
opened descriptor/configuration bytes, claims interface 0 without forced
detach, checks its alternate/descriptor and creates those endpoints. It exposes
no public discovery, reset, device clear-halt, configuration/alternate change, control request,
MIDI entry, loader upload/jump, keepalive or flash-write API.

Source inspection also identifies limits that must survive future activation:

- macOS nusb open first attempts a whole-device exclusive `USBDeviceOpen`.
  **Follow-up, 2026-10-16:** exact-ID lookup iterates registry services; unrelated
  services are not opened/commanded. Device-exclusive-open failure may be
  tolerated; ordinary selected `USBInterfaceOpen` is the exclusive Busy/refusal
  boundary. Ordinary claim alone does not describe the entire access lifecycle.
- Linux opens the saved bus/address; address reuse can defeat a stale selection.
  Matching cached descriptors cannot prove continuity of a physical object.
- Linux active-configuration access rereads sysfs but falls back to cached
  information on error. macOS configuration information is cached. Neither
  proves current RAM-loader execution or flash preservation.
- Open/claim waits, endpoint cancellation and Drop/release effects need platform
  qualification. nusb's explicit interface release has the same effect as Drop;
  it does not establish bounded or reset-free teardown.
- [Report 41](41-rom-fd07-macos-entry-return.md) distinguishes cancellation's
  AbortPipe controller halted-bit clear from device CLEAR_FEATURE/ClearPipeStall
  and data-toggle reset. Drained normal close skips AbortPipe; pending cleanup
  can retain callback-owned handles. No new operation or activation is selected.

[Pinned macOS implementation](https://github.com/kevinmehall/nusb/blob/c80218d342740321190436bc45304ebbbf16bef0/src/platform/macos_iokit/device.rs),
[pinned Linux implementation](https://github.com/kevinmehall/nusb/blob/c80218d342740321190436bc45304ebbbf16bef0/src/platform/linux_usbfs/device.rs).

## Finite BOT engine and transfer queue

The private engine constructs only canonical INQUIRY and fixed sequential
512-byte `FD05` raw reads over the modeled 1 MiB capacity. Exactly one INQUIRY
and two complete sweeps are allowed: **4,097 transactions**, distinct nonzero
tags, no arbitrary caller-supplied CDB. Every second-sweep block must equal
the corresponding retained first-sweep block. Capacity is a model assumption
until the exact device/flash/layout is independently established.

Each transaction sends a 31-byte read CBW, receives 36 or 512 logical data
bytes and a 13-byte CSW. All statuses, actual counts, initialized input bytes,
CSW signature/tag/zero residue/success status must match exactly. Native input
capacity rounds up to the 64-byte packet size; the logical result is never
padded or truncated to hide a short/excess completion. Error status remains
an error even when a full byte count is present.

A monotonic local lease is at most 600 seconds; each transaction shares a
five-second budget across phases. Both lease and transaction checks run after
consistency callbacks as well as before transfers. This detects elapsed work;
it cannot preempt an OS operation or attest to a live loader lease.

The same private queue helper is used by actual nusb endpoints and injected
queues. It refuses preexisting pending work/zero budget, deducts submission
time, preserves native completion status/count/data and treats late success as
deadline failure. On timeout it cancels and allows at most 100 ms of explicit
drain waiting. It avoids nusb's convenience blocking-transfer path, whose
cancellation wait is unbounded. Kernel/framework and destructor completion can
still exceed the local wait bound. No protocol retry/reset follows failure;
the session becomes terminal and requests host cancellation once.

## Offline guard, tests and current evidence

The canonical runner audits the exact manifest, empty defaults, locked pins,
five source files, module gate, constant-refusal factory and empty permit before
Cargo. It rejects build hooks, additional targets, native APIs outside the
compile-only module, repository Cargo configuration and compile-time file
include macros. Source/tool/dependency fingerprints are checked after build
and after test execution. This is a regression guard, not an OS sandbox.

Only default-feature-free injected tests execute. `--check-native` performs
`cargo check` with actual calls compiled and never runs a native-feature test.
Tests neither construct nusb objects nor invoke the closed factory with a real
selection. The refusal and isolation are source-audited rather than presented
as a physical or runtime device test.

Validation on local macOS arm64, Rust/Cargo 1.99.0 and Python 3.12.15:

- **26 debug and 26 optimized injected tests passed**, covering every command,
  tag, address and byte of both fresh reads, malformed/error/short/excess
  completions, CSW faults, drift, mismatch, terminal state and time budgets.
- Five queue cases cover pending/zero-budget refusal, time spent submitting,
  remaining wait, completion preservation, cancel/drain/no retry and late success.
- Deterministic clock regressions cover exact/beyond lease expiry during a
  checkpoint and final INQUIRY/read checkpoints crossing transaction expiry.
- **12 file-only guard tests passed**, including refusal for absolute, escaping,
  computed and relative compile-time file input, factory/permit/default changes,
  native probes and build hooks. Format and source/dependency audits passed.
- **Actual macOS nusb-feature compilation passed**. No native-feature test ran.
- Canonical unchanged Python guard runner: **302 tests passed in 200.290 s**,
  no skips. No hardware API was used by the test suite.
- The existing native core's source/API audit passed (37 Rust files); its prior
  53-file input map remains byte-identical. Its expensive unchanged writer
  suite was not rerun; [report 21](21-injected-native-read-driver.md) retains
  that earlier source's 193-test result.
- Local Linux cross-check stopped with `E0463` because this Homebrew toolchain
  lacks Linux `std/core` and has no rustup. **Linux compilation did not pass.**
  The added Ubuntu/macOS workflow is configured but has not run or been pushed.

Logs and the exact current input map remain under ignored `native-usb/target`.
The 11-file binding input-map SHA-256 is
`[digest omitted from public archive]`;
it includes the shared core audit and binding guard tests, alongside Rust,
manifest, lock and dependency-policy inputs.
The patterned image is independently specified as
`(i * 73 + i // 251 + 19) % 256` for each of 1,048,576 bytes; its SHA-256 is
`[digest omitted from public archive]`.
The test's FNV coverage oracle is not used as backup cryptography.
No new CLI artifact size or release qualification is claimed.

## Owner-supplied preliminary USB observation

On 2026-10-15 the owner supplied a macOS System Information screenshot and
reported **macOS 27, Apple Silicon, stock firmware with unknown version**.
The displayed device is `FM-1`, VID/PID `4C4A:C755`, USB device revision
`0x0100`, link speed 12 Mb/s. Serial and location identifiers are deliberately
omitted from this public-style report. The screenshot is not a repository
fixture, and the implementation did not enumerate, open or command hardware.

This is owner-supplied OS inventory evidence only. The identifiers are consistent
with the documented application-mode candidate; they do not authenticate stock
firmware, prove V15, establish layout/capacity, qualify MIDI entry, or demonstrate
the `4C4A:8057` ROM/read path. USB device revision is not an authenticated
firmware version. No ROM entry, loader execution, raw read, backup acceptance or
normal-return test is evidenced. Keep the native factory closed. Further
hardware instructions need a reviewed route for this still-unknown stock version;
the existing no-flash-write constraint remains in force.

The owner subsequently reports stock **V15** from fm1-editor.com's identity
display on that Mac. This is a reported UI result, not a strict raw reply or
firmware-integrity proof. [FM1-034](26-native-midi-identity.md) adds the separate
bounded native MIDI identity query; the USB/loader factory described here stays
closed, and no CLI hardware query has been performed during development.

## Loader audit and remaining ordered work

[Static follow-up 23](23-loader-static-followup.md) independently decodes the
exact 24,064-byte USB loader using the pinned per-512-byte transform. Decoded
SHA-256 is `[digest omitted from public archive]`.
Root independently reconfirmed the transform, roundtrip, digest and four
candidate reference byte offsets. Flash protection/status-register/quad-mode
diagnostics make a specific transitive audit necessary. They prove neither
that startup/read writes flash nor that such effects are absent. No loader was
executed or qualified, and no source/build/redistribution linkage was established.

Continue D2 in this order:

1. Obtain actual Linux build evidence and review platform-specific effects.
2. Recover loader control flow for startup, `FD05`, protection/status register
   paths, watchdog, failure and return; resolve exact-stock entry evidence.
   An audited loader replacement would be a separate provenance/build task.
3. Specify actual native read/storage provenance and held-session continuity
   without promoting fictional records or cached consistency to physical proof.
4. Qualify open/claim/cancel/release, device continuity and loader lifetime on
   the exact platform/route. Keep activation closed while these gates remain.
5. Only then perform FM1-033-E under the existing bounded backup authorization:
   exact identity/capacity, two fresh equal complete reads, durable independent
   copies and evidenced normal return. No flash writes or development fault
   tests on the owner's only FM-1.

Running-stock raw reads remain unknown; temporary ROM/loader access is a
different capability. USB-C is still the intended route, with no implicit
RP2040 requirement, firmware replacement, V15 detour or automatic fallback.
