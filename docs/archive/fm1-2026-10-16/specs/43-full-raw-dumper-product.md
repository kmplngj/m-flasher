> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Full raw dumper as a standalone product capability

Date: **2026-10-16**. Owner clarification: provide a complete flash dumper in
addition to musical-data backups. This makes the existing FM1-033 full-backup
route a user-facing capability in its own right, independent of installing or
restoring firmware. This document defines the contract; it does not activate a
hardware command.

## Three distinct outputs

| Output | Content and purpose | Existing work |
| --- | --- | --- |
| Musical-data archive | Supported presets, projects, settings, samples and other declared musical objects, according to an exact firmware's schema/capabilities. | FM1-023; Felucca's object backup is a reference. This does not establish stock-V15 export support. |
| **Full raw flash dump** | Every physical byte from offset zero through the independently established flash capacity, including head/SPL, application, encrypted bytes, configuration, musical data, device-specific data and unused/tail areas. Preserve the stored representation exactly. | **FM1-033**, including existing two-read/two-copy native work; standalone product surface in 033-F below. |
| Selective firmware/preset export | Reviewed subsets derived from a dump or an exact supported data protocol, with declared omissions and compatibility. | FM1-023. Neither a `.fwsc` package nor a selective archive is a full flash dump. |

The full raw dump is the complete private recovery snapshot already required by
the write policy. A user may create and inspect it without planning any write.
It must not require an update package, successful firmware extraction, portable
preset support, a working target firmware, or a selected installation target.
Qualified transport/chip/entry requirements still apply.

**Priority use case:** capture original stock **before the first Felucca or other
firmware installation**, using only USB-C when that exact route qualifies. No
temporary old/custom-firmware installation is allowed to create that original
snapshot. A dumper running inside already installed Felucca cannot retrospectively
provide the original pre-installation state. An optional USB_KEY activator is a
separate recovery-entry route for cases that need it, not an implicit normal
backup prerequisite.

## Raw acquisition and acceptance contract

1. Select one exact device/session and a qualified read route. Bind the actual
   flash identification and independently supported capacity; do not infer
   capacity solely from a package, preset count or firmware version. The current
   1 MiB model is not an identification result for the owner's flash chip.
2. Audit entry, helper startup, read commands, status-register effects and return
   for the exact bytes/chip/OS. No flash erase, program, status-register write,
   migration or persisted configuration change is permitted. RAM execution and
   mode change must be described honestly. Unknown paths refuse before use.
3. Obtain **two fresh, complete device passes** over `[0, capacity)` at an
   unchanged flash state. The boot head is included in reads; the protected-head
   write prohibition remains unchanged. A cached stream or duplicate host buffer
   does not constitute a second read. Quiescence must cover autonomous stores
   and migrations as well as host commands; do not install a new app to freeze
   the old one.
4. Preserve physical bytes, including ciphertext. Do not substitute XIP-decrypted
   reads, logical APP bytes or normalized/filled regions. Compare both length
   and SHA-256 plus actual byte equality, with complete coverage accounting.
   Any different byte, missing/duplicate/overlapping coverage, timeout or unknown
   chip/state makes the attempt incomplete or failed. Do not silently retry or
   accept selected regions as a full dump.
5. Store the accepted raw bytes and versioned provenance through the existing
   secure two-copy storage path: exclusive creation, bounded operations,
   flush/sync/reopen/hash checks and an independently verified second location.
   Keep the original immutable and private. A second file on the same media is
   not automatically an independent recovery copy. Reuse the report-39 boundary:
   storage-only manifests do not claim final session continuity or stock return.
6. Record fresh final continuity and the documented return to the original
   firmware separately. Distinguish captured bytes, matching passes, stored
   copies, final session acceptance and observed normal operation. Cancellation
   preserves diagnostic evidence but cannot publish an accepted full backup.

The export consists of exact raw binary plus metadata binding capacity, hashes,
source identity/evidence, route/OS, exact helper/profile where applicable,
coverage, pass/copy results and limitations. Detailed device-specific fields stay
private; public reports omit raw bytes, secrets and personal device identifiers.
Offline inspection may identify recognizable regions, but an unknown or corrupt
application does not invalidate an otherwise complete raw acquisition. A dump
from a broken unit must be labeled as such; byte completeness does not attest
to bootability, compatibility or successful restoration.

No full-chip/head restore command is added by this capability. Raw acquisition,
musical-data import, bounded firmware installation and physical recovery writes
retain their separate policies.

## Local Felucca cross-check

The supplied `/workspace/Felucca` path is absent on this host. The current local
checkouts were inspected read-only instead:

