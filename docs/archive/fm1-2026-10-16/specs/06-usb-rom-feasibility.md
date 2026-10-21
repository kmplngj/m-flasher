> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# Direct USB ROM feasibility — FM1-019

Assessed 2026-10-11, D0 offline only: public source/documents, saved descriptors
and fictional endpoint tests. No device enumeration, native backend, hardware
read/write, privilege/driver change, loader/binary download or firmware build.
The owner's normal-install goal remains one USB-C data cable, without mandatory
RP2040 or a hidden stock-V15 detour. This report is not hardware qualification.

## Decision

| Capability | Decision now | Missing gate |
|---|---|---|
| Offline read-protocol model | Go, delivered | Physical transport and loader behavior are outside the model |
| Linux native selected-device SCSI | Candidate; no released backend | Full-transfer/status/sense validation, explicit OS/USB identity binding, permissions and external read qualification |
| Windows native selected-device SCSI | Candidate; no released backend | Correct access flags, alignment, SCSI status/sense/transfer length, identity and external qualification |
| macOS generic disk SCSI pass-through | No-go for assuming a drop-in backend | Supported exact-peripheral route under Apple storage restrictions |
| macOS direct BOT via IOUSBHost/libusb | Research candidate; no-go for product enablement | Supported authorization/deployment and capture/claim/release checkpoint preservation |
| Bare-ROM cable-only full backup | No-go today | Audited, licensed loader, separately authorized entry/upload and held-session raw-read evidence |
| Cable-only install/recovery write | No-go today | Backup acceptance, exact edge/layout/loader audit, held full readback and external writer/recovery qualification |
| Dark device with no USB | No assumed cable recovery | Validated independent electrical ROM entry; broken power/USB is separate |

No OS has a qualified native backend in this host. macOS support is not
categorically impossible; available mechanisms do not meet current gates.

## Exact identity and read commands

