# Contributing

`origin` is the integration fork, `kmplngj/m-flasher`; its shared integration
branch is `main`. `upstream` is `jvitkauskas/m-flasher`, with base `main`.
The fork can progress while an upstream PR is pending.

## Select and isolate work

1. Read the issue and its acceptance criteria. Keep decisions, dependencies,
   evidence and remaining gates in that issue; PRs describe the delivered change.
   Use fully qualified cross-repository references such as
   `kmplngj/m-flasher#4` and `jvitkauskas/m-flasher#123`.
2. Inspect the working tree and preserve unrelated changes. Start a short
   `kori/<issue>-<scope>` branch from current `origin/main` for fork work.
   Use a separate worktree when isolation is needed. Avoid implementation on
   shared `main`; never force-push or rewrite it.
3. Deliver one reviewable behavior per PR. Related work may reference dependent
   PRs explicitly; do not bundle unrelated fixes, policy changes or assets.

Publication requires a human request covering the action. These instructions
alone do not authorize publishing issues, PRs or comments, pushing, merging or
releasing. Continue authorized local preparation up to a concrete reviewable
result when publication is outside the request.

## Validate and review

Read the offline boundary in `AGENTS.md` and the selected issue before running
checks. `docs/development.md` describes the existing host build and tests;
inspect the relevant test/build path for device access before execution.
Use injected fakes and saved fixtures. The only target artifact build exception
is the selected RAM reader under `ram-reader/README.md`, using existing locked
tools and a cached, network-disabled container without devices. Host-native
fake harnesses and file-parser tests validate only the stated offline behavior.

Preserve upstream firmware, bundled images, pins and notices. Physical tests
need a separate authorized task with operation-specific gates. The sole owner FM-1
never serves writer failure, recovery fault or autonomous agent cycles.

Before merge, obtain an independent review of the actual final diff and pass
its applicable checks. Review must cover acceptance criteria, regressions,
failure behavior and offline safety. Authorized subagents may provide bounded
independent review; they must not simply repeat the author's summary. Address
findings and recheck affected behavior. If reviewers or checks are unavailable,
record that gate rather than calling the PR ready to merge.

## Fork integration and upstream contributions

For a fork PR, target `kmplngj/m-flasher:main`. It may merge after independent
review and applicable checks pass, with human authorization for the merge,
even while its upstream counterpart remains pending. Record the relationship
in both repositories using qualified references; upstream acceptance is a
separate state. Preserve shared `main` and verify the remote result after an
authorized push or merge.

For an upstream candidate, create a separate short `kori/<issue>-<scope>` branch
from current `upstream/main`. Cherry-pick or reimplement only the self-contained
change; prove that its build, tests and behavior have no dependency on fork-only
commits, FM-1 features or policy. Target `jvitkauskas/m-flasher:main` and explain
the upstream user's problem with directly relevant validation. Keep fork-only
`AGENTS.md`, contribution policy, planning documents and infrastructure outside
the contribution unless upstream explicitly requests them. Upstream review or
merge does not automatically authorize a fork release.

## Evidence and issue closure

A PR records the concrete behavior change, issue references, check results,
review findings and remaining gates. Distinguish these states explicitly:
local, committed, pushed, merged, offline-validated, hardware-qualified and
released. Cite commit/artifact identities where relevant. Passing offline tests
never implies hardware qualification; merging never implies a release.

Close an issue only when its acceptance criteria are satisfied, or keep the
remaining scope in an explicitly linked issue agreed by the maintainer. Use
issues and small PRs as the primary workflow. Add labels or issue templates only
when they improve actual routing; no extra status system is required.
