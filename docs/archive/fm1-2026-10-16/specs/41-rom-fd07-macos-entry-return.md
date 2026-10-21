> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# ROM FD07, macOS entry and stock return — R4-C follow-up

Date: **2026-10-16**, branch `init`. Continues [exact V15 audit 40](40-v15-entry-rom-backup-audit.md).
The owner's backup-read permission remains present; flash writes are forbidden.
This increment used public primary sources, saved binaries, static analysis and
hardware-free tests. No device inventory/open/query, CoreMIDI client creation,
entry, loader execution, firmware build or flash operation occurred.

**Result:** new evidence narrows the open paths but does not qualify a physical
ROM capture. An official WL82-named ROM image is actually identical to BR28;
the precise WL82 FD07 implementation remains missing. macOS's exact selected
open/cancel/close effects are now separated, including registry iteration and
AbortPipe's controller-state effect. A deliberate manual cold-power return is
the selected candidate; FB08/FC0C are not adopted as a bare-ROM exit shortcut.
CLI discovery now reports the historical V15 artifact accurately while keeping
entry, backup and physical-write capabilities closed. The separate optional
identity-only command is unchanged.

**Static packetizer follow-up, 2026-10-16:** [report 46](46-macos-entry-packetizer-v15-receiver.md)
now resolves the class/device/C0 dispatch and the quiet six-byte encoding,
with exact V15 descriptor/IRQ/DMA/aggregation evidence. The packetizer gap
described below is the historical state of this report; actual selected
driver/object/delivery and reset/ROM/read/return qualification remain open.
The follow-up also identifies driver-owned endpoint halt clearing on callback
errors; it does not enable stock entry or change the native factory.

## ROM and reset-mailbox findings

