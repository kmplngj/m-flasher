> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# FM-1 flasher specifications

Initial research date: **2026-10-11**; current-state update: **2026-10-16**, Europe/Berlin. Target: **M-VAVE FM-1**, also commonly written M-Wave/FM1.

## Decision

Use an **FM-1 Transporter fork as the code/recovery foundation**, retain its small RP2040 firmware and address restrictions, and build a standalone cross-platform host CLI. **Normal installation should use a direct USB-C cable via USB-MIDI, without an RP2040 or compulsory old-firmware detour**, for explicitly qualified source/target combinations. The RP2040 is an optional emergency recovery accessory for a unit that no longer enumerates over USB. Make the browser frontend **step 2**, after the CLI and recovery path qualify.

The goal is direct installation of supported firmware without a compulsory stock/old-firmware detour. Unknown combinations are refused; a version number or successful transfer alone is insufficient evidence of compatibility.

**The CLI will also offer a standalone full raw flash dumper alongside musical
data backups.** Capture every stored physical byte, including encrypted areas,
boot/configuration/device data and musical data, with two fresh matching reads
and protected independent copies. Original stock before the first installation
is the priority route; no update package or firmware change is a prerequisite.
[Contract 43](43-full-raw-dumper-product.md) defines this FM1-033 product goal and
its distinction from FM1-023 archives. F1 now provides offline saved-file
`dump inspect`/`dump verify`; actual acquisition and hardware qualification remain
pending. See [implementation 44](44-offline-raw-dump-cli.md).

On 2026-10-13 the owner selected **Rust for the distributed CLI**. Python remains
the offline reference for independent models, fixtures and policy contracts.
FM1-027 establishes the measured native foundation; subsequent ports proceed
one scoped ticket at a time. FM1-033 provides offline USB-C read/capture/storage,
platform result contracts and an isolated actual USB binding with macOS and
Linux arm64 compile evidence. Native connections remain closed pending route,
loader, storage provenance and platform qualification.

**Additional goal: an AI coding agent can eventually use the CLI to install a development build and check whether that exact build boots and passes declared health checks.** Start with an offline lifecycle harness; physical development cycles require a separately authorized spare/external bench. The CLI must enforce its policy independently of the caller. This goal does not authorize hardware access or experimental writes on the owner's only FM-1.

**Single-device rule: no development, fault-injection, experimental firmware, or recovery-write tests on the owner's only FM-1.** All implementation starts offline. A dongle and a backup improve recoverability but cannot guarantee that a single-bank device will never become temporarily unbootable. If absolute zero brick risk remains the requirement, hardware writes stay disabled.

## Documents

