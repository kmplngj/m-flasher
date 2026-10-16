# Bounded FMR1 CLI qualification

This experimental command connects the reproduced minimal reader to a separate
host path. It performs exactly two fresh 256-byte reads at physical address zero.
It does not capture a full backup or qualify firmware installation/recovery.
The existing `info`, `backup` and GUI still use their original combined loader;
they do not select this reader.

## Build and file-only preparation

```sh
rtk proxy python3 tools/run_offline_tests.py --cargo
rtk proxy cargo build --offline --locked --release --features reader-qualification
rtk proxy target/release/m-flasher reader-check --help
```

The qualification feature is opt-in; normal application builds have no active
native FMR1 adapter. Native access and the privileged helper are compiled out
under `cfg(test)` or `offline-tests`. The optional dependency `nusb = 0.2.8`
provides selected-device access. Existing package/dependency versions, original
libusb/writer paths, bundled loaders and target reader source remain unchanged.

An existing complete Cargo source cache is needed for offline builds. This
task populated the original locked source cache explicitly and added four
locked packages for the optional adapter; no system driver was installed.

`--prepare` requires a regular image file, exact image digest, explicit device
path, physical location, stock serial label and a new output directory name.
It does not enumerate/open/query devices or create the output directory:

```sh
rtk proxy target/release/m-flasher reader-check --prepare \
  --reader target/fmr-reader/build-7/fmr-reader.bin \
  --device usb:1-1 --location 01010000 --serial 0123456789ABCDEF \
  --output target/private-read-result
```

The example path is illustrative, not a saved live selector. At execution the
ROM bus/address must be freshly selected at the observed physical port. The
stock serial argument is only a label: ROM serial equality is not assumed.
This task qualifies macOS first; Linux activation/selection and Windows access
remain separate work. Do not infer platform qualification from compilation.

## Fixed transaction sequence

Only this exact image is accepted: 1,456 bytes, SHA-256
`63c12a9ded28b53fa991c8261cbc8065ef93f8e71a805f53553999175422618a`,
build ID `3830fc94`. Independent reproduced outputs and their source/build
bindings are recorded in [the M1 build report](fm1-m1-offline-build.md).

1. Open the explicitly selected new ROM object at `4c4a:8057`, configuration 1,
   interface 0/alt0, class `08/06/50`, two bulk endpoints `01/81` of 64 bytes.
2. Upload three independently scrambled RAM blocks (512, 512, 432 bytes),
   with ciphertext CRC16, using `FB06` at `01C20000`, `01C20200`, `01C20400`.
3. Invoke `FB08` once at `01C20000`, argument 1; verify its 16-byte echo.
   The ROM supplies the callback interface; no host-picked callback address
   or ROM memory read is used.
4. Require `FC40`: `FMR1`, ABI 1, status zero, exact build ID, DMA count zero.
5. Require `FC0B`: JEDEC `85 60 14`, status zero, 1 MiB, read size 256 and
   zero reserved bytes. A different chip refuses before either flash read.
6. Read `FD05(0,256)`, check identity/count 1, read `FD05(0,256)` again,
   compare every byte, then require identity/count 2.

Each transaction sends one exact 31-byte CBW, checks actual data counts and the
13-byte CSW signature, fresh tag, zero residue and successful status. One shared
three-second deadline covers every phase of that transaction. Native status is
checked separately from counts. IN buffers use packet-aligned native requests;
extra actual bytes are refused rather than clipped into apparent success.

Every error permanently ends the attempt. There is no retry, fallback loader,
BOT reset, device halt clearing, reconnect or arbitrary CDB/range/length input.
The reader itself contains only NOR `9F` and `03`; it has no write-enable,
status-register programming, erase/program, unprotect, FC0A or vendor initializer.

Initial device open/claim and OS permissions are distinct from a USB transaction.
For this macOS qualification, one explicit root-only capture of the exact
registry object precedes the ordinary selected open/claim. It uses only the
SDK capture bit, changes driver ownership and is not a USB reset/re-enumeration
or transport recovery. No release-mask, device reset or automatic claim retry
is issued. The capture lease stays alive through the retained session. The SDK
notes that a mass-storage interface driver may remain attached; a failed single
claim still refuses before RAM upload. macOS can require administrator
authorization; its dedicated one-shot helper accepts only the
fixed image/selection frame, re-verifies the digest and returns samples over IPC.
It accepts no filesystem paths, generic CDBs or legacy writer operations.

