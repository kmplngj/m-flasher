> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Exact WL82 USB loader: inert decode and static follow-up

Date: **2026-10-15**. Scope: FM1-033-D2, continuing the exact binary gap in
[report 20](20-stock-entry-loader-audit.md). **The encoded loader can now be
reproducibly decoded offline. Its decoded bytes contain references relevant to
flash protection and status-register handling. Startup, `FD05` and return
effects remain unqualified; the physical no-flash-write gate stays closed.**

This task read public source and local inert files and ran original Python
stdlib byte analysis. It did not import or execute vendor/firmware scripts,
execute or emulate the loader, build firmware, install an analysis engine or
hardware backend, or enumerate/open/probe/command hardware. No upstream firmware,
PIO file, submodule pin or license notice changed. The binary and downloaded
source snapshots remain outside this repository; only this report is added.

## Evidence identity and decoding

The input is exactly jl-uboot-tool's USB `wl82loader.bin` at commit
`adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a`, not stock `uboot.boot` or an OTA
loader. The source catalog specifies MengLi encoding, 512-byte blocks and
load address `0x01C02000`.
[Exact blob](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/data/loaderblobs/usb/wl82loader.bin),
[metadata](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/data/usb-loaders.yaml#L201-L208).

| Evidence ID | File-only observation |
|---|---|
| `LOCAL23-01` | Encoded input: 24,064 bytes (`0x5E00`), SHA-256 `[digest omitted from public archive]` |
| `LOCAL23-02` | Per-512-byte decoded output: 24,064 bytes, SHA-256 `[digest omitted from public archive]` |
| `LOCAL23-03` | Independently written bitwise CRC16 and Python `binascii.crc_hqx` produce byte-identical decoded output; repeating the same transform restores every encoded input byte |
| `LOCAL23-04` | Continuous, whole-file transform control has SHA-256 `[digest omitted from public archive]`; its bytes diverge after the first 512-byte block and lack the coherent loader diagnostics seen with the metadata's block geometry |
| `LOCAL23-05` | Decoded byte/constant observations below; no control-flow graph, execution observation or reachability claim |

These are local analyst evidence IDs, **not REA Evidence records**. Input bytes
were reused from `[local artifact location omitted]` after rechecking the
digest. Outputs and source snapshots are under
`[local artifact location omitted]`; `evidence.json` has SHA-256
`[digest omitted from public archive]`.
These temporary paths are inspection artifacts, not a durable fixture or a
redistribution grant. The pinned URLs and transform description are the durable
reproduction inputs.

The published `jl_crc_cipher` algorithm initializes a non-reflected CRC16 with
the low half of key `0xFFFFFFFF`, then feeds the high half as two little-endian
bytes. This makes the initial running CRC zero. For each byte it feeds the
cyclic GB2312 encoding of the source's eight-character magic, then XORs the
data byte with the CRC's low byte. Polynomial is `0x1021`; there is no final
XOR. The magic bytes are
`c3 cf c0 e8 ce d2 b0 ae c4 e3 a3 ac d3 f1 c1 d6`. Reset both CRC and magic
position at each 512-byte block. This reproduces the host's invocation geometry:
the helper is called separately for each block. The WL82 ROM-encoding quirk
explains why the uploader itself can send the encoded artifact unchanged; this
offline decode does not prescribe an extra on-device transform.
[Cipher source](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/cipher.py#L22-L32),
[CRC definition](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/crc.py#L8-L10),
[block loop and transform decision](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jluboottool.py#L680-L711),
[WL82 quirk](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/data/chips.yaml#L441-L448).

The two independent implementations establish that the local transform matches
its source specification. A reversible XOR roundtrip alone would not establish
the correct key or block size; the pinned metadata and coherent decoded byte
structure provide separate corroboration. No decoded bytes were read back from
an actual ROM session.

## New byte evidence and bounded address candidates

The decoded first 32 bytes are:

```text
d8 e8 07 00 c0 ff 00 96 c0 01 41 20 c2 ff d0 07
00 00 a2 a2 02 03 81 05 f2 5d d4 e8 07 00 c0 ea
```

The first byte is the host-selected entry candidate at `0x01C02000`; this does
not establish what its first instruction does. The image has no ELF header at
offset zero and supplies no ELF section/import/symbol tables. Loader-to-ROM
calls and memory-mapped peripherals must be recovered from instructions/data;
the lack of an import table cannot imply a lack of external effects. The last
460 decoded bytes, `[0x5C34,0x5E00)`, are zero. That observed suffix does not
define BSS, stacks, writable ranges or a safe execution boundary.

The following offsets refer to **decoded** bytes. Mapped addresses simply add
the catalog's load base; they are not observed runtime addresses.

| Decoded offset | Observed diagnostic content/category | Bounded follow-up |
|---|---|---|
| `0x360C` | `norflash` name | Device registration/dispatch candidate |
| `0x369C`, `0x36DC` | `norflash_write_protect` success/failure diagnostics | Recover protection routine and all callers |
| `0x36BC` | Both status-write registers mentioned | Resolve write commands, values and volatility |
| `0x374C`, `0x376C` | Quad-mode success/failure diagnostics | Resolve whether read initialization changes flash configuration |
| `0x59AC`, `0x59BE` | SPI/SFC initialization names | Trace argument-1 startup and peripheral initialization |
| `0x59B5`, `0x59EF` | `14:22:36`, `Apr 27 2022` | Embedded build-text leads; not an authenticated build date or source revision |
| `0x59D9` | `QUAD_MODE` | Recover mode selection and conditional reachability |
| `0x5A27` | `UIMMTAG-` | Investigate its data use without assuming a persistent write |
| `0x5AA9`, `0x5AC0` | Separate `write w_reg1` / `write w_reg2` diagnostics | Audit protection/status paths and chip-specific branches |
| `0x5AD7`–`0x5B20` | Status-register read/check diagnostics | Resolve device status/protection behavior |
| `0x5B5E` | `Hello wl82 Loader` identity text | Corroborates loader family; does not authenticate effects |

Some relevant strings have direct little-endian address constants in the
decoded image. An additional **candidate-only** scan matched the published
pi32v2 immediate-load encoding (`0xFFC0`–`0xFFCF`, little-endian halfword,
followed by the 32-bit immediate) on two-byte boundaries:

| Candidate instruction file offset | Six observed bytes | Constant target |
|---|---|---|
| `0x18C8` | `c0 ff 39 7b c0 01` | `0x01C07B39`, SPI mode/clock/port diagnostic |
| `0x1B6C` | `c0 ff d9 79 c0 01` | `0x01C079D9`, quad-mode diagnostic |
| `0x1B8E` | `cb ff d7 7a c0 01` | `0x01C07AD7`, status-register diagnostic |
| `0x31B6` | `c8 ff a9 7a c0 01` | `0x01C07AA9`, first status-write diagnostic |
| `0x31FC`, `0x341E` | `c0 ff 9c 56 c0 01` | `0x01C0569C`, protection-success diagnostic |
| `0x32CE` | `c0 ff bc 56 c0 01` | `0x01C056BC`, combined status-write diagnostic |
| `0x3356` | `c0 ff dc 56 c0 01` | `0x01C056DC`, protection-failure diagnostic |

The bytes and numeric equality are observations. Interpreting these matches as
instructions/references is an inference until instruction boundaries and code
reachability are established. A linear signature scan can match data or the
middle of an instruction. It gives precise seeds for a proper disassembler,
not a recovered call graph or proof that startup/`FD05` reaches a status write.
[Published immediate-load specification](https://github.com/kagaimiq/ghidra-jieli/blob/b5e60122b6cd3e6b615387035994b8bed0ea1a26/data/languages/pi32v2_ins_loadimm.sinc#L29-L38),
[little-endian/alignment definition](https://github.com/kagaimiq/ghidra-jieli/blob/b5e60122b6cd3e6b615387035994b8bed0ea1a26/data/languages/pi32v2.slaspec#L1-L2).

The diagnostics make a specific transitive audit necessary. They do **not**
prove that this loader writes flash on startup, that a status write is
nonvolatile, or that `FD05` is unsafe. Conversely, neither the host's IN
direction nor two equal full reads establishes that these paths are absent.

## Analysis-tool and provenance boundary

The prior report already records REA's unconfigured Ghidra provider; that
identical readiness call was not repeated. The connected tool catalog was
checked for a genuinely applicable target-free operation. Its firmware-region
tool requires Linux with caller-supplied Binwalk 3.1.x and only scans signatures.
Its binary-layout tool supports Linux x86-64 ELF64 and can use instruction
emulation for PLT derivation. This macOS session's raw JieLi pi32v2 artifact
fits neither supported profile, and target emulation is outside this task.
The native open tool advertises no pi32v2/raw-firmware import option; its
explicit headerless option is DOS COM. Reinterpreting the loader as DOS/x86
would produce misleading evidence. No such analysis was invoked, and there
is no new REA Evidence ID or disassembly claim.

The public ghidra-jieli revision used only for the candidate encoding definition
is `b5e60122b6cd3e6b615387035994b8bed0ea1a26`. Its author labels pi32v2 early
and lists missing group-7 instructions, flags/stacks, parallel execution and
repeat handling. It must not be treated as a complete effects oracle merely
because a subset of bytes decodes.
[Processor-module limitations](https://github.com/kagaimiq/ghidra-jieli/blob/b5e60122b6cd3e6b615387035994b8bed0ea1a26/README.md#L7-L28).

A bounded additional public search covered the exact loader's decoded date,
WL82 protection diagnostic, and USB-loader source terms. It found no matching
published disassembly or source-to-binary build linkage. The freshly retrieved
FM-1-RE recursive tree at `ec832f281ca165a7d5e0722985f3acb1465046d5` still
offers stock application/SPL/OTA analyses, not a proven analysis of this exact
USB loader. This records the inspected boundary, not universal nonexistence.
[Pinned FM-1-RE tree](https://github.com/AL-255/FM-1-RE/tree/ec832f281ca165a7d5e0722985f3acb1465046d5).

The 2022 text, the 2023 public blob addition and the unchanged 2025 repository
pin are separate facts. None supplies a vendor build manifest, matching source,
compiler/linker inputs, a reproducible build or redistribution authority. The
separate stock `uboot.boot` reconstruction remains evidence for that other
artifact only.

## Remaining loader gates

| Question | Result of this follow-up |
|---|---|
| Exact encoded identity | Rechecked |
| Offline decoding and decoded identity | Newly established with independent CRC check |
| Candidate flash/status/protection audit locations | Newly established byte offsets and inferred immediate-reference seeds |
| Entry/constructors, BSS/stacks and loader argument-1 initialization | Unresolved |
| SPI status/protection/quad-mode writes, volatile versus nonvolatile effects | Unresolved; now concrete diagnostic locations to trace |
| `FD05` dispatch, read implementation and all indirect/ROM callees | Unresolved |
| Interrupts, callbacks, errors, idle/watchdog and `FC0C`/normal return | Unresolved |
| Exact source/build lineage and blob redistribution rights | Unresolved |
| Physical no-flash-write route / owner-device qualification | **Closed / false** |

The next useful loader step is instruction-boundary/control-flow recovery for
this decoded digest with a suitable already available pi32v2 analyzer, followed
by callee and hardware-register interpretation from exact primary definitions.
Trace entry and `FD05` separately, including indirect tables, ROM calls and
every branch that can modify flash status/protection, nonvolatile keys or
persistent data. Pin the decoder/analyzer version and record undecoded ranges;
unresolved instructions or indirect calls remain open gates. A file-only audit
can proceed without any owner-device commands or target execution. Installing
an engine, executing a firmware script/build, or switching to a different
loader is not an implicit continuation of this task.

No host/native code changed in this research subtask. Root integration owns the
canonical guarded regression checks and any commit; no hardware, delivery or
normal-return acceptance is claimed here.
