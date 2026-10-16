> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Native Rust foundation evidence

Date: 2026-10-13. Ticket: [FM1-027](04-backlog.md#fm1-027--small-native-rust-cli-foundation).
Owner decision: Rust for the distributed CLI, Python retained as the offline
reference. This evidence covers the first native slice only.

## Implemented boundary

The separate Cargo package (historical reference `../native/README.md`; omitted from public archive) has zero third-party
dependencies and forbids unsafe Rust. The executable reads command arguments
and writes standard output/error. It has no filesystem/device API, native
transport, FFI, firmware build, Python embedding or child-process invocation.

Commands: help, version, `status [--json]` and `agent contract --json`.
The contract command embeds the pinned Python reference inside a versioned
native envelope labelled `discovery-only-not-native-lifecycle`. The native
lifecycle and all hardware/physical/CPU capabilities remain false. Unknown,
malformed and destructive commands return sanitized JSON refusal with exit 2;
caller paths and authority-shaped arguments are never opened or echoed.

Python's parser, planner, model writer and agent lifecycle are **not ported**.
[FM1-028–032](04-backlog.md#fm1-028--rust-package-parser-and-wire-codecs) scope
the successive native ports, independent conformance checks and release work.
Browser tooling remains step 2; Rust does not by itself establish browser
transport support. No hardware access or flash write occurred.

## Actual local artifact

| Item | Observed evidence |
|---|---|
| Toolchain | Homebrew rustc 1.99.0 (`b940084d7`), Cargo 1.99.0 (`5f94df478`) |
| Target | Mach-O 64-bit macOS arm64 executable |
| Build | `cargo build --manifest-path native/Cargo.toml --release --offline --locked` |
| Profile | `opt-level="z"`, strip, LTO, one codegen unit, overflow checks retained |
| Uncompressed bytes | **302,784** (295.69 KiB) |
| SHA256 | `[digest omitted from public archive]` |
| Linked OS library | `/usr/lib/libSystem.B.dylib`, compatibility 1.0.0, current 1359.0.0 |
| Artifact path | Ignored `[local artifact location omitted]` |

The local executable ran version/status/contract discovery without Python.
An actual `flash` invocation with an inert device-shaped argument returned
refusal; no path operation exists in dispatch. Pure tests also cover private
paths, injected text and force/authority-shaped arguments.

This measured artifact used the reviewed local FM1-027 source before its commit
and is unsigned development output. It is not a published release or a
cross-platform reproducibility result. No startup benchmark, minimum OS,
Intel Mac, Linux or Windows runtime result is inferred from this local build.
The complete flasher will contain more code; this size is not its promised size.
The Python wheel excludes its interpreter and is not a comparable native size.

## Verification and CI

- `cargo fmt --manifest-path native/Cargo.toml -- --check` passed.
- Five pure Rust dispatch tests passed with offline, locked Cargo. They check
  exact reference embedding, false capabilities, typed command refusals,
  sanitized malformed inputs and honest help/version output.
- One focused Python 3.11+ test passed through the canonical runner with its
  no-hardware guard installed. It parses the native fixture and compares it
  with the live FM1-024 reference contract. The preceding full reference suite
  passed **274 tests**; that full result predates this single added parity test.
  At source commit `259293e`, [hosted run 38425822966](https://github.com/kmplngj/fm1-flasher/actions/runs/38425822966)
  subsequently passed all 12 Linux/macOS/Windows × Python 3.11–3.14 cells.
  Inspected 3.11 logs show 275 tests on each OS, with 100 platform skips on
  Windows and none on Linux/macOS. These host/model checks do not qualify
  Windows secure persistence or physical-device behavior.
- A GPT-6.1 Sol high subagent independently reviewed source/CI/contract
  boundaries and ran the Rust checks. A Windows multi-command step was split
  so a failed format check cannot be hidden by a passing test command.
- At source commit `259293eaa7fbe94b9c96951e98739363debdc16c`,
  [native hosted run 38425828449](https://github.com/kmplngj/fm1-flasher/actions/runs/38425828449)
  completed successfully on all three Linux/macOS/Windows runners: format,
  five pure tests, offline locked release build and actual version/status/contract
  smoke. Recorded rustc/Cargo versions were 1.99.0 on Linux and 1.98.1 on
  macOS/Windows. These software results establish neither a minimum toolchain
  nor physical transport support. Later hosted evidence belongs in
  [progress](05-implementation-progress.md).

The owner's only FM-1 remains excluded. Native local IO, adapters, full core
parity, release signing and physical qualification remain separately scoped
work under the [safety policy](03-single-device-safety.md).
