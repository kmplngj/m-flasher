> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Offline full-raw file CLI — FM1-033-F1

Date: **2026-10-16**. Branch: `init`. This implements the offline product slice
from [contract 43](43-full-raw-dumper-product.md). GPT 6.1 Sol high subagents
implemented the raw-file core and actual-executable checks/review; a medium
subagent wrote independent conformance vectors. Root integrated dispatch,
guarded runners/CI, documentation and validation. No network, hardware inventory,
MIDI/USB open/query, target execution, vendor script, firmware build, backend
installation or flash write occurred. All test files are fictional.

## Available commands

```text
fm1-flasher dump inspect --file ABSOLUTE_PATH --capacity BYTES [--json]
fm1-flasher dump verify --file ABSOLUTE_PATH --capacity BYTES --sha256 HEX [--second-file ABSOLUTE_PATH] [--json]
```

Both commands treat the complete file as opaque stored flash bytes. They hash
the head, ciphertext, configuration/data and tail without decoding, normalizing,
extracting an APP or writing any bytes. Unknown or corrupt application content
does not prevent raw inspection. No `.fwsc`, preset codec, catalog transition or
installation target is required.

`--capacity` must be explicit canonical decimal, a power of two from 128 KiB to
16 MiB, matching the existing chip-expectation arithmetic range. It is
**caller-declared**, not a chip-identification result or evidence that the file
is complete relative to a particular device. Inspection requires exact length
and computes SHA-256. Verification additionally requires a 64-digit hexadecimal
expected digest, accepts either letter case and compares the complete bytes of
an optional second accepted file. The files must have distinct identities.

Arguments can be ordered differently but must be complete, unique and applicable
to the chosen command. Unknown/duplicate flags, noncanonical numbers, unsupported
capacity, malformed hashes, relative/network/device/pseudo-file/control-character
paths and live/write overrides refuse. The full argument list is validated
before any executable file IO. Non-UTF-8 CLI arguments retain the existing
sanitized refusal behavior.

`dump capture [--json]` returns `full-raw-capture-unavailable`, exit 2. Supplying
an output path cannot activate it or create a file. It does not construct a
hardware adapter, execute a loader or touch the FM-1. The existing backup entry
assessor, identity-only feature and native hardware-factory refusal remain intact.

## Implementation and file boundary

- `native/src/raw_dumps.rs` adds byte-only `inspect_bytes`/`verify_bytes`, shared
  capacity validation and `inspect_file`/`verify_file`. It reuses `digest` and the
  existing crate-private `local_files::Directory`; it has no direct filesystem,
  syscall, transport, network or process API. Acquisition, secure capture and
  two-copy persistence remain in the existing modules; no second read engine,
  fake capture facade, writer or new backup format was introduced.
- `lib.rs` supplies strict pure command parsing and result rendering. Pure
  library `dispatch` never opens an input file; validated file commands return
  `raw-dump-file-io-executable-only` there. Only `main.rs` connects a fully
  validated intent to the reviewed local-file functions. Help/status describe
  saved-file operations separately from unavailable capture.
- Existing IO requires absolute paths, safe descriptor-relative ancestry,
  owner-private containing directories, privately owned single-link regular
  files and pre-open type/size checks. Symlinks, special files, hard links and
  unsafe permissions refuse. Read-only/nonblocking/nofollow descriptor flags,
  before/opened/after metadata comparisons and retained directory ancestry are
  reused unchanged. These are reviewed regression boundaries, not an OS sandbox
  against arbitrary malicious code or a real-time filesystem guarantee.
  The existing `/Volumes`/device/pseudo-file path exclusions remain in force;
  this slice does not qualify external-volume access or media independence.
- The first input's returned identity is retained and checked around reading
  the optional second input and after full hash/byte comparison. Both inputs
  remain unchanged. Windows/other unsupported secure-file platforms refuse
  through the existing storage stub; pure byte APIs remain portable.

## Versioned output and authority limits

JSON schema 1 uses `group=full-raw`, `mode=full-raw`,
`scope=saved-historical-only`, byte count, declared capacity, computed SHA-256,
`expected_hash_verified` and `second_input_compared`. These digest reports are
local/private user outputs; they omit paths, contents and device/session IDs,
but intentionally include the requested snapshot digest. They are not sanitized
public support reports.

