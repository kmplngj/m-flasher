# M-Flasher's flash loader

A small program (3.5 KB) that M-Flasher runs in the chip's RAM from the JieLi
WL82 (AC791N) mask ROM's USB download mode, instead of JieLi's 30 KB loader.

**How it works.** JieLi's loader is a plug-in for the ROM's own USB
mass-storage stack, and so is this one: the ROM's JUMP command calls
`loader_entry` with a pointer to `{send, recv, handler_slot}`; the loader puts
its handler in the slot and returns. The ROM then calls the handler for every
command (the USB command block, and a 16-byte answer buffer); the handler moves
data with `send()` / `recv()` and returns 1 when it handled the command, 0 to
leave it to the ROM (its memory read/write and jump keep working). A handler
must not return 0 after taking a command's data: the ROM cannot fail it then
(a refused write is "handled", nothing written, and reported by FC46).

**Commands.** The flash commands of JieLi's loader (so either loader works):
erase sector/block, write (CRC-checked), read, chip key, JEDEC ID, CRC16,
buffer size. And new ones:

| Command | |
| --- | --- |
| FC40 | identify: "MFLR", version, its own address, features, CPU clock |
| FC41 | CRC32 of a flash range, computed on the chip (1 MiB in 0.4 s): a read is checked without reading it twice, a write without reading it back |
| FC42 | the flash chip's factory-unique ID (which unit this is) |
| FC43 / FC44 / FC45 | the unit's own display: on (board), a line of text, a progress bar |
| FC46 | status: the last refused command |

Not implemented, on purpose: burning the chip-key efuse, chip erase.

**Speed.** The loader raises the CPU from the ROM's 48 MHz to 96 MHz (the
low-speed bus, which clocks the SPI flash and the display, stays at 48 MHz),
and reads the flash by DMA while the previous piece goes out over USB. A full
1 MiB read takes 1.2 s (860 KB/s; full-speed USB tops out near 1 MB/s), a
firmware install about 10 s in all.

The PC's USB controller sets the limit beyond that. Some collect only 3-4 of
the unit's 64-byte packets per millisecond, against 13-15 on others: about
230 KB/s, a 1 MiB read in 4.5 s, whatever the unit does. AMD's 600-series
chipset controller is one, on all of its ports (including a Thunderbolt card's,
whose USB 2 runs through the chipset); the CPU's own USB 3 ports on the same
PC read at full speed. M-Flasher logs the read speed, and a tip when the port
is slow.

**Build.** With JieLi's pi32v2 toolchain: `./build.sh 0x01C20000` (and
`0x01C28000`: the app carries both, so a new loader is never written over a
running one). `data/mflr-loader-a.bin` and `-b.bin` are the builds.

**Sources.** The SPI flash, efuse (P33) and display code and the register map
come from the [open bootloader](https://github.com/jvitkauskas/fm1-bootloader)
(Apache License 2.0); this folder is under the same license. The ROM
interface was worked out from JieLi's loader in the AC79 SDK.
