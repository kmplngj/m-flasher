> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Native USB capture and secure two-copy storage — FM1-033-C

Date: **2026-10-15**. Branch: `init`, based on the locally committed FM1-033-A/B
increment `f69e37e99e6c6ed2652204a1faabc8221beeaa58`. Scope is offline host
development: concrete in-memory USB endpoints and actual private regular files.
No hardware enumeration, access, physical read/write, loader execution, firmware
build or backend installation occurred.

## Result and limits

USB capture now has its own persistence path. It reuses the reviewed macOS/Linux
file primitives without borrowing Transporter source-profile or writer authority.
The immutable files contain a full modeled 1 MiB snapshot and exact fictional
USB session provenance. Saved-file acceptance remains historical, with source
firmware, loader behavior and every physical/write claim false.

This implements FM1-033-C's library boundary. It does not expose a live CLI
capture command or qualify stock entry, a RAM loader or a native platform
adapter. FM1-033-D/E remain separate. The [stock-route research](16-stock-usbc-backup.md)
and [A/B implementation record](17-native-usbc-backup.md) retain their original
evidence and limitations. No owner backup was created by these tests.

GPT-6.1 Sol high implemented the storage API and independently reviewed it;
medium wrote independent conformance tests. Root integrated private USB
provenance, capabilities, the guarded test selector and documentation.

## Capture and authority boundary

`usb_backups::capture_usb_backup(reader, primary, second, created_at, fault)`
accepts one concrete `BotReader` and two explicit existing private directories.
It takes no package, `PlanRequest`, caller snapshot or saved report as current
source authority.

1. Validate the timestamp and open both directories through descriptor-anchored
   `local_files::Directory`. Unsupported OS persistence, unsafe paths, identical
   directories or existing USB artifact names refuse before any new full read.
2. Read exact private provenance from the current fake reader. It records saved
   descriptor fields, fictional [local artifact location omitted] identity, endpoint lifetime,
   generation, lease and their existing session commitment. Decimal u64 strings
   avoid signed JSON overflow. The parser reconstructs the digest; this proves
   record consistency only, never a physical device or trusted loader.
3. Issue two new complete reads, including head, APP and data/tail. Require equal
   bytes, exact session and consecutive latest read numbers. Previous caller
   reads cannot replace either new sweep. A differing fresh second read creates
   no snapshot or acceptance marker.
4. Exclusively create `usb-snapshot.bin` in both directories, then canonical
   `usb-accepted.pending.json` manifests. Each create uses actual file/directory
   sync, reopen and byte/identity verification. Recheck the reader after every
   persistence operation.
5. Publish the second manifest, then the primary, as `usb-accepted.json` using
   the existing no-overwrite publication primitive. Verify both complete copies
   and current reader again before returning the private `UsbBackup` handle.

No failure triggers cleanup that could discard evidence, file overwrite, replay,
automatic resume, reset or device command retry. Partial artifacts remain
unaccepted. A later explicit saved verification may accept a complete consistent
historical pair; it cannot reconstruct the original live capture authority.

`UsbBackup::verify` reopens both snapshots/manifests, checks retained complete
bytes/hashes and file identities, syncs, rechecks named identities/ancestry and
rejects pending markers. `verify_current` additionally checks the original exact
reader/provenance/latest pair before and after all storage IO. Further full
reads, reconnect/session/descriptor drift or file replacement expire current
acceptance. A `fresh_capture` label records origin; a support report does not
assert that a reader is currently present.

`UsbBackupLocator` is private serialization of two paths plus the metadata hash.
It opens nothing when constructed/parsed. `verify_saved_usb_backup` reopens and
validates both actual files, canonical closed-schema metadata, exact locations,
full bytes and distinct inodes before creating a historical handle. Such a
handle always refuses `verify_current`, including identical saved session hashes.
User-created locators and self-consistent manifests cannot prove that a capture
occurred; historical acceptance reports no source/session/hardware authority.

There is no conversion to Transporter `VerifiedBackup`, planner backup evidence
or writer eligibility. Manifest scope is `fictional-usb-only`, transport
`synthetic-usb-bot`, source state `fictional-held-loader`, source firmware
`unknown`. Every source/loader/hardware/physical/writer proof field is required
to be false. Exact schema/tool version is enforced; older tool versions require
an explicit future compatibility decision rather than silent acceptance.

## Secure file and privacy behavior

The existing reviewed file layer is unchanged. It anchors every directory
component with no-follow/nonblocking opens, private ownership/permissions and
before/opened/after identity checks. Snapshot/manifest creation is exclusive;
published files are read-only. Symlink, hardlink, FIFO, ancestry rebinding,
unsafe names, changed files, short writes and sync failures refuse through the
same boundary. IO remains restricted to the runner's scratch tree in tests.