All physical/current authority fields remain false: actual capacity identified,
fresh device passes, independent media, physical durability, final continuity,
stock return, bootability, hardware read/evidence/raw verification, capture,
physical writes and current writer eligibility. Equal saved files cannot stand
in for two fresh device reads, an accepted live capture, independent storage or
restore qualification. User-supplied files/reports cannot mint such authority.

Exit 0 means the requested saved-file checks passed. Exit 2 covers argument,
policy, unsupported-platform and IO refusal; an oversized file can fail at the
pre-open size-policy check. Exit 3 covers processed size/hash/second-file
mismatch. The executable's existing output-write failure also returns 3.
Failure codes are static and do not echo file paths, bytes or supplied hashes.

## Validation and delivery evidence

The sanitized evidence manifest (historical reference `evidence/44-offline-raw-dump-cli.json`; omitted from public archive) binds
source and retained-log hashes, test results and the actual default executable.
Checks use the canonical source-auditing runner with `FM1_NATIVE_TEST_ROOT`
scratch confinement; no test enables the MIDI backend or imports hardware code.

- **8 independent conformance tests passed in debug and optimized builds.**
  An independently computed SHA-256 literal covers a whole opaque fixture;
  mutations at the head, application-like bytes, data and last byte all fail.
  Truncation/padding, capacity boundaries, malformed/uppercase hashes, strict
  CLI flags/paths, pure dispatch and historical authority are covered.
- **3 new secure-file unit tests passed in debug and optimized builds.**
  They cover distinct files, same-file refusal, unsafe files/permissions and
  deterministic regular-file replacement before open or during second-input
  opening. They reuse existing scratch-only IO helpers and mutation hooks.
- **7 existing root dispatch tests and all 11 integration targets passed in
  debug: 73 integration tests total**, including the new 8. Unchanged expensive
  Rust writer/persistence library sweeps were not rerun. Existing capture/file
  implementations and their previous evidence remain separate.
- **44 checks passed against the actual default release executable** through
  `native/run_tests.py --release --check-raw-dump-cli`. Independent Python
  `hashlib` checks cover complete 128 KiB and 1 MiB files, corruption at three
  physical regions, fresh file reads, optional copy comparison, truncation,
  same-file/hardlink/symlink/ancestor/FIFO/permission refusals, privacy and
  unavailable capture. This is saved-file evidence, not physical flash capture.
  The same 44 checks also passed with an explicitly redirected Cargo target
  directory and build target. Review caught and corrected a hardcoded executable
  path in the initial runner: it now selects the unique current Cargo JSON
  artifact, requires empty features/non-test/current source and confined output,
  and refuses missing, foreign, feature-enabled or ambiguous artifacts before
  execution. Pure host regressions cover those provenance failures. Final
  independent review confirmed the finding resolved.
- **318 guarded Python host tests passed in 230.923 seconds**, with no skips;
  the final result is retained in the evidence manifest. The native API/source
  audit covers **50 Rust files**.
  Existing mutation guards now exercise the new module's prohibition on direct
  filesystem/hardware/process APIs as well. Formatting and local link/diff
  checks apply to this increment.
- The optional macOS MIDI-identity executable **typechecked** in release mode
  without execution. Default features, dependency/lock/policy pins, firmware,
  PIO, submodules and license notices are unchanged. Four existing core warnings
  remain in unrelated atomic/fake-writer code; no new module warning was added.

The local empty-feature macOS arm64 release executable is **369,408 bytes**,
SHA-256 `[digest omitted from public archive]`,
built with Rust 1.99.0 (`b940084d7`) for `aarch64-apple-darwin`. `otool -L` lists
only system libiconv and libSystem; it neither embeds nor launches Python and
contains no active MIDI feature. This is the measured F1 artifact, not a final
all-platform flasher-size or signed/notarized distribution claim.

The workflow now schedules actual empty-feature file-CLI checks on its existing
OS matrix before the optional macOS identity build. This change is local; no
new Linux/Windows runtime, hosted CI, release, push or hardware result is claimed.
Windows file cases are explicitly skipped/refused while discovery and capture
refusal remain testable.

## Completion boundary

**FM1-033-F1 is implemented offline.** R4-C still qualifies exact stock entry,
ROM/helper effects, read path and macOS lifecycle. E remains the first two-fresh-
pass/two-copy physical acceptance, and F2 remains the standalone qualified-route
release. No additional F1 polish prerequisite is imposed on the first qualified
backup. Musical/selective exports remain FM1-023; browser access remains step 2.
No owner's RAM/flash was read, no full hardware backup was created, and the
factory/entry/loader/physical-write gates remain closed.
