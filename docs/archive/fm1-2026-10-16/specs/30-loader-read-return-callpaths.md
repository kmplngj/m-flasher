> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Exact loader READ and RETURN call paths

Date: **2026-10-15**. Scope: offline FM1-033-D2 analysis of the exact loader
in [report 23](23-loader-static-followup.md), using the existing vendor pi32v2
object dumper and inert analyst metadata described in report 31.

**The actual `FD05`, `FC0A` and `FC0C` dispatch paths are now recovered.
`FC0A` is not a harmless keepalive: its normal padded request reinitializes
NOR and can reach status-register writes with protection-mask target zero.
The physical no-flash-write qualification remains false.** This proves a
conditional write-capable path in these exact bytes; it does not prove that the
owner's unknown chip would take that branch or that every status write is
nonvolatile.

Only saved files and original static extraction code were used. There was no
network access, hardware inventory/open/probe/command, firmware build, loader
execution/emulation, backend installation or vendor script execution. No host
implementation, activation boundary, upstream firmware, PIO, pin or license
notice changed. No commit was created by this subtask.

## Artifact and interpretation boundary

The encoded 24,064-byte artifact has SHA-256
`[digest omitted from public archive]`;
its retained per-512-byte decode has SHA-256
`[digest omitted from public archive]`.
Every mapped address below uses base **`0x01C02000`**. A separate `file offset`
column prevents confusing a mapped address's low digits with a file position.
No symbol names below came from this stripped loader: descriptive names identify
observed behavior, not recovered source identities.

The root audit found an already installed vendor LLVM 4.0.1 pi32v2 `objdump`.
It read an inert synthetic ELF containing the unchanged decoded bytes; report 31
records its exact identity, wrapper and isolated file-only invocation. The wrapper
is analyst metadata, not a linked firmware build or a vendor section map. Inline
branch tables in a linear listing are data even when the dumper prints invented
instructions for them. The table arithmetic below is checked directly against
raw bytes. Parallel instruction pairs, marked `#` by the vendor dumper, read
pre-pair register values; treating their printed lines as sequential operations
would produce an incorrect backend lookup and hook assignment.

Temporary evidence is under
`[local artifact location omitted]`. Original
`extract_read_return.py` rehashes the exact decoded file, verifies every displayed
instruction byte against it, extracts 27 bounded ranges, independently checks
13 selected direct-call displacements, resolves all 14 low `FC` table entries,
and verifies the six NOR operation pointers. Independent carriers beginning at
the key targets reproduce their instruction boundaries and mnemonics; the
boundary comparison is saved in `independent-boundary-check.json` and reports
1,314 instruction rows with zero byte, boundary or mnemonic differences.
`manifest.json` records addresses,
file offsets, source-listing line ranges and hashes. These temporary files are
inspection evidence, not a bundled loader or redistribution grant.

## Boundary-anchored dispatch and backend table

| Observed boundary | Mapped address | File offset | Exact evidence |
|---|---|---|---|
| Installed command hook | `0x01C03E4A` | `0x1E4A` | Startup loads this address at `0x01C028A4`; its parallel store at `0x01C028AC` writes it through the ROM-supplied hook pointer |
| Any command resets idle accumulator | `0x01C03E70` | `0x1E70` | Stores zero to `G+44`, where `G = 0x01C09700` |
| Prefix dispatch | `0x01C04198`, `0x01C041A2` | `0x2198`, `0x21A2` | Compares first CDB byte with `253` (`FD`) and `252` (`FC`) |
| `FD05` branch | `0x01C0424C` → `0x01C0437C` | `0x224C` → `0x237C` | Compares next CDB byte with `5`; direct conditional target is the read handler |
| Low `FC` table | `0x01C042C4` | `0x22C4` | `tbh`; table begins at `0x01C042C6`, indexed by `(subcommand-9)*2` |
| `FC0A` table entry | `0x01C042C8` → `0x01C0462C` | `0x22C8` → `0x262C` | Little-endian entry `0x01B3`; target `table_base + 2*entry` |
| `FC0C` table entry | `0x01C042CC` → `0x01C0477C` | `0x22CC` → `0x277C` | Entry `0x025B`, with the same arithmetic |
| Registered NOR device | `0x01C07C28` | `0x5C28` | Three words: name `0x01C0560C` (`norflash`), operations `0x01C07964`, instance `0x01C0974C` |
| NOR operations | `0x01C07964` | `0x5964` | Six words: init `0x01C04E3C`, read `0x01C04E1A`, ioctl `0x01C0505A`, null, write `0x01C05430`, null |

