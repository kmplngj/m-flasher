"""Save the window's states as flasher/ui/demo/NAME.js for previews in a browser
(index.html?demo=NAME), by driving the real controller against a simulated
unit. Needs the real images in FLASHER_TEST_DATA (see tests/test_ops.py).

    FLASHER_TEST_DATA=... python3 tools/ui_demo.py
"""
import json
import os
import shutil
import sys
import tempfile
import threading
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'tests'))
from flasher.controller import Controller  # noqa: E402
from test_ops import FakeConn  # noqa: E402

DATA = os.environ['FLASHER_TEST_DATA']
OUT = os.path.join(os.path.dirname(__file__), '..', 'flasher', 'ui', 'demo')


def img(name):
    with open(os.path.join(DATA, name), 'rb') as f:
        return f.read()


class Unit(FakeConn):
    def __init__(self, flash, flash_id=0xEB6014):
        super().__init__(flash, flash_id=flash_id)
        self.path = '/dev/sg0'
        self.gate = None                # (sector, Event): stop writing there

    def program(self, addr, data, progress=None):
        for o in range(0, len(data), 4096):
            super().program(addr + o, data[o:o + 4096])
            if progress:
                progress(o + 4096)
            if self.gate and (addr + o) // 4096 + 1 == self.gate[0]:
                self.gate[1].wait()

    def close(self):
        pass


class Dialogs:
    def __init__(self):
        self.next = None

    def open_file(self, title, types, start=None):
        return self.next

    def open_folder(self, path):
        pass


def wait(ctl):
    time.sleep(0.05)
    while ctl.busy:
        time.sleep(0.02)


def save(ctl, name, review=False, **extra):
    st = ctl.snapshot()
    st.update(extra)
    shown = 'C:\\Users\\you\\Documents\\M-Flasher\\backups'     # instead of the temporary folder
    st['backup_dir'] = shown
    st['log'] = [l.replace(ctl.backup_dir + os.sep, shown + '\\') for l in st['log']]
    if st.get('alert'):
        st['alert'] = dict(st['alert'], text=st['alert']['text'].replace(ctl.backup_dir + os.sep, shown + '\\'))
    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, name + '.js'), 'w') as f:
        f.write('window.MF_DEMO = ' + json.dumps(st) + ';\n')
        if review:
            f.write('window.MF_DEMO_REVIEW = true;\n')
    print('saved', name)


def session(flash, flash_id=0xEB6014, backups=()):
    bdir = tempfile.mkdtemp()
    for name, src in backups:
        shutil.copy(os.path.join(DATA, src), os.path.join(bdir, name))
        with open(os.path.join(bdir, name[:-4] + '.json'), 'w') as f:
            json.dump({'firmware': 'SMK-37 Pro 016 (stock)', 'model': 'SMK-37 Pro / Elite'}, f)
    unit = Unit(flash, flash_id)
    dialogs = Dialogs()
    ctl = Controller(bdir, dialogs=dialogs, find=lambda: [('/dev/sg0', 'WL82 UBOOT1.00')],
                     connect=lambda path, log: unit)
    ctl.backup_dir = 'C:\\Users\\you\\Documents\\M-Flasher\\backups' if False else bdir
    return ctl, unit, dialogs


def main():
    fw = os.path.join(DATA, 'smk37-016.fwsc')
    tmp = tempfile.mkdtemp()
    pkg = os.path.join(tmp, 'SMK-37 Pro_016.fwsc')
    shutil.copy(fw, pkg)

    # waiting
    ctl, unit, dlg = session(img('smk37-snoop37.bin'))
    ctl.log('M-Flasher started · open bootloader d16c403 bundled')
    ctl.log('listening for a unit in download mode…')
    save(ctl, 'waiting')

    # connected, firmware chosen, review
    ctl._connect_to('/dev/sg0'); wait(ctl)
    save(ctl, 'connected')
    dlg.next = pkg
    ctl.choose_file('firmware'); wait(ctl)
    save(ctl, 'review')
    save(ctl, 'review-modal', review=True)

    # writing, stopped at sector 0x5C
    gate = threading.Event()
    unit.gate = (0x5C, gate)
    ctl.confirm()
    time.sleep(1.0)
    save(ctl, 'writing')
    gate.set(); wait(ctl)
    save(ctl, 'done')
    ctl.dismiss(); wait(ctl)

    # bootloader, open
    ctl.select_mode('bootloader'); wait(ctl)
    save(ctl, 'bootloader')

    # restore with recent backups
    ctl2, _, dlg2 = session(img('smk37-baudgirl.bin'), backups=[('smk37-20261014-020413.bin', 'smk37-stock.bin'),
                                                                 ('smk37-20261013-221530.bin', 'smk37-snoop37.bin')])
    ctl2._connect_to('/dev/sg0'); wait(ctl2)
    ctl2.select_mode('restore'); wait(ctl2)
    ctl2.pick_backup(ctl2.recent_backups()[1]['path']); wait(ctl2)
    save(ctl2, 'restore')
    # firmware over Baud Girl: the layout warning
    dlg2.next = pkg
    ctl2.select_mode('firmware'); wait(ctl2)
    ctl2.choose_file('firmware'); wait(ctl2)
    save(ctl2, 'warning', review=True)
    # refused: an FM-1 package
    fm = os.path.join(tmp, 'FM-1_015.fwsc')
    shutil.copy(os.path.join(DATA, 'fm1-015.fwsc'), fm)
    dlg2.next = fm
    ctl2.choose_file('firmware'); wait(ctl2)
    save(ctl2, 'refused')
    ctl2.select_mode('backup'); wait(ctl2)
    save(ctl2, 'backup')


if __name__ == '__main__':
    main()
