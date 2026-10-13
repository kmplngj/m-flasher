"""The devices this tool knows, and how to tell them apart from a flash
image."""
import re

from .formats import (BOOT_AREA, app_files_status, app_read, app_table, boot_table,
                      bootloader_identity, stored_bootloader)

FLASH_SIZE = 0x100000
CHIP_KEY = 0x980F           # every FM-1 and SMK-37 seen so far
KEY_MAC = (0xFF000, 0x100000)


class Board:
    def __init__(self, key, name, tag, models, flash_id, factory):
        self.key = key
        self.name = name                # shown to the user
        self.tag = tag                  # board name in our bootloader's identity block
        self.models = models            # package models (identity without version)
        self.flash_id = flash_id        # JEDEC ID seen on units (a hint only)
        self.factory = factory          # per-unit data, never written from another unit's image

    def __repr__(self):
        return self.name


FM1 = Board('fm1', 'FM-1', 'fm1', ['FM-1'], 0x856014, [KEY_MAC])
SMK37 = Board('smk37', 'SMK-37 Pro / Elite', 'smk37', ['SMK-37 Pro', 'SMK-37 Elite'], 0xEB6014,
              [(0xC1000, 0xC2000), KEY_MAC])     # USRTRIM: factory key and pad calibration
BOARDS = [FM1, SMK37]

SMK_TRIM_MARKER = (0xC1224, b'trimdat\0')
GUARD_COPY = (0xFF000, b'TRIMCOPY')          # our bootloader's copy of the SMK-37 calibration

_IDENT = re.compile(rb'(FM-1|SMK-37 Pro|SMK-37 Elite|SMK-37\+VA)_(\d{3})')


class Analysis:
    """What a flash image says about the unit: the model (with the evidence),
    the bootloader and the installed firmware."""

    def __init__(self, flash, flash_id=None, chip_key=CHIP_KEY):
        self.flash = flash
        self.evidence = []          # (board, strong, text)
        self._model(flash, flash_id, chip_key)
        self._bootloader(flash)
        self._firmware(flash, chip_key)

    # -- model --

    def _model(self, flash, flash_id, key):
        ev = self.evidence
        ours = bootloader_identity(stored_bootloader(flash) or b'')
        if ours:
            for b in BOARDS:
                if b.tag == ours[0]:
                    ev.append((b, True, f'the installed open bootloader is the {b.name} build'))
        a, marker = SMK_TRIM_MARKER
        if flash[a:a + len(marker)] == marker:
            ev.append((SMK37, True, 'SMK-37 factory calibration found'))
        a, magic = GUARD_COPY
        if flash[a:a + len(magic)] == magic:
            ev.append((SMK37, True, 'copy of the SMK-37 calibration found'))
        regions = {e.name: e.addr for e in app_table(flash, key) if e.attr in (0x12, 0x92)}
        if regions.get('USRTRIM') == 0xC1000:
            ev.append((SMK37, False, 'the firmware has the SMK-37 calibration area'))
        elif regions.get('VM') == 0x93000 and regions.get('USR') == 0xEA000:
            ev.append((FM1, False, 'the firmware uses the FM-1 flash layout'))
        if flash_id is not None:
            for b in BOARDS:
                if flash_id == b.flash_id:
                    ev.append((b, False, f'flash chip 0x{flash_id:06X}'))
        strong = {b for b, s, _ in ev if s}
        weak = {b for b, s, _ in ev if not s}
        if len(strong) == 1:
            self.board, self.certain = strong.pop(), True
            if weak - {self.board}:
                self.certain = False        # something disagrees: let the user decide
        elif not strong and len(weak) == 1:
            # two independent signals that agree (layout and flash chip), nothing against
            self.board = weak.pop()
            self.certain = sum(1 for b, _, _ in ev if b is self.board) >= 2
        else:
            self.board, self.certain = None, False

    # -- bootloader --

    def _bootloader(self, flash):
        boot = stored_bootloader(flash)
        if boot is None:
            self.bootloader = 'damaged' if boot_table(flash) == [] else 'unknown'
            self.bootloader_text = 'damaged or missing' if self.bootloader == 'damaged' else 'unreadable'
        elif bootloader_identity(boot):
            board, version = bootloader_identity(boot)
            self.bootloader = 'open'
            self.bootloader_text = f'open bootloader {version} ({board})'
        else:
            self.bootloader = 'stock'
            self.bootloader_text = 'stock (M-VAVE)'

    # -- firmware --

    def _firmware(self, flash, key):
        self.app_status = app_files_status(flash, key)
        self.firmware_text = 'none' if self.app_status == 'not found' else 'unknown'
        self.model_name = None
        if self.app_status == 'not found':
            return
        table = app_table(flash, key)
        app = [e for e in table if e.name == 'app.bin']
        if not app or BOOT_AREA + app[0].addr + app[0].len > len(flash):
            return
        text = app_read(flash, BOOT_AREA, app[0].addr, app[0].len, key)
        idents = sorted({(m.group(1).decode(), m.group(2).decode()) for m in _IDENT.finditer(text)},
                        key=lambda x: x[1])
        if idents:
            self.model_name = idents[-1][0]      # the firmware's own name: its highest version
        sloop = re.search(rb'SLOOP (\d+\.\d+(?:\.\d+)?)', text)
        if sloop:
            name = 'SLOOP ' + sloop.group(1).decode()
            if self.model_name and self.model_name.startswith('SMK'):
                name += ' (Snoop37)'
        elif b'Felucca' in text:
            name = 'Felucca'
        elif idents:
            model, version = idents[-1]
            name = f'{model} {version}'
            if model == 'SMK-37+VA':
                name += ' (Baud Girl)'
            elif version not in ('900', '000') and len(idents) == 1:
                name += ' (stock)'
        else:
            name = 'unknown'
        self.firmware_text = name

    def summary(self):
        model = self.board.name if self.board else 'unknown'
        if self.board and not self.certain:
            model += ' (probably)'
        return {
            'model': model,
            'bootloader': self.bootloader_text,
            'firmware': self.firmware_text,
            'app': APP_TEXT[self.app_status],
        }


APP_TEXT = {
    'ok': 'OK',
    'bad crc': 'DAMAGED (checksum fails; it may still start)',
    'incomplete': 'INCOMPLETE (partly erased: an interrupted write)',
    'not found': 'none installed',
}
