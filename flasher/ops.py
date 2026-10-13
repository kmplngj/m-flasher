"""The operations, with their safety rules.

Every session starts by reading the whole flash twice; the two reads must
match. Each change is planned against that image: a target image and the
address ranges the operation may touch. Only the 4 KiB sectors that differ
inside those ranges are erased and written, each run is read back and
compared (and rewritten once on a mismatch), and the image is saved as a
backup before the first write of a session.
"""
import datetime
import hashlib
import json
import os

from . import __version__
from .boards import BOARDS, CHIP_KEY, FLASH_SIZE, Analysis
from .formats import (BOOT_AREA, SECTOR, FormatError, Package, app_files_status, app_table,
                      bankcb_check, bootloader_identity, build_boot_area, stored_bootloader)

DATA = os.path.join(os.path.dirname(__file__), 'data')


class Refused(Exception):
    """An operation the tool will not do, with the reason."""


def bundled_bootloader(board):
    path = os.path.join(DATA, f'bootloader-{board.key}.boot')
    with open(path, 'rb') as f:
        boot = f.read()
    ident = bootloader_identity(boot)
    if not ident or ident[0] != board.tag:
        raise Refused(f'the bundled bootloader for the {board.name} is damaged')
    return boot, ident[1]


def _ranges_text(ranges):
    return ', '.join(f'0x{a:05X}-0x{b - 1:05X}' for a, b in ranges)


class Plan:
    """A change: the target image, the ranges it may touch, the sectors that
    differ and what the user should know before confirming."""

    def __init__(self, title, current, target, allowed, notes=(), warnings=()):
        self.title = title
        self.target = bytes(target)
        self.allowed = list(allowed)
        self.notes = list(notes)
        self.warnings = list(warnings)
        if len(self.target) != len(current):
            raise Refused('internal error: image size mismatch')
        self.sectors = []
        for a in range(0, len(current), SECTOR):
            if current[a:a + SECTOR] != self.target[a:a + SECTOR]:
                if not any(s <= a and a + SECTOR <= e for s, e in self.allowed):
                    raise Refused(f'internal error: the change at 0x{a:05X} is outside the '
                                  f'allowed ranges ({_ranges_text(self.allowed)})')
                self.sectors.append(a)

    def runs(self):
        out = []
        for a in self.sectors:
            if out and out[-1][1] == a:
                out[-1][1] = a + SECTOR
            else:
                out.append([a, a + SECTOR])
        return [tuple(r) for r in out]

    def describe(self):
        lines = [self.title]
        if not self.sectors:
            lines.append('Nothing to write: the flash already holds this.')
        else:
            lines.append(f'Writes {len(self.sectors)} sectors ({len(self.sectors) * 4} KiB): '
                         + _ranges_text(self.runs()))
        lines += self.notes
        return '\n'.join(lines)


