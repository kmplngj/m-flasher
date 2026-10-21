> Archived 2026-10-16 from the local upstream-preparation handoff. Historical statements and test/review counts below are preserved reports, not checks rerun for this archive. Sanitization and current-state verification are documented in [README.md](README.md).

# Prevent timestamp collisions from overwriting backup files

Saving twice within one second currently overwrites the previous raw backup and JSON sidecar. Reserve both filenames with exclusive creation before writing either file, and return a contextual error if either name exists. This also prevents a leftover old sidecar from being paired with newly written raw bytes after an error.

The helper syncs both files and verifies their exact bytes after reopening with bounded reads. Unix also syncs the containing directory. Failures retain newly created partial files for diagnosis; subsequent calls cannot overwrite them. Filenames, directory layout, sudo-user ownership handling and the rule that `backup_path` changes only after success remain unchanged.

Six standalone filesystem tests pass, covering repeat timestamps, either preexisting counterpart, an existing Unix symlink, partial writes, all nine I/O failure boundaries and reopened short/changed/extra bytes. macOS `cargo check --offline --locked` passes. Independent agent Standards and Spec reviews report zero findings; the Spec reviewer separately reran all six tests. No hardware or application execution was used.

This is ordinary local-file persistence, not ACL/private storage, hostile-directory protection or independently stored copies. Windows requests file flushing; portable `std` directory sync is Unix-only. These checks do not establish physical power-loss durability on every OS/filesystem.

Based on upstream `main` at `9d962747`; commit `9613b66ec860c2c412b099e7193d1211e0785d78`. No fork-only dependencies. Related scope: kmplngj/m-flasher#10; private two-copy storage and independent-media qualification remain separate work.

Validation from this branch:

```sh
rtk proxy mkdir -p target/file-tests
rtk proxy rustc --edition=2024 --test --forbid=unsafe_code src/backup_files.rs -o target/file-tests/backup-files-tests
rtk proxy target/file-tests/backup-files-tests
rtk proxy cargo check --offline --locked
rtk git diff --check
```