Device-open code at `0x01C02556` (file `0x0556`) scans that registration,
compares its name, calls its init operation, and puts the operations pointer at
returned-device `+8`. Thus the read indirection at `0x01C034F6` (file `0x14F6`)
resolves through operation `+4` to `0x01C04E1A`; ioctl wrapper `0x01C025B8`
(file `0x05B8`) resolves operation `+8` to `0x01C0505A`. This is an exact-byte
linkage, rather than a transfer of same-SDK library semantics.

## `FD05`: bounded chunks, SPI read core and remaining dependencies

At `0x01C0437C` (file `0x237C`), the handler assembles the four address bytes,
byte-reverses the combined value, and calls `0x01C03490` (file `0x1490`) to
assemble the two big-endian length bytes. Its loop uses at most **256 bytes**
per chunk. For each chunk it calls the registered read operation through
`0x01C034F6`, calls `0x01C0350A` (file `0x150A`) for conditional RAM data
transformation, then calls the transport function saved in `G+36`; it subtracts
the chunk length and advances the raw address. Zero length skips the loop.

The exact read operation at `0x01C04E1A` (file `0x2E1A`) conditionally waits for
busy to clear through `0x01C03A88` (file `0x1A88`), then tail-calls
`0x01C04CB4` (file `0x2CB4`). Busy polling sends `0x05`, reads a byte and tests
bit zero. The read core loads word `[G+76]` and branches on bits 8–15,
byte `G+77`, using its `TBB` data at file `0x2CE4`; see the exact selector
table in [session report 33](33-loader-no-fc0a-read-session.md).
Its paths issue `0x03`, `0x0B`, `0x3B`, `0x6B`,
`0xBB` or `0xEB`; the latter address/mode phases include `0x00` or `0xA0`.
Some modes retain continuous-read state in `G+80` and omit the read opcode on
later chunks. The address helper `0x01C04B7A` (file `0x2B7A`) conditionally
sends a fourth address byte using `G+52`. Data then transfers through the SPI
controller at `0x11C00`; chip select is controlled through bit zero of `0x500C0`.

No status-write, erase or page-program command is reached in the resolved NOR
read subgraph. In particular, the same-SDK IR's per-read `B7`/`E9` enter/exit
sequence must not be substituted for this implementation. This loader's
initialization has a separate capacity-dependent `B7` path at
`0x01C03D46` (file `0x1D46`). That startup/configuration path, continuous-read
state, all IRQ behavior and the ROM-supplied transport callback still require
the complete session-level effects audit.

The `FD05` handler does **not** consume the read operation's return value and
contains no demonstrated chip-capacity/address-bound check. Unsupported mode
branches return the requested count without a data phase. A successful handler
return therefore cannot certify fresh, correctly bounded NOR data. The normal
fresh-entry path leaves transformation flag `G+32` zero. The call at
`0x01C0350A` passes `(buffer=0x01C09600, length=chunk_length, raw_address)`
to `0x01C033F4` (file `0x13F4`). If `G+32` is zero that function returns
without changing the buffer. If nonzero, it requires 16-byte alignment and
modifies the RAM payload in 16-byte blocks through CRC/stream-cipher and
AES-shaped routines; those bytes must not be accepted as an untransformed raw
physical backup. This is a data transformation path, not a flash write. The
separate accepted `F5` sequence sets `G+32=1` at `0x01C040EE` (file `0x20EE`);
no such command is part of the scoped fresh-entry route. Startup ioctl `11`
changes byte `G+79`, which is a different state field. The raw-backup contract
therefore needs an explicit fresh exact session and zero transformation state
in addition to supported mode/range and fresh-data evidence. This report does
not qualify `F5` or treat `FD07` memory reads as raw NOR.

## `FC0A`: concrete conditional status writes

The generic host's empty argument list becomes trailing `FF` padding in its
CDB. In this exact loader, `FC0A` checks argument byte 4 at `0x01C0462C`
(file `0x262C`): a value other than `0x5F` jumps to `0x01C04AE0`
(file `0x2AE0`). The ordinary padded request takes that branch.

The common path copies saved configuration `G+68` into active `G+72`, calls the
NOR open/init route again, stores its returned device pointer, and invokes
ioctl `100` to retrieve the flash ID. A nonzero ID other than `0xFFFFFF` then
calls ioctl **`13` with argument zero** at `0x01C04B26–0x01C04B2E`
(file `0x2B26–0x2B2E`). Init itself reaches `0x01C03ACA` (file `0x1ACA`), so
its conditional quad/status configuration is also re-entered. A successful
response or repetition frequency cannot make this a preservation-only command.

