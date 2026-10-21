> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Exact loader: conditional start without status-register writes

Date: **2026-10-15**. Offline FM1-033-D2 follow-up on `init`, starting at
`2e73103`. This report narrows the start conditions for the unchanged 24,064-byte
loader with decoded SHA-256
`[digest omitted from public archive]`.
It uses the reverse-engineer-anything skill and the retained static evidence
from [29](29-loader-startup-callpaths.md) and [31](31-loader-callpath-audit.md).
No network, hardware access/inventory, target execution/emulation, firmware
script/build or flash operation occurred. No loader, host implementation,
adapter, allowlist or write flag was changed or enabled.

## Result and preferred candidate

**The unchanged loader has a concrete local startup path that issues no
status-register-write request: select mode 2 before NOR initialization.**
Mode 3 is normalized to mode 2 before opening NOR. Mode 4 also skips the writer
if the first SR2 read returns bit 1 set, but this is a chip-state condition;
checking it after startup is too late to prevent an earlier write attempt.

The preferred future candidate is **observe existing configuration before
entry, statically assess it, and refuse unless a supported no-status-write
path is established**. This report derives an especially simple no-key/default
predicate that can be assessed from inert RAM bytes without executing the
target. It does not require changing loader bytes or writing configuration RAM.
The proposed policy rejects mode 4 unless its pre-entry status prerequisite is
independently established; the preferred profile uses modes other than 4 and
therefore does not depend on QE already being set.
The current host upload sources do not implement that predicate or attest that
configuration remains stable through the ROM call and startup. Therefore this
is a feasible conditional path, **not an enabled or qualified entry contract**.
The owner's actual RAM configuration, ROM implementation, flash ID and status
remain unobserved here.

## Exhaustive branch condition in the local initializer

Offsets below are decoded file offsets; add `0x01C02000` for mapped addresses.
Let `G = 0x01C09700`, `C = word[G+72]`, and `m = (C >> 15) & 7` at
`0x1B5A–0x1B5C`. The main initializer's `0x08C8–0x08EC` first maps mode 3 to
mode 2 and installs the corresponding defaults. The flash initializer's
complete mode split is:

| Input mode before main normalization | Mode reaching NOR init | Local startup status-write condition |
|---|---:|---|
| 0, 1, 2, 5, 6, 7 | Unchanged | No edge to the status writer |
| 3 | 2 | No edge to the status writer |
| 4 | 4 | Writer is skipped exactly when the first SR2 byte read at `0x1BA8` has bit 1 set |

This table classifies the synchronous local initializer. It does not qualify
every mode for reading: `FD05` dispatches on the separately derived selector
in byte `G+77`, not directly on `m`.
Modes 0/1 are not rejected merely because the no-key witness uses mode 2.
Their actual captured configuration and derived selector must be evaluated.
The independent read audit recognizes selectors 0/1/2/6/7/11 as non-continuous
read paths; 3/8 retain continuous-read state, while 4/5/9/10 can return without
a data phase. Mode 2 with the default low read byte 4 derives selector 6;
**selector 6 is non-continuous**, despite its numerical value. No mode, pin
assignment or divider is forcibly changed by the observe-and-refuse candidate.

At `0x1B60`, mode 2 branches to `0x1C8C`; at `0x1B68`, every mode other than
4 branches to `0x1CD0`. Thus `0x1B6C–0x1C88`, including the initial write and
all retries, has exactly one mode entry: 4. In that branch the SR2 test at
`0x1BC4` precedes the first write-enable call. A clear bit reaches `06` followed
by `01` and two status bytes; the retries can issue `06/01` and `06/31`.
Readback success after such a write cannot satisfy a no-write-request policy.

The invalid-ID retry at `0x1B1C–0x1B3A` clears configuration bits 18–19 and
repeats SPI setup/ID reading. It does not change mode bits 15–17 or introduce
a status-write edge. After the mode branch, `0x1C8C–0x1D76` modifies RAM read
configuration, computes capacity and optionally sends `B7` for capacity above
16 MiB; it does not call the status writer. `B7` changes address mode and
requires its own chip semantics; absence of status writes is narrower than
absence of every device state change.

## Whole local startup closure and its explicit boundary

The audited synchronous path remains
`0 → 0x063A → 0x0556 → table-resolved 0x2E3C → 0x1ACA`.
Additional local helpers were inspected rather than treating SPI setup as an
opaque call:

