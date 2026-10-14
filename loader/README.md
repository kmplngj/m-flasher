# M-Flasher's flash loader

A small program (3.4 KB) that M-Flasher runs in the chip's RAM from the JieLi
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
| FC40 | identify: "MFLR", version, its own address, features |
| FC41 | CRC32 of a flash range, computed on the chip (1 MiB in 0.8 s): a read is checked without reading it twice, a write without reading it back |
| FC42 | the flash chip's factory-unique ID (which unit this is) |
| FC43 / FC44 / FC45 | the unit's own display: on (board), a line of text, a progress bar |
| FC46 | status: the last refused command |

Not implemented, on purpose: burning the chip-key efuse, chip erase.

**Build.** With JieLi's pi32v2 toolchain: `./build.sh 0x01C20000` (and
`0x01C28000`: the app carries both, so a new loader is never written over a
running one). `rust/data/mflr-loader-a.bin` and `-b.bin` are the builds.

**Sources.** The SPI flash, efuse (P33) and display code and the register map
come from the [open bootloader](https://github.com/jvitkauskas/fm1-bootloader)
(Apache License 2.0); this folder is under the same license. The ROM
interface was worked out from JieLi's loader in the AC79 SDK.
