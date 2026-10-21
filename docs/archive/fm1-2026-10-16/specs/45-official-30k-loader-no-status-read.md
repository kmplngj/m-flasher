> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Official 30 KiB helper: startup and raw reads without FC0A — R4-C

Date: **2026-10-16**. Static audit on `init`, using previously retained files.
No network, hardware access, target execution/emulation, firmware build/script,
backend installation or flash write occurred. GPT 6.1 Sol high agents examined
startup and read/return separately; a medium agent independently checked bytes,
tables, address arithmetic and the consolidated report. This extends
[report 42](42-targeted-wl82-rom-external-evidence.md), without changing the
selected loader, native factory, command allowlists or R1–R4 implementation.

## Decision and direct answer about hardware

**A conditional local raw-read path exists without FC09 or FC0A. The unchanged
30 KiB helper is not qualified for a trial start on the owner's only FM-1.**
Its initialization still contains status-register writes, and its configuration
and external startup interface must be established before executing it. A
post-start observation cannot prove that initialization avoided a write.

This audit answers the local control-flow question. It does not solve the
selected V15 reset-to-ROM consumer, bare-ROM FD07 restoration/RAM ownership,
macOS framing/access/return, external startup call or physical timing. These
are finite, operation-specific gaps, not a requirement to understand all ROM,
obtain a prior full backup, test restoration/writing or qualify every OS.
The existing backup-read permission needs no renewal; this task is offline.

The earliest useful physical milestone remains a **bounded pre-loader RAM
context capture**, after its selected entry/read/return prerequisites are
established, ending before FB08. Only a later qualified start and positive
flash read can lead to two fresh complete passes and independent durable copies.
No full backup was made and no activation is enabled here.

## Exact input and method

