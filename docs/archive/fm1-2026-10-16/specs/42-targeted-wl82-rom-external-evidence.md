> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Targeted WL82 ROM and external backup evidence — R4-C

Research date: **2026-10-16**. Branch: `init`. GPT 6.1 Sol high subagents
investigated SDK/ROM artifacts and external FM-1 sessions; a medium subagent
checked USB_KEY, the JL SoC forum and the SMK-37 notes. The parent inspected
Melodee and Groove OS. Sources were fetched online and inspected as inert data.
No hardware inventory, open/query/command, MIDI client, target execution, vendor
script/build, backend installation or flash write occurred.

## Result and implication for the first backup

**There is new, useful external read evidence, but no newly qualified strict
no-flash-write stock-V15/macOS backup route.** A dated Linux session records
actual loader FD07 callback values. Another developer reports two equal full
1 MiB reads on Windows. A newly published macOS-capable recovery tool supplies a
different exact helper and a concrete direct-USB implementation to investigate.
These findings narrow the search; none supplies the selected bare-ROM FD07
implementation, its failure/restoration behavior or A/B runtime ownership.

The successful external sessions use `FC0A`. The different Melodee helper has
not received our transitive startup/status-register audit. A host option called
backup, an IN transfer, or absence of explicit erase/program opcodes cannot by
itself establish the owner's no-flash-write requirement. Conversely, this report
does **not** claim these tools necessarily corrupt a device or that the new
helper has the old helper's exact effects.

No extra prior-backup, experimental restore/write, all-ROM or all-platform gate
is introduced. Existing owner permission for qualified reads remains present.
Our CLI's hardware factory and all entry/backup/write activation remain closed.

## The supplied sources: exact applicability

