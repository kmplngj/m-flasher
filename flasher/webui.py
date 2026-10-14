"""The window: flasher/ui (HTML/CSS/JS) in a pywebview webview (Edge WebView2
on Windows, WebKitGTK on Linux), driven by the controller."""
import json
import os
import subprocess
import sys

import webview

from .cli import default_backup_dir
from .controller import Controller

UI = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'ui')


class Dialogs:
    def __init__(self):
        self.window = None

    def open_file(self, title, types, start=None):
        kinds = tuple(f'{name} ({pat})' for name, pat in types) + ('All files (*.*)',)
        result = self.window.create_file_dialog(webview.FileDialog.OPEN, directory=start or '',
                                                file_types=kinds)
        return result[0] if result else None

    def open_folder(self, path):
        if sys.platform == 'win32':
            os.startfile(path)
        else:
            subprocess.Popen(['xdg-open', path])


class Api:
    """Called from JavaScript (window.pywebview.api.*)."""

    def __init__(self, ctl):
        self._ctl = ctl

    def ready(self):
        return self._ctl.snapshot()

    def select_mode(self, mode):
        self._ctl.select_mode(mode)

    def choose_file(self, kind):
        self._ctl.choose_file(kind)

    def pick_backup(self, path):
        self._ctl.pick_backup(path)

    def set_option(self, name, value):
        self._ctl.set_option(name, value)

    def set_boot_source(self, src):
        self._ctl.set_boot_source(src)

    def set_model(self, key):
        self._ctl.set_model(key)

    def start_action(self):
        self._ctl.start_action()

    def confirm(self):
        self._ctl.confirm()

    def dismiss(self):
        self._ctl.dismiss()

    def open_backups(self):
        self._ctl.open_backups()

    def clear_log(self):
        self._ctl.clear_log()


def main():
    dialogs = Dialogs()
    ctl = Controller(default_backup_dir(), dialogs=dialogs)
    window = webview.create_window('M-Flasher', url=os.path.join(UI, 'index.html'), js_api=Api(ctl),
                                   width=1280, height=853, min_size=(900, 600),
                                   background_color='#28221f')
    dialogs.window = window

    def push(state):
        window.evaluate_js(f'window.mf && window.mf.update({json.dumps(state)})')
    ctl.push = push
    window.events.loaded += lambda: ctl.changed()
    webview.start(ctl.start, http_server=True)
    ctl.stop()