The manufacturer input is `cpu/wl82/tools/wl82loader.bin` from official SDK tag
`AC79NN_SDK_V1.2.1_2023-12-13`, commit
[`d179b4484759423312073f5fbb232501aa491047`](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/d179b4484759423312073f5fbb232501aa491047/cpu/wl82/tools/wl82loader.bin).
It is byte-identical to the retained Melodee asset at commit
[`0c21568a03626f1240229f7f368cfc2a7b09bafb`](https://github.com/keremimo/melodee/blob/0c21568a03626f1240229f7f368cfc2a7b09bafb/assets/recovery/wl82loader.bin).
These links identify already saved sources; no new retrieval was performed.

| Input/result | Bytes | SHA-256 |
| --- | ---: | --- |
| Encoded official/Melodee helper | 30,720 | `[digest omitted from public archive]` |
| Per-512-byte decoded helper | 30,720 | `[digest omitted from public archive]` |
| Inert analyst ELF carrier | 31,116 | `[digest omitted from public archive]` |
| Full static listing | 619,356 | `[digest omitted from public archive]` |

Independent bitwise CRC16-1021 and stdlib CRC implementations agree on decoding
with a reset every 512 bytes and on roundtrip. Continuous decoding produces a
different digest. The transform magic at **file 0x516C / absolute 0x01C0716C** is
`c3cfc0e8ced2b0aec4e3a3acd3f1c1d6`; it is unchanged from the old helper.

REA rejected the synthetic carrier's unsupported ELF architecture and opened
no native session. As in [report 31](31-loader-callpath-audit.md), an already
installed, digest-pinned x86 host `objdump` read inert ELF32 little-endian
ET_REL carriers: machine 241, no entry/program headers, `.text` at 0x01C02000.
The carrier supplies metadata, not original symbols or a compiled payload.
The existing cached image was pinned by ID, with network disabled, read-only
root/mounts, no devices, no capabilities and no new privileges; only the host
reader executed. No target instruction was run or emulated. Full and independent
range decodes exited zero with empty stderr. No toolchain or image was installed.

Every one of **12,048** printed full-listing byte rows matches the exact raw
image and covers all 30,720 bytes without gaps/overlap. Linear listing includes
866 unknown rows and inline data; this is not evidence of whole-file code
coverage. Selected startup/read rows exclude inline tables and unknowns.
Independent range starts corroborate instruction boundaries with the same
decoder; this does not independently prove ISA, peripheral or runtime semantics.
The 42 independent starts produce 2,356 matching byte/instruction rows, with
zero differences after removing symbolic annotations. They cover all 1,072
selected startup rows and 1,171 of 1,219 selected read/session rows. The remaining
48 rows are an optional mode-change branch `[2848,28DE)`, excluded from the
read candidate; they are byte-checked but have no separate restart. Four inline
tables are checked as raw data. Startup has 260 printed-displacement/target
arithmetic checks; the read audit separately checks 21 call targets from raw
instruction encoding. Detailed counts and retained digests are in the
evidence manifest (historical reference `evidence/45-official-30k-loader.json`; omitted from public archive).

All short hexadecimal locations below are **decoded file offsets**. Add
`0x01C02000` for absolute addresses. Function names are analyst labels.

## Startup: resolved local edges and conditional writes

| Offset/absolute region | Observed local behavior |
| --- | --- |
| `0000 → 033E` | Preserve r0–r2, clear BSS `[0x01C0B000,0x01C0B9D0)`, enter main; no new stack is established. |
| `033E`, `034C` | Incoming r0 is the external interface; incoming r1 is overwritten before use. Changing FB08's argument does not choose mode 2. |
| `02E8 → 031C → 4530 → 1A72` | First populated device record is `norflash`; raw device/ops tables resolve the indirect open to NOR initialization. |
| Main ioctl11 → `0338 → 4676 → 47B8` | Argument zero clears byte G+83; this startup call does not invoke ioctl13/protection. |
| `0472 → 0x01E00428` | Unconditional external startup call, before NOR open. Matching body/ABI/effects remain unknown. |
| `0562..057C` | External interface words supply send/receive at G+32/+36 and a writable hook slot at interface+8; installs hook `0x01C03DA6`. |
| `02BE..02E6`, IRQ `05D6` | Timer/vector registration occurs before NOR initialization and again after ioctl11. Runtime IRQ/ROM interaction is outside a synchronous graph. |

Here **G = 0x01C0B1B0**. NOR operations at absolute `0x01C09318` begin
`{0x01C06530,0x01C0650E,0x01C06676,0,0x01C069DC,0,0,0}`. The device table at
`0x01C09650` has two 12-byte records, the second empty. Stack, interface,
writable hook slot, BSS and configuration nonaliasing/ownership are prerequisites,
not established by these address calculations.

NOR initialization `1A72` sets up the controller, clears continuous state G+84,
then uses `AB` with dummy bytes/read, `9F` JEDEC and `4B` UID. Invalid JEDEC plus
all-zero/all-FF UID can enter a fallback that issues **WREN 06**, reads SR1 and
changes the selected pin bank before retrying. Capacity above 16 MiB can issue
**B7** for four-byte addressing. Chip-specific interpretation remains required;
absence of status programming does not mean absence of SPI state changes.

Only normalized **mode 4** reaches the conditional status writer:

1. At `1B60`, final JEDEC **CD6014** bypasses SR reads/writes.
2. Otherwise `05`/`35` read SR1/SR2. Initial SR2 bit 1 already set bypasses writes.
3. Otherwise `1BA2` emits WREN `06`, then `1BAE` emits `01` with
   `(SR1 & ~3),(SR2 | 2)`. Bounded retries include `06/01` and `06/31`.

The exact local no-01/31 predicate is therefore:

```text
normalized_mode != 4 OR final_JEDEC == 0xCD6014 OR (first_SR2 & 2) != 0
```

Default **mode 2** reaches read word **0x00020604**, selector byte **G+81 = 6**.
This supplies a local no-status-start candidate. The retained Melodee host
expects `856014`, which does not match `CD6014`; the owner's chip is unobserved.
Do not infer its ID or SR2 state from that host constant.

## Configuration: new locations and what must precede startup

The new upload covers `[0x01C02000,0x01C09800)`. It overwrites the old R1/R2
configuration and metadata locations. **A = 0x01C09800**, immediately after
the new image; **B = 0x01C0A800**. Neither fact proves available/owned RAM.
The candidate pointer table at G+88 contains **A,B,A,B**, not four successive
1 KiB blocks. Each candidate's 34-byte transformed header has a nonzero
little-endian CRC at +32 equal to CRC(bytes0..31). First valid wins; no valid
header leaves A selected. Main transforms the selected block's two 512-byte
halves in place, with independent resets.

The key parser `017C` begins at +34, accepts value lengths 1..32, copies
32 key bytes, then calls unbounded strlen/strcmp and advances over key/value.
It has no 1024-byte bound. A safe offline assessment must have every actual
key-copy byte, bounded NUL, value and cursor transition; malformed or partial
saved data cannot establish a safe mode.

| Key | Relevant call/behavior |
| --- | --- |
| SPI | `0386`: reads value bytes 0/2/4/6 regardless of returned length; at least seven initialized bytes needed. No match gives C=`0x00010300` and low read byte 4. |
| RESET | `0410`: pin/string parsing accesses terminated content and relative end positions; short copied values are not sufficient evidence. |
| EX_FLASH_IO | `0450`: starts with a zeroed 32-byte value buffer; no match stores 1. |
| LATCH_DLY_RELEASE | `046E`: reuses that buffer without clearing, then invokes external `0x01E00428`; a missing key does not prove a harmless ROM call. |

A narrow sufficient **existing-RAM observation** for defaults is: transformed
A/B header CRC fields are zero, and transformed A byte +34 is zero. All key
lookups then terminate immediately. An inert algebraic witness uses
`K = transform(512 zero bytes)` with SHA-256
`[digest omitted from public archive]`,
A=`K||K`, B-header=`K[:34]`. This is file arithmetic, **not a RAM-writing
recipe** or observation from the owner.

For that sufficient case the logical observation is **512+512+34 = 1,058
bytes at the new A/B addresses**, after qualifying the exact bare-ROM read,
ownership/restoration and selected entry/access/return. A different selected
configuration needs all bounded parser inputs. Observe before upload/jump;
do not force defaults, patch a loader or merely replace an old profile digest.

## FD05: concrete positive raw-read path without FC0A

Hook `1DA6` fills its response prefix, then clears idle accumulator G+40 at
`1DCC`. Raw prefix halfword table `1DFC` resolves FC→`2140`, FD→`21A2`;
FD05→`22B6`, FD07→`2318`. Both prefixes pass guard `0B9E`, unchanged when
**byte G+28 = 0**. FD05 parses big-endian address/count through `111E`.

```text
FD05 → 22B6 → 120E → NOR ops.read 450E → 43C4 → SPI receive
                         each chunk → 1224/1092 → external send G+32
```

The indirect read edge is resolved using the raw ops table and simultaneous
packet semantics at `1212`: the read pointer is loaded through the pre-packet
ops pointer while the address argument changes. Scratch is **0x01C0B000**.
A 512-byte host request is **two chunks of at most 256 bytes**, each using
`[0x01C0B000,0x01C0B100)` before sending, so it does not overlap G at B1B0.
Idle refresh occurs once per host command. Zero count does no NOR read;
the first checkpoint must have a positive count.

Raw reader `43C4` always busy-polls SR1 with **05**. Selector byte G+81 chooses:

| Selector | Offset | Read command |
| ---: | --- | --- |
| 0 | `43FE` | 03 |
| 1 | `4412` | 0B with dummy phase |
| 2 / 3 | `442A` / `443E` | BB, mode00 / modeA0 continuous |
| **6** | **`446A`** | **3B, address and dummy phase, non-continuous** |
| 7 / 8 | `4484` / `449A` | EB, mode00 / modeA0 continuous |
| 11 | `44CE` | 6B with dummy phase |
| 4,5,9,10 or >11 | `4508` | Returns requested count without a data phase |

The candidate restricts selector to 6, continuous state G+84 to zero, and
supported address state/chip semantics. Its resolved synchronous NOR closure
has no `06`, `01`, `31`, `11`, erase or program edge. Postprocess `1224→1092`
returns bytes unchanged when G+28 is zero. Fresh entry/no F5 mutation and
continued exact session identity are required.

There is no demonstrated capacity bound in FD05, and read/send returns are
ignored. Busy/DMA/delay/callback completion and external IRQs need qualification.
An unsupported selector can return stale bytes. Full BOT length and two equal
hashes alone cannot establish fresh raw SPI reads.

## Fixed loader metadata, FC0A and return hazards

**Loader FD07** `2318` uses memmove `4C28` into B000, then external send;
it does not invoke NOR or the payload postprocess. A hypothetical fixed
**88-byte** observation at **0x01C0B1B0** has disjoint aligned source/destination:
`[B1B0,B208) → [B000,B058)`. Candidate CDB is
`FD0701C0B1B00058FFFFFFFFFFFFFFFF`. It is unimplemented/unqualified and
does not establish the behavior of **bare-ROM FD07**.

| G offset | Field/check candidate |
| --- | --- |
| +8 / +12 | Opened handle / class-state (3 after successful NOR open) |
| +16 / +20 | SPI width / mutable transfer state; old uploaded-code flag does not apply |
| +24 / +28 | Non-atomic timer / transform byte (require 0) |
| +32 / +36 | External send/receive callbacks |
| +40 / +44 | Non-atomic idle accumulator / suppression byte (require 0) |
| +48 / +52 | Alternate-device/open state |
| +56 / +60 / +64 / +68 | Four-byte-address state / full cached 24-bit JEDEC / special-chip mode / computed capacity |
| +72 / +76 / +80 / +84 | Saved C / active C / read word / continuous state (require 0) |

Only established static fields can be compared; timers and transfer state are
not atomic identity. Post-start metadata cannot make an unsafe start safe.
FC0B/FD0B's response omits the low JEDEC density byte: it copies ID bytes
3,2,1 to response positions 2,3,4; random-prefilled position 5 is not density.
Use neither that short reply nor a guessed byte as full chip/capacity evidence.

**FC09** reaches key-like P33 transactions (`222C→16F4`); it is not a local
FD05/open prerequisite. Omitting it avoids adding an unnecessary operation,
without asserting that its broader hardware effects are qualified.

**FC0A** resolves to `2526`, checks literal `SD`, and ordinary FF-padded
nonmatch reaches `2A38`, restores saved configuration and reopens NOR. The
common valid-ID branch then invokes **ioctl13 with argument zero**
(`2AB0→2812→0330→4676→47BE`). Protection handler `47BE` reads 05/35,
compares mask **0x42FC**, and on mismatch can issue **06/01**, **06/31**, or
combined **06/01+two status bytes**. It cannot serve as a no-write heartbeat
or metadata command. Zero transform state does not remove this edge.

Timer IRQ `05D6` increments T=G+24; each T divisible by100, with suppression
zero, adds100 to A=G+40. A>=3001 requests reset `05CC` through MMIO10000 bit4.
Normal interval after an actual hook clear is 3001..3100 serviced IRQs;
32-bit timer wrap affects phase. A conservative <=2900 IRQ budget is arithmetic,
not milliseconds or measured timing. A blocked BOT request cannot be rescued
by concurrent keepalive. Ambiguous timeout/cancellation must poison the session;
no automatic reset/retry follows from a host deadline.

**FC0C** reaches `2AB6→158A→05CC`. Cleanup can issue **06 then 11 with changed
status** for cached IDs **856017/856018** when G+64 is nonzero. Expected
856014 in the retained host source excludes that one branch, but the owner's
chip remains unknown. FC0C is not a universally status-free exit. Preselected
deliberate cold power after completed/quiesced reads remains a stage-specific
return candidate requiring evidence, never an automatic error fallback.

## Why the retained host's backup switch is insufficient

Pinned [Melodee recovery source](https://github.com/keremimo/melodee/blob/0c21568a03626f1240229f7f368cfc2a7b09bafb/tools/fm1_recover.py)
was parsed/read, never imported or run. Its helper upload uses 60 FB06 blocks,
FB08 argument1, **FC09 then FC0A**, with 14 FF argument-padding bytes.
Its main flow also performs **bare-ROM FD07(0x01C7C000,512)** before upload.
The default read is one full 1 MiB device pass; fsync/reopening that saved file
is not a second fresh device pass or independent copy. `--write` guards host
erase/program calls, not loader startup, FC0A or bare-ROM effects.

Thus deleting FC0A is necessary for the selected candidate but insufficient
for qualification. Changing FB08's argument does not choose configuration.
No stock-firmware rollback, package key or FC09 is a local raw-read prerequisite;
the exact external interface/chip/configuration facts still need evidence.

## Bounded next work and completion boundary

| Gate | Result of this audit / next concrete evidence |
| --- | --- |
| C1 exact V15 local entry | Remains complete offline from report 40; no repeat implementation selected. |
| C2 selected ROM reset/read/return | Matching reset-consumer effects, bare-ROM FD07 success/failure restoration and ownership of the selected new A/B intervals remain open. Loader FD07 is separate. |
| C3 selected macOS access/framing | Continue report 41's exact quiet MIDI packetizer/receiver and USB lifecycle evidence; shipping packaging alone is insufficient. |
| C4 finite pre-loader context | After C2/C3, a new-address 1,058-byte default-case observation can end before FB08; no arbitrary RAM/ROM dump or forced configuration. |
| C5 helper startup/positive read | Local mode-2/selector-6/FD05 closure is established statically. Matching external 0x01E00428 ABI/effects, interface/stack/hook ownership, callbacks, chip semantics and live timing/return remain open before enabling execution. |
| Full backup acceptance | Positive fresh-read checkpoint, two fresh full passes, stable supported metadata, two reopened independent durable copies, continuity and observed original-stock return; no writes. |

If this helper is later selected, it needs a separately reviewed profile and
typed 88-byte validator bound to its own digest/layout; old R1/R2/R3/R4 engines
cannot be enabled by swapping a hash. This audit selects no alternative loader
automatically. Evidence acquisition is the next work, rather than another fake
model presented as physical progress.

Raw binaries/listings, original file-only audit scripts, per-range carriers and
agent receipts remain ignored under `[local artifact location omitted]`.
The committed sanitized manifest binds exact source/decoded/reader/listing
digests, selected ranges, dispatch results and false authority fields without
device data. Validation covers file hashes, independent decoding/arithmetic,
JSON, local links and diff scope. Product suites were not rerun for this
documentation-only audit. No hardware access, new backup or physical return is
claimed; all real activation and flash writing remain closed.
