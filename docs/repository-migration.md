# Repository migration, 2026-10-21

**Status:** the owner renamed the original repository to
[`kmplngj/m-flasher-legacy`](https://github.com/kmplngj/m-flasher-legacy).
Its repository ID remains `1422153335`; the existing issues, branches, PRs,
Actions history and pinned installer commit remain accessible. The legacy
repository remains writable for the remaining open loader-safety PR.
The canonical work tracker for the existing FM-1 roadmap stays there.

The source port is staged in that repository. Creating the fresh fork is pending:
GitHub returned `403 Resource not accessible by integration` for fork creation
with the current credential. The owner must create the fresh fork before the
prepared branch can be pushed there and a draft backport PR opened.

[`kmplngj/m-flasher`](https://github.com/kmplngj/m-flasher) will be a fresh GitHub fork
of the recreated [`jvitkauskas/m-flasher`](https://github.com/jvitkauskas/m-flasher).
The recreated upstream has unrelated Git history. This port starts at upstream
`7cd66e0` (0.5.1) and applies only our changes since old upstream `9d96274`,
with adaptations for the newer source. It does not merge the unrelated histories
or replace the current upstream bootloader/loader assets with the old ones.

## Ported changes

- Legacy main `e897df2`: experimental FMR1 reader and protected two-pass raw
  backup, saved-file diagnostics, parser/transaction checks, file/fake tests,
  contribution/evidence documents and optional native Felucca MIDI OTA CLI.
- Upstream `src/midi.rs` retains the download-mode soft key. The fork's OTA
  codec moves to `src/midi_ota.rs`. Test/offline builds exclude both native
  adapters and test their refusal paths. The real sequencer loopback test is
  excluded from this offline-only suite.
- The upstream SCSI byte-count contract is retained. The offline adapter is
  adapted to it. Current board recognition and m-bootloader support remain.
- Pending [legacy PR 32](https://github.com/kmplngj/m-flasher-legacy/pull/32),
  `af3cd2b`: exact compatible-loader gating, ported as a separate commit. This
  preserves the safety constraint already required by Felucca's pinned helper;
  it does not mark that PR merged or establish new hardware qualification.

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

Felucca's installer pins `af3cd2b75e175e09b695a261481774952d6e414f` and its
existing native Actions artifacts. It must fetch those from `m-flasher-legacy`
until a separately reviewed adoption changes its source pin. Existing manifests
using the original repository name remain historical provenance for that exact
revision. The migration PR is not a replacement binary or release.

## Validation and review status

The source port is staged on `kori/recreated-upstream-backport` for a draft PR
after the rename and fresh fork. Independent review, hosted build checks and
hardware validation remain separate gates. Local file/fake tests do not qualify
USB/MIDI access, bootability, restoration, physical-media independence or writer
failure/recovery. No target code is run, no device is accessed and no release is
created as part of the migration.

The README adds an integration-fork overview and links the migration and existing
capability guides. There is no changelog in this repository; this is a source
migration, not a new feature release. Existing dated evidence remains attributed
to its original source/artifact and device observation.
