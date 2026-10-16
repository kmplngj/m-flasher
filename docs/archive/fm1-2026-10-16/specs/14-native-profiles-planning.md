> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Native profiles and transition planning evidence

Date: 2026-10-13. Ticket: [FM1-029](04-backlog.md#fm1-029--rust-profiles-and-transition-planner).
All inputs and outcomes are offline or fictional. No hardware access, inventory,
probe, firmware build, backend install or flash write occurred.

## Implemented boundary

The zero-dependency Rust library now has strict immutable Source, Evidence,
Artifact, Hardware, Profile, Qualification, PhysicalEffects, Transition and
Catalog records. Exact field/type/nullability checks, cross-references, evidence
namespaces, OS/version/transport cells and independent aligned/protected range
constraints match the Python reference. Unknown/duplicate fields and claimed
write capabilities refuse. Initial catalog metadata is pinned; vendor images
and loaders remain absent. Identity resolution returns a candidate only and
requires exact hardware plus INFO/full-snapshot evidence, with ambiguity refused.

The planner binds original package, full equal reads, INFO, exact shared loader,
device/session and explicitly fictional backup/recovery receipts. It re-extracts
and rehashes original bytes, checks target head against the original snapshot,
proves the source APP, and calculates the full expected image. Differing sectors
must fit the edge/profile and fixed `[0x4000,0x93000)` gate; head/data/tail and
unchanged sectors remain preserved. Target package data is never copied silently.

Accepted plans remain `scope=simulation-only` and never stage an OTA loader or
enable writing. `check_plan` and `expected_image` fully rebuild against current
original inputs. Structurally valid saved plan JSON or a matching plan hash cannot
grant authority, survive changed bindings or issue physical capabilities.

Direct resolution defaults to USB-MIDI, selects exactly one chosen edge and
refuses ambiguity, unknown layouts/data, equal MIDI identities and missing
ordinary OTA safety evidence. Transporter requires an explicit route. There is
no graph traversal, identity bump, firmware detour, fallback or automatic resume.
The complete hypothetical MIDI model is a separate port; ordinary MIDI remains
gated here.

Historical rollback planning checks the original source package and prior
snapshot/copy bytes, exact known-good fictional APP, a fresh current full snapshot
and receipts, a distinct current session/copy binding and an exact reverse edge.
Every recheck rebuilds that reverse plan. Only old APP bytes may restore; current
head and data remain preserved. These are **injected fictional copy bytes**,
not verified native files, independent media or physical storage durability.
Those gates and the fake writer belong to FM1-030. The reports explicitly keep
native storage durability and physical backup acceptance false.

## JSON and privacy

The byte-only JSON reader rejects duplicate decoded keys (including escaped
aliases), nonfinite syntax, invalid UTF-16 surrogates, trailing data and scalar
confusion. Supported records require integer/boolean/string types exactly;
integers must fit i64, and valid Unicode scalars must fit Rust strings. These
bounded native limits are stricter than Python's unlimited integers/surrogate
strings; no broader compatibility is claimed.

Limits are **32 MiB input, depth 64 and 100,000 values**. A complete standard
fictional rollback input measured **23,097,126 bytes** and exceeded the original
16 MiB prototype cap. The finite bound was raised, `RollbackInputs::from_json`
added and the complete public JSON roundtrip retained as a test. Runtime file,
device, process, network and FFI operations remain absent.

Canonical supported-schema bytes match Python: sorted keys, compact separators,
ASCII/UTF-16 escapes and one trailing LF. Saved plan/catalog/request bindings
therefore use the same digest preimages. The serializer is not a qualification
or authorization mechanism.

Independent review reproduced raw bytes/session IDs in derived debug formatters,
arbitrary unknown routes in reports and catalog qualification text in refusal
diagnostics. Requests/evidence/direct/rollback types now use field-free Debug;
unsupported routes and qualification diagnostics are fixed/sanitized. Retained
sentinel tests check private text and decimal byte dumps are absent, including
nested wrappers. Plans expose defined catalog/OS/operation metadata and hash
private device/session/copy/INFO inputs; they are not raw input dumps.

## Independent fixtures and validation

| Saved fixture | Cases | SHA256 |
|---|---:|---|
| Initial catalog | Pinned reference metadata | `[digest omitted from public archive]` |
| Catalog acceptance/refusal | 25 | `[digest omitted from public archive]` |
| Planner/direct matrix | 41 | `[digest omitted from public archive]` |

The planning fixture deduplicates original fictional hex blobs. Five cases
accept plans: two-sector update, explicit bridge, no-op, target-data preservation
and fresh session. An independent cursor parser and stdlib hashes assemble all
plan fields, sector/rollback changes, full expected image and canonical binding
hashes. Production planner/resolver/extractor helpers are forbidden during oracle
generation and compared only as secondary checks. Input fixture production uses
the existing fictional recipe; no vendor or owner-device bytes are included.

Root integrated and ran **57 Rust tests**: 47 unit and ten integration tests in
debug and optimized release, with release overflow checks retained. The debug
full result precedes the final capability-text update; its focused status test
and the complete optimized suite passed afterwards. All 41 case assertions and
full plan/hash comparisons pass. An initial aggregate assertion incorrectly
expected six accepted cases; it was corrected to the corpus's five.

The fixture agent's canonical guarded Python **3.12.15** run passed **286 tests
in 192.355 s, no skips**. Root inspected the final log. Five new tests verify
exact regeneration, oracle independence, schema labels, full plan/reference
parity and two-sector/no-op/fresh bindings. An independent high review also ran
the guarded suite: 286 passed in 186.547 s. Earlier unsupported Python 3.9/mock
import attempts are not final evidence; the correct runner/guard stayed intact.

GPT-6.1 Sol high subagents implemented schemas/planning and independently reviewed
the final boundaries; medium prepared fixtures. Review found no remaining
protected-range, stale-plan or authority bypass after the corrections.

Reproducible commands from the checkout root:

```sh
rtk proxy cargo fmt --manifest-path native/Cargo.toml -- --check
rtk proxy cargo test --manifest-path native/Cargo.toml --offline --locked
rtk proxy cargo test --manifest-path native/Cargo.toml --release --offline --locked
rtk proxy cargo build --manifest-path native/Cargo.toml --release --offline --locked
rtk proxy python3.12 host/run_tests.py
```

Explicit fixture generation uses the guard first (Python 3.11+):

```sh
rtk proxy python3.12 -c 'import sys; sys.path[:0] = ["host", "host/src", "host/tests"]; from offline_guard import install; install(); import native_planning_fixtures; print(native_planning_fixtures.write_saved())'
```

The actual macOS arm64 unsigned dispatch executable remains **302,784 bytes**,
SHA256 `[digest omitted from public archive]`,
with observed `libSystem.B.dylib` dependency. Actual status reports the implemented
in-memory APIs and false native storage/writer/lifecycle/physical capabilities.
Library algorithms can be discarded from the dispatch executable when unused;
this is not a complete planner/flasher footprint or release qualification.

Native durable IO/backup/writer, firmware health hooks, agent lifecycle, release
artifacts/signing, lawful real corpus and physical qualification remain separate.
CLI file/plan commands still refuse. Browser tooling remains step 2. Source
`5ec573e5d424615b7a1d765bcf6e33d4243acccd` is pushed to `init` in
[draft PR 2](https://github.com/kmplngj/fm1-flasher/pull/2). Its
[native run](https://github.com/kmplngj/fm1-flasher/actions/runs/38456708678) and
[host run](https://github.com/kmplngj/fm1-flasher/actions/runs/38456708496) were
blocked before any step started by GitHub's account payment/spending-limit
restriction. Current-source hosted platform verification remains pending;
[progress](05-implementation-progress.md) records exact run evidence. Earlier
platform runs are not a result for this port. The subsequent delivery record
changes documentation only.
