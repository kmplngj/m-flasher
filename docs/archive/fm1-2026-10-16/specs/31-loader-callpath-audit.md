> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Exact RAM-loader call-path audit and first-backup decision

Date: **2026-10-15**, Europe/Berlin. FM1-033-D2 on `init`, continuing from
`6447523`. The owner requested offline inspection of startup, reading and
return using GPT 6.1 Sol subagents. **No network or hardware access, target
execution/emulation, firmware build/script, installation or flash operation
occurred. The native factory, command allowlist and all write flags remain
unchanged and closed.**

## Decision

The exact loader now has recovered instruction paths, rather than only strings
and related library IR. [Startup report 29](29-loader-startup-callpaths.md)
resolves entry to NOR initialization. [Read/return report 30](30-loader-read-return-callpaths.md)
resolves `FD05`, `FC0A`, `FC0C` and the idle-reset path.

**The ordinary upstream lifecycle is not qualified under the owner's
no-flash-write constraint.** Startup has a configuration-dependent status-write
branch. Ordinary padded `FC0A`, previously proposed as a keepalive/reference
operation, repeats initialization and can deliberately change protection bits.
Those branches transmit actual command/data bytes through the SPI controller.
Their effects cannot be dismissed because the host subsequently requests reads.

This establishes conditional write attempts in these exact loader bytes. It
does not establish the owner's chip/status/configuration, whether those branches
would execute on that unit, or the volatility of its status registers. No claim
of an observed device write follows from static disassembly.

## Tool and artifact provenance

| Input / analyzer | Recorded identity |
|---|---|
| Encoded USB loader | 24,064 bytes; SHA-256 `[digest omitted from public archive]` |
| Per-512-byte decoded loader | 24,064 bytes; SHA-256 `[digest omitted from public archive]` |
| Existing vendor object dumper | `~/.jieli/jieli-linux-toolchains-20260730.1/common/bin/objdump`; 3,523,120 bytes; SHA-256 `[digest omitted from public archive]` |
| Tool-reported version | LLVM 4.0.1; registered targets `pi32`, `pi32v2`, `q32s` |
| Existing execution image | Cached `gcc:13`, local image ID `sha256:[digest omitted from public archive]`; no image download or build |
| Full analyst ELF carrier | 24,460 bytes; SHA-256 `[digest omitted from public archive]` |
| Full linear disassembly | SHA-256 `[digest omitted from public archive]` |
| Independently starting ranges | 26 carriers; listing SHA-256 `[digest omitted from public archive]` |

Earlier reports correctly found no suitable registered REA pi32v2/raw profile.
The new usable reader was found at the already installed JieLi toolchain path;
it was not installed or registered by this task. Homebrew LLVM 23.1.3's object
dumper does not register pi32v2 and was not used to decode these instructions.
No REA Evidence ID is claimed: these are local analyst/tool records.

The vendor tool accepts object files. Original Python stdlib code therefore
created **inert metadata carriers**, without compiling or linking firmware:
ELF32 little-endian, `ET_REL`, machine 241, entry zero, no program headers, and
three section headers (null, analyst `.text`, string table). Machine 241 was
cross-checked against an existing pi32v2 ELF header, not guessed from another
architecture. The full carrier's payload at file offset `0x100` is byte-identical
to the decoded input; its declared address is `0x01C02000`. The original raw
loader is unchanged and contains none of these ELF headers or section claims.

The actual reader invocation is `objdump -d -triple=pi32v2 <carrier>`.
It ran through a temporary `docker run --rm --pull=never --platform=linux/amd64`
container with `--network=none --read-only --cap-drop=ALL`, no-new-privileges,
UID/GID `501:20`, a 64-process limit, 1 GiB memory and two CPUs. Only the reader
file and inert inputs were bind-mounted, read-only; `/tmp` was disposable
tmpfs. No device passthrough, privileged mode, Docker socket or host source tree
was mounted. This runs an x86 host analysis tool; it does not emulate pi32v2
target execution. Existing image/toolchain contents were preserved. Containers
were automatically removed.

## Verification and limits

The root check rehashed both input and reader, validated each carrier's payload,
`ET_REL`/zero-entry/no-program-header fields, and compared all **9,478** printed
instruction-byte rows in the full listing against the input. A full linear
listing necessarily decodes tables/strings as instructions: its 821 unknown
rows are not a measure of reachable code coverage.

The 26 separately starting carriers address the particular risk of losing
instruction alignment after inline branch tables. Root checked **1,314** printed
rows against the same raw input. The read/return analyst compared their bytes,
boundaries and normalized instructions against the full listing with zero
differences. All **761** startup-selected records also match independent starts.
Two unknown rows in the independent output lie in inline read-switch data;
neither their byte validity nor agreement with the full listing turns them into
instructions or supplies an unknown-free transitive graph.

The table destinations, selected call displacements and command counterexample
were separately checked by original static extraction code. This is static
arithmetic and byte verification, not execution of a target simulator. One early
range extended into a truncated following instruction; its endpoint was corrected
before the final range hashes/checks above. No final call-path claim relies on
that truncation.

