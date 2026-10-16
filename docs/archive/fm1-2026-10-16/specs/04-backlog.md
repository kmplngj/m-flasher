> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# FM-1 flasher backlog

Date: 2026-10-11. These are **local planning tickets**, not issues already created on GitHub/Jira. Recommended base: an [FM-1 Transporter fork](02-cli-and-web-plan.md). All tickets inherit the [single-device safety policy](03-single-device-safety.md).

## Ordering and status

Work from small offline components toward integrated writers. The owner chose the personal fork and authorized sequential offline implementation; current outcomes are recorded in [implementation progress](05-implementation-progress.md). **Offline** tickets become executable when their dependencies finish. **Hardware gate** means concrete missing physical evidence/access, not an implementation task that should be guessed. On 2026-10-15 the owner authorized hardware access for backup work, with **no flash writes**. Development/tests remain offline; an actual read session still needs an evidenced read path and exact target. FM1-012 kit/entry evidence is still incomplete. **Step 2** waits for a qualified CLI release. The owner's normal-install priority is USB-C-only; the RP2040 remains an emergency fallback. USB ROM research follows its offline prerequisites and does not block basic offline inspection/simulation.

Effort is an implementation estimate, not a delivery promise: S = about 1–2 engineer-days; M = 3–5; L = 6–10. Hardware and external qualification wait time is unestimated.

| ID | Ticket | Stage/status | Depends on | Size |
|---|---|---|---|---|
| FM1-001 | Establish Transporter fork and isolated host project | Complete offline | Fork destination chosen | S |
| FM1-002 | Build pinned provenance and transition profile schema | Complete offline | 001 | S |
| FM1-003 | Parse packages and extract correct raw flash regions | Implemented offline; real fixture gated | 002 | M |
| FM1-004 | Harden parser with independent and malformed fixtures | Implemented offline; real corpus gated | 003 | S |
| FM1-005 | Implement pure device-pull SysEx codec and replay | Complete offline | 002 | S |
| FM1-006 | Model NOR flash, boot modes and interruption faults | Complete offline | 002 | M |
| FM1-007 | Implement immutable plans and enforced safety policy | Complete offline | 004, 006 | M |
| FM1-008 | Deliver offline inspection, planning and simulation CLI | Implemented offline; Windows file IO gated | 005, 007 | S |
| FM1-009 | Integrate explicit Transporter serial discovery/reads | Complete offline; native inventory/reads gated | 005, 007 | M |
| FM1-010 | Build durable verified backups and job records | Complete offline; physical/Windows durability gated | 009 | M |
| FM1-011 | Diagnose saved dumps and route recovery states | Complete offline | 004, 008, 010 | S |
| FM1-012 | Commission owner recovery kit and read-only backup | Hardware gate | 011, kit and separate scope | S + gate |
| FM1-013 | Implement bounded bridge writer against simulations | Complete offline; real writes disabled | 006, 007, 010 | M |
| FM1-014 | Implement MIDI updater against virtual devices | Complete offline; real MIDI gated | 005, 007, 010 | M |
| FM1-015 | Implement direct transitions and bounded rollback | Complete offline; real routes gated | 011, 013, 014 | M |
| FM1-016 | Package and test macOS/Linux; qualify Windows adapters | Implemented offline; other OS/native evidence gated | 008, 009, 013, 014 | M |
| FM1-017 | Qualify recovery/writes/transitions away from owner unit | Hardware gate | 004–016, spare/external access | L + gate |
| FM1-018 | Release selectively enabled CLI capabilities | Release gate | 012, 016, 017 | S |
| FM1-019 | Assess direct ROM SCSI/BOT platform alternatives | Complete offline; native access gated | 002, 011 | M |
| FM1-020 | Decide browser feasibility and shared contracts | Step 2 | 018 | S |
| FM1-021 | Deliver browser offline/backup frontend | Step 2 | 020 | M |
| FM1-022 | Qualify and enable browser installation/recovery | Step 2 hardware gate | 017, 021 | L + gate |
| FM1-023 | Back up old firmware with/without presets, or presets only | Planned; format qualification gated | 010, 011, 015; exact data formats | M |
| FM1-024 | Define agent automation contract and offline lifecycle harness | Complete offline; physical workflow gated | 007, 008, 010, 011, 013–016 | M |
| FM1-025 | Define development build identity and firmware health checks | Planned; firmware integration gated | 024; reviewed firmware project/toolchain | M |
| FM1-026 | Qualify agent flash/boot/check cycles on an external bench | Hardware gate | 017, 018, 024, 025; spare and separate scope | L + gate |
| FM1-027 | Establish a small native Rust CLI foundation | Complete offline; full core port pending | 024; owner Rust decision | S |
| FM1-028 | Port validated package parsing and wire codecs to Rust | Complete offline; real corpus/profile/IO gates | 027; 003–005 reference evidence | M |
| FM1-029 | Port profile resolution and transition planning to Rust | Complete offline; native persistence/writer gates | 028; 002, 007, 015 reference evidence | M |
| FM1-030 | Port durable backups and bounded fake writer to Rust | Complete offline; hosted/OS/release/hardware gates | 029; 006, 009–011, 013–015 reference evidence | L |
| FM1-031 | Port the offline agent lifecycle to Rust | Planned offline | 030; 024 reference contract | M |
| FM1-032 | Qualify native CLI artifacts and release packaging | Planned offline; signing decisions pending | 028–031; 016 reference evidence | M |
| FM1-033 | Standalone full raw dumper over direct USB-C, including unchanged stock before installation | A–C/D1/F1 complete offline; D2 source audit and injected driver/capture complete offline; qualification/E/F2 gated | 019, 030; stock protocol/entry evidence | L + gate |

Dependency notation such as `004–016` means the relevant completed offline evidence from those tickets, excluding a need to perform owner-device writes. FM1-012 only permits bounded reads; FM1-017 does not use the owner's only FM-1. Begin device-independent development while the kit is being obtained.

## Common definition of done

On 2026-10-13 the owner selected Rust for the eventual distributed CLI. After completing FM1-024's Python offline reference, FM1-027 establishes a native vertical slice before real adapters. It does not port or qualify a physical writer; existing Python models and independent fixtures remain the reference. Other format, firmware-integration and hardware gates stay in force.

Every implementation ticket has a narrow reviewed change, meaningful tests of its failure modes, updated support/evidence documentation and reproducible offline commands. No tests open the owner's hardware. Source/artifact hashes and licenses are pinned, unexpected permissions/network activity are absent, and no unrelated parent-repository files are changed. A hardware claim includes the actual unit/revision, OS, transport, image/profile hashes, test steps and outcome; simulated evidence is labeled simulated. No public release enables an unknown profile or an override of protected ranges.

## FM1-001 — Establish Transporter fork and isolated host project

**Problem/outcome:** the recovery foundation exists, but this empty folder currently resolves Git operations to the user's home repository. Establish a dedicated fork/checkout with upstream history and a host development boundary.

**Scope:** retain upstream firmware/PIO files and pins unchanged; add the specs, host package structure, fake-transport interfaces and offline CI. Record the fork/upstream remotes and base commit. Choose a fork destination before creating a remote; creating one is separate from this research.

**Acceptance criteria:**

- Git top-level is the flasher checkout, not the home directory; only project files are tracked.
- Upstream base is `a632d923203170e3a08565f4056dbb3fd3d65ca5`; MIT notices and submodule attribution remain intact.
- Running host tests cannot import/open real MIDI/serial/SCSI backends; a guard test proves that boundary.
- The default build exposes no real flash-write capability. No FM-1 command or firmware build script runs during bootstrap.

**Validation:** checkout/root checks, offline CI smoke test, scope diff. **Dependency:** owner-selected fork destination. **Size:** S.

## FM1-002 — Pinned provenance catalog and transition profile schema

**Problem/outcome:** versions, layouts and identities overlap across forks. Make support a reviewed exact profile rather than a family-name heuristic.

**Scope:** versioned schema for SoC/revision, capacity/JEDEC/key, package/loader/stub hashes, address maps, source INFO evidence, data schemas, transport capabilities, forbidden ranges and qualification records. Preserve evidence tiers and blocked reasons. Start all write profiles disabled.

**Acceptance criteria:**

- Initial candidate profiles distinguish V13/V14 boundaries and expanded FM-1+VA layouts from V15's `[0x4000,0x93000)` bridge boundary; an expanded layout cannot weaken the board gate.
- Shared `FM-1_9xx` identity is insufficient, and numbers below 100 do not prove stock (`FM-1_092` is a counterexample); missing INFO/snapshot evidence resolves to unknown/refused.
- Official V15 file/region hashes and pinned WL82 loader source are recorded; downloading is explicit, verified and offline-capable thereafter.
- Licensing catalog distinguishes MIT recovery code, GPL-only Felucca tools, WTFPL RE code and separately sourced vendor binaries. No uncertain binary is bundled.
- Evidence status is per exact edge/OS/transport and cannot be upgraded by a version comparison alone.

**Validation:** valid/invalid schema fixtures, deterministic serialization and disabled-profile tests. **Depends:** 001. **Size:** S.

## FM1-003 — Validated package parser and raw-region extractor

**Problem/outcome:** historical wrong-offset extraction could write a correct-length but incorrect image. Produce the exact physical application bytes from a validated `.fwsc`.

**Scope:** separate package/logical/flash/XIP offset types; marker removal; bounded UFW entry parsing; CRC/hash/header checks; raw flash-region extraction and inspection. Do not execute scripts embedded in packages.

**Acceptance criteria:**

- Official V15 produces the exact known package/application-region hashes and `[0x4000,0x93000)` bytes.
- Regression fixture fails for the historical logical-`0x4000` slice; directory-derived extraction succeeds.
- Reject truncated, overlapping, overflowed or out-of-capacity entries, bad CRCs, wrong SoC and unknown layouts before transport creation.
- Distinguish plaintext `app.bin` from raw encrypted JLFS flash bytes; reject the wrong representation as a flash input.
- Preserve the original package and generate no modified identity/version.

