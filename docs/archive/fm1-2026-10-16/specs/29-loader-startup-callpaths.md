> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Exact WL82 loader: startup call paths and configuration effects

Date: **2026-10-15**. Scope: FM1-033-D2, offline startup audit of the exact
decoded RAM loader from [report 23](23-loader-static-followup.md). **The loader
entry now has real instruction boundaries and a resolved path to flash
initialization. A configuration-dependent startup branch transmits status-write
commands before any `FD05` read. This closes the strings-only evidence gap;
it does not qualify a no-flash-write backup session.**

This subtask read local inert files and ran static byte extraction only. The
parent task supplied the existing vendor disassembler's output; its isolated,
offline invocation and synthetic carrier are recorded in the integration
report. No network, hardware enumeration/access, target execution/emulation,
firmware script/build, installation or flash operation occurred. This is the
only repository file written by this subtask; upstream, pins and licenses were
preserved. Host tests and integration remain the parent task's responsibility.

## Evidence identity and interpretation

All hexadecimal offsets below are **decoded file offsets**. Add `0x01C02000`
for their mapped addresses. In particular, `0x1ACA` means `0x01C03ACA`.
Function names below describe recovered behavior; they are analyst names,
not recovered vendor symbols.

| Evidence | Identity / scope |
|---|---|
| `START29-01` | Encoded input, 24,064 bytes; SHA-256 `[digest omitted from public archive]`, independently rechecked |
| `START29-02` | Decoded input, 24,064 bytes; SHA-256 `[digest omitted from public archive]`, independently rechecked |
| `START29-03` | Parent-produced `tool/wl82.analysis.dis`; SHA-256 `[digest omitted from public archive]`; vendor LLVM 4.0.1 pi32v2 decoder, analyst-created non-loadable ELF carrier with unchanged `.text` bytes, base `0x01C02000` |
| `START29-04` | `startup/startup-evidence.json`, SHA-256 `[digest omitted from public archive]`: 761 selected instruction records, each checked byte-for-byte against the exact decoded input, plus raw pointer-table words |
| `START29-05` | Original stdlib helper `startup/extract_startup.py`, SHA-256 `[digest omitted from public archive]`; selected listing SHA-256 `[digest omitted from public archive]` |
| `START29-06` | Parent-produced independently anchored raw-slice disassembly `tool/independent-ranges.dis`, SHA-256 `[digest omitted from public archive]`; range manifest SHA-256 `[digest omitted from public archive]`. All 761 selected records match in bytes and normalized instruction text; same decoder, separate boundary starts |

Temporary analysis artifacts are under
`[local artifact location omitted]`; they are not committed fixtures or a
redistribution grant. These are local analyst evidence IDs, not REA Evidence
records. The carrier has no original symbols, section provenance, entry point
or program headers; merely disassembling its whole `.text` does not turn data
into reachable instructions.

`START29-06` re-decodes finite raw slices beginning at the recovered entry and
callee boundaries, so state from a whole-file linear pass through data does not
determine their initial decode. The comparison excludes angle-bracket address
annotations, which can round odd data pointers down by one; exact pointer
values come from instruction bytes/numeric immediates. This is an anchoring
cross-check with one decoder, not independent validation of the ISA semantics.

The entry path is anchored at the host-selected offset zero. Direct destinations
below come from decoded branch/call instructions on that path; the one device
open dispatch is resolved through exact blob pointer words. The conditional
branches inside the reached flash initializer are code paths, not guesses from
nearby strings. Static reachability describes possible execution given branch
inputs; no particular device execution was observed.

The retained published SLEIGH subset remains useful as a cross-check for the
long `goto` at `0x001E` and full-width immediate loads. It is insufficient as a
sole decoder: it lacks entry's `d8 e8 07 00` multi-register push. Small-immediate
`41 20` gives `r1 = 0` in the vendor listing and the retained immediate-field
definition; treating its destination-register bit as immediate data would
incorrectly give 1. The vendor listing also marks parallel issue with `#`; the paired
operations must preserve packet semantics rather than be read as arbitrary
sequential assignments. Do not extend the earlier signature scan into a
qualified graph using that incomplete specification alone.

## Entry, RAM initialization and preserved argument

| Offset / bytes | Recovered startup behavior |
|---|---|
| `0x0000`, `d8 e8 07 00` | Push `r2-r0` to the existing stack; does not initialize `sp` |
| `0x0004`, `c0 ff 00 96 c0 01` | `r0 = 0x01C09600` |
| `0x000A`, `41 20`; `0x000C`, `c2 ff d0 07 00 00` | `r1 = 0`, `r2 = 2000` decimal (`0x7D0` bytes) |
| `0x0012–0x0018` | Shift count right by two; repeat word stores through `[r0++=4]`; loop back to `0x0014` while count remains |
| `0x001A`, `d4 e8 07 00` | Restore original `r2-r0` |
| `0x001E`, `c0 ea 0c 03` | Unconditional `goto 0x063A` |
| `0x063A–0x0640` | Save registers, reserve 36 stack bytes; preserve incoming `r1` in `r8` and incoming `r0` in `r11` |
| `0x0642–0x065C` | Zero five words at `0x01C09974` and thirteen at `0x01C09988`; link the two RAM structures |

