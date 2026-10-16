# FMR1: minimal FM-1 RAM reader

This source-defined reader is an offline candidate for the stricter FM-1 backup
route in [issue #3](https://github.com/kmplngj/m-flasher/issues/3). It is built and
tested separately from the existing M-Flasher loader and application. It is not
connected to the current CLI, and no hardware profile is enabled by these files.

The candidate sends only JEDEC Read ID (`9F`) and fixed raw Read Data (`03`) to
the NOR chip. It contains no NOR write-enable, status-register programming,
erase, program or unprotect command. It also removes the original loader's
ROM fallback, vendor initializer, optional peripherals and receive path. It
still executes code in RAM and changes peripheral state; physical startup,
read completion, memory ownership and unchanged-stock return remain separate
qualification work in [#5](https://github.com/kmplngj/m-flasher/issues/5) and
[#8](https://github.com/kmplngj/m-flasher/issues/8).

## Build only, using existing local tools

Use Python 3.11 or newer, the already installed pinned JieLi toolchain, and the
already cached container image identified in [toolchain-lock.json](toolchain-lock.json).
The builder refuses mismatched tool hashes and never installs or downloads
tools. It compiles only this reader, with networking disabled and no device
mounts in its container. Each output directory must be new and below `target/`.

```sh
rtk proxy python3 ram-reader/build.py --output target/fmr-reader/first
rtk proxy python3 ram-reader/build.py --output target/fmr-reader/second
```

The default toolchain directory is `~/.jieli/<toolchain_revision>`. An explicit
`--toolchain PATH` still requires the exact locked bytes. The build ID binds the
C/header/linker sources, builder, verifier, toolchain lock, compiler flags and
container image; it is not a self-embedded hash of the final binary.

Outputs include the ELF, raw load image, map, symbols, disassembly, build receipt
and structural audit. The vendor `objcopy` extracts `.text` and `.data`
separately; the builder checks and packs both at their linked offsets. The
initialized entry marker in `.data` is required. BSS is excluded from the
upload and initialized explicitly by the entry code. Reproducibility compares
the actual ELF/raw/object/disassembly bytes from independent builds, not just
their build ID. These artifacts are inert files; the build does not execute,
emulate, upload or install them.

## Candidate wire contract

This ABI is distinct from the combined upstream and vendor helpers. It does not
inherit their FC0A probe, 32-byte debug interface, A/B pre-read or 84/88-byte
metadata. All multibyte fields below are big endian unless specified otherwise.

Every command is one canonical 31-byte BOT CBW: `USBC` signature, little-endian
transfer length, direction `80`, LUN zero and a 16-byte CDB. The reader never
interprets the host tag; the future host must check the returned CSW signature,
matching tag, zero residue, status and exact transfer counts independently.

| CDB | Data IN | Fields |
| --- | --- | --- |
| `FC40` then 14 zero bytes | 16 bytes | Opcode (2), `FMR1` (4), ABI `01`, status, build ID (4), completed DMA count (4) |
| `FC0B` then 14 zero bytes | 16 bytes | Opcode (2), JEDEC (3), status, capacity (4), read size `0100` (2), four zero bytes |
| `FD05`, address (4), `0100`, eight zero bytes | 256 bytes | One raw physical flash block; address must be 256-byte aligned and at most `000FFF00` |

Only JEDEC `85 60 14` is a candidate, with a fixed 1 MiB capacity. This is bound
to external FM-1 evidence and is not an observation of the owner's chip. All
other IDs report unsupported/zero capacity and cannot read. There is no generic
capacity discovery or support for other chips in this candidate.

Status values are `0` OK, `1` unsupported chip, `2` initialization failed,
`3` bad command, `4` read failed, `5` reentrant invocation and `6` invalid pointer.
Failure poisons the session: identity and JEDEC may report the retained error,
but reads cannot resume. A malformed, forbidden or unknown command is consumed
without intentionally delegating to the ROM. No automatic reset, retry or
fallback exists. Failed or poisoned reads do not send the scratch payload.

The saturating counter counts completed 256-byte SPI DMAs. It does not count
successful USB transfers. The opaque ROM send callback's return value is not
interpreted as success. Fresh buffer consumption before reuse and the actual
USB data/CSW completion are integration and qualification gates.

## Memory and execution contract

The link reservation is `[0x01C20000, 0x01C28000)`; the entry is its first byte.
The supplied ROM interface is exactly three 32-bit words: send, unused receive,
and a pointer to a separate writable handler slot. The loadable entry-once marker
is `0x46524D31`, serialized as `31 4D 52 46`. State and scratch do not assume
zero-initialized BSS. Repeated entry or observed reentrancy poisons the session.

The core checks natural alignment, overflow-safe SRAM/ROM ranges and disjoint
interface/slot/CBW/answer/reservation regions. Those numeric checks cannot prove
that live ROM code owns the supplied memory or stack. Invalid entry arguments
can return before installing a handler; do not infer that this disables an
existing ROM dispatcher. The handler ABI has no CBW-length argument, so fresh,
complete 31-byte command delivery must be established externally.

Startup and each read share a finite 262,144-iteration polling budget across
their byte transfers and DMA. Cleanup disables the SPI controller, clears its
count, disables ENC and deasserts chip select. The budget is not a calibrated
wall-clock deadline. Actual pending/W1C, cancellation, DMA and send/interrupt
semantics require selected device evidence. No complete ROM reconstruction is
required, and foreign vendor helper contracts are not prerequisites.

## Offline validation

Run the hosted fake-core and simulated-MMIO harnesses using the exact commands
in [tests/README.md](tests/README.md). They use AddressSanitizer and UBSan and
never link the application's hardware adapters. The MMIO harness compiles the
same SPI source with every register access redirected to local storage; its
assumptions are not silicon evidence.

After an offline artifact build, run the pure file-parser mutation suite:

```sh
rtk proxy python3 ram-reader/tests/test_artifact.py --artifact target/fmr-reader/first
```

The artifact checker binds section/symbol/image geometry and full disassembly
byte coverage and rejects unexpected symbols and nonlocal direct calls. It is
a structural audit, not a proof of every instruction's behavior. The recorded
manual control-flow/MMIO review complements it. The build report in
[docs/fm1-m1-offline-build.md](../docs/fm1-m1-offline-build.md) records the exact
tested hashes and remaining gates. The broader implementation sequence and
transferred findings are in [the roadmap](../docs/fm1-roadmap.md).

## License

This derivative preserves the Apache-2.0 lineage and notices from M-Flasher and
fm1-bootloader. See [LICENSE](LICENSE) and [NOTICE](NOTICE). Existing upstream
loader files, embedded payloads, application adapters and dependency pins are
unchanged.
