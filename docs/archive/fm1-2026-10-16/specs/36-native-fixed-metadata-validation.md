> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Typed fixed FD07 saved-metadata validation

Date: **2026-10-15**, Europe/Berlin. FM1-033-D2-R2 on `init`, following
[R1](35-native-start-context-assessor.md) and
[decision 34](34-no-status-backup-profile-decision.md). GPT 6.1 Sol high
subagents implemented and independently audited the exact-loader contract;
a medium subagent supplied independent synthetic fixtures and guarded
regeneration. Root integrated capabilities, tests and documentation.

**The initial R2 mode-2 subset is implemented offline.** It validates saved
84-byte RAM observations and their consistency with the original supported R1
context, declared chip expectations and a second observation. It sends no FD07
request, grants no device permission and cannot qualify startup retroactively.
The owner renewed read-only hardware access for backup testing and prohibited
flash writes. That permission remains valid; a physical backup still requires
the unresolved R3/R4 route/session evidence. No hardware inventory/access,
network, target execution, firmware build/script, installation or flash operation
occurred here. Existing factories and command allowlists remain unchanged.

## Pure Rust interface

`native/src/loader_metadata.rs` exports immutable/private observation types:

- `ExpectedMetadataBinding::new(device, session, start_capture, observation)`;
- `ExpectedChip::new(full_jedec_id, independently_declared_byte_capacity)`;
- `ExpectedObservation::new(binding, start_context_fingerprint,
  observation_fingerprint, chip)`;
- `FirstObservation::from_json` and `FinalObservation::from_json`, each with
  `fingerprint()` and `assess(&ExpectedObservation, &SavedContext)`;
- `FirstObservation::compare_final(&FinalObservation, &first_expectation,
  &final_expectation, &SavedContext)` and sanitized `Assessment::to_value()`.

The fixed source is **`0x01C09700`, exactly 84 bytes**. No API accepts an
arbitrary memory address/count, CDB or adapter. First/final are distinct Rust
types; swapping their phase labels refuses at parse time. This is a library
interface, without a CLI file command or transport/storage integration.

## Strict saved schema and private binding

Input JSON is at most **8 KiB**, with exactly these fields. Unknown or duplicate
decoded keys refuse, including injected address/count/authority fields. Bytes
are exactly 168 lowercase hex characters. Expected digest inputs are length
checked before copying and must be canonical 64-character lowercase SHA-256.

| Field | Required value |
|---|---|
| `schema` | `fm1-loader-fixed-metadata-v1` |
| `source` | `saved-loader-fd07-response` |
| `phase` for first | `loader-before-first-full-read` |
| `phase` for final | `loader-after-storage-checks` |
| `device_id`, `session_id`, `start_capture_id`, `observation_id` | Each 1–128 ASCII letters/digits or `._:-` |
| `start_context_fingerprint` | Exact original R1 canonical saved-input fingerprint |
| `loader_encoded_sha256` | `[digest omitted from public archive]` |
| `loader_decoded_sha256` | `[digest omitted from public archive]` |
| `bytes` | Exact saved 84-byte response as lowercase hex |

Each assessment compares independently supplied binding/fingerprint expectations,
then reassesses the **original `SavedContext`** using its expected device/session/
start-capture labels and fingerprint. Metadata labels and hashes cannot admit
an unsupported R1 mode or a changed start context. The metadata's selected
configuration pointer must match that R1 assessment's selected A/B address.

All labels, phase names, expected chip values and fingerprints remain **saved
claims**. They establish neither trusted chronology, storage completion, capture
age, physical device identity nor stability across ROM entry. Distinct observation
IDs do not establish fresh measurements. No caller-provided qualification bit
is accepted.

## Exact selected-field checks

The response is little-endian SRAM, not a vendor metadata schema. The exact
listing retained by [report 33](33-loader-no-fc0a-read-session.md) was independently
reviewed for field widths, initialization and read-state changes. These checks
describe the narrow R1 default/SPI2 profile between completed operations:

| Offset | Width | Required conditional value |
|---|---|---|
| 4 | word | Original R1 selected configuration address |
| 8 | word | Nonzero private open-device linkage |
| 16 | word | Zero uploaded-callback flag |
| 20 / 24 | words | SPI mode/state `2` / `0` |
| 32 | byte | Zero transform flag |
| 36 / 40 | words | Nonzero private ROM transfer-callback linkage |
| 48 | byte | Zero idle suppression |
| 52 | byte | Zero four-byte-address state |
| 56 | word | Full cached 24-bit JEDEC ID, high byte zero, neither 0 nor `FFFFFF`, equal to declared ID |
| 60 | word | Cached byte capacity equal to independently declared expectation |
| 68 / 72 | words | Saved and active configuration both `0x00010300` |
| 76 | word | `0x00020604`: low configuration 4, selector 6, transfer parameter 2, high flag zero |
| 80 | word | Zero continuous-read state |

The JEDEC sampler constructs manufacturer/type/density as
`manufacturer << 16 | type << 8 | density`; density is the low byte. Unlike
FC0B, this observation contains that density byte. `ExpectedChip` deliberately
supports only density codes **17–24**, power-of-two **128 KiB–16 MiB** capacity
and `capacity == 1 << density`. These are internal format/addressing checks on
two caller-declared expectations, **not independent chip qualification** or a
generic supported-chip catalog. Other density/addressing profiles refuse.

G+20 is set to mode 2 by initialization. Selector 6 leaves G+24 zero between
completed reads. Startup's invalid-ID retry can change the active pin bit while
retaining the saved configuration; requiring both configuration words to match
deliberately rejects that altered profile. Open/callback pointers are checked
only for nonzero/stability; no guessed address range or alignment is imposed,
and matching pointers cannot prove callback behavior or identity.

