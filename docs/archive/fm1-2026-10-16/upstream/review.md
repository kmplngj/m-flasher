> Archived 2026-10-16 from the local upstream-preparation handoff. Historical statements and test/review counts below are preserved reports, not checks rerun for this archive. Sanitization and current-state verification are documented in [README.md](README.md).

# Independent reviews of the final upstream candidates

Base: upstream `main` at `9d962747437d92cbab594c2320b586d162c6572d`. Two independent GPT-6.1 Sol high reviewers inspected the actual tracked diffs and new helper files in parallel, one for Standards and one for Spec. Both reviews completed before committing; the reviewed source was committed unchanged.

## Standards

**Exact flash reads (`f7086b3`):** no findings. The FD05-only count check retains other IN behavior and existing recovery paths, with a pure `FnOnce` test seam and injected transfers. No fork policy or hardware dependencies enter the candidate.

**Exclusive backups (`9613b66`):** no findings. Both final names are reserved before writing; failures retain partials. Sync and bounded reopen verification precede success. Naming, sidecar layout, ownership calls and `backup_path` ordering remain intact. Its limited local-file contract is explicit.

Standards violations: 0. Actionable smells: 0, for each candidate.

## Spec

**Exact flash reads (`f7086b3`):** no findings. Successful FD05 counts must equal the requested buffer length; generic short-IN compatibility and transport-error handling remain intact. Failure propagates through existing adapter recovery and higher-level retry paths. Independent protocol literals cover untouched tails, chunk boundaries, generic commands and transport errors. Reviewer independently compiled and ran all five standalone tests successfully.

**Exclusive backups (`9613b66`):** no findings. Exclusive reservation of both names precedes data writes; collisions preserve existing data, including symlink targets. File/Unix directory sync and bounded exact reopen checks precede success. The existing apply gate propagates save errors before erase/program. Windows directory-sync limits are explicit. Reviewer independently compiled and ran all six standalone filesystem tests successfully, including nine injected failure stages.

Spec findings: 0, for each candidate.

Reviews and tests were offline. No hardware or application execution, installs, publication or physical qualification was performed. Both worktrees were moved outside `target/` after review without changing source bytes.
