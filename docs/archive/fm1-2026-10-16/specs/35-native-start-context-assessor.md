> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Bounded saved start-context assessor

Date: **2026-10-15**, Europe/Berlin. FM1-033-D2-R1 on `init`, following
[decision 34](34-no-status-backup-profile-decision.md). GPT 6.1 Sol high
subagents implemented the parser, independently generated vectors and reviewed
the exact-loader contract; a medium subagent made fixture regeneration guarded
and reproducible. Root integrated capabilities, runner selection and evidence.

**R1's initial selected-profile implementation is complete offline.** The pure
Rust API evaluates saved bytes against the conditional default/mode-2 path. It
does not acquire a capture, establish physical freshness, approve ROM entry,
execute a loader or provide a live backup command. No hardware, network, target
execution, firmware build/script, dependency installation or flash operation
was used. Native factories and all read/write command allowlists are unchanged.

## API and strict input

`native/src/startup_context.rs` exports:

- `SavedContext::from_json(&str) -> Result<SavedContext, InputError>`;
- `ExpectedContextBinding::new(device, session, capture)` for caller expectations;
- immutable saved-input `fingerprint()` and redacted `binding()`;
- `SavedContext::assess(&expected_binding, expected_fingerprint) -> Assessment`;
- typed result accessors and `Assessment::to_value()` for sanitized JSON reports.

The parser accepts exactly these JSON fields; unknown and duplicate decoded keys
refuse, including supplied authority fields. All byte/digest text is lowercase
hex. JSON is limited to **16 KiB**, at most **64 intervals** and **2,048 captured
bytes**; each interval is nonempty and at most 1,024 bytes.

| Field | Required value |
|---|---|
| `schema` | `fm1-pre-entry-saved-context-v1` |
| `phase` | `rom-ram-after-rom-decrypt-before-loader` |
| `source` | `saved-rom-ram-capture` |
| `device_id`, `session_id`, `capture_id` | Each 1–128 ASCII characters from letters, digits, `._:-` |
| `loader_encoded_sha256` | `[digest omitted from public archive]` |
| `loader_decoded_sha256` | `[digest omitted from public archive]` |
| `transform` | `jl-crc16-xor-reset-512-v1` |
| `intervals` | Ordered objects containing only integer `address` and hex `bytes` |

Every interval must lie wholly within `[0x01C07E00,+1024)` or
`[0x01C08E00,+1024)`. Overflow, overlap and unsorted intervals refuse. Adjacent
splits are supported; gaps in either 34-byte header or the selected 1 KiB block
refuse. Both headers remain mandatory even if A wins immediately.

The fingerprint hashes canonical JSON, including all captured bytes, labels,
addresses and interval segmentation. Formatting/key order normalize; a changed
representation or capture needs a new expectation. Binding/fingerprint equality
checks **saved-input consistency only**. Caller labels do not attest to a device,
trusted timestamp, ROM implementation or stability across a mode change. A caller
can compare a capture with its own labels/hash without gaining any authority.

## Exact algorithm and deliberately narrow support

The two header candidates reproduce the unchanged loader's **A, B, A, B**
sequence. Each 34-byte header has an independently reset transform; its nonzero
little-endian CRC16 field must match the first 32 decoded bytes. The first valid
candidate wins; when neither is valid, the retained candidate is A. The selected
1 KiB is decoded in **two independently reset 512-byte halves**. These are inert
host arithmetic operations, with no RAM upload or target execution.

Records begin at byte 34. The leading byte is value length (1–32), followed by a
variable NUL-terminated name and that value. The loader copies a fixed 32-byte
name window; the assessor requires that entire window within the capture and a
NUL within it. Values and cursor advancement are checked. A zero value length
terminates; other invalid lengths, truncation and missing terminators refuse.
The complete record stream is checked even after the first SPI match, so later
malformed data refuses. The first well-formed SPI match determines its value.

Initial supported cases are:

- No matched startup keys: default configuration `0x00010300`.
- SPI raw value `2,3,0,0`, with exactly seven bytes or one extra trailing NUL.
  The loader consumes offsets 0/2/4/6 rather than a numeric string parser.
  Comma, colon or space are accepted at the three unused separator positions.

Both yield read word **`0x00020604`**, selector **6**, and the candidate repeated
non-continuous `3B` read path from [report 33](33-loader-no-fc0a-read-session.md).
The narrow no-key label requires both decoded header CRC fields zero and the
first record length zero; other well-formed no-SPI streams have a distinct label.
Unknown record names have no matched startup effect and may be skipped only
after the same structural bounds checks.

