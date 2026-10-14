# M-Flasher

Back up, restore and reinstall the firmware of an **M-VAVE FM-1** or **SMK-37
Pro / Elite** over a plain USB cable, and install the
[open bootloader](https://github.com/jvitkauskas/fm1-bootloader). It works with
the chip's built-in USB download mode, so it also works when the firmware
on the unit is broken.

> Unofficial. Not affiliated with M-VAVE or JieLi. Every write is checked, and
> the tool saves a backup before it writes anything, but use it at your own risk.

![M-Flasher connected to an SMK-37 Pro in download mode: the device panel, the operations and the flash map](docs/screenshot.jpg)

The **flash map** shows the unit's 1 MiB flash as its 256 sectors of 4 KiB,
coloured by what they hold (bootloader, firmware, settings, pairings,
calibration, presets, sounds). Before a write it marks exactly the sectors
that will change; while writing they light up as they are written and
checked.

## Windows: getting started

1. Download `M-Flasher-…-windows.zip` from the [releases](../../releases),
   extract it and run `M-Flasher.exe`. Windows asks for administrator rights
   (needed to talk to the unit's download mode). The app is not signed, so the
   first time Windows may say it protected your PC: choose **More info → Run
   anyway**.
2. Put the unit in **download mode** and connect it with a USB **data** cable
   (directly, not through a hub):
   - with the open bootloader installed: hold **REC** (SMK-37: **SEQ REC**)
     while switching it on;
   - Felucca, SLOOP or Snoop37 running: hold **OCT-** and **OCT+** for 5 s;
   - a unit whose firmware is damaged often ends up there by itself.

   In download mode the screen stays as it was. Nothing shows up in Explorer
   or Finder (Windows' Disk Management lists an empty 0-byte disk, "WL82
   UBOOT1.00"); M-Flasher finds the unit by itself.
3. The app finds the unit, reads its whole flash and shows the model, the
   firmware and the bootloader. The unit's own screen shows what is going on.
4. Choose an action. When you are done, unplug the unit and switch it off and
   on again.

A unit with the stock bootloader and a working stock firmware cannot be put
in download mode with its keys. Install the open bootloader first: on the
SMK-37 Pro with M-VAVE's own updater (see the bootloader's README), or with a
USB_KEY dongle (a Raspberry Pi Pico).

## What it can do

| Action | What it writes |
| --- | --- |
| **Back up the whole flash** | nothing: saves the 1 MiB flash image (and a `.json` with its details) to `Documents\M-Flasher\backups` |
| **Restore a backup** | the sectors that differ; a backup of another unit keeps this unit's own factory data (calibration, Bluetooth address) |
| **Install firmware (.fwsc)** | the firmware from an M-VAVE package (or a custom one in the same format), from 0x4000 to the end of its image; optionally its stock bootloader, and optionally erases the settings and Bluetooth pairings that firmware declares |
| **Install the open bootloader** | the boot area only (16 KiB), with the build for the identified model |
| **Put back the stock bootloader** | the boot area only, from an official package |

Use the official `.fwsc` files from M-VAVE's website as they are: no
conversion is needed. The app checks that a package is for the connected
model and that its firmware decrypts with this unit's chip key before it
writes anything.

## Safety

- The flash is read when the unit connects and checked against a CRC32 the
  chip computes itself (with JieLi's loader: read twice and compared); if they
  differ, nothing is written.
- A full backup is saved before the first write of a session.
- Only the 4 KiB sectors that change are written, and only inside the ranges the
  action allows (for example, installing firmware never touches the boot area
  unless asked; installing a bootloader writes nothing else).
- Everything written is checked (the chip's CRC32 of it, or read back and
  compared); a mismatch is rewritten once, then the app stops and says so.
- The model is identified from the flash (the SMK-37's factory calibration,
  the open bootloader's board tag, the firmware's layout, the flash chip).
  If it is not certain, the app says so and lets you choose; a package or
  bootloader for another model is refused.
- Firmwares keep their settings and data in different places. When the
  installed firmware's layout differs from the package's (Baud Girl, for
  example, keeps its settings inside the stock presets area), the app warns:
  what is left behind can confuse the new firmware. Restoring a backup made
  while a firmware was installed is the clean way back to it.
- Not covered: the synth sounds (USR) that official updates never write come
  back only from a backup.

## Speed

A full read takes about 1.2 s and a firmware install about 10 s, on a good
USB port. Some USB controllers are much slower with the unit (seen: every port
of AMD's 600-series chipset controller, about 4.5 s for a read); the app logs
the read speed and suggests another port then, often one at the back next to
the video outputs. Details in [loader/README.md](loader/README.md).

## Status

Tested on an SMK-37 Pro from Linux: identification (stock, Baud Girl,
Snoop37), backups (identical to jl-uboot-tool's reads), installing the open
bootloader, installing the stock firmware over Baud Girl and over Snoop37,
restoring a backup (the flash afterwards identical to it); on Windows with the
same unit; on macOS (an M3 Mac) the window and the command line, installing
firmware and restoring. Not yet tried: an FM-1.

## Linux

Download `m-flasher-…-linux-x86_64.tar.gz` from the [releases](../../releases)
and extract it. It needs the SCSI generic driver (`sudo modprobe sg` if
`/dev/sg*` does not exist). Either run it as root, or install the udev rule
once:

```sh
sudo cp m-flasher/70-m-flasher.rules /etc/udev/rules.d/ && sudo udevadm control --reload
m-flasher/m-flasher
```

`m-flasher.png` is the icon, for a launcher of your own.

## macOS

Download `M-Flasher-…-macos.dmg` from the [releases](../../releases) (for
Apple silicon and Intel), open it and drag M-Flasher to Applications. The app
is not signed with an Apple certificate, so the first time macOS blocks it:
**System Settings → Privacy & Security → Open Anyway**.

macOS does not pass the download mode's SCSI commands through, so M-Flasher
takes the unit from the system's disk driver and talks USB mass storage
itself (libusb). Only root may take a device from its driver: when the app
finds the unit it asks for your password in the system's dialog and starts a
small root helper that does the USB transfers for it (see `src/macos.rs`); the
window stays a normal app. macOS asks once to let M-Flasher access files on a
removable volume (the unit): choose **Allow**. Start it soon after connecting
the unit: left with macOS's disk driver, the unit may leave download mode
after about half a minute. Backups go to `~/Documents/M-Flasher/backups`.

The command line is inside the app:
`/Applications/M-Flasher.app/Contents/MacOS/M-Flasher info` (or `backup`,
...), from Terminal; it asks for the password the same way, or run it with
sudo. Not over SSH: macOS can ask about the volume only on the screen.

`macos/package.sh` makes the app and the disk image; `macos/render.sh`
renders the icon and the disk image's background from their SVGs.

## Command line

```sh
m-flasher info
m-flasher backup [DIR]
m-flasher restore BACKUP.bin
m-flasher firmware PACKAGE.fwsc [--with-bootloader] [--wipe-settings]
m-flasher bootloader [--stock PACKAGE.fwsc | --file UBOOT.boot]
```

Without arguments it opens the window. Changes are shown first and need a
typed `yes` (or `--yes`). On Windows, run it from an administrator prompt.

## How it works

The JieLi AC791N's mask ROM has a USB download mode (UBOOT1.00) that shows up
as a USB mass-storage device and takes vendor SCSI commands. The app writes a
flash loader into the chip's RAM through it and starts it; the loader reads,
erases and writes the SPI flash. It is M-Flasher's own loader
([loader/](loader/README.md): faster, a CRC32 computed on the chip, the
flash's unique ID, progress on the unit's screen), or, if that does not
answer, JieLi's own WL82 loader. The protocol follows
[jl-uboot-tool](https://github.com/kagaimiq/jl-uboot-tool) by kagaimiq.

One native executable: the window (egui), the command line, the loaders, the
open bootloader builds and the fonts are all inside it.

| | |
| --- | --- |
| `src/formats.rs` | flash and package formats: CRCs, ENC, file tables, boot area, `.fwsc` |
| `src/boards.rs` | the models and how a flash image identifies them |
| `src/sectormap.rs` | the flash map: a region for each of the 256 sectors |
| `src/scsi.rs` | SCSI pass-through: Linux SG_IO, Windows SCSI_PASS_THROUGH_DIRECT |
| `src/usb.rs` | macOS: USB mass-storage Bulk-Only Transport through libusb (on Linux with `--features usb`) |
| `src/macos.rs` | macOS: the root helper that reaches the unit, started through the system's password dialog |
| `src/jl.rs` | the download-mode protocol and starting a loader |
| `src/ops.rs` | the operations and their safety rules |
| `src/controller.rs` | the window's logic in background threads |
| `src/ui.rs` | the window: the panel and the flash map, drawn with egui |
| `src/cli.rs` | the command line |

```sh
cargo build --release          # target/release/m-flasher
cargo test --release           # FLASHER_TEST_DATA=DIR adds the tests with real flash images
```

GitHub Actions builds the Linux, Windows and macOS packages
(`.github/workflows/build.yml`), and publishes them for a `v*` tag.

## Credits and licenses

MIT, see [LICENSE](LICENSE); the loader in `loader/` is Apache 2.0. Bundled:
JieLi's WL82 USB loader (from the AC79 SDK, Apache 2.0), the open bootloader
builds (Apache 2.0), see [data/NOTICE.md](data/NOTICE.md), and the window's
fonts (OFL, see [assets/README.md](assets/README.md)). Thanks to kagaimiq for
jl-uboot-tool, and to the
[MvaveFM1Unbricker](https://github.com/Quixotic7/MvaveFM1Unbricker) and
[FM-1 Transporter](https://github.com/kurogedelic/FM-1-transporter) projects,
whose safety rules this follows.