The decoded clear sequence targets `[0x01C09600,0x01C09DD0)` in its intended
repeat semantics. Stack validity and RAM layout are incoming-environment
requirements; a zero suffix at the end of the file does not establish them.
The host's argument `1` is preserved rather than replaced by the temporary
zero used for clearing. In the covered initializer, the saved argument is
placed on the stack for a diagnostic call at `0x08B4`; it is not a guard that
skips the flash open/initialization at `0x08F4`.

## Recovered startup path ledger

| Anchored instruction edge | Callee / concrete action | Remaining boundary |
|---|---|---|
| `0x065E`, `0x066A` → `0x0084` | Select configuration block; initialize RAM state at `0x01C09700` | Consumes pre-existing RAM after the loader, outside these 24,064 bytes |
| `0x0666`, `0x0676` → `0x0024` | Transform two 512-byte configuration blocks using the local byte-transform routine | Blocks' actual runtime contents are not present in this artifact |
| `0x0684`, `0x0710`, `0x071C`, `0x0738`, `0x0776` → `0x044C` | Look up configuration keys and copy values into stack/RAM configuration | Data-dependent success/default branches |
| `0x078E` → `0x01E00428` | Direct call outside the loader range | ROM/external callee semantics unresolved |
| `0x0796`, `0x0830`, `0x0864` → `0x051C` | Convert recognized clock names to a clock value; returned 48 MHz for recognized values | Not a complete clock-domain qualification |
| `0x079A–0x087C` | Peripheral/clock/GPIO register stores; notably `0x10600`, `0x12000`, `0x12200`, `0x51020`, `0x50080`, `0x10010` | Register names/effects require exact hardware definitions |
| `0x0890–0x08AC` | Read incoming interface fields through saved `r11`; copy callbacks/context and install callback address `0x01C03E4A` through an incoming pointer | Interface pointer/ROM ownership and callback invocation are outside this startup slice |
| `0x08C8–0x08EC` | If mode field is 3, replace relevant configuration bits with the encoded mode-2 defaults | Does not eliminate the separately supported mode-4 path |
| `0x08F4` → `0x0556` | Find/open the `norflash` device through the exact table below | Open failure leads to diagnostic at `0x090C`; this is not a proved safe abort |
| `0x0918` → `0x05B8` → device operation `0x305A` | Operation code `11`, argument `0`; reached case at `0x3198` stores zero to byte `[0x01C09700+79]` | This specific startup operation is not protection operation `13` |
| `0x0924` → `0x060E` → `0x05C6` | Register callback `0x01C02936` for interrupt 5; configure timer registers at `0x10500` | Async behavior belongs to the lifecycle audit |
| `0x0928–0x092A` | Release frame and return through saved PC/registers | No proof that all prior callees/branches are effect-free |

`0x0084` seeds the candidate configuration area at `0x01C07E00` and builds
four pointer words. The subsequent [exact selector review 32](32-loader-no-status-start-profile.md)
corrects their interpretation: the sequence is `0x01C07E00`, `0x01C08E00`,
`0x01C07E00`, `0x01C08E00`, not four positions spaced by 1 KiB. The selector
checks 34-byte headers; the selected configuration is then transformed in two
512-byte calls. The loader file ends at `0x01C07E00`. These inputs are therefore
external RAM state, not recovered configuration bytes in the exact loader.
The no-key default installs `0x00010300` into the relevant configuration word,
whose mode bits select 2. A successful key lookup can set another mode; the
artifact does not prove which actual configuration the owner-device entry
would supply. Neither the default branch nor argument `1` establishes that
quad initialization is unreachable.

## Resolved device dispatch reaches flash initialization

The table scan at `0x055A–0x057A` covers exactly `[0x5C28,0x5C34)` with
12-byte records. Its single raw record is:

| Raw word offset | Value | Static interpretation |
|---|---|---|
| `0x5C28` | `0x01C0560C` | `norflash` name |
| `0x5C2C` | `0x01C07964` | Operations table at decoded `0x5964` |
| `0x5C30` | `0x01C0974C` | RAM device context |
| `0x5964` | `0x01C04E3C` | Open implementation at decoded `0x2E3C` |
| `0x5968` | `0x01C04E1A` | Read implementation at decoded `0x2E1A` |
| `0x596C` | `0x01C0505A` | Operation dispatcher at decoded `0x305A` |
| `0x5970`, `0x5978` | `0` | Null entries |
| `0x5974` | `0x01C05430` | Additional operation at decoded `0x3430`; not needed for the startup-11 resolution |

At `0x0582–0x058E`, the selected operations pointer supplies `r3`, and
`call r3` resolves statically to `0x2E3C`. This is a table-based resolution,
not evidence that every indirect call in the loader is resolved. It assumes
the loaded table retains these bytes. The concrete resulting path is:

