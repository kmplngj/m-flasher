> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Exact-loader read session without FC0A

Date: **2026-10-15**. FM1-033-D2, offline candidate design. Companion to
[startup profile 32](32-loader-no-status-start-profile.md) and
[call-path audit 31](31-loader-callpath-audit.md).

**A bounded `FD05` request is a concrete candidate to refresh this loader's
idle accumulator. `FC0A` is unnecessary for that effect and must stay excluded.**
This does not qualify the whole session: its safe start, physical chip/mode,
ROM delivery, fresh SPI data and timing have unmet prerequisites. No request,
allowlist, native factory, capture schema or physical capability changes here.
No hardware or network access, backend installation, vendor script/build,
loader execution/emulation or flash operation occurred.

## Exact evidence and scope

The unchanged decoded 24,064-byte image is SHA-256
`[digest omitted from public archive]`,
mapped at `0x01C02000`. Every address below is a **decoded file offset**;
add that base for its mapped address. The full listing is
`[local artifact location omitted]`, SHA-256
`[digest omitted from public archive]`.
Report 31 supplies the vendor object-reader and inert-carrier provenance.

Original stdlib-only extractor and receipts are under
`[local artifact location omitted]`. `extract_session.py` rehashes the
raw image, verifies all 9,478 listing byte rows against it, extracts 22 ranges,
checks ten direct-call displacements from raw instruction bytes and resolves
the read-selector, FC40–48 and ioctl100–110 tables from raw data. Its selected
ranges overlap 368 independently-started carrier rows with matching bytes.
Those carriers do not cover every new metadata range; the extraction is not a
new complete instruction-boundary or transitive-effects proof. `manifest.json`
records hashes, exact line ranges, table targets and counter arithmetic.
No raw vendor artifact is added to the tracked repository.

Names such as `G` and “read selector” are analyst descriptions. `G` is RAM at
`0x01C09700`. Parallel issue packets consume pre-packet register values.
The disassembler's symbolic annotation of odd immediates can round down;
raw/numeric values govern the table and ioctl interpretations.

## FD05 effects and acceptance limits

The command hook begins at `0x1E4A`; `0x1E70` stores zero to `[G+44]` before
prefix dispatch. `FD` at `0x2198` and subcommand `05` at `0x224C` lead to
`0x237C`. The handler takes a big-endian 32-bit raw address and big-endian
16-bit byte count, splits the count into at-most-256-byte internal chunks,
reads each into RAM `0x01C09600`, conditionally transforms it, and passes it to
the ROM-supplied callback `[G+36]`. A zero length issues no NOR read and is
excluded from the checkpoint design below. A 512-byte request has two internal
chunks but **only one hook-entry refresh**.

`0x14F6` resolves the registered read operation to `0x2E1A`. When `[G+80]` is
zero this wrapper calls busy poll `0x1A88`; it then calls the raw SPI reader
`0x2CB4`. Raw displacement checking confirms `0x2E2E → 0x1A88`, **not** the
separate `0x188E` FF escape routine. Busy polling sends `05`, reads status bit
zero, deasserts chip select and waits/repeats. It does not send `06`, `01` or
`31`. Its finite loop and diagnostic return do not provide a trustworthy
read-error signal: the wrapper ignores that result and proceeds, and FD05
ignores the read operation's return as well.

The raw reader loads the word `[G+76]` and extracts bits 8–15 at `0x2CC8`.
Its switch selector is consequently **byte `G+77`**, distinct from the low
configuration byte `G+76`, whose values 4/8/12 help initialization select an
SPI width/mode. The directly checked TBB table at `0x2CE4` is:

