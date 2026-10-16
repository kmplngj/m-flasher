# FM-1 M1: offline reader build

Date: 2026-10-16. Baseline: M-Flasher
`9d962747437d92cbab594c2320b586d162c6572d`; working branch `init`.
Tracking: [#3](https://github.com/kmplngj/m-flasher/issues/3), within
[the transferred roadmap](https://github.com/kmplngj/m-flasher/issues/1).

The minimal RAM reader is implemented and reproducibly built as a separate
offline candidate. The existing CLI, loader images, adapters and dependency pins
are unchanged. No hardware was enumerated or opened, no image was uploaded or
executed/emulated, and no flash operation occurred. This result does not yet
provide a usable physical backup command.

## Exact artifact

Two independent builds in fresh output directories produced identical bytes
for all 12 recorded outputs and identical build receipts. They used the same
frozen inputs, existing toolchain and cached image, with container networking
disabled. The [machine-readable evidence](evidence/fm1-m1-build.json) records
input/tool/output hashes and the limits of each check.

| Property | Result |
| --- | --- |
| Build ID | `3830fc94` |
| Input fingerprint | `3830fc9499f6a2a16cfcb2f2a6a57b2b623c1828f070adccb81ce15d11189a44` |
| Raw upload image | 1,456 bytes |
| Raw SHA-256 | `63c12a9ded28b53fa991c8261cbc8065ef93f8e71a805f53553999175422618a` |
| ELF SHA-256 | `f5078a8a399897565a35d4f4324e4d1676d880192b775e31770cef5d9d653c67` |
| Entry/text | `[0x01C20000, 0x01C205AC)`; 1,452 bytes |
| Initialized entry marker | `[0x01C205AC, 0x01C205B0)`; 4 uploaded bytes |
| BSS | `[0x01C205B0, 0x01C206CC)`; 28-byte state and 256-byte scratch |
| Fixed reservation | `[0x01C20000, 0x01C28000)` |

The raw image includes the fresh entry marker `31 4D 52 46`. The vendor objcopy
requires one section selector per invocation, so the builder extracts and
checks `.text` and `.data` separately before packing them. Omitting `.data`
would leave startup dependent on stale RAM. The entry explicitly initializes
state and scratch; it does not assume BSS has been cleared by the ROM.

The toolchain lock binds all five used executable/plugin files and the cached
container image. Source/build input hashes are separate from the final image
hash; the protocol returns an input-derived build ID, not a self-embedded final
SHA-256. See [build instructions and protocol](../ram-reader/README.md).

## Verified offline behavior

The hosted protocol-core harness and the actual SPI source compiled with
simulated MMIO both passed C11 warnings-as-errors, AddressSanitizer and UBSan on
the local Apple Silicon host. They execute only host-native code with injected
callbacks/registers. Literal wire vectors are independent of the implementation.

The core checks all 65,533 forbidden 16-bit opcodes and 765 single-byte variants
of the sole candidate JEDEC ID, as well as canonical framing, size, first/last
block, overflow, pointers, dirty state, permanent failure and reentrancy. The
MMIO model checks exact `9F`/`03` transactions, all four JEDEC and five read
completion failure phases, a shared finite polling budget, DMA configuration
and cleanup. It assumes pending/W1C semantics and does not generate DMA payloads
or qualify chip behavior.

All **37 artifact mutation tests** passed against the final artifact. They cover
ELF/raw geometry, loaded marker, section flags/types/alignment, required symbol
membership/function bounds, invalid instruction markers, listing byte coverage,
unexpected symbols and nonlocal direct calls. The positive artifact has 12
defined functions and 506 fully byte-matched disassembly rows covering all
1,452 text bytes, with no unknown instruction rows.

Independent static review also checks local call/tail-jump targets from encoded
displacements, the local SPI/MMIO path and the single opaque send callback.
Local stack peaks are 68 bytes for entry and 72 bytes for handler/read, excluding
ROM callback frames, interrupts and arbitrary reentrancy. These local figures
do not establish a safe supplied stack or global stack bound.

## Result and remaining gates

M1 has only artifact identity, JEDEC metadata and one aligned 256-byte raw-read
operation. The supported JEDEC `85 60 14` and 1 MiB capacity remain a candidate
based on external evidence. The local SPI command set is `9F` and `03`; there is
no NOR write-enable/status/program/erase command, FC0A, direct vendor
`0x01E00428` call or unknown-command fallback. Pointer and framing failures
poison the session; failed reads never send scratch bytes. A completed-DMA
counter and the ROM send return value cannot attest USB delivery.

The next implementation work is strict transport
[#4](https://github.com/kmplngj/m-flasher/issues/4) and the distinct reader host
profile [#5](https://github.com/kmplngj/m-flasher/issues/5). The current constructor
with its probes/retries/fallback must not invoke this artifact unchanged.
Actual fresh CBW delivery, ROM entry/hook/send and consumption ordering, live
code/state/stack/buffer ownership, chip/MMIO effects and selected macOS access
remain operation-specific gates. The new ABI has no supplied CBW byte length;
range checks alone cannot establish complete fresh command delivery.

A separately scoped small read-and-return task
[#8](https://github.com/kmplngj/m-flasher/issues/8) precedes full capture.
Original-stock entry and return are tracked in
[#7](https://github.com/kmplngj/m-flasher/issues/7). Two fresh complete raw passes
[#9](https://github.com/kmplngj/m-flasher/issues/9) and two durable independent
copies [#10](https://github.com/kmplngj/m-flasher/issues/10) remain distinct from
this build result. No prior full backup, write/restore experiment, all-ROM
reconstruction or qualification of every operating system is a prerequisite
for a narrowly evidenced first read. The sole owner unit remains excluded from
writer/fault/agent development cycles.
