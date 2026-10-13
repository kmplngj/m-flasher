"""Command-line front end (the same operations as the window).

    python -m flasher info
    python -m flasher backup [DIR]
    python -m flasher restore BACKUP.bin
    python -m flasher firmware PACKAGE.fwsc [--with-bootloader] [--wipe-settings]
    python -m flasher bootloader [--stock PACKAGE.fwsc | --file UBOOT.boot]

Every command connects, reads the flash twice and shows what it found;
changes are shown and need --yes or a typed "yes".
"""
import argparse
import os
import sys

from . import __version__, ops
from .boards import BOARDS
from .jl import Connection
from .scsi import SCSIError, find_devices


def default_backup_dir():
    home = os.path.expanduser('~')
    docs = os.path.join(home, 'Documents')
    return os.path.join(docs if os.path.isdir(docs) else home, 'FM Flasher', 'backups')


def progress(done, total):
    pct = 100 * done // max(total, 1)
    sys.stderr.write(f'\r  {pct:3d}%')
    if done >= total:
        sys.stderr.write('\n')
    sys.stderr.flush()


def connect(args):
    if args.device:
        path = args.device
    else:
        devs = find_devices()
        if not devs:
            sys.exit('No unit in download mode found (a USB disk "WL82 UBOOT1.00").')
        if len(devs) > 1:
            sys.exit('More than one unit in download mode: connect only one, or pass --device.')
        path = devs[0][0]
    print(f'Connecting to {path}')
    conn = Connection(path, log=print)
    s = ops.Session(conn, args.backup_dir, log=print, progress=progress)
    if args.board:
        s.board_override = {b.key: b for b in BOARDS}[args.board]
    return s


def show(s):
    for k, v in s.info().items():
        print(f'  {k:11} {v}')
    for b, strong, text in s.analysis.evidence:
        print(f'    {"*" if strong else "-"} {text}')


def run(s, plan, yes):
    print()
    print(plan.describe())
    for w in plan.warnings:
        print('WARNING: ' + w)
    if not plan.sectors:
        return
    if not yes and input('Type "yes" to write: ').strip().lower() != 'yes':
        print('Nothing written.')
        return
    s.apply(plan)


def main(argv=None):
    ap = argparse.ArgumentParser(prog='flasher', description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--version', action='version', version=__version__)
    ap.add_argument('--device', help='device path (/dev/sgN, \\\\.\\PhysicalDriveN); found by itself otherwise')
    ap.add_argument('--board', choices=[b.key for b in BOARDS], help='the model, if it cannot be identified')
    ap.add_argument('--backup-dir', default=default_backup_dir(), help='where backups go (default: %(default)s)')
    ap.add_argument('--yes', action='store_true', help='do not ask before writing')
    sub = ap.add_subparsers(dest='cmd', required=True)
    sub.add_parser('find', help='list units in download mode (opens nothing)')
    sub.add_parser('info', help='connect and show what is installed')
    p = sub.add_parser('backup', help='save the whole flash')
    p.add_argument('dir', nargs='?')
    p = sub.add_parser('restore', help='write a full backup back')
    p.add_argument('file')
    p = sub.add_parser('firmware', help='install firmware from an M-VAVE .fwsc package')
    p.add_argument('package')
    p.add_argument('--with-bootloader', action='store_true', help='also put back the package\'s stock bootloader')
    p.add_argument('--wipe-settings', action='store_true', help='also erase the settings and Bluetooth pairings')
    p = sub.add_parser('bootloader', help='install the open bootloader (or another)')
    g = p.add_mutually_exclusive_group()
    g.add_argument('--stock', metavar='PACKAGE', help='put back the stock bootloader from this .fwsc')
    g.add_argument('--file', metavar='UBOOT', help='install this uboot.boot')
    args = ap.parse_args(argv)

    if args.cmd == 'find':
        for path, desc in find_devices():
            print(f'{path}\t{desc}')
        return
    try:
        s = connect(args)
    except ops.Refused as e:
        sys.exit(f'Refused: {e}')
    except SCSIError as e:
        sys.exit(str(e))
    try:
        show(s)
        if args.cmd == 'backup':
            s.save_backup(args.dir)
        elif args.cmd == 'restore':
            with open(args.file, 'rb') as f:
                run(s, s.plan_restore(f.read()), args.yes)
        elif args.cmd == 'firmware':
            with open(args.package, 'rb') as f:
                run(s, s.plan_firmware(f.read(), args.with_bootloader, args.wipe_settings), args.yes)
        elif args.cmd == 'bootloader':
            if args.stock:
                with open(args.stock, 'rb') as f:
                    plan = s.plan_bootloader(raw_package=f.read())
            elif args.file:
                with open(args.file, 'rb') as f:
                    plan = s.plan_bootloader(boot=f.read())
            else:
                plan = s.plan_bootloader()
            run(s, plan, args.yes)
    except ops.Refused as e:
        sys.exit(f'Refused: {e}')
    except SCSIError as e:
        sys.exit(f'USB error: {e}. Nothing more was written; check the cable and try again.')
    finally:
        s.conn.close()
