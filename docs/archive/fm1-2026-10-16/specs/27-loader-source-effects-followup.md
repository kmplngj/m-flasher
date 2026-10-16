> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Exact WL82 loader: manufacturer distribution and effects follow-up

Date: **2026-10-15**. Scope: FM1-033-D2, following
[report 23](23-loader-static-followup.md). **The exact encoded USB loader is
now independently matched to the manufacturer's public AC79 SDK. This improves
artifact provenance; it does not establish its source/build lineage or close
the startup/read/return effects audit. The physical no-flash-write gate remains
closed.**

Subsequent [exact-machine call-path audit 31](31-loader-callpath-audit.md)
resolves the loader's actual startup/read/return paths, including conditional
status-write attempts. The library IR in this report is supporting historical
evidence and must not replace those recovered machine-code effects.

This was file-only research using primary repository APIs/source and inert
binary bytes. No device enumeration, open, probe, command, vendor script/build,
target execution/emulation, backend installation or analysis-engine installation
occurred. Upstream firmware, PIO, submodule pins and license notices were not
changed. Downloaded evidence stays outside the repository in
`[local artifact location omitted]`; this report is the only repository
file written by this research subtask. Existing report 24 was read and preserved.

## Exact manufacturer-distributed bytes

The manufacturer's public repository is `Jieli-Tech/fw-AC79_AIoT_SDK` on Gitee.
Its branches API resolved the following immutable revisions on this date. The
`contents` API returned the loader as base64; an original Python stdlib routine
decoded the API envelope, hashed the file, and compared it byte-for-byte with
the previously retained jl-uboot-tool artifact. This is transport-envelope
decoding, separate from the MengLi loader decoding in report 23.

| Manufacturer SDK revision | Path | Bytes | SHA-256 |
|---|---|---:|---|
| `51712dd206536f165499f2313454b486b19a6175` (`release/AC79NN_SDK_V1.0.2`) | `cpu/wl82/tools/wl82loader.bin` | 24,064 | `[digest omitted from public archive]` |
| `9e7166fa9d8e3d0a8c671e848647c33a4e57f07f` (`release/AC79NN_SDK_V1.0.3`) | Same | 24,064 | Same |

