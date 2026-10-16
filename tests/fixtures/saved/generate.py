#!/usr/bin/env python3
"""Independent synthetic saved-file oracles. No firmware/capture input.

Only Python's binascii CRC and a separate shift/XOR ENC reference are used.
Rerun to reproduce the checked-in minimal raw/package bytes.
"""
from pathlib import Path
from struct import pack_into
from binascii import crc_hqx
ROOT = Path(__file__).resolve().parent

def enc(data, key=0xffff):
    out = bytearray()
    for b in data:
        out.append(b ^ (key & 255))
        high = key & 0x8000
        key = (key * 2) & 65535
        if high:
            key ^= 0x1021
    return out

def entry(name, address, length, attr, crc=0):
    out = bytearray(32)
    pack_into('<HII', out, 2, crc, address, length)
    out[12] = attr
    out[16:16+len(name)] = name.encode('ascii')
    pack_into('<H', out, 0, crc_hqx(out[2:], 0))
    return out

def raw(vm=0x93000, app_payload=b'offset-oracle-240\0', size=0x5000):
    out = bytearray(b'\xff' * size)
    head = bytearray(32)
    pack_into('<H', head, 0, crc_hqx(head[2:], 0))
    out[:32] = enc(head)
    payload = b'BANK!'
    boot = bytearray(16)
    pack_into('<HH', boot, 0, 1, len(payload))
    pack_into('<I', boot, 8, 16)
    pack_into('<H', boot, 12, crc_hqx(payload, 0))
    pack_into('<H', boot, 14, crc_hqx(boot[:14], 0))
    boot.extend(payload)
    out[0x20:0x40] = enc(entry('uboot.boot', 0xa0, len(boot), 0x82, crc_hqx(boot, 0)))
    out[0xa0:0xa0+len(boot)] = enc(boot[:16]) + enc(boot[16:])
    payload = app_payload
    entries = [entry('app.bin', 0x240, len(payload), 0x82, crc_hqx(payload, 0)),
               entry('VM', vm, 0xE9000-vm, 0x12),
               entry('BTIF', 0xE9000, 0x1000, 0x12),
               entry('USR', 0xEA000, 0x12000, 0x12)]
    for i, e in enumerate(entries):
        out[0x4000+32*i:0x4020+32*i] = enc(e, 0x980f ^ (i*8))
    padded = payload + b'\0' * ((-len(payload)) % 32)
    for offset in range(0, len(padded), 32):
        out[0x4240+offset:0x4260+offset] = enc(padded[offset:offset+32], 0x980f ^ ((0x240+offset) >> 2))
    return out

def package(flash, malformed=None):
    # One flash payload and one non-flash payload test all-payload integrity.
    payloads = [('flash.bin', 0, flash), ('test.dat', 2, b'PRIVATE-PACKAGE-CANARY')]
    start = 64 + 80*len(payloads)
    image = bytearray(start)
    directory = bytearray()
    for i, (name, typ, data) in enumerate(payloads):
        e = bytearray(80)
        pack_into('<HHH', e, 0, typ, i, crc_hqx(data, 0))
        pack_into('<II', e, 8, len(image), len(data))
        e[64:64+len(name)] = name.encode('ascii')
        if malformed == 'overlap' and i:
            pack_into('<I', e, 8, start)
        if malformed == 'duplicate' and i:
            e[64:] = b'flash.bin'.ljust(16, b'\0')
        directory.extend(enc(e))
        image.extend(data)
    image[64:start] = directory
    head = bytearray(64)
    pack_into('<HIH', head, 2, crc_hqx(directory, 0), len(image), len(payloads))
    pack_into('<H', head, 0, crc_hqx(head[2:], 0))
    image[:64] = enc(head)
    if malformed == 'aux-crc':
        image[-1] ^= 1
    # Weave 20 identity characters, then ordinary bytes. The last char is not
    # 0x7d, so the parser stops after the fixed minimum identity window.
    ident = b'FM-1_015'.ljust(20, b'\0')
    out = bytearray()
    for i in range(20):
        out.extend(image[47*i:47*(i+1)])
        out.append((ident[i] + i + 1) & 255)
    out.extend(image[47*20:])
    return out

# Identity termination convention: a 0x7d marker terminates identity text,
# while block 20 must contain a non-marker to end the woven window scan.
def wire_package(flash, malformed=None):
    p = package(flash, malformed)
    for i in range(8, 20):
        p[48*i+47] = 0x7d
    return p

if __name__ == '__main__':
    base = raw()
    (ROOT/'minimal.raw').write_bytes(base)
    (ROOT/'valid.fwsc').write_bytes(wire_package(base))
    (ROOT/'incompatible.fwsc').write_bytes(wire_package(raw(0x94000)))
    for kind in ('overlap', 'duplicate', 'aux-crc'):
        (ROOT/(kind+'.fwsc')).write_bytes(wire_package(base, kind))

    for kind, values in {
        'overflow': ('app.bin', 0xfffffff0, 0xfffffff0, 0x82),
        'zero-length': ('app.bin', 0x240, 0, 0x82),
        'directory-overlap': ('app.bin', 0, 32, 0x82),
        'overlap': ('VM', 0x4240, 32, 0x12),
    }.items():
        table = bytearray(base[0x4000:0x4080])
        index = 1 if kind == 'overlap' else 0
        table[32*index:32*(index+1)] = enc(entry(*values), 0x980f ^ (index*8))
        (ROOT/(kind+'.table')).write_bytes(table)

    missing = bytearray(base)
    # Its CRC deliberately matches what erased logical padding decrypts to.
    absent_plain = enc(b'\xff'*32, 0x980f ^ (0x8000 >> 2))[:17]
    missing[0x4000:0x4020] = enc(entry('app.bin', 0x8000, 17, 0x82, crc_hqx(absent_plain, 0)), 0x980f)
    (ROOT/'missing-app.fwsc').write_bytes(wire_package(missing))

    (ROOT/'incident.raw').write_bytes(raw(app_payload=b'OLD-APP-SECTOR!'.ljust(0x3000, b'\xa1'), size=0x8000))
