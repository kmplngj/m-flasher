> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# CLI distribution options

Date: 2026-10-13. Decision aid for the owner's goal of shipping one small CLI
binary when possible. The comparison informed the subsequently approved Rust
host implementation. No release or hardware authorization follows from it.
The first native executable measurement is in [report 12](12-native-foundation.md);
no Python-freeze comparison or startup benchmark has been performed.

Owner decision, 2026-10-13: **Rust is selected for the shipped CLI.** Keep Python
as the offline reference; [FM1-027](04-backlog.md#fm1-027--small-native-rust-cli-foundation)
starts the native foundation and measurement. This authorizes host implementation,
not device access or real flashing. The comparison below explains the decision;
current native artifact evidence is recorded in [report 12](12-native-foundation.md).

## Current implementation

The host CLI is Python >=3.11 with no third-party runtime dependencies. Its
current distribution is a setuptools-built development wheel, requiring an
installed Python interpreter. The existing [packaging evidence](07-platform-packaging.md)
does not establish a standalone executable or enabled physical adapter.

Python is a useful reference implementation for package parsing, byte-exact
protocol fixtures, policy checks and fault simulation. Those contracts and
independent expected results can survive a later implementation in another
language. They are valuable even if Python is not the final distributed runtime.

## Options

| Option | One downloadable executable | Compact native CLI fit | Main cost |
|---|---|---|---|
| Python with PyInstaller | Yes, per target platform | Includes Python runtime and dependencies | Packaging/runtime footprint, startup extraction and per-platform builds |
| Python with Nuitka | Standalone/onefile modes supported | Must measure; compilation alone is no size guarantee | CPython-coupled build, C compiler and dependency packaging |
| Rust implementation | Native executable per target | Strong candidate when footprint is a priority | Port and revalidate the core; select/qualify OS transport libraries |
| Go implementation | Compiled executable per target | Plausible alternative; actual size needs measurement | Port and revalidate the core; assess C/native transport dependencies |

[PyInstaller's operating-mode documentation](https://pyinstaller.org/en/stable/operating-mode.html)
states that it bundles the active interpreter and imported modules. Onefile
launch extracts support files into a temporary directory and starts more slowly
than one-folder mode. Its outputs are platform-specific; users do not need an
installed Python. Therefore, one artifact is achievable, but it is still a
Python runtime bundle rather than a small standalone implementation of the core.

[Nuitka's manual](https://nuitka.net/user-documentation/user-manual.html) describes
standalone/onefile modes that do not require the user's Python installation,
requires a C compiler, and notes its close coupling to CPython. It is worth
evaluating if retaining Python is important; this research has not established
that its result meets a compact-binary requirement.

[Cargo's build documentation](https://doc.rust-lang.org/cargo/commands/cargo-build.html)
supports target-specific release builds. [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
offer size optimization, stripping and LTO; the documentation explicitly advises
measurement because size optimizations do not guarantee the smallest result.
Rust does not need to bundle CPython when the core is implemented in Rust.
Embedding the existing Python core inside a Rust launcher would retain its
runtime burden and would not deliver the main footprint benefit.

[Go's compilation tutorial](https://go.dev/doc/tutorial/compile-install) shows
compiling a program and its dependencies into an executable. Go is another
reasonable native CLI candidate. [Go's cgo documentation](https://pkg.go.dev/cmd/cgo)
explains that calling C libraries introduces C toolchain and cross-compilation
considerations; future transport choices must be evaluated before claiming
simple cross-platform builds or absence of external dependencies.

## Selected direction and remaining measurement

Keep the tested Python implementation as the offline reference and implement
the distributed CLI in Rust before introducing real adapters. The owner selected
this direction. It remains an engineering choice, not a measured Rust-versus-Go
or complete-flasher size result.

A bounded future offline comparison should first set target OS/architectures,
acceptable artifact size, startup time and allowed runtime dependencies. Compare
an existing Python freeze against one native vertical slice using the same saved
fixtures, JSON schemas, typed failures and CLI exit codes. Record uncompressed
artifact size, cold/warm launch behavior, dependencies, provenance and contract
parity. Do not replace the safety suite with output-only smoke tests.

Every option still needs per-OS qualification, signing/release decisions and
reviewed native MIDI/serial/USB integrations. A compiled binary does not qualify
hardware behavior. All physical adapters, firmware builds, device access and
flash writes remain outside this research under the [single-device policy](03-single-device-safety.md).
