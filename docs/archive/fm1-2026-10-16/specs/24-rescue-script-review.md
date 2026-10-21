> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Rescue-script review and existing-plan carry-over

Date: **2026-10-15**. FM1-033-D2 source-only review. Only public text was
downloaded/read; neither rescue script was imported or executed. No hardware
inventory/open, backend installation, firmware build or loader execution occurred.

## Decision

**Retain this as an additional implementation reference within FM1-033-D2.**
The useful detail is a small macOS-oriented capture/claim and prompt-lifetime
example, with concrete failure cases to compare against our native boundary.
The ROM/RAM-loader/read route, inactivity problem and persistence gate already
exist in [report 16](16-stock-usbc-backup.md), [report 21](21-injected-native-read-driver.md)
and [report 22](22-actual-usb-bindings.md). This source supplies neither a new
stock-entry route nor a qualified macOS backup. Keep the native activation gate
closed and retain the existing order of D2 work.

## Exact source identity

These are the repository `main` snapshots resolved through GitHub's commit API
at review time; links below use immutable commits. They are not the scripts'
last-change commits.

| Artifact | Repository snapshot | Downloaded bytes / SHA-256 |
|---|---|---|
| Requested shell wrapper | [`charlesvestal/fm1-amb` `d4a211c7f45096d8e9af6f72297695fd9da6c463`](https://github.com/charlesvestal/fm1-amb/blob/d4a211c7f45096d8e9af6f72297695fd9da6c463/tools/fm1_rescue.sh) | 2,153 / `950765275956540239617c5d862f5ecd2e648c34d9cb8fbecdc92db533e8afc6` |
| Python actually fetched by that wrapper | [`charlesvestal/fm1-x0x` `cc2e140563bfe770c533fa0da61650ca649a88b6`](https://github.com/charlesvestal/fm1-x0x/blob/cc2e140563bfe770c533fa0da61650ca649a88b6/tools/fm1_rescue.py) | 18,071 / `fc8687107535049b7ac7a7257eb63ae571c75049d0c64bd5f23beba167e7fbaa` |

The wrapper fetches Python from **another repository's mutable `main`**,
installs unpinned PyUSB/libusb-package into a local virtualenv and runs Python
through `sudo`, passing `--ask` to the script. The V15 package has an exact hash, but the Python bootstrap
has no revision/digest verification. The Python separately pins the loader URL
to jl-uboot-tool `c32b28e7a04f873f526ba929cb1bae9f05442fe9` and checks encoded
loader digest `d41da6126760c9d66660bcc0cac8d27d221806c5e369a8036921efe68dca5376`,
the same artifact already audited in [report 23](23-loader-static-followup.md).
It is not a new loader or source/build lineage.
[Bootstrap](https://github.com/charlesvestal/fm1-amb/blob/d4a211c7f45096d8e9af6f72297695fd9da6c463/tools/fm1_rescue.sh#L5-L47),
[loader constants/check](https://github.com/charlesvestal/fm1-x0x/blob/cc2e140563bfe770c533fa0da61650ca649a88b6/tools/fm1_rescue.py#L36-L41).

## What the source establishes

| Detail | Observed implementation and limit | Existing-plan use |
|---|---|---|
| macOS capture | Polls the first matching `4C4A:8057`; attempts active-kernel-driver detection/detach on interface 0, tolerates unsupported/error cases, then claims it. It may set a configuration. This is an attempted access strategy, not evidence the driver was detached or the device captured on macOS. | Compare exclusive open, configuration, ordinary claim, busy/permission failure and teardown with report 22. Do not adopt forced detach or configuration changes by analogy. |
| Reset and reacquisition | Failed initial INQUIRY triggers BOT reset/clear-halt; failures are swallowed. Open failures are polled again; the complete rescue defaults to five attempts. | Model these as behaviors our terminal-failure policy must continue to exclude. A reset or another same-VID/PID object is not continuity. |
| Held prompt | `keep_alive` serially sends `FC0A` once per second while a confirmation callback waits, then joins the thread before writing. The approximately three-second idle reset is a source comment, already reported by Transporter. | Concrete reference for an audited serialized session-lifetime design; no automatic addition of `FC0A` to our read allowlist. |
| Persistence waits | Keepalive starts **after** a single full read, ordinary backup file write/close and diff calculation. It covers the confirmation callback only, not file I/O, sync/reopen/second-media work, unrelated waits, release or error cleanup. | Extend existing D2 lifetime analysis to every persistence/cancellation boundary; report 21 already requires this. A local timeout is not a live-loader checkpoint. |
| Entry and return | Waits for an already visible ROM device after a described X0X crash loop. Implements no running-stock MIDI entry. Read-only completion releases resources without an explicit normal-return check; writer completion attempts `FC0C` and suppresses errors. | Exact-stock entry, transitive loader effects and original-stock return remain unresolved. |

[Capture/reset/claim](https://github.com/charlesvestal/fm1-x0x/blob/cc2e140563bfe770c533fa0da61650ca649a88b6/tools/fm1_rescue.py#L119-L173),
[polling](https://github.com/charlesvestal/fm1-x0x/blob/cc2e140563bfe770c533fa0da61650ca649a88b6/tools/fm1_rescue.py#L343-L370),
[backup/prompt ordering](https://github.com/charlesvestal/fm1-x0x/blob/cc2e140563bfe770c533fa0da61650ca649a88b6/tools/fm1_rescue.py#L236-L310),
[retry/release](https://github.com/charlesvestal/fm1-x0x/blob/cc2e140563bfe770c533fa0da61650ca649a88b6/tools/fm1_rescue.py#L381-L421).

The initial rescue commit explicitly records simulation with the real package
and **no hardware test yet**. The inspected three path-history commits, pinned
README and script provide no subsequent independent macOS runtime record.
The header's hardware-verification wording refers to Transporter's protocol
and policy; it cannot qualify this Python/libusb implementation. This is a
bounded review finding, not a claim that no external result exists.
[Initial commit](https://github.com/charlesvestal/fm1-x0x/commit/a61701e7034cd4027f16ddfeb3e77ecb8709e752),
[later prompt/backend change](https://github.com/charlesvestal/fm1-x0x/commit/70440e57c2b1390c93569baa55b8e6b7eacbabc2).

## Gaps and action mapping

Our existing requirements remain stricter. The script checks the stock package
and loader hashes, INQUIRY vendor and chip key/type/flash ID, but uploads/runs
the loader before chip validation. It selects no explicit unique device and
does not bind a physical session across retries. BOT checks signature/tag/status
but discards CSW residue and does not require exact data/write counts. Its raw
read loop can keep making no progress on empty successful replies. Backup is
one read into a timestamped `wb` path, without exclusive creation, a second
fresh read, durable synchronization, reopened verification, provenance manifest
or an independently verified copy. These are comparison/negative-fixture leads,
not components to substitute for our implemented safeguards.
[Transport](https://github.com/charlesvestal/fm1-x0x/blob/cc2e140563bfe770c533fa0da61650ca649a88b6/tools/fm1_rescue.py#L155-L217),
[validation and backup](https://github.com/charlesvestal/fm1-x0x/blob/cc2e140563bfe770c533fa0da61650ca649a88b6/tools/fm1_rescue.py#L236-L264).

- **FM1-033-D2:** retain the pinned comparison; review selected-device OS access
  effects separately from descriptor matching; audit exact `FC0A`/idle/error/
  return effects before designing any keepalive; specify serialized ownership,
  cancellation and continuity throughout both reads and durable independent
  persistence. Keep reset/retry/detach unavailable and physical claims false
  until exact route/platform evidence exists. No new ticket is needed.
- **FM1-013/015/017, future writer only:** changed-sector bounds, per-sector
  readback and final expected-full-image comparison are already planned.
  The source's descending sector order depends on a specific X0X crash-recovery
  assumption and is not universal recovery evidence. Three sector attempts,
  whole-job retries, automatic restart and a single backup do not meet our
  session/retry/durability requirements. Carry only the source comparison into
  later offline review; no writer work or qualification is enabled here.
- **Provenance/license review:** Python declares `GPL-3.0-only`; retain separate
  attribution/license treatment before any code reuse. This note incorporates
  observations, not copied implementation. The pinned loader's separate
  redistribution/source-linkage questions remain open in report 23.
  [License header](https://github.com/charlesvestal/fm1-x0x/blob/cc2e140563bfe770c533fa0da61650ca649a88b6/tools/fm1_rescue.py#L1-L2).

This documentation-only review supplies no new build/test/runtime evidence and
does not change the read allowlist, native factory, profile or hardware gate.
