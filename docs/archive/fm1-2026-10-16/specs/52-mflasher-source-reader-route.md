> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# M-Flasher source-defined RAM reader — alternative backup route

Date: **2026-10-16**, branch `init`. GPT 6.1 Sol high agents audited the
reader/startup and host route; a medium agent independently checked source
and evidence. Primary sources and two prebuilt files were retrieved by GET,
then inspected as inert local data. No hardware inventory/open/query,
MIDI client, [local artifact location omitted] execution or emulation, firmware script/build,
backend installation, remote mutation or flash write.

## Decision

**Yes: this reader provides a concrete source-defined route without the
vendor initializer at 0x01E00428 or a dependency on FC0A. Prefer a minimal
read-only derivative as the next development step.** This is a positive
route decision, not another request for the missing vendor ROM function.

The selected source startup initializes SPI directly and FD05 issues NOR
read **03**. The matching prebuilt entry/read anchors corroborate that local
flow. USB handoff/send still use ROM services; their transitive bodies were
not recovered. No claim is made that no internal ROM operation can ever
reach a particular address.

**The existing full M-Flasher program is not the artifact selected for a
first test.** Its RAM binary includes erase/program handlers, and its host
still probes FC0A, can fall back to the vendor loader, retries transfers and
resets BOT. A reduced reader plus a strict host path removes these concrete
issues. No source is copied into the product or profile activated here.

The owner's suggestion to make a practical hardware test is recorded below.
This task explicitly excludes hardware access, so only the test plan is
prepared. The next physical milestone is a small read/return test, followed
by the full backup after that succeeds; complete ROM reconstruction is not
required for this different reader.

## Exact sources and prebuilt artifacts

