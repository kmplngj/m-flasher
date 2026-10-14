#!/bin/sh
# Builds loader.bin (plain) with JieLi's pi32v2 toolchain.
set -e
T=${JIELI_TOOLCHAIN:-/root/fm1/toolchains/jieli/jieli-linux-toolchains-20250324.1}
cd "$(dirname "$0")"
for f in loader p33 display; do $T/common/bin/clang -target pi32v2 -mcpu=r3 -Os -fno-common -ffreestanding -Wall -I. -c $f.c -o $f.o; done
ADDR=${1:-0x01C20000}
sed "s/LOAD_ADDR/$ADDR/" loader.ld > build.ld
$T/pi32v2/bin/lto-wrapper -o loader.elf loader.o p33.o display.o -T build.ld --gc-sections -Map=loader.map
$T/common/bin/objcopy -O binary -j .text loader.elf loader.bin
$T/common/bin/objdump -d loader.elf > loader.dis
ls -l loader.bin
# the ROM jumps to the first byte: that must be loader_entry
[ "$(grep -m1 -E '^[A-Za-z_][A-Za-z0-9_.]*:$' loader.dis)" = "loader_entry:" ] || { echo "loader_entry is not first" >&2; exit 1; }
