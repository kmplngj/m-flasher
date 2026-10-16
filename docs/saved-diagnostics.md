# Inspect saved files without a device

`inspect`, `verify` and `dry-plan` operate on local regular files. They dispatch
before device selection, discovery, `Connection` or `Session::open`. They never
start the application window, run target code, reset, reflash or install a
backend. Leave the unit disconnected; these commands need no administrator rights.

Use a build of the revision containing this change; older release binaries do
not have these commands. Build prerequisites and `cargo build --release` are in
[development.md](development.md#building). The offline test feature is for tests,
not for building an application. The commands below use `m-flasher` to mean your
built `target/release/m-flasher` (`.exe` on Windows).

```sh
m-flasher inspect /private/backup/raw-flash.bin
m-flasher verify /private/backup/raw-flash.bin --compare /private/other/raw-flash.bin
m-flasher verify /private/backup/raw-flash.bin --receipt /private/backup/capture-receipt.json
m-flasher verify /private/backup/raw-flash.bin --sha256 EXPECTED_64_HEX_DIGEST --expected-size 1048576
m-flasher inspect /private/update.fwsc --package
m-flasher dry-plan /private/backup/raw-flash.bin /private/update.fwsc --support
```

Run `m-flasher inspect --help` for the exact grammar. `--package` explicitly
selects the shared `.fwsc` parser; filenames do not select a format. A comparison
uses the same representation for both files and compares every saved byte,
including any tail/factory bytes and package auxiliary payloads. The raw file is
never automatically treated as a package or padded. Package analysis alone pads
its embedded flash with erased bytes to interpret declared regions outside the
package; these padding bytes are not saved or acquired data. Application files
must still be fully contained in the actual embedded flash payload. Even a
crafted CRC matching erased padding cannot make missing app bytes intact.

`inspect` calculates a SHA-256, reports saved size and parses known structures.
`verify` additionally requires an expected SHA-256, another file, or a receipt.
When several references are supplied, all must match. Both commands exit 1 on
an invalid input or a reference/expected-size mismatch, otherwise 0. App damage
is reported separately: a complete corrupt-app dump can match its saved reference
and exit 0. This verifies local consistency, not a working application.

The accepted v1 receipt schema is the same `fmr-protected-copies-v1` written by
[protected raw backup](fm1-raw-backup.md). Both `storage-only.json` and
`capture-receipt.json` can bind length/hash to the selected raw file. Their status
codes distinguish storage and held-session stages. The receipt is private,
unauthenticated saved input: matching it does not establish a new physical read,
current device identity, two currently surviving copies, durable storage,
independent media, stock return or bootability. Provenance is never executed or
promoted to fresh evidence. The reader rejects unknown schemas, missing/duplicate
fields and invalid v1 facts rather than guessing their meaning.

## Share a sanitized diagnostic

```sh
m-flasher inspect /private/backup/raw-flash.bin --support > support.json
m-flasher verify /private/backup/raw-flash.bin --receipt /private/backup/capture-receipt.json --support > support.json
```

`--support` emits allowlisted JSON: sizes, fixed integrity/recovery codes,
comparison/receipt results and sector ranges with fixed region names. It omits
raw hashes, file paths, device selectors, unit IDs, keys, MAC addresses, firmware
identity/version strings, receipt provenance and raw/preset bytes. Errors also
use fixed codes without echoing input. The default output adds a
`private_local_sha256` field for private verification; share only the output
made with `--support`. Raw images, receipts and private hash output remain private.
Redirecting stdout is optional; the command itself creates no output files.

Interpret the fields separately:

| Field/status | What it says |
| --- | --- |
| `saved_bytes` / `expected_size_bytes` | Observed file size and an optional user declaration. Neither is actual device capacity. |
| `historical_capacity_bytes` / `matches_historical_size` | The known 1 MiB layout reference and whether a raw file has that size. A matching size alone does not prove a complete physical acquisition. |
| `declared_capacity_bytes` / `actual_device_capacity` | `null` / `unknown`: this command does not infer a chip's capacity from filenames, package identity, receipt length or historical labels. |
| `intact-app-checksums` | Known app entries have valid bounds/checksums. It does not establish executable behavior or data compatibility. |
| `corrupt-or-incomplete-app` | Known app content or its layout is damaged, truncated, empty or overlapping. Saved acquisition integrity remains a separate result. |
| `unknown-layout` | There is no recognized app table; this does not make a saved full image incomplete. |
| `incompatible-data-layout` | Compared declarations disagree. The command does not convert, erase or migrate data. |
| `matching-declarations-only` | Region declarations agree; actual persisted-data/firmware transition compatibility is still unqualified. |
| `no-known-route-from-saved-bytes` | Saved bytes cannot establish a currently reachable USB route. This is not an observation that USB is absent. No recovery command is attempted. |
| `fresh_acquisition: false`, `bootability: not-demonstrated` | These remain true limits for every saved-file report, even with matching receipts, an intact head, a staged loader or an update record. |

Known regions come from validated directory declarations even when app payload
CRCs fail. With an absent/invalid directory only the historical boot/key-MAC
address labels remain; the rest is `UNCLASSIFIED`. These labels describe address
conventions, not verified contents or safely writable ranges. A mixed old/new
incident image cannot establish cold resume merely because its head/loader or
update record appears intact. No known live USB route means a separate,
qualified recovery task is needed; these commands never choose a fallback.

## Advisory dry plan

`dry-plan SAVED_RAW PACKAGE.fwsc` reuses `Package`, `Analysis`, region validation
and the pure `Session::plan_firmware` planning core, without constructing a
session or a device. It reports the number of candidate changed sectors, leaves
the boot area unchanged, and never returns an executable plan. It refuses
unknown/corrupt layouts, unknown saved board evidence, invalid target app bytes,
model mismatches, and changed/unknown data declarations. `candidate_sectors: 0`
means this saved file already matches those candidate bytes. A blocked plan
exits 1. No result authorizes writing: exact transition policy and immutable
application-only execution remain separate work in #13/#14.

## Input limits and tests

Inputs must be nonempty regular files, at most 16 MiB; receipts at most 128 KiB.
Symlinks, directories and special files are rejected. Unix final opens use
`O_NOFOLLOW | O_NONBLOCK` before descriptor validation, so substitution with a
symlink is not followed and a FIFO cannot block inspection. Windows opens the
final reparse point for validation and denies concurrent write/delete sharing. Work on stable private
copies in directories you control; this is a bounded local parser, not an OS
sandbox for adversarial concurrent filesystem replacement. Read/size failures
are terminal. Declared offsets, lengths, overlaps, duplicates and all package
payload CRCs are checked before those payloads can be used. Unknown free text
never enters a support report.

Run the guard-first [offline tests](offline-tests.md). Independent Python-generated
synthetic bytes cover valid/corrupt/unknown images, nondefault app offsets,
malformed packages/tables, incompatible layouts, receipt binding, mixed-sector
advisories and sanitization. The full guarded Cargo suite additionally exercises
the actual CLI dispatcher and shared planner. This is file-only evidence; no
device, emulator, firmware execution or release qualification is implied.
