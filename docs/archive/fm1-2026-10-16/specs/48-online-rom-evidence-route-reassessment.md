> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Fresh WL82 evidence and a practical backup-route decision — R4-C

Date: **2026-10-16**, branch `init`. Requested online primary-source research
with GPT 6.1 Sol high agents for ROM/external evidence and a medium agent for
the existing-tool comparison and independent review. Internet access fetched
inert source, reports and metadata only. No hardware inventory/open/query,
MIDI/USB/SCSI operation, target execution/emulation, vendor-code execution,
firmware build, backend installation, remote mutation or flash write.

## Result and answer to the owner's question

**Yes: working FM-1 installers, flashers, recovery tools and full-flash readers
already exist. We should reuse their established protocols and audit the
smallest suitable route.** Their existence was never the missing fact. Our
remaining requirement is a complete original-stock backup over USB-C on the
owner's Mac, with no installation and no flash programming, including status
register programming, followed by a documented return.

This search provides two useful new external observations: a reported full
double read with a deterministic patched helper, and a narrow **bare-ROM**
FD07 read. It does **not** obtain the matching immutable-ROM body or active
RAM map. Public source/report integrity was checked here; the external
authors' private captures, dumps and native listings were not independently
replayed or examined here.

**The selected A/B observation is one design choice, not a universal backup
prerequisite.** A direct-upload route can omit it, but must independently
establish safe startup. The current observe-before-helper route in
[decision 34](34-no-status-backup-profile-decision.md) and implemented R1–R4-B
remain unchanged. A finite comparison of a new deterministic helper candidate
is now more useful than repeating searches of the same local archives. No
candidate is selected or activated by this research.

## New primary evidence, with phase and applicability kept separate

### 1. A real external bare-ROM FD07 observation