| Local source identity | Observed contract | Reuse boundary |
| --- | --- | --- |
| Felucca-WebApp HEAD `efcf5fdb8f1926f50c4e8d079cd9f6723df35ad7`, `fm1backup.js`, clean file | Header describes musical archives using whitelisted objects, never flash addresses; commands LIST/GET/PUT = 65/66/67. | Data-archive reference. PUT/import is outside this read-only task. In the local fork this file moved out of Felucca's `web/` into the separate WebApp repository. |
| Felucca HEAD `db8037a1445a935484739eb7da7da3d62125411b`, **working-tree** `tools/fm1_install.py`, staged modification present | Installer docstring describes device-pull `.fwsc` installation over USB-MIDI, loader writes and post-install identity checking. | It is an installer, not a raw dumper. Its inspected working-tree bytes are not attributed to HEAD or changed here. |
| Same Felucca HEAD, clean `firmware/hal/fm1_sys.h`, `fm1_enter_uboot()` | Writes `usb_update_mode` to retained RAM, changes CPU guards and requests a P33 reset. | Concrete software-entry producer. It does not supply the immutable-ROM consumer or prove original stock's full entry/return behavior. Report 40 covers the exact V15 producer separately. |
| Same Felucca HEAD, clean `firmware/hal/fm1_flash.h`, `fl_read_ram()` / `fl_jedec_ram()` | RAM code uses raw SPI fast-read `0x0B` and JEDEC read `0x9F`; header cites the AC79 driver. `fl_enter` switches SFC/XIP/pins and refuses continuous modes 6/7. | Useful source for a possible AC791N-specific reader. It is not a standalone uploaded dumper or an already qualified stock backup path. |

SHA-256 of the inspected files, respectively:

- `fm1backup.js`: `[digest omitted from public archive]`.
- `fm1_install.py`: `[digest omitted from public archive]`.
- `fm1_sys.h`: `[digest omitted from public archive]`.
- `fm1_flash.h`: `[digest omitted from public archive]`.

Before considering a dedicated helper, audit its complete RAM code/data/callback/
stack closure, exact SPI state and restoration, size/address bounds, finite
polling and error propagation. The inspected `fl_read_ram` does not itself check
capacity; `fl_tx`/`fl_rx` discard `fl_spi_wait` failure results. These source
observations require a dumper-specific contract, rather than copying the helper
unchanged and treating a returned zero as proof of a successful full read.
Source reuse must preserve GPL-3.0-only obligations and attribution. No Felucca
or WebApp source, generated asset, firmware or local WIP was changed or executed.

## USB_KEY documentation conflict

The [dedicated USB_KEY notes](https://kagaimiq.github.io/jielie/isp/usb/usb-key.html)
assign **D+ = clock, D− = data**. The [pinned older jl-uboot-tool guide](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/docs/how-to-enter-uboot.md#usb_key)
assigns the opposite roles in its prose, while its ASCII waveform shows D+
toggling as a clock. The disagreement is real; source age or visual plausibility
alone does not prove the wiring of a particular activator/FM-1 combination.

Keep normal USB transfers, MIDI software entry and electrical USB_KEY generation
distinct. For activator qualification, bind its exact implementation, board-pin
routing, waveform, ACK and bus handoff. Resolve the discrepancy using trustworthy
matching captures or isolated activator/external-bench measurements, not by
trying alternate signals on the owner's only FM-1. This is a recovery follow-up,
not an additional prerequisite for a qualified direct-USB software-entry backup.

## Integration order and backlog ownership

- **FM1-033-D2 / R4-C:** continue the existing exact stock entry/ROM/helper/read/
  macOS evidence work. Report 42's new official helper candidate and callback
  addresses remain useful. A newly built AC791N helper would be a separately
  reviewed route alternative, not a silent substitution or a required rewrite.
- **FM1-033-F1 — offline product surface:** define and implement explicit full-raw
  mode/help/results and reuse existing acquisition/storage modules. Keep full
  raw versus musical/selective export labels distinct. Inspect/verify saved
  dumps offline, reject incomplete/corrupt files without hardware calls, and
  report saved integrity as historical. Never require a `.fwsc` to dump.
  **Implemented offline:** `dump inspect`/`dump verify` and gated capture;
  [report 44](44-offline-raw-dump-cli.md) records exact scope and validation.
- **FM1-033-E — first physical acceptance:** after the exact route qualifies,
  original V15 over direct USB-C/macOS, two fresh equal full reads, protected
  copies, final continuity and observed original-stock return. No flash writes.
  Product-surface polish does not become an extra prerequisite for that first
  qualified backup.
- **FM1-033-F2 — standalone release acceptance:** publish the dumper capability
  only for qualified routes/OSes with truthful capability/help/result states.
  Validate installed-Felucca or externally activated recovery routes separately;
  their availability must not be reported as original-stock support. Linux and
  optional Windows qualification follow without blocking a macOS-only release.
- **FM1-023 / step-2 web:** add reviewed musical/selective exports and later web
  access using the same byte/coverage/storage contract. A browser's musical
  backup success does not imply raw-dump support. Browser persistence/access
  feasibility remains its own gate.

Status: **F1 implemented offline; F2 and physical qualification pending**.
Existing offline components are reused; hardware factories and flash writes
remain unavailable. The initial contract-only increment checked source identities,
local links and diff scope; report 44 records the later implementation tests.