Sources are pinned to
[M-Flasher commit 9d962747](https://github.com/jvitkauskas/m-flasher/commit/9d962747437d92cbab594c2320b586d162c6572d),
whose commit record names tree `0df317043df47f19d6c45cdb5829c65054276517`.
The retained recursive listing supplies source/file blob identities; its
top-level echoed commit ID is not used as actual tree identity.

The saved [loader C source](https://github.com/jvitkauskas/m-flasher/blob/9d962747437d92cbab594c2320b586d162c6572d/loader/loader.c)
is 13,974 bytes, SHA-256
`[digest omitted from public archive]`,
Git blob `92b4ba139555b06792bf262ff1fd5cf1e6281a0c`. The adjacent README/linker
and 18 newly retrieved dependency/host/license/build-metadata/artifact bodies
match the retained Git blob identities and sizes. No build script was run.

| Plain prebuilt artifact | Address / uploaded extent | SHA-256 |
| --- | --- | --- |
| [`mflr-loader-a.bin`](https://github.com/jvitkauskas/m-flasher/blob/9d962747437d92cbab594c2320b586d162c6572d/data/mflr-loader-a.bin), 3,496 bytes | `[0x01C20000,0x01C20DA8)` | `[digest omitted from public archive]` |
| [`mflr-loader-b.bin`](https://github.com/jvitkauskas/m-flasher/blob/9d962747437d92cbab594c2320b586d162c6572d/data/mflr-loader-b.bin), 3,496 bytes | `[0x01C28000,0x01C28DA8)` | `[digest omitted from public archive]` |

An original ELF data carrier preserves every binary byte. The already
installed, hash-pinned static reader decoded both in a read-only container
with network disabled and no host device passthrough. Each listing has
**1,300 byte-matched rows / 3,496 covered bytes**, no gaps, overlaps or byte
differences. Linear unknowns include data (46/47); this is not a claim that
every row is executable code. Entry's three local direct calls were also
checked by independent signed-displacement arithmetic. The prior REA
unsupported-architecture result was reused; no new native session opened.

Original build script, headers and binaries are mutually informative, but no
reproducible rebuild or original ELF/map was obtained. A future reduced
artifact needs its own source/build/hash/listing binding; these binaries are
evidence, not the future read-only payload.

## Source and binary startup/read closure

| Operation | Observed source / artifact-A anchor | Effect and limit |
| --- | --- | --- |
| Entry | C490–499; `0x01C20000` | Reads the three interface words, initializes CRC table/SPI, installs the hook, conditionally changes clock, returns0. No vendor initializer call. |
| SPI startup | C187–206; `0x01C200B8` | Disables SFC; changes SPI0, mux and PortD; reads JEDEC with **9F**. No local NOR write-enable/status-program call. |
| FD05 dispatch | C382–384; `0x01C204C6` onward | Calls local flash stream with requested address/length. No online flag/FC0A prerequisite. |
| Raw read | C210–251; stream `0x01C20238` | **03** + 24-bit address, SPI DMA into RAM with SPI-path decryptor configured off, wait/end and send. |
| Send | C253–258; `0x01C206F6`, indirect call at `0x01C20718` | For length256 or512: one DMA fill, one supplied ROM send. Result ignored; consumed-before-reuse/next-BOT behavior still matters. |
| FC0A in this reader | C394–400 | Returns cached JEDEC metadata. It is not the vendor NOR reopen/unprotect routine; it can simply be omitted from the proposed host sequence. |

The selected local path emits no **06**, **01/31/11**, **02**, **20/D8** flash
programming sequence. Those operations do exist in the full source's
destructive handlers: FB00/FB01 → erase/unprotect, FB04 → program/unprotect.
Thus source-level read feasibility is positive, but the full image is a
combined flasher. Unrecognized commands return0 to the ROM. A future
read-only variant must address that dispatch explicitly, not just hide write
buttons in the host.

SPI polling has no timeout in the current reader. Startup may change the
CPU clock48→96 MHz. Neither is a persistent flash write, but either matters
to liveness/timing. The proposed small reader omits clock acceleration and
uses bounded waits/error handling. Optional display, efuse/P33, UID and CRC
features are not needed for its first read and can be omitted.

## Its own memory and callback requirements

This reader has its own interface and layout. The old vendor's configuration
A/B observation, `0x01E00428` caller contract, debug32-byte interface read,
timer IRQ setup and fixed84-byte FD07 metadata are **not carried over**.
M-Flasher entry loads only `{send, recv, handler_slot}`: 12 readable bytes
and a separate writable hook word at the supplied slot pointer. The selected
read invokes send; optional receive/efuse/display bodies are outside it.

Literal-derived artifact-A layout:

| Local resource | Extent / location |
| --- | --- |
| CRC table | `[0x01C20DB0,0x01C211B0)` |
| First / second4096-byte buffers | `[0x01C211B0,0x01C221B0)` / `[0x01C221B0,0x01C231B0)` |
| State at `0x01C231B0` | flash ID+0, last error+4, CRC accumulator+8, clock flag+12; send/receive/slot+16/+20/+24 |

These are object locations, not a recovered active ROM allocation map. A
higher load address does not prove the complete live ROM workspace is disjoint.
Selected local stack contributions are entry/CRC peak24 and
handler60 + stream44 + send-piece24 = **128 bytes before the opaque send**;
they are not total stack/interrupt bounds.

The linker marks BSS NOLOAD and entry does not zero all of it. For selected
FD05, io/flash ID/table are assigned and DMA fills the sent bytes, so lack of
blanket BSS zeroing is not automatically a raw-read blocker. One compiled
optional detail does depend on prior zero state: clock reporting uses a BSS
boolean to select48/96. A reduced reader must explicitly initialize every
retained state field; do not infer unused BSS contents or retain that feature.

One256-byte request avoids within-request buffer pipelining/double-send. It
still needs qualified actual entry/hook/CBW/answer/stack/buffer ownership,
SPI/chip behavior and send consumption before another command reuses memory,
including relevant failures. These may be established by matching effects or
scoped external qualification; no complete ROM body is demanded.

## Why the existing host should not be the first-test host

The pinned
[`jl.rs`](https://github.com/jvitkauskas/m-flasher/blob/9d962747437d92cbab594c2320b586d162c6572d/src/jl.rs#L123-L180)
starts with FC40, then FC0A if no own reader was identified; it may retain a
running vendor loader or fall back to the SDK loader after failed own-reader
identification. It always queries FC09/FC0A after startup. Its own-reader
read size becomes16,384 bytes and commands can retry four times. These
policies can be replaced; they are not necessary properties of FD05.

Other concrete differences:

- [`usb.rs`](https://github.com/jvitkauskas/m-flasher/blob/9d962747437d92cbab594c2320b586d162c6572d/src/usb.rs) accepts successful bulk-IN without checking the actual byte count, omits CSW residue validation, and automatically resets BOT/clears halts for some errors.
- Selection uses bus/address and a mass-storage interface; our exact identity/interface/continuity contract cannot be replaced by first-match selection or retrying claims. The macOS path includes exclusive acquisition and separate driver lifecycle behavior.
- [`ops.rs`](https://github.com/jvitkauskas/m-flasher/blob/9d962747437d92cbab594c2320b586d162c6572d/src/ops.rs#L153-L189) normally uses one full read plus target CRC32; a second full read is conditional. Its file save does not implement our fsync/reopen/two-independent-copy acceptance.
- The inspected host has no backup-only verified return command; Drop/release is not evidence of original-stock restart. An attributed macOS26/M3 SMK-37 test in PR1 explicitly leaves FM-1 untried.

None of this requires adopting another USB library/backend. Reuse our existing
strict transport/storage boundaries with a new exact reader profile; keep
the current vendor profile unchanged and disabled for unqualified hardware.

## Next implementation and first hardware test

The [backlog](04-backlog.md) records three finite steps:

1. **Offline reader artifact:** prepare a source-defined fixed256-byte
   read-only payload. Remove destructive handlers, vendor fallback, clock
   boost and optional peripherals; bind exact RAM/code/state/stack, finite
   waits, explicit initialization and fail-closed command behavior. Review
   source and resulting binary. Builds require a task explicitly scoped to
   that RAM artifact; no firmware build was run in this audit.
2. **Offline strict host route:** new reader identity/JEDEC/read contract,
   exact payload/upload binding and one owner of the selected transport;
   no FC0A/FC09/ROM-FD07, probing fallback, automatic retries/resets or old
   vendor metadata assumptions. Verify failure/short-transfer/CSW/copy cases
   with the guarded runner and injected transports; integrate existing
   two-fresh-read/two-copy storage rather than create another fake-only
   evidence milestone.
3. **Separately scoped limited hardware session:** after these exact
   artifact/entry/transport requirements are met, use one selected V15/Mac,
   known software entry, one reader upload/jump, fixed16-byte reader identity
   and JEDEC capacity check. Read256 bytes at0 twice; then, if valid, read256
   bytes at capacity−256 twice. Stop on any mismatch, wrong tag/length/
   residue/status, timeout or continuity loss. No automatic retry/reset,
   erase/program/status-write/fallback. Quiesce successful reads and use
   the reviewed manual-cold return, then confirm original-stock identity.

The small session is **not a full backup**. Its success supports the next
stage: two complete fresh equal raw passes across independently established
capacity, durable reopened independent copies and stock return. A prior
backup, restore-write trial, universal ROM audit or all-OS run is not a
prerequisite to this limited qualification.

RAM execution changes device state and can hang; absence of flash writes
does not make it passive or guarantee zero risk. The owner's willingness to
test does not justify running the combined upstream flasher/fallback now.
If this source-defined route later fails its bounded test, keep the failure
receipt and decide explicitly; do not silently fall back to the vendor
helper and its unresolved startup/status effects.

## Notices and validation

The complete root MIT text and data NOTICE were read. They distinguish the
host license from the loader's declared Apache2 scope. Complete Apache2 text
and upstream NOTICE were also retrieved/read from the declared lineage at
pinned `fm1-bootloader` revision `14b26f4f…`, with Git-blob checks. That
lineage revision does not prove the exact derivation revision of every loader
file. Any later derivative must retain applicable licenses/attributions and
mark modifications; the LZ4 decoder is not needed by this reader. No product
implementation or binary redistribution occurs here.

Sanitized evidence (historical reference `evidence/52-mflasher-source-reader.json`; omitted from public archive) records the
source/artifact hashes, command-path facts, static checks and limits.
Ignored evidence is in `[local artifact location omitted]`. Independent source,
listing-byte, digest/blob, JSON, link and scoped-diff review applies. Product
tests are not rerun for this documentation/static-analysis-only increment.
All product profiles/factories, upstream firmware/PIO/submodule pins/notices
and flash-write gates remain unchanged. Local `init`, no push; unrelated
rescue-source documentation is preserved outside this increment.
