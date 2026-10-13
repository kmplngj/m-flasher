"""Tests against a simulated download-mode device with NOR flash behaviour.

The real-image tests need flash dumps and M-VAVE packages, which cannot be
published: point FLASHER_TEST_DATA at a folder with smk37-stock.bin,
smk37-snoop37.bin, smk37-baudgirl.bin, fm1-stock.bin, smk37-016.fwsc and fm1-015.fwsc; they are
skipped otherwise.
"""
import os
import tempfile
import unittest

from flasher import ops
from flasher.boards import FM1, SMK37, Analysis
from flasher.formats import (BOOT_AREA, Package, app_files_status, bootloader_identity,
                             build_boot_area, stored_bootloader)

DATA = os.environ.get('FLASHER_TEST_DATA', '')


def data(name):
    path = os.path.join(DATA, name)
    if not DATA or not os.path.exists(path):
        raise unittest.SkipTest(f'{name} not in FLASHER_TEST_DATA')
    with open(path, 'rb') as f:
        return f.read()


class FakeConn:
    """NOR flash: erase sets a 4 KiB sector (or 64 KiB block) to 0xFF,
    programming ANDs bits in. fail_writes: corrupt that many programs."""

    def __init__(self, flash, chip_key=0x980F, flash_id=0xEB6014):
        self.mem = bytearray(flash)
        self.chip_key, self.flash_id = chip_key, flash_id
        self.fail_writes = 0
        self.erased, self.programmed = [], []

    def read(self, addr, n, progress=None):
        if progress:
            progress(n, n)
        return bytes(self.mem[addr:addr + n])

    def erase(self, addr, n):
        assert addr % 0x1000 == 0 and n % 0x1000 == 0
        self.erased.append((addr, n))
        self.mem[addr:addr + n] = b'\xff' * n

    def program(self, addr, data):
        self.programmed.append((addr, len(data)))
        data = bytearray(data)
        if self.fail_writes:
            self.fail_writes -= 1
            data[len(data) // 2] ^= 0x01
        for i, b in enumerate(data):
            self.mem[addr + i] &= b


def session(flash, **kw):
    s = ops.Session(FakeConn(flash, **kw), tempfile.mkdtemp(), log=lambda m: None)
    return s


class Synthetic(unittest.TestCase):
    def test_plan_refuses_changes_outside_allowed(self):
        cur = b'\xff' * 0x100000
        tgt = bytearray(cur)
        tgt[0x5000] = 0
        with self.assertRaises(ops.Refused):
            ops.Plan('x', cur, tgt, [(0, 0x4000)])

    def test_plan_runs(self):
        cur = b'\xff' * 0x100000
        tgt = bytearray(cur)
        for a in (0x4000, 0x5000, 0x9000):
            tgt[a] = 0
        p = ops.Plan('x', cur, tgt, [(0, 0x100000)])
        self.assertEqual(p.runs(), [(0x4000, 0x6000), (0x9000, 0xA000)])

    def test_bundled_bootloaders(self):
        for b in (FM1, SMK37):
            boot, version = ops.bundled_bootloader(b)
            self.assertEqual(bootloader_identity(boot)[0], b.tag)
            self.assertLessEqual(0xA0 + len(boot), 0x38D0)

    def test_reads_must_match(self):
        class Flaky(FakeConn):
            n = 0

            def read(self, addr, n, progress=None):
                self.n += 1
                out = bytearray(super().read(addr, n))
                if self.n == 2:
                    out[100] ^= 1
                return bytes(out)
        with self.assertRaises(ops.Refused):
            ops.Session(Flaky(b'\xff' * 0x100000), tempfile.mkdtemp(), log=lambda m: None)


class RealImages(unittest.TestCase):
    def test_analysis(self):
        a = Analysis(data('smk37-stock.bin'), 0xEB6014)
        self.assertIs(a.board, SMK37)
        self.assertTrue(a.certain)
        self.assertEqual(a.bootloader, 'stock')
        self.assertEqual(a.firmware_text, 'SMK-37 Pro 016 (stock)')
        a = Analysis(data('fm1-stock.bin'), 0x856014)
        self.assertIs(a.board, FM1)
        self.assertTrue(a.certain)
        a = Analysis(data('smk37-snoop37.bin'))
        self.assertIs(a.board, SMK37)
        self.assertEqual(a.firmware_text, 'SLOOP 2.6 (Snoop37)')

    def test_package_boot_area_round_trip(self):
        pkg = Package(data('smk37-016.fwsc'))
        boot = stored_bootloader(pkg.flash)
        self.assertEqual(build_boot_area(pkg.flash, boot), pkg.flash[:BOOT_AREA])

    def test_firmware_over_snoop37(self):
        s = session(data('smk37-snoop37.bin'))
        p = s.plan_firmware(data('smk37-016.fwsc'))
        self.assertTrue(p.sectors)
        self.assertTrue(all(BOOT_AREA <= a < 0x9D000 for a in p.sectors))
        s.apply(p)
        stock = data('smk37-stock.bin')
        self.assertEqual(s.conn.mem[BOOT_AREA:0x9D000], stock[BOOT_AREA:0x9D000])
        self.assertEqual(app_files_status(bytes(s.conn.mem), 0x980F), 'ok')
        self.assertTrue(os.path.exists(s.backup_path))

    def test_firmware_layout_change_warned(self):
        s = session(data('smk37-baudgirl.bin'))
        self.assertTrue(s.plan_firmware(data('smk37-016.fwsc')).warnings)
        s = session(data('smk37-snoop37.bin'))        # the stock layout
        self.assertFalse(s.plan_firmware(data('smk37-016.fwsc')).warnings)

    def test_firmware_wrong_model_refused(self):
        s = session(data('smk37-stock.bin'))
        with self.assertRaises(ops.Refused):
            s.plan_firmware(data('fm1-015.fwsc'))

    def test_firmware_wrong_chip_key_refused(self):
        s = session(data('smk37-stock.bin'), chip_key=0x1234)
        with self.assertRaises(ops.Refused):
            s.plan_firmware(data('smk37-016.fwsc'))

    def test_firmware_wipe_settings(self):
        s = session(data('smk37-snoop37.bin'))
        p = s.plan_firmware(data('smk37-016.fwsc'), wipe_settings=True)
        self.assertIn((0x9D000, 0xC0000), p.allowed)
        self.assertIn((0xC0000, 0xC1000), p.allowed)
        self.assertFalse(any(0xC1000 <= a < 0xC2000 for a in p.sectors))

    def test_open_bootloader(self):
        stock = data('smk37-stock.bin')
        s = session(stock)
        p = s.plan_bootloader()
        self.assertTrue(p.sectors)
        self.assertTrue(all(a < BOOT_AREA for a in p.sectors))
        s.apply(p)
        mem = bytes(s.conn.mem)
        self.assertEqual(bootloader_identity(stored_bootloader(mem))[0], 'smk37')
        self.assertEqual(mem[0x38D0:], stock[0x38D0:])          # isd_config and the rest kept
        # and back to stock from the package
        p = s.plan_bootloader(raw_package=data('smk37-016.fwsc'))
        s.apply(p)
        self.assertEqual(bytes(s.conn.mem), stock)

    def test_fm1_bootloader_refused_on_smk(self):
        s = session(data('smk37-stock.bin'))
        boot, _ = ops.bundled_bootloader(FM1)
        with self.assertRaises(ops.Refused):
            s.plan_bootloader(boot=boot)

    def test_restore_same_unit(self):
        stock = data('smk37-stock.bin')
        s = session(data('smk37-snoop37.bin'))
        p = s.plan_restore(stock)
        self.assertEqual(p.allowed, [(0, 0x100000)])
        s.apply(p)
        self.assertEqual(bytes(s.conn.mem), stock)

    def test_restore_other_unit_keeps_factory_data(self):
        stock = bytearray(data('smk37-stock.bin'))
        other = bytearray(stock)
        other[0xC1100] ^= 0xFF          # another unit's calibration
        other[0xFF800] ^= 0xFF          # and key_mac
        s = session(bytes(stock))
        p = s.plan_restore(bytes(other))
        self.assertTrue(p.warnings)
        self.assertEqual(p.sectors, [])

    def test_restore_fm1_backup_on_smk_refused(self):
        s = session(data('smk37-stock.bin'))
        with self.assertRaises(ops.Refused):
            s.plan_restore(data('fm1-stock.bin'))

    def test_write_retried_then_refused(self):
        s = session(data('smk37-snoop37.bin'))
        p = s.plan_firmware(data('smk37-016.fwsc'))
        s.conn.fail_writes = 1
        s.apply(p)                      # one bad program: rewritten
        s2 = session(data('smk37-snoop37.bin'))
        p = s2.plan_firmware(data('smk37-016.fwsc'))
        s2.conn.fail_writes = 10 ** 6
        with self.assertRaises(ops.Refused):
            s2.apply(p)


if __name__ == '__main__':
    unittest.main()