Mode 4 refuses unconditionally. Valid modes 0/1/3, other dividers/read/pin values,
and matched `UTTX`, `UTBD`, `RESET`, `EX_FLASH_IO` records remain **unsupported**
in this initial subset. Mode 3 normalizes to mode 2 with divider 1 in the exact
loader; it is not accepted as the divider-3 default. The assessor never guesses
unimplemented numeric or pin parsers and cannot force configuration or patch the
loader. Broader safe profiles would need another reviewed extension.

## Result and authority boundary

`conditional_profile_supported=true` states only that these saved bytes satisfy
the modeled profile. The report always retains historical/unverified, unresolved
ROM/entry-stability, and physical chip/read/return blockers. Private IDs and raw
RAM are omitted from both result JSON and Debug. The saved-input fingerprint and
selected address/configuration may appear. Errors contain fixed sanitized text.

On every result, `hardware_permit_issued`, `real_entry_available`,
`real_read_available`, and `flash_write_available` are **constant false**. A
binding or fingerprint mismatch also refuses conditional support. This does not
detect capture age or establish physical freshness. `status` advertises only
`saved_start_context_assessment_available=true` with scope
`conditional-unverified-saved-mode2-only`. CLI file commands, native USB entry,
loader execution and full physical backup remain unavailable.

## Independent validation

`native/fixtures/startup-context-v1-vectors.json` contains **34 original synthetic
vectors**, consumed by six Rust integration tests. The independent guarded
stdlib oracle in `host/native_startup_fixtures.py` imports no product transform
or parser; `binascii.crc_hqx` is cross-checked against a separate bitwise CRC.
Four guarded Python tests regenerate the exact committed bytes, check both
published K512 hashes, reset geometry, provenance and vector shapes.

Coverage includes A/B valid/invalid CRC selection and fallback, first SPI match,
defaults, optional SPI NUL, cross-half and late records, incorrect continuous
transform, invalid modes/read/pins, missing key NUL, short values, malformed
tails, fixed-copy truncation, true value truncation with an in-bounds key copy,
and a complete final record ending exactly at 1,024 without a terminator.
Additional Rust tables check split coverage, missing/overlapping/outside/overflow
intervals, schema/pin/JSON/hex/allocation limits, binding/digest drift and report
privacy. Positive and refused results both retain false physical flags.

Reproduce through the canonical guards (Python **3.12.15**, Rust **1.99.0**
were used locally):

```sh
rtk proxy python3.12 native/run_tests.py
rtk proxy python3.12 native/run_tests.py --release --test startup_context_conformance
rtk proxy python3.12 host/run_tests.py
rtk proxy cargo fmt --manifest-path native/Cargo.toml -- --check
```

- Focused new Rust target: **6 debug and 6 optimized tests passed**, including
  all 34 literal vectors; no ignored or filtered tests.
- Canonical guarded Python suite: **310 passed in 236.275 s**, no skips.
- Full canonical native debug suite: **213 passed — 165 unit and 48 integration**,
  no ignored/skipped/filtered tests. Unit-suite time was **563.50 s**; the
  audited native and retained 62-file source-map inputs remained unchanged.
  The unchanged full optimized writer sweep was not repeated.
- Source/API/dependency audit passed for **42 Rust files**, with unchanged
  dependency pins and no new policy exceptions. Formatting and local relative
  documentation-link checks passed. The default offline locked release build
  passed; its actual status/help and backup-capture refusal preserved all false
  physical flags. No optional identity or USB backend was executed.
- Independent final static review found no actionable defects in the selected
  subset, integration, fixtures or documentation. This is software evidence,
  not target execution or physical no-write qualification.

Ignored evidence remains under `native/target/`: `fm1-r1-native-full.log`,
`fm1-r1-host-full.log`, focused debug/release logs, default-build/status/refusal
receipts and the independent contract review. The **62-file** source/fixture map
`fm1-r1-source-map.json` covers audited native inputs plus the two new host oracle
files; canonical sorted compact JSON SHA-256 is
`[digest omitted from public archive]`.
The host log SHA-256 is
`[digest omitted from public archive]`.
The full native log SHA-256 is
`[digest omitted from public archive]`.
No push, hosted CI, additional OS execution or hardware acceptance is claimed.

## Next gate

**R2** is the next offline ticket: typed fixed `FD07(0x01C09700,84)` metadata
validation with private phase/session provenance. It cannot add arbitrary RAM
access or open an allowlist. R3 then models finite checkpoint/storage scheduling.
R4 still needs actual ROM/configuration stability, exact chip/read/addressing,
macOS access, native-to-storage provenance, session timing and original-stock
return evidence before any physical backup. R1 does not read the owner's context
or show that the owner's actual device satisfies this profile.