The [official bootloader repository](https://github.com/Jieli-Tech/fw-Bootloader/tree/ff3b9299d1b94d834b995fd858b3f75e2faa9840)
contains `user_boot/cpu/wl82/output/rom.image`. However, its complete Git tree
gives that file and `cpu/br28/output/rom.image` the **same Git blob SHA-1**,
`17a2b10c3d0c21b3a6116f8c978e2697ec910b52`. The retrieved 131,072-byte image
contains `BR28    UBOOT1.00` at offset `0x45DB`; independently computed SHA-256
is `[digest omitted from public archive]`.
The repository's README distinguishes AC791N/WL82 from AC701N/BR28. This is a
foreign candidate despite its directory name; it was not remapped or
disassembled as WL82. [Image](https://github.com/Jieli-Tech/fw-Bootloader/blob/ff3b9299d1b94d834b995fd858b3f75e2faa9840/user_boot/cpu/wl82/output/rom.image),
[target taxonomy](https://github.com/Jieli-Tech/fw-Bootloader/blob/ff3b9299d1b94d834b995fd858b3f75e2faa9840/README.md).

The same revision's [WL82 mask-ROM stubs](https://github.com/Jieli-Tech/fw-Bootloader/blob/ff3b9299d1b94d834b995fd858b3f75e2faa9840/user_boot/cpu/wl82/output/maskrom_stubs.ld)
are empty. Its user-boot RAM linker and `mask_api.h` provide no exact FD07 body
or download-runtime allocation map. Path histories inspected in this pass did
not yield another image; this is a bounded negative result, not universal absence.

The manufacturer's [JTAG configuration](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/51712dd206536f165499f2313454b486b19a6175/cpu/wl82/tools/jtag/dbg.config)
declares a read-only region `[0x01E00000,0x01E02800)`, encompassing external
loader target `0x01E00428`, and reset MMIO `0x10000` bit 4. It also explicitly
labels `wl80`/`wl80_rom`; sibling configuration contains WL80/BR22 paths.
Consequently this is an inherited architectural lead, not exact WL82 code,
ownership proof or permission to read guessed memory. No JTAG program was run.

The exact V15 SPL follow-up resolves another **producer** of `usb_update_mode`:
`0x01C0529A–0x01C052A0` copies 16 bytes from marker base `0x01C05710` to the
mailbox; the branch through `0x01C048CA` calls `0x01C027CA`, whose tail sets
MMIO `0x10000` bit 4 and spins. Other marker-base references address adjacent
data. Eight small ranges, **59 instruction rows**, match the existing mapped
SPL bytes with zero unknown rows. This does not resolve the incoming marker's
reset consumer, indirect callees or all post-reset behavior. It must not be
promoted to a complete SPL/ROM no-write proof.

For FD07, the [primary protocol](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/usb-loader-v2.md#L176-L193)
warns of family-dependent source-RAM encryption, USB transfer, then restoration.
Neither that description nor the WL82 `memory-rw-mengli-crypt` host flag proves
the exact success/failure restoration paths. Still needed for the chosen capture:

- Reset/boot marker consumption and autonomous effects before ROM becomes idle.
- Exact FD07 transform/send/restore behavior, including timeout, cancellation,
  short transfer and disconnect paths relevant to that operation.
- Active stack, USB/DMA, descriptor and alias ownership for A
  `[0x01C07E00,0x01C08200)` and B header `[0x01C08E00,0x01C08E22)`.

The minimum candidate remains **512 + 512 + 34 = 1,058 bytes**, with the wire
transform restarted per request, stopping before loader upload/jump. IN direction
and equal repeated data do not establish absence of target-side RAM mutation.
Matching local implementation/allocation evidence or suitably scoped external
bench evidence can close these questions; unrelated ROM commands and whole-ROM
coverage are not requirements.

## macOS send, access and lifecycle

V15's established requirement is **the receiver's actual eight-byte buffer**
`04 F0 22 24 07 35 7D F7`, not an abstract MIDI API call. The endpoint reader
may aggregate available data; one eight-byte USB transfer is the known
Transporter candidate, not a proven architectural necessity for every sender.
CoreMIDI documents logical events and merging, and USB-MIDI defines four-byte
events; neither contract promises the receiver's read partition. [MIDISend](<https://developer.apple.com/documentation/coremidi/midisend(_:_:_:)>),
[USB-MIDI 1.0, section 4](https://www.usb.org/sites/default/files/midi10.pdf).

Pinned midir 0.11.1 constructs a packet buffer and calls CoreMIDI; its input
side accumulates segmented SysEx before callbacks. Receiving an assembled MIDI
message therefore cannot reconstruct USB read boundaries. The current CLI
backend sends only the identity query and has no entry method.
[Exact CoreMIDI backend](https://github.com/Boddlnagg/midir/blob/db32c3f4fa017d7145395bf04d220625d2733305/src/backend/coremidi/mod.rs).

The actual nusb version remains **0.2.8**, source commit
`c80218d342740321190436bc45304ebbbf16bef0`; no dependency was changed.

| Selected operation | Source-supported behavior | Consequence for R4-C |
|---|---|---|
| Lookup | Iterates USB registry services to find the exact saved registry-entry ID; unrelated services are not opened or commanded. | Correct the earlier “no discovery sweep” comment; this is metadata discovery inside a future open. |
| Device/interface open | Ordinary `USBDeviceOpen` failure may be tolerated; selected `USBInterfaceOpen` is exclusive and can refuse Busy. | Device-open Busy alone is not the refusal boundary. No seize/detach/configuration fallback is selected. |
| OUT submission | One initialized buffer produces one `WritePipeAsync`; no padding-to-64, timeout variant or force-short flag on this path. | Raw eight-byte OUT is a candidate only if exact stock interface/endpoint ownership is established. |
| Cancellation | Calls AbortPipe; a 100 ms application drain wait does not bound the synchronous OS call or final callback. | Cancellation poisons the attempt; it neither undoes entry nor establishes target restoration. |
| Normal drained close | No AbortPipe when no transfers remain; release interface, conditionally close exclusive device handle, release IOKit objects. | Host close is not a documented device reset or stock return. |
| Pending Drop | AbortPipe and callback-held ownership can defer final cleanup. | Do not treat Drop return as fully drained handles or permission to start another attempt. |

The lookup is explicit in [enumeration.rs](https://github.com/kevinmehall/nusb/blob/c80218d342740321190436bc45304ebbbf16bef0/src/platform/macos_iokit/enumeration.rs#L109-L113).
The remaining backend paths are in [device.rs](https://github.com/kevinmehall/nusb/blob/c80218d342740321190436bc45304ebbbf16bef0/src/platform/macos_iokit/device.rs)
and [IOKit wrappers](https://github.com/kevinmehall/nusb/blob/c80218d342740321190436bc45304ebbbf16bef0/src/platform/macos_iokit/iokit_usb.rs).
No native API above was called during this audit.

Apple's [IOUSBLib.h AbortPipe contract](https://github.com/apple-oss-distributions/IOUSBFamily/blob/1398331b04a6bb9ea9b9f76248b8b584811ebcd0/IOUSBFamily/Headers/IOUSBLib.h#L1980-L1992)
specifies a **controller endpoint halted-bit clear**, without a data-toggle
reset. This differs from device CLEAR_FEATURE/ClearPipeStall, which our binding
does not request. Earlier generic statements of “no halt clear” need this
distinction. Historical API documentation is not a measured current-kernel
latency or FM-1 behavior result.

Our actual selected binding admits only the loader's `4C4A:8057`, interface
class/subclass/protocol `08/06/50`, endpoints `01/81`. It cannot be reused
unchanged for stock MIDI entry. Ordinary interface access depends on existing
configuration and ownership; a kernel-owned interface must refuse. Current
source does not establish which driver would own this unit's interface after
reenumeration. No detach or alternate SCSI backend is an implicit fallback.

A reset creates a new USB registry object. Old registry/CoreMIDI IDs are not
capabilities for the new object. A saved port/location helps correlate topology
but alone cannot exclude substitution or ambiguous arrivals. The intended
transition must bind selected MIDI endpoint → pre-entry USB object → removal →
one expected new ROM object; matching VID/PID or cached descriptors is inadequate.
Actual selected access/cancellation/timing and this continuity are still unobserved.
[Apple reenumeration/access guidance](https://developer.apple.com/library/archive/documentation/DeviceDrivers/Conceptual/USBBook/USBDeviceInterfaces/USBDevInterfaces.html).

### Installed Apple driver: bounded static result

Read-only system-file metadata reports macOS **27.0.1, build 26A434** and
AppleMIDIUSBDriver **1.10/build 88**. The installed universal Mach-O is 704,624
bytes, SHA-256 `[digest omitted from public archive]`.
Static analysis selected the third ARM64 slice, subtype `0x8000000C`, SHA-256
`[digest omitted from public archive]`.
These are installed-file facts; no loaded-driver, runtime-slice or device
ownership observation was made. Other slice addresses must not be substituted.

The selected `Send` path at `0xDE44` copies logical packets into queue nodes
and schedules work. `DoWrite` at `0xE940` reaches a legacy packetizer through
driver-vtable `+0xC0`; the distinct UMP path uses `+0xC8`. The returned byte count
then feeds an interface call identified from IOUSBLib layout as WritePipeAsync.
On immediate error codes `0xE000404F` or `0xE00002ED`, the selected code calls
that interface slot again with the same buffer/count. This is a bounded static
driver error-path retry, not an observed duplicate command on the FM-1.

The exact legacy packetizer target, class binding and capacity/flush behavior
remain unresolved. A separate helper named `appendSysExToBuffer` processes UMP
words and must not stand in for the legacy MIDI1-to-USB path. Thus this analysis
narrows the next step without claiming complete six-logical-byte to
eight-received-byte delivery. One application send also cannot be described as
proof that lower layers never retry.

An independent file decoder verified **557 LC_FUNCTION_STARTS entries** and
**1,189 instruction rows across six selected functions**, with zero byte
mismatches. Hopper's newer-PAC instruction coverage was incomplete; its negative
xrefs do not prove absence. Existing LLVM 23.1.3 decoded the selected ranges;
no whole-driver disassembly, target execution or installation occurred. REA
session closed. Source identity Evidence
`ev_[digest omitted from public archive]`
and retained selected-analysis receipts are bound in the evidence manifest.

The smallest next static task is to bind the object at USBMIDIDevice `+0x50`
and its `+0xC0` packetizer, then inspect its single quiet SysEx capacity/flush
path and exact interface dispatch. A scoped external observation may instead
establish the receiver-buffer result for this selected route. Neither requires
a universal guarantee for every packet size or queue state.

## Return to original stock

**Select deliberate manual cold-power return before entering the intended read
stage.** Use it after completed, quiesced transactions with no flash operation
underway. This is the smallest source-supported candidate; it is not an automatic
error handler or already-qualified return on this device.

| Stage | Selected disposition |
|---|---|
| Bare ROM, before FB08 | No guessed exit CDB/address. Preserve the bounded capture and receipts, complete/release the qualified access path, then use the preselected manual cold return. |
| Unchanged loader, successful no-FC0A full read | Complete two fresh full reads, durable independent copies and final continuity, quiesce, then deliberate cold return. FC0C is unnecessary. |
| Timeout, pending/ambiguous cancel or drift | Fail acceptance and retain evidence. No automatic reset, reentry, retry or power cycle follows uncertainty. |

`FB08` executes supplied code with an argument; zero is not a documented ROM
exit. `FC0C` belongs to loader-dependent implementations, not a generic bare-ROM
return. For this exact loader its local path requests MMIO reset; command
completion alone does not prove stock return. [Protocol variants and jump](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/usb-loader-v2.md#L195-L255),
[exact local return analysis](30-loader-read-return-callpaths.md).

Transporter's later known-limit report says V15 may run after warm RUN_APP or
watchdog reset while normal USB stays absent until cold power-on. Its earlier
expectation of automatic reattachment must not override that later observation.
Closing a macOS handle is not equivalent to releasing an RP2040 USB host, and
unplugging USB need not remove the FM-1's battery power.
[V15 return limit](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/docs/DEVLOG.md#L145-L151).

The [external Lunar bench](https://github.com/ip2k/lunar-modulator/blob/bfa042446f36c8647efddc6d76b06753d2a5b612/notes/2026-10-07-fm1-softkey-bench.md#L71-L171)
observed cold return after a loader session using FC0A, with later identity
FM-1_092 and a separate write experiment. It did not demonstrate this owner's
stock-V15 bare-ROM A/B or unchanged-loader/no-FC0A return. No write experiment
is reproduced or needed here. Zero mailbox observations do not prove reset
clearing semantics or authorize a mailbox write.

Successful stage-specific acceptance must observe normal display/operation,
normal USB and one fresh selected identity query after the chosen return.
Enumeration, a host boolean or ignored command error is insufficient. The first
qualified capture can supply this observation; no older full backup or restore
experiment is required beforehand.

## Offline implementation and remaining work

`backup entry assess --platform macos --stock-version v15 --json` now labels
the exact historical manufacturer artifact and its RAM/reset effect boundary.
Other reported versions receive no borrowed V15 artifact evidence. Installed
firmware proof, framing, flash preservation and all access/entry/write flags
remain false; an artifact report cannot create the private qualification permit.
The misleading native-access comments were corrected without changing calls.

R4-C remains partial. Its next concrete dependencies are the selected reset/ROM
FD07/ownership evidence, receiver-buffer result, selected access/object continuity
and stage-specific manual-return effects. C4 still stops before loader upload;
external startup/chip/callback/read qualification belongs to C5. No Linux/Windows,
writer/restore or universal ROM proof is added as a prerequisite. Existing read
permission needs no renewal once the actual operation qualifies.

An optional subsequent offline stock-entry transition implementation must remain
separate from the loader binding and model one selected send, exact arrival
ordering, ambiguity/timeout refusal and cleanup uncertainty. Such a model could
test future host orchestration, but would not close the missing physical evidence.
This increment does not add another model or open any hardware factory.

## Evidence and validation

GPT 6.1 Sol **high** subagents audited ROM and macOS paths; **medium** researched
return sources. Root integrated source corrections, CLI artifact discovery and
independent regression checks. Raw vendor/system binaries, source snapshots,
REA/static notes and logs stay ignored under `[local artifact location omitted]`.
The sanitized evidence manifest (historical reference `evidence/41-rom-macos-return.json`; omitted from public archive) retains
source URLs, artifact hashes and scope/qualification distinctions.

- Canonical guarded Python 3.12 runner: **317 tests passed in 231.845 s**, no skips.
- Canonical native checks: **three selected stock-backup library tests** and
  **six stock-backup integration tests in both debug and optimized builds**.
  The new independent matrix checks all 12 platform/reported-version combinations:
  only V15 receives historical artifact metadata and no combination gains
  firmware/framing/access/entry/write proof. Per-run source fingerprints stayed stable.
- Core audit: **48 Rust files**; native-USB audit: **nine files, 34 pinned registry
  dependencies**. Formatting, whitespace and evidence/hash/link checks passed.
- Default-feature-free offline locked release build passed. The actual default
  executable's entry-assessment/status outputs show historical artifact evidence,
  blocked entry and false hardware/write availability. No optional backend ran.
- No full writer sweep, physical test, new OS build qualification, hosted CI or
  release qualification is inferred. Existing firmware/PIO/submodule/license and
  dependency-lock inputs remain unchanged. Pre-existing rescue-source WIP is
  preserved outside this local increment; no push occurred.