**Validation:** real lawfully obtained local fixture + synthetic fixtures; independently computed oracle. **Depends:** 002. **Size:** M.

## FM1-004 — Independent parser checks and malformed-image corpus

**Problem/outcome:** tests that repeat the same extractor bug are weak evidence. Verify package interpretation with an independent implementation/corpus.

**Acceptance criteria:**

- Compare extraction/CRC/address results with independent upstream tools or separately written test oracles without executing a writer.
- Include stock, exact selected custom and malformed variants; provenance/hash/legal-use metadata accompanies every fixture.
- Bounded fuzz/property tests cover arithmetic, entry counts, lengths, marker handling and wrong-type files; every failure reproduces from a seed/fixture.
- Unknown/bad packages produce zero hardware calls, including during errors and cleanup.

**Validation:** corpus report and fuzz seeds in CI. **Depends:** 003. **Size:** S.

## FM1-005 — Device-pull MIDI codec and transcript replay

**Problem/outcome:** the wrong host-push model and an off-by-one final response previously broke updates. Match the evidenced wire bytes.

**Scope:** pure request/identity parsing, LSB-first 8→7 packing, response framing/checksum, exact range serving and stage completion rules.

**Acceptance criteria:**

- Golden responses match independently obtained vendor/AL-255 captures byte for byte, including the 481-byte final request and zero-padded stock identity.
- Round trips cover all byte values and lengths around 7/8/512 boundaries; reject corrupt/noncanonical framing according to the reviewed profile.
- Duplicate/out-of-order valid requests are served correctly; invalid ranges/lengths/flags are refused; stale completion does not mark success.
- Replay is deterministic with no clocks/devices/network required. No generic “send arbitrary SysEx” product operation exists.

**Validation:** golden transcripts, negative requests, independent codec cross-check. **Depends:** 002. **Size:** S.

## FM1-006 — NOR flash and boot/recovery fault model

**Problem/outcome:** a mock that always enters recovery after a failure hides the observed dark/no-USB state. Simulate actual destructive semantics.

**Scope:** 1 MiB raw flash, 4 KiB erase, page programming, 1→0 behavior, volatile/persistent state, stock/app/OTA/ROM/no-USB modes, external bridge entry and injectable transport/host/storage failures.

**Acceptance criteria:**

- Model partial erase/program and preserve flash across virtual power loss; reject illegal NOR operations.
- Reproduce mixed new/old sectors with intact head and loader, yet no automatic cold OTA resume.
- Distinguish firmware-dependent rescue from external USB_KEY; model unsupported/damaged identity separately.
- Enumerate destructive cut points deterministically, record resulting protected bytes and recovery availability, and replay every failure.
- Synthetic stub scenarios retain the first/changed-stub unprotected window; never simplify it away.

**Validation:** incident-shaped fixtures, invariants across all cut points and generated traces. **Depends:** 002. **Size:** M.

## FM1-007 — Immutable plans and safety policy

**Problem/outcome:** validated bytes need a non-bypassable execution contract. Compute and reject operations offline before any write adapter exists.

**Acceptance criteria:**

- Plan binds exact package/profile/backup/device evidence and includes transport, all affected ranges, expected full-image hash and rollback candidate.
- Always refuse boot-head writes, chip/block erase, unknown layouts/edges, incompatible data, ambiguous identity and missing stable backup/recovery evidence.
- Candidate writes are only differing aligned app sectors within the exact profile and bridge bounds; protected head/data remain unchanged in every generated result.
- Changed input/hash/device or failed durable backup invalidates the plan. No `--force`/`--yes` can bypass safety.
- MIDI plans additionally bind audited source staging code/target loader and their real physical write sets; stock OTA rollback cannot satisfy head-never-written.

**Validation:** policy decision matrix, tampered-plan/input tests, zero destructive-call assertions. **Depends:** 004, 006. **Size:** M.

## FM1-008 — Offline inspect/plan/simulate CLI

**Problem/outcome:** ship useful inspectable functionality while hardware writes are blocked.

**Acceptance criteria:**

- `package inspect`, `plan`, `simulate` and saved-report commands operate with all hardware libraries absent.
- Show exact identity/hash/layout, direct-edge decision, affected regions, missing evidence and failure stage in human-readable and JSON outputs.
- Exit codes distinguish bad package, unknown/unsupported transition, policy refusal and simulated execution failure.
- Development help/output explicitly states that real writes are unavailable. No auto-download or probing side effect.

**Validation:** end-to-end CLI snapshots on valid/invalid/offline fixtures. **Depends:** 005, 007. **Size:** S.

## FM1-009 — Explicit serial adapter and Transporter reads

**Problem/outcome:** upstream macOS globbing and generic port probing do not establish safe cross-platform selection. Implement a narrowly scoped adapter.

**Acceptance criteria:**

- Use OS serial inventory on macOS/Linux/Windows with explicit user-selected port/Transporter identity; distinguish data CDC from console CDC.
- Never issue `ping`, reset/touch or other commands to arbitrary serial devices; ambiguous selection is refused.
- Parse data length, CRC/trailer and timeout strictly; no short/corrupt read is accepted as a dump.
- Reconnect invalidates old handles/identity; external-mode changes are reported. No erase/write/RAM-run operation is reachable from the read adapter.
- Adapter tests use fake ports and recorded chunks only; actual physical reads belong to 012/017.

**Validation:** platform inventory fixtures, fragmented stream/CRC/timeout/busy/reconnect tests. **Depends:** 005, 007. **Size:** M.

## FM1-010 — Verified backup persistence and job records

**Problem/outcome:** a backup has to survive host crashes and be recoverable before a destructive phase starts.

**Acceptance criteria:**

- Require two equal 1 MiB reads; write immutable snapshot + device/profile/tool/transport/time/hash metadata, flush durably, reopen and verify.
- Keep original backups, detect path collisions/disk full/fsync/permissions/corruption, and record a verified second copy before enabling writes.
- Journal stages distinguish planned, backup accepted, entering loader, writing, raw verified, rebooted/identity checked and failed/unknown.
- Killing the host at each persistence boundary never creates a valid-looking incomplete backup or permits an unbacked write.
- Journal presence alone never authorizes auto-resume or implies the device reached a stage.

**Validation:** storage/crash injection and restore inspection of persisted fixtures. **Depends:** 009. **Size:** M.

## FM1-011 — Offline dump analysis and recovery routing

**Problem/outcome:** differentiate a corrupt app, incompatible data, an intact app and an unreachable USB device without guessing destructive fixes.

**Acceptance criteria:**

- Analyse head/application/data hashes, per-sector package matches, staged loader and known metadata using exact profile layouts.
- Classify evidence as working MIDI, OTA reachable, ROM disk present, no USB or unknown; black screen alone does not imply ROM.
- A byte-correct app with incompatible/unknown data never triggers an automatic rewrite or erase.
- Emit a concrete supported next route, prerequisites and limits. A bricked snapshot is not labeled a known-good rollback.
- Sanitized report export omits raw personal preset/project data unless deliberately selected.

**Validation:** incident-shaped and incompatible-data fixtures; wrong-head and unknown-loader negative cases. **Depends:** 004, 008, 010. **Size:** S.

## FM1-012 — Owner kit commissioning and first full backup

**Status: HARDWARE GATE.** Missing: standard XIAO RP2040 kit, exact FM-1 identity and a separately requested bounded hardware session. No writes to the FM-1 are allowed by this ticket.

**Acceptance criteria:**

- Prepare a pinned known-good dongle image with the FM-1 disconnected; retain its provenance/hash and independently check adapter signal routing and VBUS isolation.
- Record installed firmware/USB identity and exact chip/flash identity using reviewed queries only.
- Demonstrate external ROM entry and two equal full reads; durably save metadata and a second copy.
- Exit through the documented read workflow and verify the original device still boots, without installing or erasing anything.
- Mark entry/read/normal-return as tested; write/restore remains explicitly unproven.

**Validation:** bounded session record and hashes; no erase/program calls in captured transcript. **Depends:** 011 + hardware gate. **Size:** S, procurement unestimated.

## FM1-013 — Bounded Transporter writer in simulation

**Problem/outcome:** implement the writer without using the owner's device as a development fixture.

**Acceptance criteria:**

- Consume only validated plans and durable backup records; re-read/compare the selected live-model snapshot before the first erase.
- Write differing 4 KiB app sectors only, verify every sector and then the full expected image before declaring raw verification complete.
- Enforce operation bounds independently at host entry points/retries; keep the original bridge firmware gate unchanged.
- Retry at most one failed sector after valid readback/session checks; persistent mismatch remains a failure with evidence and recovery guidance.
- All-cut-point simulations preserve the head/untouched data and yield recoverable or explicitly blocked states; no hardware adapter is enabled by this ticket.

**Validation:** exhaustive interruption and corruption/dead-cell tests, wrong-device/changed-ref tests. **Depends:** 006, 007, 010. **Size:** M.

## FM1-014 — MIDI updater in virtual-device harness

**Problem/outcome:** unify normal→OTA→installed workflows without assuming the app survives every failure.

**Owner priority:** the normal installation path uses only a USB-C data cable, with no RP2040 or hidden firmware detour. The RP2040 route is a separate emergency fallback. Deliver a concrete USB-only capability decision under the existing safety contract; missing preboot readback/backup evidence remains a gate.

**Acceptance criteria:**

