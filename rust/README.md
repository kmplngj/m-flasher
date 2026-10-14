# M-Flasher in Rust

The same app as the Python version (`../flasher`), as one small native
executable: the window (egui), the command line, JieLi's flash loader, the open
bootloader builds and the fonts are all inside it. No runtime, no web view.

| | |
| --- | --- |
| `src/formats.rs` | flash and package formats: CRCs, ENC, file tables, boot area, `.fwsc` |
| `src/boards.rs` | the models and how a flash image identifies them |
| `src/sectormap.rs` | the flash map: a region for each of the 256 sectors |
| `src/scsi.rs` | SCSI pass-through: Linux SG_IO, Windows SCSI_PASS_THROUGH_DIRECT |
| `src/jl.rs` | the download-mode protocol and the flash loader |
| `src/ops.rs` | the operations and their safety rules |
| `src/controller.rs` | the window's logic in background threads |
| `src/ui.rs` | the window: the panel and the flash map, drawn with egui |
| `src/cli.rs` | the command line (`m-flasher info`, `backup`, ...) |

```sh
cargo build --release          # target/release/m-flasher
cargo test --release           # with FLASHER_TEST_DATA for the real-image tests
```

Without arguments it opens the window; `m-flasher --help` lists the commands.

The flash loader (`data/wl82loader-sdk.bin`) is JieLi's WL82 USB loader from
the [AC79 SDK](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK)
(`cpu/wl82/tools/wl82loader.bin`), published by JieLi under the Apache License
2.0. Fonts: OFL (`assets/OFL-*.txt`).
