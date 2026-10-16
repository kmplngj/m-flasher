> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Finite native backup wire engine and staged evidence gates

Date: **2026-10-15**, Europe/Berlin. FM1-033-D2-R4-A on `init`, following
[R3](37-native-finite-backup-session.md) and the conditional
[no-status profile](34-no-status-backup-profile-decision.md).

**Follow-up:** [R4-B/report 39](39-native-secure-file-integration.md) now adds
the real protected-file worker and storage-receipt constructor. The closed
production receipt described below records R4-A's original delivery state;
the public hardware factory remains closed after R4-B as well.

## Delivered scope

R4-A implements a private finite BOT engine in `native-usb/src/backup.rs`.
It serializes the planned fixed FD07/positive FD05 sequence against the same
private `BulkIo` interface as the actual selected-device adapter. A private
unreachable `BackupSession<NativeIo>` bridge checks that composition when
compiling the native feature. This is wire implementation with injected tests,
not another simulated durability claim or hardware qualification.

The public factory still refuses before any OS operation; its private permit
remains uninhabited. No device inventory, MIDI/USB open, entry, loader execution,
physical read, flash write or backend installation occurred in this increment.
The owner's read-only permission remains present. Technical evidence, rather
than a renewed permission question, precedes any physical operation.

## Immutable command and state boundary

The new engine accepts no raw command, address, pass number or response verdict
from a caller. It constructs only these 16-byte CDBs, padded with `FF`:

| Purpose | Shape | Successful-attempt count |
|---|---|---|
| Initial/final metadata | FD07, address `0x01C09700`, length 84 | 2 |
| New full-pass coverage | FD05, internal addresses `0,512,...,C-512`, length 512 | `2N=4096` |
| Initial/optional/final checkpoints | FD05, address 0, length 512 | `2 <= h <= H` |

The selected subset fixes `C=1,048,576` and `N=2048`. A successful attempt has
`2N+h+2` complete BOT transactions: 4,100 with only the two required
checkpoints. Each transaction closes before another begins. Checkpoints cannot
contribute coverage, replace a second-pass request or overwrite captured data.
The first block of each pass must match the immutable initial reference; all
second-pass blocks must match their separately read first-pass blocks.

CBWs use strictly checked nonzero monotonically increasing tags, IN direction,
exact lengths and fixed command shapes. Every completion must have successful
native status, exact actual and initialized input lengths, and a strict CSW
signature/tag/zero-residue/success result. There are no FC0A, FC40, FC44,
INQUIRY heartbeat, generic memory requests, retries, resets or return commands.
The legacy INQUIRY plus 4,096-read engine remains unchanged.

`ExpectedMetadata` accepts only the original R1 A/B selected-context locations
and a declared complete 24-bit JEDEC ID with density 20 for this 1 MiB subset.
Both exact 84-byte observations check R2's fixed state fields: open device and
ROM callbacks, no uploaded callback/transform/addressing/idle suppression,
mode 2, cached chip/capacity, configuration `0x00010300`, read word
`0x00020604` and inactive continuation state. The established static fields
must also match between observations; mutable counters, online state and opaque
bytes are excluded. These checks do not prove the physical chip or retroactively
qualify startup. The separate R1 pre-entry capture/start binding is still needed
before activation; no saved JSON or caller boolean supplies physical authority.

## Finite time, storage and failure

Immutable local budgets cap the lease at 600 seconds, individual transactions
at 5 seconds and checkpoints at 4,096, including a reserved final checkpoint.
They require positive bounds, at least two checkpoints and a transaction bound
strictly smaller than the hook-gap bound. Checked arithmetic and monotonic
`Instant` observations reject overflow, clock rollback, zero remaining time,
lease expiry and exhausted budgets. Waiting between operations and storage
handoff time consume the same original lifetime; no call extends it.

The old modeled hook interval remains in force until a new CBW has completed
and its consistency check passes. The next interval is charged from that CBW's
host submission. These timestamps are an **unqualified engineering model**,
not observations of the device's actual idle-counter store. A local deadline
does not establish physical hook-to-hook timing, OS teardown bounds or liveness.

