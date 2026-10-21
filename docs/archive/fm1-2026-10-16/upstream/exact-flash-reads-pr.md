> Archived 2026-10-16 from the local upstream-preparation handoff. Historical statements and test/review counts below are preserved reports, not checks rerun for this archive. Sanitization and current-state verification are documented in [README.md](README.md).

# Reject truncated USB flash-read payloads

USB flash reads currently discard the successful transfer count, allowing untouched buffer tails to enter backups and verification. Reject incomplete FD05 payloads before the caller treats the requested chunk as complete. Do not issue another data read to fill the missing bytes: the following transfer may contain the command status.

The small pure data-phase helper preserves existing handling for other IN commands and transport errors. Existing CSW validation, adapter reset/clear-halt behavior and bounded higher-level retries are unchanged; this PR does not establish a strict no-recovery transport.

Five standalone tests pass, covering every short count for a 256-byte read, complete payloads, partial-final and loader chunk sizes, generic short-IN compatibility and transport errors. macOS `cargo check --offline --locked` also passes. Independent agent Standards and Spec reviews report zero findings; the Spec reviewer separately reran all five tests. The tests cover the data-phase seam, not physical transfers or the complete native adapter. No hardware or application execution was used.

Based on upstream `main` at `9d962747`; commit `f7086b370f474fe22795edf7de3561f6d64e72fe`. No dependency on pending upstream PR #7 or fork-only features. Related scope: kmplngj/m-flasher#4; this narrow contribution does not satisfy its complete strict-transport acceptance criteria.

Validation from this branch:

```sh
rtk proxy mkdir -p target
rtk proxy rustc --edition=2024 --forbid unsafe_code --test src/usb_data_in.rs -o target/usb-data-in-tests
rtk proxy target/usb-data-in-tests
rtk proxy cargo check --offline --locked
rtk git diff --check
```
