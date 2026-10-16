> Archived 2026-10-16 from the local upstream-preparation handoff. Historical statements and test/review counts below are preserved reports, not checks rerun for this archive. Sanitization and current-state verification are documented in [README.md](README.md).

# Published upstream contributions — 2026-10-16

Fork PR [kmplngj/m-flasher#24](https://github.com/kmplngj/m-flasher/pull/24) merged normally into `main` as `9005a0da9a879ca8a09aec55d57a46850649406b`, after the exact reviewed head `4efefbecc8982c526c7ffeb2f8937a4be352052d` passed all 47 posted checks (success or expected skip). Both review threads are resolved. The original handoff reported that local `main` matched its then-fetched `origin/main`; this is not an archive-time checkout claim.

The two contributions below are **committed, independently reviewed, offline-validated, pushed to `kmplngj/m-flasher` and published as separate open PRs against `jvitkauskas/m-flasher:main`**. Remote branch and PR head identities were verified. Both are exactly one commit on freshly fetched upstream `main`, `9d962747437d92cbab594c2320b586d162c6572d`, which is unchanged since review. Neither includes fork policy, dependencies, bundled firmware, loader changes or pins. The worktrees are outside Cargo's `target/` directory so a normal build cleanup does not remove their sources or these documents. Neither contribution is merged or released.

| Contribution | Branch / commit | Local checkout | State |
| --- | --- | --- | --- |
| Reject incomplete USB command-wrapper writes | `kori/4-reject-short-cbw` / `de20d0e` | Existing branch in the main repository | Existing upstream [PR #7](https://github.com/jvitkauskas/m-flasher/pull/7), open; no new reviews/comments at latest inspection |
| Reject truncated USB flash-read payloads | `kori/4-upstream-exact-flash-reads` / `f7086b370f474fe22795edf7de3561f6d64e72fe` | [published candidate tree](https://github.com/kmplngj/m-flasher/tree/f7086b370f474fe22795edf7de3561f6d64e72fe) | Published, open [PR #8](https://github.com/jvitkauskas/m-flasher/pull/8); [PR text](exact-flash-reads-pr.md) |
| Prevent timestamp collisions from overwriting backup files | `kori/10-upstream-exclusive-backups` / `9613b66ec860c2c412b099e7193d1211e0785d78` | [published candidate tree](https://github.com/kmplngj/m-flasher/tree/9613b66ec860c2c412b099e7193d1211e0785d78) | Published, open [PR #9](https://github.com/jvitkauskas/m-flasher/pull/9); [PR text](exclusive-backups-pr.md) |
| Optional minimal FMR1 reader integration | Architecture discussion before code | [Discussion draft](reader-rfc.md) | Local draft only; larger native/CLI integration stays separate |

Each new contribution is independent of pending upstream PR #7 and of the other contribution. All three PRs remain separate. Both new PR descriptions link the related fork issues using fully qualified references; those issues remain open for their broader acceptance criteria. The FMR1 architecture discussion remains a local draft, outside this publication request.

At publication verification, upstream CI for both new PRs reports `action_required`, with no jobs run or checks attached yet: [PR #8 workflow](https://github.com/jvitkauskas/m-flasher/actions/runs/38936309207), [PR #9 workflow](https://github.com/jvitkauskas/m-flasher/actions/runs/38936332451). These are not passing CI results. Local tests and prior independent reviews remain the evidence below. [Verified PR metadata](publication.json) records the exact heads/base/files and open state.

A separate GPT-6.1 Sol medium publication preflight confirmed clean worktrees, exact ancestry/file scope, alignment of descriptions with the code and review record, and absence of private device data. No code changes or redundant tests were needed for publication. Pushes were ordinary non-forced pushes; both PRs are attached to the current chat.

## Validation

Commands below are run from the corresponding candidate checkout. They use only files, injected transfers and local filesystem fixtures. The upstream application test suite has no fork adapter-denial guard, so broad Cargo tests and application execution were not used.

Exact-read candidate:

```sh
rtk proxy mkdir -p target
rtk proxy rustc --edition=2024 --forbid unsafe_code --test src/usb_data_in.rs -o target/usb-data-in-tests
rtk proxy target/usb-data-in-tests
rtk proxy cargo check --offline --locked
rtk git diff --check
```

Five tests passed; the independent Spec reviewer also compiled and reran all five. macOS compile-only check passed. The exact-count rule applies to FD05; generic short-IN replies and legacy transport recovery are outside this change.

Exclusive-backup candidate:

```sh
rtk proxy mkdir -p target/file-tests
rtk proxy rustc --edition=2024 --test --forbid=unsafe_code src/backup_files.rs -o target/file-tests/backup-files-tests
rtk proxy target/file-tests/backup-files-tests
rtk proxy cargo check --offline --locked
rtk git diff --check
```

Six tests passed, including nine injected failure boundaries; the independent Spec reviewer also compiled and reran all six. macOS compile-only check passed. This protects against overwriting existing final filenames and checks reopened bytes. It does not implement private ACL storage, hostile-directory protection, two copies or media independence. Windows directory flushing is outside portable `std`; Unix directory sync and file `sync_all` are requested without claiming physical power-loss qualification.

[Independent Standards and Spec review](review.md): zero findings for each candidate on each axis. No source changes followed the reviews, only commits and worktree relocation. The earlier canonical fork harness passed 4 hardware-denial guards, 53 std-only fixtures and 118 Cargo tests, with 11 private-corpus tests explicitly ignored; that result is separate from these upstream standalone tests.

No device inventory, opening, commands, firmware execution, target builds, flash/status-register writes, new physical tests or release occurred during this work. Continue under the main repository's offline-development instructions. Historical device evidence remains pinned to its recorded host revision; these new upstream candidates have no physical qualification.