The public Transporter M0 log records ROM `4C4A:8057`, USB revision `0110`, device
revision `0300`, configuration 1, interface 0/alternate 0, MSC class/subclass/
protocol `08/06/50`, IN `81`, OUT `01`, 64-byte bulk packets and LUN0. USB strings
are `WL80UBOOT1.00`; SCSI vendor/product/revision are `WL82` / `UBOOT1.00` / `1.00`.
Serial index is zero. Shared values do not uniquely identify an owner's unit,
prove board/firmware identity or establish loader state.
[Pinned M0 evidence](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/docs/logs/m0_success_2026-10-01.log#L132-L201).

Documented running-app `4C4A:C755`, stock OTA `4D4A:4155` and static V13 template
`4C4A:4155` are different identities, not ROM targets.
[Pinned USB analysis](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/06-usb.md#L18-L37).

```text
INQUIRY      12 00 00 00 24 00                       IN 36 bytes
READ_FLASH   FD 05 [address:u32 BE] [N:u16 BE] FF×8    IN N bytes
READ_MEMORY  FD 07 [address:u32 BE] [N:u16 BE] FF×8    IN N bytes
```

Vendor CDBs are 16 bytes with `FF` padding. `FD07` is ROM memory access;
`FD05` is LoaderV2 raw flash access. A RAM/XIP memory read is not a raw NOR
snapshot. [CDB construction](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/uboot.py#L77-L134),
[ROM/loader classes](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jltech/uboot.py#L238-L304).

The upstream raw-flash path uploads a RAM loader with `FB06 WRITE_MEMORY`, then
executes it with `FB08 JUMP`: mode-changing code execution, not passive reading.
ROM memory reads decode a cipher per transfer. Transporter defaults to 512-byte
flash reads and can negotiate larger buffers; the prototype fixes a 512-byte
maximum and has no negotiation/upload/jump. [Upload](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/src/jieli_uboot.c#L115-L175),
[ROM read gate](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/src/jieli_uboot.c#L284-L325),
[Limits](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/src/jieli_uboot.h#L15-L22).

Upstream describes loader reset after roughly three seconds and a one-second
keepalive. Mock deadlines bound host operations only: there is no physical
watchdog or keepalive evidence. [Session observations](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/docs/PROTOCOL.md#L23-L32),
[Keepalive](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/src/jieli_uboot.c#L268-L280).

## Native source/platform audit

Linux's pinned adapter opens the selected path `O_RDWR` and uses `SG_IO` with a
five-second timeout. It copies requested buffer size even with nonzero residue;
the higher protocol ignores that residue. Short transfers can appear full and
partly zero-filled. Future implementation must validate realized bytes and
SCSI/host/driver status/sense. [Pinned Linux adapter](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/scsiio/linuxport.py#L49-L117).

Windows's adapter uses `CreateFileW` and `IOCTL_SCSI_PASS_THROUGH_DIRECT`, with a
five-second timeout. It checks the API boolean but leaves SCSI status/sense/
residue incomplete and lacks explicit alignment handling. `GENERIC_READ=1` and
`GENERIC_WRITE=2` differ from Microsoft's `80000000` and `40000000`. Pinned
Unbricker imports the backend without correcting those constants. These are
source gaps, not evidence of a qualified Windows route.
[Pinned Windows adapter](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/scsiio/win32port.py#L27-L143),
[Microsoft access constants](https://learn.microsoft.com/en-us/windows/win32/secauthz/generic-access-rights),
[Pass-through requirements](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntddscsi/ni-ntddscsi-ioctl_scsi_pass_through_direct).

Generic discovery opens/probes candidate disks. Do not reuse that behavior or
override targeting. Future native selection must resolve one explicitly chosen
USB location/OS object, correlate descriptors and then inquiry/loader/chip
evidence on that object. No default disk, all-disk sweep, model name or VID/PID
alone may authorize targeting. [Generic finder](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/jldevfind.py#L6-L81),
[Unbricker selection/overrides](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/fm1_unbrick.py#L368-L464).

Apple's archived SAM guide excludes arbitrary user-space SCSI commands to
ordinary storage devices without driver cooperation. `SCSITaskDeviceInterface`
serves other peripheral/authoring cases, not a proven generic disk escape. Its
current exclusive-access API can also return busy for mounted media. BSD block
reads cannot substitute for `FD05`; a suitable ROM user client is unproven.
[Apple SAM guide](https://developer.apple.com/library/archive/documentation/DeviceDrivers/Conceptual/WorkingWithSAM/WWS_SAMDevInt/WWS_SAM_DevInt.html),
[Exclusive access](https://developer.apple.com/documentation/iokit/scsitaskdeviceinterface/1575374-obtainexclusiveaccess).

IOUSBHost `deviceCapture` terminates existing device/interface drivers. It
requires the documented entitlement plus `IOServiceAuthorize`, or root;
destruction resets and re-registers interfaces. The entitlement is documented
for VM USB access, so flasher deployment cannot assume entitlement approval.
[Apple capture](https://developer.apple.com/documentation/iousbhost/iousbhostobjectinitoptions/devicecapture),
[Entitlement purpose](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.vm.device-access).

libusb's current FAQ describes whole-device capture and CLI/entitlement
constraints. Independently inspected Darwin source at
`abba63ed3cad642cc2540ad407b5304109c0ca45` (2026-09-30) enforces entitlement/root
authorization and explicitly reenumerates for capture/final reattachment.
Current detach support supersedes historical unavailable-patch claims, but does
not prove FM-1 compatibility. We infer a risk to RAM-loader state; no physical
reset behavior was observed here. No privilege/driver bypass is proposed.
[FAQ](https://github.com/libusb/libusb/wiki/FAQ),
[Pinned Darwin source](https://github.com/libusb/libusb/blob/abba63ed3cad642cc2540ad407b5304109c0ca45/libusb/os/darwin_usb.c#L3537-L3623).

## Delivered prototype and limits

`host/src/fm1flasher/usb_bot.py` binds only the exact in-memory endpoint with
caller-saved fictional `simulation:` identity/location. Descriptor field values
come from the public log; identity, flash bytes, loader digest and INQUIRY header
are generated. The log publishes decoded fields, not all 36 response bytes;
the synthetic header is not a physical device acceptance fingerprint. There are
no native imports, OS APIs, filesystem/device paths or USB CLI commands.

CBW31 → bounded IN data → CSW13 is serialized, with `USBC`/`USBS`, little-endian
tag/length, IN direction and LUN0. Exact tag, zero residue, success status and
stage lengths are mandatory. [USB-IF BOT 1.0 sections 5–6](https://www.usb.org/sites/default/files/usbmassbulk_10.pdf).

Malformed/short/trailing frames, stall, timeout, status/tag/residue errors,
clock exceptions and descriptor/session/loader drift poison the handle. Reset,
clear-halt, capture, reconnect, retry, RAM entry and writes are absent. Invalid
caller ranges refuse before I/O and retain the session; no failed transaction
is retried. Frozen input records are revalidated at boundaries against forced
mutation and bool-as-int values.

Tests cover independent golden wire bytes, fragmented binary stages, all three
saved OS labels, two fresh 1 MiB reads, selection ambiguity, ROM/app/no-USB
refusal, busy/permission faults and poisoning. Full reads issue 2048 new blocks;
they are not durable backups, chip/firmware evidence or physical verification.
Reports explicitly set `real_write_available`, `native_access_available`,
`hardware_read`, `loader_proven` and `durable_backup` false. Validation uses
Python 3.11 `host/run_tests.py` with its guard installed before host imports.

## Source/license inventory

| Source / immutable pin | License and use |
|---|---|
| [Transporter a632d923203170e3a08565f4056dbb3fd3d65ca5](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/LICENSE) | MIT; public protocol/log research only; upstream/PIO untouched |
| [Unbricker b8c8e8afbcba3e45b262b3b624ab9d0244b92628](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/LICENSE) | MIT; source/selection research |
| [FM-1-RE ec832f281ca165a7d5e0722985f3acb1465046d5](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/LICENSE) | WTFPL v2; documentation research |
| [jl-uboot-tool adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/LICENSE) | MIT; command/adapter audit; no backend copied/imported |
| [libusb abba63ed3cad642cc2540ad407b5304109c0ca45](https://github.com/libusb/libusb/blob/abba63ed3cad642cc2540ad407b5304109c0ca45/COPYING) | LGPL-2.1; research only, no bundled source/dependency |
| Apple/Microsoft/USB-IF documentation | Linked, not redistributed |
| New host model/fixtures | Original MIT code, fictional bytes |

Source licenses do not establish vendor firmware/loader redistribution rights.
`wl82loader.bin` provenance/license is unresolved; no binary was downloaded or
bundled. Unbricker documents a placeholder UF2 with the real loader added later,
which does not grant binary rights. [Loader metadata](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/data/usb-loaders.yaml#L201-L208),
[Placeholder notice](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/transporter/README.md#L1-L21).
Pico-PIO-USB pins/notices remain unchanged.

## Separate follow-on proposals

These proposals do not authorize native or physical operations:

| Proposed ticket | Scope and gate |
|---|---|
| USB-ROM-A: entry/loader contract | Offline cable-entry audit per source firmware, licensed loader/digest/behavior; no hidden V15 hop or head change. Unknown edges refuse. ROM visibility does not imply raw-read availability. |
| USB-ROM-B: Linux/Windows adapter contract | Exact OS/USB target binding, status/sense/residue/alignment; fake native-result tests first. No generic scan/override. Separate authorization/external evidence before native enablement. |
| USB-ROM-C: macOS access/checkpoint decision | Supported SCSI/IOUSBHost/DriverKit deployment and all capture/claim/release/reset effects. Approved permissions and held checkpoint preservation are gates; no protection bypass. |
| USB-ROM-D: read/backup qualification | Authorized exact-unit entry/read, known loader/chip/layout, keepalive/disconnect evidence; two equal full reads, reopen/hash verification, independent durable copy and documented normal return. No writes. |
| USB-ROM-E: external direct writer qualification | Spare/external unit only: allowlisted sectors, preserved head/data, sector and held full-image readback, interruption/recovery evidence, residual risk. Never experiment on the owner's sole unit. |

Reuse FM1-012/015/018 requirements rather than weakening USB gates. No-USB needs
independent electrical entry evidence; direct USB does not establish cold resume.
