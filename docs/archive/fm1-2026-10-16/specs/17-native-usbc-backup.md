> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Native stock USB-C backup foundation — FM1-033

Date: **2026-10-15**. Branch: `init`. This is FM1-033-A/B's offline increment,
not a completed live-device backup feature. The owner authorized backup hardware
access with **no flash writes**. This change used no hardware, enumeration,
device commands, mode changes, loader execution or firmware builds.

## Outcome and boundaries

The [research](16-stock-usbc-backup.md) includes all five owner-supplied Reddit
threads and pinned source code. It identifies a credible direct-cable route
from exact stock V15 through temporary ROM/RAM-loader modes. It does not prove
a full raw-flash read endpoint in the normally running stock application, a
write-free opaque loader or a working macOS native transport. Installed stock
remaining unchanged is distinct from the application continuing to run.

The distributed implementation remains Rust. GPT-6.1 Sol high agents performed
source research, implemented the USB model and independently reviewed the code.
A medium agent updated current-state documents and added independent public-API
conformance tests. Root implemented stock-backup assessment/comparison, CLI
integration and validation. No hardware backend or dependency was installed.

## Implemented interfaces

`native/src/usb_bot.rs` ports the existing Python FM1-019 read model into a
separate concrete fictional endpoint. It does not generalize the Transporter
reader or existing backup authority.

- Explicit saved selection checks OS, fictional identity/location and exact
  MSC/BOT descriptor fields. Ambiguous or unsupported selection refuses before
  commands. Numeric values come from the earlier pinned public ROM log; the
  standards-shaped INQUIRY response is explicitly generated, not a capture or
  physical compatibility fingerprint.
- Only INQUIRY and LoaderV2 FD05 raw-flash reads can be constructed. Bounds
  are the modeled 1 MiB capacity and a 512-byte maximum. There is no arbitrary
  CDB executor, FD07 memory export, stock-to-ROM command, FB06/FB08 upload/jump,
  reset, clear-halt, retry, erase or programming operation.
- Every transaction checks exact CBW/data/CSW lengths, signatures, tag, zero
  residue and success status. Fragmented binary data remains bounded. Failures
  poison the session; malformed ranges fail before a transaction. Transaction
  and full-dump deadlines use an injected concrete clock.
- A full read starts at zero and issues 2048 new reads through the last flash
  block. Private snapshots bind complete bytes, session and monotonic read
  number. Private endpoint lifetime/generation/lease and exact descriptor,
  target and loader detect drift. Counters distinguish fictional sessions in
  one process; they are not physical or restart-persistent identity evidence.
- Tests can inject finite replies, faults and a scheduled mid-read state change.
  No callback, backend trait or OS API can become an adapter. Public Debug
  output redacts identities, flash contents and retained snapshot digests.

`native/src/stock_backup.rs` provides two independent, non-authorizing APIs:

1. `BackupAssessment` reports fixed missing requirements per requested platform
   and caller-reported source state: running stock, bare ROM or held loader.
   All actual backup, access, source-proof and write flags are false. A reported
   loader is never accepted as loader evidence. macOS access/checkpoint and
   Linux/Windows result-validation gates remain separate. No firmware detour or
   RP2040 fallback is selected.
2. `compare_replay_reads` requires the current concrete reader, equal session,
   consecutive full reads and the most recent read number. It compares every
   byte and returns a fictional comparison report. Same/reversed/skipped/stale
   pairs, different sessions, drift and differing bytes refuse. This result
   has no conversion to `VerifiedBackup` or writer authority and is not durable
   storage or stock-firmware provenance.

The real snapshot must eventually cover the independently established complete
flash capacity, including head, application, data/presets/calibration and tail.
The 1 MiB fixture is a model assumption, not a measurement of the owner's unit.
Preset exports or package downloads cannot replace that snapshot.

## CLI behavior

```sh
rtk proxy [local artifact location omitted] backup assess --platform macos --source-state running-stock --json
```