- Model stage-specific requests, loader re-enumeration, explicitly paired MIDI ports, busy/permission/queue failures and bounded timeouts.
- A port name/9xx identity never chooses another synth; reconnect only accepts the bound candidate with fresh reviewed evidence.
- Load/hash bytes before transfer; serve only requested ranges from the immutable image; do not inflate highest-offset/bytes-served progress into flash verification.
- Simulate source staging/target loader write sets; unknown loader/head behavior blocks the transition before upgrade entry.
- Model all staged loader/record/data effects and require an evidenced preboot raw-readback checkpoint for any future enabled MIDI writer. Existing ordinary OTA lacks that demonstrated checkpoint, so this ticket ships no real MIDI write capability.
- Resume only qualified exact image/session cases; cold stock/legacy interruptions may require bridge recovery. No blanket “replug and finish” promise.
- Real writer remains disabled pending 017; SysEx reception/backpressure and Windows buffers are adapter-specific test cases.

**Validation:** transcripts + virtual endpoints + destructive-mode model. **Depends:** 005, 007, 010. **Size:** M.

## FM1-015 — Direct transitions and bounded rollback

**Problem/outcome:** avoid mandatory firmware detours while respecting data compatibility and same-version constraints.

**Acceptance criteria:**

- Resolve one qualified direct source→target edge; prioritize USB-C-only normal installation and refuse unknown/unsafe edges without a hidden V15 install or identity bump. Do not claim universal all-to-all firmware replacement.
- Same-version different payload can select a qualified bridge route explicitly; MIDI equality refusal is reported honestly.
- Stock V15 rollback uses the bounded raw application-region bridge plan, not the official head-writing OTA loader.
- Validate target persisted-data compatibility; fixed target firmware can be eligible where exact older targets are blocked. Do not assume V15 cleans custom data.
- Raw bridge install does not claim to stage the target OTA loader; rescue capabilities depend on actual verified contents.
- Rollback from prior snapshot uses only approved regions; full-chip/head restore and automatic data erase remain absent.

**Validation:** virtual transition matrix including Felucca 0.9 ORGAN bug, shared identities, Sloop/ChoralRoot unknown edge and safe refusal. **Depends:** 011, 013, 014. **Size:** M.

## FM1-016 — Platform packaging and adapter evidence

**Problem/outcome:** distinguish executable builds from genuine device support.

**Acceptance criteria:**

- Reproducible macOS arm64/x86-64 and Linux host packages, plus Windows build/mock CI where available; state minimum supported versions based on actual build results.
- No dependency on the unavailable research repo, global pip installs, implicit runtime downloads or system-wide driver replacement.
- Keep ordinary commands unprivileged; any optional SCSI access uses a narrowly identified device/permission path.
- OS fake/virtual transport tests cover device names, dual CDC ports, SysEx reception, locks, reconnect and errors. Record native-device tests separately in 017.
- Release metadata states offline/MIDI/serial/SCSI support independently. Signing/notarization and Windows signing are explicit release tasks when selected, not silently claimed.

**Validation:** build artifacts, checksum/provenance report and per-OS software matrix. **Depends:** 008, 009, 013, 014. **Size:** M.

## FM1-017 — External physical qualification matrix

**Status: HARDWARE GATE.** Missing: spare/external FM-1 access or independently reviewable exact qualification evidence. The owner's only device is excluded from writer and failure tests.

**Scope:** qualify the exact bridge image, loader, flash identity, host OS/backends, direct edges and rollback. Arrange testing separately; this ticket does not authorize contacting other maintainers.

**Acceptance criteria:**

- Record successful backup→bounded write→full preboot verification→cold boot→rollback for every enabled edge on matching hardware.
- Test interrupted erase/page/program, host/process loss, reconnect and failed verification on the external unit; recover through independent USB_KEY and bounded restoration.
- Prove protected head and excluded data are unchanged by the writer; assess postboot data effects separately.
- Check UI/audio/MIDI and data compatibility after cold boot; a returned version string alone is insufficient.
- Any stub introduction/change, source staging change, target loader change, hardware revision or data migration creates a new qualification gate.
- Unsupported or untested OS/edge cells remain disabled. Report finite tested cases and residual power/hardware risk; never claim zero bricks from finite tests.

**Validation:** reviewable logs, snapshots/hashes, actual device outcomes and failed/recovered cases. **Depends:** relevant 004–016 evidence + external access. **Size:** L + unestimated gate.

## FM1-018 — Selectively enabled CLI release

**Status: RELEASE GATE.** If 017 is unavailable, release offline/diagnostic/read capabilities only.

**Acceptance criteria:**

- Enable write factories only for release-qualified exact profiles; unknown transitions cannot bypass the gate.
- Ship immutable artifact hashes, support matrix, recovery prerequisites, evidence and a tested rollback candidate per enabled edge.
- Owner-device development writes remain forbidden. Any later production write requires a separate owner decision with the exact reviewed plan and residual single-bank risk.
- No product claim of universal unbrick/automatic rollback; damaged hardware/head and unsupported chips/layouts are clearly outside scope.
- Offline installations work after explicit dependency/artifact preparation; uncertain vendor binary redistribution is resolved or remains user-supplied.

**Validation:** release policy tests and provenance/support review. **Depends:** 012, 016, 017 for owner-targeted write release. **Size:** S.

## FM1-019 — Direct USB ROM SCSI/BOT investigation

**Problem/outcome:** assess USB-C-only backup, bounded recovery and verification when the ROM disk is already present. The owner prefers a direct cable for normal installation; this research must identify each OS's actual capability gaps without making an RP2040 a hidden routine-install prerequisite. Dark/no-USB recovery still needs a separate validated electrical entry source.

**Acceptance criteria:**

- Inspect Linux SG_IO and Windows pass-through implementations and precise device inquiry/identity checks; arbitrary disks cannot be opened as targets.
- Assess whether a safe native macOS BOT/SCSI route is obtainable; historical unavailable patches are not treated as a working backend.
- Prototype read protocol with fake devices first and classify OS permission/kernel-driver conflicts.
- Deliver a go/no-go report, source/license inventory and separate follow-on tickets. No driver uninstall, protection bypass or FM-1 write is part of this research.

**Validation:** primary-source report and mock protocol evidence. **Depends:** 002, 011. **Size:** M, optional.

**Delivered offline:** [platform feasibility and follow-on proposals](06-usb-rom-feasibility.md). All native capabilities remain disabled; the read prototype accepts only fictional endpoints.

## FM1-020 — Step 2 browser feasibility and shared contracts

**Problem/outcome:** a working MIDI installer does not establish browser access to a dark ROM device.

**Acceptance criteria:**

- Evaluate desktop Web MIDI/SysEx, Web Serial→Transporter and actual browser/OS feature support with virtual endpoints first.
- Normal-site WebUSB mass-storage/SCSI access is excluded; no browser-protection bypass.
- Decide shared versioned manifests/fixtures and TypeScript vs WASM scope based on a measured spike; do not assume the Python core runs directly in the browser.
- Assess dual CDC selection, timing, reconnect, one-tab ownership, secure context, backup durability and permission loss.
- Write a go/no-go per capability. If the safety contract cannot be met, retain offline plan/export and CLI recovery instead of a web writer.

**Validation:** capability matrix and identical CLI/browser fixture outputs. **Depends:** 018. **Size:** S.

## FM1-021 — Step 2 browser offline/backup frontend

**Acceptance criteria:**

- Inspect/plan/simulate packages locally and show the same qualified/refused edges as the CLI.
- Use explicit permission/device selection; no auto-connect, probing or upgrade on page load.
- Provide validated backup and report downloads; a missing/durability-unproven backup blocks writes.
- Handle unsupported browsers, revoked permission, CDC ambiguity and reconnect; label unqualified capabilities.
- Keep browser write UI absent. Any physical read session is separately scoped and documented.

**Validation:** browser offline, mocked API and interaction tests across selected desktop browsers. **Depends:** 020. **Size:** M.

## FM1-022 — Step 2 browser write/recovery qualification

**Status: STEP 2 HARDWARE GATE.** External qualification only; the owner's FM-1 is excluded from destructive development tests.

**Acceptance criteria:**

- Every emitted byte/plan matches shared CLI fixtures and every enabled edge inherits qualified CLI policy.
- Physical tests cover selected browser/OS versions, background throttling, navigation/close, lost permission, disconnect, host sleep and independent recovery on the external unit.
- All assets are checked/loaded before writing; multiple tabs cannot own a write session; progress distinguishes transfer and verified completion.
- Browser persistence/timing failures leave writing disabled, with plan/log export for the CLI.
- Web Serial recovery explicitly requires the physical Transporter kit. A local-helper fallback needs authenticated loopback/origin/capability controls and a separate review.

**Validation:** external device matrix, failure/recovery records and policy parity suite. **Depends:** 017, 021. **Size:** L + unestimated gate.

## FM1-023 — Old firmware and selective preset backups

**Problem/outcome:** the owner wants to retain the installed firmware, optionally keep its presets, or export presets separately before choosing another firmware. These exports must distinguish recovery bytes from transferable presets.

**Proposed modes (not commands available today):** `firmware-with-presets`, `firmware-only`, and `presets-only`. A complete private `recovery-snapshot` remains a separate mode and a prerequisite for future flashing, regardless of which user-facing export is chosen.

**Product boundary:** the complete raw recovery snapshot is also a standalone
user-facing dumper capability in FM1-033, not merely an internal prerequisite
for a writer. Musical-object archives and selective exports remain this ticket's
scope; they must not be labeled a complete flash dump. See [contract 43](43-full-raw-dumper-product.md).

**Acceptance criteria:**

