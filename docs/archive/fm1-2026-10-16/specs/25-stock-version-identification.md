> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Stock firmware version identification without updating

Research date: 2026-10-15. Scope: source/document review only; no hardware access,
port inventory, query, tool execution, dependencies, tests or builds. The owner's
reported normal USB identity is `4C4A:C755`, revision `0100`; its installed stock
firmware remains unknown. macOS 27 / Apple Silicon compatibility has not been
verified by this research.

## Result

Owner follow-up on 2026-10-15: fm1-editor.com reports **V15**. This is an
owner-supplied UI observation, without a raw CLI reply or firmware hash.
[FM1-034](26-native-midi-identity.md) implements the separate native identity
command; its development/tests remain hardware-free. Earlier “unknown” statements
below describe the research baseline before this follow-up.

The strongest evidence is a **single vendor identity request over normal USB-MIDI**.
It does not require an update, ROM/OTA entry, RAM loader or preset transfer. A
separate owner's published macOS bench record captured stock `FM-1_015` this way.
This establishes a reported firmware identity, not a byte-for-byte firmware
integrity check. It does not identify this owner's device without a fresh,
separately scoped capture. [L1][A1]

A confirmed stock startup/version/About screen was **not found**. The current
manufacturer manual describes Globe pages for MIDI/pitch and reset/velocity/glide;
it does not document a version item. Its English display/global-settings sections
and searchable PDF contain no firmware-version procedure. Absence from this
manual does not prove that every stock release lacks such a screen. Do not
recommend guessed button combinations or exploratory settings changes. [M1]

## Exact request and response interpretation

The documented identity request is exactly ten wire bytes:

```text
F0 00 32 45 00 00 00 40 7F F7
```

This is a vendor-specific request; do not substitute the universal MIDI identity
request. aroum's updater analysis independently records these request bytes and
the packed reply channel `00 32 45 58`. Its broader scanner/flasher is not a
bounded identity-only implementation. [R1]

Unpack **all bytes between `F0` and `F7`, including the apparent channel header**,
as one continuous LSB-first 7-to-8-bit stream. AL-255's current
`parse_handshake_identity` requires 34 decoded bytes, prefix `00 59 11`, a
three-byte little-endian body length of 27, and an 8-bit complement-of-sum checksum
on the body. The ordinary stock response body is a zero-padded ASCII
`MODEL_NNN` identity. Parse the decimal suffix after `_`; e.g. `FM-1_015` reports
version 15. The parser's legacy encoded-field fallback and special custom-V16
checksum exception should not become acceptance rules for an unknown stock
capture. Reject malformed responses and retain their raw bytes. [A1]

The macOS bench's V15 example decodes to:

```text
00 59 11 | 1B 00 00 | 46 4D 2D 31 5F 30 31 35 + 19 zero bytes | 19
                         F  M  -  1  _  0  1  5                  checksum
```

That bench found the plain identity reliable; an older secondary-field algorithm
returned version zero on the same reply. It also recorded USB `bcdDevice=0x0100`
with `FM-1_015`. Therefore the owner's USB revision `0100` cannot establish stock
V1, V10, V15, or any other installed firmware number. [L1]

## Narrow future capture boundary

For a separately authorized owner capture, use an audited sender that addresses
one explicitly selected USB input/output pair, sends only the ten bytes above,
waits briefly, saves the raw reply locally and closes the ports. No broadcast,
substring-first automatic selection, parameter scan or retry escalation. Compare
a decoded response to the source structure above. On silence or ambiguity, stop;
retain “version unknown.” These are proposed constraints, not an executed session.

`tools/fm1_identify.py` in Lunar is useful reference source: its only outgoing
message is this query, but `query_device` enumerates ports and chooses the first
substring match. Its decoder reports length/checksum status rather than rejecting
all invalid identities. It is therefore **not approved unchanged** for the
repository's explicit-target requirements. It also needs hardware backends for
live use; none were installed here. [L2]

