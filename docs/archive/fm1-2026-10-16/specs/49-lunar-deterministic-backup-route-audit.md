> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Lunar deterministic helper without A/B observation or FC0A — R4-C-P1

Date: **2026-10-16**, branch `init`. Offline audit with GPT 6.1 Sol high
startup/session agents and a medium independent arithmetic/provenance reviewer.
Continues [report 48](48-online-rom-evidence-route-reassessment.md).
Only saved files, original host-side checkers and an already installed static
object reader were used. No network,
hardware inventory/open/query, fetched Python import/execution, target execution
or emulation, firmware compile/link/build, backend install or flash write.

## Decision

**R4-C-P1 is complete as an offline feasibility audit. The exact Lunar patch
provides a concrete local startup/read path without the A/B pre-read and without
FC0A. It is not yet a qualified physical backup route.**

The patch removes the retained-configuration dependency by construction within
the reviewed local path. Positive FD05 reads refresh the existing idle counter;
the fixed loader-phase FD07 metadata copy supplies cached full ID/capacity and
state without FC0A. This does not establish absent flash/status writes by the
external ROM initializer or asynchronous environment.

Recommend the deterministic helper as the candidate for a later separately
reviewed profile, initially with **256-byte FD05** requests. Record the precise
external startup/handoff dependency now; another scan of the same unchanged
material is not a new milestone. Current runtime helper selection, R1–R4-B
contracts, public factories and all physical writes remain unchanged/closed.
No new profile or patch generator is added to the product.

## Exact artifact and independent reconstruction

