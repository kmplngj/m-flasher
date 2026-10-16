> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Offline macOS/Linux read adapter contracts — FM1-033-D1

Date: **2026-10-15**. Branch: `init`, based on the locally committed FM1-033-C
increment `28f7a3b78f83cf914466956b8e79c9f6013ec04b`. This increment implements
the offline portion of D. Native device connections and physical qualification
remain D2; the actual owner-device backup remains E.

Only local source, generated fixtures, saved completion records and guarded
tests were used. Platform documentation was read online. No hardware inventory,
enumeration, open, probe, command, physical read/write, backend installation,
firmware build or loader execution occurred.

## Result and stock-device readiness

Rust now validates Linux SG_IO-shaped results and macOS IOUSBHost-shaped bulk
results against one exact fictional target and canonical read request. A finite
replay consumes an INQUIRY and two complete equal modeled 1 MiB reads on either
platform. Errors permanently poison the replay; there is no automatic retry,
reset, alternate target, firmware switch or resume.

**This does not yet allow a backup of a new stock device.** The modules are pure
result contracts, with no native handles or transport factory. A normal-running
stock raw-flash endpoint remains unqualified. The temporary ROM/RAM-loader route
is a separate candidate, with unresolved exact-stock entry and loader effects;
see [route research 16](16-stock-usbc-backup.md).

A future bounded stock-backup task needs all of:

1. Exact device/source-firmware identity, independently established flash
   capacity/layout, a reviewed entry path for that stock version and audited
   loader bytes whose execution preserves installed flash. No guessed commands.
2. A reviewed native connection on the chosen OS, including access policy,
   [local artifact location omitted] binding, actual transfer-length/error extraction, loader
   checkpoint preservation and kernel/framework side effects. D1's supplied
   records cannot prove these properties.
3. Integration of actual fresh reads with secure two-copy capture, approved
   destinations and per-platform acceptance evidence. E must record two equal
   full reads, durable copies and return to original stock operation. A backup
   test cannot establish restore/write qualification.

The intended direct USB-C route requires neither a firmware installation nor a
V15 detour or RP2040. Compatibility with an unknown new stock version is not
established. The existing backup-only permission does not expand this task:
the owner explicitly requested offline implementation with no hardware access.
Experimental writes and failure injection on the only FM-1 remain excluded.

GPT-6.1 Sol high implemented each platform contract and the replay; medium wrote
independent public-API conformance tests. A high agent independently reviewed
the common boundary, Linux implementation and replay without an offline blocker.
Root integrated selection/request types, status, the guarded test target and docs.

## Common request and authority boundary

`usb_adapter::SelectedReadTarget::bind_saved` requires an explicit unique saved
descriptor, macOS or Linux, the existing exact ROM-interface shape and a
caller-reported held-loader mode. OS-object and session names must be bounded
`simulation:` identifiers. This does not inspect or authenticate a device.
Before/after comparison includes the descriptor, identity/location, fictional
OS object and session. Debug output redacts these private values.

`ReadRequest` has only canonical INQUIRY and LoaderV2 FD05 constructors. Tags
are nonzero; requests contain at most 512 bytes inside the modeled 1 MiB flash;
timeouts are 1–5,000 ms. There is no arbitrary CDB, caller-selected command data
direction, RAM upload/jump, erase/program or preset-write constructor. A BOT
CBW travels OUT as transport framing; that does not authorize an OUT command.

`ValidatedReadData` contains unprivileged modeled bytes plus [local artifact location omitted]
bindings. It cannot create a `BotReader`, `BotSnapshot`, `UsbBackup`, current
session permit or writer authority. Saved completions are caller-controlled
input; matching hashes and successful validation prove internal agreement only.
Public reports expose hashes/counts and false physical/loader/source/durability/
writer flags, with no raw data or target identifiers. Explicit data accessors
remain private-material APIs, separate from sanitized reports.

## Linux SG_IO-shaped contract

`linux_usb::LinuxSgRequest::prepare` produces an owned field record, not a C ABI
structure or syscall. It fixes interface `S`, FROM_DEVICE `-3`, indirect IO with
flags zero, no iovec, a 32-byte sense capacity and the exact canonical CDB,
transfer length and timeout. The sense capacity and narrow success policy are
our model choices, not universal kernel limits. Request tag/hash association
fields are replay metadata rather than Linux ABI fields.

