> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Offline platform packaging and software evidence

Date: 2026-10-11. FM1-016 delivers an unsigned development wheel with no hardware
dependencies. It does not qualify a physical adapter, operating-system release
floor, standalone native executable, owner-device operation or CLI write release.
Step 2 browser work still waits for FM1-018 release qualification.

## Artifact contract

The root `pyproject.toml` pins `setuptools==80.9.0`, Python `>=3.11`, an empty
runtime dependency list and the single entry point
`fm1-flasher = fm1flasher.cli:main`. The wheel contains only the exact
`host/src/fm1flasher/*.py` snapshot, distribution metadata, root MIT `LICENSE`
and retained `host/THIRD_PARTY_NOTICES.md`. It includes fictional fixtures and
metadata references, not vendor packages, loader payloads, firmware builds,
native libraries or native MIDI/serial/SCSI/USB dependencies.

`host/build_wheel.py` builds twice from separate clean temporary source trees.
It copies only the fixed local host input set, normalizes source file mode and
mtime, fixes `SOURCE_DATE_EPOCH=1760140800`, checks the preinstalled backend
version and calls `setuptools.build_meta.build_wheel` directly. Build-time audit
hooks reject networking, native FFI, subprocess execution and device-path access.
This narrow packaging boundary is separate from the unchanged stronger host
test guard; it does not import the host product or install any dependency.

`host/wheel_evidence.py` requires an exact archive-member allowlist and compares
every source/notice byte. It checks Python/license/pure-wheel metadata, empty
`Requires-Dist`, the console entry point and every `RECORD` hash and size.
Metadata header names are case-insensitive; alternate casing cannot conceal a
dependency or duplicate singleton. Unexpected native/vendor files, source/notice
replacement, malformed records and duplicate entries fail inspection.

