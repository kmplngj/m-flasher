# Repository migration, 2026-10-21

**Status:** the owner renamed the original repository to
[`kmplngj/m-flasher-legacy`](https://github.com/kmplngj/m-flasher-legacy).
Its repository ID remains `1422153335`; the existing issues, branches, PRs,
Actions history and pinned installer commit remain accessible. The legacy
repository remains writable for the remaining open loader-safety PR.
The canonical work tracker for the existing FM-1 roadmap stays there.

[`kmplngj/m-flasher`](https://github.com/kmplngj/m-flasher) is a fresh GitHub fork
of the recreated [`jvitkauskas/m-flasher`](https://github.com/jvitkauskas/m-flasher).
Its repository ID is `1428000725` and its parent ID is `1423336301`. The backport
is tracked by [PR 1](https://github.com/kmplngj/m-flasher/pull/1), based on the
upstream baseline. Its PR records the final review and merge evidence.

The recreated upstream has unrelated Git history. This port starts at upstream
`7cd66e0` (0.5.1) and applies only our changes since old upstream `9d96274`,
with adaptations for the newer source. It does not merge the unrelated histories
or replace the current upstream bootloader/loader assets with the old ones.

## Ported changes

- Legacy main `e897df2`: experimental FMR1 reader and protected two-pass raw
  backup, saved-file diagnostics, parser/transaction checks, file/fake tests,
  contribution/evidence documents. The former native Felucca OTA CLI is excluded.
- Upstream `src/midi.rs` retains the download-mode soft key. Test/offline builds
  exclude its native adapter and test its refusal paths. The real sequencer
  loopback test is excluded from this offline-only suite.
- The upstream SCSI byte-count contract is retained. The offline adapter is
  adapted to it. Current board recognition and m-bootloader support remain.
- The native Rust OTA package, codec, adapter, fixtures, workflow and packaging
  are removed from the active fork. The pending loader-safety change from
  [legacy PR 32](https://github.com/kmplngj/m-flasher-legacy/pull/32) stays in
  legacy history; no native installer is adopted into Felucca.

The research archive from [legacy PR 27](https://github.com/kmplngj/m-flasher-legacy/pull/27)
merged into legacy main `aeb061a` during migration and is included in a separate
port commit. All archive files and their manifest are preserved byte-for-byte;
only the surrounding roadmap issue link is adapted to the renamed tracker.
The unmerged exact-read and exclusive-backup upstream candidate branches remain
in the legacy repository; those proposed behavior changes are not integrated.

## References and consumers

Historical issues and PRs do not migrate to the fresh fork. Existing roadmap
links point explicitly to `m-flasher-legacy`; historical evidence JSON is
preserved byte-for-byte. References inside historical snapshots may still use
the original name and must be resolved in the legacy repository. Reusing the
old name means GitHub redirects no longer resolve those URLs to the old repo.

Felucca uses its maintained Python updater (`tools/fm1_install.py`).
[Felucca PR 502](https://github.com/kmplngj/Felucca/pull/502) removes the native
binary downloader, source pin, cache setup and backend selection. The helper
checks `mido`/`python-rtmidi` in the caller's interpreter before firmware CI;
download/source-only modes need no MIDI packages. Historical native source and
Actions artifacts remain in legacy history and are not used by this workflow.

## Validation and review status

Review and hosted build evidence belongs to PR 1. Local file/fake tests do not
qualify USB/MIDI access, bootability, restoration, physical-media independence
or writer failure/recovery. No device is accessed and no firmware is flashed or
release created as part of this source migration.

After removing the native OTA code, local validation passed 4 offline adapter
guards and 71 shared tests. Full guarded Cargo, platform builds and reader
harnesses are required by CI; the local dependency cache is incomplete. Private
firmware corpus tests remain ignored, and physical qualification is pending.
Historical observations remain attributed to their tested artifact and device.

The README describes backup/diagnostics and upstream MIDI download-mode support,
and points to Felucca's Python updater for OTA. There is no repository changelog;
this migration creates no release or new hardware qualification.
