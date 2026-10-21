> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Stock FM-1 backup through one USB-C cable

Research date: **2026-10-15**. Scope: full raw flash backup, stock firmware
unchanged, computer connected directly to FM-1, no RP2040 and no flash writes.
This investigation used local documents and public text sources only. It did
not enumerate, open or command hardware, run upstream tools, build firmware,
download a loader binary or install a hardware backend. The owner's current
firmware, board, USB state and host transport remain unmeasured here.

## Finding

**A cable-only backup is a credible candidate through ROM plus a volatile RAM
loader; a full raw read while the normal stock application keeps running is
not established by the inspected sources.** These are different requirements.
The candidate can retain the installed stock flash image while temporarily
stopping the application. It is not passive inspection and is not qualified
for this project or the owner's device.

| Route | Evidence | Decision |
|---|---|---|
| Running stock → full raw USB-MIDI backup | No demonstrated arbitrary raw-flash read service found in the inspected stock protocol research | Unsupported; do not guess commands |
| Running **stock V15** → ROM with USB-MIDI soft key | Transporter implementation/author's V15 report; new-unit Reddit advice recommends the direct route | Exact-V15 entry candidate; direct-computer framing/behavior needs qualification |
| ROM already visible → RAM loader → raw flash over direct USB | jl-uboot-tool/Unbricker source; Windows author's two identical full dumps | Feasible source-backed route on Windows; Linux implementation candidate; no local qualification |
| Same ROM route on macOS | Unbricker explicitly has no native SCSI backend | No usable backend supplied by those tools; retain report 06's macOS access/checkpoint gates |
| No application and no ROM USB visibility | External electrical USB_KEY entry documented | Cable-only recovery is not established; RP2040 route is outside this request |
| Custom firmware's export/CDC recovery or replacement bootloader | Requires already installed custom code | Does not meet unchanged-stock requirement |

