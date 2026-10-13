# FM Flasher

Back up, restore and reinstall the firmware of an **M-VAVE FM-1** or **SMK-37
Pro / Elite** over a plain USB cable, and install the
[open bootloader](https://github.com/jvitkauskas/fm1-bootloader). It works with
the chip's built-in USB download mode, so it also works when the firmware
on the unit is broken.

> Unofficial. Not affiliated with M-VAVE or JieLi. Every write is checked, and
> the tool saves a backup before it writes anything, but use it at your own risk.

## Windows: getting started

1. Download `FM-Flasher.exe` from the [releases](../../releases) and run it.
   Windows asks for administrator rights (needed to talk to the unit's
   download mode). Windows may also warn that the app is unrecognised: choose
   **More info → Run anyway**.
2. Put the unit in **download mode** and connect it with a USB **data** cable
   (directly, not through a hub):
   - with the open bootloader installed: hold **REC** (SMK-37: **SEQ REC**)
     while switching it on;
   - Felucca, SLOOP or Snoop37 running: hold **OCT-** and **OCT+** for 5 s;
   - a unit whose firmware is damaged often ends up there by itself.

   In download mode the screen stays as it was and the computer sees a USB
   disk called "WL82 UBOOT1.00". **If Windows offers to format a disk, choose
   Cancel.**
3. The app finds the unit, reads its whole flash (twice, to be sure the
   connection is reliable) and shows the model, the firmware and the
   bootloader.
4. Choose an action. When you are done, unplug the unit and switch it off and
   on again.

A unit with the stock bootloader and a working stock firmware cannot be put
in download mode with its keys. Install the open bootloader first: on the
SMK-37 Pro with M-VAVE's own updater (see the bootloader's README), or with a
USB_KEY dongle (a Raspberry Pi Pico).

## What it can do

| Action | What it writes |
| --- | --- |
| **Back up the whole flash** | nothing: saves the 1 MiB flash image (and a `.json` with its details) to `Documents\FM Flasher\backups` |
| **Restore a backup** | the sectors that differ; a backup of another unit keeps this unit's own factory data (calibration, Bluetooth address) |
| **Install firmware (.fwsc)** | the firmware from an M-VAVE package (or a custom one in the same format), from 0x4000 to the end of its image; optionally its stock bootloader, and optionally erases the settings and Bluetooth pairings that firmware declares |
| **Install the open bootloader** | the boot area only (16 KiB), with the build for the identified model |
| **Put back the stock bootloader** | the boot area only, from an official package |

Use the official `.fwsc` files from M-VAVE's website as they are: no
conversion is needed. The app checks that a package is for the connected
model and that its firmware decrypts with this unit's chip key before it
writes anything.

## Safety

- The flash is read twice when the unit connects; if the reads differ, nothing is written.
- A full backup is saved before the first write of a session.
- Only the 4 KiB sectors that change are written, and only inside the ranges the
  action allows (for example, installing firmware never touches the boot area
  unless asked; installing a bootloader writes nothing else).
- Everything written is read back and compared; a mismatch is rewritten once,
  then the app stops and says so.
- The model is identified from the flash (the SMK-37's factory calibration,
  the open bootloader's board tag, the firmware's layout, the flash chip).
  If it is not certain, the app says so and lets you choose; a package or
  bootloader for another model is refused.
- Not covered: the synth sounds (USR) that official updates never write come
  back only from a backup.

## Linux

Needs Python 3.9+ with Tk (`python3-tk` on Debian/Ubuntu) and the SCSI generic
driver (`sudo modprobe sg`). Either run as root, or install the udev rule once:

```sh
sudo cp linux/70-fm-flasher.rules /etc/udev/rules.d/ && sudo udevadm control --reload
python3 -m flasher            # the window
python3 -m flasher info       # or the command line: find, info, backup, restore, firmware, bootloader
```

macOS is not supported (it does not allow the SCSI commands download mode needs).

## Command line

```sh
python -m flasher info
python -m flasher backup [DIR]
python -m flasher restore BACKUP.bin
python -m flasher firmware PACKAGE.fwsc [--with-bootloader] [--wipe-settings]
python -m flasher bootloader [--stock PACKAGE.fwsc | --file UBOOT.boot]
```

Changes are shown first and need a typed `yes` (or `--yes`).

## How it works

The JieLi AC791N's mask ROM has a USB download mode (UBOOT1.00) that shows up
as a USB mass-storage device and takes vendor SCSI commands. The app writes
JieLi's WL82 flash loader into the chip's RAM through it and starts it; the
loader reads, erases and writes the SPI flash. The protocol follows
[jl-uboot-tool](https://github.com/kagaimiq/jl-uboot-tool) by kagaimiq. Flash
layout and package format: `flasher/formats.py`.

Development: `python -m unittest` runs the tests against a simulated unit;
the tests with real flash images need `FLASHER_TEST_DATA` (see
`tests/test_ops.py`). The Windows build is made by GitHub Actions
(`.github/workflows/build.yml`) with PyInstaller.

## Credits and licenses

MIT, see [LICENSE](LICENSE). Bundled: JieLi's WL82 USB loader (from
jl-uboot-tool) and the open bootloader builds (Apache 2.0), see
[flasher/data/NOTICE.md](flasher/data/NOTICE.md). Thanks to kagaimiq for
jl-uboot-tool, and to the
[MvaveFM1Unbricker](https://github.com/Quixotic7/MvaveFM1Unbricker) and
[FM-1 Transporter](https://github.com/kurogedelic/FM-1-transporter) projects,
whose safety rules this follows.
