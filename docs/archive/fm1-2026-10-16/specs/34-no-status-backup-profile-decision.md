> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Conditional backup profile: no NOR status writes and no FC0A

Date: **2026-10-15**, Europe/Berlin. FM1-033-D2 continuation on `init` after
`2e73103`. Three GPT 6.1 Sol high subagents audited startup, the read session
and an independent adversarial review. Root checked supporting bytes, integrated
the results and retained the existing activation boundary.

**Result: a conditional path exists in the unchanged exact loader.** It can
avoid the local NOR status-register writer, open NOR during startup, and serve
freshly requested `FD05` reads without `FC0A`. This is a static candidate with
specified preconditions, not an operational backup command or a physical
no-write qualification. The owner's actual pre-entry configuration has not
been read. No network, hardware access, flash operation, target execution,
firmware build/script or installation occurred in this task.

## Selected candidate

Prefer **observation and refusal**: inspect the existing configuration before
the jump, derive the exact startup/read state offline, and reject anything
outside a supported no-status-write profile. Do not force configuration RAM,
patch the loader, select a different entry point or use an automatic fallback.
The loader's decoded digest remains
`[digest omitted from public archive]`;
its encoded digest remains
`[digest omitted from public archive]`.

The concrete proposed sequence is:

1. Through a separately evidenced ROM-only read path, capture the loader and
   configuration inputs that will actually be consumed after the jump. The
   exact selector examines addresses `A=0x01C07E00`, `B=0x01C08E00`, then A/B
   again. Capture both distinct 34-byte headers and the selected 1 KiB. The
   simple default predicate needs `[A,A+1024)` plus `[B,B+34)`; another selected
   block requires its complete bytes too. Bind the phase, exact device/session,
   address coverage, transform geometry and stability through entry. A saved
   file, host source implementation or later state cannot supply that authority.
2. Decode/assess those inert bytes. For the narrow default witness both headers
   decode to a zero CRC field and are rejected; A is selected, and its first
   key length after the two independent 512-byte transforms is zero. All
   startup lookups terminate without a match. This yields configuration
   `0x00010300`: normalized mode 2. The selected profile rejects mode 4 rather
   than assuming its status bit was already set. A broader parser must bound
   every record, fixed 32-byte key-name copy, NUL search, value and numeric/pin
   parse to the capture; a matching SPI value needs all seven consumed bytes.
   Unknown, malformed or incomplete inputs cause refusal before the jump.
3. Enter the **unchanged** loader only once those prerequisites and ROM effects
   have been established. Argument `1` does not select the no-write mode. The
   observed configuration must survive the ROM call/interface handoff and
   remain the configuration actually used. In the audited local initializer,
   normalized mode 2 never enters the `06/01/31` status-write branch.
4. If separately implemented and qualified, use only the typed fixed RAM
   observation `FD07(0x01C09700,84)` to cross-check the expected fresh state:
   zero transform/extra-callback/idle-suppression flags, full cached JEDEC ID,
   cached capacity, active/saved configuration, derived read selector and
   continuous-read state. This copies known SRAM to disjoint scratch RAM in
   the resolved code; it does not reopen NOR. It is **outside today's allowlist**.
   Its response is a private historical observation, not physical identity or
   a way to approve startup retroactively.
5. Read through positive bounded `FD05` requests. The no-key mode-2 result is
   `word[G+76]=0x00020604`, so selector byte `G+77` is 6. The exact selector-6
   branch sends `3B` on every internal chunk; it does not set continuous state
   `G+80`. It uses busy-status reads `05`, not status writes. Do not confuse
   this with continuous selectors 3/8 or the same-SDK library's different table.
6. Make two new complete passes of independently established capacity, with
   exact raw coverage and BOT checks. One serialized session actor may insert
   finite extra FD05 checkpoint reads while a storage worker syncs, reopens
   and verifies two independent copies. Checkpoints use separate scratch and
   receipts, never fill either pass. For N capacity blocks, coverage is exactly
   `2N` reads and total raw reads are `2N+h`, with a predeclared finite H and
   `2 <= h <= H`. Qualify hook-to-hook timing and final continuity after storage.
   No INQUIRY heartbeat, FC0A, timer disable, reconnect or automatic retry.

[Startup report 32](32-loader-no-status-start-profile.md) establishes the local
mode predicate, configuration algorithm and exact inert default witness.
[Session report 33](33-loader-no-fc0a-read-session.md) supplies the actual read
selector table, fixed metadata copy, command ledger, timer arithmetic, finite
actor/storage schedule and terminal failure rules.

## Findings that affect acceptance

- The generic upstream uploader loads the blob and jumps; it does not inspect
  outside-blob configuration. Transporter's ROM memory helpers supply a
  source-level read/cipher candidate, not proof of the exact ROM implementation,
  address ownership or stable handoff. The external startup call at `0x01E00428`
  remains unresolved. Local absence of status-write commands cannot qualify
  unknown ROM, interface, IRQ or chip effects.