| Region / callees | Recovered effects relevant to the claim |
|---|---|
| `0x0024–0x0082`, `0x0084–0x0108`, `0x044C–0x04B2` | Configuration transformation/CRC, selector and key lookup; CRC MMIO at `0x13500`, RAM copies and diagnostic output |
| `0x010A–0x044A`, `0x04B4–0x0554`, `0x05A2–0x05B6`, `0x3470–0x35DC` | Number/pin/clock parsing, format/string/memory helpers and diagnostic UART output through `0x12000`; no SPI byte-transmitter edge |
| `0x18AE–0x19D4` | Configuration-dependent SPI/GPIO/SFC setup, then deassert chip select; no status-write transaction |
| `0x19EE–0x1A72` | `9F` ID and `4B` UID transactions with their address/dummy bytes and reads |
| `0x1ACA–0x1B5C` | SPI setup, `AB` plus three zero bytes/read, delay, ID/fallback and UID read; no status-write edge before the mode test |
| `0x0914–0x0918 → 0x05B8 → 0x305A → 0x3198 → 0x342A` | Startup ioctl 11 with argument zero writes RAM byte `G+79`; it does not dispatch protection ioctl 13 |
| `0x05C6–0x0638` | IRQ registration and timer MMIO; subsequent IRQ/session effects belong to [33](33-loader-no-fc0a-read-session.md) |

The formatter has an inline byte branch table at `0x032C–0x0334`; its entries
resolve within the formatter. Those bytes were excluded from the selected
instruction count. The `norflash` open/ioctl indirections resolve through the
unchanged tables already pinned in report 29. There are no further unresolved
local call-register edges in the selected synchronous closure.

There **is** an unavoidable external edge: `0x078E → 0x01E00428`, before NOR
open. This task has no implementation of that ROM callee. Incoming stack and
interface-pointer validity, the ROM call's effects, other interrupts/cores and
the exact hardware meaning of the configuration/CRC/MMIO stores remain
preconditions. Consequently the precise local predicate
`m != 4 OR (first_SR2 & 2) != 0` is not a transitive proof of zero physical
flash writes by the entire machine. A whole-start contract must establish that
the external environment neither writes status itself nor changes the inputs
or memory/control flow on which this local proof depends.

## Configuration selector correction and no-key predicate

Report 29 described four 1,024-byte positions. Rechecking the exact arithmetic
corrects that detail. At `0x0094–0x00BE`, the loader builds four **pointer
words** at `G+84..G+99`:

```text
A = 0x01C07E00; B = 0x01C08E00
G+84 = A; G+88 = B
delta = B-A = 0x1000
G+92 = B - ((delta / 4) << 2) = A; G+96 = B
candidate sequence: A, B, A, B
```

The loop advances its pointer-table cursor by four bytes, not the candidate
address by 1 KiB. Each candidate check copies **34 bytes**, applies `0x0024`,
reads a little-endian CRC field at decoded candidate `+32`, rejects a zero
field, and otherwise compares it with the CRC of the first 32 decoded bytes.
If all candidates fail, the cached selection remains `A`. The second endpoint
`B` therefore matters even though it lies beyond `[A,B)`. The selector's cached
pointer at `G+4` is zeroed by normal entry and becomes nonzero on first use.

The loader next transforms the selected 1,024-byte block in two independent
512-byte calls at `0x0666` and `0x0676`. Key lookup starts at selected `+34`.
A length byte outside `[1,32]` immediately returns no match; a zero byte is
the simple terminator. A sufficient observational predicate is therefore:

1. The entry clear and selector run normally; the initial pointer table/cache
   cannot be altered by an external actor.
2. The 34-byte transform of initial RAM at **both** `A` and `B` yields CRC field
   zero. Every candidate is rejected and `A` is selected.
3. Transforming initial RAM `[A,A+512)` and `[A+512,A+1024)` independently yields
   zero at selected `+34`. Every startup lookup terminates without a match.
4. These bytes, the loader tables, interface and stack remain stable and
   non-aliasing until consumed; the ROM call does not mutate the resulting
   configuration or introduce additional flash effects.

The resulting `SPI` no-match branch at `0x06E0–0x070A` installs
`C = 0x00010300` from initially zero RAM: mode 2, divider field 3, pin-bank
field 0; the low byte of `word[G+76]` is 4. Later SPI setup saves mode 2 at
`G+20`. NOR init changes the read word from `0x00000004` to `0x00020604`:
`0x1C8C → 0x1CC6 → 0x1CD0` installs selector **6** in byte `G+77`.
The selector is the bits 8–15 of the word at `G+76`. The session report maps
it to the non-continuous `3B` read path. The no-key path also supplies defaults
for `UTTX`, `UTBD`, `RESET` and `EX_FLASH_IO`; it does not turn the incoming
`r0` interface pointer into the configuration source. Argument `1` is preserved
for diagnostics and does not select or force this path.

## Exact inert witness and what the host sources support

For a deterministic offline data witness, define `T_n` as the retained
CRC/XOR transform, reset for each call: polynomial `0x1021`, no reflection,
CRC seed obtained from `FFFF` with initial `FFFF`, then the repeating 16-byte
magic `c3cfc0e8ced2b0aec4e3a3acd3f1c1d6`. The routine at `0x0024` feeds
two `FF` bytes after seeding CRC `FFFF`, then the same magic from blob offset
`0x37FC`. This interpretation depends on the CRC peripheral implementing the
retained polynomial/byte-feed semantics; the exact hardware definition is a
qualification requirement.

