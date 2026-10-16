# Upstream handoff preservation — 2026-10-16

This archive preserves the six text/JSON handoff documents identified in
[fork issue #25](https://github.com/kmplngj/m-flasher/issues/25), from the local
`m-flasher-upstream-preparation/` directory. Original documents and nested code
worktrees remain untouched. This directory contains documentation only.
The [FMR1 architecture discussion](reader-rfc.md) remains an **unpublished draft**;
it has not been submitted upstream by this archival work.

## Current primary-source verification

Read-only GitHub metadata was queried on 2026-10-16 with `gh pr view --json`
and `gh api` (through `rtk proxy`). Exact timestamps, files, head/base identities,
checks and workflow outcomes are retained in [verified-status.json](verified-status.json).

| PR | State | Exact head | Exact base |
| --- | --- | --- | --- |
| [upstream #7](https://github.com/jvitkauskas/m-flasher/pull/7) | OPEN | `de20d0e7f2692ecd77f7ac95203dda5943811863` | `9d962747437d92cbab594c2320b586d162c6572d` |
| [upstream #8](https://github.com/jvitkauskas/m-flasher/pull/8) | OPEN | `f7086b370f474fe22795edf7de3561f6d64e72fe` | `9d962747437d92cbab594c2320b586d162c6572d` |
| [upstream #9](https://github.com/jvitkauskas/m-flasher/pull/9) | OPEN | `9613b66ec860c2c412b099e7193d1211e0785d78` | `9d962747437d92cbab594c2320b586d162c6572d` |
| [fork #24](https://github.com/kmplngj/m-flasher/pull/24) | MERGED | `4efefbecc8982c526c7ffeb2f8937a4be352052d` | `5b7a77ea7e5ed82bc41bac198dfa1e73ccd53917` |

All target `main`. Fork #24 merged as
`9005a0da9a879ca8a09aec55d57a46850649406b` at `2026-10-16T16:35:49Z`;
its 47 returned checks comprise 45 successes and two skipped release jobs.
Upstream #7/#8/#9 have empty status-check rollups. The recorded workflow runs
for #8 and #9 remain completed with `action_required`, not passing CI.
These are snapshot states, not evidence of future acceptance or release.

## Historical physical evidence and limits

The [original discussion](reader-rfc.md) correctly separates the full-backup
observation from later host hardening. The existing sanitized
[full-backup record](../../../evidence/fm1-complete-raw-backup-2026-10-16.json)
records a 1,456-byte reader with public artifact SHA-256
`63c12a9ded28b53fa991c8261cbc8065ef93f8e71a805f53553999175422618a`.
For this archive, all 23 recorded runtime-source SHA-256 values were compared
against file bytes obtained through `git show`: all match historical host
`4ce0342731d6acbdbb47a383e94c559e230ad7fe`. At the later reviewed PR #24 head
`4efefbecc8982c526c7ffeb2f8937a4be352052d`, 20 match; `src/fmr_cli.rs`,
`src/fmr_options.rs` and `src/fmr_storage.rs` differ. The record's
`base_commit` is an implementation base, not the complete observed host identity.
This comparison verifies the recorded source attribution; it does not reproduce
the physical observation or qualify the newer host.

That historical record reports two equal fresh 1 MiB passes, protected reopened
copies, held-session checks and a separate manually observed Stock return on
one Stock V15 unit and Apple Silicon/macOS host. Both copies were on one
filesystem; independent physical media remain unverified. The earlier
[limited head-read record](../../../evidence/fm1-reader-check-2026-10-16.json)
has its own CLI/source identities and is a separate observation. Neither
observation establishes writes, restore, fault recovery or other units/platforms.
No new hardware observation, build, test rerun or application execution occurred
for this archive. No raw backup bytes/digests or private receipts are included.

## Preserved documents and edits

| Preserved file | Source / treatment |
| --- | --- |
| [handoff-README.md](handoff-README.md) | Original `README.md`; historical banner, independent-checkout Cargo command, public immutable candidate links, qualified historical checkout statement. |
| [reader-rfc.md](reader-rfc.md) | Original architecture discussion; historical provenance banner; remains unpublished. |
| [review.md](review.md) | Original independent-review report; historical provenance banner. |
| [publication.json](publication.json) | Original publication snapshot, retaining useful PR/workflow metadata; personal owner name/account ID block removed. Not a fresh verification. |
| [exact-flash-reads-pr.md](exact-flash-reads-pr.md) | Original submitted PR description; historical banner and RTK-prefixed example commands. |
| [exclusive-backups-pr.md](exclusive-backups-pr.md) | Original submitted PR description; historical banner and RTK-prefixed example commands. |

The reported standalone counts (five and six tests), independent zero-finding
reviews and earlier guarded harness counts are historical reports, not current
rerun results. The absolute shared `CARGO_TARGET_DIR` setting was removed;
`cargo check --offline --locked` now uses each candidate checkout's own target
directory. These example commands were not executed for archival work.
Original attribution to the public repositories, PRs and reviewer model/roles is
retained; private author metadata, machine paths, device selectors/serials,
raw-image identifiers and private receipts are excluded. No external code,
firmware binaries, dependency pins or license notices are copied or changed.

This archive does not submit the draft, merge upstream PRs, resolve functional
issues or authorize device access. Issue #25's other preservation and delivery
gates are tracked by the containing archival work.