- The untouched loader's supported modes other than 4 skip the local status
  writer, but correct raw reading is a separate property. Unsupported selectors
  return without a data phase; ignored helper errors and equal stale buffers
  cannot establish fresh physical bytes. Exact chip, controller/addressing,
  capacity and supported mode still need independent evidence.
- A metadata-looking alternative is unsafe: `FC40` selects ioctl110 whose
  validation-failure route can reach NOR write/erase. Do not add it to the read
  route. `FC0B` does not reinitialize NOR, but its response omits the cached
  density byte and cannot replace full chip/capacity evidence.
- The preferred profile does not write configuration RAM. The `K512` witness
  in report 32 proves existence of a no-key input under the modeled transform;
  it is not a prepared hardware upload or authority to modify outside-blob RAM.
- Positive reads reset the loader's idle accumulator at hook entry, but a host
  submission is not evidence that entry occurred. One 512-byte FD05 request has
  two internal chunks and one reset. No hard wall-clock bound follows from the
  static timer arithmetic or a background thread. Slow storage/exhausted budget
  must fail acceptance without extending the session indefinitely.
- Original-stock return remains a separate lifecycle gate. The scoped `FC0C`
  path requests MMIO reset; actual reset/USB return and original stock operation
  have not been observed. No exit command was run or newly approved here.

## Offline implementation order within FM1-033-D2

1. **R1 — Bounded pre-entry context assessor.** Implement a pure strict parser
   over exact saved capture intervals, phase/source labels, loader hashes and
   independent transform vectors. Distinguish the narrow no-key predicate from
   supported well-formed SPI records. Refuse gaps, overlaps, wrong transforms,
   stale or changed contexts, short/NUL-less records and mode 4. Return a
   conditional assessment only; fake/saved bytes cannot mint a hardware permit.
2. **R2 — Typed fixed metadata validation.** Model only the exact 84-byte SRAM
   observation above, with private provenance and stable-field expectations.
   Counter values are not an atomic snapshot; cached ID/capacity do not prove
   physical identity or startup preservation. Preserve separate ROM/pre-entry
   and loader phases. No generic memory-address API or allowlist activation.
3. **R3 — Finite checkpoint/storage orchestration.** Extend the fake driver with
   distinct coverage/pass/checkpoint purposes, finite checkpoint and metadata
   budgets, immutable buffers and one device actor. Test slow storage,
   fsync/reopen/copy failures, changed data/state, cancellation, queue drift and
   exhausted time/count budgets. Keep both fresh complete passes mandatory.
4. **R4 — Native provenance and qualification.** Integrate actual macOS transport
   only after the exact ROM/start/chip/timing/access/return requirements have
   evidence. Never convert fake records or caller booleans to physical proof.
   The existing native factory stays a constant refusal until a separately
   reviewed activation change and qualified hardware task.

These are local work items within the existing ticket, not new external issues
or permission to access the device. R1–R3 are useful offline work; R4 retains
the physical evidence gate. Other OS release work is not a prerequisite for a
macOS-only backup.

## Verification and retained evidence

The existing vendor pi32v2 reader was reused in an ephemeral no-network,
read-only, unprivileged container with no device passthrough or Docker socket.
Tool digest and loader bytes match report 31; the existing image was selected
by its recorded immutable ID. Seven extra inert `ET_REL` carriers independently
start at additional startup/helper boundaries. Root byte-checked their **613**
printed rows against the exact decoded image; there were no unknown rows in
these particular ranges. Their listing SHA-256 is
`[digest omitted from public archive]`.
Full-listing/data coverage is not inferred from this selected count.

The startup audit checked **1,429** selected instruction rows and independent
starts, all eight mode values, the actual selector arithmetic and transform
layers. A separate reviewer checked **1,402** byte rows and independently
reproduced the exact loader decode, K512 hash and zero-config witness using
stdlib CRC rather than importing the startup helper. Session extraction checks
all 9,478 original listing byte rows, ten selected direct calls and the raw
read/metadata tables. All are static byte/data analysis, not target execution.

The canonical guarded host runner passed **306 tests in 203.047 s**, no skips.
Its ignored log `[local artifact location omitted]` has SHA-256
`[digest omitted from public archive]`.
No host/native runtime source, loader, dependency pin or factory changed.

Original scripts, listings, carriers and review receipts were copied unchanged
from `[local artifact location omitted]` to ignored
`[local artifact location omitted]`. The 41-file retained evidence map,
encoded as sorted compact JSON, has SHA-256
`[digest omitted from public archive]`.
Reports 29/30 were corrected for the repeated A/B selector and G+77 read-selector
detail. Real adapters, command expansions, no-write qualification and first
hardware backup remain unavailable.
