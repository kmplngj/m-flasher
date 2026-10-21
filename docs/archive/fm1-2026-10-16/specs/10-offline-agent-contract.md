> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# FM1-024 offline agent lifecycle contract

Date: 2026-10-13. Repository: [kmplngj/fm1-flasher](https://github.com/kmplngj/fm1-flasher), branch `init`. This is an offline host implementation; it does not grant hardware access, firmware builds or real writes. Read [safety policy 03](03-single-device-safety.md) and the [future physical goal](08-agent-firmware-validation.md).

## Entry points

The CLI exposes `agent contract --json` and `agent simulate --demo` only. It accepts no native device path, real firmware/job file, raw command, force switch or authority payload. `--json` returns one final object; `--events` returns ordered JSON records followed by the final result. Human diagnostics do not prefix machine stdout. Use the host recipes (historical reference `../host/README.md#offline-agent-lifecycle-fm1-024`; omitted from public archive).

The library provides strict immutable `AgentJob`/`AgentBinding`/`SmokeCheck` records, separately injected `FakeBenchAuthority`, `AgentLeases`, `FakeAgentClock`, `FakeHealthEndpoint` and `AgentCoordinator`. `make_agent_fixture` creates actual durable local copies of fictional bytes; `run_agent_demo` selects declared scenarios. Existing parser/planner, backup persistence and `SimulatedTransporterWriter` remain the enforcement basis. No upstream writer or native backend is imported or executed.

## Binding and authority

A job binds original package and APP hashes, catalog/source/target/edge/loader hashes, plan and full expected image, current snapshot, backup manifest, explicit fictional device/session, OS/version/transport, expected build ID, boot intent, finite deadlines, one job attempt and declared read-only smoke checks. Strict decoding refuses unknown fields, duplicates, null/wrong record types, invalid booleans, unsupported schema and nonfinite values. Reports hash device/session/OS-version fields and omit backup paths and raw personal bytes.

Job JSON can reference a fictional grant but cannot create one. The separately injected exact grant binds the complete request; cancellation and expiry can narrow it. Shared thread-safe in-process ownership rejects concurrent target claims and consumed job IDs. Release uses captured owner identity/token, so changed job fields cannot release another target. This is not an external security service or multi-process hardware lock; no production/physical capability is available.

The optional exact `AgentWritePermit` on the existing simulated writer is checked after slow backup durability work, immediately before every erase/page and bounded sector retry. It rechecks the grant, job/plan/reader/model references and held loader/session. It cannot widen the APP gate or bypass either durable copy. The default non-agent simulated writer behavior is preserved.

## Evidence and outcomes

| Stage | Evidence / limit |
|---|---|
| Plan | Original inputs recomputed against the exact immutable fictional plan |
| Backup gate | Two equal fresh fake reads, actual local files, durable/reopened copies and current binding |
| Transfer | Existing bounded NOR model operations; complete sector transfer alone is insufficient |
| Preboot verification | Full expected model bytes read back, including untouched head/data/tail |
| Explicit boot | Only if requested and verified references/session still match after the final reader check; cold model classification executes no CPU instructions |
| Health | Fresh job-bound nonce and declared fictional build/image/device/session reply; missing instrumentation is unconfirmed |
| Smoke | Each required declared fictional function is passed, failed or unavailable; no actual audio/UI/MIDI function is qualified |
| Report | Sanitized ordered events and typed result; physical/hardware/CPU/write flags remain false |

| State | Exit code | Meaning |
|---|---|---|
| `succeeded` | 0 | Requested model/fictional stages passed only |
| `refused` | 2 | Preconditions or local persistence unavailable; no accepted write cycle |
| `failed` | 3 | Verification, runtime binding or required smoke failed; no retry/rollback permission |
| `inconclusive` | 4 | Missing health response or unavailable required measurement |
| `cancelled` | 5 | Fictional authorization cancellation stopped progression |

Scenarios cover successful requested stages, transfer-only/no boot, absent/stale/wrong health, ambiguous reconnect, pre-run and mid-write cancellation/expiry, writer failure, full verification failure and failed/unavailable smoke checks. Finite stage/job deadlines stop advancement. Failure never triggers automatic power-off, target switch, reboot, full-job retry, rollback or journal-based resume. A modeled retained loader is distinguished from a usable held host handle.

## Persistence, platform and next-ticket limits

Default CLI demos remove their private temporary copies after execution and label backup validation `at-run-only`. Explicit paired directories retain fictional artifacts without overwriting existing files; retained reports still cannot authorize a later run. Actual Windows secure file operations remain gated, even though pure contract/record checks are portable. Saved OS labels on macOS/Linux are fictional profile evidence, not native OS/device qualification.

FM1-025 still owns actual build-manifest/runtime protocol integration. FM1-026 still requires a separately authorized spare/external bench and qualified critical capabilities. The owner's only FM-1 is excluded from development trials. Browser tooling remains step 2.

## Validation evidence

The independent focused review reproduced and fixed cancellation after the final durability check, stale/late target replacement before boot, mutable lease release, null binding decoding and private-field leakage. Dedicated guarded tests also exercise a real two-thread lease race, NDJSON framing, CLI refusals and the actual secure-persistence gate without OS impersonation.

The canonical full suite and current artifact evidence are recorded in [implementation progress](05-implementation-progress.md). Earlier report 07's wheel hashes describe the earlier source snapshot; new host modules and corrected attribution require fresh builds. Hosted CI is reported separately from local checks and neither constitutes hardware qualification.