The upgrade command is a different frame, `F0 22 24 35 7F F7`. Do not send it,
open an installer as an identity reader, provide an update package, serve device
file requests, change modes or send any preset/settings message. The normal-mode
identity exchange precedes the update command in the upstream state machine.
[A2]

## Practical macOS UI for an advisory version reading

The FM1 Editor & Librarian documents a firmware identity display: connect MIDI
with SysEx, choose the FM-1 output and input monitor, then read **Settings →
FM1 firmware** or the label beside **MIDI online**. Chrome/Edge are documented
desktop options. Only connect and read this field; patch selection/editing or
send actions can transmit preset, parameter and program-change messages. [B1]

At source revision `185d775005d8f1ec312d67dfb74fc87fba56e31f`, the identity
effect sends the exact query above to its selected output and listens on the
selected input, with up to three attempts. Initial port selection can select
the first available device. For this manual workflow, make the FM-1 the sole
external MIDI device before connection, close other MIDI clients and verify
both selections. Ambiguous ports mean stop. This external UI is not adopted
as our exact-selected-device backend. Its parser does not reject checksum
errors, so its label is advisory; retaining the raw reply through the MIDI
log supports an independent strict check. [B1][B2][B3]

Only public text/source was inspected. The live page was not granted MIDI
access or exercised with hardware, and its deployed bundle is not attested to
the pinned source revision. Our native factory and backup gates are unchanged.

## Evidence limits and unresolved gaps

- A matching model/version response is firmware-controlled data. AL-255's parser
  explicitly handles a modified V16 identity/checksum case; it cannot prove an
  unmodified stock image or full-flash integrity. [A1]
- The manufacturer's downloadable “V15” identifies an available package, not the
  contents of a connected unit. [M2]
- Stock screen behavior across all releases and actual macOS 27 / Apple Silicon
  execution of the documented browser workflow remain unverified. Aftermarket
  firmware startup/About behavior is not stock evidence.
- No exact MIDI pair, response, firmware hash or actual version for this owner's
  FM-1 was obtained. Existing USB descriptors remain historical owner evidence.

## Primary sources inspected

- [M1: M-VAVE Detailed User Manual, English PDF pp. 3 and 9](https://manualf.oss-cn-hongkong.aliyuncs.com/manual/MIDI-KEYBOARD/FM-1-manual.pdf), linked by the manufacturer's [product data](https://www.m-vave.com/products-data.js).
- [M2: M-VAVE FM-1 product/download page](https://www.m-vave.com/product?id=fm-1).
- [L1: Lunar's author's macOS stock-V15 bench record, sections 1–2](https://github.com/ip2k/lunar-modulator/blob/bfa042446f36c8647efddc6d76b06753d2a5b612/notes/2026-09-06-bench.md).
- [L2: Lunar identity-tool source](https://github.com/ip2k/lunar-modulator/blob/bfa042446f36c8647efddc6d76b06753d2a5b612/tools/fm1_identify.py).
- [A1: FM-1-RE parser and request constants](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/tools/fm1_ota.py).
- [A2: FM-1-RE captured protocol, Session flow](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/11-ota-protocol.md).
- [R1: aroum's updater analysis, section 4.1](https://github.com/aroum/fm1-custom-fw/blob/c48c414041fb3e0d9cd3fdb2ee3a012fba17617e/REVERSE_ENGINEERING.md).
- [B1: FM1 Editor user guide, connection and firmware display](https://github.com/benny-sparra/fm1-dx7-patch-importer/blob/185d775005d8f1ec312d67dfb74fc87fba56e31f/docs/user-guide.md).
- [B2: Editor identity query and response parser](https://github.com/benny-sparra/fm1-dx7-patch-importer/blob/185d775005d8f1ec312d67dfb74fc87fba56e31f/src/lib/fm1-firmware.ts).
- [B3: Editor port selection and identity effect](https://github.com/benny-sparra/fm1-dx7-patch-importer/blob/185d775005d8f1ec312d67dfb74fc87fba56e31f/src/hooks/use-midi.ts).
