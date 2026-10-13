"""The window: waits for a unit in download mode, connects, shows what is
installed and offers the operations. Work runs in a thread; the window
polls its messages."""
import os
import queue
import threading
import tkinter as tk
from tkinter import filedialog, messagebox, ttk

from . import __version__, ops
from .boards import BOARDS
from .cli import default_backup_dir
from .jl import Connection
from .scsi import NeedsAdmin, find_devices

HELP = (
    'Put the unit in download mode and connect it with a USB data cable:\n'
    '  • with the open bootloader: hold REC (SMK-37: SEQ REC) while switching on;\n'
    '  • Felucca, SLOOP, Snoop37: hold OCT- and OCT+ for 5 s while it runs;\n'
    '  • a unit whose firmware is damaged often goes there by itself.\n'
    'The screen stays as it is; the computer sees a USB disk "WL82 UBOOT1.00".\n'
    'If Windows offers to format a disk, choose Cancel.')
FIELDS = [('model', 'Model'), ('firmware', 'Firmware'), ('app', 'Firmware check'),
          ('bootloader', 'Bootloader'), ('flash', 'Flash chip'), ('chip key', 'Chip key')]


class App:
    def __init__(self, root):
        self.root = root
        self.q = queue.Queue()
        self.session = None
        self.busy = False
        root.title(f'FM Flasher {__version__}')
        root.minsize(660, 560)
        pad = dict(padx=10, pady=4)

        top = ttk.Frame(root)
        top.pack(fill='x', **pad)
        self.status = ttk.Label(top, text='Looking for a unit in download mode…', font=('TkDefaultFont', 11, 'bold'))
        self.status.pack(anchor='w')
        self.help = ttk.Label(top, text=HELP, justify='left', foreground='#444')
        self.help.pack(anchor='w', pady=(4, 0))

        info = ttk.LabelFrame(root, text='Unit')
        info.pack(fill='x', **pad)
        self.values = {}
        for i, (key, label) in enumerate(FIELDS):
            ttk.Label(info, text=label + ':').grid(row=i, column=0, sticky='w', padx=8, pady=1)
            v = ttk.Label(info, text='–')
            v.grid(row=i, column=1, sticky='w', padx=8, pady=1)
            self.values[key] = v
        ttk.Label(info, text='This is a:').grid(row=len(FIELDS), column=0, sticky='w', padx=8, pady=(6, 4))
        self.model = ttk.Combobox(info, state='disabled', width=24, values=[b.name for b in BOARDS])
        self.model.grid(row=len(FIELDS), column=1, sticky='w', padx=8, pady=(6, 4))
        self.model.bind('<<ComboboxSelected>>', self.on_model)

        acts = ttk.LabelFrame(root, text='Actions')
        acts.pack(fill='x', **pad)
        self.buttons = []

        def button(row, col, text, cmd):
            b = ttk.Button(acts, text=text, command=cmd, state='disabled')
            b.grid(row=row, column=col, sticky='we', padx=6, pady=4)
            self.buttons.append(b)
        button(0, 0, 'Back up the whole flash', self.on_backup)
        button(0, 1, 'Restore a backup…', self.on_restore)
        button(1, 0, 'Install firmware (.fwsc)…', self.on_firmware)
        button(1, 1, 'Install the open bootloader', self.on_bootloader)
        button(2, 1, 'Put back the stock bootloader…', self.on_stock_bootloader)
        self.wipe = tk.BooleanVar(value=False)
        self.with_boot = tk.BooleanVar(value=False)
        ttk.Checkbutton(acts, text='with firmware: also erase settings and Bluetooth pairings',
                        variable=self.wipe).grid(row=2, column=0, sticky='w', padx=6)
        ttk.Checkbutton(acts, text='with firmware: also put back its stock bootloader',
                        variable=self.with_boot).grid(row=3, column=0, sticky='w', padx=6)
        acts.columnconfigure(0, weight=1)
        acts.columnconfigure(1, weight=1)

        self.bar = ttk.Progressbar(root, maximum=1000)
        self.bar.pack(fill='x', **pad)
        logf = ttk.Frame(root)
        logf.pack(fill='both', expand=True, **pad)
        self.log_text = tk.Text(logf, height=9, wrap='word', state='disabled', relief='flat',
                                background='#f6f6f6')
        sb = ttk.Scrollbar(logf, command=self.log_text.yview)
        self.log_text.configure(yscrollcommand=sb.set)
        self.log_text.pack(side='left', fill='both', expand=True)
        sb.pack(side='right', fill='y')
        foot = ttk.Frame(root)
        foot.pack(fill='x', **pad)
        self.backup_dir = default_backup_dir()
        ttk.Label(foot, text=f'Backups: {self.backup_dir}', foreground='#666').pack(side='left')
        ttk.Button(foot, text='Open folder', command=self.open_backups).pack(side='right')

        self.root.after(100, self.pump)
        self.root.after(500, self.scan)

    # -- plumbing --

    def log(self, msg):
        self.q.put(('log', msg))

    def progress(self, done, total):
        self.q.put(('progress', done / max(total, 1)))

    def pump(self):
        try:
            while True:
                kind, val = self.q.get_nowait()
                if kind == 'log':
                    self.log_text.configure(state='normal')
                    self.log_text.insert('end', val + '\n')
                    self.log_text.see('end')
                    self.log_text.configure(state='disabled')
                elif kind == 'progress':
                    self.bar['value'] = 1000 * val
                elif kind == 'call':
                    val()
        except queue.Empty:
            pass
        self.root.after(80, self.pump)

    def work(self, fn, done=None):
        """Run fn in a thread; done(result) in the window afterwards."""
        self.set_busy(True)

        def run():
            try:
                result = fn()
            except ops.Refused as e:
                self.q.put(('call', lambda: self.fail('Refused', str(e))))
            except NeedsAdmin as e:
                self.q.put(('call', lambda: self.fail('Permission needed', str(e))))
            except Exception as e:      # the unit went away, a USB error, ...
                self.q.put(('call', lambda: self.fail('Error', f'{type(e).__name__}: {e}')))
            else:
                self.q.put(('call', lambda: done(result) if done else None))
            finally:
                self.q.put(('call', lambda: self.set_busy(False)))
        threading.Thread(target=run, daemon=True).start()

    def fail(self, title, msg):
        self.log(f'{title}: {msg}')
        messagebox.showerror(title, msg)

    def set_busy(self, busy):
        self.busy = busy
        state = 'disabled' if busy or not self.session else 'normal'
        for b in self.buttons:
            b.configure(state=state)
        self.model.configure(state='disabled' if busy or not self.session else 'readonly')
        if not busy:
            self.bar['value'] = 0

    # -- connecting --

    def scan(self):
        if not self.busy:
            devs = find_devices()
            if self.session and self.session.conn.path not in [p for p, _ in devs]:
                self.disconnected()
            elif not self.session and len(devs) == 1:
                self.connect(devs[0][0])
            elif not self.session and len(devs) > 1:
                self.status.configure(text='More than one unit in download mode: connect only one.')
        self.root.after(1500, self.scan)

    def disconnected(self):
        self.log('The unit is gone (unplugged or switched off)')
        try:
            self.session.conn.close()
        except Exception:
            pass
        self.session = None
        for v in self.values.values():
            v.configure(text='–')
        self.model.set('')
        self.status.configure(text='Looking for a unit in download mode…')
        self.help.configure(text=HELP)
        self.set_busy(False)

    def connect(self, path):
        self.status.configure(text=f'Connecting to {path}…')
        self.log(f'Found a unit in download mode: {path}')

        def fn():
            conn = Connection(path, log=self.log)
            try:
                return ops.Session(conn, self.backup_dir, log=self.log, progress=self.progress)
            except Exception:
                conn.close()
                raise
        self.work(fn, self.connected)

    def connected(self, session):
        self.session = session
        self.status.configure(text='Connected. Choose an action; do not unplug or switch off while it works.')
        self.help.configure(text='When you are done: unplug, switch the unit off and on again.')
        self.show_info()
        self.set_busy(False)

    def show_info(self):
        info = self.session.info()
        for key, label in FIELDS:
            self.values[key].configure(text=info.get(key, '–'))
        a = self.session.analysis
        board = self.session.board
        self.model.set(board.name if board else '')
        if not a.certain:
            reasons = '; '.join(t for _, _, t in a.evidence) or 'no evidence'
            self.log(f'The model is not certain ({reasons}): check "This is a" before writing.')

    def on_model(self, _):
        name = self.model.get()
        board = next(b for b in BOARDS if b.name == name)
        if board is not self.session.analysis.board:
            if not messagebox.askyesno('Model', f'Treat this unit as a {name}? Writing the wrong '
                                       'model\'s firmware or bootloader can make it unusable.'):
                self.model.set(self.session.board.name if self.session.board else '')
                return
        self.session.board_override = board
        self.log(f'Model set to {name}')

    # -- actions --

    def confirm_and_apply(self, plan):
        if not plan.sectors:
            messagebox.showinfo(plan.title, plan.describe())
            return
        text = plan.describe()
        if plan.warnings:
            text += '\n\n' + '\n'.join('⚠ ' + w for w in plan.warnings)
        text += '\n\nA backup of the unit is saved first. Write now?'
        if not messagebox.askokcancel(plan.title, text, icon='warning'):
            self.log('Cancelled: nothing written')
            return

        def done(_):
            self.show_info()
            messagebox.showinfo('Done', 'Written and checked.\n\nUnplug the unit and switch it off and '
                                'on again to start it (or do something else first).')
        self.work(lambda: self.session.apply(plan), done)

    def plan(self, fn):
        try:
            plan = fn()
        except ops.Refused as e:
            self.fail('Refused', str(e))
            return
        self.confirm_and_apply(plan)

    def on_backup(self):
        def done(path):
            messagebox.showinfo('Backup', f'Saved:\n{path}')
        self.work(lambda: self.session.save_backup(), done)

    def ask_file(self, title, types):
        return filedialog.askopenfilename(parent=self.root, title=title, filetypes=types)

    def on_restore(self):
        path = self.ask_file('Choose a backup', [('Flash backups', '*.bin'), ('All files', '*')])
        if path:
            with open(path, 'rb') as f:
                data = f.read()
            self.plan(lambda: self.session.plan_restore(data))

    def on_firmware(self):
        path = self.ask_file('Choose a firmware package', [('Firmware packages', '*.fwsc'), ('All files', '*')])
        if path:
            with open(path, 'rb') as f:
                data = f.read()
            self.plan(lambda: self.session.plan_firmware(data, self.with_boot.get(), self.wipe.get()))

    def on_bootloader(self):
        self.plan(lambda: self.session.plan_bootloader())

    def on_stock_bootloader(self):
        path = self.ask_file('Choose an official firmware package (its stock bootloader is used)',
                             [('Firmware packages', '*.fwsc'), ('All files', '*')])
        if path:
            with open(path, 'rb') as f:
                data = f.read()
            self.plan(lambda: self.session.plan_bootloader(raw_package=data))

    def open_backups(self):
        os.makedirs(self.backup_dir, exist_ok=True)
        if hasattr(os, 'startfile'):
            os.startfile(self.backup_dir)
        else:
            import subprocess
            subprocess.Popen(['xdg-open', self.backup_dir])


def main():
    root = tk.Tk()
    try:
        ttk.Style().theme_use('vista' if os.name == 'nt' else 'clam')
    except tk.TclError:
        pass
    App(root)
    root.mainloop()
