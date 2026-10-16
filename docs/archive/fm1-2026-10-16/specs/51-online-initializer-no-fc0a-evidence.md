> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Matching WL82 ROM and no-FC0A backup evidence — R4-C-P2

Date: **2026-10-16**, branch `init`. Targeted online primary-source research
with GPT 6.1 Sol high ROM/external-evidence agents and a medium independent
search/review agent. Continues [report 50](50-rom-initializer-memory-callback-contracts.md).
Network reads were explicitly requested. No hardware inventory/open/query,
MIDI client, target execution/emulation, fetched-module import/execution,
firmware script/build, backend installation, remote mutation or flash write.

## Result and decision

**The finite online investigation is complete; P2 qualification remains open.**
No matching implementation/effect contract for **ROM 0x01E00428**, actual
selected ROM allocations/sender ordering, or external campaign for the exact
deterministic helper **without FC0A** was obtained within the inspected scope.
This is neither global absence nor evidence that the candidate is unsafe.

New findings narrow the available leads:

- Official WL82 mask-stub/image path histories already contain the same
  empty stub and rejected foreign image at their introduction revision.
- Two additional revision-bound SDK mirror trees and an official mask API
  header supply declarations/configuration, without the selected ROM body.
- Newly inspected `fm1-ctl` rescue source and refreshed Flipper backup source
  both explicitly send FC0A before/during reading.
- Lunar's relevant branch pins remain unchanged. Its newer USB_KEY diagnostic
  ring change supplies no new backup or ROM-startup qualification.

Existing successful full FM-1 reads remain meaningful positive evidence.
The missing evidence concerns the selected startup/handoff and no-programming
subset. No helper/profile is adopted, no runtime factory is enabled and no
replacement P3 or unchanged-search milestone is introduced.

## Official ROM lead: history and empty wrapper