1. [GitHub research and source inventory](01-github-research.md): live source snapshots, installation/recovery/rollback findings, contradictions and coverage limits.
2. [Implementation plan](02-cli-and-web-plan.md): Transporter fork, CLI architecture, direct transition policy, OS support and step 2 web feasibility.
3. [Single-device safety and hardware preparation](03-single-device-safety.md): enforced development limits, recovery kit, backup requirements and release gates.
4. [Backlog tickets](04-backlog.md): ordered, locally recorded tickets with dependencies and acceptance criteria.
5. [Implementation progress](05-implementation-progress.md): completed offline work, reproducible checks and remaining gates.
6. [Direct USB ROM feasibility](06-usb-rom-feasibility.md): native platform gaps, fake read prototype and separate follow-on proposals.
7. [Platform packaging evidence](07-platform-packaging.md): audited repeatable wheel, actual interpreter/install checks and remaining OS/native gates.
8. [Agent-driven firmware validation goal](08-agent-firmware-validation.md): future build → plan → flash → verified boot → health-check cycle, machine-readable results and external-bench safeguards.
9. [Unbricker reuse review](09-unbricker-reuse-review.md): current upstream comparison, useful implementation references and boundaries that must remain stricter in this CLI.
10. [Offline agent lifecycle contract](10-offline-agent-contract.md): FM1-024 interfaces, stage evidence, failure stops and actual validation boundaries.
11. [CLI distribution options](11-cli-distribution-options.md): Python bundle tradeoffs, the owner's Rust decision and remaining measurement/release work.
12. [Native Rust foundation evidence](12-native-foundation.md): FM1-027's measured macOS artifact, contract parity and limits of the first native slice.
13. [Native parser and codec evidence](13-native-parser-codecs.md): FM1-028's pure Rust APIs, independently expected vectors and remaining real-fixture/profile/IO gates.
14. [Native profile and planning evidence](14-native-profiles-planning.md): FM1-029's strict schemas, exact simulation/direct/historical plans, canonical bindings and persistence/hardware limits.
15. [Native backup and model writer evidence](15-native-backups-model-writer.md): FM1-030's reviewed regular-file storage, saved historical verification, concrete fakes and independent fault/authority checks.
16. [Stock USB-C backup route research](16-stock-usbc-backup.md): FM1-033 source and route evidence, separating unknown normal-stock raw reads from temporary ROM/RAM-loader entry.
17. [Native USB-C backup implementation](17-native-usbc-backup.md): FM1-033 offline BOT read boundary, pure route assessment, validation and remaining adapter/storage gates.
18. [Native USB backup storage](18-native-usb-backup-storage.md): FM1-033-C fresh USB captures, two-copy file verification, private provenance and historical-only saved acceptance.
19. [Offline native read adapter contracts](19-native-read-adapters.md): FM1-033-D1 macOS/Linux completion validation and ordered two-read replay, platform constraints and remaining stock-device backup gates.
20. [Stock entry and RAM-loader audit](20-stock-entry-loader-audit.md): exact MIDI ROM-entry candidate, version/framing limits, loader provenance and unresolved persistent effects.
21. [Injected native read driver and capture](21-injected-native-read-driver.md): FM1-033-D2 session/callback orchestration, secure schema-2 capture, pure entry assessment and remaining native/physical gates.
22. [Actual USB binding implementation](22-actual-usb-bindings.md): isolated nusb code, closed activation, strict queue/BOT tests, macOS/Linux arm64 compilation and pending platform/storage evidence.
23. [Exact loader static follow-up](23-loader-static-followup.md): independent inert decoding, protection/status-register diagnostics and unresolved transitive effects.
24. [Rescue script source review](24-rescue-script-review.md): pinned cross-repository bootstrap/Python evidence, macOS access and loader-lifetime references, and bounded carry-over into FM1-033-D2.
25. [Stock version identification](25-stock-version-identification.md): normal MIDI identity query, strict response interpretation and limits of USB revision/editor labels.
26. [Native read-only MIDI identity](26-native-midi-identity.md): FM1-034's optional macOS CLI, explicit endpoint selection, offline validation and first separate V15 hardware observation.
27. [Vendor loader source/effects follow-up](27-loader-source-effects-followup.md): matching official SDK distribution, source/library leads and unresolved exact binary paths.
28. [First-backup readiness and Linux build evidence](28-first-backup-readiness.md): isolated Linux arm64 validation, current owner-device evidence and the next no-write route gates.
29. [Exact loader startup paths](29-loader-startup-callpaths.md): entry/device-table resolution, runtime configuration and concrete conditional status-write branches.
30. [Exact loader read/return paths](30-loader-read-return-callpaths.md): FD05 backend, write-capable FC0A, scoped FC0C reset and fresh-data/lifecycle gates.
31. [Loader call-path audit and backup decision](31-loader-callpath-audit.md): existing vendor decoder, independent byte/boundary checks and updated work needed before a real macOS backup.
32. [Conditional no-status-write startup](32-loader-no-status-start-profile.md): exact mode/configuration predicate, observe-before-entry route and inert default witness.
33. [Read session without FC0A](33-loader-no-fc0a-read-session.md): typed metadata candidate, exact non-continuous read selector and finite checkpoint/storage schedule.
34. [No-status backup profile decision](34-no-status-backup-profile-decision.md): selected conditional start/read candidate and offline R1–R4 implementation order; activation remains closed.
35. [Native saved start-context assessor](35-native-start-context-assessor.md): R1's strict bounded Rust default/SPI-mode-2 subset, independent synthetic vectors and saved-only conditional authority boundary.
36. [Native fixed metadata validation](36-native-fixed-metadata-validation.md): R2's typed saved 84-byte FD07 observations, R1 linkage, cached chip/state checks and private stable-pair comparison; hardware activation remains closed.
37. [Native finite fake backup session](37-native-finite-backup-session.md): R3's concrete actor, two fresh modeled passes, finite checkpoint/metadata budgets and cooperative modeled storage; physical/native/durable-file authority remains unavailable.
38. [Native finite backup wire engine](38-native-backup-wire-engine.md): R4-A's private fixed FD07/FD05 engine and actual adapter typecheck, initially closed storage receipt, independent source review and staged gates toward the first physical backup.
39. [Native secure-file integration](39-native-secure-file-integration.md): R4-B's real protected two-copy files, private worker/session provenance, finite checkpoints, in-memory final continuity and unchanged hardware activation gates.
40. [Exact V15 entry and first-backup route](40-v15-entry-rom-backup-audit.md): manufacturer package acquisition, canonical empty-log parser fix, corrected RAM/reset callee, ROM evidence gaps and reviewed one-time backup alternatives.
41. [ROM FD07, macOS entry and stock return](41-rom-fd07-macos-entry-return.md): rejected foreign ROM image, exact selected-open/cancel effects, installed Apple packetizer findings, manual return candidate and updated offline CLI discovery.
42. [Targeted WL82 ROM and external backup evidence](42-targeted-wl82-rom-external-evidence.md): requested-source applicability, actual Linux loader callbacks, Windows full-read report, independently matched alternative SDK helper, macOS recovery leads and the remaining no-write gaps.
43. [Standalone full raw dumper](43-full-raw-dumper-product.md): complete physical-byte contract, pre-installation stock priority, local Felucca/WebApp source cross-check, USB_KEY documentation conflict and 033-F integration plan.
44. [Offline raw-dump CLI](44-offline-raw-dump-cli.md): full-byte saved-file inspection/SHA verification, optional file comparison, secure local IO, historical-only results, actual default-executable checks and unchanged hardware gates.
45. [Official 30 KiB helper no-status/read audit](45-official-30k-loader-no-status-read.md): exact startup/configuration, local FD05 without FC0A, new fixed metadata layout, conditional status-write/return hazards and finite first-read evidence gates.
46. [macOS packetizer and V15 receiver](46-macos-entry-packetizer-v15-receiver.md): concrete class/device/vtable binding, conditional eight-byte output, exact V15 endpoint/DMA/aggregation path and lower-layer error traffic; physical entry and ROM-read gates remain open.
47. [ROM FD07 effects and RAM ownership](47-rom-fd07-effects-ram-ownership.md): new OTA archive-body evidence, exact V15 interval/lifetime checks, logical versus touched-memory geometry and the matching-ROM evidence dependency before first context capture.
48. [Fresh online evidence and backup-route reassessment](48-online-rom-evidence-route-reassessment.md): new bare-ROM32B observation, deterministic-helper double-read reports, existing-tool comparison and a finite alternative-route audit; A/B observation is route-specific, with no hardware activation.
49. [Lunar deterministic backup-route audit](49-lunar-deterministic-backup-route-audit.md): exact patch/native verification, local startup/read without A/B pre-read or FC0A, separate 256-byte candidate and the remaining matching-startup/handoff dependency; no profile activation.
50. [ROM initializer, memory and callback contracts](50-rom-initializer-memory-callback-contracts.md): exact caller state, 32-byte interface/handler-slot/buffer/stack contracts, narrower padding/sender lifetime requirements, external evidence applicability and why USB installation is a different operation from original-stock raw backup.
51. [Online matching-ROM and no-FC0A evidence](51-online-initializer-no-fc0a-evidence.md): official artifact introduction history, additional SDK mirrors, refreshed Lunar sources and concrete fm1-ctl/Flipper read command paths; selected startup/handoff qualification remains open.
52. [M-Flasher source-defined reader route](52-mflasher-source-reader-route.md): positive source/binary startup/read audit without the vendor initializer/FC0A, exact memory/host differences, reduced read-only implementation and a bounded first-hardware-test plan.

