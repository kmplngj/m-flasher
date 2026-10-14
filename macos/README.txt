M-Flasher for macOS
===================

Back up, restore and reinstall the firmware of an M-VAVE FM-1 or SMK-37
Pro / Elite, and install the open bootloader.

macOS lets only an administrator take the unit's download mode (a USB disk
called "WL82 UBOOT1.00") from its own disk driver, so M-Flasher runs with
sudo, from Terminal (not over SSH: macOS asks you to allow the access, and
can only ask on your screen). In Terminal, in the folder this came in:

    xattr -d com.apple.quarantine m-flasher     (once: it is not signed)
    sudo ./m-flasher                             (the window)
    sudo ./m-flasher info                        (or the command line)

The first time, macOS asks whether to allow access to the USB device:
choose Allow.

Put the unit in download mode and connect it with a USB data cable:
  - with the open bootloader: hold REC (SMK-37: SEQ REC) while switching on;
  - Felucca, SLOOP, Snoop37: hold OCT- and OCT+ for 5 s while it runs.
Start M-Flasher soon after: left alone with macOS's disk driver, the unit
may leave download mode and start its firmware after about half a minute.
Once M-Flasher has taken it, it stays in download mode until unplugged.

Backups go to Documents/M-Flasher/backups.

More: https://github.com/jvitkauskas/m-flasher
