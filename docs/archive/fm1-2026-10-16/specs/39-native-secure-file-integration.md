> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Native session integration with real protected two-copy files

Date: **2026-10-15**, Europe/Berlin. FM1-033-D2-R4-B on `init`, following
[the finite wire engine](38-native-backup-wire-engine.md).

## Outcome and limits

R4-B is implemented and validated offline. A private native-package coordinator
now joins the finite FD05/FD07 engine to **real protected regular-file storage**.
Tests use injected bulk I/O and actual temporary files. The actual nusb adapter
composition compiles on macOS arm64, but its public factory still refuses and
the private qualification permit remains uninhabited.

No device inventory, MIDI/USB open/query, ROM entry, loader execution, physical
read, flash write, firmware build/script, backend installation or network access
occurred. The owner's read-only permission remains present; R4-C's concrete
route/ROM/loader/chip/access evidence still precedes a physical operation. No
additional permission question or prior successful backup is inherently needed.

## Protected host-file primitive

`native/src/capture_files.rs` reuses the existing descriptor-relative
`local_files::Directory` implementation rather than copying it or loosening
its path protections. The narrow public interface is:

- `PreparedPair::prepare(primary, second)` opens and binds two distinct private
  directories and refuses snapshot/manifest collisions. It creates no files.
- `PreparedPair::persist(passes, provenance, cancellation)` consumes two owned
  complete equal 1 MiB buffers, canonical bounded JSON and a revocation flag.
  It creates both snapshots and storage-only manifests, fsyncs files/directories,
  publishes manifests without overwrite, reopens/hashes/compares bytes and
  rechecks all held file/directory identities.
- `StoredCopies::verify()` repeats protected verification on the same held
  host-file evidence. Its digest/provenance accessors disclose private artifact
  information deliberately; Debug remains opaque. There is no saved-data
  constructor for this handle or any current native result.

Snapshots use `capture-snapshot.bin`; published manifests use
`capture-storage.json`, with `capture-storage.pending.json` during publication.
The existing creation helper makes files owner-readable (`0400`) after writing;
directories must be private and owned. Ancestor replacement, symlinks, extra
hardlinks, nonregular files, collisions and identity/content drift refuse.

The facade accepts **untrusted bytes/provenance**, not native/source authority.
An input JSON claim is nested under `untrusted_provenance`; the outer schema
`untrusted-capture-storage-only` fixes current-session, hardware/source/loader,
physical durability, independent media, writer eligibility and auto-resume flags
to false. Equal inputs and fsync do not establish physical reads or media
independence. Two directories on one filesystem remain only two checked copies.
The existing restriction on `/Volumes` is unchanged; media/path qualification
for an actual independent macOS destination remains R4-C work.

The cancellation flag is checked before/after every wrapped protected operation,
including verification and the final identity sweep. A wrapped operation may
contain several file syscalls. An already blocked OS write/fsync cannot be
interrupted by the flag; revocation prevents its eventual result from becoming
a live accepted session. Partial files are retained without overwrite/retry.

## One wire owner and an independent file worker

`native-usb/src/storage.rs` takes a **preflighted `PreparedPair` before any wire
request**. The future activated route must preflight before mode entry. The
coordinator owns `BackupSession<BulkIo>` throughout; neither the worker nor the
file facade receives device handles, a queue, a command sender or the actor.

1. The actor obtains initial fixed FD07 metadata, the positive scratch reference
   and exactly 4,096 new coverage reads in two equal full passes.
2. A move-only handoff transfers both complete immutable buffers and its private
   attempt/sequence identity. The actor builds bounded provenance from its
   actual receipts: purposes/tags, relative host timing, returned-byte hashes,
   metadata, declared selected context/chip, counts and immutable budgets.
3. One ordinary worker thread receives the handoff, preflighted directories,
   provenance and revocation token. It performs real persistence/verification.
   The actor waits through a bounded channel and submits finite positive
   checkpoints between complete transactions. Checkpoints never fill coverage.
