# Bundled files

| File | What | From | License |
| --- | --- | --- | --- |
| `wl82loader-sdk.bin` | JieLi's USB flash loader for the WL82 (AC791N), run in the chip's RAM in download mode (stored scrambled, as the ROM expects it) | JieLi's [AC79 SDK](https://gitee.com/Jieli-Tech/fw-AC79_AIoT_SDK) at `d179b448`, `cpu/wl82/tools/wl82loader.bin` (SHA-256 `d9de9f1c…eb55`) | Apache License 2.0 (the SDK's license), (c) Zhuhai Jieli Technology |
| `bootloader-fm1.boot`, `bootloader-smk37.boot` | the open bootloader for each board | [fm1-bootloader](https://github.com/jvitkauskas/fm1-bootloader) `d16c403` (`make BOARD=fm1`, `make BOARD=smk37`) | Apache License 2.0 |