| Source and inspected revision | Finding | What it can establish |
| --- | --- | --- |
| [USB_KEY](https://kagaimiq.github.io/jielie/isp/usb/usb-key.html), [source `1657d25e`](https://github.com/kagaimiq/jielie/blob/1657d25e6e51df6b2c18cd55cfc576c4a6370c63/isp/usb/usb-key.md) | Physical `0x16EF` key waveform, acknowledgement, timing and USB-bus caveats; concrete hub/SOF experiments use **AC696N**. | Electrical entry mechanism on the reported family. It is separate from MIDI SysEx and the V15 RAM mailbox/reset path. No WL82 FD07 body, allocation map or qualified FM-1 return. |
| [JL SoC forum](https://esp8266.ru/forum/threads/jl-soc.5500/), bounded pages/posts below | Activator, SDK and dumping observations mainly for AC690/692/696. | Attributed community leads only; copying a key/programming command to WL82 would exceed the evidence. |
| [SMK-37 Pro gist](https://gist.github.com/probonopd/18b3ed65a69d0229eb630c47d7e316dc), core revision `b2c81a54b81445adb8275edcd2346f07dfcbcb0b` | Package transcript names AC791N, key `980F`, 14,384-byte `uboot.boot`, APP VMA `0x02000120`. Later comments disagree on physical chip identification. | Shared packaging/SDK clues; no package hash or byte-identical FM-1 bootloader/ROM proof. |
| [jl-misctools `0a5b12db`](https://github.com/kagaimiq/jl-misctools/tree/0a5b12db0ef38f3042acffbe2452730a37fd2405) | Container/key/UI/older-format utilities; 42-entry tree. [Issue 4](https://github.com/kagaimiq/jl-misctools/issues/4) supplies a WL82 package. | Package-processing references, not a selected ROM capture or runtime memory map. No script was run. |
| [jl-uboot-tool `adb3f188`](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/README.md) | Support table still marks WL82/AC791N **unknown**. A WL82 definition and 24,064-byte helper exist. CLI initialization calls `chip_key()` and `online_device()` before the requested command. | Protocol/source references; the default read prelude includes FC0A. Helper availability is not established FM-1/macOS support. |
| [Official AC79 SDK](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK), exact branch identities below | Four additional complete branch trees, four mirrors, new ELF and OTA archive indexes inspected. | More manufacturer artifacts, but no matching immutable-ROM body/map obtained in this scope. |
| [AL-255/FM-1-RE `ec832f28`](https://github.com/AL-255/FM-1-RE/tree/ec832f281ca165a7d5e0722985f3acb1465046d5) | [PR 2](https://github.com/AL-255/FM-1-RE/pull/2) reports a Windows modified-V16→V15 OTA write; another comment reports stock-V15/macOS identity only. PR 3 concerns package extraction. | Genuine update/identity observations, not a full raw backup or ROM A/B read/return receipt. |

The forum pass covered targeted index searches and pages 1, 9, 12, 13, 14, 18,
31 and 41. Concrete examples: [#224](https://esp8266.ru/forum/threads/jl-soc.5500/post-87065)
(BIOS, 2022-05-30) concerns AC69xx-era dumping; [#620](https://esp8266.ru/forum/threads/jl-soc.5500/post-88508)
(ewogs30, 2022-09-03) reports AC696N stock USB-disk entry, with alternate keys
proposed as experiments. Page 12's `br21` invocation programs firmware. None is
a WL82 no-write recipe. Linked AC690x/IDE archive names were checked, not their
contents. Direct forum downloads timed out; indexed browser text supported the
bounded post review. This is not a search of every post or linked archive.

The SMK comments led to an actual [AC7911BA datasheet](https://jlcpcb.com/api/file/downloadByFileSystemAccessId/8604442319861137408):
V1.1, 2021-09-03, 16 pages, 1,710,498 bytes, SHA-256
`[digest omitted from public archive]`.
Its printed p.10 identifies pins 47/48 as FUSBDM/FUSBDP. It contains no needed
ROM implementation/map. It does not identify the owner's silicon or require
opening the FM-1, changing wiring or purchasing a dongle.

## New external observations: distinguish ROM from loader

### Lunar: actual callbacks and narrow loader reads

The live branch `chore/2026-10-10@fm1-live-bringup` resolves to
[`65e9f63942ee8e94d5e7bb4c2cccaf510daea2d7`](https://github.com/ip2k/lunar-modulator/commit/65e9f63942ee8e94d5e7bb4c2cccaf510daea2d7).
Its [dated callback receipt](https://github.com/ip2k/lunar-modulator/blob/65e9f63942ee8e94d5e7bb4c2cccaf510daea2d7/notes/data/2026-10-10-loader-callback-observation.json)
contains 52 SCSI commands, independently recounted against the saved source:

| Phase | Actual sequence |
| --- | --- |
| ROM | INQUIRY; 47 FB06 helper blocks; FB08 at `0x01C02000`, argument 1 |
| Loader | FC14; **FC0A**; FD07 at `0x01C09724`, length **8** |

The 2026-10-10 09:19:18–09:19:19 UTC Linux session reports FM-1_092. It returns
send pointer **`0x01E008E4`** and receive pointer **`0x01E0089A`**. All four tool
hashes and the operational wrapper match the receipt. These are internal
source/receipt checks, not an independent reproduction of the hardware event.
There are **zero ROM-phase FD07 requests** in this receipt. Its callback-session
`power_cycle_confirmation` is pending.

The [earlier 64-byte readback](https://github.com/ip2k/lunar-modulator/blob/65e9f63942ee8e94d5e7bb4c2cccaf510daea2d7/notes/data/2026-10-10-loader-readback-success.json)
at `0x01C043DE` matches known uploaded code bytes, SHA-256
`[digest omitted from public archive]`.
It also followed FC14/FC0A. The owner subsequently reported working operation
after a power cycle; no new identity was queried in that return observation.
[Session/return qualification](https://github.com/ip2k/lunar-modulator/blob/65e9f63942ee8e94d5e7bb4c2cccaf510daea2d7/notes/2026-10-10-fm1-live-bringup.md#L397-L434).

These are positive loader-read observations. The callback values now give two
specific addresses for offline ROM-artifact research. They supply neither the
ROM FD07 transform/send/restore body nor A/B allocation/error-path evidence.
The source's historical preparation docstring must not override its later
dated execution receipt; equally, a loader receipt must not be relabeled ROM.

### Flipper/Windows: two full reads; return observed after writes

[`merthsoft/flipper-fm1-recovery` at `df88b862`](https://github.com/merthsoft/flipper-fm1-recovery/tree/df88b862a4794855f792fed529629a255cba0a0e)
adds a [2026-10-09 developer report](https://github.com/merthsoft/flipper-fm1-recovery/blob/df88b862a4794855f792fed529629a255cba0a0e/docs/DEVELOPMENT-NOTES.md#L21-L27)
of Flipper USB_KEY entry, cable transfer to Windows and **two equal
1,048,576-byte reads**, SHA-256
`[digest omitted from public archive]`.
Raw backups/logs remain private, so this is an attributed first-party report.

The [backup source](https://github.com/merthsoft/flipper-fm1-recovery/blob/df88b862a4794855f792fed529629a255cba0a0e/tools/pc_backup.py#L96-L139)
uses the old 24,064-byte helper, FB08 argument 1, FC09 and FC0A, then FC0A every
16 KiB and in a subsequent keepalive loop. It performs no FD07. Chip/flash
checks occur **after** helper execution, despite the README's before-upload
wording. Its helper digest is logged rather than enforced.

Subsequent successful display/audio/MIDI return is to **Felucca FM-1_900 after
17 sectors were rewritten**. This does not establish original-stock return
after backup alone, nor our no-status path or macOS access. It does strengthen
the external evidence that a full physical flash read is feasible.

## New direct-USB macOS lead: Melodee's different helper

[Melodee v1.3.1 release notes](https://github.com/keremimo/melodee/releases/tag/v1.3.1)
advertise standalone recovery executables for Windows, Linux and macOS x64/arm64;
the later note adds original V15 empty-log metadata support. The actual source
inspected here is separately pinned to **`0c21568a03626f1240229f7f368cfc2a7b09bafb`**,
tree **`19c38e20650b419d1b14e3f7b820c7712077bfab`**. The release tag points to
`757b65b0037aea8c62eed7a160351d3e3678d913`; do not assume a release executable
is byte-bound to the newer inspected source. No executable was downloaded/run.

The [source](https://github.com/keremimo/melodee/blob/0c21568a03626f1240229f7f368cfc2a7b09bafb/tools/fm1_recover.py#L232-L291)
and [recovery documentation](https://github.com/keremimo/melodee/blob/0c21568a03626f1240229f7f368cfc2a7b09bafb/docs/RECOVERY.md)
provide a concrete PyUSB/libusb implementation. It requires **already-entered
ROM `4C4A:8057`, WL82/UBOOT1.00**. It does not implement running-stock V15 MIDI
entry. Interface opening may detach an active kernel driver. In default backup
mode the actual order is:

1. INQUIRY, then **bare-ROM FD07 at `0x01C7C000`, 512 bytes**, host decode and
   local retained-RAM file.
2. **60 FB06 blocks** of its different helper; FB08 argument 1; FC09; **FC0A**;
   flash/key checks after helper execution.
3. **One** complete 1 MiB FD05 pass, exclusive file creation, flush/fsync and
   reopen/byte comparison. This verifies one captured stream on disk; it is
   not a second independent device read or two-copy acceptance.
4. Without `--write`, return from the host function and ask the owner to power
   cycle. Explicit erase/program calls are behind `--write`.

That host write flag is useful but does not audit autonomous helper effects or
the preceding ROM FD07. Packaging/build smoke tests are not a physical test of
every OS, stock entry, error restoration or original-stock return. We did not
run its `--info`, backup, restore, build or test commands.

The exact helper is **30,720 bytes**, SHA-256
**`[digest omitted from public archive]`**,
Git blob **`cbe653f78d3cba8c5f4a84b90a5ac093e04025d8`**. Its size/hash are enforced
by the host source. The [asset provenance note](https://github.com/keremimo/melodee/blob/0c21568a03626f1240229f7f368cfc2a7b09bafb/assets/recovery/README.md)
attributes it to official AC79 SDK tag `AC79NN_SDK_V1.2.1_2023-12-13`.
An independent inert retrieval of the [official file at tag commit
`d179b4484759423312073f5fbb232501aa491047`](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/d179b4484759423312073f5fbb232501aa491047/cpu/wl82/tools/wl82loader.bin)
confirms equal bytes, SHA-256 and Git-blob identity. This closes provenance for
this candidate, not startup/read behavior or suitability for our route.

This is **not** our audited 24,064-byte helper (SHA-256 `d41da612…`). Pure upload
arithmetic puts the new helper in `[0x01C02000,0x01C09800)`. That overlaps the
old profile's A/B intervals and its fixed metadata address `0x01C09700`.
The old profile and field offsets cannot be reused for this helper. This is
an address-extent check, not disassembly of the new helper or evidence about
its final runtime allocations. No silent helper substitution is selected.

## Groove OS: a related recovery claim with different evidence

The [first-party firmware manual](https://www.groove-os.com/manual/firmware)
names `fm1_enter_uboot.py` / `fm1_uboot_recover.py` and describes macOS recovery
with PyUSB. It reports a successful bootloop recovery, but the presented recipe
performs a firmware restore, mentions stopping MIDIServer and reports USB
`WL80UBOOT1.00`. Its custom-firmware MIDI window is not exact stock-V15 evidence.
No public matching script/trace/raw-backup/ROM-map was obtained from that page.
We did not stop any service or execute the recipe. The manual is retained as
dated, mutable HTML rather than a pinned code artifact. It is a recovery lead,
not a strict no-write backup qualification.

## Additional official SDK / ROM search

The official branch metadata and commit objects give these **distinct commit
and actual tree** identities. All four newly fetched recursive trees are
non-truncated; API top-level response labels were not substituted for the
commit object's tree ID.

| Branch | Commit | Actual tree |
| --- | --- | --- |
| `release/AC79NN_SDK_V1.0.3` | `9e7166fa9d8e3d0a8c671e848647c33a4e57f07f` | `e85b17a05e6e03806b97b92c6736d38cf8973036` |
| `release/AC79NN_SDK_V1.1.0` | `586a4fd8fb541777e4052274ba32c99f20ecedc8` | `68be68f7ca3d7c1e2d19d1bab35ccfcede4422f6` |
| `release/AC79NN_SDK_V1.2.0` | `5fa84a9b2f723a5e9337ad79dc2ad741fc98fe2d` | `8a048c57fa919f6b34cd46c20691303a4266b414` |
| `AC791N_OTA_loader` | `5c33e71d9e4762595aea28587e76d4068f353870` | `95bc5304c7660de9aa4956891ace17222ed1b5b0` |

The current 1.2.0-named branch's [commit message](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/commit/5fa84a9b2f723a5e9337ad79dc2ad741fc98fe2d)
is actually **V1.2.14, 2026-10-13**. Four complete SDK mirror trees and three
Bootloader forks were also checked. The forks retain the rejected BR28 blob
under the WL82 filename and empty WL82 mask stubs; report 41's rejection stands.
Path searches did not find a matching ROM image/API/map in the new SDK trees.

Newly acquired [`cpu/wl82/tools/loader_tools/sdk.elf`](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/5fa84a9b2f723a5e9337ad79dc2ad741fc98fe2d/cpu/wl82/tools/loader_tools/sdk.elf)
is 1,295,788 bytes, SHA-256
`[digest omitted from public archive]`.
REA rejected its unsupported architecture without opening a native session.
Independent ELF32 structure/symbol parsing finds entry `0x02000120`, application
`.text`, 23,920 symbols and **zero symbols in `[0x01E00000,0x01E10000)`**.
It is an application ELF, not the missing ROM body or a symbol-bound version
of Melodee's helper. This check is structural, not instruction analysis.

OTA-branch reset/memory headers and two library export indexes (352/318 symbols)
provide application/OTA APIs, not an absolute mask-ROM service map. Member bodies
were not analyzed. Neither symbol/path absence nor a generated SDKWiki page
proves the whole library/history lacks relevant code.

The [reconstructed FM-1 bootloader](https://github.com/jvitkauskas/fm1-bootloader/blob/14b26f4faaa8876bcd266ceeba0b73d522a0b050/README.md)
is a useful SPL lead, not an immutable-ROM consumer. The [fm1-emu recon note](https://github.com/joakimeriksson/fm1-emu/blob/7196a9f45bb19677b31d258a23a23dd09e0db9de/docs/recon.md)
describes public-source synthesis with a physical device still ordered at that
checkpoint; it is not an original ROM capture. Named sibling SDK/misctools
checkouts were absent locally; no missing checkout was mistaken for an online
absence.

## Backlog disposition: finite next evidence, no automatic activation

| Gate | Effect of this research | Remaining evidence |
| --- | --- | --- |
| C1 exact V15 local handler/reset producer | Already complete offline in report 40; external identity/OTA reports remain separate. | No repeat package/model work selected. |
| C2 selected reset/ROM FD07/A/B/return | New callback addresses narrow offline artifact search. New SDK trees and foreign images do not close it. | Matching reset consumer and autonomous effects; selected FD07 success/failure restoration and active A/B ownership; stage-specific return. |
| C3 selected macOS entry/access/continuity | New direct libusb source and shipping macOS packages are useful implementation leads. | Exact stock-V15 receiver framing and selected backend lifecycle/outcome remain unqualified. Packaging is not the receipt. |
| C4 finite ROM-only context capture | No matching external A/B session found. | After C2/C3, capture only the preselected **512 + 512 + 34 = 1,058 bytes**, before helper upload/jump. No arbitrary ROM dump. |
| C5 selected no-status helper/full read | Positive Linux narrow reads and Windows full-read reports, with FC0A; a separate exact helper candidate exists. | Old exact helper's startup/callback/chip and no-FC0A subset, or a separately audited alternative with its own layout/contract; then live timing, two fresh passes, durable independent copies, continuity and observed stock return. |

**Next scoped investigation:** search matching ROM artifacts/maps using the newly
observed callback addresses alongside the existing `0x01E00428` startup target.
If the 30,720-byte helper is considered instead, use its now-bound manufacturer
bytes and audit its own startup/read/configuration requirements offline. Establish
whether it can avoid ROM FD07 and FC0A before proposing a route change; do not
port old offsets or assume that merely deleting FC0A is sufficient. That is a
candidate investigation, not another mandatory prerequisite for the current
route. No new fake-only model, blind read, patched loader or write experiment
is selected by this research report.

## Evidence retention and checks

Sanitized evidence manifest (historical reference `evidence/42-targeted-wl82-external.json`; omitted from public archive) records
source identities, hashes, exact observed phase/command distinctions, helper
extent calculations and false activation flags. Raw packages, binaries, page
downloads and detailed receipts remain ignored under
`[local artifact location omitted]`; no device dump or
personal device identifier is committed. The three research notes are retained
there alongside retrieval results. New vendor source was never imported.

Search coverage is finite: selected branches/forks, index queries, issue/PR
bodies/comments, forum pages and named artifacts. GitHub unauthenticated 403s
were rate limits; subsequent authenticated read-only API results are recorded
separately. Wrong-owner/inaccessible 404s and forum timeouts are not proof of
global absence. No remote repository, issue, message or account was modified.

Validation is file-only: JSON/digest/Git-blob/source-receipt checks, command-phase
recount, helper extent arithmetic, local documentation links and diff scope.
The parent rehashed **97 fetched bodies** and **90 indexed SDK files**, confirmed
official/helper byte equality and checked the saved Python AST without imports
or execution. An independent medium subagent found no blocking documentation
issue; its provenance wording clarification is incorporated.
No product code changed, so host/native tests from report 41 were not rerun or
claimed as new physical evidence. No first hardware backup has been created.