4. Only successful `StoredCopies` consumed with that exact handoff can create
   the production storage receipt. A foreign or late receipt refuses. Earlier
   engine-only modeled receipts contain no file proof and cannot produce a
   coordinator `CompletedCapture`.
5. After the durable receipt, the owner requires a new positive FD05 checkpoint
   and final FD07 static-pair validation. Final transcript computation consumes
   the remaining local and real time budgets. No backup-file operation follows
   these final observations on the successful path.

The native source variant contains an opaque `NativeCaptureSource` minted only
from held `NativeIo` selection/descriptor data. It records the cached selection
and exact candidate loader digests without claiming freshness, firmware
integrity, loader execution or hardware qualification. Injected provenance has
a separate test-only source variant and explicit `injected-wire-real-files-only`
scope. There is no caller JSON/boolean constructor for native source evidence.
R1's actual pre-entry capture/start binding remains an activation requirement;
post-start metadata cannot manufacture it.

## Final evidence and saved files

Both manifests are **storage-only records fully persisted before the final
checkpoint/FD07**. They contain the internally generated pre-storage transcript
summary and explicitly set `session_final_observations_persisted=false`.
The final observations and updated transcript exist only in the private
in-memory `CompletedCapture` alongside the sealed host-file handle.

This ordering avoids adding another blocking manifest write after the final
continuity check. File presence after a process crash, timeout, cancellation or
failed final read does not imply that the device session completed. A saved
manifest cannot recreate a live result, a writer permit or retry/resume authority.
Future durable export of the final session report needs its own bounded
finalization design; it is not silently supplied by these storage manifests.
Existing fictional USB schema 1/2, `UsbBackup` types and writer behavior are
unchanged. The default CLI gains no live backup/file command in this increment.

## Time and failure behavior

The actor retains R4-A's finite transaction/checkpoint/lease limits. The
coordinator additionally uses a real monotonic wall deadline so a frozen injected
clock cannot turn the polling loop into an infinite wait. Polling leaves room
for a complete bounded transaction; it never resets the total lifetime.
The additional wall-clock model conservatively retains the old hook interval
until a whole transaction closes. These are engineering bounds, not physical
loader-hook or IOKit timing qualification.

Storage/worker errors, panics that disconnect the worker channel, drift, bad
completion, cancellation, expired budgets and invalid ordering revoke the
attempt and request I/O cancellation once. There is no automatic reset,
reconnect, retry, rollback or return command. Replay after a completed run also
revokes that attempt; it cannot submit another pass. Core host-file verification
remains distinct from physical/current authority.

The wire owner never joins a potentially blocked file worker. Dropping the
thread handle detaches it; an in-flight file syscall may finish after the owner
has returned failure. The worker owns only guarded file state and immutable
buffers. Late file completion cannot issue USB commands or resurrect a revoked
session. Worker termination and OS fsync latency are not guaranteed.

The result owns held directory handles for later host-file verification.
Their eventual close/Drop, OS scheduling and native access/cleanup calls have
no qualified hard wall bound. The implemented finite polling/budget checks do
not turn those OS operations into hard real-time guarantees. Successful live
coordination performs no further snapshot/manifest IO after final observations;
later explicit host-file verification is a separate operation.

## Build and offline guard integration

`native-usb` has an exact local dependency on `fm1-flasher` through alias
`fm1-core`, path `../native`, with `default-features=false`. The default core
does not depend on the native package, so there is no cycle or new default CLI
hardware backend. Its MIDI implementation remains executable-only and disabled.
No registry dependency version/checksum, license notice, firmware, PIO or
submodule pin changed. The binding lock adds only the local core package/link;
the **34 registry pins are unchanged**.

The dependency direction is deliberately one-way. Future live CLI integration
needs an executable layer that depends on both packages, or a reviewed boundary
change; adding the native package as a dependency of this core would create a
cycle. The current default executable and qualified identity-only entry point
remain intact. A future composed Rust executable can still be distributed as
one binary; no binary-size or release result is measured in this increment.