- Implement the three modes for an exact reviewed firmware/layout/data-schema profile. Reuse FM1-010 double-read, durability, collision and hash checks; develop only with synthetic snapshots and virtual endpoints.
- Firmware-with-presets exports the approved application plus reviewed preset payloads in a versioned manifest, identifying source firmware, image hashes and schema. Include additional loader/rescue bytes only when their regions and redistribution rights are reviewed. Never imply it is a vendor updater package or a full-chip restore image.
- Firmware-only excludes preset/project data, device identity, MAC/key and other personal storage using reviewed region/schema rules. Do not remove presets from the FM-1 or silently fill unknown regions. Unknown separation rules produce a specific refusal.
- Presets-only supports a reviewed preset codec or explicitly labels a firmware-bound raw data archive. Validate counts, lengths, integrity and schema; do not call arbitrary data-partition bytes portable presets. Permit firmware-specific USB read/export capability only after separate qualification; cable-only preset export is a goal, not evidence available today.
- Keep the full recovery snapshot private and immutable. Selective exports do not satisfy the complete-backup prerequisite and cannot bypass planner or rollback policy. Exporting a snapshot does not prove the installed app was bootable.
- Restore/import is a separate explicit operation with a dry run and an exact source→target data compatibility check. Unknown cross-firmware migration, full-chip/head restore and automatic preset wiping remain unavailable. Incompatible preset archives stay preserved rather than converted speculatively.
- CLI help, JSON reports and a future web UI distinguish firmware, presets, recovery snapshot, compatibility and omitted data. Exports contain no raw personal data beyond the mode explicitly requested; sanitized support reports still omit it.

**Validation:** independent synthetic layouts/codecs, all three mode round trips, forbidden-region/personal-data exclusion, incompatible/unknown schemas, corrupted archives and crash/durability failures. Hardware reads/imports remain outside this task's development scope.

**Depends:** 010, 011, 015 plus lawfully available, reviewed firmware-specific preset formats and future read capability qualification. **Size:** M; external format/evidence wait time unestimated. **Status:** planned, with format and hardware capability gates.

## FM1-024 — Agent automation contract and offline lifecycle harness

**Problem/outcome:** a coding agent should eventually consume the CLI as a bounded tool and distinguish a transferred image from one that actually runs. Define and exercise the [agent-driven validation goal](08-agent-firmware-validation.md) without opening hardware.

**Scope:** versioned noninteractive job/event/result contracts and an injected lifecycle coordinator. Reuse existing parser, plans, backup gates, bounded simulated writers and sanitized reports. No native adapter, firmware build, boot command or write capability becomes available through this ticket.

**Acceptance criteria:**

- Machine-readable output has stable schema versions, typed refusal/failure states, exit codes, job IDs and separate transfer, raw-verification, boot and smoke-test results. Diagnostics do not corrupt stdout; reports omit private backup/preset bytes.
- One job binds the original artifact and application hashes, explicit device/session, profile/OS/transport, fresh full backup, plan and finite deadlines/attempt budgets. Exclusive ownership rejects competing jobs; no automatic target selection, `--force` or hidden V15 hop.
- A fictional external-bench authorization is separate from agent-supplied job data. The agent cannot grant itself writes, change protected ranges or promote simulated evidence to physical qualification. Production and experimental-bench capabilities remain distinct and disabled.
- Deterministic fakes exercise plan → backup gate → flash → full preboot verification → separately requested boot → fresh health response → declared smoke checks → report. A model classification is never reported as actual CPU execution.
- Wrong/stale image, device, session, build ID or nonce; expired/cancelled authorization; concurrent jobs; failed verification; missing health replies and ambiguous reconnect all produce bounded typed outcomes. No failure authorizes a full-job retry, automatic reboot/rollback or journal-based resume.
- Cancellation/authorization expiry blocks new destructive actions and follows an explicit modeled abort/held-session procedure; it does not invent a safe immediate power-off. The guarded runner proves zero hardware access on every path.

**Validation:** independent scripted lifecycle fixtures, race/cancellation and stale-evidence cases, structured-output compatibility checks and existing guarded host regressions. **Depends:** 007, 008, 010, 011, 013–016. **Size:** M. **Status:** complete offline on `init`; 267 canonical guarded tests passed locally, plus audited repeatable wheel/install smoke. See [contract 10](10-offline-agent-contract.md) and [progress 05](05-implementation-progress.md). No real adapter, firmware hook, hardware access or write release is enabled.

## FM1-025 — Development build identity and firmware health-check contract

**Problem/outcome:** MIDI enumeration or a shared firmware version does not prove the expected development build is executing. Specify an identifiable, fresh, read-only health response and honest functional checks.

**Scope:** build-manifest and proposed firmware health protocol, initially with fictional replies. Editing/building a firmware project or adding real runtime hooks requires a separately scoped task and reviewed toolchain; this ticket does not authorize upstream firmware changes or hardware access.

**Acceptance criteria:**

- The manifest records source commit, reviewed build recipe/toolchain, configuration, package/application hashes, layout/data schema and an embedded build ID mapped to the final artifact. Do not require a binary to embed its own final complete hash.
- A proposed response echoes a fresh job-bound challenge and expected build ID, with bounded heartbeat/uptime and protocol capabilities. Wrong or stale replies cannot satisfy boot confirmation; port names, VID/PID and `FM-1_9xx` alone remain insufficient.
- Firmware lacking a qualified health endpoint returns boot-unconfirmed/inconclusive rather than passing. Full raw readback proves bytes; a fresh runtime reply provides separate execution evidence.
- Each smoke check declares its interface, observable criterion and timeout. Read-only MIDI health cannot establish audio/DSP/UI correctness; those checks need separate sensors/harnesses and can report unavailable. No preset/settings writes are hidden in a health check.
- Document runtime storage effects and data compatibility. A host-bounded flash write cannot sandbox a booted experimental application; changes to flash/boot/OTA behavior require additional review and qualification.

**Validation:** fictional build manifests and response transcripts covering ID/challenge mismatches, replay, timeout, reset and partial/unavailable smoke checks. **Depends:** 024 plus a reviewed firmware project, build recipe and runtime hooks for future integration. **Size:** M; integration wait time unestimated. **Status:** planned; firmware integration gated.

## FM1-026 — External qualification of the agent flash/boot/check loop

**Status: HARDWARE GATE.** No hardware work is authorized now. The owner's only FM-1 is excluded from development flashing, including successful-path trials.

**Problem/outcome:** establish a limited experimental application lane on a spare/external bench without weakening normal release qualification or claiming zero brick risk.

**Acceptance criteria:**

- A separate human-authorized bench scope enrolls one exact unit, OS, transport, recovery route and reviewed build policy. It fixes allowed ranges, loaders, data effects, duration and maximum jobs/attempts. A reviewed executor and authorization policy outside the agent-editable checkout enforce this scope; editable job JSON is not authority.
- Critical layout, loaders, writer, backup, held-session preboot verification, controlled boot and independent recovery qualify before experimental application trials. Every candidate is inspected and hash-bound; unknown application behavior is evaluated only within this external-bench lane. This is not a production `--force` switch, and a passing run cannot self-promote a profile.
- A fresh double-read durable full backup precedes each destructive job. Qualified preboot raw verification must complete before the separately requested boot stage. A USB-only lane stays gated if its backup/checkpoint evidence is missing; the agent cannot silently switch to an RP2040 or another firmware.
- Record actual device/OS/transport, candidate/build/loader hashes, authorization, backup/plan binding, raw verification, fresh build response and each smoke outcome. macOS/Linux and any Windows support are qualified separately.
- Finite external-bench tests cover wrong/stale identity, lost sessions, firmware boot/health failures and approved recovery scenarios. Stop and preserve evidence on ambiguity; recovery or rollback requires a separate exact qualified plan and explicit scope, never a blind retry loop.
- If the application loses USB, use only an already qualified independent recovery entry within its authorized scope. A cable alone is not claimed to recover the dark state. No head restore, full-chip write or owner-unit fault test is introduced.

**Validation:** separately authorized physical bench records plus offline replay of every supported result/failure path. No physical result is inferred from existing simulations. **Depends:** 017, 018 capability-control machinery, 024, 025, spare/external access and separate hardware authorization. **Size:** L + unestimated gate. **Status:** gated; not authorized or implemented.

## FM1-027 — Small native Rust CLI foundation

**Problem/outcome:** the owner wants one small downloadable executable and selected Rust for the shipped CLI. Establish and measure a native offline entry point while preserving the tested Python reference.

**Scope:** a separate host-only Cargo package with no third-party dependencies, version/help/status and read-only agent-contract discovery. No firmware compilation, Python embedding/subprocess, hardware adapter, package writer or runtime device access. This is the first vertical slice, not a claim that the full Python lifecycle is ported.

**Acceptance criteria:**

- A release executable runs without Python and reports native implementation capabilities honestly; native lifecycle and every hardware capability are false until separately implemented/qualified.
- Versioned reference contract data is pinned to the Python FM1-024 contract. A native capability envelope distinguishes contract discovery from implemented lifecycle behavior; it cannot promote simulation to hardware evidence.
- Unknown or destructive commands refuse without opening paths, invoking subprocesses or executing the Python reference. Machine stdout is stable JSON; diagnostics stay on stderr.
- Host-only Cargo tests and release build run offline. Record actual [local artifact location omitted] uncompressed executable size/hash and OS-library dependencies; do not extrapolate the foundation's size to a complete flasher or claim untested platform support.
- Rust source and build outputs remain separate from upstream Transporter/PIO/submodules and retain their license notices. Python guard/model tests remain applicable reference evidence.

**Validation:** five pure dispatch/refusal tests, guarded reference-fixture equality, actual native CLI invocation and release artifact evidence in [report 12](12-native-foundation.md). **Depends:** 024 and the owner's Rust decision. **Size:** S. **Status:** complete offline foundation; full parsing/planning/lifecycle remains unavailable in Rust.

## FM1-028 — Rust package parser and wire codecs

**Problem/outcome:** native discovery cannot yet inspect a candidate. Port the validated byte contracts without trusting the Python implementation as the sole oracle.

