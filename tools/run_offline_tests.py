#!/usr/bin/env python3
"""Compile file-only Rust tests without Cargo dependencies, then prove the guard.

No devices, native adapters, firmware builds, or target execution are involved.
The optional Cargo pass requires an already populated locked dependency cache.
"""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys


GUARD_TESTS = (
    "every_forbidden_operation_is_refused",
    "actual_inventory_entrypoint_fails_loudly",
    "actual_open_entrypoint_fails_before_using_path",
    "actual_command_entrypoint_preserves_the_caller_buffer",
)


def reject_overrides(environment):
    # The runner owns the compiler configuration and never forwards extra flags.
    names = {
        "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS",
        "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTDOC",
        "CARGO_BUILD_TARGET", "CARGO_BUILD_RUSTC", "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
    }
    blocked = sorted(name for name, value in environment.items() if value and (
        name in names or (name.startswith("CARGO_TARGET_") and name.endswith("_RUSTFLAGS"))
    ))
    if blocked:
        raise ValueError("compiler/target overrides are forbidden: " + ", ".join(blocked))


def run(arguments, root, capture=False):
    print("+ " + " ".join(str(argument) for argument in arguments), flush=True)
    return subprocess.run(arguments, cwd=root, check=True, text=True, timeout=1200,
                          stdout=subprocess.PIPE if capture else None)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cargo", action="store_true",
                        help="also run the full guarded suite with cargo --offline --locked")
    options = parser.parse_args(argv)
    try:
        reject_overrides(os.environ)
    except ValueError as error:
        parser.error(str(error))

    root = Path(__file__).resolve().parent.parent
    output = root / "target" / "offline-tests"
    output.mkdir(parents=True, exist_ok=True)
    harness = output / "harness.rs"
    modules = {name: root / "src" / (name + ".rs")
               for name in ("scsi", "formats", "boards", "sectormap", "jl")}
    modules["offline_formats"] = root / "tests" / "support" / "offline_formats.rs"
    # JSON string quoting is also valid for these Rust path string literals.
    source = "#![forbid(unsafe_code)]\n#![allow(dead_code, unused_imports)]\n"
    for name, path in modules.items():
        source += "#[path = " + json.dumps(path.as_posix(), ensure_ascii=False) + "]\nmod " + name + ";\n"
    harness.write_text(source, encoding="utf-8")
    executable = output / ("offline-tests.exe" if os.name == "nt" else "offline-tests")
    # Also prove feature-only builds exclude native code without cfg(test).
    run(["rustc", "--edition=2024", "--crate-type=lib", "--forbid=unsafe_code",
         "--allow=dead_code", "--allow=unused_imports", "--cfg", 'feature="offline-tests"',
         str(root / "src" / "scsi.rs"), "-o", str(output / "offline-scsi.rlib")], root)
    run(["rustc", "--edition=2024", "--test", "--cfg", 'feature="offline-tests"',
         str(harness), "-o", str(executable)], root)
    listed = run([str(executable), "--list"], root, capture=True).stdout
    for name in GUARD_TESTS:
        if "offline_guard_tests::" + name + ": test" not in listed:
            raise RuntimeError("required guard test missing: " + name)
    # Prove the deliberate forbidden calls before running any shared test.
    run([str(executable), "offline_guard_tests", "--nocapture"], root)
    run([str(executable), "--skip", "offline_guard_tests"], root)
    print("Private firmware corpus: not exercised by the std-only harness; "
          "optional Cargo corpus tests are explicitly ignored.", flush=True)
    if options.cargo:
        # No download, driver/bootstrap installation or dependency substitution.
        # cfg(test)/offline-tests compiles all project native adapters out.
        run(["cargo", "test", "--offline", "--locked", "--no-default-features",
             "--features", "offline-tests"], root)
        # The test-only feature must never produce a distributable application
        # with a denying adapter, including through cargo build --all-features.
        print("+ checking that offline-tests refuses a production build", flush=True)
        refused = subprocess.run(
            ["cargo", "check", "--offline", "--locked", "--no-default-features",
             "--features", "offline-tests"], cwd=root, text=True,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=1200)
        if refused.returncode == 0 or "offline-tests is test-only" not in refused.stderr:
            print(refused.stderr, file=sys.stderr)
            raise RuntimeError("production build did not report the test-only feature refusal")
        print("Production build correctly refuses the test-only feature.", flush=True)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, subprocess.CalledProcessError, subprocess.TimeoutExpired, RuntimeError) as error:
        print("offline tests failed: " + str(error), file=sys.stderr)
        sys.exit(1)