The binding runner now recursively audits and fingerprints all core source,
manifest/lock/policy/runner and fixture inputs as well as its own sources. Its
exact local lock graph/manifest, private modules, empty native permit and constant
factory refusal remain enforced. The trusted Python bootstrap creates private
temporary destination slots and sets `FM1_NATIVE_TEST_ROOT` before executing
tests. This matters because core compiled as a dependency has `cfg(test)=false`;
without that environment, its existing production path guard would not confine
it to the test scratch. Rust tests gain no public file-mutator or backend hook.
This source regression guard is not a sandbox for malicious reviewed code.

`--check-offline` additionally compiles the ordinary non-test library without
native features. It covers the uninhabited production source type that test cfg
and native-feature checking alone cannot cover. Independent review found that
empty-reference exhaustiveness issue, which was fixed and compiled in all three
configurations. No native-feature test/executable was run.

## Validation

GPT 6.1 Sol **high** subagents implemented and independently reviewed the core
storage and native integration; a **medium** subagent extended guard regressions.
Root integrated dependency/build/guard/native source seams and validation.

- **46 native-package tests passed in debug and optimized builds**: 38 prior
  wire/queue regressions and 8 real-file coordination tests. They cover the full
  stored stream, private source/receipt distinction, blocked worker with active
  checkpoints, cancellation/count exhaustion, frozen-clock wall expiry, worker
  error/panic, late collision, post-real-receipt final checkpoint/metadata drift,
  and real foreign/expired storage receipts.
- **11 new core storage tests passed in debug and optimized builds**, including
  30 injected persistence cuts, cancellation after all 28 wrapped operations,
  disk-full/permission/sync error paths, actual helper short-write behavior,
  immutable-byte/hash checks, late replacement, symlinks and hardlinks. The
  additional FIFO branch is Linux-only and was not executed on this Mac.
- The native worker gate pauses **before real persistence**, demonstrating
  independent owner checkpoints; it does not stall an actual OS fsync. Core
  fault injection exercises file-sync errors separately. HostCrash cuts are
  returned simulated faults, not killed processes or physical power failures.
- **26 file-only guard tests passed in 45.471 seconds**, including recursive
  dependency fingerprints, forbidden core APIs, exact path/default-feature
  overrides, storage privacy and retained native factory/permit mutations.
- **314 canonical guarded host tests passed in 188.223 seconds**, no skips.
  Audits passed for **48 core and 9 binding Rust files**, with unchanged registry
  pins. Formatting and diff/documentation checks passed.
- Ordinary default-library checking and actual **macOS arm64 native-feature
  release checking passed**, using Rust 1.99.0. Four existing core deprecation/
  dead-code warnings remain; unrelated code was preserved. Core writer/model
  sweeps, Linux builds and physical acceptance were not repeated here.

An independent Python `hashlib` oracle for all 1,048,576 synthetic bytes matched
the stored stream SHA-256:
`[digest omitted from public archive]`.
Logs, reviewed input maps and independent review notes remain ignored under
`native/target/` and `native-usb/target/`.

The **81-input** `[local artifact location omitted]` includes the complete
binding and recursive core fingerprint. Its canonical sorted compact JSON
SHA-256 is `[digest omitted from public archive]`.
Final binding debug log SHA-256 is
`[digest omitted from public archive]`;
optimized log SHA-256 is
`[digest omitted from public archive]`;
host log SHA-256 is
`[digest omitted from public archive]`.

## Next: R4-C before the first physical backup

The remaining work is operation-specific qualification and activation, as
ordered in [report 38](38-native-backup-wire-engine.md): exact V15 `7D` entry
handler/callee/framing; ROM FD07 effects and ownership of the exact A/B capture
intervals; selected macOS object/access and a documented return; then actual
R1 capture/stable handoff, loader external startup/callback/chip/fresh-data
semantics, physical timing/cancellation and independent destination evidence.
Only the needed stage must qualify before its operation. A bounded ROM-only
capture can precede loader jump/full reads once its own requirements are met.

This increment closes R4-B's real-file/native provenance implementation, not
those physical evidence gaps. The factory stays closed pending a separately
reviewed activation change. No new owner-device observation, full hardware
backup, remote push or hosted build result is claimed. Existing rescue-source
documentation work remains outside this increment.
