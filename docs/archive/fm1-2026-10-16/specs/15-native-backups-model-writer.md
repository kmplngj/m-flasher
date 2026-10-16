> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Native backups and bounded model writer evidence

Date: 2026-10-14, Europe/Berlin. Ticket: [FM1-030](04-backlog.md#fm1-030--rust-local-backups-and-bounded-fake-writer).
All source/device/boot outcomes below are fictional or offline. No hardware
inventory, probe, native adapter, firmware build or physical flash write occurred.
The owner's only FM-1 remains outside development and fault tests.

## Reviewed storage design

The macOS/Linux library uses pinned safe Rustix descriptor APIs. Every absolute
path is checked lexically before filesystem calls; device/inventory roots,
traversal and unsupported spellings refuse. Traversal opens each directory with
`DIRECTORY|NOFOLLOW|NONBLOCK|CLOEXEC`, compares before/opened/after identities and
retains an anchored descriptor. Ancestors must be root/user-owned without unsafe
write permissions, with only the reviewed root-owned sticky-directory exception.
Final directories must be user-owned and private. Revalidation reopens the entire
ancestry; replacing a parent with a same-content tree cannot reuse a handle.

Files must be bounded private regular files, user-owned and single-linked.
Before/opened/after/named identities include device/inode, owner, permissions,
size and change/modify timestamps. Creation is exclusive, private and immutable
after writing. Short writes refuse. Actual file and directory fsync, full reopen
and bytes/hash/identity comparison precede acceptance. Exclusive hardlink
publication refuses overwrite; pending/interrupted artifacts remain unaccepted.
Both snapshot/manifest identities and directories are rechecked at the end, so
checking the second copy cannot make a stale first copy acceptable.

`capture_backup` requires two fresh equal sealed full reads from one current fake
session. It independently qualifies original source APP/layout/device/catalog
inputs rather than trusting the supplied target plan or backup/recovery JSON.
The reader's bridge chip INFO is distinct from the original source firmware INFO.
Snapshots, paths and [local artifact location omitted] identifiers stay private; reports expose hashes.

`BackupLocator` contains private paths and an expected metadata digest. It is an
input locator, never proof. `verify_saved_backup` requires original source inputs
and reopens both actual complete copies, checks canonical exact metadata and
source/session/catalog/path/snapshot bindings, performs fresh fsyncs and pins the
current identities. A private origin enum makes reopened handles historical-only.
They cannot satisfy current writer gates even when session hashes match.
Historical APP rollback accepts an explicitly verified old handle only alongside
a separate fresh current capture. Pending/missing/corrupt/swapped files refuse;
there is no automatic recapture, overwrite or resume after a failed publication.

Fake session counters are process-local. A digest collision after restart can
conservatively refuse a rollback; no process-global uniqueness is claimed.
Native lifecycle work must preserve closed fresh-current provenance and specify
fresh process/session identity separately. Saved report/locator data cannot waive
those gates. File fsync completion is not physical power-loss durability or proof
that the two locations are independent media; both report flags remain false.

Windows and other unreviewed persistence platforms immediately refuse. They have
no fallback to ordinary filesystem opens. Linux design is compiled/configured but
requires current-source hosted execution before any platform result is claimed.

## Native offline test boundary

The canonical development command is `python3.12 native/run_tests.py` (Python
3.11+); `--release` executes the optimized suite. Before Cargo build, the audit
rejects hardware/backend, network, process-launch, arbitrary FFI/unsafe APIs,
unreviewed dependencies/checksums, build/custom/implicit target overrides and
computed/escaped/out-of-repository fixture includes. Reviewed syscall/file APIs
are confined to `local_files.rs`; owner identity is the only process syscall.
The project retains `forbid(unsafe_code)`. Pinned Rustix internally implements
safe filesystem bindings; those trusted dependencies/compilers are outside the
project's test regression boundary.

Tests receive one private scratch tree. The runtime path check refuses protected
paths and paths outside that tree before local IO. Test setup/fault helpers use
only checked scratch descendants. A local FIFO is created by the trusted
bootstrap and never opened; storage must reject its type before open. The runner
hashes source, manifests, itself and fixtures before build and after execution;
source drift refuses certification. Small independent targets execute before
the long writer sweep; `--test NAME` provides an audited focused diagnostic run.
Native test subprocesses are launched by
this separate bootstrap, never inside or by relaxing the canonical Python guard.

This is a source/API regression guard, **not an OS sandbox for arbitrary malicious
Rust or a physical-device guarantee**. Review and the closed production IO surface
remain necessary. Direct Cargo tests lack the required IO scratch boundary.
CI audits before fetching exact locked packages, then builds/tests offline.

Independent review reproduced audit gaps in character-literal handling, process
aliases, custom targets/source overrides and computed/escaped fixture includes.
They are corrected and retained in pure guarded Python audit tests. Review also
found metadata counter overflow, private Debug output, rollback preflight reader
poisoning and callback panic handling; corrected tests retain those regressions.

## Concrete fake/model boundary

The persistent NOR model implements erase-before-program, 1→0 checks, page/sector
geometry, finite byte-prefix tears, dead cells and separate read-channel corruption.
Cold classification never assumes ROM reachability from a damaged APP. Persistent
bytes survive modeled power loss; volatile loader/session state is cleared. Stub
rescue and external entry need exact explicitly fictional evidence and remain
separate from ordinary writer success. No firmware CPU is executed.

The reader accepts one saved explicit descriptor and one exact in-memory endpoint.
It validates INFO and finite DATA/CRC/END frames, fragments, bounds, deadlines,
fresh dump sequences and [local artifact location omitted] stability. Ambiguity
sends zero fake commands; framing/clock/permission/reconnect failure poisons the
handle. It has no native factory, inventory or generic adapter interface.

The writer consumes immutable recomputed plans, original bytes, a fresh captured
backup, an exact fake reader/target and a separately revocable simulation authority.
Callbacks cannot issue address/byte/device authority. After callbacks and before
every erase/page/retry, independent APP bounds, identity/session, plan/reference,
actual reopened copies and authority are checked. Initial snapshot mismatch causes
zero modeled writes. Sector verification and full expected-image readback are
required; retry is limited to one failed sector and never touches an unrelated or
protected region. Failures/panics poison execution; writers are single-use.

The explicit rollback writer additionally rechecks old source/known APP/original
package and actual historical copies at every boundary. Current head/data/tail
remain preserved. Reopened current handles refuse and poison the reader despite
identical hashes/session, with zero target mutation. Reopened historical handles
can support a separately fresh current rollback; late old-copy loss blocks the
next erase/page. No full-chip/head restore, automatic boot or resume exists.

The journal is a strict finite immutable predecessor/hash chain. Pending files,
gaps, changed job/plan/backup bindings, terminal transitions and authority/resume
claims refuse. It records diagnostic stages only, never a proven device stage.
Saved dump diagnostics expose advisory region/sector matches and scoped saved
observations; an APP match proves neither source identity nor bootability.

## Independent fixtures and validation

The original compact corpus uses ramp/fill recipes and patches, never owner/vendor
bytes. Its independent array/stdlib hash/CRC oracle imports no production modules.
Seven guarded Python tests check regeneration and secondary reference comparisons.

| Category | Saved rows |
|---|---:|
| NOR acceptance/refusal and tears | 16 |
| Physical-prefix/transport trace expectations | 15 |
| Writer success/no-op/boundary/tear expectations | 712 |
| Framed fake reader | 8 |
| Advisory saved dumps | 10 |

`model-v1.json`: **563,704 bytes**, SHA256
`[digest omitted from public archive]`.
The manifest pins counts and bytes. Rust conformance compares independent whole,
head, APP and tail hashes; CRC/frame coverage/read numbers/command counts; and
actual writer operation counts, failure states and protection. Optimized release
executes all **712** writer rows using actual private backup files; the debug
matrix executes **142** selected rows across every fault kind and operation class.
The separate unit sweep covers all **76** nominal before/after power-loss points.
Test-profile optimization level 1 retains debug assertions and overflow checks;
release size optimization also keeps overflow checks. No integrity gate is skipped.

Trace conformance checks saved prefix bytes/cold classification; there is no public
native generic fault-trace dispatcher. The ten incident-shaped advisory dump rows
have no independently pinned matching reference packages, so their package-match
fields are not claimed as native parity. A separate test checks independently
pinned source/target packages against original, mixed-one-sector and two-sector
recipes: exact per-sector results and totals **143/141, 142/142 and 141/143**,
while identity/mode/rollback/write authority remain unknown/false.

Storage tests retain **90** persistence cuts, actual short-write/error seams,
copy loss/replacement, hardlink/symlink/FIFO refusal, ancestry/open races, strict
metadata, saved-handle provenance and finite journal cuts/corruption. Writer tests
retain callback authority/identity/copy drift, panic poisoning, finite retry/dead
cell/read corruption, protected bounds, historical/current capture and saved-copy
rollback checks. NOR tests include every **257** page prefix positions.

The canonical guarded Python **3.12.15** suite passed **302 tests in 216.353 s,
no skips**. Final canonical native debug and optimized release both passed
**111 tests: 96 unit and 15 integration**, no ignored/skipped tests. Unit-suite
timings were **518.52 s debug** and **1843.17 s release**, reflecting the larger
release corpus. Both runners exited 0 and verified unchanged fingerprints. Rust
and Cargo are 1.99.0, LLVM 23.1.2. The exact 41-file native test-input manifest
has SHA256 `[digest omitted from public archive]`.
Source delivery and hosted CI are recorded in [progress](05-implementation-progress.md).
An initial diagnostic test recipe incorrectly assumed the old source package was
present in the blob set. It now reconstructs only the fictional input container,
verifies its existing pinned whole-package hash and leaves production extraction
unchanged; all five focused model conformance tests passed afterwards.
Preliminary/stopped/source-drift runs do not replace final current-source results.
GPT-6.1 Sol high agents implemented and independently reviewed storage/writing;
medium prepared the independent corpus/conformance. No source-review finding
remains after the corrections; execution evidence is a separate gate.

## Dependencies and remaining scope

The former dependency-free foundation now pins **rustix 1.1.5** with only
`fs`, `process`, `std`, default features disabled, on macOS/Linux. Cargo.lock and
the audit pin its exact seven registry packages/checksums. No dependency was
installed from hardware/backend projects. Local cached crates built offline.
Native dependency notices (historical reference `../native/THIRD_PARTY_NOTICES.md`; omitted from public archive) retain source and
license information; the original upstream/root/component notices are unchanged.

Native status describes regular-file storage from fictional reads and concrete
fake-only writing, unavailable on unsupported persistence OSes. CLI file/plan/model
commands still refuse; native agent lifecycle belongs to FM1-031, release/signing
qualification to FM1-032. Lawful real corpus, real adapters, firmware health hooks,
external-bench qualification and all owner-device access remain separate tasks.
The actual unsigned macOS arm64 dispatch executable is **302,784 bytes**, SHA256
`[digest omitted from public archive]`.
Observed dynamic dependencies are system `libiconv.2.dylib` and `libSystem.B.dylib`.
Status and refusal commands ran without Python and expose the specified fake-only
scope. Unused library algorithms can be discarded by linking; this is not a
complete flasher footprint, signed release or cross-platform binary qualification.

Source `9f1364eb8e8e92b51fcfe9b1d64346f12bb445c3` is pushed to `init` in
[draft PR 2](https://github.com/kmplngj/fm1-flasher/pull/2). Its
[native run](https://github.com/kmplngj/fm1-flasher/actions/runs/38524922958) and
[host run](https://github.com/kmplngj/fm1-flasher/actions/runs/38524922962) were
blocked before any step started by GitHub's account payment/spending-limit
restriction. Current-source hosted qualification remains pending. Local checks
above completed; the later delivery record changes documentation only.

Browser tooling remains step 2. Transfer, model readback, physical raw verification,
boot and functional health remain distinct; this ticket enables no physical stage.
