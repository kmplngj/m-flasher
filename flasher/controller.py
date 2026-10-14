"""The app's logic, independent of the window: finds the unit, connects,
plans the selected operation, writes it with progress per sector, and
keeps one state the window renders (snapshot()). The window calls the
methods below; work runs in threads and every change is pushed through
the push(state) callback, at most every PUSH_S seconds."""
import datetime
import glob
import json
import os
import threading
import time

from . import __version__, ops
from .boards import BOARDS, CHIP_KEY
from .formats import SECTOR
from .jl import Connection
from .scsi import NeedsAdmin, SCSIError, find_devices
from .sectormap import REGIONS, region_ranges, sector_map

PUSH_S = 0.06
SCAN_S = 1.5
MODES = ('backup', 'restore', 'firmware', 'bootloader')
STAGES = ('read', 'backup', 'write', 'verify')


def _t():
    return datetime.datetime.now().strftime('%H:%M:%S')


class Controller:
    def __init__(self, backup_dir, push=lambda state: None, dialogs=None,
                 find=find_devices, connect=Connection):
        self.backup_dir = backup_dir
        self.push = push
        self.dialogs = dialogs          # .open_file(title, kinds) -> path | None, .open_folder(path)
        self._find, self._connect = find, connect
        self._lock = threading.RLock()
        self._dirty = threading.Event()
        self._stop = threading.Event()
        self.session = None
        self.busy = False
        self.phase = 'waiting'          # waiting, connecting, ready, working, done, error
        self.message = ''
        self.mode = 'firmware'
        self.files = {'restore': None, 'firmware': None, 'stock': None, 'bootfile': None}
        self.options = {'wipe': False, 'with_boot': False, 'boot_source': 'open'}
        self.plan = None
        self.plan_json = None
        self.map = ['free'] * 256
        self.marks = {}                 # sector -> 'change' | 'written' | 'verified' | 'now'
        self.stage = {s: 'idle' for s in STAGES}
        self.pct = 0
        self.log_lines = []
        self.multiple = False
        self.alert = None               # {'kind': 'refused' | 'error', 'text'}: shown until dismissed
        self.cur = None                 # address being written or verified
        try:
            self.open_version = ops.bundled_bootloader(BOARDS[0])[1]
        except Exception:
            self.open_version = '?'

    # ------------------------------------------------------------ plumbing --

    def start(self):
        self.log(f'M-Flasher {__version__} started · open bootloader {self.open_version} bundled')
        threading.Thread(target=self._scan_loop, daemon=True).start()
        threading.Thread(target=self._push_loop, daemon=True).start()
        self.changed()

    def stop(self):
        self._stop.set()

    def changed(self):
        self._dirty.set()

    def _push_loop(self):
        while not self._stop.is_set():
            self._dirty.wait()
            self._dirty.clear()
            try:
                self.push(self.snapshot())
            except Exception:
                pass
            time.sleep(PUSH_S)

    def log(self, text):
        with self._lock:
            self.log_lines.append((_t(), text))
            del self.log_lines[:-200]
        self.changed()

    def _work(self, fn, name):
        """Run fn in a thread unless something is running."""
        with self._lock:
            if self.busy:
                return False
            self.busy = True
        self.changed()

        def run():
            try:
                fn()
            except ops.Refused as e:
                self._fail(f'Refused: {e}', 'refused')
            except NeedsAdmin as e:
                self._fail(str(e), 'error')
            except (SCSIError, OSError) as e:
                self._fail(f'USB error during {name}: {e}', 'error')
                self._drop()
            except Exception as e:
                self._fail(f'{name} failed: {type(e).__name__}: {e}', 'error')
            finally:
                with self._lock:
                    self.busy = False
                self.changed()
        threading.Thread(target=run, daemon=True).start()
        return True

    def _fail(self, msg, kind):
        self.log(msg)
        with self._lock:
            self.message = msg
            if self.phase in ('working', 'connecting'):
                self.phase = 'ready' if self.session else 'waiting'
            self.alert = {'kind': kind, 'text': msg}
        self.changed()

    # ---------------------------------------------------------- connection --

    def _scan_loop(self):
        while not self._stop.is_set():
            if not self.busy:
                try:
                    devs = self._find()
                except Exception:
                    devs = []
                paths = [p for p, _ in devs]
                if self.session and self.session.conn.path not in paths:
                    self.log('The unit is gone (unplugged or switched off)')
                    self._drop()
                elif not self.session and len(devs) == 1 and self.phase != 'connecting':
                    self._connect_to(devs[0][0])
                with self._lock:
                    multiple = not self.session and len(devs) > 1
                    if multiple != self.multiple:
                        self.multiple = multiple
                        self.changed()
            self._stop.wait(SCAN_S)

    def _drop(self):
        with self._lock:
            if self.session:
                try:
                    self.session.conn.close()
                except Exception:
                    pass
            self.session = None
            self.phase = 'waiting'
            self.plan = self.plan_json = None
            self.marks = {}
            self.map = ['free'] * 256
            self.stage = {s: 'idle' for s in STAGES}
            self.pct = 0
        self.changed()

    def _connect_to(self, path):
        def fn():
            with self._lock:
                self.phase = 'connecting'
                self.stage = dict(read='active', backup='idle', write='idle', verify='idle')
                self.pct = 0
                self.marks = {}
            self.log(f'Unit found in download mode: {path}')
            conn = self._connect(path, log=self.log)

            def progress(done, total):
                self.pct = int(100 * done / max(total, 1))
                self.changed()
            try:
                s = ops.Session(conn, self.backup_dir, log=self.log, progress=progress)
            except Exception:
                conn.close()
                with self._lock:
                    self.phase = 'waiting'
                    self.stage['read'] = 'idle'
                raise
            with self._lock:
                self.session = s
                self.phase = 'ready'
                self.stage['read'] = 'done'
                self.pct = 0
                self._refresh_map()
            info = s.info()
            self.log(f'{info["model"]} · {info["firmware"]} · bootloader {info["bootloader"]} · flash read twice, identical')
            if not s.analysis.certain:
                self.log('The model is not certain: check it under DEVICE INFO before writing')
            self._replan()
        self._work(fn, 'connecting')

    def _refresh_map(self):
        s = self.session
        self.map = sector_map(s.flash, s.conn.chip_key, s.board) if s else ['free'] * 256

    # ------------------------------------------------------------ planning --

    def _replan(self):
        """Plan the selected operation against the current flash (no writing)."""
        s = self.session
        with self._lock:
            self.plan, self.plan_json, self.alert = None, None, None
            self.marks = {}
            if self.phase == 'done':
                self.phase = 'ready'
        if not s:
            self.changed()
            return
        mode, f, o = self.mode, dict(self.files), dict(self.options)
        try:
            if mode == 'backup':
                pj = dict(kind='backup', title='Save a full backup', count=0, reads=256,
                          ranges='0x00000-0xFFFFF', notes=['Reads all 256 sectors (1 MiB); writes nothing to the unit.'],
                          warnings=[], ready=True, dest=self.backup_dir)
                with self._lock:
                    self.plan_json = pj
                self.changed()
                return
            if mode == 'restore':
                if not f['restore']:
                    return self._need('Choose a backup file')
                plan = s.plan_restore(self._read(f['restore']))
            elif mode == 'firmware':
                if not f['firmware']:
                    return self._need('Choose a firmware package (.fwsc)')
                plan = s.plan_firmware(self._read(f['firmware']), o['with_boot'], o['wipe'])
            else:
                src = o['boot_source']
                if src == 'open':
                    plan = s.plan_bootloader()
                elif src == 'stock':
                    if not f['stock']:
                        return self._need('Choose an official package (.fwsc) for its stock bootloader')
                    plan = s.plan_bootloader(raw_package=self._read(f['stock']))
                else:
                    if not f['bootfile']:
                        return self._need('Choose a bootloader file (uboot.boot)')
                    plan = s.plan_bootloader(boot=self._read(f['bootfile']))
        except ops.Refused as e:
            with self._lock:
                self.plan_json = dict(kind='refused', title='Refused', refused=str(e), ready=False,
                                      count=0, notes=[], warnings=[])
                self.alert = {'kind': 'refused', 'text': str(e)}
            self.log(f'Refused: {e}')
            self.changed()
            return
        except OSError as e:
            return self._need(f'Cannot read the file: {e}')
        runs = plan.runs()
        short = '–'
        if runs:
            short = f'0x{runs[0][0]:05X}-0x{runs[-1][1] - 1:05X}' + (f' IN {len(runs)} RUNS' if len(runs) > 1 else '')
        pj = dict(kind='write', title=plan.title, count=len(plan.sectors), kib=len(plan.sectors) * 4,
                  ranges=', '.join(f'0x{a:05X}-0x{b - 1:05X}' for a, b in runs) or '–', short=short,
                  notes=plan.notes, warnings=plan.warnings, ready=bool(plan.sectors))
        with self._lock:
            self.plan, self.plan_json = plan, pj
            self.marks = {a // SECTOR: 'change' for a in plan.sectors}
        if plan.sectors:
            self.log(f'Plan: {plan.title} · {len(plan.sectors)} sectors · {pj["ranges"]}')
        self.changed()

    def _need(self, what):
        with self._lock:
            self.plan_json = dict(kind='need', title=what, ready=False, count=0, notes=[], warnings=[])
        self.changed()

    @staticmethod
    def _read(path):
        with open(path, 'rb') as f:
            return f.read()

    def replan_async(self):
        if self.session and not self.busy:
            self._work(self._replan, 'planning')
        else:
            self.changed()

    # ----------------------------------------------------- window actions --

    def select_mode(self, mode):
        if mode in MODES and not self.busy:
            self.mode = mode
            self.replan_async()

    def choose_file(self, kind):
        kinds = {
            'restore': ('Choose a backup', [('Flash backups', '*.bin')], self.backup_dir),
            'firmware': ('Choose a firmware package', [('Firmware packages', '*.fwsc')], None),
            'stock': ('Choose an official package (its stock bootloader is used)', [('Firmware packages', '*.fwsc')], None),
            'bootfile': ('Choose a bootloader file', [('Bootloaders', '*.boot')], None),
        }
        if kind not in kinds or self.busy or not self.dialogs:
            return
        title, types, start = kinds[kind]
        path = self.dialogs.open_file(title, types, start)
        if path:
            self.files[kind] = path
            self.log(f'{os.path.basename(path)} chosen')
            self.replan_async()

    def pick_backup(self, path):
        if not self.busy and path and os.path.exists(path):
            self.files['restore'] = path
            self.mode = 'restore'
            self.replan_async()

    def set_option(self, name, value):
        if name in ('wipe', 'with_boot') and not self.busy:
            self.options[name] = bool(value)
            self.replan_async()

    def set_boot_source(self, src):
        if src in ('open', 'stock', 'file') and not self.busy:
            self.options['boot_source'] = src
            self.replan_async()

    def set_model(self, key):
        s = self.session
        if s and not self.busy:
            board = {b.key: b for b in BOARDS}.get(key)
            if board:
                s.board_override = board
                self.log(f'Model set to {board.name}')
                with self._lock:
                    self._refresh_map()
                self.replan_async()

    def start_action(self):
        """BACKUP saves at once; a write plan needs confirm() (after review)."""
        s = self.session
        if not s or self.busy:
            return
        if self.mode == 'backup':
            def fn():
                with self._lock:
                    self.phase = 'working'
                    self.stage.update(backup='active', write='idle', verify='idle')
                path = s.save_backup()
                with self._lock:
                    self.phase = 'done'
                    self.stage['backup'] = 'done'
                    self.message = f'Backup saved: {os.path.basename(path)}'
                    self.alert = {'kind': 'done', 'text': f'Backup saved:\n{path}'}
            self._work(fn, 'backup')

    def confirm(self):
        s, plan = self.session, self.plan
        if not s or not plan or not plan.sectors or self.busy:
            return
        total = len(plan.sectors)
        state = {'run': 0}

        def event(stage, addr=None):
            with self._lock:
                if stage == 'backup':
                    self.stage.update(backup='active')
                elif stage == 'erase':
                    self.stage.update(backup='done', write='active', verify='idle')
                    state['run'] = addr // SECTOR
                    self.marks[state['run']] = 'now'
                elif stage == 'write':
                    self.stage['write'] = 'active'
                    end = addr // SECTOR
                    for sec in range(state['run'], end):
                        if self.marks.get(sec) in ('change', 'now'):
                            self.marks[sec] = 'written'
                    if self.marks.get(end) == 'change':
                        self.marks[end] = 'now'
                elif stage == 'verify':
                    self.stage['verify'] = 'active'
                    for sec in range(state['run'], addr // SECTOR):
                        self.marks[sec] = 'verified'
                if stage == 'verify':           # the bar follows the current pass
                    done = sum(1 for m in self.marks.values() if m == 'verified')
                else:
                    done = sum(1 for m in self.marks.values() if m in ('written', 'verified'))
                self.pct = int(100 * done / total)
                self.cur = addr
            self.changed()

        def fn():
            with self._lock:
                self.phase = 'working'
                self.alert = None
                self.stage.update(backup='done' if s.backup_path else 'idle', write='idle', verify='idle')
                self.pct = 0
                self.cur = None
            s.apply(plan, event)
            with self._lock:
                self.phase = 'done'
                self.stage.update(backup='done', write='done', verify='done')
                self.pct = 100
                self.marks = {a // SECTOR: 'verified' for a in plan.sectors}
                self.message = f'{total} sectors written and verified'
                self._refresh_map()
                self.plan = None
                self.plan_json = dict(self.plan_json or {}, ready=False, done=True)
                self.alert = {'kind': 'done', 'text': f'{total} sectors written, read back and verified. '
                              'Unplug the unit and switch it off and on to start it, or choose another operation.'}
            self.log(f'Done: {total} sectors written, read back and verified. Switch the unit off and on to start it.')
        self._work(fn, 'writing')

    def dismiss(self):
        with self._lock:
            self.alert = None
            if self.phase == 'done':          # the finished operation's progress, cleared
                self.stage.update(write='idle', verify='idle')
                self.pct = 0
        if self.phase == 'done':
            self.replan_async()
        self.changed()

    def open_backups(self):
        os.makedirs(self.backup_dir, exist_ok=True)
        if self.dialogs:
            self.dialogs.open_folder(self.backup_dir)

    def clear_log(self):
        with self._lock:
            self.log_lines = []
        self.changed()

    # ------------------------------------------------------------- state --

    def recent_backups(self, n=6):
        out = []
        for path in sorted(glob.glob(os.path.join(self.backup_dir, '*.bin')), key=os.path.getmtime, reverse=True)[:n]:
            meta = {}
            try:
                with open(path[:-4] + '.json') as f:
                    meta = json.load(f)
            except (OSError, ValueError):
                pass
            out.append(dict(path=path, name=os.path.basename(path), firmware=meta.get('firmware', ''),
                            model=meta.get('model', ''),
                            time=datetime.datetime.fromtimestamp(os.path.getmtime(path)).strftime('%Y-%m-%d %H:%M')))
        return out

    def snapshot(self):
        with self._lock:
            s = self.session
            dev = None
            if s:
                info = s.info()
                a = s.analysis
                dev = dict(info, board=s.board.key if s.board else None, certain=bool(a.certain or s.board_override),
                           evidence=[t for _, _, t in a.evidence], path=s.conn.path,
                           flash_size='1 MiB', chip_key_ok=s.conn.chip_key == CHIP_KEY)
            return dict(
                version=__version__, phase=self.phase, busy=self.busy, multiple=self.multiple,
                message=self.message, alert=self.alert, device=dev,
                boards=[dict(key=b.key, name=b.name) for b in BOARDS],
                mode=self.mode, files={k: (v and os.path.basename(v)) for k, v in self.files.items()},
                options=dict(self.options), open_version=self.open_version,
                plan=self.plan_json, map=list(self.map), regions=REGIONS,
                ranges=region_ranges(self.map) if s else {},
                marks={str(k): v for k, v in self.marks.items()}, stage=dict(self.stage), pct=self.pct,
                cur=self.cur,
                log=[f'[{t}] {m}' for t, m in self.log_lines[-60:]],
                backups=self.recent_backups(), backup_dir=self.backup_dir,
                backup_saved=os.path.basename(s.backup_path) if s and s.backup_path else None)
