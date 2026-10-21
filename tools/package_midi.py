#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Package an existing host binary. --capabilities never initializes MIDI."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

root = Path(__file__).resolve().parents[1]
name = 'm-flasher.exe' if os.name == 'nt' else 'm-flasher'
build_name = 'm-flasher-midi.exe' if os.name == 'nt' else 'm-flasher-midi'
binary = root / 'target/release' / build_name
capabilities = json.loads(subprocess.check_output([str(binary), '--capabilities'], text=True))
if capabilities != {'schema': 1, 'midi_ota': True, 'protocol': 'felucca-device-pull-v1'}:
    raise SystemExit('unexpected MIDI capabilities')
destination = root / 'target/midi-dist'
destination.mkdir(parents=True, exist_ok=True)
shutil.copy2(binary, destination / name)
shutil.copy2(root / 'LICENSE', destination / 'LICENSE')
(destination / 'source.json').write_text(json.dumps({
    'repository': 'kmplngj/m-flasher', 'revision': os.environ['GITHUB_SHA'],
    'platform': os.environ['PACKAGE_PLATFORM'],
    'binary': name, 'sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
    'capabilities': capabilities,
}, indent=2) + '\n')