| Selector G+77 | Target | Bytes and state in the resolved read core |
|---|---|---|
| 0 | `0x2CF0` | `03`, address, data |
| 1 | `0x2D04` | `0B`, address, dummy byte read, data |
| 2 | `0x2D1C` | `BB`, three address bytes plus mode `00`, data |
| 3 | `0x2D36` | First chunk: `BB`, address plus mode `A0`; sets G+80=1. Later chunks omit BB while retaining that state |
| 6 | `0x2D68` | `3B`, address, dummy phase, data |
| 7 | `0x2D82` | `EB`, three address bytes plus mode `00`, dummy phase, data |
| 8 | `0x2D9E` | First chunk: `EB`, address plus mode `A0`; sets G+80=1. Later chunks omit EB while retaining that state |
| 11 | `0x2DDA` | `6B`, address, dummy phase, data |
| 4, 5, 9, 10; above 11 | `0x2E14` | Returns the requested count without a data phase |

The helper `0x2B7A` sends a fourth address byte only when `G+52` is nonzero.
The BB/EB paths visibly build a three-address-byte packet themselves; do not
infer four-byte support for them from that helper. Data/dummy transfers use
SPI MMIO and generated bus clocks. Dummy MOSI/controller details and the
chip's interpretation of command/address/mode sequences require qualification;
none is a newly approved arbitrary SPI request.

No status-register write, erase or page-program edge occurs in this resolved
FD05 NOR subgraph. There is no demonstrated capacity/address bound in the
handler. Require checked `address + length <= independently established
capacity`, supported addressing and selector, correct SPI/chip semantics and
fresh transfer evidence. An unsupported selector can return the requested
count with stale buffer contents; full BOT length, a success status and even
two equal dump hashes cannot exclude that failure by themselves.

The prefix calls command transformation guard `0x0F00`; fresh `G+32==0` skips
it. The payload helper `0x150A → 0x13F4` also returns without changing RAM when
that flag is zero. Only a qualified fresh exact entry, no F5/key/extra-code
commands and no outside state modification justify raw untransformed bytes.
The independently installed callback flag `G+16` must remain zero. FD05 does
not reopen NOR or call the FC0A protection-clear route.

The narrowest proposed profile chooses a proven **non-continuous** supported
selector and keeps `G+80==0`; the default no-key mode-2 result `0x00020604`
selects 6 (`3B`) when startup independently proves that configuration. This
path emits `3B` on both internal chunks of every FD05 and on all later
checkpoints; it leaves G+80 zero and repeats the busy-05 check for each chunk. This avoids depending on an
opcode-less continuation for checkpoints. Continuous selectors 3/8 have been
mapped, but are not silently covered: their A0 state, inter-command retention,
idle/reset behavior, and behavior on this exact chip need a separate supported
profile. FD05 never changes G+77 or clears continuous state.

## Metadata without FC0A

No complete safe replacement for FC0A's identification/configuration result is
established by the currently modeled INQUIRY/FD05 commands. INQUIRY belongs to
the ROM transport and does not demonstrably enter this command hook. It is not
a heartbeat or proof that NOR was opened safely. A presumed 1 MiB capacity,
stock firmware version or repeated dump content is not chip metadata.

There is a precisely bounded **candidate**, not a permitted new command:
FC0B, and its FD0B alias, enter `0x22E2 → 0x149A → ioctl100 → 0x30B0`.
Ioctl100 copies cached `[G+56]` to RAM and performs no SPI transaction or NOR
reinitialization. The getter then transforms/returns its response under the
same fresh G+32==0 requirement. It does not verify a freshly read chip ID.
More seriously, its explicit response stores copy cached-ID bytes 3, 2 and 1
to response bytes 2, 3 and 4; byte 0 (density) is **not copied**. Init's ID
sampler `0x19EE` constructs a 24-bit ID with density in that low byte.
Do not interpret the following prefilled response byte as the missing density.
This getter therefore cannot establish the full JEDEC ID or capacity. It is
outside the existing allowlist and is unnecessary for the FD05-only candidate.

The related metadata-looking paths are particularly unsuitable for substitution:

| Existing branch | Exact effect | Decision |
|---|---|---|
| FC40 `0x228C` | Calls ioctl110, target `0x30D8`. Reads/validates a header; failure at `0x339E` reaches NOR write `0x2E8A` and erase `0x2FFA` | Exclude; an apparent query has a destructive fallback |
| FC41 `0x28E4` | Nonzero argument takes ioctl `0x80045309` and can write the flash tail; zero selects a tail read | Exclude from candidate; adjacent write branch, no mode/capacity attestation |
| FC44 `0x2926` | Zero argument sets G+48=1 to suppress idle increments; nonzero reinstalls timer | Exclude; do not change watchdog policy to hold a session |
| FC45 `0x293E` | Builds response echo and returns; no demonstrated chip/mode/capacity field | No metadata authority |
| FC46 `0x2946` | Changes G+72/G+76, calls init `0x1ACA`, may issue reads and reconfigure controller | Exclude; repeats the startup status-write risk |
| Direct ioctl103 `0x30D2` | Copies cached capacity G+60 | A backend operation, not a demonstrated safe standalone host request; do not invent a CDB to reach it |

A more useful **fixed RAM metadata candidate** is the existing FD07 branch,
with source `0x01C09700` and length **84 bytes**, never a caller-selected
address/count. The prospective 16-byte CDB is
`FD 07 01 C0 97 00 00 54 FF FF FF FF FF FF FF FF`.
At `0x23DE–0x2420`, the loader parses those fields, calls copy routine
`0x351C`, and sends the 84-byte destination through `[G+36]`. With this exact
range, source `[0x01C09700,0x01C09754)` and destination
`[0x01C09600,0x01C09654)` are disjoint, aligned and below the 256-byte scratch
limit. The source exceeds the destination, selecting the forward aligned word
copy at `0x354A`; neither source nor destination is MMIO. This path has no NOR
backend, SPI operation, reinitialization, status writer or payload-transform
call. Prefix `0x0F00` must still be a no-op under fresh G+32==0. The same ROM
callback and asynchronous timing qualifications remain necessary.

The fixed response is little-endian RAM, not a vendor-defined metadata schema:

| Offset in 84-byte response | Meaning established by these exact instructions | Use/limit |
|---|---|---|
| 8, word | Open NOR device pointer | Private linkage evidence, not device identity |
| 16, word | Optional uploaded-callback flag | Must be zero for the fresh scoped route |
| 20/24, words | SPI transfer configuration/state | Compare only with the exact supported profile |
| 28, word | Timer T | Changes during sampling; not a session clock attestation |
| 32, byte | Transform-enabled flag | Require zero; post-start observation cannot undo an unsafe start |
| 36/40, words | ROM-supplied transfer callbacks | Private observed pointers, not independent ROM effects proof |
| 44, word | Idle A | Changes after hook reset and during copying; no atomic pair with T |
| 48, byte | Idle suppression flag | Require zero; never set it to hold the session |
| 52, byte | Four-byte-address state | Bind to supported capacity/addressing profile |
| 56, word | Cached 24-bit JEDEC ID | Includes density byte omitted by FC0B; reject 0/FFFFFF. Still cached startup data |
| 60, word | Cached computed byte capacity | Cross-check independently supported chip density/layout, not merely this formula |
| 68/72, words | Saved/active configuration | Bind to the pre-entry safe-start proof and expected supported result |
| 76, four bytes | Low width/config byte; selector at +77; transfer parameter at +78; flag at +79 | Default no-key mode-2 result is `04 06 02 00` when the startup profile proves it |
| 80, word | Continuous-read state | Require zero for the preferred selector-6 profile |

Only static profile fields can be compared for stability; IRQ-updated counters
are not an atomic snapshot. Byte 12's online flag can remain zero without FC0A;
do not require FC0A's flag-setting side effect to admit the already-open device.
This RAM copy does not prove the chip's actual status bits or fresh NOR data,
and a matching cached value does not attest physical identity. It is suitable
only as a candidate private, per-session observation with exact receipts;
saved responses are historical evidence after the session ends.

