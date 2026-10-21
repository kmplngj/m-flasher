> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# macOS entry packetizer and exact V15 USB receiver — R4-C

Date: **2026-10-16**, branch `init`. Continues
[report 41](41-rom-fd07-macos-entry-return.md) and
[report 45](45-official-30k-loader-no-status-read.md). Saved system/firmware
files only; no network, device inventory/open/query, CoreMIDI client, driver
execution, target emulation, firmware build/script, installation or flash write.
GPT 6.1 Sol high agents investigated packetization and reception separately;
a medium agent independently verified byte/fixup/dispatch evidence and this report.

## Result and scope

**The previously unresolved local macOS packetizer is now bound to its driver
and device constructors.** One quiet legacy six-byte packet can produce exactly
`04 F0 22 24 07 35 7D F7` in a single eight-byte submission. The exact retained
V15 receiver is also resolved down to its DMA/SIE access and advertises bulk
OUT endpoint4 with a 64-byte maximum packet size.

This closes a concrete static producer/consumer compatibility question, under
the conditions below. It does **not** establish which driver/slice/queue owns
the owner's active endpoint, prove actual delivery/reenumeration or qualify
post-reset ROM effects. The native factory and stock-entry command remain
disabled; no raw backup or RAM capture was made.

The next missing evidence is the already identified selected V15 reset-to-ROM
consumer, bare-ROM FD07 restoration and selected RAM ownership, alongside the
actual macOS endpoint/object/lifecycle binding. No new universal packetization,
whole-ROM, prior-backup, write/restore or all-platform gate is introduced.

## Exact retained inputs and static method

| Input | Bytes | SHA-256 |
| --- | ---: | --- |
| Selected AppleMIDIUSBDriver ARM64e.x1 slice | 229,488 | `[digest omitted from public archive]` |
| Other retained ARM64e slice, excluded from this proof | 229,504 | `[digest omitted from public archive]` |
| Exact V15 app.bin | 581,564 | `[digest omitted from public archive]` |
| Exact V15 initialized application RAM | 41,052 | `[digest omitted from public archive]` |

The hopper-named saved Mach-O is the selected **original** subtype
`0x8000000C` member, not a patch of the other `0x80000002` slice. The
previously retained FAT/extraction receipt binds it to offset475136 in installed
driver 1.10/build88, whole-file digest `990e404d…`. This increment rechecks the
saved slices and receipts; it does not reread the whole installed executable,
observe a loaded slice or assert device ownership from a filename.
V15 provenance and extraction remain bound to [report 40](40-v15-entry-rom-backup-audit.md).

Both Mach-O slices have 557 LC_FUNCTION_STARTS entries. The selected image has
569 entries in its actual chained-pointer walks; selected vtable words below
are confirmed **chain members**, not guessed pointers based on low bits.
Pointer format is12, with authenticated rebases, 16 KiB pages and161 imports.
The existing pinned LLVM23.1.3 reader decoded only bounded static ranges.
Hopper/REA again lacks procedures at selected new-PAC addresses; these negative
results describe its coverage. They are not evidence that functions are absent.
The REA session was exported and closed, with no driver execution.

The existing pinned pi32v2 host reader decoded six independent V15 ranges
inside the same cached, digest-pinned, network-disabled/read-only container
used in earlier reports. Inputs are synthetic ET_REL metadata carriers with
no entry/program headers; only the x86 host reader runs. No target instruction
was executed or emulated and no image/toolchain was installed.

Addresses below refer to the **selected Mach-O slice** unless prefixed
`0x0200…` / `0x01C0…`, which identify mapped V15 firmware. Analyst labels are
not recovered source symbols.

## Driver class → device → legacy packetizer

| Link | Exact selected evidence |
| --- | --- |
| Factory → driver | Exported `1BEFC → 1BFC4` installs authenticated vptr **28AD0**. |
| Driver's device maker | Slot **28BA0 = vptr+D0**, raw `80093346000181F8`, authenticated rebase to **181F8**, diversity3346. |
| Device argument propagation | `181F8` saves its driver pointer at stack+8, passes its address to `1B980`, then `1BA18`; that dereferences the pointer and calls `B3B0 → ABA0`. |
| Device constructor | `ABA0` installs device vptr284D8 with diversity9824; `AC10` stores that incoming driver pointer at **device+50**, matching DoWrite's receiver type. |
| Legacy dispatch | `EAA8..EAB8` loads **driver vptr+C0**, authenticating with diversityB10A. Slot **28B90**, raw `8009B10A0001CAC8`, is a rebase to **1CAC8**. |
| Thunk → packetizer | Thunk `1CAC8` rearranges device/queue/output/capacity arguments and branches at `1CADC` to **19254**. The separate C8 slot belongs to the other queue/protocol path. |