Equal complete wheel hashes demonstrate repeatability for the measured source
snapshot, pinned backend, Python and local environment. They do not promise
cross-Python/cross-OS byte equality. Setuptools' pinned wheel implementation
[uses the epoch for ZIP timestamps and sorts packaged paths](https://github.com/pypa/setuptools/blob/v80.9.0/setuptools/_vendor/wheel/wheelfile.py).
The tag `py3-none-any` describes this artifact's Python/ABI/platform encoding;
it is not evidence that native device access or every file operation works on
every OS. The backend's incidental `bdist` staging-directory name is not an OS
minimum-version claim.

## Reproduce without implicit downloads

Run from the repository root using Python 3.11+. Select unused environment and
output directory names: these examples intentionally do not replace existing
qualification outputs. No administrator rights or driver changes are needed.
Do not run legacy firmware tools or initialize/build firmware submodules.

Explicit preparation may fetch **only the pinned build-tool wheel**. If offline
already, obtain that wheel through a separately prepared local wheelhouse and
omit the `pip download` command. Python must already provide `venv`/`ensurepip`;
there is no automatic runtime provisioning. No global installation is needed.

```sh
rtk proxy python -m venv host/.packaging/build-env
rtk proxy host/.packaging/build-env/bin/python -m pip download --only-binary=:all: --no-deps --dest host/.packaging/wheelhouse setuptools==80.9.0
rtk proxy host/.packaging/build-env/bin/python -m pip install --no-index --no-deps --find-links host/.packaging/wheelhouse setuptools==80.9.0
rtk git rev-parse HEAD
```

Use the printed full commit hash explicitly; it is a declared revision label,
not a claim that the checkout is clean. The subsequent build uses the already
prepared backend and blocks network/process/native access. The report fingerprints
every actual wheel input, plus build/check/guard tooling separately, so uncommitted
inputs remain auditable without circularly hashing the report itself.

```sh
rtk proxy host/.packaging/build-env/bin/python host/build_wheel.py --source-revision FULL_40_CHARACTER_COMMIT_HASH --output qualification
rtk proxy python -m venv host/.packaging/install-env
rtk proxy host/.packaging/install-env/bin/python -m pip install --no-index --no-deps host/dist/qualification/build-1/fm1flasher-0.1.0.dev0-py3-none-any.whl
rtk proxy host/.packaging/install-env/bin/fm1-flasher --version
rtk proxy host/.packaging/install-env/bin/fm1-flasher status
rtk proxy host/.packaging/install-env/bin/python host/wheel_smoke.py
rtk proxy python host/run_tests.py
```

Windows uses the venv `Scripts/python.exe` and `Scripts/fm1-flasher.exe` paths.
These Windows commands are a configured recipe, not a reported local Windows
execution. Installing a prepared wheel requires no setuptools download or build
frontend. The documented pip options are supported by the
[official pip installation documentation](https://pip.pypa.io/en/stable/cli/pip_install/).
The SPDX license fields use the
[setuptools >=77 metadata support](https://github.com/pypa/setuptools/blob/main/docs/userguide/license_migration.rst),
provided by the explicitly pinned 80.9.0 backend.

Outputs under ignored `host/dist/<name>/` are two wheels, `SHA256SUMS` and
`provenance.json`. Provenance records SHA256/size/member hashes, source-input
fingerprint, separate tooling hashes, declared Git revision, actual backend,
Python, OS/version and architecture. It explicitly records absent adapters,
hardware access/writes false, unsigned/not notarized/not release-qualified.
Artifacts are local development evidence; they are not committed binaries or a
published release.

## Platform and capability evidence

The actual local host is **macOS 27.0.1, arm64**. Python `>=3.11` is the declared
runtime floor; actual local interpreter results below establish only the listed
patch versions. No minimum supported macOS/Linux/Windows version is established.

| OS/architecture | Guarded source suite | Wheel/install/model smoke | Native-device evidence |
|---|---|---|---|
| macOS 27.0.1 arm64 | Local Python 3.11–3.14 results recorded below | Two clean builds and isolated installation; actual results below | None; all adapters absent |
| macOS x86-64 | Not run locally | Same pure wheel can be distributed, execution not established | None; disabled |
| Linux | Not run locally; CI configured | CI build/install/mock recipe configured, hosted outcome not established | None; disabled |
| Windows | Not run locally; CI configured | CI pure model smoke configured; secure local file IO and durable-copy operations remain gated | None; disabled |

The saved macOS/Linux/Windows serial fixtures cover explicit candidate names,
dual CDC data/console selection, busy/permission failures, fragmented CRC reads,
locks, deadlines and reconnect invalidation. Pure MIDI fixtures cover exact
pairing, SysEx framing/reception behavior, queue errors and stale reconnects.
These fixtures are replayed on the actual host; their platform labels do not
mean the named OS or native backend was used. Actual Windows runs skip the
POSIX secure persistence tests. Native SysEx buffers, physical locks/permissions,
serial inventory and physical storage durability remain unqualified.

`host/wheel_smoke.py` installs the existing no-hardware guard before importing the
installed package, rejects a `host/src` source fallback, checks installed metadata
and exercises default USB-MIDI refusal, an explicit fictional bridge plan and
in-memory simulation. Model verification stays distinct from hardware readback.
POSIX-capable hosts also check sanitized saved-report reads; Windows file IO is
reported as gated. Only `--version`/`status` console-launcher smokes run outside
the guard; neither has a transport factory or discovery side effect.

The workflow is **configured** for `ubuntu-latest`, `macos-latest` and
`windows-latest` with Python 3.11–3.14. Each job uses separate build/install venvs,
an explicit pinned preparation phase, offline wheel builds/installation and the
canonical guarded tests. Its runner labels do not prove architecture, precise
OS version or successful hosted execution; a future run's provenance records
the actual environment. No CI runner has been used as hardware evidence here.

| Capability | Implementation/software evidence | Enabled physical capability |
|---|---|---|
| Local package inspection / exact-edge planning | Pure Python; POSIX secure-file IO locally exercised | No device access |
| Fictional NOR/bridge/MIDI simulations | Guarded injected models and saved transcripts | No real write factory |
| Serial reads/backups | Fake candidate selection/read/persistence models | No native serial adapter; physical/Windows durability gated |
| USB-MIDI normal install | Fake pairing/updater; ordinary OTA backup/checkpoint gaps refused | No native MIDI adapter or writer |
| USB ROM SCSI/BOT | Saved fictional endpoint/protocol model and feasibility report | No native adapter on any OS |
| Signing / notarization / Windows signing | Not selected or performed | Separate future release tasks |

FM1-017 requires external native-device qualification; FM1-012 needs a separately
authorized owner read session. FM1-018 remains the release gate. No signing,
notarization, native standalone build, hardware inventory, driver replacement,
firmware build or physical write occurred in FM1-016.

## Local qualification record

Both final clean builds produced `fm1flasher-0.1.0.dev0-py3-none-any.whl`,
**116,858 bytes**, SHA256
`[digest omitted from public archive]`.
The input-manifest fingerprint is
`[digest omitted from public archive]`.
The build used isolated Python 3.11.17 / setuptools 80.9.0 on macOS 27.0.1 arm64.
The separately prepared setuptools wheel has SHA256
`[digest omitted from public archive]`.

The local output directory is `host/dist/qualification-final/`, with the two
wheels in `build-1/` and `build-2/`, `SHA256SUMS` and `provenance.json`.
The declared Git revision is predecessor
`28b9a4b22da755f0fd92105d4a087dc25c9a2060`; actual uncommitted wheel inputs and
build/check/guard methods are bound by their separate hashes. This is not a
claim that the complete build method existed at that revision. The provenance
files preserve this actual build snapshot independently of a later docs commit.

The same final wheel was installed with `--no-index --no-deps` into four fresh
local venvs using Python **3.11.17, 3.12.15, 3.13.16 and 3.14.8**. Actual console
`--version`/`status` passed on each; each guarded installed-wheel smoke passed
default MIDI refusal, explicit bridge demo, verified in-memory simulation and
sanitized saved-report read. Each loaded the package from its venv's
`site-packages`, never `host/src`. The saved smoke JSON files are
`host/dist/wheel-smoke-3.11.json` through `wheel-smoke-3.14.json`.

Five focused artifact-boundary regression tests passed under the unchanged
guard in 0.004 seconds. The final canonical `host/run_tests.py` runs passed:

| Actual interpreter | Guarded source-suite outcome | Reported duration | Installed-wheel/console smoke |
|---|---|---|---|
| CPython 3.11.17 | 235 tests, PASS | 332.864 s | PASS |
| CPython 3.12.15 | 235 tests, PASS | 321.527 s | PASS |
| CPython 3.13.16 | 235 tests, PASS | 323.380 s | PASS |
| CPython 3.14.8 | 235 tests, PASS | 327.538 s | PASS |

These independent local runs overlapped; timings are observations, not a
performance comparison between Python releases. Their saved logs are
`host/dist/matrix-python-3.11.log` through `matrix-python-3.14.log`.
`git diff --check` passed. Linux/Windows/macOS x86-64 native execution remains
unproven; no configured hosted workflow has been claimed as a successful run.