The official
[Bootloader revision](https://github.com/Jieli-Tech/fw-Bootloader/commit/ff3b9299d1b94d834b995fd858b3f75e2faa9840)
binds actual tree `d3d521fc6365b86198d6f5245e22787faf52a612` (386 entries,
non-truncated). Both selected path histories return one introduction revision,
**38e6ff29b990ea3fe3819fb621bb882812abd613, 2022-03-24**:

| Path under `user_boot/cpu/wl82/output/` | Introduction/current Git blob | Consequence |
| --- | --- | --- |
| [`maskrom_stubs.ld`](https://github.com/Jieli-Tech/fw-Bootloader/blob/38e6ff29b990ea3fe3819fb621bb882812abd613/user_boot/cpu/wl82/output/maskrom_stubs.ld) | `e69de29bb2d1d6434b8b29ae775ad8c2e48c5391`, 0 bytes | No useful earlier version along this inspected path history. |
| [`rom.image`](https://github.com/Jieli-Tech/fw-Bootloader/blob/38e6ff29b990ea3fe3819fb621bb882812abd613/user_boot/cpu/wl82/output/rom.image) | `17a2b10c3d0c21b3a6116f8c978e2697ec910b52`, 131,072 bytes | Same already rejected BR28-identical artifact from reports 41–42. |

This checks the named default-branch path histories, not deleted/renamed paths,
every branch or unpublished releases. The image was not downloaded/decoded
again.

The official
[`mask_api.h`](https://github.com/Jieli-Tech/fw-Bootloader/blob/ff3b9299d1b94d834b995fd858b3f75e2faa9840/user_boot/include_lib/cpu/wl82/mask_api.h)
contains an **empty `uboot_mask_init` wrapper** and declarations without the
required address binding. Its 1,089 bytes hash to
`[digest omitted from public archive]`;
Git blob `6a8675990a06a535556a9bd7a2a1d49eeadeeff1`.
An empty SDK wrapper does **not** establish that the binary ROM call is a
no-op. Different-family prototypes do not establish its formal signature.

Two additional complete mirror listings were inspected for scoped
CPU/ROM/mask/API/stub/boot/linker paths:

| Mirror revision | Actual tree / entries | Finding |
| --- | --- | --- |
| [ComputingPlay `918168e3…`](https://github.com/ComputingPlay/fw-AC79_AIoT_SDK/commit/918168e385418f9122a5891f74af42d404bbe9a8) | `5c89b2f3b02ecf3169f5b8324d195689668e37c7` / 10,695 | Startup/NOR headers are configuration/declarations; no matching ROM body/map in the scoped paths. |
| [msr002 `e30b1ee3…`](https://github.com/msr002/fw-AC79_AIoT_SDK/commit/e30b1ee375d1f2993fc23bf92c8b99006a6e5f9d) | `98fbac6b62aba5acd5181d51327c58f62b30d73f` / 15,140 | No matching body/map in the scoped paths. |

Both list `truncated:false`. Neither is an archive-wide body audit or
manufacturer qualification. Exact/shortened initializer/callback addresses,
chip names and configuration-key searches produced no further matching lead.
No new binary required native analysis; no REA/decoder session was started.

## External tools: actual command paths

Newly inspected
[`fm1-ctl` rescue source](https://github.com/charlesvestal/fm1-ctl/blob/de372e7cec8c10c8c18a7765fd89bf9c988edaaf/tools/fm1_rescue.py#L199-L250)
has SHA-256
`[digest omitted from public archive]`.
It pins the original `d41da612…` helper, uploads/jumps, then unconditionally
calls `chip()` containing **FC09 → FC0A** before the full read. Its host
`--write`/confirmation branch occurs later. Thus the check/no-host-write mode
does not supply the selected patched startup or a no-FC0A campaign.

Refreshed
[Flipper `pc_backup.py`](https://github.com/merthsoft/flipper-fm1-recovery/blob/df88b862a4794855f792fed529629a255cba0a0e/tools/pc_backup.py#L103-L137)
has SHA-256
`[digest omitted from public archive]`.
Its read-only allowlist includes FC0A, and its source calls `online_device()`
at startup, while reading, and during keepalive. The related external
two-equal-read report remains useful; this source does not qualify our
no-FC0A sequence. No fetched source was imported or executed.

Host erase/program gating is valuable, but does not by itself exclude target
helper startup or NOR status-register programming. Conversely, seeing FC0A in
source is **not an observation of physical status writes** in this turn.
Reports 33/45/49 establish why its target-side effects matter to the selected
constraint.

Explicit commit → tree → source-blob bindings verify these two files. Some
initial `/git/trees/HEAD` responses echoed commit IDs as top-level `sha`;
those fields were not used alone as tree identities. Additional commit and
exact-tree GETs resolved the distinction; source entry arrays/blobs agree.

## Lunar refresh and retained physical evidence

All **407 public Lunar branch heads** were listed in five API pages. Relevant
main/cold-ROM/debug heads remain `bfa04244…` / `53fa05b2…` / `c6c4e941…`.
Fresh commit records also confirm unchanged M-Flasher, SMK, Unbricker and
Transporter heads; unchanged implementations were not rescanned as new work.
The selected patch and physical-backup-note Git blobs remain unchanged.

The newer
[USB_KEY event-history change](https://github.com/ip2k/lunar-modulator/compare/c6c4e941873a0dab68d4504f239a78f4a8aa5850...eb26a73e648d4536bfd7362fbffa2097baf99356)
changes four diagnostic gateware/test/README files. Its
[README](https://github.com/ip2k/lunar-modulator/blob/eb26a73e648d4536bfd7362fbffa2097baf99356/tools/cynthion_usb_key/README.md)
describes a compact ring representation, explicitly distinct from continuous
line sampling or hardware acceptance. A separate source freshness fix also
adds no physical startup/backup receipt. Neither supplies the missing ROM
initializer/send body or allocation/ordering contract.

The existing exact-helper
[physical-backup report](https://github.com/ip2k/lunar-modulator/blob/53fa05b278d217476b7e92ba24d6d5dd1e1e1010/notes/2026-10-11-fixed-loader-physical-backup.md)
still materially corroborates upload/start and **two equal complete 1 MiB
reads, 8,192 × 256-byte FD05 requests**. It uses candidate encoded hash
`[digest omitted from public archive]`,
but includes **515 FC0A**, one FC14 and **no fixed 84-byte loader FD07**.
This is not new evidence, a no-FC0A/status-write proof or an independently
inspected private capture. Later stock-return evidence follows writes on
Linux/FM-1_092; it does not qualify backup-only return on the owner's V15/Mac.

## Community sources and unavailable leads

The refreshed
[new-unit backup discussion](https://www.reddit.com/r/MVaveFM1/comments/1x4fnhm/best_way_to_take_a_full_device_backup_of_a_new/)
recommends the software MIDI entry and a 1 MiB `jl-uboot-tool` read. The
[soft-brick author's report](https://www.reddit.com/r/MVaveFM1/comments/1wz2gnj/softbricked_fm1_wl82_uboot100_successfully_dumped/)
reports a successful full SPI dump. These support practical USB-only access;
the returned pages supply no exact helper digest/no-FC0A transcript,
startup/status analysis, actual RAM allocations or sender ordering. The
first is advice, not a newly verified successful stock-V15 campaign.
Collapsed/unreturned replies were not treated as inspected.

The indexed JL SoC forum check and its
[last page](https://esp8266.ru/forum/threads/jl-soc.5500/page-41)
add no matching selected-ROM evidence in the returned material. This is not
a fresh inspection of all 41 pages. Other chip-family anecdotes are not
transferred to WL82.

An indexed `kurogedelic/fm-1-research-lab` snippet looked relevant, but GitHub
page/API and raw-main README retrievals returned **404**. No source body or
ROM qualification is inferred from that snippet; the repository may have
been renamed, removed, made private or otherwise unavailable to these reads.

## Exact retained dependency and next useful input

The [P2 contract](04-backlog.md) and report 50 remain authoritative. Matching
implementation/effects **or scoped external qualification** must cover:

1. The selected 24,064-byte deterministic helper's ROM call at 0x01E00428,
   critical live state and required startup effects without prohibited
   flash/status programming.
2. Actual entry/stack/IRQ, hook, command/answer/scratch/state ownership and
   selected send(256)/send(84) consumption before reuse and next-BOT/error
   progression. Report 50's 32-readable-byte interface and separate writable
   hook slot remain the exact local requirements.
3. The selected no-FC0A path. A host transcript can exclude that command;
   startup/status effects also need qualified target-side evidence.

Unused receive-body/nonzero-target checks, dead padding after the final
lookup, 512-byte reads, complete ROM reconstruction, prior backup,
write/restore trials and all-OS acceptance are **not** added as requirements.
Chip/capacity/freshness, selected macOS access/continuity, bounded timing,
durable fresh double reads/two copies and original-stock return remain the
existing separate acceptance stages.

Another unchanged local scan or reclassification of a successful installer
cannot close this external dependency. A new matching artifact or scoped
qualification addressing the above would change the decision. A separately
audited source-defined reader could be another route, but this report does
not select, implement or qualify it.

## Evidence and validation

The sanitized evidence manifest (historical reference `evidence/51-online-initializer-no-fc0a.json`; omitted from public archive)
binds saved sources/receipts, revisions, hashes, scope and false capability
flags. Ignored research files reside under `[local artifact location omitted]`.
HTTP bytes, connector-returned UTF-8 and web-tool extracted text are labeled
separately; none is a signed release or local hardware observation.

File hashes, relevant Git source-blob/commit/tree identities, local links,
JSON and scoped diffs are checked, followed by independent review. Product
tests are not rerun for documentation/source-retrieval-only work; earlier
test counts remain historical. Product code/profiles, firmware, PIO,
submodule pins, license notices and all physical write gates are unchanged.
No implementation is copied or binary redistributed. Work stays local on
`init`, without push; unrelated rescue-source documentation remains separate.