## Current owner and workspace state

- One FM-1; the browser and a separate single CLI identity query report V15 (`FM-1_015`) on the owner-reported macOS 27 / Apple Silicon. Firmware integrity, board and flash-chip identity remain unverified; see [report 26](26-native-midi-identity.md).
- Owner reports no full backup and no recovery dongle yet, and is willing to build one.
- Recommended board: **standard Seeed Studio XIAO RP2040, SKU 102010428**. [Official product](https://www.seeedstudio.com/XIAO-RP2040-v1-0-p-5026.html), [Transporter recommendation](https://github.com/kurogedelic/FM-1-transporter).
- This directory was empty when researched. Implementation now has its own Git repository, based on upstream commit `a632d923203170e3a08565f4056dbb3fd3d65ca5`, initially on branch `kori/offline-flasher`; current implementation work uses `init`.
- The original foundation fork is [kmplngj/FM-1-transporter](https://github.com/kmplngj/FM-1-transporter). On 2026-10-13 the owner requested an independent implementation repository: [kmplngj/fm1-flasher](https://github.com/kmplngj/fm1-flasher), created private with work on `init`. `origin` now points to that repository; `transporter-fork` retains the foundation fork and `upstream` retains `kurogedelic/FM-1-transporter`. Existing history, firmware and submodule pins are preserved.
- On **2026-10-15**, the owner authorized hardware access for backup work and explicitly prohibited flash writes. This supersedes the earlier blanket access prohibition; exact target, evidenced read path and reviewed entry/loader effects remain gates. FM1-012 kit/entry evidence remains incomplete.
- FM1-033 begins with local files, synthetic fixtures and injected fakes only. No hardware enumeration, connection, probing, loader execution or physical read is part of this implementation. Full raw backup from the normal stock application remains unknown; a temporary ROM/RAM-loader route is a distinct capability. Real adapters and all physical writes remain disabled.
- The native scope includes concrete-fake USB BOT reads, pure route/entry assessments, USB-specific two-copy storage, macOS/Linux result contracts, finite replay and an injected session/callback driver. Driver capture uses separate schema-2 fictional provenance; saved verification remains historical-only. Standalone saved replays cannot mint current capture authority. A separate opt-in `native-usb` package now contains actual nusb calls, compiled locally on macOS and tested through injected bulk/queue seams. Its public factory always refuses; actual compilation also passed on isolated Linux arm64. R4-B now integrates protected two-copy files and private session provenance offline (report 39); final session observations remain in memory. Other target builds and physical qualification remain pending. The exact loader audit now resolves startup/read/return paths and identifies conditional status-write attempts in startup and ordinary FC0A; the unchanged upstream lifecycle is not qualified for the no-write backup route. A conditional observe-before-entry, mode-2 read profile without FC0A is documented in report 34; ROM/handoff/platform evidence and implementation remain gates. Actual native connections, entry/loader execution, live capture and USB writer authority are unavailable. Existing fictional Transporter storage remains separate.
- Tickets are recorded locally; no external issue tracker tickets have been created.
- Manufacturer V15 package bytes are now retained and exactly hash-bound (report 40). They confirm the MIDI special branch through its local RAM mailbox/P33-reset chain and correct an earlier ROM-address annotation error. They are not a full device backup or installed-image attestation. Reset-to-ROM effects, exact ROM FD07 A/B ownership, macOS framing/access/return and loader handoff remain operation-specific gates; hardware activation stays closed.
- Targeted online research supplies real external loader-read observations and a reported full double read (report 42). These use FC0A; the observed successful Windows return follows firmware writes. Report 45 now audits Melodee's official-matched 30,720-byte helper offline: conditional mode-2 startup and positive FD05 without FC0A are local candidates, with different configuration/88-byte metadata addresses. Startup, FC0A and some FC0C cleanup branches still include status writes. Bare-ROM selected-read/ownership, macOS entry/access/return and external startup/chip/timing evidence remain open; old profiles cannot be reused by changing a hash. No helper switch, activation, new hardware access or full backup occurred.
- Report 46 resolves the installed-file legacy packetizer's class/device path and the exact historical V15 OUT4 descriptor/IRQ/DMA/receiver. A quiet six-byte legacy send with the selected capacity/cable conditions can produce the required eight bytes; V15 aggregates currently ready data rather than preserving host submission boundaries. Driver-owned error retries and endpoint halt clearing remain explicit. Matching physical endpoint/object/delivery and reset-to-ROM/read-restoration evidence are still missing; no entry, capture or write factory is activated.
- Report 47 checks previously unexamined OTA library bodies and exact V15 RAM occupancy. Neither supplies matching bare-ROM FD07 success/failure restoration or active allocation evidence. Selected A/B are application data before reset; the optional helper's A includes mutable state. C2 now has an explicit matching-artifact/external-evidence dependency; another scan of the same local material is not a physical milestone. Logical default-case byte minimization does not narrow unknown target memory effects or change the implemented1058-byte capture. All activation remains closed.
- Report 48 adds a narrow external bare-ROM32B read and reported full double reads with a different deterministic patched helper. These are useful new evidence, with private raw captures, different context/firmware/OS and unresolved startup/status effects; later stock return follows writes. Existing tools do install/flash/recover/dump, but none inspected is qualified for our strict original-V15/macOS backup conjunction. R4-C-P1 now prioritizes a finite offline comparison that may remove the optional A/B pre-read; no patch, profile or hardware capability is adopted. Matching ROM code/active allocation remains unavailable within the inspected scope.
- Report 49 completes that bounded P1 audit offline: the exact 16-byte patch and candidate hashes reproduce independently; selected local startup/read needs neither A/B observation nor FC0A. A future 256-byte profile is preferred to avoid unqualified intra-request double-send/scratch reuse. Existing R2's G+4 linkage/digest and 512-byte contracts cannot be reused unchanged. P2 records the remaining external initializer/entry ABI/memory/callback dependency; selected chip, Mac access/timing and stock-return qualification remain open. No runtime helper/profile, factory or physical capability changed.
- Report 50 records the bounded offline P2 audit while leaving qualification open: no matching ROM 0x01E00428 body was obtained. The selected path requires 32 readable interface bytes, a separate writable hook slot and specific stack/CBW/answer/scratch/IRQ lifetimes. It needs only the sender body; the receiver target is unused, and zero padding is dead before the ROM call. Async consumption-before-reuse can qualify without globally synchronous DMA. Existing full-read reports materially support usability but include FC0A and omit the selected metadata read. Saved Felucca 1.6.1 source explains its host-package service versus our original-stock raw capture. No P3, repeated-scan milestone, profile or factory activation is introduced.

- Report 51 completes a finite online P2 refresh. Official WL82 stub/image histories and two additional mirrors supply no matching initializer body; fm1-ctl/Flipper backup sources explicitly use FC0A. Fresh Lunar branch records add a diagnostic-ring change, without a new no-FC0A/84-byte campaign. Existing successful full dumps remain meaningful; precise startup/status/memory/sender evidence is still required. No repeated-search milestone, profile selection or factory activation.
- Report 52 positively audits a source-defined alternative: M-Flasher startup/raw03 reads avoid the vendor initializer and FC0A, corroborated by both small prebuilt images. A minimal read-only derivative and strict host route are preferred next; the combined upstream writer, fallback/retry/reset and single-read/CRC storage are not adopted. M1/M2 lead to a separately scoped small hardware read/return test (M3), then full double-read/two-copy acceptance. Existing runtime gates remain closed; vendor-specific debug/config/84-byte metadata requirements are not transferred.


## Completion boundaries

The findings and backlog are reviewable design artifacts, not a tested flasher. Hardware support must be recorded separately for each transport, OS, hardware revision, exact firmware/package and recovery scenario. CI and simulations never count as physical-device verification.
