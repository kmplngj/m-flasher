> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Injected native read driver and capture integration — FM1-033-D2

Date: **2026-10-15**. Branch: `init`, based on D1 commit
`9b64c05`. This is D2's next offline integration increment. It implements
submission/session handling and secure capture from injected platform results;
actual native factories and physical qualification remain unavailable.

No hardware enumeration, inventory, open, probe, MIDI/USB command, physical
read/write, firmware build or hardware-backend installation occurred. Platform
documentation and public sources were read online. The loader was inspected as
inert data for hashing by the separate [entry/loader audit](20-stock-entry-loader-audit.md),
not executed or added to this repository.

## Result and answer about USB-C MIDI entry

**There is a source-backed USB-MIDI ROM-entry candidate for reported stock
V15:** `F0 22 24 35 7D F7`, carried by the documented eight-byte USB-MIDI
transfer `04 F0 22 24 07 35 7D F7`. It changes the running mode; it is not
a full-backup command. The `7F` OTA trigger is a different route and is excluded.
Transporter's V15-named guard checks shared VID/PID only. Its experiment used
an RP2040 USB host, so neither the exact owner firmware nor direct macOS/Linux
MIDI packet delivery is qualified by it. The installed firmware need not be
replaced merely to investigate the candidate.

[Report 20](20-stock-entry-loader-audit.md) pins the exact primary sources and
distinguishes bounded stock V13 receiver evidence from a fully qualified V15
handler/callee. It independently verifies the WL82 loader's 24,064-byte size
and digest, but leaves startup, transitive FD05 read, persistent configuration,
watchdog and return effects unresolved. REA reported an unconfigured Ghidra
provider; no fresh binary call graph or source-to-binary build was produced.
The host read-command sequence alone cannot clear that gate.

**The CLI still cannot perform a real stock-device backup or send entry MIDI.**
Its new assessment reports source evidence without granting execution:

```sh
rtk proxy [local artifact location omitted] backup entry assess --platform macos --stock-version v15 --json
```

Exit 0 means an assessment was generated; its state is always `blocked`.
Caller-reported `v15` exposes the documented candidate bytes, never authenticates
firmware or grants entry. `v13` reports the partial branch evidence with null
command fields; `v14` and `unknown` retain unqualified handlers and null fields.
Malformed/unsupported inputs refuse with sanitized output. Entry/send, loader
execution and actual capture commands still refuse with exit 2.

The owner again allowed backup hardware access while prohibiting flash writes.
Implementation and tests in this task remain offline. No renewed permission
question is required for the already authorized bounded backup once its concrete
evidence gates are satisfied. No entry/RAM execution is inferred from permission.

## Session driver

`usb_read_driver::InjectedReadDriver` owns one exact `SelectedReadTarget`, one
boxed `InjectedReadIo`, one private finite `ReadReplay` and a fresh process-local
driver lifetime. The trait has only `current_checkpoint` and `submit`. Its
checkpoint reports the exact fictional target and held-loader mode; there is no
discovery, generic CDB, mode switch, RAM upload/jump, reset, retry, keepalive,
flash write or native factory method. `InjectedIoFailure` supplies closed
sanitized provider failures without accepting diagnostic strings.

Each `step`:

1. Checks the current [local artifact location omitted] and held-loader checkpoint.
2. Derives the next immutable canonical INQUIRY/FD05 request from D1's replay.
   A private submission token binds driver lifetime and monotonically increasing
   sequence, separate from the canonical request's wire tag.
3. Submits that exact request once and checks the checkpoint after return.
4. Rejects a callback from another lifetime/sequence, then applies the existing
   Linux or macOS completion validator and finite replay rules.
5. Checks the checkpoint again before advancing the accepted request count.

An IO error/panic, old callback, platform/target drift, unknown/lost checkpoint,
wrong status/length/request, deadline or read mismatch permanently poisons the
driver. Further calls cannot retry or reset it. The final report requires all
4,097 accepted transactions: one synthetic INQUIRY and two equal full 1 MiB
sweeps in 512-byte blocks. It reports fictional byte agreement only. Driver and
callback Debug redact identifiers, tokens, diagnostics and bytes.

Tokens distinguish in-process driver instances. They are not globally unique
hardware sessions or restart/resume authority. IO results and checkpoint claims
remain caller-controlled fictional input; even perfectly consistent records
cannot prove an OS call, real device or preservation of installed flash.
Panics are converted to a fixed error; normal Rust panic-hook behavior is not
a private-diagnostics sandbox for an arbitrary external IO implementation.

The finite replay retains D1's supplied-duration bounds: 5,000 ms per
transaction and 600,000 ms aggregate. These are not independently measured or
preemptive wall-clock deadlines. A real binding must separately enforce clock,
cancellation/completion and loader-lease behavior, including expiry during disk
sync. The reported three-second loader inactivity timeout is not solved by
adding unreviewed keepalives to the read allowlist.