Vendor parallel issue packets (`#`) require pre-packet operand semantics.
Odd data-pointer immediates must be read from their raw/numeric values: symbolic
angle-bracket annotations can round down by one. Function labels in reports
29/30 are analyst descriptions; raw loader symbols were not recovered.
Unresolved ROM calls, callbacks, runtime configuration and chip semantics remain
explicit. The evidence is not a complete effects proof or a reproducible vendor
source build.

Canonical guarded host validation also passed **306 tests in 198.040 s**, with
its hardware guard installed before importing tests. The ignored log is
`[local artifact location omitted]`, SHA-256
`[digest omitted from public archive]`.
No host/native runtime source changed, so
this is regression evidence for the retained boundary, not a new hardware claim.

Original scripts, exact listings, carriers, range maps and verification receipts
are retained locally under ignored
`[local artifact location omitted]`, copied from
`[local artifact location omitted]`. The scripts include `tool/make_ranges.py`,
`startup/extract_startup.py`, `read-return/extract_read_return.py` and the
supporting IR inventory. Reports 23/27 retain the pinned distribution/ISA/library
sources. Raw vendor binaries and generated disassembly are not committed or
treated as newly licensed redistribution inputs.

## Recovered paths and what they establish

Offsets in this table are decoded **file offsets**; add `0x01C02000` for addresses.

| Stage | Recovered path / finding | Consequence |
|---|---|---|
| Start | `0 → 0x063A → device open 0x0556 → table-resolved 0x2E3C → init 0x1ACA` | Argument `1` does not bypass NOR initialization |
| Startup status branch | Mode 4 and SR2 bit 1 clear reaches `0x1BC8`: write-enable `06`, then `01` plus changed status; split `01/31` retry exists | Runtime mode/status and chip volatility must be established before qualifying this start |
| Runtime configuration | Configuration blocks begin at mapped `0x01C07E00`, exactly beyond the blob; no-key default mode is 2 | Default values do not prove actual runtime mode or make the write branch unreachable |
| Read | `FD05 → 0x237C → 0x14F6 → NOR table read 0x2E1A → 0x2CB4` | Resolved NOR read subgraph has no status-write/erase/page-program command; startup, IRQ and transport effects remain separate |
| Raw-data interpretation | Postprocess `0x150A → 0x13F4` returns unchanged when `G+32 == 0`; a separate `F5` route can enable RAM transformation | Fresh loader/session provenance and supported read mode are prerequisites; matching dump hashes alone cannot prove physical raw bytes |
| Read errors | Handler ignores read-helper result; unsupported modes can return without a data phase; no demonstrated chip-capacity check | Host bounds, chip/capacity qualification and fresh-data/error acceptance cannot rely on transfer success |
| Proposed `FC0A` keepalive | `0x262C → 0x2AE0 → reopen/init → ioctl 100 → ioctl 13(arg 0) → 0x319E` | A valid ID plus mismatching protection mask can attempt `06/01/31`; do not add this command to the read allowlist |
| Idle | Registered timer path increments/reset-checks an accumulator and can call `0x092C` | Timing is not established merely by the host's deadline; continued session ownership needs a qualified design |
| Return | Scoped fresh-session `FC0C → 0x277C → MMIO/delay → 0x2B6E → reset 0x092C` | This requests reset, not a verified jump/return to original stock; peripheral/ROM/reset and observed stock return remain gates |

The same-SDK library IR helped identify motifs, but its initialization/read
closure differs from this loader's machine code. Its lack of a defined
synchronous status-writer edge cannot override the concrete loader branches
above. The supporting IR closure also has unresolved external/IRQ effects.

## When a real backup becomes actionable

There is **no defensible date or promise of “the next hardware command” yet**.
The principal uncertainty is now a concrete write-capable lifecycle, rather than
missing instruction boundaries. For the requested USB-C/macOS route, the next
work is:

1. Establish a read-only start/session profile for this exact artifact, including
   actual configuration/status prerequisites, chip-specific persistent effects
   and ROM dependencies. The default mode and argument `1` do not supply that
   proof. If those prerequisites cannot be evidenced without an unsafe bootstrap,
   this lifecycle cannot be enabled under the constraint. A different minimal
   reader would require its own separately scoped implementation/audit; it is
   not an implicit fallback or an approved payload.
2. Design the held-session lifecycle without padded `FC0A`. The counter refresh
   occurs on entry to the hook, but which alternative requests reach that hook
   depends on the ROM transport. Do not infer a new safe heartbeat from the
   counter store alone. Cover slow disk I/O, sync/reopen, independent media,
   errors and cancellation without automatic reset/retry.
3. Finish actual native-to-storage provenance and exact macOS selected-object
   access/return qualification, using injected fakes first. Require the correct
   fresh transformation state, supported mode, chip capacity and complete fresh
   coverage; never relabel the existing fictional schemas as hardware proof.
4. In a subsequent explicitly scoped hardware-backup task, take two fresh equal
   complete raw reads, durably save and reopen/hash-verify two independent copies,
   then document return to the original stock firmware. No restore/program/erase
   test follows from that acceptance.

Linux/x86-64/Windows release work and the future writer are not prerequisites
for a macOS-only first backup. On the owner's side, a charged unit, stable data
cable and two suitable independent storage destinations will be needed for that
later session; no unplugging, mode entry or new device command is requested now.
