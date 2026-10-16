> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# ROM startup and memory/callback contracts — R4-C-P2

Date: **2026-10-16**, branch `init`. Offline inspection with GPT 6.1 Sol high
agents for initializer/callback analysis and a medium independent verifier.
Continues [report 49](49-lunar-deterministic-backup-route-audit.md). Saved
artifacts and original file-only checks only: no network, device inventory,
open/query, vendor-module execution, firmware scripts/builds, target execution
or emulation, installation, flash write or remote mutation.

## Is a backup harder than Felucca's USB installation?

**Raw reading is not fundamentally harder than writing. Existing tools already
demonstrate full FM-1 dumps. Our extra work concerns original-stock access and
the no-flash/status-programming startup requirement.** A USB-only installer
does not automatically provide the reverse data path or that same contract.

The exact saved
[Felucca README](https://github.com/hugelton/Felucca/blob/df4498e8d95966853ac38643c146919268e68aa1/README.md)
contains the owner's **1.6.1** wording. Its
[CLI installer](https://github.com/hugelton/Felucca/blob/df4498e8d95966853ac38643c146919268e68aa1/tools/fm1_install.py#L286-L365)
explains and implements the same device-pull package service as the web path:
the FM-1 requests offsets in the **host's logical package image**, and
`serve()` answers with `image[addr:addr+n]`. Its second stage intentionally
writes firmware; its final `verify()` checks the returning identity. It does
not read back/hash the entire physical device flash. This source observation
does not requalify every possible installation path or the current live site.

| Existing operation | Data direction and result |
| --- | --- |
| Felucca installation | Host package → device updater → intentional firmware writes/restart. |
| [Felucca musical archive](https://github.com/hugelton/Felucca-WebApp/blob/884928139b1cc62122973c2ef33bf87c99645542/fm1backup.js#L1-L11) | Supporting installed firmware → host; named musical objects, not arbitrary physical addresses. |
| Requested original-stock raw backup | All physical flash bytes → host before installation, through a separately qualified entry/read route; preserve original flash/status. |

We have not established a complete raw-export endpoint in running V15. This
is not a proof that every undocumented stock command lacks one. Installing
Felucca to obtain its archive would change the pre-install state being sought.
No RP2040 is inherently required by the successful plain-USB read protocol.

## Result of this P2 pass

**The bounded offline investigation is complete; the P2 qualification remains
open.** No matching body/effect contract for ROM 0x01E00428 or exact ROM
allocations/sender consumption was obtained. There are meaningful new caller
and memory facts, and several earlier requirements can be narrowed:

- Only the selected **send** callback needs body/consumption qualification;
  the receive field is read/copied but its target is never called here.
- The zero configuration terminator is needed only through the final lookup,
  **before** the ROM call. Post-call padding immutability is unnecessary.
- Actual entry needs **32 readable interface bytes**, not just its 12-byte
  functional fields, and writes through a pointer to a handler slot.
- Async send is acceptable if selected ordering preserves bytes until consumed;
  globally synchronous DMA is not required.

The deterministic 256-byte candidate, current R1–R4-B and runtime factories are
unchanged. No new helper, profile, payload, generic diagnostic or P3 is added.

## Exact initializer call: observed state versus unknown API

The 24,064-byte candidate decoded SHA-256
`[digest omitted from public archive]`
contains unchanged bytes `8fea4bee` at **0x01C0278E**. Independent signed 21-bit
halfword arithmetic gives byte displacement 2,088,086 and target
**0x01E00428**, outside the upload `[0x01C02000,0x01C07E00)`.

| Selected deterministic callsite fact | Limit |
| --- | --- |
| Residual r0=0, r1=1, r2=0xFFFFFFD0 after EX_FLASH_IO absence/default processing | These are observed caller register values, not a recovered formal signature or semantic function name. |
| r0 is overwritten immediately at return PC 0x01C02792 | No result is consumed or tested; ignoring it does not establish harmlessness or successful initialization. |
| r6=0 and r9=0x01C0797F are reused immediately; r8/r10/r11 and SP/frame remain live | The caller needs those values and critical configuration/code/handoff state preserved. The callee's actual preservation, stack use and side effects remain unknown. |
| SP at the call is entry SP−72 | Entry's earlier 12-byte veneer save has already been restored. Callee/interrupt requirements add to this frame. |

The saved official 30,720-byte helper also calls this address, from 0x01C02472.
It performs an additional **LATCH_DLY_RELEASE** lookup immediately before it,
unlike the selected 24 KiB helper. Thus neither a `void(void)` prototype nor an
EX_FLASH_IO-specific API can be inferred from the lack of explicit pointer
preparation in one caller. No arbitrary symbol name is assigned to this ROM
function; “initializer” remains a workflow label.

An ignored result does not justify deleting the call: required peripheral/state
effects may exist independently of that result. M-Flasher omits it in a
different source-defined loader; that is not evidence for NOPing this call in
the selected vendor helper. No additional patch is made.

One bounded cross-reference inventory covered **18 retained files / 155,520
bytes** of SDK headers/linkers/API/indexes and M-Flasher source/readme/linker.
It contains no exact selected-ROM address mapping or bound implementation.
SDK application linker region `rom` at 0x02000120 is external-flash placement,
not a map of immutable ROM 0x01E00428. The retained `sdk-api.txt` is a CPU
header. M-Flasher reconstructs a plug-in interface but neither invokes nor
implements this call. The rejected BR28 image and OTA archives were not
decoded again. This is a bounded unavailable-artifact result, not global absence.

## Minimum selected memory and call contracts

Notation: I=entry interface, H=u32[I+8], C=hook command buffer, A=answer,
S=0x01C09600 scratch, G=0x01C09700 state. These describe requirements derived
from local code; they are **not observed ROM allocation addresses**.

| Resource | Exact local requirement / effect | What is not established |
| --- | --- | --- |
| Entry interface I | Word loads at +0/+4/+8; naturally aligned readable I is a sufficient profile condition. Debug dump 0x01C0240C(I,32) reads all `[I,I+32)`. Only the first 12 bytes have established field meaning. | Actual I, 32-byte allocation/lifetime and owner on the selected ROM/entry; no formal architectural ABI was recovered. |
| Handler slot H | One writable word receives 0x01C03E4A through H; natural alignment is a sufficient profile condition. The old value is not saved/restored. This writes the pointed-to slot, not ordinarily I+8. | H's actual owner, non-aliasing and subsequent ROM dispatch to the hook. |
| Command C | Selected local branches read through C+22 inclusive: 23 contiguous bytes; actual BOT still uses its 31-byte wire CBW. With G+32=0, the command-transform mutation path is skipped. | ROM's full CBW/tag/header lifetime. 23 local bytes never authorize a shortened wire CBW. |
| Answer A | Before dispatch, 16 bytes `[A,A+16)` are filled from repeated reads of MMIO 0x13B00, then idle G+44 is cleared. Selected FD05/FD07 do not send A as their data payload. | Actual A allocation; safe aliasing with all still-live command/header/state/stack data. The initial bytes are not a fixed-zero response. |
| Scratch/state | S/256-byte scratch and G/84-byte metadata are adjacent/disjoint, within entry-clear `[1C09600,1C09DD0)`; both are outside the upload. | ROM/DMA/other active users granting those allocations; address arithmetic alone is not ownership. |
| IRQ5 | Helper installs vector word 0x01C7FE14 and programs timer/IRQ registers; the local handler touches counters/UART and can request idle reset. | Selected vector/stack/IRQ-mode ownership, permitted nesting and finite service timing. No all-IRQ audit is demanded. |

Own software frames are 12 bytes for the restored clear veneer, 72 for the
entry body, 360 for the hook and 28 for its timer IRQ save. The entry and hook
have nested callees; ROM, sender and exception/nesting behavior add concurrent
usage. These are not sufficient total stack allocations. The selected local
frame arithmetic preserves 4-byte alignment if incoming SP is aligned, without
proving a stronger ABI.
Entry establishes neither SP nor SSP. Qualify the actual selected stack mode
and concurrent writable envelopes; a map of every ROM stack is unnecessary.

Ordinary entry returns after timer setup, leaving r0=**0x10500**, rather than
explicitly returning success 0. M-Flasher's independently authored return 0
convention cannot replace this binary's observed return. The actual ROM
plug-in caller convention and successful dispatch, not a guessed C prototype,
must govern host/target acceptance.

### Configuration lifetime correction

All five lookups of the patched zero record precede 0x01C0278E; the last call is
**0x01C02776**, reading terminator byte **0x01C07C62** before it returns. The
later clock-name helper 0x01C0251C does not fetch that configuration. The selected
remaining startup/FD05/fixedFD07/timer paths do not reread the padding.

Consequently the zero terminator must remain valid **through that last lookup**.
Afterward the live requirements concern resulting critical G/configuration,
code, stack/interface, buffers and controller state. Requiring the later ROM
initializer to preserve already-dead padding adds no protection to this subset.
This narrows report 49/P2's earlier broad padding-preservation wording. It does
not show that the initializer preserves the live fields or avoids flash writes.

## Sender contract and what external results already support

FD05/256 calls the supplied G+36 target with **(S,256)**; fixed loader
FD07(G,84) calls it with **(S,84)**. Both discard the sender return and reach
handled result r0=1. The FD05 backend result is also ignored. “Handled” does
not prove fresh NOR bytes, send success, DMA completion or a good CSW.

The exact lifetime condition is that every payload byte be consumed/copied
**before the next write to reused scratch**. Synchronous send, a completed copy
into separately owned storage, or qualified async serialization can satisfy
it. Callback return alone cannot. For 256-byte requests there is one send and
no same-request refill, so only selected next-command/ROM completion ordering
is needed; the 512-byte double-send path is excluded. Cover relevant error,
timeout/cancellation and stack/register/state preservation at this boundary.

The receive field G+40 must be readable/copyable from I+4. Its target need
neither be nonzero nor be called/qualified for this subset. A future exact
profile can express that narrower contract; current R2 validation is unchanged.
Fixed 84-byte **loader** FD07 remains selected; bare-ROM FD07/A/B is not reintroduced.

The retained
[Lunar physical report](https://github.com/ip2k/lunar-modulator/blob/53fa05b278d217476b7e92ba24d6d5dd1e1e1010/notes/2026-10-11-fixed-loader-physical-backup.md)
is important positive evidence, not “nothing”: it binds the exact encoded
candidate `fe6d1bd3…` to reported upload/start and **8,192 × 256-byte FD05**,
two complete equal 1 MiB reads on an external FM-1/Linux campaign. The pinned
reader source hash matches the reported reader. Inventory arithmetic is 8,756
vendor requests, 8,757 including inquiry.

That sequence includes **515 FC0A** and FC14, excludes the proposed fixed 84-byte
loader FD07, and expressly leaves initializer/supplied operations unresolved.
Its private raw captures/dumps/register allocations were not inspected here.
It supports practical entry/read/send functioning on that campaign, while not
establishing absent status programming, the exact no-FC0A subset or all required
ownership/ordering. Missing evidence is not proof that the candidate is unsafe.

## Disposition: keep P2 open on the actual missing input

P2 stays **Needs matching artifact/external qualification**, with this offline
callsite/contract audit recorded. The smallest useful input is matching
implementation/effect evidence or a source/digest-bound external qualification
covering the selected initializer, live-state/stack/interface/slot/buffer/IRQ
ownership and send 256/send 84 consumption/next-BOT ordering, including relevant
failure behavior and absence of prohibited flash/status programming.

Complete ROM reconstruction, qualification of the unused receiver or 512-byte
path, post-use padding immutability, a prior backup, write/restore experiments
and every OS are **not prerequisites**. No universal zero-risk claim is sought.
Existing actual-chip/freshness/capacity, selected macOS access/continuity,
finite timing/cancellation, two fresh full reads/two copies and observed
original-stock return remain separate selected-route requirements.

Do not schedule another unchanged local scan as a new milestone or invent P3
to rename this dependency. Saved-only profile implementation cannot supply the
missing physical/effect evidence. No owner-ROM extraction/probe or alternate
payload experiment is performed or authorized here.

## Evidence and validation

Sanitized manifest (historical reference `evidence/50-rom-initializer-contracts.json`; omitted from public archive) binds exact
source, binary/listing, call arithmetic, 12 byte anchors, inventory and original
checker digests; raw inputs/notes stay ignored under
`[local artifact location omitted]`. Original initializer and callback checkers both
passed; the latter checks all 9,771 retained listing rows against the candidate.
An original AST/text check confirms saved installer package-serving and
identity-verification semantics without importing/running it. Independent
medium review distinguishes external usability from no-programming proof.

No new native analysis session or decoder execution was needed; report 49's
unsupported-architecture result and static listings were reused. Documentation,
JSON/file-digest/local-link/scope/diff checks apply; product suites are not rerun
for this docs/static-evidence-only increment. Product code, profiles, firmware,
submodules and notices remain unchanged. No accepted backup or physical return
was observed. Local `init`, no push; unrelated rescue-source WIP preserved.