**Scope/acceptance:** implement explicit package/logical/flash offset types, checked arithmetic, FWSC/UFW marker/header/entry/CRC validation, exact APP extraction and SysEx/framed codec functions. Reuse independently expected golden bytes and malformed/seeded corpora from FM1-003–005; compare accepted data and typed refusals across implementations. Preserve the historical `0x400` extraction regression. No file/device IO, embedded package script execution, firmware build or transport backend. Missing lawful real fixtures remain documented gates.

**Validation:** pure offline Rust debug/release tests and 1,096 saved independently expected vectors; six guarded Python regeneration/secondary-reference checks, including truncation, overflow, overlap, checksum and framing failures. [Report 13](13-native-parser-codecs.md) records exact hashes and scope. **Depends:** 027 and reference 003–005. **Size:** M. **Status:** complete offline core; lawful real artifacts, catalog/profile authority, CLI file inspection and hardware remain unqualified/unavailable.

## FM1-029 — Rust profiles and transition planner

**Problem/outcome:** an inspected package must resolve to one explicit supported edge and immutable plan.

**Scope/acceptance:** port strict versioned profile/catalog inputs, exact digest/layout/data/OS/session bindings, direct-edge resolution, full expected-image calculation, protected-range policy and historical application-only rollback planning. Reject duplicate/unknown fields, ambiguous identities, unknown layouts, unqualified edges and unsupported data. Preserve USB-MIDI as the planning default and its missing-evidence refusals. Saved reports and version strings cannot issue authority or enable writing. No device/file IO or automatic firmware detour.

**Validation:** 25 catalog cases, 41 planning/direct cases and full independently calculated plans/hashes; retained mutation, privacy, protected-head/data, saved-plan and complete historical JSON/copy-binding checks. [Report 14](14-native-profiles-planning.md) records evidence and limits. **Depends:** 028 and those reference contracts. **Size:** M. **Status:** complete offline byte-only core; accepted plans are simulation-only, native durable storage/writers/agent lifecycle and physical qualifications remain unavailable.

## FM1-030 — Rust local backups and bounded fake writer

**Problem/outcome:** port the backup and write-policy boundaries before exposing a native lifecycle.

**Scope/acceptance:** implement the NOR/recovery model, injected framed readers, exact fake sessions, double-read immutable snapshots, local two-copy durable storage and no-resume journal, advisory dump analysis and bounded application writer/rollback. Every erase/page/retry must recheck exact identity, immutable inputs, backup copies and authority after injected callbacks. Local IO must establish reviewed per-OS protections against device paths, symlinks, FIFOs, overwrite and stale handles; unsupported OS persistence refuses. Keep all hardware adapters absent. A native offline test boundary must block device, network, process and FFI escape before introducing local IO.

**Validation:** reviewed secure-IO design; 90 persistence cuts and protected/zero-write/copy-loss/callback/saved-provenance tests; 111 Rust tests in debug and release, with 142/712 independent writer rows respectively; 302 guarded Python tests. [Report 15](15-native-backups-model-writer.md) records results and limitations. **Depends:** 029 and the listed reference evidence. **Size:** L. **Status:** complete offline core; macOS/Linux storage implementation, unsupported OS persistence refuses, current-source hosted/release and physical qualification remain separate.

## FM1-031 — Rust offline agent lifecycle

**Problem/outcome:** make the native CLI execute the already specified fictional lifecycle.

**Scope/acceptance:** port FM1-024 strict jobs, separately injected fictional authority, exclusive ownership, consumed IDs, cancellation/deadlines, bounded fake transfer, full model verification and separately requested fictional boot/health/smoke. Preserve JSON/NDJSON schemas, typed exits, sanitized failures and explicit evidence labels. The native envelope may mark only the offline lifecycle implemented after passing parity; every physical capability stays false. No automatic reboot, reflash, resume or rollback.

**FM1-030 integration constraint:** saved-reverified backups are historical-only and cannot become current writer evidence, even when session hashes coincide. Current jobs require fresh captured copies. Native fake session counters are process-local; specify fresh process/session identity and fail closed on collisions before exposing a lifecycle across invocations.

**Validation:** all contract scenarios and adversarial late-gate/race/privacy cases against independent expectations and reference fixtures. **Depends:** 030 and 024. **Size:** M. **Status:** planned offline; firmware instrumentation and external qualification remain FM1-025/026 gates.

## FM1-032 — Native artifact and release qualification

**Problem/outcome:** produce reviewable native CLI artifacts for macOS/Linux and assess Windows independently.

**Scope/acceptance:** choose and pin reviewed Rust/build inputs and OS/architecture targets; record actual size, startup, runtime libraries, checksums, licenses and source/toolchain provenance. Exercise installed binaries without Python on each claimed target, audit reproducibility and publish a capability/support matrix. Decide signing/notarization and distribution policy before release. Optional browser/WASM sharing is an FM1-020 spike; no WebUSB or browser flashing is inferred from Rust use.

**Validation:** software-only per-target build/install/CLI/parity records and artifact audits. A complete native offline CLI can release with hardware capabilities disabled; enabling physical writes still requires FM1-017/018 and separate authorization. **Depends:** 028–031 and 016 evidence. **Size:** M. **Status:** planned offline; signing/release decisions pending.

## FM1-033 — Full raw backup over USB-C from stock firmware

**Problem/outcome:** the owner needs a complete recovery snapshot from a stock FM-1 using one computer USB-C data connection, without a firmware install, V15 detour or required RP2040. The current native storage library accepts fictional USB and Transporter reads; it is not a live backup command.

**Authorization:** on 2026-10-15 the owner explicitly allowed hardware access and prohibited flash writes. Begin with offline implementation on `init`. No erase, programming, firmware replacement, preset/settings writes or guessed device commands. Permission alone does not establish a stock raw-read protocol. Mode changes and volatile loader execution must be identified explicitly and supported by an audit before use; do not label them passive reads.

**Work items, in order:**

1. **033-A: source and route evidence.** Check current stock editor/MIDI, direct ROM USB and upstream recovery sources. Separate a running-stock read endpoint from a temporary ROM/RAM-loader session that preserves installed flash. Pin sources, command effects, loader provenance, platform constraints and unknowns. Community reports are leads, not protocol or compatibility qualification. Record findings in [report 16](16-stock-usbc-backup.md).
2. **033-B: offline Rust read boundary.** Port the existing finite USB BOT read model and validate explicit saved descriptors, allowlisted INQUIRY/raw-flash requests, exact tag/status/residue/length checks, bounded full reads and session drift/fault refusal. Add an offline stock-backup requirement/route assessment with honest machine-readable capability reporting. No native adapter, automatic mode switch, reset, RAM upload/jump or arbitrary CDB interface. ROM memory reads and preset exports cannot satisfy full raw backup.
3. **033-C: offline capture/storage integration.** Complete offline. The concrete USB reader takes two fresh complete equal reads internally, creates immutable USB-specific snapshots/manifests in two private directories and verifies reopened hashes/file identities. Exact fictional descriptor/target/loader/session provenance stays private. Saved verification is historical-only and never becomes Transporter or writer authority. Source firmware and physical/loader proof remain unknown/false. See [report 18](18-native-usb-backup-storage.md).
4. **033-D: platform read adapters.** **D1 complete offline:** exact fictional selected-[local artifact location omitted] contracts, Linux SG_IO-shaped/macOS IOUSBHost-shaped result validation and finite two-full-read replay; see [report 19](19-native-read-adapters.md). **D2 injected increment complete:** [source/loader audit 20](20-stock-entry-loader-audit.md), pure entry assessment, injected per-session request/callback driver and secure capture with separate schema-2 provenance; see [integration 21](21-injected-native-read-driver.md). Callbacks bind lifetime/sequence and exact held-loader checkpoints; failure poisons the driver, and saved files remain historical. **Actual binding code added offline:** separate opt-in `native-usb` package, actual nusb 0.2.8 selected-open/claim/queue/completion code, strict finite BOT engine and cancellation/deadline fakes. macOS and Linux arm64 feature compilation passed; see [readiness/build report 28](28-first-backup-readiness.md). Other Linux architecture builds and platform qualification remain pending. The public factory is a constant refusal behind an uninhabited private permit; no CLI/storage activation. See [report 22](22-actual-usb-bindings.md). [Loader follow-up 23](23-loader-static-followup.md) independently decodes the pinned image and identifies flash-protection/status-register diagnostics; startup/read/return effects remain unqualified. The V15-named upstream guard checks VID/PID only. **Remaining D2, in order:** implement and validate the conditional observe-before-entry/no-FC0A profile from [decision 34](34-no-status-backup-profile-decision.md), retaining external ROM/configuration/chip evidence gates; define actual native-to-storage provenance and held-session continuity; qualify selected-object freshness and macOS/Linux open/claim/cancel/release before replacing the closed gate. Extend build evidence to each additional shipped architecture; Linux product validation does not block a macOS-only backup once its own route qualifies. Cached descriptors and a local deadline do not attest to a running loader. Preserve strict status/residue/length validation, refuse ambiguous/unsupported access and keep physical evidence separate from injected records. Qualify held-loader lifetime during disk sync, destinations and independent media; assess Windows separately. No device sweep, automatic retry/reset, OTA/firmware fallback or implied dongle. D2 development/tests remain offline; backup hardware permission does not waive exact evidence or permit flash writes.
5. **033-E: authorized read-only acceptance.** With an evidenced route and exact owner-device identity, make two fresh equal full reads, verify chip/capacity/layout and durable independent copies, and document the return to original stock operation. Stop on uncertainty; no automatic retry/reset or write. Record actual transport, OS, source firmware, entry/loader effects and hashes. No claim of restore or recovery-write qualification follows.
6. **033-F: standalone raw-dumper product surface.** [Contract 43](43-full-raw-dumper-product.md) makes full acquisition a product goal alongside FM1-023 data archives. **F1 implemented offline:** `dump inspect --file ABS --capacity BYTES [--json]`, `dump verify --file ABS --capacity BYTES --sha256 HEX [--second-file ABS] [--json]`, explicit full-raw/historical results, strict pure parsing and secure saved-file reads through existing Directory IO. Caller-declared powers-of-two capacity128KiB–16MiB bounds every byte; no `.fwsc` or APP/preset parser is required. Computed SHA and optional distinct-file byte/hash comparison never attest to actual chip capacity, fresh device passes, independent media, final continuity/return or writer authority. `dump capture` stays refused; no acquisition engine, file export, native factory or write gate was activated. [Report 44](44-offline-raw-dump-cli.md) records independent whole-byte vectors, secure-file drift/refusal tests, 44 actual empty-feature release CLI checks, 73 integration/7 root-dispatch/3 new file tests in debug, new 8+3 tests optimized and 318 guarded host tests. **F2 release:** expose only exact qualified routes/OSes after D2/E; label original-stock, already-installed-Felucca and external recovery separately. Full capture still requires identified capacity, two fresh complete passes and protected independent copies. CLI first, web later. No full-chip restore or write activation. F1 does not block the first qualified E backup; no duplicate acquisition engine or mandatory helper rewrite is selected. **Status:** F1 complete offline; physical qualification and F2 remain gated.

