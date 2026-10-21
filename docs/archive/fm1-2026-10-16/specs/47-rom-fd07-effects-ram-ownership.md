> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# ROM FD07 effects and first-context RAM ownership — R4-C

Date: **2026-10-16**, branch `init`. Offline analysis with GPT 6.1 Sol high
agents for archive bodies/RAM ownership and a medium independent verifier.
No network, hardware inventory/open/query, target execution/emulation, vendor
script/import, firmware compile/link/build, backend installation or flash write.
Continues [reports 41](41-rom-fd07-macos-entry-return.md),
[45](45-official-30k-loader-no-status-read.md) and
[46](46-macos-entry-packetizer-v15-receiver.md).

## Result

**The available local artifacts still do not establish the exact bare-ROM
FD07 transform/send/restoration effects or ownership of the first capture
intervals. C2 remains unresolved and no context capture is enabled.**

This pass adds body-level evidence from previously unexamined SDK archive
members and exact V15 interval/lifetime evidence. The archives are an OTA
UBOOT2 build with a real flash-write edge, not a bound implementation of the
selected UBOOT1 ROM. The candidate RAM is demonstrably V15 data/state before
reset; the application and SPL maps do not identify its later ROM occupants.

The evidence gap is specific: a matching implementation/allocation relationship
or suitably scoped external-bench evidence for this selected read. Repeating
the same local path/symbol searches, substituting another boot artifact or
probing the owner's only device does not supply that evidence. Backup-read
permission exists; this task explicitly remains offline. No prior full backup,
write/restore trial, all-ROM coverage or other-OS qualification is added.

## Selected capture and separate representations

The implemented R1 subset still binds the **24,064-byte** encoded helper
SHA-256 `[digest omitted from public archive]`,
decoded `[digest omitted from public archive]`.
The optional30 KiB helper audited in report45 is a separate candidate; it is
not selected by this investigation.

| Implemented R1 input | Half-open source interval | Bytes |
| --- | --- | ---: |
| A first half | `[0x01C07E00,0x01C08000)` | 512 |
| A second half | `[0x01C08000,0x01C08200)` | 512 |
| B header | `[0x01C08E00,0x01C08E22)` | 34 |

Total is **1,058 logical bytes**, stopping before FB08. The three inert CDBs
remain `FD0701C07E000200FFFFFFFFFFFFFFFF`,
`FD0701C080000200FFFFFFFFFFFFFFFF`, and
`FD0701C08E000022FFFFFFFFFFFFFFFF`. These are saved geometry evidence, not
an enabled command interface. Selecting B would require its full configuration
or refusal; the narrow implemented case cannot assume B's missing tail.

