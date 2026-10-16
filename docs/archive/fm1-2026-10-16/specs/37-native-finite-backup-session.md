> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Finite concrete-fake backup session and modeled storage

Date: **2026-10-15**, Europe/Berlin. FM1-033-D2-R3 on `init`, following
[R2](36-native-fixed-metadata-validation.md) and the sequence in
[report 33](33-loader-no-fc0a-read-session.md). GPT 6.1 Sol high subagents
implemented the actor, independently developed adversarial tests and reviewed
the contract. Root integrated discovery, runner selection and evidence.

**R3's initial 1 MiB/mode-2 model is implemented offline.** One concrete-fake
actor obtains two complete passes, keeps checkpoint/metadata purposes separate,
and schedules finite checkpoints around cooperative modeled storage. The model
does not access hardware or files and cannot create an accepted physical backup.
The owner's read-only hardware permission remains present; R4's route, timing,
native provenance and stock-return evidence still precede physical activation.
No network, hardware inventory/access, flash operation, target execution,
firmware build/script, backend installation or dependency change occurred here.

## Separate seam and actual work

`native/src/backup_session.rs` provides `SessionPlan`, `SessionBudgets`,
`FakeEndpoint`, `ModeledStorageWorker` and `SessionActor`. These are concrete
types with private state, not a generic device/filesystem interface. The actor
owns the endpoint, worker, original R1 context and all capture buffers. Public
methods permit bounded fake configuration, controlled fault injection, stepping,
cancellation and sanitized reporting. There is no saved-result import/resume,
native permit, writer conversion, reset, reconnect, retry or stock-return path.

The plan validates its private device/session/start labels and original expected
R1 fingerprint. It constructs `ExpectedChip` using the declared full JEDEC ID
and **exact 1 MiB** capacity; other capacities/densities remain unsupported by
this initial actor. Its metadata expectations use those same labels, original
context and chip declaration, avoiding an unrelated successful R2 assessment.

The endpoint accepts only an owned 1 MiB fictional byte array and two fixed
84-byte metadata inputs. Each actor-generated transaction obtains a new reply
from the endpoint. Previously completed pass buffers or saved schedules cannot
replace a read. First/final metadata are responses to distinct fixed typed
submissions; the actor constructs their R2 envelopes/expectations internally.
These inputs and receipts establish **modeled ordering only**, not physical
freshness, ROM preservation or device identity.

Legacy `usb_read_driver`/`usb_read_replay` retain their exact 4,097-request
contract. USB backup schemas, sealed `UsbBackup` authority, secure regular-file
storage, writers and actual native factories remain unchanged. R3 cannot turn
modeled copies into any of those durable/current handles.

## Purposes, coverage and order

For C=1,048,576 bytes and 512-byte blocks, N=2,048. A successful run contains:

1. One fixed initial metadata transaction for FD07(0x01C09700,84), assessed by
   R2 against the original supported R1 capture.
2. One positive FD05(0,512) initial checkpoint into separate scratch memory.
3. Exactly N new full-pass-1 submissions at `0,512,...,C-512`.
4. Exactly N new full-pass-2 submissions over the same ranges. Separate owned
   vectors must reach C bytes each and compare equal before storage begins.
5. Eight ordered modeled storage phases, with finite storage checkpoints when
   required by the predeclared timing budget.
6. One new final positive checkpoint **after** all modeled storage receipts,
   compared with the initial scratch reference.
7. One new final fixed metadata observation after that checkpoint. R2's stable
   pair must pass before modeled acceptance.

Raw reads are exactly **`2N+h`**, with **`2 <= h <= H`**. Metadata requests are
separate and exactly two; total transactions are **`2N+h+2`**. Checkpoints never
fill pass coverage, and metadata never counts as raw flash bytes. There is no
INQUIRY heartbeat or FC0A. The fast default model has **4,100 transactions**:
4,096 coverage reads, two checkpoints and two metadata transactions.

Each submission binds a private process-local actor lifetime, sequence, tag,
queue/session token and purpose. Callback linkage, exact CBW completion count,
realized data length and shared strict CSW tag/signature/status/residue validation
must pass. Private 16-byte command shapes permit only modeled FD05 block/
checkpoint and fixed FD07 purposes; no actual adapter allowlist is expanded.
The model retains ordered receipts and private per-transaction byte hashes;
public receipts omit private device/session/start/lifetime identifiers, bytes
and metadata hashes. Submission attempts
and validated completion receipts have separate report counts.

## Modeled time and finite budgets

All time is explicitly **fictional ticks**, without conversion to IRQs,
milliseconds or physical host deadlines. Defaults are H=64, transaction budget
4,162, strict hook gap less than 10 ticks, transaction completion at most 4 ticks
and total wall budget 100,000 ticks. The maximum accepted checkpoint budget is
4,096; transaction budget is bounded to 8,194; hook-gap/wall values are bounded
to 10,000,000 ticks. Completion must be positive and less than the hook gap.
Worker delay per phase is bounded to 50,000 cooperative polls.

Positive transaction budgets below the successful minimum are deliberately
accepted as finite failure-model inputs; the actor refuses when exhausted.
They are not viable hardware plans. No budget grows after the attempt starts.

The model refreshes its hook at request start and includes completion plus
subsequent scheduling/storage slices in the gap calculation. The actor alone
serializes complete transactions. Cooperative worker polls consume modeled
slices; when the next slice would exhaust the hook gap, a distinct checkpoint
is completed first. This is neither an actual background thread nor proof that
real filesystem or OS scheduling can meet those limits. R4 must establish that
separately before any native timing or continuity claim.

