# Experimental complete FM-1 USB-C raw backup

This separate CLI path extends the pinned FMR1 reader, not the legacy writer
session. It preserves stored bytes, including encrypted firmware, boot/config
areas and musical data. It does not parse, install, restore or rewrite them.
On 2026-10-16, the owner's separately authorized Apple Silicon/macOS 27.0.1
test completed two fresh, equal 1 MiB acquisitions and both protected reopened
copies in about 11.3 seconds. Final reader checkpoints passed after storage;
manual cold return produced the normal Stock screen and a fresh `FM-1_015`
reply equal to the pre-entry reply. The initial 256-byte head also matched the
earlier small read. See the [sanitized observation](evidence/fm1-complete-raw-backup-2026-10-16.json).
Both chosen destinations share one filesystem; independent media remain unverified.

## Contract

Build with the opt-in `reader-qualification` feature. Default/test builds cannot
activate the complete native backup. Full native activation is currently macOS
only; the pure protocol and Unix storage tests do not qualify Linux or Windows
hardware behavior. No new target image is needed: the exact 1,456-byte FMR1 image
and its SHA-256 remain the [small-check artifact](fm1-reader-check.md).

```sh
m-flasher raw-backup --prepare --reader /absolute/fmr-reader.bin \
  --device usb:1-1 --location 01010000 --serial 0123456789ABCDEF \
  --copy-a /private/location-a/new-check --copy-b /private/location-b/new-check
```

`--prepare` validates the pinned image and reserves two new private directories
and empty raw files without accessing devices. These names are consumed; a
subsequent `--run` uses **new** names. Existing parent directories are required.
Selectors in this example are placeholders. Running requires fresh selected
ROM enumeration from a separately authorized V15 entry, as in the small-check
contract. Storage is preflighted again before any native lookup/open/upload in
the backup process.

Replace `--prepare` with `--run` and supply the actual freshly observed selectors
and two new destination directories for the authorized physical operation.
There is no automatic MIDI entry, reset, retry, driver fallback or stock-return
command. The owner FM-1 is never used for writer/fault/agent experiments.

The default policy requires different filesystem device identities. When the
user explicitly selects directories on the same filesystem,
`--allow-same-filesystem` records that reduced redundancy. Two directories on
one SSD protect against some file mistakes; they cannot survive that SSD's
failure. Different filesystems alone also do not prove separate physical media.
Receipts retain `physical_media_independence_verified:false` unless a future
separately evidenced policy establishes it. A later manual copy to external
media is recommended for recovery retention.

## Acquisition and persistence

The helper accepts one fixed, pinned-image operation with canonical selectors.
It receives no output paths, arbitrary CDBs or user-chosen read ranges. One
retained selected USB owner spans bootstrap, acquisition, storage acknowledgement
and final continuity checks.

1. Verify FMR1 ABI/build/status/DMA zero and exact JEDEC `85 60 14` profile.
   Capacity is the selected 1 MiB JEDEC profile, not a generic autodetection claim.
2. Read `0x000FFF00` twice, verify equality and the DMA counter. Stop on failure.
3. Read every aligned 256-byte block in `[0,1048576)` in pass 1, then request
   every block again in pass 2. Check identity/chip metadata every 64 KiB.
4. Require both exact lengths, SHA-256 equality **and byte equality**. Exactly
   8,194 FD05 reads cover the two full passes plus two tail qualification samples.
5. Persist pass 1 in copy A and pass 2 in copy B. Exclusive creation, private
   permissions (including clearing/checking macOS extended ACLs on new objects),
   no-follow descriptor traversal, inode/link checks, sync and
   reopened length/hash/byte checks precede `storage-only.json` receipts.
6. The client acknowledges the exact raw digest. Only then may the retained
   helper recheck FC40/FC0B with DMA count 8,194 and return its final digest.
7. The client revalidates storage and publishes `capture-receipt.json` in each
   directory. These receipts retain stock return as pending. Partial receipts
   or raw files from a failed job are diagnostic remnants, not an accepted pair.

All post-bootstrap CDBs are FC40, FC0B and FD05. The bootstrap remains three
FB06 RAM blocks and one FB08 jump. The target has no NOR write-enable,
status-register programming, program or erase path, no FC0A and no unknown-command
ROM fallback. No package parser or musical-data rewrite is part of acquisition.

Each strict BOT transaction has a three-second deadline. The full read budget
is ten minutes and the enclosing helper budget fifteen minutes. Cancellation,
metadata mismatch, incomplete transfers, storage errors or late acknowledgements
stop acceptance without device recovery traffic. Checks occur around endpoint
operations and blocking callbacks. OS open/sync/drop operations cannot be
forcibly preempted; deadlines reject late results rather than promising a hard
OS scheduling limit. A disconnected client cannot authorize final acceptance.

## Files and evidence

Each destination contains `backup.raw` (1,048,576 bytes), `storage-only.json`
and, only after final continuity, `capture-receipt.json`. Raw files and detailed
provenance contain device-private data; do not commit, upload or quote them in
public reports. Reopened storage evidence cannot authorize flash writes or
claim the target returned to its normal application.

After accepted capture, fully switch the FM-1 off, unplug USB, wait about five
seconds, then switch on normally and reconnect. A fresh selected Stock USB
observation and MIDI `FM-1_015` identity are a separate return outcome. USB
unplugging alone may leave the battery-powered reader running.

The operation's report must distinguish fresh complete acquisition, protected
copies, filesystem/media independence, and observed return to Stock. Successful
backup does not qualify restore, recovery or subsequent firmware flashing.

This implementation and observation cover the selected #9 acquisition slice
and protected-copy composition from #10. They do not close #10's independent
destination/media qualification for released platforms. Raw-copy receipts
remain immutable capture-phase records; the subsequent manual return is kept
as a separate private operator outcome and summarized in the public observation.

## Offline verification

`python3 tools/run_offline_tests.py --cargo` proves the no-hardware guard before
the shared and complete Cargo tests. New injected tests cover full command
coverage/freshness, changed successful reads, metadata/counter drift, deadlines,
cancellation, persistence failure and final-checkpoint rejection. File-only
tests cover path/link substitution, private exclusive creation, write/sync/reopen
failure and receipt tampering. Local IPC tests use socket pairs, not devices.
