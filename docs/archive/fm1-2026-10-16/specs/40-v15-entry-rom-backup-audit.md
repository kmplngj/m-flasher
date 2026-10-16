> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Exact V15 entry and first-backup route — FM1-033-D2-R4-C

Date: **2026-10-16**. Branch: `init`. The owner authorized online firmware
research after the initial offline inventory, and already permits backup reads
while prohibiting flash writes. This increment used local files, public
downloads, static disassembly and injected tests only. No device enumeration,
open, MIDI/USB command, RAM upload/execution, firmware build, backend installation
or flash write occurred. Hardware activation remains closed.

**Result:** manufacturer-hosted V15 bytes are available and the exact matching
MIDI branch is now resolved through its RAM mailbox/P33-reset routine. The
previous apparent absolute ROM call was a load-address annotation error. Both
package parsers needed a narrow compatibility fix for V15's empty `info.log`.
The subsequent immutable-ROM FD07 behavior/interval ownership and selected
macOS framing/access/return remain unresolved; no full device backup was made.

## Manufacturer artifact and byte bindings

The manufacturer's [download page](https://www.m-vave.com/download) lists
**PC Firmware → FM-1 → V15, 2026_07_30**. Its linked
[FM-1.fwsc package](https://yms-file-store.oss-cn-hongkong.aliyuncs.com/software/firmware/FM-1.fwsc)
was downloaded as inert data. The similarly named V14 Windows/Mac items are
different downloads; an installer/DMG must not be treated as these package bytes.

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| Original `FM-1.fwsc` | 699,956 | `[digest omitted from public archive]` |
| Type-0 raw `flash.bin` | 602,112 | `[digest omitted from public archive]` |
| Raw encrypted APP `[0x4000,0x93000)` | 585,728 | `[digest omitted from public archive]` |
| Decoded `app.bin` | 581,564 | `[digest omitted from public archive]` |
| Initialized RAM copy | 41,052 | `[digest omitted from public archive]` |
| Decoded `uboot.boot` BANKCB | 14,384 | `[digest omitted from public archive]` |

FWSC identity is `FM-1_015`, header chip `AC791N`. Header CRC `A45D`, encrypted
entry-table CRC `13B0` and type-0 data CRC `9B24` were independently checked.
The eight entries are `flash.bin`, empty `info.log`, `USR`, `isd_config.ini`,
`ota.bin`, `script.ver`, `blimit.bin` and `tail.bin`. The empty log's logical
offset `0x93400` is also the next `USR` payload's beginning.

An original file-only decoder derives SFC key `0x980F` from CRC-checked top-level
configuration, decrypts the application in 32-byte blocks with the offset key,
and validates JLFS headers and application/container data CRCs. These are package
and mapping checks, not a digital manufacturer signature or installed-image
attestation. The downloaded package is shorter than the modeled 1 MiB flash and
contains no capture of the owner's current device-specific data. It cannot
replace a full raw backup or prove the unit's actual capacity/calibration/presets.

Files remain ignored under `[local artifact location omitted]`; proprietary
firmware bytes are not added to Git. Download, extraction, carrier, disassembly
and independent range receipts are retained there. A sanitized durable summary
is evidence/40-v15-entry.json (historical reference `evidence/40-v15-entry.json`; omitted from public archive).

## Exact instruction and address checks

An existing vendor pi32v2 reader was used only as a static disassembler, with
analyst-created ET_REL carriers containing unchanged slices. Application VMA
**`0x02000120`** comes from the CRC-checked application metadata. Startup at
`0x0200014C–0x02000166` independently copies XIP `0x02084060` (file offset
`0x83F40`) to RAM `0x01C00000`, length `0xA05C`. Thus data and instructions in
that initialized section have a verified second mapping.

| Selected code | Exact evidence and effect |
|---|---|
| Receive wrapper `0x02006E1E` | Requests at most 64 bytes into its stack buffer, then passes buffer and returned length to `0x02006A60`. The upstream USB read implementation and computer packetization are distinct boundaries. |
| Handler `0x02006A60` | Requires length 8, compares first six bytes against RAM `0x01C0800D` using leaf comparison at `0x02044524`, requires byte 7 = `F7`, branches on byte 6. |
| Constant | RAM copy maps it to app file offset `0x8BF4D`: `04 F0 22 24 07 35 7D F7`. The handler compares six prefix bytes and separately checks the trailer. Logical SysEx is `F0 22 24 35 7D F7`. |
| `7D` branch | At `0x02006A96`, bytes `80 ff 08 bc bf ff` encode a 6-byte relative call: `0x02006A96 + 6 - 4211704 = 0x01C026A4`. It then loops if the callee returns. |
| RAM `0x01C026A4` | Disables interrupts, changes RAM lock/counter state, copies 16 bytes `usb_update_mode\0` from XIP `0x020550AA` to mailbox `0x01C7FD80`, then calls RAM `0x01C0108C` with `(0xA0,4,1,1)` and loops. |
| P33 wrapper `0x01C0108C` | Calls read `0x01C0100A`, modifies bit 4, calls write `0x01C0104E`. Helpers `0x01C00FA0/FC8/FE2` handle RAM locking and MMIO `0x13E08/0x13E0C`. The byte transaction has a hardware-busy loop. |
| `7F` branch | Separate task-message/OTA path; excluded from read-only entry. No neighboring command is an entry fallback. |

This closes the selected local `7D` subgraph to a mailbox/reset request. It has
no direct mask-ROM call, indirect call or NOR erase/program/status routine in
the resolved subgraph. The P33 reset interpretation is corroborated by the
[pinned author's analysis](https://github.com/ip2k/lunar-modulator/blob/bfa042446f36c8647efddc6d76b06753d2a5b612/notes/2026-10-05-softkey-efuse.md#L42-L80).
The mailbox `[0x01C7FD80,0x01C7FD90)` overlaps eight bytes of the record
beginning at `0x01C7FD88`; it does change update-related RAM. Reset completion,
the mailbox consumer, ROM boot behavior and persistence effects after that boundary
are not established by examining these instructions. A busy loop is not a
bounded host operation. This is a mode change with RAM/MMIO writes, not passive
inspection or a guarantee that reading can never cause a problem.

**Correction to earlier reports:** AL-255's V13 `0xFFC02532` is a file-relative
annotation; adding VMA `0x02000120` modulo 32 bits gives RAM `0x01C02652`.
It must not be used as an absolute ROM entry. V15's independently checked target
above is `0x01C026A4`. This correction does not resolve the separately mapped
RAM loader's external startup target `0x01E00428`.

### Available stock SPL: scoped follow-up

The package also supplies an encoded `uboot.boot` at physical flash `0xA0`,
length 14,384. Separately decoding its 16-byte header and remaining payload
matches the previously retained stock/SDK BANKCB hash above and JLFS CRC `71C8`.
Header CRC `BCF5` and payload CRC `81A2` pass. Its single bank specifies data
offset 16, length `0x3820`, load `0x01C02000`; thus actual code maps to
`[0x01C02000,0x01C05820)`. Mapping the container header as code would shift
addresses by 16 bytes. The mapped `usb_update_mode` string is `0x01C05710`.

A bounded block at `0x01C04DB4–0x01C04DF0` validates a CRC over 78 bytes
beginning at `0x01C7FD8A` against the update record and copies that record.
This block does not establish a comparison of the first 16 mailbox bytes.
String-address references can be anchors for adjacent data and cannot alone
identify a consumer. No complete USB-marker-to-ROM delegation path or exact
FD07 dispatcher was recovered in this bounded follow-up. It does not prove
that no other SPL path consumes the mailbox. The external `0x01E00428` body
is outside this bank's declared mapping; these bytes do not resolve that call.

The same static reader checked 4,889 SPL listing rows against input bytes;
59 unknown rows include mixed data. This is not a complete SPL call graph.
Original extraction/mapping/listing receipts and the separate
`[local artifact location omitted]` retain the bounded findings; their
hashes are included in the sanitized evidence manifest. No boot code ran.

### Static reader and independent checks

The reader SHA-256 is
`[digest omitted from public archive]`.
It ran with `-d -triple=pi32v2` in the already cached image
`sha256:[digest omitted from public archive]`,
`--pull=never --network=none --read-only --cap-drop=ALL`, no host devices and
read-only mounts of the reader and one inert carrier. No target code was run.
All 209,790 application and 16,031 RAM listing rows were byte-compared with
their input slices; linear listings also include data/unknown instructions.
Separately restarting disassembly at six selected boundaries matched **195
instruction rows with zero unknowns** against the full listings. This scoped
check does not claim that all application instructions have been analyzed.

## Package parser compatibility

The existing Python and Rust structural parsers rejected every empty entry,
including the manufacturer's valid empty metadata marker. They now accept an
empty entry only when all these checks hold:

- Type 2; ordinal equals its actual table index.
- Exact 16-byte, NUL-padded `info.log` name; zero data CRC and reserved fields.
- Existing offset/header/package bounds and overlap checks still pass. Sharing
  a payload boundary is accepted; a marker strictly inside a payload refuses in
  either table order. Empty raw flash and other empty entries still refuse.

Independent synthetic builders repair table/header CRCs after each adversarial
mutation, so tests exercise the metadata rule rather than failing an earlier
checksum. No production hardware, writer, catalog qualification or CLI activation
is added. Nonempty unknown-entry behavior is unchanged. The actual downloaded
input is checked separately from fictional committed fixtures.

## ROM read evidence and one-time alternatives

The [primary ROM protocol warning](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/usb-loader-v2.md#L176-L193)
describes source-memory encryption before USB send and restoration afterward.
The exact WL82 implementation, restoration/failure behavior and active-memory
ownership for A `[0x01C07E00,0x01C08200)` and B header
`[0x01C08E00,0x01C08E22)` were not obtained. Host read blocks are bounded at
512 bytes with a fresh wire-transform state per request; a continuous 1,024-byte
host decode differs from two separately decoded 512-byte requests. The minimal
candidate configuration capture is **512 + 512 + 34 = 1,058 bytes**, before any
loader jump. It is not a full flash backup.

The [manufacturer SDK linker](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/51712dd206536f165499f2313454b486b19a6175/cpu/wl82/sdk_ld_sfc.c)
names external-flash XIP at `0x02000120` as `rom`; it is not an immutable-ROM
image or a map of ROM download-runtime ownership. Its application RAM bounds
cannot prove FD07 avoids active stack/USB/DMA aliases. The available SDK archive
symbol indexes and foreign BR25 documentation do not fill this exact gap.
`kurogedelic/fm-1-research-lab` returned public HTTP 404; no source body was
obtained. These are bounded failed leads, not a claim that no such artifact exists.

| Existing option, pinned revision | Can it satisfy this owner's current strict route? |
|---|---|
| [Transporter](https://github.com/kurogedelic/FM-1-transporter/tree/a632d923203170e3a08565f4056dbb3fd3d65ca5) | Implements stock soft-key entry and full reads through an RP2040. Default loader startup and FC0A keepalive/info can reach status changes; the dongle does not remove that audit requirement. |
| [jl-uboot-tool](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jluboottool.py#L698-L749) | Direct Windows/Linux path expects ROM already present, uploads/starts the loader and uses FC0A in its prelude. No reviewed macOS full-backup workflow is supplied. |
| [Unbricker](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L920-L962) | Already-ROM Windows/Linux backup does two reads, but initialization/FC0A remain. Its optional 512-byte ROM read at `0x01C7C000`, swallowed on failure, is an unqualified extra diagnostic and is not adopted. |
| [Slipway](https://github.com/Discodave-Glitch/slipway/blob/c067f969586d030483c78cf4eb7f5c646f65265d/slipway/gui.py#L959-L968) | “Save stock V15” imports a downloaded package into a host library. It does not dump the connected unit; recovery instructions use the already-ROM Windows/Linux route. |

Therefore no reviewed default command can be presented as a one-time full backup
meeting running-stock V15 + macOS + USB-C + strict no-flash/status-write effects.
Moving to another OS or buying a dongle alone does not fix loader side effects.
A preset export can preserve supported user data, but cannot substitute for
boot head, raw application, current settings/calibration and tail coverage.
Do not run restoration/updater commands to obtain a backup.

## Remaining sequence and ticket disposition

R4-C is **partially complete**, not physically qualified:

1. **C1 complete offline:** acquire exact manufacturer V15; bind extraction,
   VMA, initialized RAM, handler/prefix/trailer and reset-callee subgraph; fix
   genuine metadata parsing and correct earlier ROM attribution.
2. **C2 next evidence:** qualify the reset-to-ROM mailbox-consumer path and its
   autonomous effects after the evidenced P33 request. Establish the exact ROM
   FD07 transform/restoration and ownership of the selected A/B ranges only,
   plus a documented no-flash return
   from that ROM state. Exact ROM source/disassembly or suitably scoped external
   bench evidence can satisfy this; unrelated ROM commands/all-memory coverage
   are not requirements. Do not guess addresses or try optional diagnostics.
3. **C3 selected macOS boundary:** evidence that the selected stock endpoint
   returns the required 8-byte USB-MIDI buffer to the V15 handler on a quiet
   selected stream, retain physical-object
   continuity across reset/re-enumeration, and qualify the actual selected
   access/cancel/close operations. Identity alone does not prove packetization.
   [Follow-up 41](41-rom-fd07-macos-entry-return.md) distinguishes the receiver's
   actual buffer from USB transfer grouping: one eight-byte transfer is the known
   candidate, not a proven universal necessity if the reader aggregates events.
4. **C4 first bounded hardware capture:** once C2/C3 hold, use existing backup
   permission for the finite A/B ROM-only capture, stop before FB08 and assess
   untouched bytes through R1. No renewed generic permission question, forced
   configuration or alternate firmware is needed.
5. **C5 full reads:** qualify the unchanged loader's external startup/callbacks,
   exact chip and no-status mode-2/read subset from reports 32–34. Compose the
   already implemented native reader/two-copy storage behind a separately
   reviewed activation change. Then take two fresh equal full reads, verify
   reopened independent copies and final continuity, and observe original stock
   after the chosen return. Failures stop without automatic reset/retry.

No previous full backup, writer/restore test, Linux/Windows support or universal
unbrick guarantee is a prerequisite for the first narrowly qualified macOS read.
Acquiring this package resolves the missing-V15-input gate; it does not waive the
remaining operation-specific evidence. A date for the real backup would be
speculative until those concrete gaps are closed.

## Validation and delivery

GPT 6.1 Sol **high** subagents traced V15 and ROM effects; a **medium** subagent
researched alternatives and implemented parser regressions. Root reviewed raw
instructions, corrected earlier reports and integrated the findings. Firmware,
PIO, submodule pins, license notices and dependency locks remain unchanged.

- **317 guarded Python tests passed**, including three new regression groups
  and the existing independent fixture regeneration/no-hardware checks.
- Canonical native source/API audit passed for **48 Rust files**; **11 package
  tests and four conformance tests passed in both debug and optimized builds**.
  The new groups cover 74 CRC-repaired metadata corruptions and 85 total
  acceptance/refusal cases per language, including both overlap orders.
- The original downloaded package passed Python `verify_v15` under the offline
  guard and a separate ignored Rust verifier using compile-time local bytes,
  default-feature library APIs and exact extraction expectations. Both returned
  the independently bound 585,728-byte APP hash above. No test embeds vendor
  firmware into the committed corpus; structural `artifact_qualified` remains false.
- `cargo fmt --check`, diff whitespace checks and the native-USB audit passed
  (nine binding Rust files, 34 pinned registry dependencies; activation closed).
  No platform adapter was executed and no new physical-platform result is claimed.

Reproduce the software checks with Python 3.11+ and the existing locked toolchain:

```sh
rtk proxy python3.12 host/run_tests.py
rtk proxy python3.12 native/run_tests.py --test lib --filter packages
rtk proxy python3.12 native/run_tests.py --test lib --filter packages --release
rtk proxy python3.12 native/run_tests.py --test conformance
rtk proxy python3.12 native/run_tests.py --test conformance --release
rtk proxy python3.12 native-usb/run_tests.py --audit-only
```

The real-input check uses the exact ignored artifact and separate file-only
verification described above; those synthetic test commands alone do not prove
the contents of a newly downloaded package.