**Rescue-source follow-up within 033-D2:** [Report 24](24-rescue-script-review.md) evaluates the owner-supplied `fm1_rescue.sh` and its actual cross-repository Python download. It supplies another concrete host implementation of the known ROM/RAM-loader route, not a new stock-entry protocol or qualified macOS backend. Keep the existing D2 order and refine its platform/session work as follows; no separate implementation ticket or backend replacement is selected.

- **Selected-device access comparison:** compare the pinned rescue open/detach/configure/claim/error/cleanup sequence with our selected-object binding. Record which operations can change device state or trigger reset/re-enumeration, and what evidence is needed for the exact macOS route. First-match VID/PID discovery, forced detach, BOT reset and automatic reopening are not adopted. Source comments about capturing a crash-loop device do not qualify entry from running stock or a no-USB failure.
- **Held-loader lifetime:** use the rescue confirmation-time keepalive as an additional reference for the existing loader-expiry gate. Audit the exact command and transitive loader effects before considering any allowlist change. Specify one serialized session owner across complete BOT transactions, user waits, both full reads, persistence/fsync/reopen/copy checks and cancellation. Add offline injected cases for slow storage, idle expiry, lost keepalive and cancellation at those boundaries when implementing this design. A local deadline or keepalive thread is not evidence of continued loader/device identity; unresolved lifetime or loss must refuse acceptance without reset/retry.
- **Provenance and reuse:** retain the immutable wrapper and actually downloaded Python revisions/digests from report 24; do not execute a mutable `main` download with elevated privileges. Check source-license obligations before copying code. The stock-package comparison, single backup read, retry/reset behavior and restoration commands cannot replace our package-independent, two-read/two-copy backup acceptance or authorize physical writes. Existing entry, loader-effects, storage-provenance and physical gates remain open.

**Exact-loader audit, 2026-10-15:** [Startup 29](29-loader-startup-callpaths.md) resolves entry to NOR initialization and a mode-4/status-dependent `06/01/31` write path. [Read/return 30](30-loader-read-return-callpaths.md) resolves FD05 and shows ordinary padded FC0A reopening NOR and selecting protection ioctl 13(arg zero), with concrete conditional status-write attempts; FC0C requests an MMIO reset. The strict no-write lifecycle therefore stays unqualified. Do not add FC0A to the read allowlist. Model fresh untransformed data state, supported read mode, ignored helper errors and ROM callback/idle continuity explicitly; equal dump hashes alone cannot establish raw preservation. These are findings within D2, not permission for another loader or physical experiment.

**Conditional no-status profile, 2026-10-15:** [Start 32](32-loader-no-status-start-profile.md) derives an exact local mode-2/no-key path; [session 33](33-loader-no-fc0a-read-session.md) maps selector 6 to repeated non-continuous 3B reads without FC0A. The selected candidate inspects existing pre-entry RAM and refuses uncertainty; no forced configuration, patched loader or alternate entry is selected. Actual ROM/handoff/chip/macOS qualification remains false.

**Offline subitems within D2 (R1/R2/R3 and R4-A/R4-B subsets complete; R4-C evidence/activation remains):**

- **R1 — initial selected subset complete offline:** pure Rust bounded saved-context assessor with exact A/B/A/B header selection, CRC16 and 512-byte transform resets, complete bounded records/key copies/values/NULs and binding/fingerprint drift refusal. Supports the no-SPI default and exact SPI mode 2/divider 3/read 0/pin 0; other modes and matched startup keys are explicitly unsupported, mode 4 refuses. Independent synthetic vectors and guarded regeneration are recorded in [report 35](35-native-start-context-assessor.md). Saved consistency is not capture-age/freshness proof; output remains conditional and cannot mint a physical permit. No CLI file command, capture or hardware activation is added.
- **R2 — initial selected subset complete offline:** typed fixed FD07(0x01C09700,84) saved observations, private phase/session/start linkage and reassessment of the original R1 context. Exact mode-2 state, full cached 24-bit ID and independently declared capacity expectations are checked; typed initial/final observations compare established static fields while excluding non-atomic counters, online state and opaque bytes. Unknown schemas/phases, drift, unsupported starts and reused observation IDs refuse. All physical/freshness authority stays false; no generic memory/CDB/allowlist or CLI activation. See [report 36](36-native-fixed-metadata-validation.md). Read-only backup permission was renewed for this continuation; R3/R4 evidence still precedes a physical backup, and no flash writes are authorized.
- **R3 — initial selected subset complete offline:** isolated concrete-fake serialized actor obtains two new full 1 MiB passes, separately accounted positive FD05 checkpoints and two fixed FD07 observations validated through R1/R2. Cooperative in-memory storage performs ordered modeled write/sync/reopen/hash/copy stages, with finite checkpoints during waits and final continuity observations after both modeled copies verify. Immutable budgets, drift/byte/protocol/storage faults, cancellation and panics permanently fail the attempt; no retry/reset/finish/resume. Storage receipts are modeled and cannot become durable-file, native, physical or writer proof. Legacy request counts/schemas and sealed secure-file backups remain unchanged; actual file/native integration and qualification stay in R4. See [report 37](37-native-finite-backup-session.md).
- **R4-A — finite native wire engine complete offline:** private actual `BulkIo` serialization of the no-FC0A schedule, fixed initial/final FD07 metadata, two new full 1 MiB passes and separately budgeted positive checkpoints. A private unreachable bridge typechecks the actual macOS/Linux adapter composition; the public factory and empty permit remain unchanged. Finite monotonic timing, protocol/byte/metadata/order/budget failure and cancellation poison the attempt. An opaque move-only handoff owns both passes; R4-B subsequently adds its real host-file receipt. Injected modeled receipts cannot qualify physical/native storage. See [report 38](38-native-backup-wire-engine.md).
- **R4-B — complete offline for the selected subset:** the native handoff now integrates real protected regular-file storage and typed private source/transcript provenance without changing fictional schemas or granting physical proof. Preflighted destinations precede any wire request; one worker owns immutable full buffers and private directory/file identities, creates two fsynced/reopened/hash-verified snapshots and storage-only manifests, and returns a one-use same-attempt receipt. One wire owner performs finite checkpoints during file work and final FD05/FD07 continuity after the durable receipt, with real-wall and local timing checks. Final observations remain in the private in-memory result; saved manifests explicitly withhold session acceptance/current/writer authority. Storage/worker faults, late/foreign/reused receipts, drift, cancellation and exhausted budgets refuse without retry/reset. **Validation:** 46 binding tests and 11 new core tests passed in both debug/optimized builds, 26 file-only guard tests, default-library/macOS feature checking and 314 guarded host tests. See [report 39](39-native-secure-file-integration.md). No hardware access/CLI activation; independently qualified media/timing/freshness and any durable export of final session observations remain distinct follow-up work.
- **R4-C — operation-specific evidence and later activation:** for the currently selected observe-before-helper route, establish exact V15 stock-to-ROM handler/callee/framing, ROM FD07 effects and A/B interval ownership, selected macOS object/access and a documented return before any bounded ROM-only capture. That capture can stop before FB08 and supply R1 inputs; loader/chip/handoff qualification gates the later jump/full reads. A/B observation is not a universal backup requirement: R4-C-P1 below compares a separate deterministic-start candidate that could remove this operation, without inheriting current profiles or bypassing safe-start evidence. Full-backup acceptance then needs actual timing/cancellation, native/storage continuity, independent-copy evidence and observed stock return. See reports 38 and 48 for staged gates and route choices. Existing read-only permission needs no renewal once those requirements are established. No prior full backup, write/restore qualification or Linux/Windows qualification is required for the first qualified macOS read. The activation implementation must compose a CLI executable over the one-way core/native packages without a dependency cycle, preserve the identity-only command and explicitly describe storage-only manifests versus any future durable final-observation export. Factory refusal changes only in a separately reviewed activation change; no hardware work is included in R1–R4-B.

**R4-C source progress, 2026-10-16:** [Report 40](40-v15-entry-rom-backup-audit.md) acquires the exact manufacturer V15 package and resolves its actual USB-MIDI handler, initialized RAM and complete local `usb_update_mode`/P33-reset callee chain. The earlier `0xFFC02532` absolute-ROM interpretation is corrected to a file-relative annotation; V15's actual callee is RAM `0x01C026A4`. Python/Rust parsers now accept only the canonical empty type-2 `info.log` metadata record found in genuine V15, with CRC-repaired adversarial regressions and exact-package extraction checks. **C1 complete offline; R4-C remains partial.** Next C2 evidence is scoped reset-to-ROM mailbox-consumer/autonomous effects, ROM FD07 restoration and selected A/B ownership/return; C3 is exact selected macOS framing/access/continuity. Only then may C4 capture the finite pre-loader configuration under existing read permission. C5 separately qualifies loader external startup/callbacks/chip and the selected no-status full-read subset before activation and two-copy acceptance. Existing tools' default backup preludes still use unqualified startup/FC0A; changing OS or adding a dongle alone is not a no-write shortcut. No hardware operation or flash write occurred.