```text
entry 0x0000 → goto 0x063A
  → call at 0x08F4 → device open 0x0556
    → indirect call at 0x058E, table-resolved target 0x2E3C
      → call at 0x2E40 → flash initializer 0x1ACA
        → call at 0x1ACE → peripheral/SPI setup 0x18AE
        → chip command paths below
```

`0x2E3C` calls the initializer unconditionally, sets an output context pointer
to `0x01C099BC`, returns zero, and does not propagate a failure result from
that initializer. The flash initializer issues `0xAB` plus three zero bytes
and a byte read (`0x1ADE–0x1B02`), reads a three-byte ID through `0x19EE`, and
reads a 16-byte UID through `0x1A36`. If the computed capacity exceeds
16 MiB, `0x1D46–0x1D5E` sends `0xB7` and sets a RAM address-mode flag.
These are actual binary command paths; matching names in the same-SDK library
are corroboration, not their source of reachability.

## Startup can transmit status-write commands

The initializer extracts `(configuration >> 15) & 7` at `0x1B5A–0x1B5C`.
Mode 4 branches to `0x1B6C`. It sends `0x05` and `0x35` to obtain the status
bytes, then tests status-register-2 bit 1 at `0x1BC4`.
The corresponding mapped locations are `0x01C03B5A`, `0x01C03B6C` and
`0x01C03BC4`; the initial write transaction is at `0x01C03BC8–0x01C03BEC`
and split retries at `0x01C03BF8–0x01C03C3A`.

| Condition / offsets | Exact recovered command sequence |
|---|---|
| Mode 4, SR2 bit 1 already set; `0x1BC4 → 0x1C88` | Skip the initial status rewrite and take the success diagnostic path |
| Mode 4, SR2 bit 1 clear; `0x1BC8–0x1BEC` | Clear low two bits of saved SR1; call `0x1A74`, which sends `0x06`; assert chip select, send `0x01`, send `SR1 & 0xFC`, send `SR2 | 0x02`, deassert chip select |
| `0x1BF0` and `0x1C40–0x1C84` | Busy/status polling through `0x1A88`; re-read `0x05`/`0x35`; test the readback bit |
| Retry path `0x1BF8–0x1C3A` | Send `0x06`, `0x01`, low status byte; busy wait; send `0x06`, `0x31`, high status byte; busy wait; repeat readback check within the bounded byte-counter loop |
| Exhausted retry `0x1CA0` | Emit quad failure diagnostic and continue through configuration processing; no no-write result follows |

The command transmitter is also concrete: `0x1882` sets `r1 = 0x11C00`, clears
bit 12 of `[r1]`, writes the command/data argument to `[r1+8]` at `0x188A`,
then branches to `0x1870`, which polls peripheral bit 15 and sets bit 14.
The chip-select helper `0x1854` updates bit 0 at `0x500C0` and clears bit 0
at `0x500C8`. Interpreting those as the SPI byte and chip-select operations is
supported by the repeated transaction structure and same-SDK API vocabulary;
pinning the complete hardware register definition is still required.

This establishes **write-capable startup behavior**, including byte values and
conditions. It does not assert that this branch ran on the FM-1 or that its
particular chip persists those status writes. No exact owner-device flash ID,
chip datasheet, status volatility or selected configuration was obtained in
this file-only task. A strict no-flash-write qualification cannot dismiss
`0x06 → 0x01/0x31` merely because the eventual host request would be a read.

## Covered paths and remaining gates

The entry prefix, main initializer, selected configuration paths, single device
open dispatch, actual flash initializer's mode selection/status transactions,
startup operation 11 and timer-registration edge were inspected. Exact byte
checks cover the selected 761 decoded records in `START29-04`; that number is
neither full loader coverage nor a proof that all transitive effects are known.
None of those selected records is printed as an unknown instruction by this
decoder. That bounded result does not classify unknown-looking words in
unreached tables/data or validate every instruction semantic and packet effect.

Remaining gates are concrete:

- External call `0x078E → 0x01E00428`, ROM/interface state and incoming stack
  semantics remain unresolved; there is no complete transitive startup proof.
- Configuration bytes at/after `0x01C07E00` are outside the artifact. The
  actual mode, clock, pin and flash settings cannot be inferred from defaults.
- The supported quad branch reaches status writes. Exact chip/register evidence
  must establish their persistent effects and acceptable policy; no such
  evidence was supplied by this task.
- Interrupt callback `0x0936`, installed callback `0x1E4A`, subsequent host
  dispatch, error/idle behavior and normal return require the companion
  lifecycle/read audit; they are not excluded by this startup report.
- Unexamined byte ranges, table data printed as instructions, and unresolved
  indirect/external callees cannot support a negative effects claim.
- Exact source/build lineage, redistribution rights and owner-device
  qualification remain separate gates from the newly recovered instructions.

The recovered path supersedes the earlier statement that startup control flow
was wholly unknown. It leaves the physical no-flash-write gate **closed** and
does not authorize hardware, loader execution, flash writes or a substitute
loader.
