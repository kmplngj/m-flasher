> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Native parser and wire codec evidence

Date: 2026-10-13. Ticket: [FM1-028](04-backlog.md#fm1-028--rust-package-parser-and-wire-codecs).
Development and tests use local fictional bytes only. No hardware access,
inventory, native backend, firmware build or flash write occurred.

## Implemented APIs and boundaries

| Module | Implemented behavior | Boundary |
|---|---|---|
| `packages` | FWSC marker removal, Latin-1 identity, UFW metadata XOR/CRC, strict entry parsing, hashes and raw APP extraction | Pure bytes; exact fixed APP range; no catalog or write authority |
| Offset types | Separate file, logical, physical and XIP types; checked conversions and address arithmetic | No universal XIP conversion; original file length must fit u32 |
| `sysex` | Canonical continuous little-endian seven-bit packing, requests/responses, checksums and strict identity parsing | No sender, port, replay session or firmware classifier |
| `framing` | Finite Transporter DATA/END frame encoding/validation and CRC32 | 1..65,536 payload bytes; exact requested length, trailer and no trailing bytes |
| `digest` | Byte-only SHA-256 and canonical digest formatting | No OS crypto, IO, FFI or artifact qualification |

Inspection preserves reference refusal ordering for truncation, CRC, count,
chip, entry length, overflow, bounds, overlap and duplicate flash. It accepts
unknown nonempty ASCII chip names and absent flash entries for structural
reporting, then refuses unreviewed chips or incomplete raw flash during
extraction. Inspections own validated bytes privately and always report
`artifact_qualified=false`.

Extraction expectations bind exact caller identity/chip/SoC, package size/hash
and application size/hash. Only `[0x4000,0x93000)` is accepted, located relative
to the validated flash entry. Expectations are **not catalog profiles or
qualification evidence**. Profile/catalog resolution remains FM1-029; there is
no API granting physical installation permission or inferring data compatibility.

Input-sized parser/cipher/application byte buffers use fallible reservation.
Parser identity/chip/entry metadata are bounded. Caller configuration strings
use ordinary owned allocation; no claim of universal allocation-failure or
process/power-loss recovery is made.

SysEx completion markers and matching names remain byte observations only.
Decoding never establishes raw flash correctness, boot, functionality or an
authorized session. BOT codecs, stream readers and fake writer/lifecycle ports
are outside this ticket.

The native capability envelope now distinguishes implemented **in-memory
library APIs** from unavailable file inspection, catalog resolution, agent
lifecycle and every physical capability. CLI package/flash/device commands
still refuse without opening any supplied path. No Python is embedded/launched.

## Independent saved corpus

The manifest (historical reference `../native/fixtures/fixture-manifest-v1.json`; omitted from public archive) records exact file
hashes, sizes, provenance, existing package-manifest equality and both pinned
seed streams. The original package oracle uses a cursor, bit-list cipher and
stdlib CRC; SysEx uses independent bit-string/frame builders; framing uses
stdlib zlib CRC; digest expectations use Python hashlib. Production Python
helpers are secondary checks. A monkeypatch test proves generation succeeds
when production parser/codec helpers are forbidden.

| Corpus | Rows | SHA256 |
|---|---:|---|
| Package | 261 | `[digest omitted from public archive]` |
| SysEx | 658 | `[digest omitted from public archive]` |
| Transporter framing | 37 | `[digest omitted from public archive]` |
| Digest | 140 | `[digest omitted from public archive]` |

Package coverage includes three independent positive layouts, 18 malformed
cases, 120 seeded valid and 120 seeded invalid cases (`0xF1000401/2`). Exact
original/logical/application hashes preserve both the historical wrong logical
`0x4000` extraction and the assumed `0x400` flash-entry shift regressions.
Identity marker holes and every marker value are exercised.

Wire vectors include every response payload length 1..512, empty/odd septet
lengths, high bits, noncanonical padding, checksum/header errors, overflow,
completion payloads and strict plain/OTA identities. Frames include binary
newlines, maximum chunks, short/corrupt trailers, mismatched lengths and tail
data. Digest vectors cover padding/block boundaries; separate known-message
and million-`a` vectors check the original SHA-256 implementation against the
[FIPS 180-4 algorithm](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf).
No cryptographic-module certification is claimed.

Rust integration tests embed the TSV files only in test builds; no runtime
file IO or third-party JSON parser is introduced. `native/fixtures/** -text`
preserves exact bytes on Windows. TSV empty final byte fields intentionally end
with a tab delimiter; only these files exempt Git's blank-at-EOL lint. Exact
regeneration, hashes and column counts still validate every byte. The Python
guard remains unchanged and blocks
device/network/process/FFI escape before test imports.

## Actual local validation

- Cargo debug and optimized release tests passed: **30 tests** (23 unit,
  four independently expected corpus and three retained adversarial tests).
  The release profile retains overflow checks. Adversarial tests cover all
  65,536 two-septet pairs with 256 independently expected canonical acceptances;
  2,049 inputs at lengths 0..2048 from LCG seed `0x28`; and 12,633 byte mutations
  of the pinned 4,211-byte `valid-000` fixture (masks `1/0x80/0xff`, original
  corpus seed `0xF1000401`). They retain panic/refusal, immutable-inspection,
  protected-head and unqualified-evidence assertions without physical claims.
- The fixture agent ran Python **3.12.15** `host/run_tests.py`: **281 tests passed
  in 195.600 seconds, no skips**. Root inspected the final log. Six new guarded
  tests check exact regeneration, oracle independence and secondary reference
  parser/extraction, wire, framed fake-endpoint and digest behavior.
- GPT-6.1 Sol high implemented the parser and independently expected corpus;
  medium implemented the codecs. A separate high agent audited the final
  source, arithmetic, immutable bindings, canonicality and evidence boundaries.
  Root inspected the retained adversarial source and reran all Rust tests after
  integration. Review found no remaining actionable defects.
- Actual macOS arm64 development release build passed offline/locked Cargo.
  It remains **302,784 bytes**, SHA256
  `[digest omitted from public archive]`,
  observed system dependency `libSystem.B.dylib`. Status JSON correctly reports
  pure APIs and unavailable physical/file/catalog capabilities; a package
  command with an inert device-shaped argument returned sanitized exit 2.
  This unsigned dispatch executable may discard unused library code; its size
  is **not a complete parser/flasher benchmark** or reproducibility result.

Commands from the checkout root:

```sh
rtk proxy cargo fmt --manifest-path native/Cargo.toml -- --check
rtk proxy cargo test --manifest-path native/Cargo.toml --offline --locked
rtk proxy cargo test --manifest-path native/Cargo.toml --release --offline --locked
rtk proxy cargo build --manifest-path native/Cargo.toml --release --offline --locked
rtk proxy python3.12 host/run_tests.py
```

Fixture regeneration is explicit and guarded (Python 3.11+):

```sh
rtk proxy python3.12 -c 'import sys; sys.path[:0] = ["host", "host/src", "host/tests"]; from offline_guard import install; install(); import native_fixtures; print(native_fixtures.write_saved())'
```

These are host/model checks. Lawfully usable exact stock V15 and selected real
custom artifacts remain absent; no actual firmware corpus is qualified. Native
profile/planner, secure IO/backup/writer, agent lifecycle, release/signing and
physical qualification remain separately scoped. Browser tooling remains step 2.
Latest hosted CI outcomes are recorded against their exact source commits in
[progress](05-implementation-progress.md).

For source `0244f5b`, [native](https://github.com/kmplngj/fm1-flasher/actions/runs/38433700078)
and [host](https://github.com/kmplngj/fm1-flasher/actions/runs/38433700106) jobs
were not started because GitHub reported an account payment or spending-limit
restriction. Local validation is complete; current-source hosted checks remain
pending. Earlier platform results do not qualify this port.