**R4-C ROM/macOS/return follow-up, 2026-10-16:** [Report 41](41-rom-fd07-macos-entry-return.md) rejects the official WL82-named ROM image as byte-identical BR28 and narrows another exact-SPL marker producer/reset branch. C2 still needs matching WL82 reset-consumer/FD07 success-and-failure restoration and A/B active-memory evidence; inherited WL80 JTAG limits are not a substitute. C3 now distinguishes exact-ID registry iteration, tolerated device-open failure, exclusive interface refusal, AbortPipe controller halted-bit effects and pending versus drained release. Static installed Apple-driver analysis identifies its queued sender, error-path retry and unresolved legacy packetizer vtable slot. **Next bounded offline C3 work:** bind USBMIDIDevice+0x50/class/vtable+0xC0 and its quiet six-byte SysEx capacity/flush path, then the receiver's actual eight-byte result; one USB transfer is a candidate, not a universal requirement. Matching external selected-stream evidence can also close the outcome. Return candidate is preselected deliberate manual cold power after completed/quiesced reads; neither FB08(argument zero) nor FC0C is a bare-ROM exit. No automatic error fallback. CLI artifact discovery is accurate and historical-only, with all entry/backup/write gates still closed. No hardware access or flash write occurred; a new fake-only transition model is not substituted for these evidence gaps.

**R4-C targeted online evidence, 2026-10-16:** [Report 42](42-targeted-wl82-rom-external-evidence.md) checks the supplied USB_KEY/forum/SMK/misctools/uboot/official-SDK/AL-255 sources and new external sessions. Lunar's actual callback values `0x01E008E4`/`0x01E0089A` are loader-phase FD07 after FC0A, with no ROM FD07 in that receipt. A Windows report adds two equal full 1 MiB reads; its observed Felucca return follows 17 sector rewrites. Melodee supplies macOS packaging and a different 30,720-byte helper, independently byte-matched to official SDK tag `AC79NN_SDK_V1.2.1_2023-12-13`; default backup still performs bare-ROM FD07 and FC0A, then one full pass. Its upload overlaps the old profile's A/B and metadata addresses, so no old offsets or behavior claims may be transferred. Four additional official SDK branch trees, four mirrors, three Bootloader forks and a new application ELF do not supply the missing matching ROM body/map in the inspected scope. **Next bounded evidence:** use the new callback addresses to seek matching ROM artifacts; optionally audit the now-bound alternative helper's own startup/status/read/configuration contract and whether it can avoid ROM FD07/FC0A. No automatic helper switch, new fake-only model, prior backup/write/all-platform prerequisite or activation is selected. C2/C3/C5 remain unresolved for the selected no-write route; existing read permission remains present. Research only; no hardware access or flash writes.

**R4-C official 30 KiB helper audit, 2026-10-16:** [Report 45](45-official-30k-loader-no-status-read.md) completes that alternative's bounded local startup/read/return analysis, bound to encoded SHA-256 `d9de9f1c…`. Default mode 2 yields non-continuous selector 6/3B reads with no local status-program edge; incoming FB08 argument does not select mode. Startup still conditionally issues 01/31 in mode 4, and ordinary FC0A reopens NOR then invokes ioctl13 protection with possible 01/31 writes. FC0C also has a conditional 11 status-cleanup path. A local positive FD05 path needs neither FC09 nor FC0A, but no helper is switched or activated. New A/B are `0x01C09800`/`0x01C0A800`, G is `0x01C0B1B0`, and fixed loader metadata is 88 bytes; old R1/R2 layouts cannot be reused. **Next evidence remains C2/C3:** selected reset consumer, bare-ROM FD07 restoration/new A/B ownership and exact macOS framing/access/return. Only then may C4 observe the sufficient default-case 1,058 bytes before FB08. Matching external startup `0x01E00428`, stack/interface/hook/callbacks, chip semantics and timing gate C5's later positive read/full capture. Post-start metadata cannot prove a safe startup. No prior backup, write/restore, whole-ROM or all-OS gate is added. All factory/profile/writer gates remain closed; no hardware access or flash writes occurred.

**R4-C macOS packetizer/V15 receiver, 2026-10-16:** [Report 46](46-macos-entry-packetizer-v15-receiver.md) closes report 41's concrete local class/device/legacy-packetizer dispatch and follows V15 reception through IRQ, callback, DMA and SIE. Exact retained Apple driver `3e0c5e6e…` maps vptr28AD0+C0 through1CAC8 to19254; one fresh quiet legacy six-byte node, cable0 and capacity>=8 produces the required eight USB bytes in one selected submission. Exact V15 descriptor/setup binds bulk OUT4/maxpacket64. Its reader aggregates currently ready data and passes the actual length; split invocations or extra bytes need not enter. Static compatibility does not attest to loaded driver, active queue/pipe or receiver delivery. The Apple driver's immediate retries and callback ClearPipeStallBothEnds/ClearFeature error traffic are explicitly documented. **C3 local framing substantially resolved; actual selected object/access/continuity remains open. Next C2 evidence:** matching reset consumer, bare-ROM FD07 restoration and selected pre-loader RAM ownership, followed by the already scoped qualified capture/return. No helper switch, additional fake model, prior-backup/write/all-ROM/all-OS gate or capability activation. Offline files only; no hardware access or flash writes.

**R4-C ROM effects/RAM ownership follow-up, 2026-10-16:** [Report 47](47-rom-fd07-effects-ram-ownership.md) examines20 previously unexamined OTA archive members (374,904 bytes;258 inventoried definitions) and exact V15/SPL RAM lifetimes. These candidates identify an OTA UBOOT2/E3B5:2305 build; a real boot-record flash-write edge cannot be attributed to the selected ROM or used as a no-write substitute. No matching bare-ROM FD07 dispatch/transform/send/restoration/failure closure or active RAM map is obtained. Selected old A/B are initialized V15 application data, with live app references inside A; optional30 KiB A has six nonzero initial bytes and explicit mutable state, while B is application BSS. Application/SPL startup geometry is not ROM ownership. **C2 stays unresolved on an external-evidence dependency:** the next useful input is matching selected-ROM implementation/allocation evidence or scoped external read/restoration/ownership evidence; repeating the same local scans is not a new milestone. Current1058-byte R1 capture remains unchanged. A69-byte contiguous-prefix refinement is logically sufficient only for the narrow zero-fields/no-key predicate, not a new profile or target-touched-span guarantee; later loader mutation still covers1024 bytes. No shorter CDB, forced context, helper switch or new model is enabled. Existing permission and C3/C5 stages remain; no prior backup, write/restore, whole-ROM or all-OS gate added. No network, hardware access or flash writes.

**R4-C fresh online route reassessment, 2026-10-16:** [Report 48](48-online-rom-evidence-route-reassessment.md) finds a narrow external **bare-ROM** FD07 read at0x01C033D8/32B after an installed diagnostic returned to ROM, superseding report42's earlier snapshot within that narrow scope. It does not qualify our selected A/B ownership, restoration or errors. New Lunar reports describe two equal1MiB reads using a distinct16-byte-patched24KiB helper, no pre-loader FD07 and515 FC0A; separate return evidence follows application writes/restoration and an ambiguous FC0C disconnect, then normal FM-1_092 identity on Linux. Neither is the owner's no-status V15/macOS acceptance. New M-Flasher source and related SMK double-read reports add useful interface/alternative-reader leads, not matching immutable-ROM code or an active RAM map. Fresh tool comparison confirms existing installers/recovery/dumpers are practical; our A/B pre-read is route-specific. No matching ROM obtained in the bounded online scope, no helper/profile switched and no factory enabled.

**R4-C-P1 — bounded offline audit: finite backup-route decision. Problem:** remove an optional risky pre-read if a deterministic existing helper can meet the same no-flash/status-write constraint. **Scope:** compare Lunar's pinned16-byte profile source (`9664e132…`, candidate encrypted `fe6d1bd3…`) against our exact original and unchanged observe-before-helper route; independently check patch/control flow, image-owned zero-padding lifetime, external initializer/supplied operations, raw-chip behavior and compatibility with positive FD05 **without FC0A**. Resolve reuse/license obligations before copying implementation. **AC/DoD:** produce one source/hash-bound route decision and the smallest concrete implementation/evidence follow-up; explicitly retain selected macOS access/continuity, timing and original-stock return requirements. A/B ROM effects/ownership are needed only if that operation is retained; startup/reset effects cannot be bypassed. Stop repeating unchanged searches if the same external dependency remains. No patch execution, firmware build, vendor-code execution, helper selection, new capture profile, hardware access or gate activation in this item; a minimal source reader remains a fallback design option. Existing R1–R4-B contracts stay unchanged. **Depends:** report48 evidence and existing retained original; no prior full backup, write/restore trial or all-ROM/all-OS requirement. **Size:** M bounded audit. **Status:** Complete offline in [report 49](49-lunar-deterministic-backup-route-audit.md). Exact 16-byte reconstruction and startup/read audit support a deterministic 256-byte FD05 candidate without A/B observation or FC0A. Current runtime profiles remain unchanged; external startup/handoff evidence still prevents activation. Existing route remains partial and all physical writes remain unavailable.

