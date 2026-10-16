> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Stock entry and RAM-loader audit — FM1-033-D2

Date: **2026-10-15**. Scope: the exact stock-to-ROM candidate, volatile
`wl82loader.bin` upload/startup, raw-read commands and return lifecycle. This
extends [report 16](16-stock-usbc-backup.md) and the D2 gates in
[report 19](19-native-read-adapters.md); it does not activate hardware access.

**Result: USB-C MIDI can request ROM entry on the stock V15 route reported by
Transporter's author. It is a credible entry candidate, not a qualified route
for an unknown stock version or this owner's device. The complete no-flash-write
route remains unproven because the exact RAM loader's startup and transitive
effects have not been audited.** A native read adapter must not turn those
unknowns into an approved physical session.

Only local policy/specifications, public source text, published disassembly and
one downloaded loader file for digest verification were inspected. No device
inventory, enumeration, open, probe, command, firmware build/script execution,
backend installation or loader execution occurred. The downloaded blob stayed
outside the repository. No upstream source, PIO, submodule pin or notice changed.

## Refreshed source identities

GitHub's public commit API resolved these repository HEADs on the audit date;
they match report 16. Links below use immutable commits, not moving branches.

| Repository | Current HEAD | Commit date UTC | Role |
|---|---|---|---|
| [FM-1-transporter](https://github.com/kurogedelic/FM-1-transporter/commit/a632d923203170e3a08565f4056dbb3fd3d65ca5) | `a632d923203170e3a08565f4056dbb3fd3d65ca5` | 2026-10-01 | Exact entry packet, RP2040 host observations, upload/read implementation |
| [jl-uboot-tool](https://github.com/kagaimiq/jl-uboot-tool/commit/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a) | `adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a` | 2025-03-16 | ROM/LoaderV2 framing, WL82 metadata, loader blob |
| [FM-1-RE](https://github.com/AL-255/FM-1-RE/commit/ec832f281ca165a7d5e0722985f3acb1465046d5) | `ec832f281ca165a7d5e0722985f3acb1465046d5` | 2026-09-08 | Published stock V13 receiver disassembly and OTA analysis |
| [MvaveFM1Unbricker](https://github.com/Quixotic7/MvaveFM1Unbricker/commit/8b4992ff684158ed1b27299792eedcfd0f9617ea) | `8b4992ff684158ed1b27299792eedcfd0f9617ea` | 2026-10-14 | Independently pinned blob and direct-computer upload/capture host code |

## Exact entry and version limits

The candidate logical MIDI message is **`F0 22 24 35 7D F7`**. Transporter sends
one eight-byte USB-MIDI transfer containing **`04 F0 22 24 07 35 7D F7`**:
cable 0/CIN 4 start plus cable 0/CIN 7 end. Its comment describes the stock V15
receiver comparing this transfer and calling mask ROM without returning. Its
implementation may repeat the send up to three times with 20 ms sleeps. That
retry behavior is not a recommendation for this project's single-shot,
fail-closed entry boundary.
[Exact implementation](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/src/pio_host.c#L99-L119).

The author's experiment reports stock V15 disconnecting 21 ms after the packet
and ROM appearing roughly one second later. The sender was an RP2040 USB host,
with its own preceding electrical/pulse lifecycle. It does not measure a direct
macOS/Linux MIDI API, its packet coalescing, or the owner's stock version.
[Experiment](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/docs/DEVLOG.md#L128-L150).

The `mounted_is_v15` guard checks only descriptor VID/PID `4C4A:C755`. It does
not authenticate firmware bytes, version, board or state. Consequently the
variable's name is not evidence of an exact V15 identity; these shared IDs cannot
be copied as this project's entry authorization gate.
[Constants](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/src/pio_host.c#L20-L22),
[comparison](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/src/pio_host.c#L281-L298).

There is a useful independent static distinction in the published **V13**
receiver at `0x02006384`. The assembly checks receive length exactly 8, compares
the first six bytes against a RAM constant at `0x01C07E2C`, requires final byte
`F7`, and then branches on byte 6:

| Trailer | Published V13 handler effect |
|---|---|
| `7D F7` | Call the target annotated `0xFFC02532`, then loop forever if it returns; see address correction below |
| `7F F7` | Post task message 148 through `0x02059A68`, then continue parsing |
| Other trailer or wrong transfer length/magic | Does not take either special branch |

**Address correction, 2026-10-16:** `0xFFC02532` is a file-relative tool
annotation. With application VMA `0x02000120`, it resolves to RAM
`0x01C02652`, not an absolute immutable-ROM entry. [Report 40](40-v15-entry-rom-backup-audit.md)
independently checks the manufacturer V15 package: its corresponding call
lands at RAM `0x01C026A4`, writes the `usb_update_mode` mailbox and requests a
P33 reset. The reset's ROM aftermath remains unqualified. Earlier comments
about a direct mask-ROM call must not be used as implementation evidence.

No explicit flash erase/program call appears in that small pre-ROM branch.
This is a bounded statement about the published instructions, not the callee.
The effective six-byte RAM constant and its initialization were not independently
bound to V15 or to a supplied stock image here. FM-1-RE labels its narrative
loader-entry interpretation medium confidence and identifies the top-level
analysis as V13. Do not infer complete V13/V14/V15 equivalence from similar
symbols or from that narrative.
[Published receiver instructions, addresses 0x02006384–0x020063CA](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/analysis/shards/shard_02005ce4_020081a4.txt#L571-L594),
[version scope](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/firmware-images/README.md),
[confidence annotation](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/05-midi.md#L88-L99).

**OTA is a different route and is rejected.** The ordinary updater uses
`F0 22 24 35 7F F7`; its first verification stage writes an update boot record
and resets into the OTA loader. The second stage performs the update. A
file-request/read-shaped exchange with the host is not a raw flash read, and
changing `7D` to adjacent `7F` is not an entry fallback.
[OTA session and persistent state](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/11-ota-protocol.md#L29-L49).

## ROM upload and jump: what the host code establishes

The generic protocol distinguishes on-chip **UBOOT1.00** from the downloaded
**Loader** variant. ROM memory access does not itself establish arbitrary raw
flash access. `FD07` must not produce an accepted full-NOR backup. The protocol
also warns that some ROM memory reads encrypt memory in place before sending
and decrypt it afterward; arbitrary RAM/peripheral reads are not inherently
side-effect-free. Unbricker's extra retained-RAM diagnostic is outside the
minimal proposed capture route.
[ROM versus loader and memory-read warning](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/usb-loader-v2.md#protocol-variationsimplementations),
[retained-RAM host path](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L849-L862).

WL82 metadata selects load address `0x01C02000`, MengLi encoding and 512-byte
blocks. Chip metadata says the WL82 ROM already transforms memory transfers;
therefore the inspected host path sends the already encoded blob without a
second host transform. A reviewed exact upload would consist of **47** RAM
writes spanning `[0x01C02000,0x01C07E00)`, followed by one jump to `0x01C02000`
with argument **`0x0001`** for SPI NOR. These are RAM writes and execution,
not passive inspection.
[Loader metadata](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/data/usb-loaders.yaml#L201-L208),
[WL82 ROM quirk](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/data/chips.yaml#L441-L448),
[bounded upload implementation](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L816-L846).

The host constructs 16-byte CDBs with trailing `FF` padding. `FB06` carries a
big-endian address and length, zero byte, then little-endian CRC16 of the
transferred bytes; `FB08` carries the big-endian entry address and argument.
The protocol says ROM sends the jump response after the invoked code returns,
which permits a loader to install a command hook first. Successful transport
completion therefore still does not describe all code run during startup.
[CDB implementation](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/uboot.py#L100-L134),
[ROM commands](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/uboot.py#L238-L268),
[jump semantics](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/usb-loader-v2.md#jump-to-memory).

## Exact loader provenance and unresolved binary effects

The current pinned public `data/loaderblobs/usb/wl82loader.bin` was downloaded
as inert data and independently hashed:

| Property | Verified value |
|---|---|
| Size | 24,064 bytes |
| SHA-256 | `[digest omitted from public archive]` |
| Git blob SHA-1 | `3b47cffaf7bd87f27163ca7bf0c7deb0c783ac87` |
| Initial repository addition | `64670123a1ec55d1cdc89a482b235d98af8a29b7`, 2023-02-23, “more loaders” |
| Relocation into `data/` | `c32b28e7a04f873f526ba929cb1bae9f05442fe9`, byte-identical rename |

[Exact blob](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/data/loaderblobs/usb/wl82loader.bin),
[initial addition](https://github.com/kagaimiq/jl-uboot-tool/commit/64670123a1ec55d1cdc89a482b235d98af8a29b7),
[move](https://github.com/kagaimiq/jl-uboot-tool/commit/c32b28e7a04f873f526ba929cb1bae9f05442fe9).

This corroborates Unbricker's pinned digest, not a source-to-binary build.
The reviewed jl-uboot-tool tree supplies host Python, documentation and blobs;
no matching WL82 loader source or reproducible build linkage was identified.
Transporter's build merely embeds an externally supplied file into a generated
header. Its existence check does not itself pin a digest, and its source comment
calling the subset read-only does not audit the embedded program.
[Unbricker pin](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L87-L100),
[embedding recipe](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/CMakeLists.txt#L33-L50).

The loader must not be confused with the stock flash-resident `uboot.boot` SPL
or the stock OTA executable. FM-1-RE's SDK match and disassembly for `uboot.boot`
cover a different 14,384-byte artifact; they do not prove this 24,064-byte USB
loader's behavior.
[SPL identity](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/analysis/device/uboot/README.md).

**Binary audit is unresolved.** The reverse-engineer-anything workflow was
consulted; REA's session inspection reported no open target and an unconfigured
Ghidra provider. No fresh decryption/disassembly, emulator run or binary call
graph was produced, and no analysis engine was installed. Published stock
receiver disassembly was read as prior static evidence. The following exact
loader paths therefore remain unknown: entry and constructors; SPI initialization
and status/protection-register handling; callbacks/interrupts; `FD05` dispatch
and its full transitive flash-access implementation; watchdog; error/timeout
handling; and return/reset behavior. A digest or two identical dumps cannot
resolve these unknowns or prove an absence of persistent changes.

The host project's MIT license is present, but this investigation establishes
neither a vendor source provenance chain nor a separate blob redistribution
grant. Preserve notices and avoid bundling the blob until that question is
resolved. No broader rights conclusion is implied.
[Repository license](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/LICENSE).

## Reads, supporting commands and return

The intended data operation is **`FD05`**, with big-endian raw byte address and
16-bit big-endian size. The inspected double-backup loop issues that operation
for bounded flash intervals and compares two complete reads. No erase/program
CDB is called by that loop. This proves the host's command selection only;
the loader's internal read implementation is still opaque.
[Read method](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/uboot.py#L365-L369),
[capture loop](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L514-L533),
[two full reads](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/8b4992ff684158ed1b27299792eedcfd0f9617ea/fm1_unbrick.py#L1073-L1092).

Supporting traffic requires its own review. `FC09` reads the chip key with
argument `0x00AC6900`; generic protocol documentation assigns this argument an
unlock effect on BR17, which must not be silently generalized to WL82.
`FC0A` gets online-device information and Transporter also uses it every second
as a keepalive; `FC14` gets the USB buffer size. These are not added to the
project's existing INQUIRY/FD05 allowlist by this report. An unknown previously
running loader must not be accepted merely because such a command succeeds.
[Supporting host methods](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/uboot.py#L387-L404),
[buffer query](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/uboot.py#L439-L442),
[BR17-specific unlock](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/usb-loader-v2.md#read-key).

The reported loader resets after about three idle seconds. `FC0C`/`RUN_APP`
is an active lifecycle transition, not a read. The generic protocol explicitly
allows implementation-dependent reset/no-op behavior. Transporter's author
reports stock V15 running after a warm loader return/watchdog reset but failing
to attach USB until a cold power cycle. Thus neither a successful `RUN_APP`
response nor a missing MIDI port qualifies normal return. Its host implementation
even returns success after issuing `RUN_APP` without requiring command success;
that behavior must not be inherited as a device-health receipt.
[Observed lifecycle](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/docs/DEVLOG.md#L107-L150),
[return and keepalive code](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/src/jieli_uboot.c#L266-L280),
[generic return semantics](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/usb-loader-v2.md#run-app).

## Route verdicts and next evidence

| Exact route or operation | Verdict under the no-flash-write requirement |
|---|---|
| Running stock → arbitrary full raw MIDI read | No qualified endpoint; disabled |
| Reported stock V15 → `7D` USB-MIDI → ROM | Entry candidate supported by external observation; exact image/handler/callee/framing and selected-device binding remain gates |
| V13 published receiver → `7D` ROM branch | Bounded static handler evidence; no full effective-magic/callee or physical qualification |
| V14 or unknown stock → inferred same entry | Unqualified; refuse rather than infer version equivalence |
| `7F` ordinary OTA as entry fallback | Rejected: update-state writes are part of the documented flow |
| Known ROM → exact 47-block RAM upload + jump argument 1 | Host sequence understood; volatile execution and loader audit gates remain |
| Exact loader → bounded `FD05` full reads | Host request shape understood; transitive binary effects and real selected-device transport remain gates |
| Arbitrary ROM memory read or arbitrary RAM payload | Outside the approved route; do not expose |
| Previously running, unidentified loader | Provenance unknown; refuse |
| Loader watchdog/`RUN_APP` → normal stock | Mode-changing and incompletely qualified; explicit normal-return evidence required |
| Complete stock → ROM → loader → double backup → stock | **Not qualified for physical activation by this audit** |

Offline adapter work may proceed behind a closed connection/activation boundary.
It must bind actual handles, stock/ROM physical-location continuity and fresh
transfer results without treating saved records or caller assertions as authority.
It must not expose entry, upload/jump, keepalive, reset/retry, flash/key/status
writes or arbitrary CDBs merely because a native API can send them. OS/kernel
capture, release and error-recovery effects remain the separate access audit in
report 19; host IN direction alone cannot establish preservation.

Before an authorized physical backup, obtain an exact owner/source profile and
effective receiver bytes; resolve the ROM and loader startup/read/return audit
including persistent flash configuration and nonvolatile key effects; pin lawful
checked loader bytes and execution parameters; qualify direct-computer framing
and chosen-OS lifecycle; then demonstrate two fresh full reads, durable independent
copies and documented return. The owner's existing bounded-backup authorization
does not need to be asked again once these concrete evidence gates are met.
Until then, hardware capability and no-flash-write qualification remain false.

Verification for this report was source/metadata inspection and independent
SHA-256/Git-blob hashing. No tests were run by this research task; canonical host
and native implementation validation belongs to the integrating task.

A bounded follow-up public search covered the exact loader SHA-256, loader name
with source/audit/disassembly/startup terms, the exact eight-byte entry transfer,
and V15 entry with Windows/Linux. It found no additional exact-loader build/effects
audit or independent direct-computer framing receipt that clears these gates.
This is search coverage, not proof that such evidence cannot exist.
