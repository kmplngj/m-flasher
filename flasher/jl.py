"""JieLi UBOOT1.00 (mask ROM USB download mode) and its flash loader.

The ROM accepts memory writes and a jump. The tool writes JieLi's WL82 USB
loader (data/wl82loader.bin, taken from jl-uboot-tool) to 0x01C02000 and
starts it; the loader then answers the flash commands (protocol "V2").
Commands are 16-byte CDBs: a 16-bit opcode, big-endian arguments, padded
with 0xFF. Protocol and loader handling follow jl-uboot-tool (MIT, (c) 2023
Andrey Grigoryev).
"""
import hashlib
import os
import time

from .formats import crc16
from .scsi import SCSIDevice, SCSIError, PRODUCT, VENDOR

LOADER_ADDR = 0x01C02000
LOADER_ARG = 1                      # target: SPI NOR flash
LOADER_SHA256 = 'd41da6126760c9d66660bcc0cac8d27d221806c5e369a8036921efe68dca5376'
_DATA = os.path.join(os.path.dirname(__file__), 'data')

_CRC_MAGIC = '孟黎我爱你，玉林'.encode('gb2312')


def crc_cipher(data, key=0xFFFFFFFF):
    """JieLi's "CrcDecode" cipher (the chip key comes back in it)."""
    out = bytearray(data)
    c = crc16((key >> 16).to_bytes(2, 'little'), key & 0xFFFF)
    for i in range(len(out)):
        m = i % len(_CRC_MAGIC)
        c = crc16(_CRC_MAGIC[m:m + 1], c)
        out[i] ^= c & 0xFF
    return bytes(out)


def loader_blob():
    with open(os.path.join(_DATA, 'wl82loader.bin'), 'rb') as f:
        blob = f.read()
    if hashlib.sha256(blob).hexdigest() != LOADER_SHA256:
        raise RuntimeError('the bundled flash loader is damaged (SHA-256 mismatch)')
    return blob


class Protocol:
    WRITE_MEMORY, JUMP = 0xFB06, 0xFB08
    ERASE_BLOCK, ERASE_SECTOR, WRITE_FLASH, READ_FLASH = 0xFB00, 0xFB01, 0xFB04, 0xFD05
    READ_KEY, ONLINE_DEVICE, READ_ID, RUN_APP = 0xFC09, 0xFC0A, 0xFC0B, 0xFC0C
    USB_BUFFER_SIZE = 0xFC14

    def __init__(self, dev):
        self.dev = dev

    @staticmethod
    def cdb(cmd, args=b''):
        c = cmd.to_bytes(2, 'big') + args
        return c + b'\xff' * (16 - len(c))

    def call(self, cmd, args=b''):
        resp = bytearray(16)
        self.dev.execute(self.cdb(cmd, args), None, resp)
        if int.from_bytes(resp[:2], 'big') != cmd:
            raise SCSIError(f'unexpected answer to command 0x{cmd:04x}')
        return bytes(resp[2:])

    def data_in(self, cmd, args, n):
        buf = bytearray(n)
        self.dev.execute(self.cdb(cmd, args), None, buf)
        return bytes(buf)

    def data_out(self, cmd, args, data):
        self.dev.execute(self.cdb(cmd, args), data, None)

    @staticmethod
    def _addr_len_crc(addr, data):
        return addr.to_bytes(4, 'big') + len(data).to_bytes(2, 'big') + b'\0' + crc16(data).to_bytes(2, 'little')

    # ROM and loader
    def mem_write(self, addr, data):
        self.data_out(self.WRITE_MEMORY, self._addr_len_crc(addr, data), data)

    def jump(self, addr, arg):
        self.call(self.JUMP, addr.to_bytes(4, 'big') + arg.to_bytes(2, 'big'))

    # loader
    def chip_key(self):
        r = self.call(self.READ_KEY, (0xAC6900).to_bytes(4, 'big'))
        return int.from_bytes(crc_cipher(r[4:6][::-1]), 'little')

    def flash_id(self):
        return int.from_bytes(self.call(self.READ_ID)[:3], 'big')

    def online_device(self):
        r = self.call(self.ONLINE_DEVICE)
        return r[0], int.from_bytes(r[2:6], 'little')

    def buffer_size(self):
        return int.from_bytes(self.call(self.USB_BUFFER_SIZE)[:4], 'big')

    def read(self, addr, n):
        return self.data_in(self.READ_FLASH, addr.to_bytes(4, 'big') + n.to_bytes(2, 'big'), n)

    def write(self, addr, data):
        self.data_out(self.WRITE_FLASH, self._addr_len_crc(addr, data), data)

    def erase_sector(self, addr):
        self.call(self.ERASE_SECTOR, addr.to_bytes(4, 'big'))

    def erase_block(self, addr):
        self.call(self.ERASE_BLOCK, addr.to_bytes(4, 'big'))

    def run_app(self):
        self.call(self.RUN_APP, (1).to_bytes(4, 'big'))


class Connection:
    """An open download-mode session with the loader running."""

    def __init__(self, path, log=print):
        self.path = path
        self.dev = SCSIDevice(path)
        try:
            vendor, product = self.dev.inquiry()
            if vendor != VENDOR or product != PRODUCT:
                raise SCSIError(f'{path} is "{vendor} {product}", not a {VENDOR} {PRODUCT} download-mode device')
            self.p = Protocol(self.dev)
            blob = loader_blob()
            log('Starting the flash loader')
            for a in range(0, len(blob), 512):
                self.p.mem_write(LOADER_ADDR + a, blob[a:a + 512])
            self.p.jump(LOADER_ADDR, LOADER_ARG)
            time.sleep(0.1)
            self.chip_key = self.p.chip_key()
            self.flash_id = self.p.online_device()[1]      # JEDEC ID, as jl-uboot-tool shows it
            try:
                self.io = min(max(self.p.buffer_size(), 256), 4096)
            except SCSIError:
                self.io = 512
        except Exception:
            self.dev.close()
            raise

    def close(self):
        self.dev.close()

    def read(self, addr, n, progress=None):
        out = bytearray()
        while len(out) < n:
            k = min(self.io, n - len(out))
            out += self.p.read(addr + len(out), k)
            if progress:
                progress(len(out), n)
        return bytes(out)

    def erase(self, addr, n):
        """Erase [addr, addr + n): 64 KiB blocks where aligned, else 4 KiB."""
        end = addr + n
        while addr < end:
            if addr % 0x10000 == 0 and end - addr >= 0x10000:
                self.p.erase_block(addr)
                addr += 0x10000
            else:
                self.p.erase_sector(addr)
                addr += 0x1000

    def program(self, addr, data):
        """Program erased flash; skips chunks that are all 0xFF."""
        for o in range(0, len(data), self.io):
            chunk = data[o:o + self.io]
            if chunk.count(0xFF) != len(chunk):
                self.p.write(addr + o, chunk)