After both full passes compare equal, a move-only `StorageHandoff` takes their
owned immutable buffers and a private attempt/sequence identity. The actor
retains separate scratch and may issue only budgeted checkpoints while storage
is pending. A same-attempt, one-use `StorageReceipt` would unlock the mandatory
post-storage checkpoint and final metadata pair. **Production has no receipt
constructor.** Only dependency-free injected tests have a modeled constructor;
it performs no filesystem operation. It cannot create a native open permit.

Real secure-file fsync/reopen/hash/copy integration remains R4-B. This engine
does not relabel actual handles or modeled receipts as schema-1/schema-2
fictional backups, physical durability, independent media or writer authority.
The handoff/receipt identity supports transfer to a future storage worker;
no thread or worker is launched here. Private receipts retain bounded returned
bytes and host timing; Debug omits payloads, metadata, identifiers and timing.

Every `Result` error permanently poisons the actor and requests adapter
cancellation once. Later calls cannot submit another transaction. The private
trusted `BulkIo` implementation must return errors rather than unwind; no
general caller-supplied adapter or panic-to-success path exists. Cancellation
is host cleanup and is not a qualified device-return operation.

## Operation-specific gates toward the first backup

The independent local route audit separates stages so later acceptance does
not become a circular prerequisite for the first observation. The owner's
permission does not need renewal for the bounded reads once their concrete
requirements are established.

1. **Finish the offline native storage bridge (R4-B).** Use owned immutable
   buffers, real protected regular-file operations, fsync/reopen/hash/copy
   receipts and one serialized finite session owner during slow storage.
   Bind the native attempt and final continuity to those receipts. Do not
   convert R3 modeled storage or historical saved data into current proof.
2. **Before entering ROM or a bounded ROM-only RAM capture (R4-C):** evidence
   the exact supported V15 `7D` handler/callee and computer framing, the ROM
   FD07 transform/side effects and ownership of the exact A/B intervals, the
   selected macOS access object and a documented way back from ROM. Then a
   bounded pre-entry configuration capture can stop **before FB08** and be
   assessed by R1. Do not probe adjacent SysEx or guess ROM command effects.
3. **Before loader upload/jump:** bind the freshly checked exact image, original
   captured A/B bytes and stable handoff; qualify external startup/callback
   effects, exact chip/read semantics and the selected no-status subset.
   Post-start FD07 metadata cannot cure unsafe pre-start conditions.
4. **Before accepting the full backup:** obtain two fresh equal complete reads,
   actual bounded timing/cancellation/continuity, two reopened verified files
   with independent-copy/media evidence, and final observations after durable
   storage. The first qualified backup can supply these observations; an older
   successful backup or write/restore tests are not prerequisites.
5. **Return:** choose an evidenced no-flash-write lifecycle before entering a
   mode that needs it, then observe original-stock operation afterward. FC0C
   requests MMIO reset; command completion does not demonstrate stock return.
   No automatic reset, reopen, retry or power-cycle follows a failure.

macOS-only qualification does not wait for Linux/Windows qualification. Neither
write/restore testing on the owner's unit nor qualification of unreachable
arbitrary write APIs is required for a narrowly evidenced read operation.

## Retained-source audit findings

The exact actual macOS dependency is nusb **0.2.8's `macos_iokit` backend**,
using `IOUSBDeviceInterface500`/`IOUSBInterfaceInterface500`. The earlier
IOUSBHost-shaped contracts are result models, not that backend. Its selected
registry-ID path iterates services to resolve the exact ID, then attempts ordinary
`USBDeviceOpen`, tolerating failure of that device-exclusive attempt. Interface
claim uses ordinary exclusive `USBInterfaceOpen` and refuses busy access. Our
binding requests no seize, detach, reset, configuration/alternate change or device
ClearPipeStall/CLEAR_FEATURE. AbortPipe cancellation separately clears the
controller's halted bit without resetting the data toggle; see [report 41](41-rom-fd07-macos-entry-return.md).
Accessor
checks are cached consistency, not a running-loader attestation. Open/claim
waits, IOKit completion, AbortPipe cancellation and close/Drop wall bounds still
need scoped qualification; unused API effects need not be invented as gates.