The retained [FD07 protocol description](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/usb-loader-v2.md#L176-L193)
warns that some ROMs transform source RAM in place, send via USB DMA and restore
it. The host WL82 `memory-rw-mengli-crypt` flag selects host handling; it is
not the exact target implementation or a failure-restoration guarantee.

Keep three representations distinct: **ROM wire ciphertext → actual pre-loader
RAM → loader-decoded configuration**. Each actual ROM request has its own host
wire-transform state. R1's later header/configuration transforms are separate.
Joining the two512-byte wire decodes into one continuous1024-byte transform
does not preserve their result. Loader-phase FD07 is a separate local copy
implementation and cannot establish bare-ROM effects.

## New SDK archive-body evidence

The two previously saved official OTA-branch archives are pinned to commit
[`5c33e71d9e4762595aea28587e76d4068f353870`](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/tree/5c33e71d9e4762595aea28587e76d4068f353870/uboot/include_lib/liba/wl82/usb_update2).
No new retrieval occurred.

| Archive | Bytes | SHA-256 |
| --- | ---: | --- |
| cpu_lib.a | 370,598 | `[digest omitted from public archive]` |
| logic_lib.a | 693,582 | `[digest omitted from public archive]` |

Original file-only ar parsers extracted **20 selected members /374,904 bytes**;
the existing LLVM23.1.3 bitcode reader inventories **258 function definitions**.
This is not a claim that258 complete control-flow bodies were individually
audited. The selected relevant bodies/constants establish:

- `usb_reg.o` defines inquiry data **WL82 UBOOT2.00**, while `usb_slave_ctl.o`
  contains descriptor VID/PID **E3B5:2305**, rather than the selected4C4A:8057.
- Seven selected logic USB/command members contain **zero function definitions**:
  hook_dev_rw, usb_slave_scsi, usb_slave_reg, usb, usb_slave_update,
  usb_slave_ctl and usb_slave_api. This is body-level evidence, beyond an
  exported-symbol-index search; it says nothing about unexamined members.
- `mask_dev_interface_open_syd` really reads/verifies a flash header/file record
  through supplied callbacks. The word `mask` supplies no immutable-ROM identity.
  `ex_api_code_load` returns0; its status callback is a no-op.
- `uboot_start_first` reads/compares a boot record and, on a selected mismatch
  branch, calls **spi0flash_any_write**. This is a real autonomous boot/update
  edge in this artifact, **not evidence that the selected ROM does so**.
- `usb_g_ep_write` has concrete chunk-copy/status/timeout paths and returns after
  queuing its final packet without separately waiting for that packet's completion.
  Nothing binds this unlinked body to ROM callback0x01E008E4 or FD07. Its source
  nonmutation/return behavior cannot qualify target-ROM restoration or DMA order.
- `get_updata_ram_end_addr` returns external `UPDATE_BEG+8`, with no absolute
  allocation. The memory header describes application virtual-memory constants;
  neither it nor reset-source reporting supplies an active-ROM RAM map.

All20 selected bitcode reads exited zero. Three members report invalid legacy
debug attachments; the reader ignores that debug metadata and retains bodies.
Full warnings and digests are saved. Debug paths, symbols and shared MMIO
constants are not a linked native map to0x01E00428/008E4/0089A. No matching ROM
image was obtained or relabeled, and no new REA native session was opened.

## Exact V15 RAM occupancy: package facts and lifetime limits

The exact package initializes `[0x01C00000,0x01C0A05C)` from its app image,
SHA-256 `[digest omitted from public archive]`.
Startup also clears BSS `[0x01C0A05C,0x01C2361C)`. These ranges describe
**application startup**, not a snapshot of the running owner unit or ROM.

| Interval | Saved initialized bytes / nonzero bytes | Occupancy evidence |
| --- | --- | --- |
| Selected old A `[7E00,8200)` within0x01C00000 | 1024 /777 | Fully inside V15 initialized RAM; contains the USB special prefix at0x01C0800D and data read by RAM routines0x01C047F4 and0x01C04E24. |
| Selected old B header `[8E00,8E22)` | 34 /26 | Fully inside V15 initialized RAM. No later ROM ownership established. |
| Optional30 KiB A `[9800,9C00)` | 1024 /6 | Six initial nonzero bytes at0x01C0995E..9963. V15 explicitly sets byte0x01C09954 to1 at0x02005196..51A0 and clears it at0x02038EE8..8EF4. |
| Optional30 KiB B header `[A800,A822)` | No file bytes available | Inside V15 BSS initialization range; file absence is not evidence of free or still-zero live RAM. |

Full interval digests are recorded in the manifest. In particular optional A
is **not** an all-zero region: its first64 zero bytes do not describe its tail,
and initially zero byte9954 is later mutable. Neither optional interval is
adopted as a way around ownership evidence.

Applying the configuration transform to the **historical initialized bytes**
is also not a default-mode witness. Old A's decoded header CRC field is57703
versus computed12595; old B is14748 versus12076. Optional A is413 versus64378.
These are invalid but **nonzero** fields, with a nonzero decoded first-record
byte. They fail the narrow zero-fields/no-key predicate. This does not predict
actual ROM configuration, prove a dangerous actual mode or attest installed bytes.

The exact SPL maps executable bytes to `[0x01C02000,0x01C05820)`. Its startup
clears `[0x01C05840,0x01C065A0)`, installs SSP=0x01C05C40 and SP=0x01C06040.
Those explicit initial bounds do not cross the selected A/B intervals. That
does **not** bound later stack growth, reset entry into/bypass of SPL, ROM
stack/interface/CBW buffers, DMA, aliases, interrupts or other active users.
V15 application initial SP/SSP and USB buffers are likewise application-phase
facts. They cannot be inherited by a new ROM execution phase.

## Logical data minimization is separate from touched memory

The existing1058-byte R1 format is an implemented assessment/input contract,
not an information-theoretic minimum. For **only** the zero-CRC-fields/no-key
case, A's first35 bytes and B's first34 bytes are sufficient contiguous prefixes:
the CRC/XOR keystream prefix does not depend on total input length. Two zero
decoded CRC fields reject A/B/A/B, leaving A; decoded A byte34=0 terminates
every key lookup. Two independent implementations check this69-byte sufficient
case and prefix equivalence, including nonzero synthetic inputs.

This is **not** a globally minimal claim, a shorter capture profile or a new
command proposal. R1 still requires1058 bytes. Exact ROM alignment/rounding,
transform and DMA spans are unknown; requesting fewer bytes does not prove it
touches fewer bytes. Later loader startup still transforms the full selected
1024-byte block in place. Logical byte minimization cannot remove either
ROM-ownership or loader-ownership requirements. No shorter CDB is emitted or
enabled, and this optional refinement does not block the existing route.

## What remains necessary for the first context capture

| Selected boundary | Exact missing fact | Why current evidence cannot replace it |
| --- | --- | --- |
| Reset → ROM idle | Matching mailbox consumer and autonomous effects for the selected entry/revision | V15/SPL marker producers and unrelated OTA boot bodies do not identify that consumer. |
| FD07 source access | Actual address/length interpretation, transform condition, alignment/rounding and complete touched interval | Host length checks and decoded output establish neither target writes nor their extent. |
| Send → restore | Transform/send/DMA-completion/restoration ordering on success and relevant short/error/timeout/cancel/disconnect paths | A generic endpoint sender or loader FD07 is not a bound ROM dispatcher/callback closure. |
| Active ownership | Source and all actual touched bytes avoid stack, interface, CBW, scratch, descriptors, DMA and asynchronous users for the selected operation | V15 startup data and SPL initial stacks describe different lifetimes. |
| Capture continuity/return | Stable selected context and the already scoped macOS access/object continuity and preselected return | Equal bytes, host close, local timeout or a metadata report cannot establish target restoration or stock return. |

Matching implementation evidence need cover **these selected paths and intervals**,
not all ROM commands. Suitably scoped external qualification can instead supply
the needed behavior/ownership evidence without requiring a write experiment.
No prescribed method for acquiring ROM bytes on the owner's unit is introduced.

**Disposition:** the unexamined retained OTA-body lead has now been checked and
the RAM-phase assumptions refined. With this unchanged local material, the
exact-ROM effects/ownership qualification cannot be closed. The next useful
input is a provenance-bound matching ROM implementation/allocation artifact or
matching external read/restore/ownership evidence. This is an external-evidence
dependency inside R4-C, not a new permission request or another fake model.
Once the selected C2/C3 operation qualifies, existing read permission can cover
the finite context capture ending before FB08. C5's helper/chip/timing and later
full double-read/two-copy acceptance remain separate.

## Evidence and validation

The sanitized manifest (historical reference `evidence/47-rom-fd07-ram.json`; omitted from public archive) binds source/member/IR
digests, exact intervals, transform geometry, diagnostics and false authority
fields. Raw package/library bytes, IR, selected listings and original file-only
scripts stay ignored under `[local artifact location omitted]`.

Checks include source API/base64/Git-blob/archive-index equality,20 exact member
payloads/IR hashes, independent interval/CRC/transform arithmetic, selected V15
and SPL instruction-byte checks and independently starting bounded decodes.
No definition count is promoted to whole-library analysis; no same-decoder
agreement proves peripheral or runtime behavior. Documentation/JSON/hash/link
and diff checks apply. No product-suite rerun is claimed for docs-only research.

Work is local on `init`, without push. Existing unrelated rescue-source changes
are preserved. Entry/capture factories, helper selection and all physical writes
remain closed; no RAM context, full backup or physical return was observed.