class Session:
    """A connected unit: its flash image as read, and the operations."""

    def __init__(self, conn, backup_dir, log=print, progress=None):
        self.conn = conn
        self.backup_dir = backup_dir
        self.log = log
        self.progress = progress or (lambda done, total: None)
        self.backup_path = None
        self.board_override = None
        if conn.chip_key != CHIP_KEY:
            self.log(f'Note: chip key 0x{conn.chip_key:04X} (0x{CHIP_KEY:04X} expected)')
        self.flash = self._read_twice()
        self.analysis = Analysis(self.flash, conn.flash_id, conn.chip_key)

    # -- reading --

    def _read_twice(self):
        self.log('Reading the flash')
        a = self.conn.read(0, FLASH_SIZE, lambda d, t: self.progress(d, 2 * t))
        self.log('Reading it again to compare')
        b = self.conn.read(0, FLASH_SIZE, lambda d, t: self.progress(t + d, 2 * t))
        if a != b:
            raise Refused('the two reads of the flash differ: the USB connection is unreliable. '
                          'Nothing was written. Try another cable or port, without a hub.')
        return a

    @property
    def board(self):
        return self.board_override or self.analysis.board

    def info(self):
        s = self.analysis.summary()
        s['chip key'] = f'0x{self.conn.chip_key:04X}'
        s['flash'] = f'0x{self.conn.flash_id:06X}'
        return s

    # -- backup --

    def save_backup(self, directory=None):
        directory = directory or self.backup_dir
        os.makedirs(directory, exist_ok=True)
        stamp = datetime.datetime.now().strftime('%Y%m%d-%H%M%S')
        model = (self.board.key if self.board else 'unknown')
        path = os.path.join(directory, f'{model}-{stamp}.bin')
        with open(path, 'wb') as f:
            f.write(self.flash)
        meta = dict(self.info(), sha256=hashlib.sha256(self.flash).hexdigest(),
                    saved=stamp, tool=f'fm-flasher {__version__}')
        with open(path[:-4] + '.json', 'w') as f:
            json.dump(meta, f, indent=2)
        self.backup_path = path
        self.log(f'Backup saved: {path}')
        return path

    # -- planning --

    def _need_board(self):
        if not self.board:
            raise Refused('the model could not be identified: choose it first')
        return self.board

    def plan_restore(self, data):
        if len(data) != FLASH_SIZE:
            raise Refused(f'not a full backup: {len(data)} bytes, {FLASH_SIZE} expected')
        backup = Analysis(data)
        board = self.board
        notes, warnings = [], []
        if backup.board and board and backup.board is not board:
            raise Refused(f'this backup is from a {backup.board.name}, the connected unit is a {board.name}')
        factory = board.factory if board else [(0xFF000, FLASH_SIZE)]
        same_unit = all(data[a:b] == self.flash[a:b] or self.flash[a:b] == b'\xff' * (b - a)
                        for a, b in factory)
        if same_unit:
            allowed = [(0, FLASH_SIZE)]
            notes.append('The whole flash is restored, the bootloader included.')
            for a, b in factory:
                if data[a:b] != self.flash[a:b]:
                    warnings.append(f'This unit\'s factory data at {_ranges_text([(a, b)])} is blank; '
                                    'the backup\'s is restored. Use only a backup of this unit.')
        else:
            allowed, at = [], 0
            for a, b in sorted(factory):
                if a > at:
                    allowed.append((at, a))
                at = b
            if at < FLASH_SIZE:
                allowed.append((at, FLASH_SIZE))
            warnings.append('This backup seems to come from another unit: its factory data '
                            f'({_ranges_text(factory)}: calibration, Bluetooth address) is not '
                            'restored, this unit keeps its own.')
        target = bytearray(self.flash)
        for a, b in allowed:
            target[a:b] = data[a:b]
        return Plan('Restore a backup', self.flash, target, allowed, notes, warnings)

    def plan_firmware(self, raw, with_bootloader=False, wipe_settings=False):
        try:
            pkg = Package(raw)
        except FormatError as e:
            raise Refused(str(e))
        board = self._need_board()
        if pkg.model not in board.models:
            raise Refused(f'this package is for the {pkg.model}, the connected unit is a {board.name}')
        if len(pkg.flash) > FLASH_SIZE:
            raise Refused('the package is larger than the flash')
        padded = pkg.flash + b'\xff' * (FLASH_SIZE - len(pkg.flash))
        if app_files_status(padded, self.conn.chip_key) != 'ok':
            raise Refused('the package\'s firmware does not decrypt with this unit\'s chip key '
                          f'(0x{self.conn.chip_key:04X}): writing it would not start')
        warnings, notes = [], []
        installed = self.analysis.model_name
        if installed and installed.startswith('SMK-37 ') and installed != pkg.model:
            warnings.append(f'The unit runs {installed} firmware; this package is for the {pkg.model}. '
                            'The unit will then report itself as that model.')
        start = 0 if with_bootloader else BOOT_AREA
        end = len(pkg.flash)
        target = bytearray(self.flash)
        target[start:end] = pkg.flash[start:end]
        allowed = [(start, end)]
        notes.append(f'Installs {pkg.identity} ({_ranges_text(allowed)}).')
        if with_bootloader:
            notes.append('Also puts back the package\'s (stock) bootloader.')
        if wipe_settings:
            for e in app_table(padded, self.conn.chip_key):
                if e.name in ('VM', 'BTIF') and e.attr in (0x12, 0x92) and e.len and \
                        e.addr % SECTOR == 0 and e.addr + e.len <= FLASH_SIZE:
                    if any(a < e.addr + e.len and e.addr < b for a, b in board.factory):
                        raise Refused(f'the package declares {e.name} over factory data')
                    target[e.addr:e.addr + e.len] = b'\xff' * e.len
                    allowed.append((e.addr, e.addr + e.len))
                    notes.append(f'Erases {e.name} (0x{e.addr:05X}-0x{e.addr + e.len - 1:05X}): '
                                 + ('settings' if e.name == 'VM' else 'Bluetooth pairings'))
        return Plan(f'Install firmware {pkg.identity}', self.flash, target, allowed, notes, warnings)

    def plan_bootloader(self, boot=None, raw_package=None):
        """Our bundled bootloader (boot None), a given uboot.boot, or the stock
        one from a package's boot area."""
        board = self._need_board()
        if raw_package is not None:
            try:
                pkg = Package(raw_package)
            except FormatError as e:
                raise Refused(str(e))
            if pkg.model not in board.models:
                raise Refused(f'this package is for the {pkg.model}, the connected unit is a {board.name}')
            boot = stored_bootloader(pkg.flash)
            if boot is None:
                raise Refused('the package has no readable bootloader')
            title = f'Put back the stock bootloader (from {pkg.identity})'
        else:
            if boot is None:
                boot, version = bundled_bootloader(board)
            else:
                try:
                    bankcb_check(boot)
                except FormatError as e:
                    raise Refused(str(e))
                ident = bootloader_identity(boot)
                if ident and ident[0] != board.tag:
                    raise Refused(f'this bootloader is built for "{ident[0]}", the unit is a {board.name}')
                version = ident[1] if ident else 'unknown'
            title = f'Install the open bootloader {version} for the {board.name}'
        try:
            area = build_boot_area(self.flash, boot)
        except FormatError as e:
            raise Refused(f'{e}; install firmware with its bootloader instead')
        target = area + self.flash[BOOT_AREA:]
        notes = ['Only the boot area (0x00000-0x03FFF) is written; the unit\'s own '
                 'configuration in it is kept.']
        return Plan(title, self.flash, target, [(0, BOOT_AREA)], notes)

    # -- writing --

    def apply(self, plan):
        if not plan.sectors:
            self.log('Nothing to write')
            return
        if not self.backup_path:
            self.save_backup()
        total, done = len(plan.sectors) * SECTOR, 0
        current = bytearray(self.flash)
        for a, b in plan.runs():
            want = plan.target[a:b]
            for attempt in (1, 2):
                self.log(f'Writing 0x{a:05X}-0x{b - 1:05X}' + (' (again)' if attempt == 2 else ''))
                self.conn.erase(a, b - a)
                self.conn.program(a, want)
                got = self.conn.read(a, b - a)
                if got == want:
                    break
                if attempt == 2:
                    current[a:b] = got
                    self.flash = bytes(current)
                    raise Refused(f'0x{a:05X}-0x{b - 1:05X} does not read back as written. '
                                  'Do not switch the unit off: try again, with another cable or port. '
                                  f'Your backup: {self.backup_path}')
            current[a:b] = want
            done += b - a
            self.progress(done, total)
        self.flash = bytes(current)
        self.analysis = Analysis(self.flash, self.conn.flash_id, self.conn.chip_key)
        self.log('Done: everything written was read back and matches')