The retained Lunar source is pinned to
[`53fa05b278d217476b7e92ba24d6d5dd1e1e1010`](https://github.com/ip2k/lunar-modulator/blob/53fa05b278d217476b7e92ba24d6d5dd1e1e1010/tools/jieli/loader_config_profile.py),
file SHA-256 `[digest omitted from public archive]`,
Git blob `ff8746ea97fb0ab9dbf90f17a6c83b26ae902a86`. Original stdlib-only
arithmetic reconstructed inert data; the fetched module and its imports were
never run. Independent bitwise CRC and stdlib CRC implementations agree.

| 24,064-byte artifact | SHA-256 |
| --- | --- |
| Original encoded | `[digest omitted from public archive]` |
| Original decoded | `[digest omitted from public archive]` |
| Candidate decoded | `[digest omitted from public archive]` |
| Candidate encoded | `[digest omitted from public archive]` |

Per-512-byte decode/encode roundtrip and all three independently reconstructed
files match. Exactly **16 bytes** change in three decoded spans; every other
decoded byte is identical. The same upload remains 47 × 512 bytes covering
`[0x01C02000,0x01C07E00)`, with no extra A/B writes.

| Decoded offset; add 0x01C02000 | Original bytes | Replacement and native effect |
| --- | --- | --- |
| `0x84..0x8C` | `7804e297c8ff0097` | `c0ff407cc0018000`: load r0=0x01C07C40, then `rts` |
| `0x666..0x66A` | `bfeaddfc` | `00000000`: two NOPs, removing first 512-byte transform |
| `0x676..0x67A` | `bfead5fc` | `00000000`: two NOPs, removing second 512-byte transform |

These are byte-analysis facts, not a command/upload interface or permission to
execute the candidate. Encoded bytes/listings remain ignored and private.

## Startup and memory effects

The three direct selector callers are **0x01C02454, 0x01C0265E, 0x01C0266A**.
Their raw relative-call arithmetic reaches the patched leaf at 0x01C02084.
It does not push a stack frame, initialize the selection cache, read A/B,
perform header CRC or call their transform. Old selector code remains after
the leaf's return but these calls do not fall through into it. The second
call's pointer+512 calculation remains; removing the following call leaves
that pointer undereferenced before replacement by the next operation.

The original 64 zero bytes at `[0x01C07C40,0x01C07C80)` lie inside the upload,
after the driver registration table `[0x01C07C28,0x01C07C34)`. The parser first
reads length at returned pointer+34 = **0x01C07C62**. Zero terminates before
record-name/value copies. All five startup lookups take absence, installing
configuration **0x00010300**, SPI mode 2 and default read word **0x00020604**,
selector 6/non-continuous 3B. The reviewed local mode 2 initializer bypasses the
mode 4 `06/01/31` status-program branch from report 32. Startup operation 11
sets local state; it does not select protection operation 13.

Entry clears `[0x01C09600,0x01C09DD0)`; the explicit startup context/BSS,
scratch, registered timer and selected SPI read/metadata destinations do not
resolve to the zero span. Under absence, key/value copying is unreachable.
The selected local path has no resolved store to those 64 bytes, subject to
valid, non-aliasing supplied stack/interface/buffer assumptions. This is a
bounded write-destination review, not RAM protection or a map of every ROM,
other-core, IRQ and DMA user. The interface hook installation is itself a
write through externally supplied `[interface+8]`; address non-aliasing matters.

The unchanged external call **0x01C0278E → 0x01E00428** occurs after defaults
but before NOR open. Its effects on G, padding, code, stack, controller or flash
remain unknown. Entry does not establish its own SP; ROM supplies the interface
and transfer callbacks. Post-start metadata cannot retrospectively prove this
call preserved the no-status startup premise. Reset-to-ROM effects and FB06
wire-transform/handoff correctness remain necessary for the selected route.

Thus the candidate removes the **optional bare-ROM A/B FD07 operation**, its
1,058-byte snapshot, its transform/restore/failure qualification and retained
configuration continuity requirement. It does not remove safe startup or
make every FD07 command unnecessary. Destructive handlers and other A-address
references remain in the image and stay outside the proposed subset.

## Read/metadata lifecycle without FC0A

| Selected operation | Local result | Remaining limit |
| --- | --- | --- |
| Positive FD05 | Hook zeroes idle accumulator G+44; startup already supplied NOR handle, cached JEDEC/capacity, selector and send callback. No FC0A flag/reopen is needed. | Backend/send results are ignored. Host completion/equal dumps alone cannot exclude stale SPI data or prove physical freshness. |
| Fixed loader FD07(G,84) | Local RAM copy from `[1C09700,1C09754)` to disjoint scratch `[1C09600,1C09654)`, followed by supplied sender; no NOR reopen/status operation. | This is loader FD07, not bare-ROM FD07. Cached chip/state is a cross-check, not independent chip/status evidence or safe-start attestation. |
| Timer/finite checkpoints | Positive FD05 reaches the existing hook refresh without FC0A; timer/IRQ code is unchanged. | 3001 idle threshold and the existing finite budget still need qualified clock, callback/OS latency and cancellation behavior. No indefinite keepalive or watchdog suppression. |
| Finish/failure | Stop/drain according to a preselected qualified policy; deliberate manual cold return remains a candidate. | No automatic FC0C/reset/re-entry. External return after firmware restoration does not establish this owner's backup-only V15 return. |

**Correction to reports 33/36 online-flag description:** successful startup stores
byte 1 at G+12 at decoded 0x0900 / mapped0x01C02900, before FC0A. FC0A has a
later duplicate store. The read path does not depend on FC0A setting it.
Current R2 deliberately excludes this flag from identity/continuity authority;
no runtime validation change is required merely for this correction.

The patch also skips selector cache/pointer-table stores, so fresh **G+4
remains 0** under the selected entry premises; it is not the returned zero-data
pointer. Current `loader_metadata.rs` reassesses the original SavedContext and
requires its selected address to equal G+4. Its exact-loader digest is also the
original one. Consequently the candidate must not pass through old R1/R2 by
changing only a hash or weakening that check. It needs an explicit deterministic
start binding and separate metadata expectation if implemented later.

## Why the initial candidate should use 256-byte reads

The external full-read receipt in report48 uses **8,192 FD05 × 256 bytes**
for two 1 MiB passes. Existing R3/R4 code uses **4,096 FD05 × 512 bytes**.
A 512-byte handler performs two internal 256-byte reads, invokes G+36 twice and
reuses scratch after each callback returns, with one hook refresh for the whole
request. The external 256-byte result does not qualify that intra-request reuse
or 512-byte timing. Selecting 256 avoids that additional operation; it does not
eliminate the selected sender's consumption/transfer contract.

The 1 MiB size is the candidate/external receipt geometry, not an observation of
the owner's chip. Actual chip/capacity must be independently established; the
selected supported geometry also avoids the initializer's optional >16 MiB B7
address-mode branch. Cached metadata alone cannot qualify that chip contract.

A later exact 256 profile would have 4,096 fresh coverage requests per 1 MiB pass,
8,192 total, plus bounded separate 256-byte checkpoints and two fixed 84-byte
metadata observations. If h is actual checkpoints, `2 <= h <= H`, the active
data/checkpoint/metadata stage totals **8,194+h**, minimum 8,196 requests;
entry/upload/jump and any separately selected discovery are counted separately.
No FC0A, FC09, FC44, FC46 or arbitrary CDB is introduced.

This is a new explicit profile/schema/count/budget change, not a configurable
length silently accepted by current code. Reuse existing serialization,
two-copy storage, error poisoning and evidence separation when implementing
it. Current 512 contracts and captures retain their exact meaning. Qualification
need cover the selected 256 operations, not 512 or all ROM commands.

## Precise next dependency and backlog disposition

**R4-C-P1 complete offline; R4-C physical route remains partial.** Record
R4-C-P2 as the matching **startup/handoff evidence dependency**: the selected
0x01E00428 call and entry SP/interface/hook/callback memory/effects contract,
including preservation of the deterministic configuration and absence of
flash/status programming on this start/read path. A matching implementation
or appropriately scoped external qualification can satisfy it; a complete ROM
reconstruction, prior backup or write/restoration trial is not required.

Selected raw-chip freshness/capacity, macOS object/access/transfer continuity,
finite timing/cancellation and observed original-stock return remain existing
requirements. A saved-only deterministic 256 profile can be prepared separately
but cannot close this evidence gap. Do not present another fake model or a
repeat of unchanged searches as progress toward physical qualification. No
new source reader, helper rewrite or hardware experiment is started here.

The exact patch source expressly declares MIT and attributes the underlying
protocol/cipher facts. Retained tree metadata lists LICENSE-MIT/LICENSING.md
and another LICENSE, but their bodies are not saved. Full file-specific scope
and notice obligations must be read before copying implementation; unavailable
terms are not evidence of prohibited reuse. Vendor binary redistribution rights
are separate. No tracked implementation was copied and no binary is published;
these missing publication terms do not block this private analysis decision.

## Validation and evidence limits

REA refused the inert ET_REL carrier as unsupported ELF architecture without
opening a native session. The already installed, hash-pinned object reader
from report 31 ran only as a **host static analyzer**, in an already cached,
network-disabled/read-only container with no device passthrough or target run.
No fetched patch module, vendor firmware script or firmware instruction ran.

All **9,480 full listing byte rows** and **291 separately starting rows** match
the candidate. Outside original instructions intersecting the patch, 9,473 full
rows match the old bytes/boundaries/normalized instructions. Five independent
starts have zero differences. The linear listing has 822 unknown rows, mostly
data plus a new unused half-instruction at 0x01C0208C after the leaf return;
that unreachable tail is not part of the new leaf's control flow. No count
asserts whole-image reachable-code or independent ISA/peripheral proof.

The medium verifier independently reproduces CRC/patch/upload geometry, three
root artifacts and saved source Git blob. High agents check startup ownership
and read/metadata/timer/return separately. Original file-only checks, sanitized
JSON/hash/local-link/diff checks and final independent review apply. Product
tests are not rerun for this documentation/static-evidence-only change.

Sanitized manifest (historical reference `evidence/49-lunar-deterministic-route-audit.json`; omitted from public archive) binds
source/input/listing/reader/checker digests, exact differences, contract deltas
and false authority. Raw artifacts stay ignored under
`[local artifact location omitted]`. No hardware capture, accepted full backup or
physical return occurred. Work is local on `init`, without push; unrelated
rescue-source WIP remains separate.