An expired pending native transfer is cancelled as terminal cleanup, with a
finite callback-drain wait. macOS cancellation uses AbortPipe, which clears a
controller halted bit without a device ClearFeature/data-toggle reset. The
synchronous OS operation and final callback lifetime are not hard wall-clock
bounded. If callbacks remain pending at native drop, the capture reference is
kept until the one-shot process exits, rather than released before callback-owned
handles. Normal fully completed cleanup releases retained handles and the COM
reference; it never requests release-mask re-enumeration.

## Selected Stock entry and actual return

The owner explicitly authorized this bounded physical test with no flash or
status-register writes. This authorization does not enable future autonomous
writer or fault-injection cycles on the sole FM-1.

Before any entry, retain a fresh `FM-1_015` identity and the selected stock USB
registry object/serial/location. Close other MIDI clients. The source-reviewed
one-shot host helper `tools/fmr_stock_entry.c` sends only
`F0 22 24 35 7D F7` to one exact physical CoreMIDI destination ID. Its inert
`--describe` returns before any MIDI API call. It is never run by tests or the
application. It can be compiled and inspected without sending anything:

```sh
rtk proxy clang -std=c11 -Wall -Wextra -Werror -Wno-deprecated-declarations \
  tools/fmr_stock_entry.c -framework CoreMIDI -framework CoreFoundation \
  -o target/fmr-stock-entry
rtk proxy target/fmr-stock-entry --describe
```

Its `--send-once` operation requires the separately selected current destination
ID and physical session; it is not part of file-only preparation.
The V15 branch requests RAM-mailbox/P33 reset; its known local
branch contains no NOR programming. The exact later immutable-ROM consumer has
not been recovered, so this is source-supported selection for a practical test,
not a proof of every autonomous ROM effect.

Bind removal of the old stock object and appearance of a new ROM object at the
same port before invoking `reader-check --run` with that new bus/address. Never
select the first matching VID/PID or reuse an old MIDI/ROM identifier.

After all responses are drained, the reader quiesces SPI and the host releases
the retained session. There is no software exit command: deliberately switch
the FM-1 fully off and on. USB cable removal alone may leave battery power on.
Observe normal display/USB and a fresh selected `FM-1_015` reply. Until these
observations exist, `stock_return` stays pending even if both samples match.

## Private results and tests

The command uses the shared protected storage primitives on macOS/Linux for a
new mode-0700 directory and mode-0600 files: `started.json`, `first-256.bin`,
`second-256.bin`, `result.json`. Descriptor-relative exclusive creation refuses
symlinks; ownership, modes, link counts and identities are checked. Inherited
macOS ACLs are cleared only on newly created, empty objects, then verified empty
before private bytes. Later ACL grants or pathname substitutions stop the check.
The protected starting receipt is persisted and revalidated before native open.
Both modes resolve relative output paths against the current working directory
without following symlinks, reject `..` components, and validate the existing
parent and absent leaf before reporting preparation success.
All files are synced, reopened and compared; unsupported platforms refuse this
persistence. These are private samples of one region, not full independent
recovery copies. Do not publish them or device identifiers.

Independent fixtures cover exact bootstrap/cipher/CRC framing, every short and
error boundary, CSW corruption, deadline expiry, metadata/status/counter changes,
sample mismatches, terminal reuse, bounded helper IPC and regular-file preflight.
The frozen artifact mutation suite is separate from host tests. Offline checks
do not establish physical startup, real chip identity, sample freshness or stock
return; retain the actual session outcome separately.

The shared private-output hardening is a later offline host change. The frozen
reader image is unchanged; the physical observation below identifies its tested
host build and does not qualify this newer host source or its filesystem handling.

## First selected physical result

On 2026-10-16, this exact image and the recorded CLI build completed the requested
two reads on one Stock V15 FM-1 under macOS 27.0.1 / Apple Silicon. The samples
were equal, nonconstant 256-byte buffers, persisted privately and verified after
reopening; reader identity, JEDEC/capacity, DMA counters and strict BOT checks
passed. The owner performed actual cold power-off/on and reported normal Stock
operation; a new normal USB object and a fresh `FM-1_015` reply matching the
pre-test identity were observed. No flash/status-programming command, tail read
or full capture was requested. The [sanitized observation](evidence/fm1-reader-check-2026-10-16.json)
binds the tested artifact; private raw receipts and samples are not published.

This is the first bounded head-read/return milestone, not completion of every
acceptance criterion in issue #8. Full two-pass raw capture and protected
independent recovery copies remain #9 and #10.
