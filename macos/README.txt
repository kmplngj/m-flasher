M-Flasher for macOS
===================

Back up, restore and reinstall the firmware of an M-VAVE FM-1 or SMK-37
Pro / Elite, and install the open bootloader.

macOS lets only an administrator take the unit's download mode (a USB disk
called "WL82 UBOOT1.00") from its own disk driver, so M-Flasher runs with
sudo. In Terminal, in the folder this came in:

    xattr -d com.apple.quarantine m-flasher     (once: it is not signed)
    sudo ./m-flasher                             (the window)
    sudo ./m-flasher info                        (or the command line)

Put the unit in download mode and connect it with a USB data cable:
  - with the open bootloader: hold REC (SMK-37: SEQ REC) while switching on;
  - Felucca, SLOOP, Snoop37: hold OCT- and OCT+ for 5 s while it runs.
If macOS says the disk is not readable, choose Ignore.

Backups go to Documents/M-Flasher/backups.

More: https://github.com/jvitkauskas/m-flasher