Two directories and separately created files establish distinct local copies.
They do not establish separate physical media or physical durability; those
flags remain false. Secure persistence is available on macOS/Linux. Other OSes
refuse before fake full reads; that path has a source-reviewed test for future
platform CI and is not claimed locally exercised.

The existing lexical path policy is unchanged, including refusal of `/Volumes`.
Common macOS removable-media destinations therefore need separate reviewed path
support; this increment does not qualify an external-drive backup destination.

Public reports and Debug redact paths, raw snapshot bytes and source identifiers.
Explicit manifest/locator serialization includes private provenance/paths and
must be treated as private backup material. The report distinguishes local
fictional file verification from source firmware proof and physical backup.

## Capability and CLI behavior

Native status adds OS-dependent `native_fake_usb_backup_capture_available` and
`historical_usb_backup_verification_available`, with
`native_usb_backup_storage_scope=regular-files-from-fictional-usb-reads`.
`stock_usbc_full_backup_available`, hardware access, physical evidence and real
writes remain false. `backup capture`, entry and upload commands still refuse.
The existing pure `backup assess` command reports unmet route requirements.

## Validation

Validation completed against stable source. The unchanged
source/API audit passed for **30 Rust files**; dependency pins, the hardware
guard and local IO implementation are unchanged.
The native runner adds a bounded Rust-name substring filter behind its existing
audit/build/scratch/fingerprint checks. Invalid and zero-match filters fail with
exit 2; default full validation remains unfiltered.

- Canonical Python **3.12.15** runner: **302 tests passed in 211.822 s**, no skips.
- Focused native debug USB tests: **25 passed in 0.87 s**, 99 deliberately
  filtered out. Optimized USB tests: **25 passed in 0.96 s**, 99 filtered out.
  These cover the 15 new storage/conformance cases and ten retained BOT cases.
- Invalid `--filter=--ignored` refused before build; a well-formed no-match
  filter reported zero selected tests and refused with exit 2.
- Format, offline locked release build and actual native status/assessment and
  capture-refusal checks passed. No new physical capability was reported.
- Full unfiltered native debug: **144 passed — 124 unit and 20 integration**,
  no ignored/skipped/filtered tests; unit-suite time **531.55 s**. The native
  runners finished exit 0 with unchanged source/fixture fingerprints. The
  unsupported-OS test is excluded on this Mac and is not counted as exercised.

The unsigned local macOS arm64 status/dispatch executable is **319,376 bytes**,
SHA-256 `[digest omitted from public archive]`.
Unused library code can be removed by linking; this measures the dispatch CLI,
not a full backup/flasher release. The exact 46-file native input map is saved
under ignored `[local artifact location omitted]`, SHA-256
`[digest omitted from public archive]`.

The new suite covers fresh capture despite earlier reads, full independent
patterned bytes/hash, exact four files, metadata/session/descriptor/location
binding, historical-only acceptance, privacy, collision/no-clobber, all four
files corrupted/missing/rebound, symlinks/hardlinks/ancestry, unequal full reads,
representative publication/persistence cuts and late session/copy changes.
Actual IO error mapping and a late descriptor change after file verification
have retained tests. Existing shared-primitive FIFO and full fault tests remain
in the full suite; the new suite does not consume its macOS singleton fixture.

Independent image SHA-256, computed with Python stdlib over
`bytes((i*73+i//251+19)%256 for i in range(1048576))`:
`[digest omitted from public archive]`.
All source/device bytes are original fictional fixtures, with no vendor blob.

Reproducible commands (Python 3.11+):

```sh
rtk proxy python3.12 host/run_tests.py
rtk proxy python3.12 native/run_tests.py
rtk proxy python3.12 native/run_tests.py --release --test lib --filter usb_
rtk proxy cargo fmt --manifest-path native/Cargo.toml -- --check
rtk proxy cargo build --manifest-path native/Cargo.toml --release --offline --locked
```

The actual local interpreter is
`[private host path omitted]`.
Logs are retained under ignored `[local artifact location omitted]`. Full debug,
focused optimized, local release artifact and hosted/platform qualification are
reported separately; no earlier green result is substituted for this source.
This increment is local on `init`; no push or current-source hosted CI claim.

All upstream firmware/PIO/submodules and notices, Cargo dependencies, Python
production models and its hardware guard remain unchanged. Physical reads and
all flash writes remain unavailable in this implementation.
