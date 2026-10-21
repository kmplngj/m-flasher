> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Agent-driven development firmware validation

Goal recorded: 2026-10-11. **Future physical workflow remains a design goal.** Implementation tickets are [FM1-024–026](04-backlog.md#fm1-024--agent-automation-contract-and-offline-lifecycle-harness); FM1-024's offline harness is described separately in the [implemented contract](10-offline-agent-contract.md). This document authorizes no hardware access, firmware builds or flash writes.

## Intended outcome and current boundary

An AI coding agent should eventually be able to submit a development firmware artifact to the CLI, obtain a validated plan, install it on an explicitly enrolled test unit, request a controlled boot, and check whether the expected build responds and passes declared smoke tests. It should use the same enforced policy as a human caller, consume structured results and stop when evidence is missing.

The current offline parser, immutable plans, backup checks, bounded simulated writers and reports are a foundation. **The current CLI has no native flash/boot/health workflow.** Its NOR/boot models do not execute firmware or prove hardware behavior. Proposed contracts below are not commands available today.

The owner's only FM-1 stays outside this development loop under the [single-device policy](03-single-device-safety.md). Offline fakes can be implemented first. Physical cycles require a spare/external bench, qualified critical operations and a separately authorized hardware task. If that bench is unavailable, this goal remains offline. A backup and recovery dongle do not make experimental firmware safe for the owner unit.

Normal USB-C installation remains the preferred route, without a compulsory RP2040 or V15 detour. Automation cannot make an unqualified USB-MIDI backup/checkpoint available. A dark device may still require independently qualified external entry. This CLI extension does not depend on the browser: the web frontend remains **step 2** in the [main plan](02-cli-and-web-plan.md).

## Bounded development cycle

1. **Produce and bind a candidate.** A future separately scoped firmware task uses a reviewed build recipe. Its immutable manifest records source commit, toolchain/configuration, original package and raw application hashes, layout/data schema, runtime storage effects and expected embedded build ID. The manifest maps that ID to the final artifact; a binary need not embed its own final hash. No firmware build runs during present offline host development.
2. **Select one authorized bench.** Bind an explicit unit/session, OS/transport and already qualified loader, write, verification, controlled-boot and recovery recipes. Require exclusive job ownership. Names, VID/PID or shared firmware numbers are insufficient; never probe all instruments to find a target.
3. **Plan before writing.** Inspect original bytes and simulate an immutable application-region plan. Account for every real loader/staging side effect, preserved region and data-schema edge. Refuse unknown layouts, incompatible data, missing recovery or missing raw-readback capability. Never patch identities, wipe presets or install V15 automatically.
4. **Check fresh backup and authority.** Require two identical full raw reads, reopened/hash-verified durable copies and a current-source binding before any erase. An old snapshot or selective preset export cannot substitute. Match the job/candidate to the separate limited bench authorization; do not infer permission from an agent prompt or saved journal.
5. **Execute once within bounds.** Write only approved differing regions through the qualified executor, with finite deadlines and the existing bounded retry policy. Verify sectors and the complete expected raw image in the held session. Transfer completion is not sufficient to proceed.
6. **Request a separate controlled boot.** Permit only the qualified boot action explicitly requested for the same fully verified job/device/image/session. This can be a predeclared stage in an authorized job, but must not be a hidden side effect of flashing. Reconnect only to the enrolled endpoint with fresh identity evidence. Ambiguity, failed verification or a lost session stops progression.
7. **Confirm the expected runtime.** Send a fresh job-bound challenge to a proposed qualified firmware health endpoint. Require the expected build ID, challenge response and bounded heartbeat/uptime observations. A stale response, enumeration or matching version string cannot pass. Existing firmware without this instrumentation reports boot unconfirmed/inconclusive.
8. **Run declared checks and report.** Read-only smoke checks state their observable criteria and timeouts. Report each result and supporting evidence, then end the job. The agent may use that report to propose the next source change; every new artifact/run requires fresh planning, backup and applicable authorization checks.

Firmware hooks in step 7 are a future firmware integration task, not evidence already present in stock/custom firmware. A MIDI response can demonstrate that a build is responding through that interface. Audio generation, DSP correctness, controls and display behavior need separate enrolled measurement fixtures and explicit test criteria. Checks cannot silently write settings/presets or mark unavailable measurements as passing.

## Proposed automation interface

Use the CLI as a noninteractive subprocess interface first; an agent-specific SDK or MCP server is optional later. Keep raw hardware commands inaccessible to callers. The CLI owns validation, job ownership, bounds, qualification checks and result semantics.

| Contract | Required behavior |
|---|---|
| Candidate / plan | Immutable hashes and exact device, profile, layout, data, OS/transport, backup and recovery bindings; simulated/physical evidence is explicit |
| Job request | Versioned schema, unique job ID, explicit requested stages and bench authorization reference; finite stage deadlines and job/attempt budgets |
| Progress | JSON/NDJSON events with schema version, job/stage/state and verification level; machine output on stdout, diagnostics on stderr |
| Result | Typed success/refusal/failure/inconclusive states and documented exit codes; separate raw-verification, boot and per-smoke outcomes |
| Evidence | Build/plan/artifact hashes, actual stage, authorized target, observation timestamps and bounded sanitized logs; no private raw backup/preset payload in support reports |
| Cancellation / restart | No new destructive action after cancellation or authorization expiry; follow a qualified abort/held-session procedure. A prior report/journal cannot authorize resume |

The final result must preserve these distinctions:

| Evidence | What it establishes |
|---|---|
| Transfer complete | Requested bytes were served/sent; it does not prove flash contents or boot |
| Model verification | Fictional bytes/state satisfied the offline model; no physical execution claim |
| Actual full preboot raw verification | This job's physical image matched the expected complete bytes before boot |
| Boot confirmed | The enrolled device freshly responded as the expected build after the requested boot |
| Smoke check passed / failed / unavailable | Only the declared function and measurement criterion were evaluated |

Overall success requires the requested evidence levels and all required checks. A raw-verified job with no health reply remains boot-unconfirmed; a responsive build that fails an audio check remains a failed functional check. Neither can be reduced to “firmware works.”

## Authority and experimental qualification

Production qualification binds exact source/target artifacts. It would therefore reject an unqualified new development image. Do not solve that by disabling checks. Create a **separate external-bench experimental application capability**, gated by FM1-026, while preserving exact-image requirements for public/owner production writes.

That capability requires reviewed critical hardware/layout, loaders, bounded writer, backup/preboot checkpoint, boot procedure and independent recovery. Each experimental application is still inspected, hash-bound and checked for declared data/layout effects; its unknown application behavior is what the spare bench evaluates. Changes to flash drivers, boot design, OTA loaders or protected-region behavior fall outside the initial experimental application scope and need separate qualification. A passing bench run does not automatically enable a production profile.

A human-controlled bench authorization fixes the device, project/build policy, OS/transport, permitted ranges/loaders/data effects, expiry, maximum jobs and attempt limits. Initially it may approve one candidate; later it may grant a bounded iterative bench session. The agent cannot issue/extend this authority, change qualifications, widen bounds or turn a failure into an implicit rollback/recovery permission.

Enforce this through a reviewed executor and policy outside the agent-editable checkout, with appropriate OS permissions. An agent that can rewrite its executor or credentials can bypass a same-process check; the existing offline test guard is a regression guard, not this isolation mechanism. Offline authorization fixtures are only fictional evidence.

## Failure and recovery limits

On failed verification, crash, wrong identity, stale response, timeout or lost session, stop advancement and retain evidence. Preserve a reachable held recovery session where the qualified procedure permits. Cancellation or grant expiry must not trigger an unreviewed reset or unsafe immediate handle close during programming. Do not run a blind power-cycle, full reflash, target switch, preset wipe or rollback loop.

Recovery is a separate exact qualified plan with fresh reachable-state evidence and explicit scope. A previous backup is a candidate recovery artifact, not proof of the current device state or a bootable restore. No full-chip/head restore is added. If the application loses USB, cable-only recovery cannot be assumed; an independently qualified entry source may be required. Hardware/head faults remain outside the application-only unbrick claim.

Host write bounds do not sandbox firmware after it starts executing. A development build may hang, corrupt its permitted runtime data, misuse flash or lose USB. Single-bank interruptions can also leave an unbootable application. This is why physical agent experiments require an external bench with accepted residual risk, rather than a promise that an AI-driven loop can never brick the only unit.

## Delivery milestones

1. **FM1-024: offline contract and lifecycle harness.** Exercise structured stages, authority fakes, explicit boot intent, deadlines, stale/wrong evidence, concurrency and failure stops under the no-hardware guard.
2. **FM1-025: build identity and health protocol.** Specify reviewed manifests and fresh read-only runtime replies; use fictional transcripts first. Firmware edits/builds and actual hooks are separately scoped.
3. **FM1-026: external bench qualification.** Only after critical capabilities qualify and a spare/external hardware task is authorized, collect actual flash/boot/health and bounded failure/recovery evidence per OS/transport. Keep the owner unit excluded.

These milestones extend the CLI goal; they neither replace the existing qualification backlog nor move the browser ahead of step 1. Documented future automation is not hardware authorization or a currently implemented command.
