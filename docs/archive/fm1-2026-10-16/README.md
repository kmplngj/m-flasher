# FM-1 research and upstream handoff archive

Preserved for [kmplngj/m-flasher#25](https://github.com/kmplngj/m-flasher/issues/25)
on 2026-10-16. This directory is a sanitized documentation export. The active
implementation and backlog remain in M-Flasher; archived plans, old acceptance
states and commands do not reactivate the abandoned reference implementation.

## What is preserved

- [Assessment and commit inventory](assessment.md): all 30 unpublished reference
  commits, the three working-tree documents, transferred findings, superseded
  implementation choices and current issue ownership.
- [Reference research index](specs/README.md): 53 Markdown documents, including
  the unique [rescue-source review](specs/24-rescue-script-review.md) and the
  pending backlog/index edits. The source reference is `kmplngj/fm1-flasher:init`
  at `3edc26552b0b16eb972e3aca6dc916fd96203b0c`; its remote `init` was
  `c393bd8b8f983d87818262ca849ecb95fe25d34e` at inventory time.
- [Export manifest](export-manifest.json): original committed Git blob identities
  and SHA-256/size checks for each exported text. Modified working-tree inputs
  are explicitly flagged; their export is not claimed to equal the original
  committed blob.
- [Upstream handoff](upstream/README.md): sanitized contribution descriptions,
  prior independent reviews, exact PR metadata and the FMR1 discussion draft.
  Saving that draft in this fork does not submit it to upstream.

The rescue review retains immutable public script revisions, byte lengths and
source digests, attribution and its original limitations. It is linked from
the existing [research ledger](../../fm1-roadmap.md#earlier-recovery-and-source-reuse-findings).
Its functional follow-ups stay in the existing issues; no duplicate feature
backlog is created by this archive.

## Private recovery copy and publication boundary

Before export, the complete original `init` history was saved in a private Git
bundle. Bundle verification and a fresh isolated bare restore reproduced the
exact source head. Byte-for-byte copies of the three working-tree documents,
their tracked patch and the six original handoff files were saved separately
beside a private checksum manifest. That local directory uses mode 0700 and its
files mode 0600. It is outside both repositories and is never pushed. The owner
receives its local location separately; no personal filesystem path is recorded
here. This is a local recovery copy, not independently stored media.

The public export deliberately excludes original Git history, legacy host/native
source, submodules, builds, firmware/loader binaries, private firmware corpus,
raw backups, raw-image digests, private device identifiers/selectors and detailed
private receipts. Reference `specs/evidence/*.json` and ignored analysis outputs
are not exported. Original history is retained privately rather than declared
safe for publication. Existing remote history is neither replaced nor rewritten.

All 64-character digests in reference Markdown are conservatively omitted,
except the reviewed public script/loader source digests in report 24. Local
home/temp/build artifact locations are also omitted. The manifest hashes only
the sanitized exported text; those are not flash-image hashes. Exact original
values remain in the private recovery copy. Embedded primary-source links and
Git revisions retain the research trail. References to omitted legacy source
or evidence files are historical pointers, not files supplied by this export.

## Provenance and licenses

The reference documentation came from the owner's local `kmplngj/fm1-flasher`
checkout and working-tree edits. Its full [MIT notice](REFERENCE-LICENSE.txt),
including `Copyright (c) 2026 kurogedelic`, accompanies the substantial copied
documentation. M-Flasher's existing root license and bundled-artifact notices
are unchanged. No third-party implementation or binary is imported here.

The rescue Python source is attributed to `charlesvestal/fm1-x0x` at the pinned
revision and declares GPL-3.0-only. Report 24 contains source observations and
links, not copied Python implementation. Its shell bootstrap fetches a different
repository's mutable main; do not execute it as an archival verification step.
Vendor loader/package redistribution rights and source linkage remain separate
questions; preserving research does not grant binary redistribution rights.

## Evidence limits and validation

This task performs file, Git and GitHub metadata work only. It requires no
hardware test. No inventory, device open/probe/command, application or firmware
execution, target build, backend installation or flash/status-register write is
part of archival acceptance. Hardware qualification gates remain unchanged.

The historical full-capture observation is bound to its tested CLI artifact,
reader artifact and runtime source map, matching host revision `4ce0342`.
The observation JSON's `base_commit` identifies a development base, not the exact
tested source. Later hardening at `4efefbe`, merged in fork PR #24 as `9005a0d`,
has separate offline evidence and gains no new physical qualification here.
Both historical copies used one filesystem; media independence was not proved.

Archive validation checks the exported text hashes, exact source/commit
inventory, permitted file types, links between retained documents, private-data
omissions and the final documentation-only diff. The canonical guarded host
runners supply offline regression evidence separately; their success is not
hardware qualification. Remote preservation is established by the fork branch
and PR recorded in issue #25 after publication and remote content verification.