The prospective route is therefore: independently capture/analyze existing
pre-entry configuration, **refuse unless it proves a status-write-free supported
start**, enter the unchanged loader, then use this bounded RAM observation to
cross-check the expected result and full cached ID/capacity. Observing G only
after startup cannot establish that startup avoided writes. No forced RAM
configuration, generic memory diagnostic or changed payload is implied.

FD07 is currently outside the read allowlist. A later implementation must add
only this separately reviewed typed fixed-range observation and bind its exact
response to the safe-start proof. This document grants no command authority.
If prerequisites cannot be acquired without the unsafe bootstrap, this exact
loader cannot be activated for the owner's no-flash-write session.

## Idle counter and asynchronous behavior

The timer callback `0x0936` acknowledges MMIO `0x10500`, increments word
`T=[G+28]`, and calls `0x0130` with byte `0x74`. That routine conditionally
writes UART-like MMIO `0x12000` and waits on its status. It is no NOR writer,
but its blocking/peripheral behavior affects service timing. On every resulting
T divisible by 100, provided byte G+48 is zero, it performs `A=[G+44] += 100`.
An unsigned A at least 3001 requests MMIO reset through `0x092C`. No resolved
SPI/status/erase/program edge is present in this timer handler. Other ROM IRQs,
ROM callbacks, controller operation and actual reset effects are not proved
harmless by that limited observation.

A hook entry resets A, **not T**. With no T wrap, the next eligible increment
can be 1–100 IRQs away; reset follows the 31st increment, after 3001–3100
IRQs from the zero store. T is a 32-bit word; at wrap, the interval between
modulo-100 boundaries can be 96 rather than 100 IRQs. Do not extrapolate an
unlimited exact 3100-tick deadline, or call these ticks milliseconds. Timer
clock/rate and host scheduling are not established by the bytes alone.

A candidate engineering budget is at most **2900 serviced IRQs between actual
hook-entry zero stores**, including the previous read, BOT completion, queues,
transport latency and scheduling; that leaves arithmetic margin under the
ordinary unsigned-counter model, including one wrap. This number is a design
condition, not a measured device guarantee. A physical implementation needs a
qualified conservative conversion to a host deadline and proof of relevant
interrupt/interleaving behavior. Host submission timestamps, successful
INQUIRY and a software keepalive thread do not observe that zero store.

FD05's internal 256-byte loop has no further refresh. A large/slow read or
blocked transport can expire *during* a request. There is no safe concurrent
second BOT transaction to rescue it. Every single request needs its own bounded
completion and hook-to-hook budget. Timeout/ambiguous cancellation poisons the
session; no reset/re-entry/whole-read retry follows automatically.

## Concrete bounded candidate sequence

This is a reviewable design for a later implementation and qualification task,
not instructions to send hardware commands now. Its data and checkpoint reads
use the existing positive 512-byte FD05 shape; the optional fixed FD07 metadata
observations above require a separately typed future contract. Let C be a qualified capacity divisible by 512,
N=C/512; the modeled 1 MiB case has N=2048. Choose scratch range `[0,512)`
within that capacity, a maximum total checkpoint count H, and a maximum session
wall duration from qualified timing. Set H before entry; no user wait or disk
stall may extend it. Preflight the exact image/start/configuration evidence,
two independent destinations and available space before entering.

1. Establish one fresh selected-object session through the independently
   qualified start route. Bind the exact loader digest, source identity,
   physical chip/capacity/profile and entry evidence. NOR is already open;
   issue no FC0A. Reject an unidentified already-running loader. If the fixed
   FD07 observation is later qualified, capture it now and cross-check the
   expected static profile fields, full cached ID and capacity before raw reads.
2. Issue one ordinary FD05(0,512) checkpoint, close the complete BOT transaction
   with exact tag/status/residue/length checks, and retain an immutable scratch
   reference and its request/response receipt. This is a sanity/continuity
   reference, not full coverage or proof against a stale SPI buffer.
