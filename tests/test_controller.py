"""The window's controller against a simulated unit: connect, plan, write
with per-sector progress, done. Needs FLASHER_TEST_DATA (see test_ops.py)."""
import os
import shutil
import tempfile
import time
import unittest

from flasher.controller import Controller
from flasher.formats import Package
from tests.test_ops import FakeConn, data


class Unit(FakeConn):
    path = '/dev/sg0'

    def program(self, addr, data, progress=None):
        super().program(addr, data)
        if progress:
            progress(len(data))

    def close(self):
        pass


class Dialogs:
    path = None

    def open_file(self, title, types, start=None):
        return self.path

    def open_folder(self, path):
        pass


def settle(ctl):
    time.sleep(0.05)
    while ctl.busy:
        time.sleep(0.01)


class ControllerTest(unittest.TestCase):
    def setUp(self):
        self.unit = Unit(data('smk37-snoop37.bin'))
        self.dialogs = Dialogs()
        self.ctl = Controller(tempfile.mkdtemp(), dialogs=self.dialogs,
                              find=lambda: [('/dev/sg0', 'x')], connect=lambda p, log: self.unit)
        self.tmp = tempfile.mkdtemp()

    def connect(self):
        self.ctl._connect_to('/dev/sg0')
        settle(self.ctl)
        self.assertEqual(self.ctl.phase, 'ready')

    def test_firmware_cycle(self):
        self.connect()
        st = self.ctl.snapshot()
        self.assertEqual(st['device']['model'], 'SMK-37 Pro / Elite')
        self.assertEqual(st['map'][0x04], 'firmware')
        self.assertEqual(st['map'][0xC1], 'calibration')
        pkg = os.path.join(self.tmp, 'SMK-37 Pro_016.fwsc')
        shutil.copy(os.path.join(os.environ['FLASHER_TEST_DATA'], 'smk37-016.fwsc'), pkg)
        self.dialogs.path = pkg
        self.ctl.choose_file('firmware')
        settle(self.ctl)
        st = self.ctl.snapshot()
        self.assertTrue(st['plan']['ready'])
        self.assertEqual(st['plan']['count'], 153)
        self.assertEqual(st['plan']['short'], '0x04000-0x9CFFF')
        self.assertEqual(len(st['marks']), 153)
        self.ctl.confirm()
        settle(self.ctl)
        st = self.ctl.snapshot()
        self.assertEqual(st['phase'], 'done')
        self.assertEqual(st['alert']['kind'], 'done')
        self.assertEqual(set(st['marks'].values()), {'verified'})
        self.assertEqual(st['stage'], dict(read='done', backup='done', write='done', verify='done'))
        self.assertTrue(st['backup_saved'])
        flash = Package(data('smk37-016.fwsc')).flash
        self.assertEqual(bytes(self.unit.mem[0x4000:len(flash)]), flash[0x4000:])
        self.ctl.dismiss()
        settle(self.ctl)
        st = self.ctl.snapshot()
        self.assertEqual(st['plan']['count'], 0)            # nothing left to write
        self.assertEqual(st['stage']['write'], 'idle')

    def test_wrong_package_refused(self):
        self.connect()
        self.dialogs.path = os.path.join(os.environ['FLASHER_TEST_DATA'], 'fm1-015.fwsc')
        self.ctl.choose_file('firmware')
        settle(self.ctl)
        st = self.ctl.snapshot()
        self.assertEqual(st['plan']['kind'], 'refused')
        self.assertEqual(st['alert']['kind'], 'refused')
        self.ctl.confirm()                  # no plan: nothing happens
        settle(self.ctl)
        self.assertEqual(self.unit.programmed, [])

    def test_backup(self):
        self.connect()
        self.ctl.select_mode('backup')
        settle(self.ctl)
        self.ctl.start_action()
        settle(self.ctl)
        st = self.ctl.snapshot()
        self.assertEqual(st['phase'], 'done')
        self.assertEqual(len(st['backups']), 1)
        self.assertEqual(self.unit.programmed, [])


if __name__ == '__main__':
    unittest.main()
