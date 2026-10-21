> Historical documentation archive, 2026-10-16. This is not the active
> implementation, current acceptance status, or permission to run commands.
> Local paths and digests are conservatively omitted; exact originals remain
> in the private recovery archive. See [archive policy](../README.md).

# M-VAVE FM-1 flashing, recovery and rollback: GitHub research

Research date: **2026-10-11**, Europe/Berlin. The device called “M-WAVE FM-1” in the request is usually named **M-VAVE FM-1 / Cuvave FM-1** in primary sources. This is a research snapshot and source review, not hardware qualification. No device was opened, enumerated, probed, reset, flashed, erased, or used for tests. Upstream source was read; upstream scripts were not executed.

## Findings that determine the plan

1. **A desktop CLI can install directly from a compatible running firmware over USB-MIDI.** Felucca already provides a Python CLI and a Web MIDI installer using the same device-pull protocol. The host serves requested package ranges; the incoming package's RAM loader performs flash writes. Existing CLI code is a useful reference, but its success message establishes post-boot identity, not a full raw-flash audit or universally safe migration. [Felucca installer](https://github.com/hugelton/Felucca/blob/ef1f561b6b105f20350f6b1eac0dc0789a357f00/tools/fm1_install.py#L1-L22), [device staging code](https://github.com/hugelton/Felucca/blob/ef1f561b6b105f20350f6b1eac0dc0789a357f00/firmware/src/ota.c#L265-L342).
2. **An intermediate V14/V15 firmware hop is not a general safety solution.** It is one workaround for some official updater same-version refusals. A documented Felucca 0.9-beta → 1.0.x boot failure persisted after a V15 hop because V15 retained the incompatible custom data. Direct support must therefore be a graph of exact source/target firmware and data-layout profiles, with unknown edges refused. [Patcher's V14 workaround](https://github.com/czietz/fm1-firmware-patcher/blob/4cfab602455168d0670fdf2ccf6979ef3b0bea3b/README.md#L35-L47), [V15 did not remove the failing records](https://github.com/hugelton/Felucca/issues/61#issuecomment-6019222152), [ChoralRoot compatibility policy](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/INSTALL-COMPAT.md#L10-L44).
3. **A plain USB cable cannot recover every black-screen case.** When the ROM already enumerates as `4C4A:8057`, `WL82 UBOOT1.00`, a normal cable plus a native SCSI transport can restore the app on Windows/Linux. When there is no USB enumeration, an independent electrical `USB_KEY` source is needed. FM-1 Transporter uses a Seeed XIAO RP2040 on D+/D−/GND and exposes a serial bridge usable from macOS. [Unbricker recovery distinction](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/README.md#L12-L35), [Transporter hardware](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/README.md#L3-L53).
4. **The ordinary stock rollback path can physically rewrite the boot head.** Felucca accepts the official incoming loader only after checking the package head against the installed head, because that loader writes it. Identical bytes do not eliminate the risk during erase/program. A project requirement that `[0,0x4000)` is never physically erased/written therefore needs an independent application-region recovery/rollback writer; standard stock OTA cannot be called write-free for that region. [Explicit loader distinction and head check](https://github.com/hugelton/Felucca/blob/ef1f561b6b105f20350f6b1eac0dc0789a357f00/firmware/src/ota.c#L269-L300).
5. **Single-bank writes are not atomic.** A ChoralRoot install interrupted on the author's only unit left a byte-correct head, a valid staged loader and record, but a mixed old/new app and no USB device. Recovery required Transporter. New ChoralRoot boot-stub work materially improves recovery for an unchanged installed stub, but first installation and stub replacement retain unprotected erase/page windows. No source establishes an unconditional “will never brick” guarantee for arbitrary firmware on the user's unknown device revision. [Actual failed update and successful recovery](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/TRANSPORTER-HANDOFF.md#L145-L170), [interruption model](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md#L365-L403).

**Consequence for the single available device:** development can meet a strict no-bricking policy by doing no device mutation during development, using offline fixtures, simulated NOR flash and replayed transport. Writer qualification must use a spare/external unit or independently supplied exact qualification evidence, with prepared recovery hardware. Obtaining a dongle and backup does not authorize development writes on the owner's sole device. Backups alone do not supply a way to reach a non-enumerating chip. This is a project-design inference from the above evidence, not a claim that a simulator proves hardware safety.

## Search scope and evidence levels

The search followed two live resource directories, GitHub repository search, source files, issue comments, pull requests, release metadata and selected forks. It used the aliases M-WAVE, M-VAVE, FM-1 and FM1. This is bounded coverage of public, reachable material on the research date. Private/deleted repositories, all historical branches, all fork diffs, all issue/discussion pages, binary runtime behavior and future updates are outside the completed scope.

| Search operation | Result / limit |
| --- | --- |
| Web: `site:github.com M-WAVE FM-1 firmware flash unbrick`, `site:github.com "M-VAVE" "FM1" firmware`, `site:github.com "FM-1" "bootloader"` | Discovered hubs and principal firmware/recovery projects; cached search results were checked against live repositories. |
| GitHub REST: `"FM-1" firmware fork:false`, 100 results/page | 41 hits, `incomplete_results=false`; unrelated FM151 and FM1828/lidar results were not treated as synth firmware. |
| GitHub REST: `"FM1" flasher fork:false`, `"FM-1" recovery fork:false`, `"FM-1" unbrick fork:false` | One hit each: aroum hub, Flipper recovery, MvaveFM1Unbricker. Directory traversal was necessary to find other important tools. |
| GitHub REST: `"M-WAVE" "FM-1" fork:false` | Three hits, mainly helpers/emulators rather than device writers. |
| GitHub connector: `"FM-1" firmware`, pages 1–2, 100/page | Inspected candidate/fork inventory; duplicates and unrelated matches were not considered independent recovery evidence. |
| GitHub connector: global `"FM-1" (brick OR flash OR install OR recovery OR rollback)` and targeted Felucca/Melodee searches | Up to 100 results per search; relevant threads were read with all returned comments, including #61, #32, #13, #120, Melodee #4, GHOULBOX #4 and Transporter #1. Search includes noisy matches; it is not a complete discussion archive. |
| Directory traversal | 82 repository URLs screened at exact HEAD: 81 resolved, one unreachable; two resolved repositories had no root README. Inventory below. The old ip2k URL resolves to the same Lunar repository. |
| Source review | 17 principal repositories cloned read-only into temporary directories; no build/install/update scripts run. Other inventory entries were README/metadata screening, explicitly not full source audits. |
| Releases | Latest four public release records of Felucca, SLOOP, Melodee and ChoralRoot; Transporter, Unbricker and Flipper recovery had no public GitHub release entries at lookup time. |

**Evidence tiers used below:** **C** = current source code inspected; **H** = public author/user report of an actual hardware run, not reproduced here; **D** = documentation claim; **I** = inference or proposed integration decision. Repository existence, stars, successful compilation, host tests and a release attachment do not establish device safety.

The discovery directories are useful navigation, but contain overgeneralizations and stale statements. For example, “all custom firmware needs V15 first” conflicts with ChoralRoot's explicit older-official policy; generic “resume after power cycle” conflicts with the later Transporter dump. The cached `kurogedelic/fm-1-research-lab` search result returned live GitHub API 404. [aroum hub snapshot](https://github.com/aroum/fm1-custom-fw/blob/6536e06c72ef812ce84ab715208d62311f660425/README.md), [awesome list snapshot](https://github.com/cicloid/awesome-fm-1/blob/b31003409227464dd781f9a2f03d9d9b62b40d67/README.md).

## Principal upstream projects

| Project / exact source snapshot | What it contributes | Assessment |
| --- | --- | --- |
| [AL-255/FM-1-RE, `ec832f281ca165a7d5e0722985f3acb1465046d5`](https://github.com/AL-255/FM-1-RE/tree/ec832f281ca165a7d5e0722985f3acb1465046d5) | Reverse-engineered two-phase OTA protocol, stock layout, Linux ALSA client, offline regression tests, retained successful Windows reflash evidence. | C/H. Good protocol origin. A Windows adapter's hardware success does not qualify the Linux transport. |
| [hugelton/Felucca, `ef1f561b6b105f20350f6b1eac0dc0789a357f00`](https://github.com/hugelton/Felucca/tree/ef1f561b6b105f20350f6b1eac0dc0789a357f00) | Cross-platform `mido`/`python-rtmidi` installer, browser installer/parser, app staging and incoming loader, backup protocol. | C/H. Main lineage for other custom firmware. GPL-3.0-only; historical boot failures documented below. |
| [isod89/sloop-fm1, `4365a1d0075b9194de1c44826325abbd39751bc1`](https://github.com/isod89/sloop-fm1/tree/4365a1d0075b9194de1c44826325abbd39751bc1) | Felucca-derived install, loader integrity/JEDEC checks, OCT− application rescue, versioned project/backup behavior. | C/D. Source has generic cold-resume assumptions; application rescue is unavailable when a partial app cannot reach it. |
| [keremimo/melodee, `01a78bdf0fb0e1a9d953567e36a59d80dea54a8a`](https://github.com/keremimo/melodee/tree/01a78bdf0fb0e1a9d953567e36a59d80dea54a8a) | Felucca-derived install and backup; user-facing skip-backup rollback path; migrated sound storage and single-core release alternative. | C/H. Illustrates capability/backup/version coupling; old data needs explicit policy. |
| [Quixotic7/ChoralRootFM1, `e8efc993c986f8d7661c515ebfc8c46c6e77057c`](https://github.com/Quixotic7/ChoralRootFM1/tree/e8efc993c986f8d7661c515ebfc8c46c6e77057c) | Source-firmware classifier; exact boot-failure investigation; Transporter recovery; boot stub, body marker and exhaustive modeled interruption cuts. | C/H. Most useful concrete safety evidence, with remaining boot-head/stub/firmware compatibility limits. |
| [kurogedelic/FM-1-transporter, `a632d923203170e3a08565f4056dbb3fd3d65ca5`](https://github.com/kurogedelic/FM-1-transporter/tree/a632d923203170e3a08565f4056dbb3fd3d65ca5) | RP2040 USB_KEY and USB host bridge; full-flash dump; application-only, changed-sector writes; serial host protocol. | C/H. Preferred practical macOS recovery architecture; needs physical dongle and external RAM-loader artifact. |
| [Quixotic7/MvaveFM1Unbricker, `b8c8e8afbcba3e45b262b3b624ab9d0244b92628`](https://github.com/Quixotic7/MvaveFM1Unbricker/tree/b8c8e8afbcba3e45b262b3b624ab9d0244b92628) | Native Windows/Linux soft-brick recovery; cross-platform Transporter recovery, double dumps, fresh baseline comparison, verification, mock NOR and fault tests. | C/H. Strong reference, but narrow classifiers/region guards must be strengthened before a universal multi-layout writer. |
| [merthsoft/flipper-fm1-recovery, `df88b862a4794855f792fed529629a255cba0a0e`](https://github.com/merthsoft/flipper-fm1-recovery/tree/df88b862a4794855f792fed529629a255cba0a0e) | Flipper Zero USB_KEY, cable handoff, Windows native app-region backup/restore, independent format validation. | C/H. Real reported 2026-10-09 recovery; alternative trigger if Flipper is already available. |
| [kagaimiq/jl-uboot-tool, `adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a`](https://github.com/kagaimiq/jl-uboot-tool/tree/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a) | Low-level UBOOT/SCSI protocol, Windows/Linux transports, `wl82loader.bin`. | C. Generic interactive erase/write commands are too broad for the single-device product policy. |
| [fuleo/fm1-linux-update, `fa850941dbd69f9172ed78dea5a1d01178a5fa2d`](https://github.com/fuleo/fm1-linux-update/tree/fa850941dbd69f9172ed78dea5a1d01178a5fa2d) | Pinned/hash-checked Felucca installer and official V15 wrapper. | H: one Ubuntu 22.04 V14→V15 run. Its stale claims about hardware recovery/stock `--force` must not be copied as current universal rules. |
| [seajaysec/fm-static, `0fa627434b7aa549af94dc819540be41f6c4dcbc`](https://github.com/seajaysec/fm-static/tree/0fa627434b7aa549af94dc819540be41f6c4dcbc) | Renamed/cached MIDI port discovery, explicit opt-in for unknown ports, re-enumeration tracking. | C/D. Useful discovery lessons; not a ROM unbricker. |
| [Keitark/fm1-nes, `90d41c5e48073af2d4f7ff94df7496004d4b6627`](https://github.com/Keitark/fm1-nes/tree/90d41c5e48073af2d4f7ff94df7496004d4b6627) | Sparse application-only `.fm1app` package/planner, protected layout and private-baseline reconstruction, exact external-tool guard patch. | C/H. Bounded V14 bench result differs from newer wrapper's host-only qualification. No copied raw `app.bin` writes. |
| [jvitkauskas/fm1-bootloader, `12b570b167b6695e60be18800c609730ab75bc09`](https://github.com/jvitkauskas/fm1-bootloader/tree/12b570b167b6695e60be18800c609730ab75bc09) | Readable C SPL reconstruction and boot flow, optional generic dual-bank selection retained. | C/D. Changed readable build explicitly untested on hardware; replacing the boot head is outside this project's allowed testing scope. |
| [czietz/fm1-firmware-patcher, `4cfab602455168d0670fdf2ccf6979ef3b0bea3b`](https://github.com/czietz/fm1-firmware-patcher/tree/4cfab602455168d0670fdf2ccf6979ef3b0bea3b) | Localized stock V15 binary patches; documented same-version updater obstacle and V14 hop. | C/H. The patched vendor image is not redistributable; a direct same-identity edge needs its own approved method. |
| [aroum/fm1-custom-fw, `6536e06c72ef812ce84ab715208d62311f660425`](https://github.com/aroum/fm1-custom-fw/tree/6536e06c72ef812ce84ab715208d62311f660425) | Resource hub, updater analysis, embedded firmware extraction. | C/D. `fm1_flasher.py` explicitly says its writer is experimental/non-functional: wrong host-push assumptions. Exclude it as a working writer base. |
| [ip2k/lunar-modulator, `b335dc2c450473dc375163aec7fa679e1b099cc3`](https://github.com/ip2k/lunar-modulator/tree/b335dc2c450473dc375163aec7fa679e1b099cc3) | Evidence-tiered stock hardware/format research, risk register, USB_KEY design, verification ladder. | C/D. Research/preview; no Lunar firmware installed on an FM-1. Its historical hypotheses must be reconciled with later actual recovery records. |
| [cicloid/awesome-fm-1, `b31003409227464dd781f9a2f03d9d9b62b40d67`](https://github.com/cicloid/awesome-fm-1/tree/b31003409227464dd781f9a2f03d9d9b62b40d67) | Wider community index, recovery/install/tool links. | D. Discovery only; not proof of each listed firmware's compatibility/safety. |

### Releases versus main branch

At lookup, the latest public releases were [Felucca v1.5.1](https://github.com/hugelton/Felucca/releases/tag/v1.5.1) (2026-10-10), [SLOOP v2.6](https://github.com/isod89/sloop-fm1/releases/tag/v2.6) (2026-10-11), [Melodee v1.1.0](https://github.com/keremimo/melodee/releases/tag/v1.1.0) (2026-10-10) and [ChoralRoot v0.17](https://github.com/Quixotic7/ChoralRootFM1/releases/tag/v0.17) (2026-10-11). Melodee publishes normal and single-core `.fwsc` variants, `BUILD_INFO.json` and `SHA256SUMS`; ChoralRoot publishes hashes and licensing files. Main-branch claims about 0.18 gestures are not automatically v0.17 release behavior. The plan must bind the source SHA, exact release asset hash and profile; it must not install whichever file a mutable web page currently advertises.

### Official V15 artifact: upstream fingerprint, not a fresh download

The vendor URL is [official FM-1.fwsc](https://yms-file-store.oss-cn-hongkong.aliyuncs.com/software/firmware/FM-1.fwsc). Two bounded HTTPS attempts on 2026-10-11 did not retrieve any bytes: the first timed out, and the second failed to connect within eight seconds. Therefore **this research did not independently verify the current live blob, its size, Last-Modified header or hash**. Do not present the following upstream constants as a newly verified vendor artifact.

The pinned fuleo snapshot reports `FM-1_015`, 699,956 bytes, Last-Modified 2026-07-30 and whole-file SHA256 `[digest omitted from public archive]`. The pinned Unbricker independently encodes that same whole-file constant and application-region `[0x4000,0x93000)` SHA256 `[digest omitted from public archive]`. These are **D/C** reference fingerprints. A future local user-supplied/downloaded package must be hashed, parsed and checked against the exact approved profile before becoming a usable recovery input. It must not be redistributed by this project. [Upstream measured metadata](https://github.com/fuleo/fm1-linux-update/blob/fa850941dbd69f9172ed78dea5a1d01178a5fa2d/README.md#L60-L69), [Unbricker constants and offset warning](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/README.md#L120-L139).

## Device and address-space facts

The AC791N/WL82 family and pi32v2 CPU are consistently identified by image and SDK evidence. The exact FM-1 chip/package subtype is less certain: photo markings and one source point to AC7911B8, while Lunar records a package/pin-count ambiguity. A hardware profile must be established from the actual unit rather than inferring it from a product name. The cited units report a **1 MiB NOR flash**, chip key `0x980F`, JEDEC ID `0x856014`. Do not treat these constants as an invitation to bypass a different unit's identity. [Hardware evidence and uncertainty](https://github.com/ip2k/lunar-modulator/blob/b335dc2c450473dc375163aec7fa679e1b099cc3/docs/01-hardware.md#L10-L29), [observed dump/chip information](https://github.com/hugelton/Felucca/issues/61#issuecomment-6018813157), [Unbricker strict identity constants](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/fm1_unbrick.py#L49-L82).

| Mode | Evidence / transport | Meaning for discovery |
| --- | --- | --- |
| Stock running | `4C4A:C755`, USB-MIDI plus UAC audio, often `FM-1` or `FM-1 Midi` | USB name is a hint, not sufficient proof of model or exact firmware. |
| Stock OTA loader | `4D4A:4155`, `ota-FM-1`, still USB-MIDI | Distinct from ROM mass-storage mode; re-query identity after re-enumeration. |
| ROM download | `4C4A:8057`, `WL82 UBOOT1.00`; some host strings say WL80 | Native vendor SCSI commands over mass storage / bulk; MIDI installers cannot speak to it. |
| RP2040 Transporter | `2E8A:000A`, two CDC serial interfaces in the reported run | Console and data interfaces differ; select the protocol responder, not the first serial port. |

[Observed normal/OTA IDs](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/11-ota-protocol.md#L13-L29), [actual ROM IDs](https://github.com/hugelton/Felucca/issues/61), [Transporter enumeration](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/TRANSPORTER-HANDOFF.md#L150-L157).

The crucial distinction is **file offset ≠ logical package address ≠ physical flash offset ≠ CPU/XIP address**: The XIP base itself is profile-dependent: AL-255’s stock analysis uses `XIP = physical − 0x4120 + 0x02000000`, while ChoralRoot’s as-built layout uses `XIP = physical − 0x4000 + 0x02000000`. Preserve that distinction instead of inferring a universal entry address. [Stock mapping](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/architecture.md#L63-L66), [ChoralRoot mapping](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md#L238-L247).

| Physical interval / address | V15-compatible meaning | Rule / qualification |
| --- | --- | --- |
| `[0x00000,0x04000)` | Flash header, SPL at approximately `0xA0`, configuration around `0x38D0` | Protected from every write/erase in the proposed tool. |
| `[0x04000,0x93000)` | V15-compatible application area, JLFS records and application/config resources; 143 × 4 KiB | Profile-specific region, not universal for all images. |
| `0x04120` | Physical beginning of encoded application bytes | A raw decrypted/linked `app.bin` is not a ready-to-program raw flash region. |
| `0x02000000` / `0x02000120` | Profile-dependent CPU/XIP mapping and entry | Stock disassembly maps physical `0x4120` to XIP `0x02000000`; ChoralRoot/Keitark layout maps physical `0x4000` to XIP `0x02000000`, with stub/app entry at `0x02000120`. Never use a CPU address as a physical erase/write address. |
| `[0x93000,0xE9000)` | V15 VM storage | V13/V14 VM started at `0x94000`; profile changes matter. |
| `[0xE9000,0xEA000)` / `[0xEA000,0xFC000)` | BTIF / stock user-preset regions | Preserve device-specific contents unless a separate approved migration is specified. |
| `0xFF000` | `key_mac`/device-specific region in the documented stock layout | Back up privately; no generic replacement. |
| `[0xE0000,0xE5000)` | Felucca-family temporary staged OTA loader, flash record at `0xE4F00` | Not the immutable mask ROM; overlaps stock VM ownership and must be profile-specific. |
| `0x01C0A800` | RAM execution address of the update loader | RAM upload/run is a state-changing operation even if no flash write is requested. |
| `0x01C7FD88` | RAM update handoff record | Cold power loss can clear it; the corresponding flash record does not guarantee cold-loader resume. |

[Stock layout and version differences](https://github.com/ip2k/lunar-modulator/blob/b335dc2c450473dc375163aec7fa679e1b099cc3/docs/01-hardware.md#L84-L111), [application-only address distinctions](https://github.com/Keitark/fm1-nes/blob/90d41c5e48073af2d4f7ff94df7496004d4b6627/APP_UPDATES.md#L24-L50), [actual staged-loader record failure](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md#L22-L41).

Baud Girl FM-1_092 expands the app/PRCT to approximately `0xD9000` and moves VM there; it is **not** the V15/Felucca `[0x4000,0x93000)` layout. SLOOP uses object 7 for autosave at `0x9F000/0xFE000`, while ChoralRoot uses that object/location for an FM6 bank. zp12 reports samples at `0xC4000..`, where stock SLOOP keeps different samples. These are concrete reasons to inspect data schemas and owners before cross-installing or restoring a backup. [Baud Girl layout](https://github.com/ip2k/lunar-modulator/blob/b335dc2c450473dc375163aec7fa679e1b099cc3/docs/01-hardware.md#L95-L104), [object collision investigation](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/INSTALL-COMPAT.md#L74-L86), [zp12 storage note](https://github.com/zvenson/zp12/blob/610f330423ae8c37e1b5137b11f04c9c185553bc/README.md#L35-L41).

## OTA protocol and package invariants

**C/H:** the protocol is device-pull USB-MIDI SysEx, with two phases: the running app validates/stages the incoming loader, then that loader requests the package and programs it. The handshake is `F0 00 32 45 00 00 00 40 7F F7`; upgrade is `F0 22 24 35 7F F7`. The separate `...7D F7` soft key can cause ROM entry and is a state mutation, not a discovery probe. Do not send guessed command families 33–36 or 48 to the sole device. [Protocol and hazardous commands](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/11-ota-protocol.md#L31-L162), [known soft key](https://github.com/ip2k/lunar-modulator/blob/b335dc2c450473dc375163aec7fa679e1b099cc3/docs/07-recovery-and-risk.md#L112-L131).

The complete decoded read/response frame is `00 59 30 | body_length:u24le | flash_type:u8 | address:u32le | requested_length:u24le | data | checksum:u8`, packed as a continuous LSB-first 8→7 bitstream between F0/F7. `body_length=8` for requests, `len(data)+8` for responses; checksum is the complement of the byte sum from flash type through data. This is not the common “seven data bytes plus an MSB bitmap” encoding. Full 32-bit sentinel addresses must survive parsing. A 481-byte final block was the concrete historical failure: a fixed packed prefix declared the wrong body length. [Current protocol](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/11-ota-protocol.md#L55-L77), [merged correction and retained hardware sequence](https://github.com/AL-255/FM-1-RE/pull/2), [non-working aroum writer](https://github.com/aroum/fm1-custom-fw/blob/6536e06c72ef812ce84ab715208d62311f660425/fm1_flasher.py#L1-L50).

`0xE0000000`, length 8, is phase-one verification completion; `0xF0000000`, length 8, is the loader's post-write finish request. Both expect `success\0`. Neither proves a healthy boot or usable sound/data. The original updater waits approximately 2 seconds after upgrade, keeps the phase-one connection open approximately 3 seconds after acknowledgement, and uses an 8-second receive timeout; current custom hosts use different phase-specific idle/re-enumeration timeouts. Timing is a tested transport/profile parameter rather than a universally safe speed-up opportunity. [Terminal signals](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/docs/io/11-ota-protocol.md#L94-L110), [current host delays](https://github.com/hugelton/Felucca/blob/ef1f561b6b105f20350f6b1eac0dc0789a357f00/tools/fm1_install.py#L40-L50).

The first 960 raw `.fwsc` bytes are 20 blocks of 47 payload bytes plus one identity marker. Removing the 20 markers creates the logical package image; the rest is appended unchanged. UFW encrypted header/entry metadata must identify the type-0 `flash.bin` entry. **Physical flash byte X is `logical[flash_entry_offset + X]`**, not `logical[X]`. In common packages the flash entry begins at logical `0x400`, but it must be parsed, not assumed. The Unbricker's older code had the wrong slice and could have written a region shifted by `0x400`; the issue was fixed on 2026-10-07. [Correct extractor source](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/fm1_unbrick.py#L189-L318), [published correction](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/README.md#L145-L156).

Current independent source uses CRC-16 polynomial `0x1021`, initial value **0**, no final XOR. Some older reverse-engineering prose says CCITT-FALSE, which would imply `0xFFFF` initialization; do not copy the name without the actual algorithm. CRCs and SHA-256 provide integrity/reference matching, not proof of a trusted publisher or cryptographic secure boot. Sources reviewed do not establish a cryptographic firmware-authentication barrier. [Extractor CRC](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/fm1_unbrick.py#L219-L241), [corrected format note](https://github.com/ip2k/lunar-modulator/blob/b335dc2c450473dc375163aec7fa679e1b099cc3/docs/02-stock-firmware.md#L38-L47), [same-version experiments](https://github.com/ip2k/lunar-modulator/blob/b335dc2c450473dc375163aec7fa679e1b099cc3/docs/03-update-protocol.md#L148-L212).

## Direct-install and rollback compatibility

The requirement “without switching to old/other firmware” is feasible as a default goal for **proved compatibility edges**, not as an unrestricted promise for every three-digit identity or arbitrary `.fwsc`. Identities collide: ChoralRoot and SLOOP can both report `FM-1_920`, and development builds share `FM-1_900`. A family INFO/capability response, exact asset hash, loader identity and layout/data profile are needed in addition to the handshake. [ChoralRoot collision/classifier explanation](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/INSTALL-COMPAT.md#L10-L44).

| Source → target | Current evidence | Policy for this project |
| --- | --- | --- |
| Stock V14 → stock V15 | One Ubuntu 22.04 live run with pinned Felucca host wrapper. | Candidate compatibility edge; actual device/profile still needs qualification. |
| Stock V15 → Felucca-family custom firmware | Public install path and current staging/loader source. | Candidate direct edge for exact packages. Do not make an intermediate downgrade mandatory. |
| Official older than V15 → ChoralRoot | ChoralRoot explicitly allows `FM-1_001..099`; this is installer policy, not qualification of every historical device/stock version. | Preserve the distinction; unknown actual version stays read-only. |
| Felucca ≥1.0 / Melodee / ChoralRoot → ChoralRoot | Allowed by ChoralRoot classifier. | Candidate direct edges scoped to known INFO/profile and package. |
| SLOOP / SLOOP rescue / Felucca ≤0.9-beta → ChoralRoot | Explicit refusal following a real black-screen/ROM case; host boot did not reproduce cause. | Refuse by default; no dangerous generic `--force` escape hatch in the single-device developer build. |
| Felucca 0.9-beta → 1.0.1/1.0.2/1.0.3 | DIGITAL ORGAN project could trigger divide-by-zero at boot, despite a complete byte-correct app write. | Historical unsafe edges; fixture required. Fix released/tested at 1.0.3.1. |
| Running compatible Felucca-family → official V15 | Existing web/CLI path, hash-checked V15; public Windows user report. Incoming official loader may rewrite head. | Ordinary OTA path conflicts with the strict no-head-write requirement; use independent audited application-only restore for that policy. |
| Stock V15 → czietz patched V15, same identity | Official updater refuses same-version; author uses V14 hop. | Direct OTA unresolved. Do not modify image identity or installed data merely to bypass policy. ROM/Transporter app-only restore is a candidate alternative after exact package validation. |
| ROM/Transporter → verified stock V15 | Hardware reports and source gate to app region. | Independent recovery path; no intermediate firmware needed to make the physical write, but fresh baselines and post-boot checks remain mandatory. |
| Arbitrary custom → arbitrary custom | Shared protocol does not establish shared data/boot/partition behavior. | Unknown edge refused with a concrete reason; no automatic stock hop. |

[Linux report](https://github.com/fuleo/fm1-linux-update/blob/fa850941dbd69f9172ed78dea5a1d01178a5fa2d/README.md#L1-L19), [ChoralRoot policy/diagnosis](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/INSTALL-COMPAT.md#L10-L124), [fixed Felucca hardware report](https://github.com/hugelton/Felucca/issues/61#issuecomment-6028357685), [Windows rollback report](https://github.com/hugelton/Felucca/issues/32#issuecomment-5997242064).

Backup types must also be distinct. A Felucca-family “complete backup” is a versioned semantic export of music/presets/samples/settings, not a byte-identical 1 MiB recovery dump. Stock firmware may not export those semantics. SLOOP 2.6 projects do not reopen in 2.5; 2.4 projects do not reopen in 2.3 and the older autosave can overwrite them. Melodee migrates Prophet storage and older versions/stock can erase slots. Require source/target schema validation and retain the original dump; never silently reinterpret an unsupported semantic export. [SLOOP downgrade limits](https://github.com/isod89/sloop-fm1/blob/4365a1d0075b9194de1c44826325abbd39751bc1/README.md#L63-L82), [older project/backup limits](https://github.com/isod89/sloop-fm1/blob/4365a1d0075b9194de1c44826325abbd39751bc1/README.md#L250-L258), [Melodee storage migrations](https://github.com/keremimo/melodee/blob/01a78bdf0fb0e1a9d953567e36a59d80dea54a8a/README.md#L120-L163).

## Recovery levels and OS coverage

| Observed state | Usable recovery path | Limit |
| --- | --- | --- |
| App runs and MIDI update service replies | Two-phase MIDI update / rollback | Cannot help an app that never reaches the service. |
| App rescue still runs | Firmware-specific OCT− rescue / safe mode | A key chord in the app is not independent of damaged app code. |
| OTA loader replies | Finish/resume using the **same identified transaction/package** | Do not infer cold resume after power loss from a record existing in flash. |
| ROM mass-storage enumerates | Native SCSI + WL82 RAM loader + backup/restore | Native jl-uboot-tool supports Windows/Linux; upstream macOS SCSI backend absent. |
| No USB device enumerates, intact hardware | USB_KEY electrical trigger at power-on, then independent restore | Seeed XIAO RP2040 Transporter or qualified Flipper alternative; cable alone cannot generate arbitrary D+/D− key signaling. |
| Boot head corrupt or physically damaged chip | Beyond the audited app-only recovery tools | Neither cited protected writer restores `[0,0x4000)`; physical/electrical failure is not software-recoverable evidence. |

[Unbricker platform/state requirements](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/README.md#L12-L140), [Transporter actual recovery](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/TRANSPORTER-HANDOFF.md#L145-L170), [Flipper successful run](https://github.com/merthsoft/flipper-fm1-recovery/blob/df88b862a4794855f792fed529629a255cba0a0e/README.md#L1-L13).

| Host | Running/OTA MIDI | ROM native recovery | Transporter bridge |
| --- | --- | --- | --- |
| macOS arm64 / x86_64 | CoreMIDI via RtMidi; existing Python CLI reference | Not implemented by upstream jl-uboot-tool/MvaveFM1Unbricker; enumerating an unreadable disk is not a usable backend | Serial bridge source; a real arm64 recovery reported, no Rosetta needed for host communication. |
| Linux | ALSA sequencer reference; access normally needs appropriate user/group permissions | `/dev/sgN`, `sg` driver, privileged access as currently documented | Serial bridge; permissions/udev need platform qualification. |
| Windows 10/11 | RtMidi Windows backend / Web MIDI Chromium references; ports may be exclusive | `\\.\PhysicalDriveN`, administrator; exact inquiry/USB identity required | Serial bridge; distribution/driver and COM selection require qualification. |

The generic `jldevfind.py` volume scan may miss the physical ROM disk on Windows. Select by actual inquiry/device identity, not a guessed disk number. Linux raw MIDI can conflict with sequencer subscribers/PipeWire; prefer the normal sequencer backend. macOS caches MIDI device names by USB identity/socket and may show another firmware's name, while different firmwares share identities and may lack a serial. [Windows finder limitation](https://github.com/hugelton/Felucca/issues/61#issuecomment-6018434624), [Linux transport note](https://github.com/ip2k/lunar-modulator/blob/b335dc2c450473dc375163aec7fa679e1b099cc3/docs/03-update-protocol.md#L217-L221), [renamed/cached ports](https://github.com/seajaysec/fm-static/blob/0fa627434b7aa549af94dc819540be41f6c4dcbc/README.md#L5-L30).

### Hardware preparation for the sole device

The primary Transporter project recommends the standard **Seeed XIAO RP2040**, with D6/GP0 → D+, D7/GP1 → D− and GND → GND, **no VBUS connection**; the FM-1 runs from its battery. Other RP2040 boards are untested there; Waveshare RP2350-USB-A is explicitly unsupported. The planned bridge must be checked against this exact wiring/firmware, not any similarly named board. The standard board is the [Seeed XIAO RP2040 product, SKU 102010428](https://www.seeedstudio.com/XIAO-RP2040-v1-0-p-5026.html), not an unqualified alternate/Plus/RP2350 board. A suitable USB breakout exposes data lines and ground while keeping VBUS disconnected. These are preparation references, not permission to wire or connect hardware in this research task. [Primary wiring/board restrictions](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/README.md#L25-L53).

Transporter implements the `0x16EF` USB_KEY signal at power-on and then acts as the USB host for ROM communication. A full flash double-read backup, matching chip/flash identity and fresh baseline are required before any write. Original `tools/fm1t.py` refers to extraction code from `fm-1-research-lab`; that unavailable external dependency is a reproducibility gap. The current standalone Unbricker parser/Transporter wrapper is a more practical reference to audit than dynamically importing a missing directory. [Transporter host dependency](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/tools/fm1t.py), [standalone recovery wrapper](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/fm1_transporter_recover.py).

## What the boot-stub work actually establishes

ChoralRoot documents a 2026-10-07 interruption at request 274 of approximately 1167: 33 sectors of new firmware followed by 110 complete old sectors. On 2026-10-09 the Transporter dump established the head was stock V15, the loader intact, and the flash update record valid. Yet no cold boot ran that loader. The SPL uses a RAM update record when present and can jump into an unchecked mixed app body; simply retaining the flash record does not create power-loss recovery. [Dump and boot behavior](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md#L22-L62).

The implemented stub lives in first app sector `0x4000`, jumps to the real body at `0x5000`, checks a marker at `0x91FE0` and body CRC, and can hand over to the staged loader or ROM. The actual marker sector is `0x91000`, not the historical design's `0x92000`; historical prose earlier in the same document is superseded by “As built.” Compensation bytes keep the directory/app CRCs fixed so a stable stub sector can remain byte-identical across body releases. The loader invalidates the marker, writes a differing stub if required, writes the body, and writes the marker last. [Implemented layout/marker](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md#L235-L310), [constant first-sector design and actual write order](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md#L339-L403).

**H:** a later section in that same pinned document reports actual stub-v3 unit tests on 2026-10-10: OCT− at cold power-on produced LED flashes and `ota-FM-1`, and a cable pull at writing 5% followed by power-cycle resumed into the loader and completed installation. An earlier heading still says v3 host-verified and an earlier list still calls cold handover/key sampling open; use the later dated result and retain its qualification as one author's unit, not independently reproduced here. [Later unit results](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md#L483-L498).

**C/D:** the source's modeled cut table covers 1854 interruption points for same-stub body updates with zero unprotected cuts, but first/no-stub installs and changed-stub installs retain 16 unprotected cuts (and a mixed old app case has 17). Rough quoted exposure is 0.1–0.5 seconds during stub erase/program. Those are simulator/design results, not a probability or a no-brick guarantee. The second failed handover/early ROM-entry path and some release-lock checks remained untested in the later unit report. This project must not deliberately reproduce cable pulls, brownouts, torn pages or stub replacement on its sole device. [Modeled uncovered windows](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md#L381-L403), [remaining unit checks](https://github.com/Quixotic7/ChoralRootFM1/blob/e8efc993c986f8d7661c515ebfc8c46c6e77057c/docs/BOOT-SAFETY.md#L493-L498).

## Issue and fork findings that should become regression fixtures

| Primary thread | Finding | Integration consequence |
| --- | --- | --- |
| [Felucca #61](https://github.com/hugelton/Felucca/issues/61) and [PR #111](https://github.com/hugelton/Felucca/pull/111) | Byte-correct 0.9→1.0.1/1.0.2 app installs failed; V15 hop retained old records. Actual compiler moved loop-invariant divides ahead of guards in DIGITAL→FM6 conversion, with ORGAN zero input. Hardware fix reported in v1.0.3.1. PR #111 itself closed unmerged; do not conflate proposal state with the later release incorporation/report. | Boot/runtime compatibility is separate from package/write correctness. Host x86 compilation may not expose pi32v2 compiler behavior. Retain exact data fixtures and boot acceptance. |
| [AL-255 PR #2](https://github.com/AL-255/FM-1-RE/pull/2) | Merged at `ec832f2`; 481-byte response framing fixed. Windows modified V16→V15 reflash retained phase-one, loader and post-boot sequence. No whole-flash readback; Linux ALSA not hardware-tested by that author. | Partial/unaligned blocks, exact lengths, sentinel addresses, post-boot identity and platform-separated claims. |
| [Felucca #32](https://github.com/hugelton/Felucca/issues/32) | Rollback “not found” resolved after host restart; names changed to Felucca and MIDI apps/tabs may keep ports open. Windows Edge rollback reported working. | Do not equate renamed/busy port with brick; avoid guessing another MIDI instrument. |
| [Felucca #13](https://github.com/hugelton/Felucca/issues/13) | User with old Mac unable to run official offline app requested browser rollback; maintainer added/identified V15 browser file selection with backup. | Offline/direct CLI route has real usability value; stock package remains user supplied. |
| [Felucca #120](https://github.com/hugelton/Felucca/issues/120) | Web installer served older firmware than the published release until deployment updated. | Exact immutable asset/hash/source receipt and post-boot verification; “latest release” UI label is insufficient. |
| [Melodee #4](https://github.com/keremimo/melodee/issues/4) | V15 rollback was blocked because existing firmware could not export complete backup; resolution added explicit skip-backup choice. | Backup capabilities must be reported separately. The strict sole-device mode should require an independently verified raw dump instead of silently skipping safety prerequisites. |
| [GHOULBOX #4](https://github.com/jasonpersinger/ghoulbox-fm1-dungeon-synth/issues/4) | Open 2026-10-10 report: firmware booted logo then crashed, rebooted and could not be found for rollback. No recovery confirmation in the thread. | Keep unresolved boot/crash reports as unsupported compatibility evidence, not a universal “all Felucca derivatives are safe” claim. |
| [Transporter #1](https://github.com/kurogedelic/FM-1-transporter/issues/1) | Build dependencies caused trouble; a later commenter offered an untested externally hosted UF2 and had initially linked the wrong variant. | Do not auto-download unverified community binaries. Pin/reproduce the correct board image and RAM loader; validate metadata/hashes. |

Selected forks such as `spinkham/Felucca` and `merthsoft/sloop-fm1` supply important divide-by-zero/Flipper work. The primary issue and extracted recovery repo were followed to the author-owned evidence above; the two connector search pages exposed many other forks, but their whole diffs were not audited. A source-only changelog or copied README does not qualify a fork's emitted image.

## Licenses and reusable inputs

This is a source-provenance inventory, not legal advice. Distribution decisions require a file/artifact-level review; a source repository license does not establish the origin/rights of every binary blob.

| Input | Verified source notice | Practical planning boundary |
| --- | --- | --- |
| AL-255/FM-1-RE original source | [root LICENSE: WTFPL v2](https://github.com/AL-255/FM-1-RE/blob/ec832f281ca165a7d5e0722985f3acb1465046d5/LICENSE); no differing notice in `fm1_ota.py` header read | Track protocol origin and any imported components separately. |
| Felucca installer, browser parser, app/update loader | [GPL-3.0-only file header](https://github.com/hugelton/Felucca/blob/ef1f561b6b105f20350f6b1eac0dc0789a357f00/tools/fm1_install.py#L1-L3), [LICENSING.md](https://github.com/hugelton/Felucca/blob/ef1f561b6b105f20350f6b1eac0dc0789a357f00/LICENSING.md) | Copied/modified implementation should retain GPL terms and complete corresponding-source obligations. Decide the new tool's license before reuse. |
| FM-1 Transporter | [MIT source notice](https://github.com/kurogedelic/FM-1-transporter/blob/a632d923203170e3a08565f4056dbb3fd3d65ca5/LICENSE) | Preserve notices and Pico-PIO-USB terms; loader blob external. |
| MvaveFM1Unbricker | [MIT](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/LICENSE) | Useful standalone reference; do not inherit unsafe overrides or overbroad family classification. |
| jl-uboot-tool source | [MIT](https://github.com/kagaimiq/jl-uboot-tool/blob/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a/LICENSE) | `wl82loader.bin` provenance/redistribution must be checked separately; not relicense merely from root MIT. |
| Flipper recovery original work | [Unlicense / third-party ledger](https://github.com/merthsoft/flipper-fm1-recovery/blob/df88b862a4794855f792fed529629a255cba0a0e/THIRD-PARTY.md) | External Flipper/Momentum SDK runtime terms remain; project expressly does not bundle/relicense RAM-loader binary. |
| Keitark planner/guard adaptations | [APP_UPDATES provenance rules](https://github.com/Keitark/fm1-nes/blob/90d41c5e48073af2d4f7ff94df7496004d4b6627/APP_UPDATES.md#L89-L112) | Source-only guards and MIT adaptations; keep unit-derived plans/stock bytes/backups private. |
| czietz patch script | [Unlicense and no redistribution of patched image](https://github.com/czietz/fm1-firmware-patcher/blob/4cfab602455168d0670fdf2ccf6979ef3b0bea3b/README.md) | User supplies official V15 locally; do not publish the modified vendor image. |
| Official firmware / updater / private dumps | Proprietary vendor/device material; no redistribution permission established by this research | User-supplied local blob, hash verification, private recovery storage. |
| aroum flasher implementation | No root license file found in inspected snapshot; writer explicitly non-functional | Use facts/source links; do not assume unrestricted code reuse. |

Felucca's licensing ledger describes three Apache-2.0 SDK files in its `.fwsc` and deliberately includes no M-VAVE material in its sources. This differs from an extracted vendor updater image or ROM helper; keep these provenance classes distinct. [Felucca SDK/package provenance](https://github.com/hugelton/Felucca/blob/ef1f561b6b105f20350f6b1eac0dc0789a357f00/LICENSING.md#L59-L85).

## Specific gaps to close before device writes

- Actual user's chip/board revision, current firmware identity/version/capabilities, flash map, current semantic data and boot-head baseline remain unknown.
- No raw backup or independently validated dongle is assumed available; the project must remain offline until preparation evidence exists.
- No host can promise that arbitrary same-identity stock/custom OTA is accepted; same-version gate work must be based on exact profiles, not identity spoofing.
- The generic Unbricker classifier labels many three-digit custom identities Felucca-family and its shared `_check_mutable` upper bound is `FLASH_SIZE`, while the restore command itself uses `[0x4000,0x93000)`. Reusing it in a universal tool needs explicit immutable allowed-region profiles in every layer. [Classifier](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/fm1_unbrick.py#L272-L352), [range gate](https://github.com/Quixotic7/MvaveFM1Unbricker/blob/b8c8e8afbcba3e45b262b3b624ab9d0244b92628/fm1_unbrick.py#L523-L576).
- Application-only restore does not establish boot-head recovery, universally compatible data migration, complete hardware-failure recovery or guaranteed lossless stock rollback.
- The changed readable `fm1-bootloader` is untested. Its optional generic double-bank support is not proof that the FM-1 currently runs A/B firmware or has space for two full images. [Explicit warning and retained dual-bank behavior](https://github.com/jvitkauskas/fm1-bootloader/blob/12b570b167b6695e60be18800c609730ab75bc09/README.md#L15-L20).
- A browser wrapper can reuse the MIDI protocol for phase two of the project, but ROM SCSI/USB_KEY require a separate transport/hardware path. A web installer does not itself solve hard recovery. The architecture/backlog documents must carry these gates forward.

## Appendix: complete bounded repository screening inventory

The table below records the 82 directory/search candidate URLs actually screened. Exact full SHAs are retained in links. **Deep** means code/docs were read beyond the root README in this research; **screened** means root README/metadata only and does not imply firmware installation qualification. Companion tools/emulators are included to prevent a “loader” or “simulator” search hit from being mistaken for a firmware writer. The old ip2k URL is an alias, not an independent implementation.

| Repository snapshot | Depth / result |
| --- | --- |
| [AL-255/FM-1-RE](https://github.com/AL-255/FM-1-RE/tree/ec832f281ca165a7d5e0722985f3acb1465046d5) | Deep code/docs |
| [ChanceTheMaker/Felucca](https://github.com/ChanceTheMaker/Felucca/tree/032758ad8e85eda061923bffa8031d1964538e2b) | README screened |
| [DeadActive/hortator](https://github.com/DeadActive/hortator/tree/2c2847e182f6daf1bda7cf89321c36da67af9959) | README screened |
| [ELI7VH/Felucca](https://github.com/ELI7VH/Felucca/tree/055c647af719c46776fcd76aeb7b5ab39e57b4b8) | README screened |
| [Erbubar23/bubba-machine](https://github.com/Erbubar23/bubba-machine/tree/be5f512614cb16f8a980209634e74a4955dbc3e8) | README screened |
| [Grandsummoner/fm-1-helper](https://github.com/Grandsummoner/fm-1-helper/tree/7a5932662a92ef8fe0e8f6938325e7dabc6dfa99) | README screened |
| [JalopyMusic/fm1-milkboyg-ambii](https://github.com/JalopyMusic/fm1-milkboyg-ambii/tree/0afdeee1413b956e9630f92f7dfc7d9ba6f2da6e) | README screened |
| [Keitark/fm1-doom](https://github.com/Keitark/fm1-doom/tree/3d8a093ddadd7e93a210c3847db6d57ce99202c0) | README screened |
| [Keitark/fm1-mdx](https://github.com/Keitark/fm1-mdx/tree/9762d417d6f3464647e8cdca4e3d9d3a4df38538) | README screened |
| [Keitark/fm1-nes](https://github.com/Keitark/fm1-nes/tree/90d41c5e48073af2d4f7ff94df7496004d4b6627) | Deep code/docs |
| [KingParamount/fm1-factory-presets](https://github.com/KingParamount/fm1-factory-presets/tree/04b34cabd69745cf3a549af588619108e2866c0d) | README screened |
| [NOVALENTI/fm1-polyseq](https://github.com/NOVALENTI/fm1-polyseq/tree/b028d3f8ebb76df2505c258ce7bb2e0f63901056) | Resolved HEAD; no root README found |
| [Nicodesy06/FM1-MOVE](https://github.com/Nicodesy06/FM1-MOVE/tree/0f213e818545aa5114aef8290f64a101b8a2efb0) | README screened |
| [Quixotic7/ChoralRootFM1](https://github.com/Quixotic7/ChoralRootFM1/tree/e8efc993c986f8d7661c515ebfc8c46c6e77057c) | Deep code/docs |
| [Quixotic7/FM1VST](https://github.com/Quixotic7/FM1VST/tree/ebecb9836173978041f90b0054827aec396411f6) | README screened |
| [Quixotic7/MvaveFM1-Designer](https://github.com/Quixotic7/MvaveFM1-Designer/tree/0ecd189b7b3a2ef8d147cda6cbf13f2ac73f3fa6) | README screened |
| [Quixotic7/MvaveFM1Unbricker](https://github.com/Quixotic7/MvaveFM1Unbricker/tree/b8c8e8afbcba3e45b262b3b624ab9d0244b92628) | Deep code/docs |
| [Quixotic7/PurpleMonkeyFM1](https://github.com/Quixotic7/PurpleMonkeyFM1/tree/b574e75dd1dcf3283c3eb5d286fd7c6d21d95517) | README screened |
| [aroum/fm1-custom-fw](https://github.com/aroum/fm1-custom-fw/tree/6536e06c72ef812ce84ab715208d62311f660425) | Deep code/docs |
| [beashoko/april-os](https://github.com/beashoko/april-os/tree/f2356c2f9030d69c7ae1261b86639b1fa6bf79f4) | README screened |
| [benny-sparra/fm1-dx7-patch-importer](https://github.com/benny-sparra/fm1-dx7-patch-importer/tree/80ca679840aca2b3aa689e7705af8b668759ac53) | README screened |
| [cartesive/fumi-1](https://github.com/cartesive/fumi-1/tree/6f763f1574d3ccac6e41989f0538fb39b481097a) | README screened |
| [catacombius/NoteSorcery](https://github.com/catacombius/NoteSorcery/tree/c13ac1b63f0d87cbbeb74dda91510946724c47eb) | README screened |
| [chancethemaker/sloop-fm1-sim](https://github.com/chancethemaker/sloop-fm1-sim/tree/3dbf5707741048f099120794660aa41b1b65e810) | README screened |
| [charlesvestal/fm1-amb](https://github.com/charlesvestal/fm1-amb/tree/60dfbec4602497616d6a196be78e9288dc7c615a) | README screened |
| [charlesvestal/fm1-ctl](https://github.com/charlesvestal/fm1-ctl/tree/1deb0f463339a274899a934a79fcf2fb25dd9dc0) | README screened |
| [charlesvestal/fm1-fomni](https://github.com/charlesvestal/fm1-fomni/tree/405ef41aef75f7e70afd6decc01505df07c708e9) | README screened |
| [charlesvestal/fm1-mocky](https://github.com/charlesvestal/fm1-mocky/tree/13121b4174596e545bcd0a32acdcfec1b4daf050) | README screened |
| [charlesvestal/fm1-x0x](https://github.com/charlesvestal/fm1-x0x/tree/47f8e4c4ad602591b81c21bc5480db7a94f6d6af) | README screened |
| [cicloid/awesome-fm-1](https://github.com/cicloid/awesome-fm-1/tree/b31003409227464dd781f9a2f03d9d9b62b40d67) | Deep code/docs |
| [cmlarsen/fm1-rocket-os](https://github.com/cmlarsen/fm1-rocket-os/tree/23dad609bacf9876fc5dfe2a4b0667e719bee399) | README screened |
| [czietz/fm1-firmware-patcher](https://github.com/czietz/fm1-firmware-patcher/tree/4cfab602455168d0670fdf2ccf6979ef3b0bea3b) | Deep code/docs |
| [czietz/fm1-read-voice](https://github.com/czietz/fm1-read-voice/tree/b7c539f2ca4a492d5ee8afdfb39ad9b3ca89a30e) | Resolved HEAD; no root README found |
| [dspaudio/orbit](https://github.com/dspaudio/orbit/tree/4c119edd6081e1abe9d20a59d731455f8092c67e) | README screened |
| [ehn96/Falua](https://github.com/ehn96/Falua/tree/73fc26cf0a6c3116decd5c337d83650782db3d7a) | README screened |
| [entitymar/MVAVE-M-UPGRADE-decompiled](https://github.com/entitymar/MVAVE-M-UPGRADE-decompiled/tree/24785855dea64276e9b0b8256356d63ebf37e281) | README screened |
| [friendsmakenoise-prog/fm1-pocket-sampler](https://github.com/friendsmakenoise-prog/fm1-pocket-sampler) | Unreachable on research date; no source claim |
| [fuleo/fm1-linux-update](https://github.com/fuleo/fm1-linux-update/tree/fa850941dbd69f9172ed78dea5a1d01178a5fa2d) | Deep code/docs |
| [hb3p8/fm1-mobile-emu](https://github.com/hb3p8/fm1-mobile-emu/tree/4959eda7129a8cd3f5d1c4af922a2c8489103ba5) | README screened |
| [hericdk/fm1-quest](https://github.com/hericdk/fm1-quest/tree/477069010c3acc7f6fde31d119920da2c9dcdb69) | README screened |
| [hugelton/Felucca](https://github.com/hugelton/Felucca/tree/ef1f561b6b105f20350f6b1eac0dc0789a357f00) | Deep code/docs |
| [ip2k/lunar-modulator](https://github.com/ip2k/lunar-modulator/tree/b335dc2c450473dc375163aec7fa679e1b099cc3) | Deep code/docs |
| [ip2k/mvave-fm1-open-firmware](https://github.com/ip2k/mvave-fm1-open-firmware/tree/b335dc2c450473dc375163aec7fa679e1b099cc3) | Old URL; same HEAD as lunar-modulator |
| [isod89/sloop-fm1](https://github.com/isod89/sloop-fm1/tree/4365a1d0075b9194de1c44826325abbd39751bc1) | Deep code/docs |
| [jadamsowers/fm1-fimba](https://github.com/jadamsowers/fm1-fimba/tree/b2b647b91bb3e9f0014529a468eeb6ac65f90ad9) | README screened |
| [jahlib/sloop-fm1-go](https://github.com/jahlib/sloop-fm1-go/tree/1f0504a9d262ddf9ff866ef79e4cc04ccf928951) | README screened |
| [jasonpersinger/ghoulbox-fm1-dungeon-synth](https://github.com/jasonpersinger/ghoulbox-fm1-dungeon-synth/tree/b61a931cd67b812a9c4f650eefc72215828dc5b0) | README screened |
| [jbschooley/Virtual-FM-1](https://github.com/jbschooley/Virtual-FM-1/tree/15e81252d2f6eb89c8a1ca14aeed5dc8a362c8be) | README screened |
| [jeymadcat-ops/SLICE64-FM-web](https://github.com/jeymadcat-ops/SLICE64-FM-web/tree/f2705083f9573f65b45a4b58cec8f09f761e9252) | README screened |
| [joshbartlettband-web/felucca-rainbow](https://github.com/joshbartlettband-web/felucca-rainbow/tree/8d47df6920204961af35a6c27f9ac58edbe44797) | README screened |
| [juanjiant-bit/JIANT](https://github.com/juanjiant-bit/JIANT/tree/0af21ffe39eb2d8a3939f777ac6e65b9f6dd9aba) | README screened |
| [jvitkauskas/fm1-bootloader](https://github.com/jvitkauskas/fm1-bootloader/tree/12b570b167b6695e60be18800c609730ab75bc09) | Deep code/docs |
| [kagaimiq/ghidra-jieli](https://github.com/kagaimiq/ghidra-jieli/tree/b5e60122b6cd3e6b615387035994b8bed0ea1a26) | README screened |
| [kagaimiq/jielie](https://github.com/kagaimiq/jielie/tree/1657d25e6e51df6b2c18cd55cfc576c4a6370c63) | README screened |
| [kagaimiq/jl-misctools](https://github.com/kagaimiq/jl-misctools/tree/0a5b12db0ef38f3042acffbe2452730a37fd2405) | README screened |
| [kagaimiq/jl-uboot-tool](https://github.com/kagaimiq/jl-uboot-tool/tree/adb3f18889e88ac512ce0a3c4d8cc3d3cb30696a) | Deep code/docs |
| [keremimo/melodee](https://github.com/keremimo/melodee/tree/01a78bdf0fb0e1a9d953567e36a59d80dea54a8a) | Deep code/docs |
| [klonecom/M-VAVE-FM1-LOADER---](https://github.com/klonecom/M-VAVE-FM1-LOADER---/tree/7e4609784952629d3e77f6679f3c37c1d28e400c) | README screened |
| [kurogedelic/FM-1-transporter](https://github.com/kurogedelic/FM-1-transporter/tree/a632d923203170e3a08565f4056dbb3fd3d65ca5) | Deep code/docs |
| [leomaimoni/fm1-bank-sender](https://github.com/leomaimoni/fm1-bank-sender/tree/16b1b75fa2516e4ae952607e0615f3ae5154af3b) | README screened |
| [math0ne/fm1-chord](https://github.com/math0ne/fm1-chord/tree/377235d9f39a362e673f8fd60a26f0b1748218e7) | README screened |
| [mene311/fm1-banks](https://github.com/mene311/fm1-banks/tree/f47b11b9035be0dfe7f899aec90798dccbcde9ee) | README screened |
| [mene311/fm1-pulses](https://github.com/mene311/fm1-pulses/tree/e219dcc28f690ca2bc031536ac158fb30f4d4704) | README screened |
| [merthsoft/flipper-fm1-recovery](https://github.com/merthsoft/flipper-fm1-recovery/tree/df88b862a4794855f792fed529629a255cba0a0e) | Deep code/docs |
| [pfkellogg/fm1-bonus-box](https://github.com/pfkellogg/fm1-bonus-box/tree/069d0234d13220e862a08e84a2da6b828f9c24c7) | README screened |
| [pfkellogg/fm1-midi-voice-tuner](https://github.com/pfkellogg/fm1-midi-voice-tuner/tree/48a8fc36b82afc39b549727cfde992d2a63dd210) | README screened |
| [pfkellogg/fm1-sustain-footswitch](https://github.com/pfkellogg/fm1-sustain-footswitch/tree/9a5f89f040875bbab9bf5105ab6c588423e3ddd7) | README screened |
| [pingywon/fm1-easy-guide](https://github.com/pingywon/fm1-easy-guide/tree/9889e4863fe541ddb6f5eccefe549f4cd1e431a7) | README screened |
| [seajaysec/fm-static](https://github.com/seajaysec/fm-static/tree/0fa627434b7aa549af94dc819540be41f6c4dcbc) | Deep code/docs |
| [shaw-core/Sloop_ALG02](https://github.com/shaw-core/Sloop_ALG02/tree/cbfcee3e79f759abffc2613b931f495d29e4c1e5) | README screened |
| [shcherbakov7/m-wave-fm1-browser-emulator](https://github.com/shcherbakov7/m-wave-fm1-browser-emulator/tree/056621b0c3187e6712326ef854d0a78ed9605600) | README screened |
| [simonjohansson/fm1-emulator](https://github.com/simonjohansson/fm1-emulator/tree/41d7485ab0c86675abbafe8e779aefb57062aced) | README screened |
| [smhulme/sloop-web-8trk](https://github.com/smhulme/sloop-web-8trk/tree/9974f8f67fc645c2683565697d84a6be14c70e71) | README screened |
| [thegiantsnail/fm1-workbench-public](https://github.com/thegiantsnail/fm1-workbench-public/tree/e35315b7ea814b99387840a4fc66754f6a7f59d5) | README screened |
| [w0ts/optimist](https://github.com/w0ts/optimist/tree/ddc4f4754396b453d086dced32a3a3af1770fe63) | README screened |
| [y6zkcqnf5r-hue/fm1-fm80s](https://github.com/y6zkcqnf5r-hue/fm1-fm80s/tree/171c51fae0c433f32b1185b79d0e3d6df86ea849) | README screened |
| [y6zkcqnf5r-hue/fm1-trawler](https://github.com/y6zkcqnf5r-hue/fm1-trawler/tree/153f4f4433f15ef5bf3576035cff61f7d009ecc2) | README screened |
| [zednaked/jangada](https://github.com/zednaked/jangada/tree/dc1b7e5d1d17bd057de664016d9210fd8a92120c) | README screened |
| [zvenson/dinghy](https://github.com/zvenson/dinghy/tree/929730019a9dc362c12a710c0b139cbfefb582e9) | README screened |
| [zvenson/dxsloop](https://github.com/zvenson/dxsloop/tree/ee420bc7c2975ae0f312bbae7a81fdbdf90f3f14) | README screened |
| [zvenson/fm1doom](https://github.com/zvenson/fm1doom/tree/640bc2dbdc69bcaf045afb9ef3695274f49fed4c) | README screened |
| [zvenson/zp12](https://github.com/zvenson/zp12/tree/610f330423ae8c37e1b5137b11f04c9c185553bc) | README screened |