## Secure capture and provenance separation

`usb_backups::capture_injected_usb_backup` accepts one driver and two explicit
existing private directories. Timestamp/path/support/collision checks precede
the first read submission. Capture requires a pristine unused driver and itself
executes every transaction. Prior steps, a completed caller replay or a saved
dump cannot replace either complete read. Once capture starts, the driver is
single-use, including failures.

The existing descriptor-anchored file layer and publication order are reused:
exclusive snapshots and pending manifests, sync/reopen/hash/file-identity checks,
second manifest published before primary, and final reopened verification. A
shared private persistence helper rechecks the source before/after file
boundaries and after final verification. Storage failure invalidates the
injected driver and returns no fresh handle; partial artifacts are not cleaned
up, overwritten or resumed automatically.

The metadata parser now accepts two closed, disjoint fictional variants:

| Metadata | Scope / transport | Current verification |
|---|---|---|
| Existing schema 1 | `fictional-usb-only` / `synthetic-usb-bot` | Original concrete `BotReader` only |
| New schema 2 | `fictional-injected-platform-only` / `injected-linux-sg` or `injected-macos-bot` | Original consumed `InjectedReadDriver` only |

Schema 2 privately records exact saved descriptor, fictional OS object/session,
reconstructed target binding, driver lifetime and completed request count 4,097.
It requires first/second read numbers 1/2, exact capacity, matching transport/
platform, known schema/tool version and all physical/source/loader/writer flags
false. Unknown fields or substituted schema-1 provenance refuse. Schema-1
canonical metadata remains compatible; shared filesystem primitives are unchanged.

`UsbBackup::verify_injected_current` checks the original completed driver before
and after reopened storage verification. Changed copies or later checkpoint
loss invalidate current acceptance. `verify_current(BotReader)` rejects schema 2;
injected verification rejects schema 1. Existing locators still identify local
files without authority. `verify_saved_usb_backup` accepts either valid variant
only as historical, even with identical serialized lifetime/session fields.
Complete files left by a late post-publication failure may reopen historically,
never reconstruct current capture authority.

No conversion to `BotSnapshot`, Transporter backup, planner qualification or
writer permit exists. Reports distinguish the two fictional scopes/transports,
redact private provenance/paths/bytes, and retain false hardware, source, loader,
physical durability, independent-media and current-writer claims. Two directories
remain distinct local copies rather than proven independent physical media.
The existing `/Volumes` restriction is unchanged.

## Native binding audit: decisions still required

Context7 documentation was checked against primary source before selecting an
implementation. No USB dependency was installed or added to Cargo/audit policy.

