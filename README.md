# M-Flasher

Back up, restore and reinstall the firmware of an **M-VAVE FM-1** or **SMK-37
Pro / Elite** over a plain USB cable, and install the
[open bootloader](https://github.com/jvitkauskas/fm1-bootloader). It works with
the chip's built-in USB download mode, so it also works when the firmware
on the unit is broken.

> Unofficial. Not affiliated with M-VAVE or JieLi. Every write is checked, and
> the tool saves a backup before it writes anything, but use it at your own risk.

![M-Flasher reviewing a firmware install: the flash map marks the 153 sectors it will write](docs/screenshot.jpg)

The **flash map** shows the unit's 1 MiB flash as its 256 sectors of 4 KiB,
coloured by what they hold (bootloader, firmware, settings, pairings,
calibration, presets, sounds). Before a write it marks exactly the sectors
that will change; while writing they light up as they are written and read
back.

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
- Firmwares keep their settings and data in different places. When the
  installed firmware's layout differs from the package's (Baud Girl, for
  example, keeps its settings inside the stock presets area), the app warns:
  what is left behind can confuse the new firmware. Restoring a backup made
  while a firmware was installed is the clean way back to it.
- Not covered: the synth sounds (USR) that official updates never write come
  back only from a backup.

## Status

Tested on an SMK-37 Pro from Linux: identification (stock, Baud Girl,
Snoop37), backups (identical to jl-uboot-tool's reads), installing the open
bootloader, installing the stock firmware over Baud Girl, restoring a backup
(the flash afterwards identical to it); the Windows build (0.1.0) on the same
unit. Not yet tried: an FM-1.

## Linux

Needs Python 3.9+ and the SCSI generic driver (`sudo modprobe sg`). The
window uses [pywebview](https://pywebview.flowrl.com/) with the system's
WebKitGTK (Debian/Ubuntu: `python3-gi gir1.2-webkit2-4.1`; Arch: `python-gobject
webkit2gtk-4.1`); without it a simpler Tk window opens (`python3-tk`). Either
run as root, or install the udev rule once:

```sh
sudo cp linux/70-m-flasher.rules /etc/udev/rules.d/ && sudo udevadm control --reload
python3 -m venv --system-site-packages .venv && .venv/bin/pip install pywebview
.venv/bin/python -m flasher           # the window (--tk: the simple one)
.venv/bin/python -m flasher info      # or the command line: find, info, backup, restore, firmware, bootloader
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

The window is HTML/CSS/JS (`flasher/ui/`) in a webview, Edge WebView2 on
Windows; `flasher/controller.py` runs everything behind it and
`flasher/webui.py` connects the two.

Development: `python -m unittest` runs the tests against a simulated unit;
the tests with real flash images need `FLASHER_TEST_DATA` (see
`tests/test_ops.py`). `tools/ui_demo.py` saves the window's states for
previewing `flasher/ui/index.html?demo=NAME` in a browser. The Windows build is made by GitHub Actions
(`.github/workflows/build.yml`) with PyInstaller.

## Credits and licenses

MIT, see [LICENSE](LICENSE). Bundled: JieLi's WL82 USB loader (from
jl-uboot-tool), the open bootloader builds (Apache 2.0) and the window's fonts (OFL), see
[flasher/data/NOTICE.md](flasher/data/NOTICE.md). Thanks to kagaimiq for
jl-uboot-tool, and to the
[MvaveFM1Unbricker](https://github.com/Quixotic7/MvaveFM1Unbricker) and
[FM-1 Transporter](https://github.com/kurogedelic/FM-1-transporter) projects,
whose safety rules this follows.