Validation compares every supplied original input field and target, then requires
syscall return zero, no OS error, zero SCSI/masked/message/host/driver status,
no sense, zero info, signed residue exactly zero, bounded duration and exact
realized data length. Negative residue, short success, padding, trailing data,
unknown bits, direct/mixed IO, stale object/session or changed command refuse.
There is no pointer, ioctl, path open, device discovery or native retry.

The field layout and constants were checked against the pinned
[Linux v6.17 SG header](https://github.com/torvalds/linux/blob/v6.17/include/scsi/sg.h).
The [v6.17 SG driver](https://github.com/torvalds/linux/blob/v6.17/drivers/scsi/sg.c#L216-L224)
also checks command permission against the file's write mode; its open path
rejects O_RDONLY with O_EXCL. An IN transfer direction alone therefore does not
establish an O_RDONLY/exclusive native access strategy. No permission elevation
or fallback open is implemented or approved by this model.

Linux's documented [SCSI error handling](https://www.kernel.org/doc/html/latest/scsi/scsi_eh.html)
can retry commands, issue REQUEST_SENSE and escalate through abort/device/bus/
host reset. A host API without a reset method does not establish that the kernel
preserves a volatile loader. Actual result extraction, driver transfer-count
reliability and kernel recovery behavior remain D2 qualification requirements.
`realized_data_len` is currently a supplied assertion, not kernel evidence.

## macOS IOUSBHost-shaped contract

`macos_usb::validate_macos_completion` requires exact before/after target and
request bindings, granted authorization and claim, and known checkpoint effects
for claim, capture, release, reset and re-enumeration. A claim must report
preservation; other effects must be unperformed or report preservation. Lost or
unknown effects refuse. A supplied `Preserved` value is an offline assertion;
it never qualifies a real operation or authorizes a reset.

The record contains exactly CBW OUT (31 bytes), command data IN (36 or 1–512
bytes) and CSW IN (13 bytes). Each stage binds endpoint, requested and transferred
length, actual bytes, status and timeout. The CBW must equal the original and
the CSW must have the exact tag, successful status and zero residue. Access
denial, busy/unsupported/unknown access, stall, timeout, disconnection, wrong
endpoint or any short/overlong stage refuse. No framework call occurs.

Apple's official documentation was consulted through Context7:

- [deviceCapture](https://developer.apple.com/documentation/iousbhost/iousbhostobjectinitoptions/devicecapture)
  terminates existing clients/drivers and has authorization/entitlement
  constraints. Default destroy behavior can reset and re-register the device.
- [reset](https://developer.apple.com/documentation/iousbhost/iousbhostdevice/reset())
  causes re-enumeration and invalidates the previous service connection.
- [deviceSurrender](https://developer.apple.com/documentation/iousbhost/iousbhostobjectdestroyoptions/devicesurrender)
  avoids reset/rematching only in its documented device-requesting-close case;
  it is not a general capture/release workaround.
- [IOUSBHostCompletionHandler](https://developer.apple.com/documentation/iousbhost/iousbhostcompletionhandler)
  supplies both status and transferred bytes; success status alone is
  insufficient to validate a read.

These are API constraints, not evidence that an FM-1 held loader survives an
actual macOS claim/release sequence. No root action, entitlement, driver detach,
capture, surrender, reset, clear-halt or native binding was added.

## Ordered two-read replay

`usb_read_replay::ReadReplay` takes one fictional selected target and derives
each next canonical request internally. It consumes exactly 4,097 completions:
tag 1 INQUIRY, then 2,048 FD05 blocks for each of two full reads, with increasing
tags and addresses starting anew for the second sweep. The second pass compares
every block with the first. The total reported duration is bounded to 600,000 ms;
each transaction is bounded to 5,000 ms. These are supplied replay durations,
not a measured wall-clock deadline.

An invalid transaction, [local artifact location omitted] drift, unknown fictional INQUIRY,
read disagreement or timeout permanently refuses subsequent acceptance and
report generation. Incomplete replay cannot issue a success report. A completed
replay refuses additional requests without destroying its historical report.
There is no cached-dump shortcut, automatic restart or fallback between OSes.
Constructing another replay is a new fictional sequence, never current authority.

The exact synthetic INQUIRY constant is a generated test fixture, not a trusted
real-loader fingerprint. The final hash/report is fictional byte agreement only;
all hardware, native access, source firmware, loader, physical durability,
independent-media, writer and full-stock-backup claims remain false.

The replay deliberately does not feed C's sealed capture API. Adding a bridge
from saved native-shaped records to a fake reader would misrepresent provenance.
D2 must integrate actual native session/read provenance into storage explicitly.

## Validation and artifact evidence

Validation uses the canonical guarded Python 3.12.15 and native runners. The
native runner now accepts `--test adapter_conformance` behind the existing
source/dependency/fixture audit, private scratch and source-fingerprint checks.
No dependency, audit allowance, FFI, hardware backend, secure file primitive or
existing USB capture/writer implementation changed. Source audit: **35 Rust files**.

- Guarded Python reference: **302 tests passed in 197.934 s**, no skips.
- Debug public-API adapter conformance: **15 passed in 0.20 s**.
- Optimized unit selections: **35 `usb_` tests in 1.52 s**, including six Linux
  tests; **13 macOS tests in 0.10 s**. The separate six-test Linux selection
  also passed in 0.08 s and is not double-counted. Optimized adapter conformance:
  **15 passed in 0.22 s**. Thus **63 distinct focused optimized tests** passed,
  including all 38 new tests and the retained 25 USB tests. The unchanged full
  writer matrix was not repeated in optimized mode.
- Full native debug regression: **182 passed — 147 unit and 35 integration**,
  no ignored/skipped/filtered tests; unit-suite time **500.66 s**. The canonical
  runner finished exit 0 with unchanged source/fixture fingerprints. These are
  local macOS results; unsupported-OS-only tests are not counted as exercised.
- Format check, offline locked release build and actual CLI checks passed.
  `status --json` exposes the three offline contract/replay capabilities with
  `native_usb_adapter_scope=fictional-result-validation-only`; both native OS
  adapter flags and all physical capabilities remain false. `backup assess`
  exits 0 with `state=blocked`; `backup capture` refuses with exit 2.

Independent replay image recipe:
`bytes((i*37+(i>>9)*11+(i>>16))&255 for i in range(1048576))`.
Python stdlib SHA-256:
`[digest omitted from public archive]`.
Both OS replays check this full hash and all requested addresses/blocks against
independent expectations. Fixtures contain original fictional data, no vendor
firmware. Fault coverage includes every modeled SG input field/status/sense/
residue, short success, every BOT stage, permissions/effects, private output,
stale sessions, order, elapsed bounds and differing full-read bytes.

The unsigned local macOS arm64 dispatch executable is **319,376 bytes**, SHA-256
`[digest omitted from public archive]`.
Linking can discard unused library code; this is not the size of a complete
backup/flasher release. The exact **51-file** native input map is saved under
ignored `[local artifact location omitted]`, SHA-256
`[digest omitted from public archive]`.
Two pre-existing dead-code warnings remain in transitions/writer.

Reproducible commands (Python 3.11+):

```sh
rtk proxy python3.12 host/run_tests.py
rtk proxy python3.12 native/run_tests.py
rtk proxy python3.12 native/run_tests.py --release --test lib --filter usb_
rtk proxy python3.12 native/run_tests.py --release --test lib --filter macos_usb
rtk proxy python3.12 native/run_tests.py --release --test adapter_conformance
rtk proxy cargo fmt --manifest-path native/Cargo.toml -- --check
rtk proxy cargo build --manifest-path native/Cargo.toml --release --offline --locked
```

The actual interpreter is
`[private host path omitted]`.
Local logs are under ignored `[local artifact location omitted]`. This is local
macOS-host validation of both pure models, not Linux execution, physical-device
qualification, current-source hosted CI or a signed release. No push occurred.

## Remaining ordered work

**D2 — native adapters, still gated:** select and review native binding/dependency
and access strategies one OS at a time; audit exact entry/loader effects; retain
actual handle/session and transfer/error provenance; qualify framework/kernel
capture/release/recovery behavior; integrate fresh native reads with C's storage
without a saved-record authority shortcut. Address approved backup destinations
and independent-media verification; the existing `/Volumes` path restriction
is unchanged. Windows remains a separate assessment.

**E — bounded read-only acceptance, still gated:** only after D2 evidence, use
the exact approved [local artifact location omitted] route and observe two fresh complete equal reads,
durable independent copies and normal return. Stop on uncertainty without
automatic retry/reset or writes. Record exact OS/transport, device/source/loader
identity, effects, capacity/layout and private hashes. Never infer write,
recovery, CPU-health or universal stock-version support from this result.