3. Capture pass 1 with exactly N new FD05 requests at addresses
   `0,512,...,C-512`. Each produces a new receipt and fills only its designated
   full-pass region. Checkpoint requests may be inserted only between complete
   transactions when needed by the qualified idle budget; they consume H,
   compare their bytes with the scratch reference and never satisfy pass data.
4. Capture pass 2 with a different pass identifier and exactly the same N
   **new** raw reads. Do not serve it from pass 1 or checkpoint bytes. Compare
   complete coverage and all bytes/hashes. Continue serial checkpoints when
   necessary between or after passes. A mismatch fails the capture.
5. Seal the two immutable complete buffers/files. A storage worker writes
   durable originals, performs fsync/directory durability as required, reopens
   and hashes, makes the independent second copy, and reopens/hashes that copy.
   Meanwhile the one session actor may issue finite checkpoints into separate
   bounded scratch memory; disk operations never own or block that actor's
   device queue. Checkpoints cannot overwrite either pass or its provenance.
6. After both copies are reopened and verified, close one final positive FD05
   checkpoint and compare it to the scratch reference. If qualified, repeat
   the fixed FD07 observation and compare stable profile fields with the first.
   Bind their receipts after the storage receipts in the acceptance manifest. Then seal all logs/receipts
   while the actor continues only within the remaining finite H/deadline budget.
   Session acceptance requires a live qualified continuity observation after
   the durable-copy work; a local deadline alone is insufficient. Normal return
   uses only a separately qualified finish procedure; FC0C is an MMIO reset
   request and original-stock boot still needs observation.

Exactly **2N coverage reads** remain necessary; total raw reads are
`2N + h`, `2 <= h <= H`, with every extra NOR read labeled checkpoint.
The optional two fixed FD07 observations are separate metadata requests and
count toward the finite total transaction/deadline budget, never raw coverage.
The minimum two checkpoints are the initial reference and final post-storage
checkpoint. Their repetitions neither invalidate two independently fresh
complete passes nor replace any missing byte. They also do not make two equal
stale/cached passes fresh: that still requires the qualified SPI/ROM/host path.
No status-register metadata is inferred from raw-array equality.

This schedule needs no simultaneous device access: only the device actor
serializes BOT, while a separate storage worker performs file IO. It does need
bounded OS scheduling despite sync/copy IO, a supported same-handle transport,
no reconnect or hidden reset and trustworthy fresh reads. Nonblocking design
alone cannot prove those bounds. Retain per-transaction purpose, pass/range,
sequence/tag, selected-object binding, clock/deadline evidence, completion,
byte hash and storage ordering; report checkpoints separately from coverage.

The existing injected replay allows exactly two finite full passes and has no
checkpoint purpose/budget or qualified hook timing. It will not accept this
schedule as-is. A later change must add a *typed finite* checkpoint contract,
all relevant cancellation/slow-storage/exhaustion fake cases and actual
native-to-storage provenance before replacing the closed activation gate.
Do not simply relax the request-count check or let arbitrary CDBs through.

At budget exhaustion, disk error, heartbeat error, changed checkpoint bytes,
identity/handle drift, unsupported selector, unclear freshness or deadline loss,
mark acceptance failed and preserve all available files/logs. Cancellation has
a predefined bounded drain/stop policy; never start endless background reads,
reopen a different object, clear status, disable the timer, reset or retry entry
as an implicit recovery. If storage cannot finish inside the finite qualified
window, the held-session acceptance cannot be delivered by this attempt.

## Decision boundary

The exact loader's **resolved FD05 branch is usable as a conditional finite
checkpoint design**, with an actual hook path and counter store; a new generic
heartbeat has not been guessed. The principal blockers remain safe fresh start,
independently established chip/mode/capacity, error/fresh-data evidence,
ROM/transport/IRQ timing and state continuity, plus observed normal stock return.
The candidate explains how to keep disk durability work from automatically
forcing FC0A or conflating checkpoints with complete captures. It leaves every
physical adapter, command expansion and flash-write flag closed.