Ioctl `13` resolves at `0x01C05092` (file `0x3092`) to the exact routine at
`0x01C0519E` (file `0x319E`). That routine reads SR1 with `0x05` and SR2 with
`0x35`, combines them, and compares **mask `0x41FC`** against the caller's
requested masked value. With argument zero, the requested masked bits are zero.
If they already match, the writer is skipped; the `FF/FF` observation also has
an early exit. Otherwise the following actual byte sequences are reachable:

| Exact instruction range | File range | Commands passed to the concrete SPI send-byte helper |
|---|---|---|
| `0x01C05238–0x01C05254` | `0x3238–0x3254` | Write-enable helper sends `06`; then `01`, new SR1 byte; busy polling |
| `0x01C05262–0x01C0527E` | `0x3262–0x327E` | `06`; then `31`, new SR2 byte; busy polling |
| `0x01C052D8–0x01C052FA` | `0x32D8–0x32FA` | After unsuccessful verification, `06`; then `01`, both new status bytes; busy polling |

The new low masked bits derive from argument `& 0xFC`; the high byte preserves
`old_SR2 & 0xBE` and merges the requested high bits. The routine rereads both
registers, compares the mask again, and retries with an eight-bit loop count
bounded below two. The sender at `0x01C03882` (file `0x1882`) clears controller
bit 12 and stores the supplied byte into `0x11C08`; this is not a string-only
association. `06` comes from `0x01C03A74` (file `0x1A74`).

A concrete static counterexample is: a valid nonzero/non-`FFFFFF` ID and,
after reopen, SR1 `0x04`, SR2 `0x00`, with the padded `FC0A` request. The
requested masked value is zero while the observed masked value is `0x04`;
neither early exit applies. The path therefore attempts `06`, then `01 00`,
and the separate second-register sequence. This is a branch/argument example,
not a measured owner-device state or an executed target simulation.

These exact instructions establish conditional status-register **write attempts**.
The actual flash ID/status, write-enable acceptance, register definitions and
volatile/nonvolatile behavior remain device-specific evidence gates. Equal
full dumps cannot prove unchanged status/protection registers.

## `FC0C`, idle reset and callback boundaries

`FC0C` at `0x01C0477C` (file `0x277C`) first checks `G+16`. If nonzero, it
invokes a function at `[0x01C07E00+28]` with pointer `0x01C04B74` and can return
early depending on its result. The loader contains a separate `FC16` upload,
validation and callback-install path at `0x01C04886–0x01C048E2`
(file `0x2886–0x28E2`). That arbitrary additional code is outside this route.
For a fresh exact entry, `G+16` lies in the initially cleared RAM range; with
only the scoped commands and no external RAM modification, it stays zero and
this optional callback branch is excluded. It is not a safe assumption for an
already-running unidentified loader.

The main `FC0C` path changes memory-mapped registers at `0x51000` and `0x11800`,
parses its four-byte big-endian argument as a delay, uses zero for values below
100, and calls delay code `0x01C0354E` (file `0x154E`). Thus ordinary host
argument `1` selects zero delay. It then branches to `0x01C04B6E`
(file `0x2B6E`), which calls reset routine `0x01C0292C` (file `0x092C`).
That routine sets bit `0x10` at `0x10000` and loops forever. This is a reset
request, not a return to the original application entry point; the chip-specific
reset effect and successful original-stock boot/USB return are not established
by disassembly. No NOR erase/program/status command appears on this scoped
main reset path.

The installed interrupt handler `0x01C02936` (file `0x0936`) acknowledges the
timer at `0x10500`, increments `G+28`, and every 100 ticks, while byte `G+48`
is zero, increases `G+44` by 100. If that accumulator is at least 3001 it calls
the same reset routine. The timer install at `0x01C0260E` (file `0x060E`)
registers this callback and configures timer registers. Without a separately
established peripheral-clock interpretation this is a tick-count condition,
not a new measured three-second timing claim. Every command entering the hook
resets `G+44`; `FC0A` is not uniquely needed to refresh this counter.

## Verdict and unresolved gates

The exact byte evidence closes the previous identification gap for these
command branches and the read/backend linkage. It **does not clear D2**.
The currently proposed `FC0A` keepalive/re-identification lifecycle has a
conditional status-write path and repeated NOR initialization. A strict
no-flash-write route cannot approve it based on its host IN direction.

Remaining gates include exact owner-chip status/mode semantics; all startup
configuration/ROM dependencies; physical selected-device transport and callback
behavior; admissible supported read mode/ranges and fresh-data error handling;
nonvolatile register/key effects; timeout/reset peripheral semantics; lawful
artifact/build provenance; and observed original-stock return. No replacement
CDB, another loader, new keepalive scheme or physical experiment is approved by
this report. Host and native test execution belongs to root integration; this
file-only subtask ran only its static extractor checks.