Lunar's pinned
[first retained diagnostic RAM read](https://github.com/ip2k/lunar-modulator/blob/53fa05b278d217476b7e92ba24d6d5dd1e1e1010/notes/2026-10-11-diagnostic-first-ram-read.md)
reports INQUIRY36 followed by a 32-byte FD07 read from **0x01C033D8**, with
zero native status/residue and a separately reviewed BOT capture. Its lineage
is a return to ROM after executing an installed diagnostic, not initial entry
from the owner's V15. No loader upload, reset or NOR command was sent in that
two-request read operation. The source records raw/decoded digests and says
the actual bytes and bus capture remain private.

This is **bare-ROM FD07**, unlike the loader-phase callback observation in
report 42's earlier Lunar snapshot. It supersedes any broader reading of
that old snapshot's “no external ROM FD07 observed” result. It still does not
establish the ROM handler's implementation, in-place restoration, DMA extent,
failure behavior, reset completion, retention cause or ownership of our A/B
intervals. The source itself expressly limits those claims. A successful
32-byte read at a different address/context cannot qualify our 1,058 bytes.

The eight reported little-endian words reconstruct the published decoded
SHA-256 `[digest omitted from public archive]`.
This checks report arithmetic, not the private wire capture or ROM cipher.

### 2. Full FM-1 double reads without the A/B pre-read

Lunar's pinned
[fixed-loader physical backup report](https://github.com/ip2k/lunar-modulator/blob/53fa05b278d217476b7e92ba24d6d5dd1e1e1010/notes/2026-10-11-fixed-loader-physical-backup.md)
reports Linux SG access, a software MIDI entry, and **two equal 1,048,576-byte
reads** matching its prior baseline. The firmware identity was `FM-1_092`,
not the owner's `FM-1_015`. The reported vendor sequence is 47 FB06 uploads,
one FB08, one FC14, **515 FC0A** and 8,192 FD05; there is **no pre-loader FD07**.
The full-read digest is
`[digest omitted from public archive]`.

The reported helper is a **different exact artifact**, derived from our
24,064-byte original: encrypted SHA-256
`[digest omitted from public archive]`,
decoded `[digest omitted from public archive]`.
The published
[profile description](https://github.com/ip2k/lunar-modulator/blob/53fa05b278d217476b7e92ba24d6d5dd1e1e1010/notes/2026-10-11-loader-config-profile.md)
and source describe three changed spans totaling 16 bytes: selector at
0x01C02084 returns image-owned zero padding at 0x01C07C40; the decode calls at
0x01C02666 and 0x01C02676 become NOPs. The proposed 64-byte padding is inside
the upload, rather than writing guessed values into retained A/B RAM. The
authors report native review of the resulting default mode-2 path.

This is a concrete **alternative-startup candidate**, not evidence that our
unchanged helper can skip its configuration check. Its public report still
leaves initializer **0x01E00428** and entry-supplied operations unresolved,
and explicitly states that matching flash dumps do not prove unchanged NOR
status. The backup ends with the loader active. Later return/install evidence
must be considered separately. We did not regenerate, compile or execute the
patch, acquire its private binary or adopt its FC0A schedule.

### 3. Stronger external recovery evidence, with writes

The same pinned
[SDK rehearsal](https://github.com/ip2k/lunar-modulator/blob/53fa05b278d217476b7e92ba24d6d5dd1e1e1010/notes/2026-10-11-sdk-rehearsal.md)
and later public
[stock-return verification](https://github.com/ip2k/lunar-modulator/blob/53fa05b278d217476b7e92ba24d6d5dd1e1e1010/notes/2026-10-12-completed-stop-trace-stock-reset/physical-stock-return-verification.json)
report application-sector writes/restoration, full baseline readback and a
normal `FM-1_092` identity after one FC0C submission ended ambiguously on
disconnect. Successful SG reset completion and the boot cause remain unproved.
This is useful evidence
that recovery/return is practical on that external setup. It is not a
backup-only V15/macOS return, and author-reported physical evidence is not a
local device result. FC0C is not adopted as our automatic cleanup.

Newer
[cold-ROM admission work](https://github.com/ip2k/lunar-modulator/blob/53fa05b278d217476b7e92ba24d6d5dd1e1e1010/notes/2026-10-15-cold-rom-backup-admission.md)
adds **offline** source/receipt handling for another lineage; it claims neither
gateware execution nor successful USB_KEY entry. Its scope does not
replace missing startup, status and Mac qualification. No Linux adapter,
external private allowlist, device identity or firmware-specific checksum
exception is transferred into our implementation.

### 4. New source-auditable loader and related-device leads

| Primary source | Useful new fact | Limit for our selected route |
| --- | --- | --- |
| [jvitkauskas/m-flasher, 9d962747](https://github.com/jvitkauskas/m-flasher/tree/9d962747437d92cbab594c2320b586d162c6572d/loader) | Original C loader source declares ROM send/receive/handler-slot interface; custom linker variants use 0x01C20000/0x01C28000. | It leaves unknown commands, including memory operations, to ROM and treats send as opaque. This is reverse-engineered interface evidence, not the missing ROM body or active allocation map. It also contains erase/program operations and changes clock/MMIO state; no binary/build is adopted. |
| [SMK-37 Pro dump report, 519b1f62](https://github.com/acidsound/smk37pro-acidsound-mod/blob/519b1f62c547c9e9def8eef2ababa7f4bde3b007/docs/m09-wl82-dump-2026-08-15.md) | Related WL82 device reportedly yielded two equal 1 MiB dumps under Windows. | Different product and **31,232-byte** helper, reported SHA 9920e666…; current reader sends FC0A. This is neither a mask-ROM dump nor the selected 24,064/30,720-byte helper. Historical output files were not found in the inspected tree; binary digest is attributed, not rehashed here. |
| [fm1-emu, 7196a9f4](https://github.com/joakimeriksson/fm1-emu/tree/7196a9f45bb19677b31d258a23a23dd09e0db9de), [Keitark/fm1-nes, 90d41c5e](https://github.com/Keitark/fm1-nes/tree/90d41c5e48073af2d4f7ff94df7496004d4b6627) | Source/notes provide related application and emulator work. | No matching ROM implementation obtained in the scoped checks. An indirect older Lunar-table “sparse planner” lead is not corroborated by the inspected current NES README; it cannot be promoted to ROM provenance. No emulator was run. |

The bounded ROM lane made 35 online requests (seven web queries and 28 API
GETs, including one invalid422 query). Callback searches/new trees did not
produce matching machine-code bodies at 0x01E00428/0x01E008E4/0x01E0089A or
an active ROM allocation map for selected A/B. Search results cover indexed
content and inspected paths only; no global absence is asserted. The rejected
BR28 ROM image and report47's OTA archives were not downloaded or reanalyzed.

## Existing tools: reuse what they actually solve

| Current primary source | Existing useful capability | Why it is not our ready-made strict backup |
| --- | --- | --- |
| [Transporter a632d923](https://github.com/kurogedelic/FM-1-transporter/tree/a632d923203170e3a08565f4056dbb3fd3d65ca5) | RP2040 entry, full raw read, application-sector flashing/recovery. `jl_ensure_loader` directly uploads/jumps, without A/B FD07. | Extra board; INFO/keepalive use FC0A and accept unobserved helper startup. The public owner fork has the same HEAD. |
| [Unbricker 8b4992ff](https://github.com/Quixotic7/MvaveFM1Unbricker/tree/8b4992ff684158ed1b27299792eedcfd0f9617ea) | Native Windows/Linux backup/recovery; two full reads; serial Transporter support. Fresh HEAD is newer than report09. | Native path refuses macOS; optional ROM crash FD07 is not A/B; online validation still uses FC0A. |
| [Melodee 0c21568a](https://github.com/keremimo/melodee/blob/0c21568a03626f1240229f7f368cfc2a7b09bafb/tools/fm1_recover.py) | Packaged direct-cable recovery including Mac distributions, when ROM is already reachable. | Different 30 KiB helper; crash FD07, FC09/FC0A and one full pass. Omitting `--write` only guards host erase/program. Local helper effects are audited in report 45, not universally qualified. |
| [Felucca df4498e8](https://github.com/hugelton/Felucca/blob/df4498e8d95966853ac38643c146919268e68aa1/tools/fm1_install.py), [WebApp 88492813](https://github.com/hugelton/Felucca-WebApp/blob/884928139b1cc62122973c2ef33bf87c99645542/fm1backup.js) | Direct USB-MIDI installation and musical-object archives under supporting firmware. | Installation writes flash; an object export is not the original-stock full physical flash. No RP2040 is inherently needed for normal MIDI installation. |
| [Slipway c067f969](https://github.com/Discodave-Glitch/slipway/tree/c067f969586d030483c78cf4eb7f5c646f65265d) | Windows firmware manager delegating Felucca installer; author's recovery guidance. | No new raw protocol; recovery guide uses mutable jl-uboot-tool, FC0A, Linux VM and eventual V15 writes. Its custom-firmware workflow still returns via V15. |
| [Groove OS publisher manual](https://www.groove-os.com/manual/firmware) | Publisher documents Web-MIDI installation, V15 rollback and developer Mac USB recovery. | It explicitly lacks a stock backup and still routes other firmware via V15. Recovery scripts were named but not obtained by our two scoped GitHub searches; the manual is a dated first-party report, not an audited script/trace. |

The Groove manual snapshot is 115,777 bytes, SHA-256
`[digest omitted from public archive]`.
That page's `WL80UBOOT1.00` report is not silently equated with selected
`WL82 UBOOT1.00`. Official V15 downloads are release packages, not backups of
the owner's unit. A VM or dongle changes access/entry, not the helper's effects.

NOR array programming/erase and status-register programming are distinct.
A source label “read-only,” absence of host erase/program, two equal dumps or
successful recovery does not by itself prove that startup and FC0A avoided
status writes. We retain the owner's strict constraint; no chip-specific
status change is newly classified as harmless or authorized.

## Next bounded backlog item: R4-C-P1 route decision

**Prioritize an offline feasibility audit of the pinned deterministic-helper
candidate against the existing unchanged-helper route.** This is an audit
priority, not permission to patch/load a helper or replace R1/R2 contracts.

1. Bind the published 16-byte transformation to our exact retained original;
   inspect the patch source and independently check bytes/native control flow,
   all selector callers, zero-padding lifetime and unchanged relevant code.
   Resolve source-license/reuse obligations before copying implementation.
   No firmware build, vendor-code execution or device traffic in this item.
2. Assess the **whole selected start/read/return subset**, including external
   initializer/supplied operations, stack/interface/scratch ownership and raw
   chip behavior. Determine whether our positive FD05/no-FC0A schedule can be
   used without the candidate's 515 FC0A calls. Simply deleting commands or
   moving the status check after startup is insufficient.
3. Produce one decision: retain unchanged-helper observation, prepare a
   separately reviewed exact deterministic-helper profile, or record the
   precise remaining external dependency. Include changed contracts and
   existing C3 access/continuity, finite timing and stock-return requirements.
   Any alternative must avoid the risky operation by construction or qualify
   it; it cannot inherit old offsets/digests/freshness authority.

**Definition of done:** a source/hash-bound route decision and minimal concrete
next implementation/evidence item, with no new physical capability. If the
comparison remains blocked on the same external initializer, say so and stop
repeating unchanged scans. A purpose-built source reader remains a fallback
design option, not a second implementation started in parallel. Bare-ROM A/B
effects/ownership remain necessary only if that operation is retained.
Reset-to-ROM effects and safe startup remain relevant to every selected route;
discarding an optional read cannot prove them.

Once a concrete route is qualified, activation review and the existing
FM1-033-E acceptance can proceed under the already granted bounded backup
permission: two fresh equal full reads, secure independent copies and observed
original-stock return. This task authorizes no hardware. No prior backup,
intentional brick, restoration write trial, complete ROM reconstruction or
all-platform qualification is added as a prerequisite.

## Evidence, validation and delivery boundary

Sanitized evidence (historical reference `evidence/48-online-rom-route-evidence.json`; omitted from public archive) binds selected
primary URLs, commits, fetched-body SHA-256/Git blobs, external reported digests,
retrieval/verification receipts and explicit false authority fields. Ignored
raw sources/notes/receipts remain under `[local artifact location omitted]`.
Search snippets, generated summaries, failed queries and filenames are not
implementation evidence. Matching source bytes do not authenticate private
external hardware receipts or attest to the owner's firmware.

Documentation/JSON/source-digest/Git-blob/local-link/diff checks and independent
review apply. No product code changed, so host/native/firmware tests were not
rerun. Existing unrelated rescue-source WIP remains separate. Work is local on
`init`, without push; all capture/entry/helper/writer factories remain closed.
No owner-device RAM capture, full backup or physical return occurred.