This pure dispatch command performs no file/device IO. Exit 0 indicates a
generated requirements report; the report's `state` is `blocked`. Unknown names,
extra flags and attempted capture/entry/upload commands return sanitized refusal
and exit 2. Caller-supplied identifiers or paths are never echoed.

`status --json` distinguishes `native_usb_bot_replay_available=true` and
`stock_usbc_backup_assessment_available=true` from
`stock_usbc_full_backup_available=false`. Existing hardware/write/physical proof
flags remain false. The owner's permission is not a software capability.

## Validation

The unchanged Python guard and native source/API audit exclude hardware access
from tests. No audit allowance, Cargo dependency change or native adapter was
added. `native/run_tests.py` adds only the new conformance target selector;
its audit, private scratch and stable-source checks remain mandatory.

Independent expectations include literal wire bytes and complete inspection of
all 4097 commands for two reads plus INQUIRY. The public-API corpus uses an
original position-dependent image whose Python-stdlib SHA-256 is
`[digest omitted from public archive]`.
It checks the head and final block, fresh second-pass data, retained sessions,
mode changes and false authority/durability/hardware reports. Unit tests cover
all saved platform labels, selection/descriptor errors, malformed data/status,
busy/permission/stall/short-CBW faults, clock/deadline failures, drift during a
dump, bad INQUIRY, lease cleanup and sanitized output. These are software-only
results, including the Linux/Windows labels.

The canonical unchanged Python runner on Python **3.12.15** passed **302 tests
in 228.390 s**, no skips. The native source/API audit passed for **28 Rust
files**. The optimized `stock_backup_conformance` target passed all **five**
public-API tests in **0.03 s**. The complete debug runner passed **129 tests:
109 unit and 20 integration**, no ignored/skipped/filtered tests; the unit
suite took **557.88 s**. Both native runs finished exit 0 with unchanged
source/fixture fingerprints. This increment adds 18 tests.
Logs are kept locally under ignored `[local artifact location omitted]`; no existing
hosted CI result or physical observation is attributed to this source.

Reproducible checks (using a Python 3.11+ interpreter):

```sh
rtk proxy python3.12 host/run_tests.py
rtk proxy python3.12 native/run_tests.py
rtk proxy python3.12 native/run_tests.py --release --test stock_backup_conformance
rtk proxy cargo fmt --manifest-path native/Cargo.toml -- --check
rtk proxy cargo build --manifest-path native/Cargo.toml --release --offline --locked
```

The actual local interpreter was
`[private host path omitted]`;
the macOS system `python3` is 3.9 and was not used to execute either suite.
The unchanged full optimized writer matrix was not rerun for this additive read
increment; current-source full debug and focused optimized results are reported
separately from FM1-030's earlier complete release evidence.

Format check, the offline locked release build and actual local CLI
assessment/status/refusal checks passed. The assessment returned `state=blocked`
and false hardware/read/write flags; attempted `backup capture` refused with
exit 2. The unsigned local macOS arm64 executable is **319,376 bytes**, SHA-256
`[digest omitted from public archive]`.
That measures this dispatch/assessment executable, not a complete flasher.
The 44-file native test-input map is saved under ignored
`[local artifact location omitted]`, SHA-256
`[digest omitted from public archive]`.

## Next backlog work

The following describes the boundary at FM1-033-A/B completion. The later
[FM1-033-C storage increment](18-native-usb-backup-storage.md) records its own
implementation and validation separately.

FM1-033-C remains open: integrate USB-specific fresh captures with immutable
two-copy durable storage while retaining source/session provenance and without
disguising USB evidence as Transporter authority. FM1-033-D then implements
reviewed selected-device adapters per OS after exact entry/loader evidence;
FM1-033-E records an actual authorized read-only backup and normal return.
No native file/capture command, ROM entry or RAM-loader executor is enabled by
this increment. FM1-031/032 and preset-only/formatted exports remain separate.

The source/license notices, upstream Transporter firmware, PIO, submodules,
Python production models and no-hardware guard are unchanged. No vendor
firmware or loader blob is bundled. A zero-brick-risk or successful restore
claim cannot be inferred from these fictional reads.