Two attractive same-offset leads were rejected: A574's +50 belongs to a distinct
small callback object, and 10E28 manipulates temporary vector/ownership fields.
Neither is evidence of an alternative USBMIDIDevice constructor or mutation.
The actual construction chain above establishes type and argument identity.
It does not identify a currently loaded instance on the owner's Mac.

## Quiet six-byte packet and capacity

The earlier `Send` path DE44 creates a fresh node from a logical MIDIPacket.
For this case, length+18 is6, inline payload+28 is `F0 22 24 35 7D F7`,
heap-payload pointer+20 is zero, consumed offset+70 is zero, and cable/pipe
metadata comes from the selected endpoint. DoWrite selects the legacy queue
at device+1B0 when the UMP path is not selected for pending data.

At **19254**, the output cursor limit is `output + capacity - 4`. Each USB
event occupies four bytes. With cable0 and room for both events:

| Step | Input consumed | USB bytes appended |
| --- | ---: | --- |
| SysEx continuation, CIN4 | F0 22 24 | `04 F0 22 24` |
| Three-byte SysEx end, CIN7 | 35 7D F7 | `07 35 7D F7` |

The quiet one-node case returns **8** at1953C. There is no transport flush
between these events inside this body. Capacity4 instead returns4 after the
first event and retains consumed offset3 for another invocation. Thus one
logical packet is not a universal promise of one USB submission.

Capacity is not guessed from six logical bytes. The selected buffer constructor
path `12434→124B8→12514` loads a u16 from pipe-record+4 (`1254C`), passes it
to9CD0, allocates payload storage and stores capacity at buffer+18 (`9D54`).
DoWrite reads that field atEA08 and payload pointer atEA04. The V15 descriptor
below supports capacity64 for the matching endpoint; the actual selected
host pipe/record and successful allocation remain part of concrete binding.

The sufficient local case is therefore **one fresh six-byte legacy node,
cable0, no other pending/competing data, correct pipe, an available buffer
with capacity>=8 and a successful selected submission**. A CoreMIDI message
callback, prior identity response or API return alone does not prove those
queue/endpoint/delivery conditions. No claim covers arbitrary queue state,
SysEx length, other drivers/slices or the UMP path.

## Exact interface dispatch and driver-owned error traffic

The selected setup atB484 calls **5968** and stores the returned interface
at device+68. That routine creates the plugin interface and queries UUID
**64BABDD2-0F6B-4B4F-8E3E-DC36046987AD** (IOUSBInterfaceInterface245),
falling back to **8FDB8455-74A6-11D6-97B1-003065D3608E** (190). The raw
UUID construction and matching retained header, not a nearby function name,
bind the interface type.