| Candidate | Evidence and consequence |
|---|---|
| Linux SG_IO | D1 models exact input/status/sense/residue/actual length. Native permission, actual underrun accounting, selected handle and kernel recovery remain unqualified. Keep the gates from [report 19](19-native-read-adapters.md). |
| Apple IOUSBHost | Official [capture](https://developer.apple.com/documentation/iousbhost/iousbhostobjectinitoptions/devicecapture) and [destroy](https://developer.apple.com/documentation/iousbhost/iousbhostobject/destroy(options:)) semantics require an access/checkpoint audit. Default capture teardown can reset; conditional surrender is not a general no-reset bypass. |
| rusb/libusb | In inspected rusb commit `bb17ee31e4d3e658b9bfad638d7b5e7c42e94864`, synchronous [bulk methods](https://github.com/a1ien/rusb/blob/bb17ee31e4d3e658b9bfad638d7b5e7c42e94864/src/device_handle.rs#L429-L506) convert timeout/interruption with positive transferred bytes into `Ok(count)`. Count-only success cannot preserve our strict native status policy. Drop also releases interfaces. A status-preserving alternative and underlying platform effects need review. |
| nusb 0.2.3 | Pinned tag commit `d939d8bea1262fb9594f4a0a14221c0cc11e8c60` exposes low-level [Completion](https://github.com/kevinmehall/nusb/blob/d939d8bea1262fb9594f4a0a14221c0cc11e8c60/src/transfer/mod.rs#L124-L147) with separate status and actual length, including partial failures. This is a candidate for further binding review, not a selected/qualified dependency. Device claim/drop, permissions, cancellation, OS effects and all transitive dependencies remain unaudited here. |

High-level convenience results must not reconstruct missing status as success.
Never pad a buffer to claim a complete native transfer. Neither a library's safe
Rust API nor an IN data direction establishes no persistent device effects.
Native code must keep exact selection and bind actual kernel/framework results
to original requests; a synthetic checkpoint record cannot become its proof.

## Validation and artifacts

GPT-6.1 Sol high implemented the driver and storage integration; medium wrote
nine independent conformance tests. A separate high agent performed the source
audit and independently reviewed driver/storage/assessment behavior. Root
integrated CLI discovery, capability flags and documentation. All test fixtures
are original fictional bytes, without a bundled vendor firmware/loader blob.

- Canonical source/API/dependency audit passed: **37 Rust files**, no new
  dependency, FFI or hardware/API allowance.
- Focused native debug: **46 USB tests passed in 3.33 s**, 112 deliberately
  filtered. This includes all **11 new tests**: nine driver/storage and two
  entry-assessment/dispatch cases.
- Focused optimized: **46 USB tests in 3.97 s**, **13 macOS contract tests in
  0.00 s** (rounded runner output), **15 adapter integration tests in 0.23 s**;
  **74 distinct tests** passed. Unchanged optimized full writer matrices were
  not repeated.
- Canonical guarded Python 3.12.15: **302 tests passed in 202.073 s**, no skips.
- Full native debug: **193 passed — 158 unit and 35 integration**, with no
  ignored/skipped/filtered tests; unit-suite time **516.11 s**. The canonical
  runner finished exit 0 with unchanged source/fixture fingerprints. These are
  macOS-host results; unsupported-OS-only tests are not counted as exercised.
- Format, offline locked release build and actual CLI status/assessment/entry
  refusal checks passed. Entry assessment exits 0 with `state=blocked`; actual
  `backup enter-rom` refuses with exit 2. Capture/upload/send remain unavailable.

The new tests check every wire request/tag/address and an independent full image
hash on both OS models. They cover stale/cross-lifetime callbacks, checkpoint
loss with unchanged target, private diagnostics, provider panic/error, changed
platform/request/length/status, differing second reads, used-driver refusal,
preflight collision, all nine existing persistence boundary types, later source/
copy loss, strict nested metadata and current-versus-historical/schema separation.
Existing complete storage fault coverage remains in the full regression suite;
the new sweep samples each boundary type without claiming every fault/cartesian
combination. Local IO uses only the runner's private guarded scratch tree.

Independent Python stdlib image oracle:
`bytes((i*73+i//251+19)%256 for i in range(1048576))`, SHA-256
`[digest omitted from public archive]`.

The unsigned local macOS arm64 dispatch/assessment executable is **319,392 bytes**,
SHA-256 `[digest omitted from public archive]`.
Unused library code may be removed by linking; this does not measure a complete
backup/flasher release. The exact **53-file** native input manifest is under
ignored `[local artifact location omitted]`, SHA-256
`[digest omitted from public archive]`.
Rust 1.99 reports a rename/deprecation warning for the stable `fetch_update`
atomic API, plus the two retained transitions/writer dead-code warnings. No
minimum-toolchain decision or release qualification is inferred from this build.

Reproduce with Python 3.11+:

```sh
rtk proxy python3.12 host/run_tests.py
rtk proxy python3.12 native/run_tests.py
rtk proxy python3.12 native/run_tests.py --release --test lib --filter usb_
rtk proxy python3.12 native/run_tests.py --release --test lib --filter macos_usb
rtk proxy python3.12 native/run_tests.py --release --test adapter_conformance
rtk proxy cargo fmt --manifest-path native/Cargo.toml -- --check
rtk proxy cargo build --manifest-path native/Cargo.toml --release --offline --locked
```

The actual local interpreter is
`[private host path omitted]`.
Logs/maps are under ignored `[local artifact location omitted]`. Both platform models
were tested locally on macOS; Linux native/physical execution, Windows support,
hosted CI, signing and distribution remain separate. This increment is local;
no push, physical backup, recovery/write or normal-return acceptance is claimed.

## Remaining D2 and E work

1. Resolve exact stock identity/effective receiver/callee and direct-computer
   MIDI delivery; resolve the pinned loader's startup/read/return and persistent
   side effects. Record explicit evidence per operation; do not broaden commands
   to OTA, ROM memory diagnostics, key/unlock, keepalive or reset by analogy.
2. Review and implement an actual native binding with exact selected handles,
   preserved status/actual lengths, independent deadlines and loader checkpoint
   semantics on one OS at a time. Keep the default/test factories closed.
3. Integrate actual source provenance into storage explicitly, without relabeling
   injected schema 2 as physical. Review destinations, independent-media support,
   shutdown/cancellation and held-session lifetime during sync.
4. Only when these gates clear, perform E's already authorized bounded read-only
   backup with exact capacity/layout, two fresh equal reads, durable independent
   copies and documented original-stock return. No automatic retry/reset, flash
   write, fallback firmware, V15 installation or mandatory dongle is added.
