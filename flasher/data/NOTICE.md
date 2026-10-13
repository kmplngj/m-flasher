# Bundled files

| File | What | From | License |
| --- | --- | --- | --- |
| `wl82loader.bin` | JieLi's USB flash loader for the WL82 (AC791N), run in the chip's RAM in download mode | [jl-uboot-tool](https://github.com/kagaimiq/jl-uboot-tool) at `adb3f18`, `data/loaderblobs/usb/` (SHA-256 `d41da612…5376`) | a JieLi binary, distributed with jl-uboot-tool (MIT, `LICENSE-jl-uboot-tool`); JieLi has published no license for it |
| `bootloader-fm1.boot`, `bootloader-smk37.boot` | the open bootloader for each board | [fm1-bootloader](https://github.com/jvitkauskas/fm1-bootloader) (`make BOARD=fm1`, `make BOARD=smk37`); the version is in each file's identity block | Apache License 2.0 |