The pinned [Apple IOUSBLib header](https://github.com/apple-oss-distributions/IOUSBFamily/blob/1398331b04a6bb9ea9b9f76248b8b584811ebcd0/IOUSBFamily/Headers/IOUSBLib.h)
and local SDK IUnknown macro establish the four-pointer prefix and slots:

| Slot | Meaning / observed site |
| --- | --- |
| +40 | USBInterfaceOpen, selected setup5D88 |
| +110 | WritePipeAsync, DoWriteEB30; packetizer return supplies outgoing count |
| +168 | ClearPipeStallBothEnds, callbackEC90 on nonzero completion status |

Normal immediate submission success avoids the known second call. Immediate
errors **E000404F / E00002ED** still take the second WritePipeAsync call with
the same count/buffer. Completion callback **EC14** clears pending state;
with recovered live device context and nonzero status, it also invokes
**ClearPipeStallBothEnds**. The header
specifies an attempt to send **ClearFeature(ENDPOINT_HALT)** to the device's
default control pipe and clear halt/toggle state. That is additional
device-facing error handling, distinct from our application's no-retry policy
and from nusb's AbortPipe controller-state behavior in report41.

No such call occurred here. The success candidate does not require these
error branches, but the host cannot claim that one application send prohibits
all lower-layer retries or endpoint-state changes on error. A timeout/error
cannot establish stock preservation or authorize an automatic second attempt.
This evidence concerns USB endpoint state, not a demonstrated NOR flash write.

## V15 descriptor, IRQ, DMA and actual handler input

The exact V15 app includes these endpoint descriptors:

| Mapped address | Raw descriptor | Meaning |
| --- | --- | --- |
| **0x0204EB9F** | `09 05 84 02 40 00 00 00 00` | Bulk IN endpoint4, maxpacket64 |
| **0x0204EBAD** | `09 05 04 02 40 00 00 00 00` | Bulk OUT endpoint4, maxpacket64 |

Builder **0x02005B32** copies28 bytes from EB9F to descriptor output+64,
including OUT at output+78, and registers configuration callback
**0x0200629C**. That callback configures OUT4/type2/maxpacket64. Descriptor
bytes, builder references and selected endpoint initialization agree; this
is historical package evidence, not the owner's active descriptor receipt.

The receive chain is now locally resolved:

```text
0x02008050 IRQ wrapper → 0x01C02A12 → OUT4 dispatch 0x01C02C98
    → registered callback 0x02006E1E → usb reader 0x0200683C
    → special handler 0x02006A60 → existing RAM entry 0x01C026A4
```

Registration at0x02005C74 stores the callback for USB0/ep4 in slot
**0x01C0EE9C**. RAM dispatch reads that slot. The reader selects USB0/ep4
through SIE MMIO11800/11804, reads count/status registers22/23 and20/21,
uses RX DMA register1183C and alternates buffers at **0x01C232C4** and
**0x01C23308**. The CSR update helper0x0200602E applies
`old & 0xFFFFFE9B | 0x10`; no new physical ACK/timing semantics are inferred
from that arithmetic.

Wrapper0x02006E1E requests **up to64 bytes**, passing the actual returned
length onward. Reader0x0200683C copies `min(current received count, remaining
capacity)`, updates the DMA selection/status and rechecks readiness. It returns
the accumulated count as soon as no data is ready, rather than waiting for64.
No software MIDI parsing/ring queue occurs before the special handler.

At0x02006A60, the special check requires length8, the exact six-byte prefix
`04 F0 22 24 07 35`, byte7=F7 and selected byte6=7D before calling the already
audited RAM reset-entry function. Normal MIDI handling begins only after the
special check at0x02006AAC. The other byte6 branch is outside this candidate.

| Received during one wrapper invocation | Consequence for the selected check |
| --- | --- |
| Exactly the required8 bytes | Reaches the selected RAM entry branch |
| Two four-byte payloads, both ready before this reader exits | Can aggregate to the same8-byte input |
| Four bytes, reader exits, remaining4 delivered to a later invocation | Neither invocation passes length8 |
| Required8 plus additional ready events/padding, total!=8 | Rejects the selected check |
| Different cable/CIN prefix or trailer | Rejects the selected check |

Consequently **one eight-byte USB submission is a sufficient transport
candidate under successful delivery/quiet reception, not an architectural
requirement of the handler**. Host submission boundaries alone do not establish
the device's callback boundary. Observed selected delivery/continuity remains
distinct from static compatibility.

## Validation and next bounded work

The packetizer audit checks1,917 printed rows in11 bounded listings, including
explicitly rejected exploratory type leads. Parent interface/buffer/callback
inspection checks1,123 rows in12 listings. These counts can overlap and are
not unique whole-driver coverage. The independent verifier confirms exact
instruction bytes, actual chained-pointer membership, branch immediates and
constructor argument propagation. Neither static decoder executes the code.

The receiver audit checks **934 rows across14 selected firmware ranges**,
with no unknowns or raw-byte mismatches. **Six independent range starts**
produce **470 matching byte/instruction rows** with no gaps, overlap, unknowns
or truncated instructions. One range includes a prefix of a neighboring helper,
not a claim to cover that whole helper. Same-decoder agreement corroborates
alignment, not independent ISA, peripheral, scheduling or physical semantics.
The sanitized manifest (historical reference `evidence/46-macos-entry-receiver.json`; omitted from public archive) binds inputs,
source/reader/receipt digests, exact claims and false physical-authority flags.

**C3's local construction/packetization and receiver question is substantially
resolved. C3 physical object/access/continuity and C2 ROM effects remain open.**
Next work is matching reset-consumer/bare-ROM FD07 restoration and RAM-ownership
evidence for the actually selected helper's finite pre-loader capture, plus the
selected macOS route's concrete binding/outcome. This entry audit does not
select the optional30 KiB helper or replace its separate startup/chip gates.

Once those existing operation-specific prerequisites are established, the first
bounded RAM observation can stop before FB08. Full fresh flash reads still need
the already documented loader/chip/callback/timing and two-copy/stock-return
acceptance. No firmware rollback, RP2040, prior full backup or write experiment
is introduced by this result. No extra fake transition model is implemented
and no public entry/native factory/loader/write capability is enabled.

Raw static artifacts and original file-only scripts remain ignored under
`[local artifact location omitted]`. Documentation, JSON, hash, local-link and scope
checks apply; product suites are not rerun for this docs-only increment.
Work is local on `init`, without push, device capture or observed stock return.