Both are byte-identical to report 23's encoded input. Its independently
calculated Git blob SHA-1 is also
`3b47cffaf7bd87f27163ca7bf0c7deb0c783ac87`.
[Manufacturer V1.0.2 artifact](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/51712dd206536f165499f2313454b486b19a6175/cpu/wl82/tools/wl82loader.bin),
[manufacturer V1.0.3 artifact](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/9e7166fa9d8e3d0a8c671e848647c33a4e57f07f/cpu/wl82/tools/wl82loader.bin),
[V1.0.2 contents API](https://gitee.com/api/v5/repos/Jieli-Tech/fw-AC79_AIoT_SDK/contents/cpu/wl82/tools/wl82loader.bin?ref=51712dd206536f165499f2313454b486b19a6175),
[branch metadata](https://gitee.com/api/v5/repos/Jieli-Tech/fw-AC79_AIoT_SDK/branches).

The existing decoded fixture was rehashed as 24,064 bytes,
`[digest omitted from public archive]`.
No new decode algorithm or instruction analysis is claimed here.

Different SDK snapshots contain different loaders. The inspected GitHub mirror
`jeffreywugz/fw-AC79_AIoT_SDK` revision
`2f7775a826f32da017736a029a5dfc5bcc9b132a` supplies a 23,040-byte file with
SHA-256 `[digest omitted from public archive]`.
Manufacturer V1.1.0 revision `586a4fd8fb541777e4052274ba32c99f20ecedc8`
supplies 30,720 bytes, SHA-256
`[digest omitted from public archive]`;
V1.2.0 revision `5fa84a9b2f723a5e9337ad79dc2ad741fc98fe2d` supplies
31,232 bytes, SHA-256
`[digest omitted from public archive]`.
These are exclusion checks, not replacement candidates or an invitation to
execute another loader.
[Mirror artifact](https://github.com/jeffreywugz/fw-AC79_AIoT_SDK/blob/2f7775a826f32da017736a029a5dfc5bcc9b132a/cpu/wl82/tools/wl82loader.bin),
[manufacturer V1.1.0 artifact](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/586a4fd8fb541777e4052274ba32c99f20ecedc8/cpu/wl82/tools/wl82loader.bin),
[manufacturer V1.2.0 artifact](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/5fa84a9b2f723a5e9337ad79dc2ad741fc98fe2d/cpu/wl82/tools/wl82loader.bin).

Manufacturer distribution presence does **not** supply a loader source tree,
compiler/linker manifest, reproducible build, binary call graph, chip-specific
effects proof or a separate redistribution grant. The SDK's Apache-2.0 LICENSE
file was inspected, but its presence alone is not treated as resolution of the
loader blob's separate rights/provenance questions.
[Pinned SDK license](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/51712dd206536f165499f2313454b486b19a6175/LICENSE).

## Useful same-revision source locations

All sources in this section use manufacturer revision
`51712dd206536f165499f2313454b486b19a6175`, which actually distributes the exact
loader. They supply terminology and audit targets; no source-to-loader match
has been established.

| Source and verified lines | Observed source behavior | Consequence for this audit |
|---|---|---|
| `include_lib/driver/cpu/wl82/asm/sfc_norflash_api.h:7–14` | Separate declarations for NOR initialization, open, ordinary read, origin read, write, erase, ioctl and write protection | Distinguish the raw-read implementation from initialization/protection and recover their callees; declarations contain no effects proof |
| Same header, `19–31` | SPI configuration, protect resume/suspend, low-level SPI entry/exit and byte access, busy wait and OTP operations | Precise additional names to resolve if recovered loader code calls corresponding routines; names alone do not establish a loader call |
| `apps/common/system/flash_write_protect.c:11,45–76` | Optional application init hook opens NOR, gets its ID, selects chip-specific protection parameters and calls the protection API; its enabling macro defaults to zero | Demonstrates intended application API semantics. Its disabled init hook cannot prove that the separately built USB loader skips protection changes |
| `apps/common/example/FLASH配置和固件升级/FLASH自定义读取寄存器数据/main.c:23–43` | Example enters SPI code, waits for not-busy, sends `0x05`/`0x35` to read status registers and exits SPI code | Primary contextual reference for status-register read interpretation, not evidence for this loader's dispatch or register writes |

[NOR API header](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/51712dd206536f165499f2313454b486b19a6175/include_lib/driver/cpu/wl82/asm/sfc_norflash_api.h),
[optional protection application source](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/51712dd206536f165499f2313454b486b19a6175/apps/common/system/flash_write_protect.c),
[status-register example](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/51712dd206536f165499f2313454b486b19a6175/apps/common/example/FLASH%E9%85%8D%E7%BD%AE%E5%92%8C%E5%9B%BA%E4%BB%B6%E5%8D%87%E7%BA%A7/FLASH%E8%87%AA%E5%AE%9A%E4%B9%89%E8%AF%BB%E5%8F%96%E5%AF%84%E5%AD%98%E5%99%A8%E6%95%B0%E6%8D%AE/main.c).

The optional protection source also contains an erase/write test routine. It
was read as text only and is not an approved backup operation. Neither that
test nor its chip table identifies the owner's physical flash.

The manufacturer's complete recursive tree for this pin contains the loader
blob and NOR API header, but no source path named for the loader or an exposed
`sfc_norflash` implementation. This is bounded tree/path coverage, not a proof
that equivalent code cannot exist under another name or elsewhere.
[Inspected recursive tree](https://gitee.com/api/v5/repos/Jieli-Tech/fw-AC79_AIoT_SDK/git/trees/51712dd206536f165499f2313454b486b19a6175?recursive=1).

One concrete compiled-library lead was inspected without running its code:
`cpu/wl82/liba/cpu.a` is 2,532,492 bytes with SHA-256
`[digest omitted from public archive]`.
An original Python stdlib parser read its standard archive index. That index
maps `norflash_init`, `norflash_open`, `norflash_read`, `norflash_origin_read`,
`norflash_write_protect`, protect resume/suspend and the low-level NOR SPI
helpers to member `vm_sfc.c.o/`, whose header starts at archive offset
`0x1F4E48`. Its payload starts at `0x1F4E84`, is 77,800 bytes, SHA-256
`[digest omitted from public archive]`,
and begins `42 43 C0 DE` (LLVM bitcode magic). This is an indexed library
implementation candidate, not recovered C, machine-code disassembly, a build
input proven to produce this loader, or an instruction match with it.
[Pinned library](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK/blob/51712dd206536f165499f2313454b486b19a6175/cpu/wl82/liba/cpu.a).

## Existing LLVM reader: concrete library IR

A bounded check of the known Homebrew LLVM location found an already installed
`/opt/homebrew/opt/llvm/bin/llvm-dis`, version **23.1.3**. Reading the retained
`vm_sfc.c.o/` through that tool produced textual LLVM IR without compiling,
linking, running or emulating target code. The tool exited zero but warned
about an invalid debug-info attachment in `norflash_ioctl` and ignored debug
information. Consequently this report uses IR functions and generated-file
lines, not trusted source-line/debug provenance.

The IR preserves target triple `pi32v2`; no host retargeting was performed.
Its embedded source filename names `lib/driver/cpu/wl82/vm/vm_sfc.c` within a
vendor build workspace. That string is a source-location lead, not recovered
source, proof of the original build environment, or linkage to the USB loader.
The generated file is retained as `vm_sfc.ll` in the temporary evidence
directory: 114,637 bytes, SHA-256
`[digest omitted from public archive]`.
The file-only reader invocation was
`llvm-dis member_vm_sfc.c.o_ -o vm_sfc.ll`, using the absolute installed tool
path above. These observations derive from the pinned library above:

| Generated IR lines / function | Concrete library observation | Boundary |
|---|---|---|
| `1248–1263`, `_norflash_read` / `norflash_origin_read` | Origin read delegates through SPI entry, conditional four-byte-address entry, `__norflash_read`, address exit and SPI exit | This establishes a library read path, not that loader `FD05` calls it |
| `497–544`, four-byte-address helpers | Conditional branches send bytes `0xB7`/`0xE9` for capacities above 16 MiB and an access whose end exceeds the three-byte address range | Even a named read can include chip-mode commands; relevance to this loader/device remains unproven |
| `1274–1321`, `norflash_init` | Initialization performs volatile peripheral stores, interrupt registration, mutex creation and conditional flash-ID reading | No decoded loader startup or complete transitive effects proof follows from the library function |
| `1707–1721`, `norflash_read_sr` | Sends `0x05` and `0x35`, reads two bytes and combines them | Corroborates the same-revision status-read example |
| `1724–1820`, `norflash_write_sr` | Input flag `0x20000` selects enabling byte `0x50` versus `0x06`; `0x40000` selects separate register sequences versus a combined sequence; `0x10000` controls the second-register byte/sequence. Sequences send `0x01`, and on the separate second-register branch `0x31`, followed by busy checks | Precise candidate command/flag semantics for later matching. Volatile/nonvolatile meaning still needs the exact chip definitions and actual loader branch/argument |
| `1823–1894`, `flash_write_protect` | Reads status, compares selected bits, and can call `norflash_write_sr` in a bounded retry loop before restoring SPI state | Confirms a real write-capable library path; no proof that loader entry/read reaches it |
| `2177–2214`, protection resume/suspend and `norflash_write_protect` | Wrappers invoke `norflash_ioctl` with command `13` and saved, zero or caller-provided protection values | An API called by initialization or cleanup needs its own transitive audit; host read-only command selection cannot settle it |

This is newly recovered **library IR**, not a disassembly or call graph of the
24,064-byte USB loader. In particular it provides no `FD05`, `FC0A`, idle,
`FC0C` or loader-entry dispatcher linkage. Its commands and flags are useful
comparison targets, but substituting this library's behavior for an unexamined
loader routine would leave the central evidence gap unchanged.

## What remains unanswered about effects

A targeted printable-byte check of the existing decoded digest confirmed the
report-23 protection/status/quad diagnostics. Additional specific seeds are
`0x36FC` (P25QxxxH distinction failure), `0x371C` (P25QxxH QP wait timeout),
`0x5A5B` (reboot delay), `0x5A7C` (efuse error), and `0x5BBD`
([local artifact location omitted] values). These are decoded file offsets, not observed calls
or runtime addresses. The P25Q text does not identify an owner-device chip.
The efuse and [local artifact location omitted] diagnostics do not prove that either path writes
a nonvolatile key or that startup reaches it.

| Required path | Current evidence status |
|---|---|
| Entry at `0x01C02000`, argument `1`, initialization/constructors and device selection | Host parameters known; binary control flow and transitive effects unresolved |
| `FD05` dispatch and all raw-read callees | Host CDB known; matching loader dispatch, indirect calls and ROM dependencies unresolved |
| Status/protection/quad configuration | Exact diagnostic seeds plus same-SDK API context; branch reachability, commands/values and volatility unresolved |
| `FC0A`, idle/watchdog, errors and `FC0C` | No new matching binary lifecycle/control-flow proof; prior host observations remain bounded |
| Source/build identity | Exact manufacturer-distributed bytes newly established; matching source/build linkage unresolved |
| macOS owner-device no-flash-write qualification | No new physical evidence; remains false |

The REA skill was read. Report 23 already establishes the unavailable suitable
pi32v2/raw import and analyzer limitations; that same readiness check was not
repeated. The existing LLVM reader above supplied library IR, but no loader
instruction graph. A library's symbols, IR and a similar diagnostic vocabulary
cannot close any of these loader control-flow/effects gates without a proven
match.

## Next safe work

Continue the **exact-digest file-only audit**, using these manufacturer source
locations as leads. The concrete deliverable should be an instruction-boundary
and call-path ledger for entry/argument `1`, `FD05`, `FC0A`, idle/error paths and
`FC0C`; enumerate every unresolved instruction, indirect call, ROM callee and
status/protection/key operation. Use exact chip/register definitions to resolve
volatile versus nonvolatile effects. A suitable qualified existing analyzer or
the manufacturer's matching loader source/build inputs are still needed for
that work. The recovered library IR now provides concrete functions and
command branches for comparison, but still needs a demonstrated match to the
loader before its semantics can be transferred.

Do not activate a backup based on manufacturer-distributed bytes, the optional
application macro, a successful compile, transport direction or equal dump
hashes. The fresh match narrows provenance research and makes an exact source
request possible; it does not qualify execution on the only FM-1. Root
integration owns host tests and the separate platform/build evidence. This
documentation-only subtask ran no host tests and created no commit.