## Stable-pair comparison and exclusions

Both typed observations must independently pass, share exact device/session/
start-capture provenance, original start fingerprint and declared chip profile,
and have **different observation IDs**. The pair compares the word/byte fields
listed above privately; changed linkage, configuration, cached ID/capacity or
state refuses. Pair comparison never substitutes for either complete raw pass.

Timer word **G+28** and idle word **G+44** can change/wrap while copying and are
not an atomic sample or host clock. They are excluded from stability checks.
The online byte **G+12 may remain zero without FC0A**; it is not required to be
one or stable. Unknown/unmodeled bytes and padding are neither zero-required nor
stability-compared. Fingerprints still bind every saved input byte, so changing
an excluded byte requires a new explicit expectation for that observation.

The fixed RAM copy cannot establish chip status bits or fresh NOR-array data.
ROM callbacks, asynchronous effects, hook timing, storage receipts and actual
stock return remain outside R2. No FC0A, FC40, watchdog change or fallback route
is enabled.

## Reporting and authority

Reports expose only conditional support/stable-pair results, fixed
`expected_configuration_word`, `expected_read_word`, `expected_read_selector`,
scope and sanitized blocker codes. Expected constants remain explicitly labeled
even on refusal. Raw RAM, pointers, identifiers, observed chip ID/capacity and
fingerprints are omitted from report JSON and Debug.

Every report keeps historical/unverified, missing continuity/age, unresolved
ROM/start stability and physical chip/read/return blockers. Hardware permit,
hardware access, real entry/read, flash write, physical proof and freshness
flags are **constant false**, including positive conditional/stable results.
`status` advertises only `saved_loader_metadata_assessment_available=true` and
scope `conditional-unverified-saved-fixed-fd07-mode2-only`.

## Independent validation

`native/fixtures/loader-metadata-v1-vectors.json` contains **39 original synthetic
84-byte vectors**. Its guarded stdlib oracle `host/native_metadata_fixtures.py`
imports no product parser/codec, captured device bytes or vendor firmware.
Four Python tests regenerate exact committed bytes and verify lengths, hashes,
literal little-endian fields and deliberate stability exclusions.

Rust conformance covers supported baselines/density endpoints; invalid full ID,
density, capacity, mode, selected pointer and word/byte flags; changed private
linkage; counter wrap/online/opaque-byte exclusions; exact schema/phase/digest/
length/allocation bounds; binding/fingerprint drift; privacy and all false flags.
The reused-ID case rebinds expectations consistently and proves the final
observation passes alone before pair rejection. Independent R1 mode-4/mode-3
vectors prove that self-consistent metadata cannot approve an unsupported start;
a positive selected-B case checks linkage to `0x01C08E00` as well as A.

Reproduce the focused new checks through the canonical runner:

```sh
rtk proxy python3.12 native/run_tests.py --test loader_metadata_conformance
rtk proxy python3.12 native/run_tests.py --release --test loader_metadata_conformance
rtk proxy python3.12 native/run_tests.py --test startup_context_conformance
rtk proxy python3.12 host/run_tests.py
rtk proxy cargo fmt --manifest-path native/Cargo.toml -- --check
```

- All **nine native integration targets passed: 55 tests**, unfiltered; this
  includes all **7 R2 tests** and the retained 6 R1 tests. Three selected library
  tests for status, reference-contract discovery and operation refusal passed;
  each intentionally filtered the other 164 library tests.
- Focused optimized suites passed **13 tests: 7 R2 and 6 R1**, unfiltered.
  Every native run used source/API/dependency audit, private guarded scratch,
  offline locked Cargo and unchanged per-run input fingerprints.
- The complete canonical guarded Python suite passed **314 tests in 198.429 s**,
  no skips; this includes four new exact-fixture regeneration tests.
- Source audit passed for **44 Rust files**, without new dependencies or policy
  exceptions. Formatting, relative documentation links and diff checks passed.
  The default offline release build and actual default CLI status/help/capture-
  refusal checks passed, retaining all false physical flags. Optional hardware
  backends were not executed. No additional OS/target or hosted CI run is claimed.
- Independent final static review found no actionable source/contract/test/doc
  defects. The retained exact-loader listing was rehashed as
  `[digest omitted from public archive]`.
  Static decoding/review is not target execution or a whole-system effects proof.

The unchanged full Rust writer/persistence fault matrix was not repeated in this
increment: R2 changes an isolated byte-only module plus dispatch discovery and
does not alter R1, adapters, storage or writer implementations. The prior full
debug result at `dfd50d1` is historical evidence in report 35, not a newly run
full-suite claim for R2.

Ignored logs/receipts/review live under `native/target/`. The **67-file** map
`fm1-r2-source-map.json` covers audited native source/fixtures and both
independent host fixture oracles/tests. Canonical sorted compact JSON SHA-256:
`[digest omitted from public archive]`.
Regression log `fm1-r2-native-regression.log` SHA-256 is
`[digest omitted from public archive]`;
host log `fm1-r2-host-full.log` SHA-256 is
`[digest omitted from public archive]`.
Default build/status/refusal receipts and `fm1-r2-contract-review.md` are retained
there as well. Work is local on `init`; no push or physical acceptance is claimed.

## Next step and physical backup gate

**R3 is next:** finite serialized fake checkpoint/storage scheduling, with
distinct metadata/checkpoint/full-pass purposes and budgets. R4 then needs actual
ROM/pre-entry stability, exact chip/read/addressing, macOS access, native-to-
storage provenance, timing and original-stock return evidence before activation.
The owner's read-only backup permission is already present; another permission
question does not replace those technical requirements. R2 has not captured
the owner's RAM or flash and does not make a physical backup possible yet.