Let `K512 = T_512(512 zero bytes)`. Desired **pre-entry RAM** values
`A: K512 || K512` and `B: K512` satisfy the predicate above. Their transformed
headers are zero, their CRC fields are zero, and the selected plaintext block
is 1,024 zero bytes. Exact `K512` hex is retained in the evidence JSON;
SHA-256 is `[digest omitted from public archive]`.
The 1,024-byte repeated value has SHA-256
`[digest omitted from public archive]`.
This is a static data witness, not a patched loader or approved RAM write plan.

The retained `uboot-jltech_uboot.py` exposes ROM `FB06` memory write, `FD07`
memory read and `FB08` jump with explicit addresses. Its generic
`uboot-jluboottool.py` uploader sends only the loader file and jumps immediately;
it does not append, clear, inspect or attest the outside-blob configuration.
The retained Transporter `jl_ensure_loader()` likewise uploads the exact encoded
blob in 512-byte transfers and jumps with argument 1.

Transporter's separate `jl_rom_mem_write()` explicitly transforms caller bytes
per transfer and computes the transfer CRC over those wire bytes;
`jl_rom_mem_read()` checks the received length and reverses the same transform.
Those functions provide a **source-level candidate** to inspect existing RAM
before the jump. If their claimed ROM transform is correct, supplying `K512`
as desired RAM to the memory-write helper would algebraically produce a zero
wire block. Sending ordinary zero *RAM plaintext* through that helper would
instead leave zeros in RAM and would not supply the witness. The loader's
configuration transform is a second stage; the layers must not be conflated.

Neither source is the WL82 ROM implementation. It does not establish that
`[A,A+1024)` or `[B,B+512)` is free/owned upload RAM, that ROM accepts these
addresses with the claimed transform, that a readback describes the bytes
actually consumed after jump, or that stack/interface/ROM scratch do not
overlap them. The witness leaves the original loader interval
`[0x01C02000,A)` byte-identical, but outside-blob RAM writes would still require
their own exact memory-ownership proof. They are a secondary candidate only;
the preferred route reads existing configuration and refuses if its predicate
cannot be established.

For pre-entry attestation the minimal logical evidence is the loaded exact
blob, the existing configuration block at `A` (1,024 bytes) and the header at
`B` (34 bytes), with ROM read/cipher semantics and read-to-entry stability
established independently. If selection can choose another block, capture that
block too and evaluate its actual bounded key records. A host may refuse on
anything outside the simple predicate rather than claim arbitrary malformed
configuration parsing is safe. Fresh post-start state or a successful `FD05`
cannot retroactively attest this prerequisite. No such hardware read or
attestation was performed here.

Any future broader offline predictor must be stricter than the loader's
unbounded lookup. It must account for the four candidate headers in order,
capture the selected 1,024 bytes, reset the transform separately at each
512-byte half, and implement the lookup's exact first-match/termination rules.
Each key-name read (the loader copies 32 bytes), its NUL search, value extent
and next cursor must remain inside captured data. A matching `SPI` value must
supply all seven bytes used at offsets 0/2/4/6; a shorter successful value
leaves some input stack bytes uninitialized. Other successful values that feed
string/pin parsers need evidenced termination and complete parser inputs too.
Malformed, truncated, nonterminating or incompletely captured records are a
refusal condition, not an invitation to predict from a partial `SPI` match.
The narrow zero-length-first-record predicate avoids all such record parsing.

## Evidence and remaining decision

Original static script and receipts are under
`[local artifact location omitted]`: `audit_start.py`,
`startup-profile-evidence.json`, and `startup-closure-selected.dis`.
Their SHA-256 values are respectively
`[digest omitted from public archive]`,
`[digest omitted from public archive]`,
and `[digest omitted from public archive]`.
The script byte-checks every selected row against the exact blob, computes
direct call destinations using each instruction's actual byte length, excludes
the formatter's inline table, and checks its original bitwise CRC against
Python's `binascii.crc_hqx`. Applying the transform to each 512-byte encoded
loader block reproduces the exact pinned decoded artifact. This is static byte
arithmetic, not target execution or emulation. All **1,429 selected instruction
rows** byte-match the exact blob and agree with retained independent starts,
including the seven additional helper ranges in `../tool/extra-ranges.dis`.
The table/data exclusion and selected ranges are recorded in the script;
this is a bounded closure review, not full-file instruction coverage. An
independent reviewer reproduced the `K512` hash, double-transform zeros and
exact encoded-to-decoded loader with stdlib CRC; receipt
`../review-k512-crosscheck.json`. Same-decoder agreement does not
independently prove ISA or peripheral semantics.

The next acceptance question is concrete: **can the exact pre-entry ROM route
attest stable configuration satisfying the no-status predicate, and can the
external ROM call/interface/IRQ environment be shown to preserve it without
status writes?** The supplied source supports an inspection candidate, while
the missing ROM implementation, RAM ownership/stability and actual device
evidence prevent making it enforceable now. A mode-2 no-key path exists; the
owner's unit has not been shown to be on it. The native factory and real
adapters remain closed, and the required session excludes `FC0A` as detailed
in the companion report.
