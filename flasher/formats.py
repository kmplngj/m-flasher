"""File and flash formats of the JieLi AC79 (WL82) devices: CRCs, the ENC
cipher, the boot area and its file table, the app area, BANKCB bootloader
containers and M-VAVE's .fwsc firmware packages.

Flash layout (FM-1 and SMK-37, 1 MiB):

    0x00000  boot area, 16 KiB: flash header (32 bytes) and file table
             (32-byte entries from 0x20), the bootloader (uboot.boot) at
             0xA0, isd_config.ini at 0x38D0; all ENC with key 0xFFFF
    0x04000  app area: its own file table (app.bin, cfg_tool.bin and the
             region descriptors VM, BTIF, USR, ...), ENC per 32-byte block
             with the chip key XOR (offset in the area >> 2)
    ...      the regions the app's table declares (settings, pairings,
             sounds, the SMK-37's calibration and presets)
    0xFF000  key_mac: Bluetooth MAC and the flash efuse page

All CRCs are CRC16/XMODEM.
"""
import binascii
import struct

SECTOR = 0x1000
BOOT_AREA = 0x4000          # boot area size, and the start of the app area
BOOT_AT, ISD_AT = 0xA0, 0x38D0


class FormatError(Exception):
    pass


def crc16(data, crc=0):
    return binascii.crc_hqx(bytes(data), crc)


def enc(data, key=0xFFFF):
    """JieLi's ENC cipher (its own inverse)."""
    out = bytearray(data)
    for i in range(len(out)):
        out[i] ^= key & 0xFF
        key = ((key << 1) ^ (0x1021 if key & 0x8000 else 0)) & 0xFFFF
    return bytes(out)


# ---- file tables ----

class Entry:
    """A 32-byte file table entry: head CRC, data CRC, address, length,
    attribute, index, name."""

    def __init__(self, raw):
        self.raw = bytes(raw)
        (self.head_crc, self.data_crc, self.addr, self.len, self.attr, _,
         self.index) = struct.unpack_from('<HHIIBBH', raw)
        self.name = raw[16:32].split(b'\0')[0].decode('latin1')
        self.valid = crc16(raw[2:32]) == self.head_crc

    def __repr__(self):
        return f'<{self.name} 0x{self.addr:x}+0x{self.len:x} attr 0x{self.attr:02x}>'


def flash_header_valid(flash):
    h = enc(flash[:32])
    return crc16(h[2:]) == struct.unpack_from('<H', h)[0]


def boot_table(flash):
    """The boot area's file table (uboot.boot, isd_config.ini, app_dir_head,
    key_mac), or [] if the flash header is damaged."""
    if not flash_header_valid(flash):
        return []
    out = []
    for a in range(0x20, BOOT_AT, 0x20):
        e = Entry(enc(flash[a:a + 32]))
        if not e.valid:
            break
        out.append(e)
    return out


def app_read(flash, base, off, n, key):
    """n plain bytes at offset off of the app area at base."""
    out = bytearray()
    a = off - off % 32
    while a < off + n:
        out += enc(flash[base + a:base + a + 32], key ^ ((a >> 2) & 0xFFFF))
        a += 32
    return bytes(out[off % 32:off % 32 + n])


def app_table(flash, key, base=BOOT_AREA):
    """The app area's file table: files (app.bin, cfg_tool.bin; addresses in
    the area) and region descriptors (VM, BTIF, ...; absolute addresses)."""
    out = []
    for i in range(16):
        e = Entry(app_read(flash, base, 32 * i, 32, key))
        if not e.valid:
            break
        out.append(e)
    return out


def app_files_status(flash, key, base=BOOT_AREA):
    """'ok', 'bad crc', 'incomplete' (a damaged file with erased sectors in
    it) or 'not found', as the bootloader's menu reports it."""
    table = app_table(flash, key, base)
    files = [e for e in table if e.attr == 0x82]
    if not files:
        return 'not found'
    for e in files:
        if base + e.addr + e.len > len(flash):
            return 'bad crc'
        if crc16(app_read(flash, base, e.addr, e.len, key)) != e.data_crc:
            start, end = base + e.addr, base + e.addr + e.len
            for s in range(start - start % SECTOR, end, SECTOR):
                if flash[s:s + SECTOR] == b'\xff' * SECTOR:
                    return 'incomplete'
            return 'bad crc'
    return 'ok'


# ---- bootloader (BANKCB container) ----

def bankcb_check(boot):
    """Payload of a valid BANKCB container, else FormatError."""
    if len(boot) < 16:
        raise FormatError('bootloader file too short')
    bank, size, _, offset, pcrc, hcrc = struct.unpack_from('<HHIIHH', boot)
    payload = boot[offset:offset + size]
    if bank != 1 or crc16(boot[:14]) != hcrc or len(payload) != size or crc16(payload) != pcrc:
        raise FormatError('not a valid bootloader container (BANKCB)')
    return payload


def bootloader_identity(boot):
    """(board, version) of one of our bootloaders, or None (stock or other)."""
    try:
        payload = bankcb_check(boot)
    except FormatError:
        return None
    if payload[0x40:0x48] != b'AC79BLID':
        return None
    board = payload[0x48:0x50].split(b'\0')[0].decode('latin1')
    version = payload[0x50:0x70].split(b'\0')[0].decode('latin1')
    return board, version