## Storage verification and terminal failures

Storage is an in-memory model with actual owned bytes and these ordered phases:

`WriteOriginal -> SyncOriginal -> ReopenOriginal -> HashOriginal -> CopySecond
-> SyncSecond -> ReopenSecond -> HashSecond`.

The worker copies bytes, models sync/reopen transitions, and compares the
reopened hashes against the immutable completed pass. It does not accept a
caller-provided verified/durable boolean. Corruption changes the appropriate
stored/reopened bytes; ordinary `HashOriginal` or `HashSecond` then detects it.
The fault injector does not return the expected hash verdict itself. Early
corruption therefore advances through write/sync/reopen receipts before failing
the normal verifier. Modeled sync/reopen still proves no filesystem durability,
independent media, real file identity or power-loss behavior.

Any byte/state/identity/queue/callback drift, malformed completion, timeout,
rollback/overflow, checkpoint/count/wall/gap exhaustion, cancellation, panic or
storage failure permanently poisons the actor. Subsequent run/step calls issue
no further submissions. Partial buffers/receipts remain in the failed model;
there are no actual files to preserve or remove. Cancellation is between
atomic synchronous model steps, not preemption of a blocking native operation.
No automatic cleanup/reset/re-entry, retry, finish or saved resume is modeled.

## Reports and authority

Reports distinguish coverage/checkpoint/metadata counts, submissions/completions,
both pass lengths, storage receipts, fictional ticks and modeled acceptance.
A full-data SHA-256 appears only when both complete pass buffers compare equal,
consistent with existing snapshot integrity reports. Raw bytes, private labels,
pointers, metadata/provenance hashes and request lifetimes are redacted. A
matching complete-data hash cannot prove physical freshness.

Positive results retain false hardware/native/entry/read/write/permit/proof/
freshness, physical durability, independent media, current-writer, saved-resume
and real-finish flags. `modeled_copies_verified` is never a durable-file claim.
Permanent blockers retain fictional transactions, modeled storage, unqualified
hook timing, ROM/entry/chip/return and native-storage provenance.
`status` advertises `fake_checkpoint_storage_orchestration_available=true` with
scope `concrete-fake-modeled-storage-only`. CLI file commands and live backup
remain unavailable.

## Independent validation

The new conformance target uses original synthetic 1 MiB bytes with a literal
independent stdlib SHA-256 expectation, retained independent R1 fixtures, and
synthetic metadata. It checks exact ranges/pass purposes/tags/counts, separate
scratch, fast/slow storage ordering, post-storage continuity and private reports.
Matrices cover every modeled storage phase and protocol/data/metadata failures,
completion boundaries, count/time exhaustion, cancellation and permanent poison.
No product hash/codec helper computes the expected full-data digest.

Reproduce focused checks through the canonical guards:

```sh
rtk proxy python3.12 native/run_tests.py --test backup_session_conformance
rtk proxy python3.12 native/run_tests.py --release --test backup_session_conformance
rtk proxy python3.12 native/run_tests.py --test loader_metadata_conformance
rtk proxy python3.12 host/run_tests.py
rtk proxy cargo fmt --manifest-path native/Cargo.toml -- --check
```

- **9 new R3 tests passed in debug and optimized builds**, including 40 storage
  boundary/fault combinations and protocol, data, metadata, cancellation and
  budget matrices. Corruption cases verify the ordinary hash failure and the
  preceding reopen receipts; terminal failures cannot issue another submission.
- All **ten native integration targets passed: 64 tests**, unfiltered. Three
  selected library tests for status, reference-contract discovery and operation
  refusal passed, each intentionally filtering the other 164 library tests.
- Focused optimized suites passed **22 tests: 9 R3, 7 R2 and 6 R1**. Native runs
  used offline locked Cargo, source/dependency audit, guarded scratch and stable
  per-run fingerprints. The unchanged full writer/persistence matrix was not
  repeated; the prior full result in report 35 remains historical evidence.
- The complete canonical guarded Python suite passed **314 tests in 189.937 s**,
  no skips. Source/API/dependency audit passed for **46 Rust files**, without new
  dependencies or policy exceptions. Formatting, relative documentation links
  and diff checks passed. The default offline release build and actual default
  CLI status/help/capture-refusal checks retained all false physical flags.
- Independent high-effort source/contract/test review found no remaining
  actionable defects; a medium-effort documentation review corrected successful-
  attempt count and public receipt wording. No optional backend, additional
  [local artifact location omitted] or hosted CI execution is claimed.

Ignored evidence under `native/target/` includes focused/regression/host logs,
default build/status/refusal receipts and independent review notes. The **69-file**
`fm1-r3-source-map.json` covers audited native inputs and both retained host
fixture oracles/tests. Its canonical sorted compact JSON SHA-256 is
`[digest omitted from public archive]`.
Native regression log SHA-256 is
`[digest omitted from public archive]`;
host log SHA-256 is
`[digest omitted from public archive]`.
Work is local on `init`; no push, physical backup or hardware qualification is
claimed. Existing rescue-source documentation WIP remains outside this increment.

## Next step: R4 evidence and integration

R4 remains evidence-gated. It must establish actual ROM/pre-entry configuration
stability, exact chip/read/addressing and fresh-data behavior, macOS selected-
object access, real native-to-secure-storage provenance, bounded timing and
cancellation/cleanup, and original-stock return. It must integrate actual durable
files and final continuity without converting these model receipts into proof.
The owner's read-only authorization is present; unresolved technical evidence,
rather than another permission question, still blocks a real backup attempt.
This increment has not read the owner's RAM/flash or created a hardware backup.