The direct-USB hardware report concerns an already failing **Felucca** unit in
ROM, not entry from untouched stock. It establishes a component of the route,
not the complete stock-V15-to-backup-to-stock sequence. Its author reports two
identical 1 MiB reads, chip key `0x980F`, flash ID `0x856014`, type 3. Do not
infer all units, boards, operating systems or failure modes from that result.
[Primary report in Felucca #61](https://github.com/hugelton/Felucca/issues/61#issuecomment-6018813157).

## Entry is a separate, mode-changing operation

Transporter sends the six-byte SysEx `F0 22 24 35 7D F7` as two USB-MIDI event
packets, `04 F0 22 24 07 35 7D F7`. Its source comments say V15 compares the
eight-byte USB transfer and calls ROM without returning; the implementation
only offers the operation for its detected stock application. The author
reports V15 leaving USB 21 ms later and ROM attaching roughly one second later.
That experiment used the RP2040 as USB host. An ordinary computer MIDI API
must be shown to preserve the framing required by this stock receiver; the
same logical SysEx alone is not proof of the direct-computer path.
[Entry source](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/src/pio_host.c#L99-L117),
[author's experiment](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/docs/DEVLOG.md#L128-L150).

AL-255's stock V13 disassembly independently identifies a `7D/7F` trailer
with a target annotated `0xFFC02532`, at medium confidence. **Correction,
2026-10-16:** this is a file-relative annotation, not an absolute mask-ROM
address; with application VMA `0x02000120` it resolves to RAM `0x01C02652`.
The exact manufacturer V15 bytes independently resolve their corresponding
callee to RAM `0x01C026A4`, which requests reset after writing a RAM mailbox;
see [report 40](40-v15-entry-rom-backup-audit.md). This does not qualify the
reset's ROM aftermath or direct-computer framing. The ordinary update
trigger ends in `7F`, and its stage-one path writes update state: it is not an
acceptable substitute for read-only entry.
[Stock MIDI analysis](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/05-midi.md#L88-L99),
[ordinary OTA state changes](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/11-ota-protocol.md#L29-L48).

These byte sequences document evidence, not instructions to send them. There
is no recommendation to install V15 merely to obtain this entry path, edit an
identity, send adjacent commands or force an unknown device profile.

## ROM presence does not itself provide a full raw backup

The documented `UBOOT1.00` implementation exposes memory read/write/jump;
the loader supplies raw flash access. ROM `FD07` memory reads must not be
represented as raw NOR bytes or a recovery snapshot. The upstream host uploads
`wl82loader.bin` to RAM at `0x01C02000` with `FB06`, then executes it with
`FB08`, argument 1 for SPI NOR. Only then does `FD05` read raw flash, with a
32-bit big-endian address and 16-bit big-endian length in the padded 16-byte
CDB. The upload is a **RAM write and code execution**, even though the desired
operation performs no flash erase/program commands.
[Protocol distinction](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/usb-loader-v2.md#L10-L30),
[loader metadata](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/data/usb-loaders.yaml#L201-L208),
[Unbricker upload](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L816-L850),
[command implementation](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/uboot.py#L238-L304).

Unbricker pins loader SHA-256
`[digest omitted from public archive]`.
Its backup flow reads `[0,0x100000)` twice, rejects mismatches, creates the
output exclusively, flushes/fsyncs it and compares the reopened file. It does
not need an official firmware package merely to capture a backup; package
comparison is additional analysis. Its generic device/identity overrides and
restore commands should not be inherited by our read-only implementation.
[Pinned inputs](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L87-L100),
[read loop](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L514-L533),
[backup and persistence](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L884-L889),
[double-read capture](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L1073-L1120).

The inspected host sources do not issue erase/program from this backup flow.
They do **not** prove the opaque RAM loader's initialization, flash status/key
handling, watchdog and return path never modify persistent state. No loader
binary audit was performed here and no redistribution rights were established.
A pinned digest proves which bytes are intended, not their behavior or rights.
The full “no flash writes” claim remains gated by that audit and external
qualification. Report [06](06-usb-rom-feasibility.md) also retains native
transfer/status/sense/residue and macOS capture/release issues.

Transporter reports a roughly three-second loader inactivity reset and uses
one-second keepalives. It also reports that a warm return through loader
`RUN_APP` or watchdog runs stock V15 without USB attachment until a cold power
cycle. Treat disconnect, loader expiry, normal return and re-identification as
explicit lifecycle evidence; do not automatically reset, retry entry or infer
a brick from the absence of a returning MIDI port.
[Observed lifecycle](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/docs/DEVLOG.md#L107-L150).

## What the other leads establish

The normal stock interface is audio plus MIDI, not CDC; the optional SDK online
configuration protocol with flash commands is not demonstrated in stock.
Unresolved read-shaped callbacks in the stock dispatcher are research leads,
not a qualified raw-read API. A newer small utility reports reading the
**currently active voice** on V14/V15; this is useful evidence that semantic
readback exists, but not a 1 MiB flash export.
[Stock surface/callback audit](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/11-ota-protocol.md#L109-L151),
[voice utility](https://github.com/czietz/fm1-read-voice/blob/b7c539f2ca4a492d5ee8afdfb39ad9b3ca89a30e/README.txt#L1-L4).

ChoralRoot's handoff documents both a no-USB failure and subsequent Transporter
recovery. It does not supply a normal-stock cable-only backup path. The
replacement `fm1-bootloader` adds recovery keys but explicitly changes the
bootloader and describes its build as hardware-untested; installing it cannot
solve this unchanged-stock requirement. Custom CDC entry and Felucca-family
semantic backups likewise require custom firmware already running.
[ChoralRoot no-USB case](https://github.com/Quixotic7/ChoralRootFM1/blob/776c740b14f191dd4d991167f647557a6e55c3ab/docs/TRANSPORTER-HANDOFF.md#L11-L45),
[reported recovery](https://github.com/Quixotic7/ChoralRootFM1/blob/776c740b14f191dd4d991167f647557a6e55c3ab/docs/TRANSPORTER-HANDOFF.md#L145-L169),
[replacement bootloader](https://github.com/jvitkauskas/fm1-bootloader/blob/14b26f4faaa8876bcd266ceeba0b73d522a0b050/README.md#L1-L19).

## The five owner-supplied Reddit threads

All five were accessible through public web text on 2026-10-15. The rendered
pages show relative dates, not independently verified absolute timestamps;
different crawler snapshots produce inconsistent ages. The dates below are
retrieval labels, **not event dates**. Direct public `.json?raw_json=1&limit=100`
requests for all five returned HTTP 403, so no exact post/comment timestamp or
complete nested-comment inventory is claimed. Deleted comments and “More
replies” placeholders remain unknown.

| Thread / displayed age at retrieval | Relevant evidence and limit |
|---|---|
| [Best way to take a full device backup of a new unit?](https://www.reddit.com/r/MVaveFM1/comments/1x4fnhm/best_way_to_take_a_full_device_backup_of_a_new/) / post 3d, relevant comment 2d | **Directly relevant working-stock lead:** c-zietz recommends the `7D` SysEx, then jl-uboot-tool full `[0,0x100000)` read. OP thanks them but supplies no measured success, firmware/OS/loader hashes or original/returned dump comparison. This supports the direct-cable candidate; it is not a qualified experiment. A reply about unit-specific Bluetooth data explicitly lacks a source, and OP's proposed two-unit comparison has no visible result. Do not assume cross-unit dumps are interchangeable. |
| [Guide: plain USB unbricking](https://www.reddit.com/r/MVaveFM1/comments/1wz48t7/guide_unbricking_a_softbricked_fm1_black_screen/) / page 12h despite older crawl label | Acrawf1 starts with a failed custom firmware **already in ROM**, reports two full reads and subsequent restoration. Technical claims trace to their GitHub #61 report and the pinned loader/host source above. The guide does not itself establish entry from working stock. Its write/retry/erase advice is outside this task. ignis32 reports RP2040-assisted 1 MiB backups through firmware-response/ROM methods and a missing `fm-1-research-lab` dependency; their comment does not describe a computer-only implementation. |
| [Soft-bricked FM-1: successfully dumped](https://www.reddit.com/r/MVaveFM1/comments/1wz2gnj/softbricked_fm1_wl82_uboot100_successfully_dumped/) / 3d | International-Soup24 reports Windows UBOOT detection and a saved full SPI dump before asking about writing. Their replies later say the linked Acrawf guide worked. This is another **already-ROM** report, not stock entry, a documented double-read receipt or a reason to experiment with the proposed offsets. |
| [Factory reset does not restore factory patches](https://www.reddit.com/r/MVaveFM1/comments/1wizbbm/factory_reset_doesnt_actually_reset_to_factory/) / 9d | Preset-reset behavior and alternate preset-updater advice explain why “factory reset” is not the requested full backup. The linked author-owned factory-preset project captures/decode preset restore traffic; it is not a raw-flash capture route. Updater version/platform claims in the comments were not independently qualified here. |
| [Cannot return to original firmware after Baud Girl](https://www.reddit.com/r/MVaveFM1/comments/1wyadir/cannot_switch_back_to_original_fw_after_baud_girl/) / 10d | Reports `calc=13, got=19`, alternate web-installer advice, and eventual SLOOP installation. A commenter associates the error with Baud Girl 0.93/0.94. This is updater/identity compatibility evidence, not raw backup evidence or qualification of a firmware hop. |

The new-unit advice's entry bytes map to Transporter's stock soft-key source;
its `read` operation maps to LoaderV2 `FD05`, which needs the RAM-upload path
described above. [Direct advice comment](https://www.reddit.com/r/MVaveFM1/comments/1x4fnhm/comment/pgfeao4/).
The factory-preset lead maps to a semantic restore capture,
not full NOR access. Its older prose that the FM-1 never sends SysEx should
not supersede the newer voice utility or its own recorded identity replies.
[Factory-preset capture source](https://github.com/KingParamount/fm1-factory-presets/blob/04b34cabd69745cf3a549af588619108e2866c0d/docs/protocol-and-provenance.md#L1-L34).
The Baud Girl checksum lead is consistent with Lunar Modulator's documented
on-device `FM-1_092` identity retaining V15's checksum, but that observation
does not prove the exact 0.93/0.94 cause/fix asserted by a commenter.
[Identity evidence](https://github.com/ip2k/lunar-modulator/blob/bfa042446f36c8647efddc6d76b06753d2a5b612/docs/03-update-protocol.md#L35-L47).

The Transporter missing-dependency report is consistent with the existing
[source review](01-github-research.md): do not assume the deleted/unavailable
research-lab can be fetched, and do not execute upstream tools to discover it.
Unbricker's inspected standalone capture path supplies source references
without relying on that private module. The additional
[RP2040 recovery post](https://www.reddit.com/r/MVaveFM1/comments/1x5lifu/mvave_fm1_unbricking_after_a_hard_brick_using_a/)
is an external-hardware lead only. Community timing, “hard/soft brick” labels
and broad recovery claims are not acceptance evidence for our unit.

## Recommended offline implementation slice

Implement a **backup feasibility/qualification model with injected fakes**
before a real transport. Reuse existing durable backup and BOT models; retain
the project's default denial of native access.

1. Model `stock_running → rom_visible → audited_ram_loader → full_reads →
   durable_backup → normal_return`, with separate evidence for each transition.
   Admit only an exact source profile; unknown version/board/loader/OS refuses.
2. Represent stock soft-key entry and RAM upload/jump as mode-changing
   capabilities, separately from flash read. Never route through OTA, replace
   firmware, use an RP2040 implicitly or claim normal-stock read support.
3. Bind the explicitly selected old/new OS objects to one physical USB
   location and session across re-enumeration; shared VID/PID, model name or
   chip key alone is insufficient. No disk sweep, first-match or override.
4. Keep flash erase/program/key writes, generic memory access, reset and retry
   absent. Future RAM upload must permit only the reviewed digest, address,
   length and execution argument. An already-running unknown loader refuses.
5. Test refusal before I/O, exact framing, ambiguous/wrong reattachment,
   loader expiry, short/status/sense/residue errors, mismatched full reads,
   persistence/copy failure and controlled lifecycle failure under the canonical
   no-hardware guard. A simulated success must still report hardware evidence
   false.

## Gates before any real backup claim

- **Exact stock entry:** user-supplied version/package or other trustworthy
  identity evidence; exact MIDI receiver/framing audit; no adjacent update
  command or hidden firmware hop. V15's reported entry must be qualified over
  a direct computer connection on the chosen OS.
- **Loader:** lawfully supplied exact bytes, independent startup/read/return
  audit including persistent side effects and watchdog behavior; attribution
  and redistribution decision. Source MIT notices do not license every blob.
- **Host access:** separately reviewed selected-device adapter with exact
  transfer/status/sense checks; supported platform permissions and preserved
  ROM/loader state through claim/release. macOS is an open access gate, not a
  proven impossibility or a reason to bypass OS protection.
- **Read-path qualification:** establish exact stock version/board/chip/layout,
  direct entry, RAM execution, held raw reads and documented return through
  reviewed code and applicable existing external evidence, including persistent
  side effects. The owner's permission covers a bounded audited read session
  once these requirements are satisfied; a new spare is not mandatory for every
  successful read. Fault injection and development/test writes require a spare
  or external bench and never use the owner's only FM-1.
- **Backup acceptance:** two equal fresh 1 MiB reads; immutable identity and
  loader/session metadata; fsync, reopened hash verification, second durable
  copy; the original retained privately. Equal reads alone do not prove
  restoration, publisher authenticity or zero risk.

An authorized owner-device read can only be scoped against these concrete
gates. This research supplies no physical qualification and no zero-risk
promise. Erase/program and restore remain unavailable.

## Search coverage and immutable revisions

Refreshed public default-branch heads through GitHub on 2026-10-15:

| Repository | Inspected revision |
|---|---|
| Quixotic7/MvaveFM1Unbricker | `8b4992ff684158ed1b27299792eedcfd0f9617ea` |
| Quixotic7/ChoralRootFM1 | `776c740b14f191dd4d991167f647557a6e55c3ab` |
| kurogedelic/FM-1-transporter | `a632d923203170e3a08565f4056dbb3fd3d65ca5` |
| kagaimiq/jl-uboot-tool | `adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a` |
| AL-255/FM-1-RE | `ec832f281ca165a7d5e0722985f3acb1465046d5` |
| ip2k/lunar-modulator | `bfa042446f36c8647efddc6d76b06753d2a5b612` |
| aroum/fm1-custom-fw | `c48c414041fb3e0d9cd3fdb2ee3a012fba17617e` |
| czietz/fm1-read-voice | `b7c539f2ca4a492d5ee8afdfb39ad9b3ca89a30e` |
| jvitkauskas/fm1-bootloader | `14b26f4faaa8876bcd266ceeba0b73d522a0b050` |
| KingParamount/fm1-factory-presets | `04b34cabd69745cf3a549af588619108e2866c0d` |

Read the requested existing reports 01/03/06/09; inspected relevant READMEs,
entry/read/upload implementations, stock USB/MIDI/OTA research, loader metadata
and issue #61 comments. Queried GitHub issue listings for Transporter,
Unbricker, ChoralRoot and Felucca; the Felucca general listing was the latest
100 items, with #61 fetched explicitly. This is targeted coverage, not every
fork/issue/stock artifact or a binary audit.

Web searches included `FM-1 backup stock USB`, `FM-1 UBOOT stock`, exact
`F0 22 24 35 7D F7`, `FM-1 READ_FLASH stock`, `AC791 read SCSI`, and Reddit
plain-USB/unbricking leads. Lunar Modulator traces its soft-key note to the
Transporter and AL-255 sources cited above. The Firmware Hub's
[rescue page](https://thefirmwarehub.com/) advertises ROM entry and links
Unbricker, but source retrieval returned HTTP 403; its page wording was not
used as proof of protocol behavior. No newly found route closes the macOS,
loader audit or exact-owner-device evidence gaps.
