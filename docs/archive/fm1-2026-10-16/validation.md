# Archival validation — 2026-10-16

This record covers [issue #25](https://github.com/kmplngj/m-flasher/issues/25).
No hardware test is required for documentation preservation.

## Final integration base

Initial archive review used `9005a0da9a879ca8a09aec55d57a46850649406b`.
While this work was underway, unrelated saved-diagnostics PR #26 merged into
`main` as `b8cf9623439b2d8c8655ea843eead37722a2e23b`. The documentation commit
was rebased onto that fetched base without conflicts or changes to archive
content. All changes relative to that base are under `docs/`.
The historical observations in the assessment remain pinned to their stated
revisions; this newer integration base gains no physical qualification.

## Checks and results

- Private recovery: `git bundle verify` passed; a fresh isolated bare clone of
  the bundle resolved `refs/heads/init` to the exact original source head.
  The temporary restore checkout was removed after comparison. Original
  working-tree documents and all six original handoff documents remain
  byte-identical to the private copies. Original source status is unchanged.
- Export integrity: all 53 text lengths/hashes and original committed Git blob
  identities matched the manifest. All 30 commit rows exactly matched the
  ordered source range. The three working-tree inputs are flagged separately.
- Publication safety: only Markdown, JSON and the original license text enter
  this change. Local document links resolve; references to deliberately omitted
  legacy/evidence files are explicit historical pointers. Home/temp paths,
  private device selectors, raw-image digests, private receipts, binary payloads
  and secret patterns are excluded. The MIT notice is byte-identical to source.
  The independent [publication review](publication-review.md) found no remaining
  blocking finding; its source/metadata checks were not hardware tests.
- `rtk proxy <existing Python 3.12.15> host/run_tests.py` in the reference
  checkout: **318 passed**, 198.477 seconds. The canonical runner installed the
  no-hardware guard before test discovery. The executable's private installation
  path is intentionally omitted from this public command record.
- `rtk proxy <existing Python 3.12.15> tools/run_offline_tests.py` in M-Flasher:
  **4 guard tests + 53 fixtures passed** on the initial base; after the unrelated
  base update, **4 guard tests + 67 fixtures passed**. Native project adapters
  were excluded by the runner. No optional Cargo or private-corpus checks were
  requested. No application/firmware execution or target artifact build was
  used by these runners.
- `rtk proxy git diff --check` and staged-diff checks passed. No upstream
  firmware, source implementation, dependency/submodule pin or existing license
  was modified by this archival branch.

The documentation commits carry `[skip ci]` to avoid triggering unrelated
application/package workflows for an archival export. Skipped CI is not a
passing hosted test result. Local checks above are the validation evidence.
There is no merge, release, new hardware observation or flash/status-register
write in this task. The FMR1 discussion remains unsubmitted to upstream.

Remote archive location, final commits, PR and remote-content verification are
recorded in issue #25 after publication. No private recovery location, original
Git bundle or private checksum manifest is uploaded with that record.