def stored_bootloader(flash):
    """The uboot.boot container stored in a boot area, or None."""
    table = boot_table(flash)
    if not table or table[0].name != 'uboot.boot' or table[0].addr != BOOT_AT:
        return None
    s = flash[BOOT_AT:BOOT_AT + table[0].len]
    boot = enc(s[:16]) + enc(s[16:])
    return boot if crc16(boot) == table[0].data_crc else None


def build_boot_area(flash, boot):
    """flash's boot area with its bootloader replaced by boot. Keeps the
    flash header, the file table and isd_config (the unit's own); the
    bootloader's entry gets the new length and CRC, the space up to
    isd_config is filled with 0xFF. Stored as JieLi's packager writes it:
    the 16-byte header and the payload each ENC 0xFFFF on their own."""
    bankcb_check(boot)
    area = bytearray(flash[:BOOT_AREA])
    table = boot_table(area)
    if len(table) < 3 or table[0].name != 'uboot.boot' or table[0].addr != BOOT_AT:
        raise FormatError('the boot area has no valid file table')
    if table[1].name != 'isd_config.ini' or table[1].addr != ISD_AT:
        raise FormatError('isd_config is not at 0x38D0')
    if BOOT_AT + len(boot) > ISD_AT:
        raise FormatError(f'bootloader too large: {len(boot)} bytes, room for {ISD_AT - BOOT_AT}')
    e = bytearray(table[0].raw)
    struct.pack_into('<HII', e, 2, crc16(boot), BOOT_AT, len(boot))
    struct.pack_into('<H', e, 0, crc16(e[2:]))
    area[0x20:0x40] = enc(e)
    area[BOOT_AT:ISD_AT] = enc(boot[:16]) + enc(boot[16:]) + b'\xff' * (ISD_AT - BOOT_AT - len(boot))
    return bytes(area)


# ---- .fwsc packages ----

FWSC_BLOCK, FWSC_KEEP = 0x30, 0x2F


def _fwsc_blocks(raw):
    n = 0
    while n < 20 or raw[n * FWSC_BLOCK + FWSC_KEEP] == 0x7D:
        n += 1
        if (n + 1) * FWSC_BLOCK > len(raw):
            raise FormatError('not a firmware package (.fwsc)')
    return n


class Package:
    """An M-VAVE firmware package (.fwsc): a JieLi UFW image with the
    product identity woven in. The file is cut into 48-byte blocks; byte 47
    of block i holds identity character i as (char + i + 1) & 0xFF, 0x7D
    once the identity has ended. Without those bytes: a 64-byte header
    (CRC16 of bytes 2..63, CRC16 of the file list, total size, file count)
    and the file list (80 bytes per file: type, index, data CRC16, offset,
    size, name at 0x40), both ENC 0xFFFF. flash.bin (type 0) is the flash
    image from address 0: the boot area as stored in flash, then the app
    area, already encrypted with the chip key the package was made for."""

    def __init__(self, raw):
        raw = bytes(raw)
        if len(raw) < 0x1000:
            raise FormatError('not a firmware package (.fwsc)')
        n = _fwsc_blocks(raw)
        ident = ''
        for i in range(n):
            m = raw[i * FWSC_BLOCK + FWSC_KEEP]
            if m == 0x7D:
                break
            ident += chr((m - i - 1) & 0xFF)
        self.identity = ident
        image = b''.join(raw[i * FWSC_BLOCK:i * FWSC_BLOCK + FWSC_KEEP] for i in range(n)) + raw[n * FWSC_BLOCK:]
        head = enc(image[:0x40])
        if crc16(head[2:]) != struct.unpack_from('<H', head)[0]:
            raise FormatError('not a firmware package, or a damaged one (header CRC)')
        count = struct.unpack_from('<H', head, 8)[0]
        lst = image[0x40:0x40 + 0x50 * count]
        if crc16(lst) != struct.unpack_from('<H', head, 2)[0]:
            raise FormatError('damaged firmware package (file list CRC)')
        self.files = {}
        for i in range(count):
            e = enc(lst[i * 0x50:(i + 1) * 0x50])
            typ, _, dcrc, _, off, size = struct.unpack_from('<HHHHII', e)
            name = e[0x40:].split(b'\0')[0].decode('latin1')
            self.files[name] = (typ, dcrc, image[off:off + size])
        if 'flash.bin' not in self.files or self.files['flash.bin'][0] != 0:
            raise FormatError('the package has no flash image (flash.bin)')
        _, dcrc, flash = self.files['flash.bin']
        if crc16(flash) != dcrc:
            raise FormatError('damaged firmware package (flash.bin CRC)')
        if len(flash) <= BOOT_AREA or len(flash) % SECTOR:
            raise FormatError(f'unexpected flash image size {len(flash)}')
        self.flash = flash

    @property
    def model(self):
        """'FM-1', 'SMK-37 Pro', ... (the identity without its version)."""
        return self.identity.rpartition('_')[0] or self.identity

    @property
    def version(self):
        return self.identity.rpartition('_')[2]