Retained Transporter source sends logical `F0 22 24 35 7D F7`; its V15 guard
checks VID/PID only. Retained V13 assembly annotates its matching `7D` call
as `0xFFC02532`. **Correction, 2026-10-16:** this file-relative value resolves
to application RAM with the correct VMA; it is not an absolute ROM call.
[Report 40](40-v15-entry-rom-backup-audit.md) now binds the corresponding
manufacturer V15 handler to a RAM mailbox/P33-reset callee at `0x01C026A4`.
The installed image, direct-computer framing and reset's ROM aftermath remain
separate qualification questions.
Related `7F` uses an OTA-task path and is excluded. ROM documentation warns
that FD07 can encrypt source RAM in place before restoring it; IN direction
therefore does not prove a harmless RAM read. Exact WL82 ROM interval ownership
and transform effects remain unresolved in the retained material.

Independent inert decoding again matched the complete 24,064-byte image:
encoded SHA-256
`[digest omitted from public archive]`,
decoded SHA-256
`[digest omitted from public archive]`,
512-byte transform witness SHA-256
`[digest omitted from public archive]`.
The already encoded upstream blob is sent in 512-byte FB06 blocks; do not
apply a second host transform. R1's configuration transform is distinct from
the ROM wire transform and resets at its own specified boundaries.

The local mode-2 and FD05 selector-6 subgraphs already provide substantial
conditional no-status evidence; do not describe the entire loader as opaque.
External startup `0x01E00428`, ROM callbacks, exact chip/fresh-data behavior and
physical timing remain scoped unknowns. See [startup 32](32-loader-no-status-start-profile.md),
[read session 33](33-loader-no-fc0a-read-session.md) and the ignored detailed
`[local artifact location omitted]` for exact source lines, byte ranges and
input hashes. This increment used retained files only; no online refresh.

## Validation and delivery

GPT 6.1 Sol **high** subagents implemented and independently reviewed the wire
engine/source evidence; a **medium** subagent extended file-only guard
regressions. Root integrated the actual adapter compile seam, guards,
documentation and validation. No dependency, firmware, PIO, submodule pin or
license notice changed.

- **38 injected Rust tests passed in both debug and optimized builds:** the
  original 26 regressions and 12 new backup tests. Coverage includes the exact
  4,100-transaction/12,300-completion fast schedule, both 1 MiB byte streams,
  phase/order/capacity/metadata checks, status/length/CSW/provider failures,
  checkpoint drift and reserved budget, foreign storage receipts, clock
  rollback, immutable lifetime, cancellation and redaction. A distinct old-hook
  boundary test fails a 2-second OUT after 9 seconds of a 10-second gap despite
  remaining below the 5-second transaction bound; its adjacent 500 ms case
  succeeds. Neither test advances the physical device.
- **18 file-only guard tests passed**, including both new files' fingerprints,
  native-import refusal, private/test module gates, extra open/claim rejection,
  constant factory refusal and empty-permit mutations. This is a regression
  guard, not a malicious-source sandbox.
- **Actual macOS arm64 native-feature release checking passed** with the real
  nusb adapter and private engine composition. No native-feature binary or test
  was executed. The native source/dependency audit covers **7 Rust files and
  34 pinned registry dependencies**. Prior Linux arm64 evidence in report 28
  predates this increment; Linux/other targets were not rebuilt here.
- **314 guarded host tests passed in 178.563 seconds**, no skips. Core
  source/API/dependency audit passed for **46 Rust files**; the core Rust
  implementation is unchanged and its lengthy writer/persistence suite was
  not repeated. Rust formatting and documentation/diff checks passed.

Logs, exact source maps and independent review notes remain ignored under
`native/target/` and `native-usb/target/`. No physical backup, hardware
qualification, remote push or hosted execution is claimed. Work stays local on
`init`; existing rescue-source documentation work is outside this increment.

The **13-input** `[local artifact location omitted]` contains SHA-256
for all binding sources, manifest/lock/policy, runner/guard tests and the shared
core audit. Its canonical sorted compact JSON SHA-256 is
`[digest omitted from public archive]`.
Debug log SHA-256 is
`[digest omitted from public archive]`;
optimized log SHA-256 is
`[digest omitted from public archive]`;
host log SHA-256 is
`[digest omitted from public archive]`.