**R4-C-P2 — matching deterministic-start/handoff evidence. Problem:** the pinned Lunar patch removes optional A/B observation but still calls ROM 0x01E00428 before NOR open and depends on supplied entry memory, hook dispatch and sender behavior. **Scope/AC:** obtain matching implementation or scoped external qualification for the exact startup and selected 256-byte FD05/fixed 84-byte loader FD07 path. Establish critical live configuration/code/stack/state preservation and absence of flash/status programming; zero terminator 0x01C07C62 only needs validity through the last lookup before that ROM call. Interface I needs 32 readable bytes; its +8 field points to a separate writable hook word. Qualify actual selected stack/IRQ envelopes and CBW/answer/scratch/state ownership, sender 256/84 consumption before scratch reuse and selected next-BOT/error ordering. Natural alignment is a sufficient profile condition, not a recovered formal ABI. The receive field is read/copied but its target is unused; no receiver-body/nonzero-target qualification or dead-padding preservation is required for this subset. Record which existing chip/freshness, macOS access/continuity, timing/cancellation and original-stock-return facts are covered or still missing. **DoD:** evidence-bound closure or a precise retained dependency; no repeated unchanged scans, saved-only-model authority or renamed P3. No all-ROM, 512-byte, prior-backup, write/restore-trial or all-OS prerequisite. Any future deterministic profile needs a new exact digest/start binding, G+4=0 metadata expectation and explicit 256-byte counts; current R1–R4 contracts stay unchanged. Read complete applicable license/scope notices before copying implementation; binary redistribution is separate. **Depends:** P1 and selected C2/C3/C5 operations actually retained. **Status:** bounded offline callsite/memory/callback audit recorded in [report 50](50-rom-initializer-memory-callback-contracts.md); qualification remains **Needs matching artifact/external qualification**. Successful exact-helper external 256-byte double reads materially corroborate usability, but include 515 FC0A and no fixed 84-byte metadata observation. No matching initializer body or actual ROM allocation/sender-ordering evidence obtained. [Report 51](51-online-initializer-no-fc0a-evidence.md) completes the requested targeted online refresh: official stub/image introduction history, two additional SDK mirrors and current tool/branch sources add no matching initializer or no-FC0A campaign; inspected fm1-ctl/Flipper read paths explicitly contain FC0A. Existing successful dumps remain positive evidence, with the selected startup/status/handoff dependency retained. No hardware action/profile selection/activation; optional saved-only implementation cannot close this dependency.

**R4-C alternative reader decision, 2026-10-16:** [Report 52](52-mflasher-source-reader-route.md) positively audits M-Flasher's source-defined SPI reader and both exact3496-byte A/B artifacts: selected local entry/FD05 needs neither direct ROM0x01E00428 nor FC0A. Prefer a minimal read-only derivative for the next implementation. This does not close the original vendor P2 contract or select a runtime profile. Vendor32-byte debug interface/fixed84-byte FD07/configuration/timer requirements do not apply to this different route. Actual ROM entry/USB services, selected memory/SPI/chip/Mac continuity and original-stock return remain operation-specific evidence; complete ROM reconstruction is not required. Upstream's combined writer/host are not adopted: destructive handlers, FC0A probing, SDK fallback, retries/reset, short-IN/residue handling and single-read-plus-CRC/storage differ from our contract. No hardware test occurs in this audit.

**R4-C-M1 — minimal source-defined read-only RAM artifact. Problem/outcome:** eliminate opaque vendor SPI initialization/status paths and destructive target commands by construction. **Scope/AC:** implement only necessary entry/hook/SPI glue, a fixed256-byte raw03 read, distinct artifact-bound identity and JEDEC reply; remove erase/program/unprotect, clock boost, display/P33/efuse/CRC/pipeline extras and arbitrary ROM-command fallback. Explicitly initialize retained state, enforce capacity/address/length/data-direction/CBW bounds and bounded SPI waits/errors; define fail-closed command handling without delegating unknown/destructive requests to ROM. Bind source, build inputs/toolchain, exact image/digest, linker/code/state/stack geometry and selected decoded read/start closure; demonstrate no prohibited NOR opcode path. Preserve applicable Apache2/upstream notices and mark changes. Keep all device adapters/factories disabled and tests hardware-free. No prior-backup/write-restore/all-ROM/all-OS prerequisite. **DoD:** reviewable exact source/artifact and static/injected evidence, no physical-qualification claim. A task scoped explicitly to building this RAM artifact is required before running a target build; this audit runs no firmware build. **Depends:** report52; selected actual ROM entry/handoff qualification remains a later physical gate, not a reason to repeat vendor-ROM searches. **Size:** M. **Status:** Ready for a separately scoped offline RAM-reader implementation; not implemented here.

**R4-C-M2 — strict host integration for the source reader. Problem/outcome:** connect the exact read-only artifact to existing native transport and protected two-copy capture without inheriting upstream fallback/retry/storage policy. **Scope/AC:** explicit distinct identity/JEDEC/fixed256 FD05 profile, exact checked payload upload/jump and selected-device continuity; noFC0A/FC09/ROMFD07/vendor-profile assumptions or vendor reuse/fallback. Require actual transfer lengths and full BOT tag/residue/status checks; no automatic command retries, BOT reset, clear-halt, reconnect or power fallback after ambiguity. Reuse existing guarded adapters, two fresh full coverage passes and durable independent copies; no cached/saved-only authority. Match each changed protocol/error seam with independent injected tests and guarded host runner, not another model-only milestone. **DoD:** offline integration and exact first-test receipt format, public factory remains closed pending the selected physical qualification; no borrowed original R2/84-byte metadata or new USB-library requirement. **Depends:** M1 artifact contract; existing transport/storage. **Size:** M. **Status:** Planned after M1.

**R4-C-M3 — first limited source-reader hardware qualification. Problem/outcome:** measure the exact selected startup/read/return before attempting a full backup. **Scope/AC:** separate hardware-scoped task with no flash/status writes; bind current selected V15/Mac, reviewed stock-to-ROM entry, exact M1 image/RAM/stack/handoff and M2 transport. One upload/jump, fixed16-byte identity and JEDEC/capacity check; two fresh256-byte reads at0, then two atcapacity−256 only after valid earlier results. Strict success/continuity/length/tag/residue/status/deadline checks, no retries/reset/vendor fallback. Preserve private receipts; quiesce successful reads and execute the reviewed manual-cold stock return/identity check. Stop on failures, no implicit fallback. **DoD:** attributed exact device/OS/artifact read/return evidence or a precise failure record. This is not full-backup acceptance; afterward, two complete equal fresh reads and existing protected two-copy storage/stock return are required. **Depends:** M1/M2 and selected operation-specific evidence; no prior backup or write/restore trial. **Status:** Planned, not authorized for execution within the current no-hardware audit; owner willingness to test recorded, original read permission remains present. No unconditional no-brick guarantee is asserted.

**Acceptance criteria:** full raw means every byte of the independently established flash capacity (currently modeled as 1 MiB), including boot head, application and data/presets/calibration/tail. Version strings, matching preset exports, successful transfer or a repeated cached dump cannot qualify it. Tests must prove fresh full coverage and refusal for gaps/overlap/corruption/stale sessions, malformed BOT replies, drift and copy failures. Public reports exclude personal bytes/device identifiers. A supported running-stock route and a supported temporarily entered loader route are distinct capabilities. Unknown/unsupported routes stay disabled with an actionable reason. All physical write flags remain false.

**Validation:** A/B evidence remains in [report 17](17-native-usbc-backup.md), C in [report 18](18-native-usb-backup-storage.md), D1 in [report 19](19-native-read-adapters.md). [Report 21](21-injected-native-read-driver.md) records D2's injected driver/storage checks. [Report 22](22-actual-usb-bindings.md) adds 26 debug/26 optimized injected binding tests, 12 file-only guard tests, actual macOS feature compilation, format/source/dependency audits and 302 guarded Python tests; [Report 28](28-first-backup-readiness.md) subsequently records isolated Linux arm64 native compilation, 26 debug/26 optimized fakes, 12 guard tests and 306 guarded host tests. Other Linux architecture builds and hosted CI remain pending. **Depends:** 019, 030 and exact stock protocol/entry evidence; 033 does not wait for the offline agent lifecycle. **Size:** L + unestimated evidence/platform gate. **Status:** A–C/D1 and D2's source-audit/injected-driver/capture increment complete offline; actual native binding code implemented with macOS/Linux arm64 compile evidence and activation closed. Additional target builds, native/storage provenance, transitive entry/loader/OS qualification and E remain gated. No live USB backup or physical write capability is claimed.

## FM1-034 — Read-only firmware identity in the native CLI

**Problem/outcome:** the owner's macOS browser editor reports V15 using a normal MIDI identity query. Provide that bounded operation in the Rust CLI independently of the still-gated full-backup/ROM/loader route.

**Scope/acceptance:** explicit MIDI metadata inventory; exact input/output endpoint IDs; one fixed ten-byte identity request; strict length/packing/checksum/model parsing; finite receive window and sanitized errors; optional macOS-only backend with empty default features. Add offline reply inspection and truthful capability/result labels. No broadcast, first-match choice, arbitrary messages, automatic retry, preset/settings transfer, mode change, RAM execution or physical flash write. Firmware-controlled identity is advisory and cannot grant backup, entry or writer authority. Development/tests remain hardware-free; compile the actual adapter without running it.

**Validation:** strict independent reply/CLI vectors, injected callback/time/port errors and source/dependency boundary checks; canonical guarded Python/native tests and macOS feature compilation. Record exact results in [report 26](26-native-midi-identity.md). **Depends:** native foundation/codecs and [identity research 25](25-stock-version-identification.md); does not wait for the loader audit. **Size:** M. **Status:** implemented for offline validation and opt-in macOS compilation; one separately authorized CLI query returned valid `FM-1_015` on the owner's Mac, recorded in report 26. Other device/OS acceptance remains separate. Existing backup/flash gates stay closed.
