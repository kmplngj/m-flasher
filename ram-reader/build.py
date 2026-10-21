#!/usr/bin/env python3
"""Build only the offline FMR1 artifact with existing, pinned local tools.

SPDX-License-Identifier: Apache-2.0
No downloads, application execution, firmware installation or device APIs.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import struct
import subprocess
import sys

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
FLAGS = ['-target', 'pi32v2', '-mcpu=r3', '-Os', '-ffreestanding',
         '-fno-builtin', '-fno-common', '-fno-ident', '-fno-stack-protector',
         '-fno-unwind-tables', '-fno-asynchronous-unwind-tables',
         '-ffunction-sections', '-fdata-sections', '-Wall', '-Wextra', '-Werror']
INPUTS = ['include/fmr.h', 'src/reader.c', 'src/wl82.c', 'reader.ld']


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, **kwargs):
    return subprocess.run(args, check=True, timeout=240, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', default='target/fmr-reader/build')
    parser.add_argument('--toolchain', type=Path)
    opts = parser.parse_args()
    lock = json.loads((HERE / 'toolchain-lock.json').read_text())
    tc = (opts.toolchain or Path.home() / '.jieli' / lock['toolchain_revision']).resolve(strict=True)
    for name, expected in lock['tools'].items():
        path = tc / name
        if not path.is_file() or path.stat().st_size != expected['bytes'] or digest(path) != expected['sha256']:
            raise ValueError('pinned tool mismatch: ' + name)
    out = (REPO / opts.output).resolve()
    target = (REPO / 'target').resolve()
    if target not in out.parents or out == target or out.exists():
        raise ValueError('output must be a new directory below this repository target/')
    source_records = {}
    for name in INPUTS + ['build.py', 'toolchain-lock.json', 'verify_artifact.py']:
        path = HERE / name
        if not path.is_file() or path.is_symlink():
            raise ValueError('missing or symlinked source: ' + name)
        source_records[name] = digest(path)
    identity_inputs = {'sources': source_records, 'toolchain': lock['tools'],
                       'image': lock['container_image'], 'flags': FLAGS}
    fingerprint = hashlib.sha256(json.dumps(identity_inputs, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    build_id = int(fingerprint[:8], 16)
    # No pull or bootstrap. Inspect the already cached image, never devices.
    image = lock['container_image']
    meta = json.loads(subprocess.check_output(['rtk', 'proxy', 'docker', 'image', 'inspect', image], timeout=20))
    if len(meta) != 1 or meta[0]['Id'] != image:
        raise ValueError('cached image identity mismatch')
    out.mkdir(parents=True, mode=0o700)
    commands = []
    for name in ['reader', 'wl82']:
        commands.append(['/toolchain/common/bin/clang', *FLAGS, '-I/input/include',
                         '-DFMR_BUILD_ID=0x%08xu' % build_id,
                         '-c', '/input/src/' + name + '.c', '-o', '/output/' + name + '.o'])
    commands += [
        ['/toolchain/pi32v2/bin/ld', '--plugin', '/toolchain/pi32v2/bin/LLVMgold.so',
         '-o', '/output/fmr-reader.elf', '/output/reader.o', '/output/wl82.o',
         '-T', '/input/reader.ld', '--gc-sections', '-Map=/output/fmr-reader.map'],
        ['/toolchain/common/bin/objcopy', '-O', 'binary', '-j', '.text',
         '/output/fmr-reader.elf', '/output/fmr-text.bin'],
        ['/toolchain/common/bin/objcopy', '-O', 'binary', '-j', '.data',
         '/output/fmr-reader.elf', '/output/fmr-data.bin'],
    ]
    script = '\n'.join(shlex.join(c) for c in commands)
    script += '\n/toolchain/common/bin/objdump -d -triple=pi32v2 /output/fmr-reader.elf > /output/fmr-reader.dis\n'
    script += '/toolchain/common/bin/objdump -t /output/fmr-reader.elf > /output/fmr-reader.symbols\n'
    cidfile = out / 'container.cid'
    docker = ['rtk', 'proxy', 'docker', 'run', '--rm', '--cidfile=' + str(cidfile), '--pull=never', '--platform=linux/amd64',
              '--network=none', '--read-only', '--cap-drop=ALL', '--security-opt=no-new-privileges',
              '--user=%d:%d' % (os.getuid(), os.getgid()), '--pids-limit=64', '--memory=1g', '--cpus=2',
              '--tmpfs=/tmp:rw,noexec,nosuid,size=128m',
              '--mount=type=bind,src=%s,dst=/toolchain,readonly' % tc,
              '--mount=type=bind,src=%s,dst=/input,readonly' % HERE,
              '--mount=type=bind,src=%s,dst=/output' % out,
              '--entrypoint=/bin/sh', image, '-ec', script]
    try:
        with (out / 'build.stdout').open('wb') as stdout, (out / 'build.stderr').open('wb') as stderr:
            run(docker, stdout=stdout, stderr=stderr)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
        sys.stderr.write((out / 'build.stderr').read_text(errors='replace'))
        raise
    finally:
        # Stop only our exact build container if a parent timeout/interruption
        # prevented --rm cleanup. Never inspect or terminate unrelated work.
        if cidfile.is_file():
            cid = cidfile.read_text().strip()
            if len(cid) == 64 and all(c in '0123456789abcdef' for c in cid):
                subprocess.run(['rtk', 'proxy', 'docker', 'rm', '-f', cid],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=20)
            cidfile.unlink()
    # This vendor objcopy accepts exactly one -j. Preserve initialized .data
    # explicitly; dropping the entry-once marker would make a stale RAM value
    # control startup. Check each extracted extent against the linked ELF.
    elf = (out / 'fmr-reader.elf').read_bytes()
    header = struct.unpack_from('<16sHHIIIIIHHHHHH', elf)
    sections = [struct.unpack_from('<IIIIIIIIII', elf, header[6] + n * 40)
                for n in range(header[12])]
    strings = sections[header[13]]
    names = elf[strings[4]:strings[4] + strings[5]]
    selected = {}
    for section in sections:
        name = names[section[0]:].split(b'\0', 1)[0]
        if name in (b'.text', b'.data'):
            selected[name.decode()] = section
    text, data = (out/'fmr-text.bin').read_bytes(), (out/'fmr-data.bin').read_bytes()
    for name, body in [('.text', text), ('.data', data)]:
        section = selected[name]
        if len(body) != section[5] or body != elf[section[4]:section[4] + section[5]]:
            raise ValueError('objcopy/ELF section mismatch: ' + name)
    padding = selected['.data'][3] - 0x01c20000 - len(text)
    if selected['.text'][3] != 0x01c20000 or not 0 <= padding < 32768 or len(text) + padding + len(data) > 32768:
        raise ValueError('raw image packing geometry')
    with (out/'fmr-reader.bin').open('xb') as binary:
        binary.write(text + bytes(padding) + data)
    from verify_artifact import audit
    checked = audit(elf, (out/'fmr-reader.bin').read_bytes(), (out/'fmr-reader.dis').read_text())
    (out/'artifact-audit.json').write_text(json.dumps(checked, indent=2) + '\n')
    if any(digest(HERE/name) != before for name, before in source_records.items()):
        raise ValueError('source changed during build; discard artifact')
    receipt = {'schema': 1, 'kind': 'offline-candidate-not-hardware-qualified',
               'build_id': '%08x' % build_id, 'input_fingerprint': fingerprint,
               'source_sha256': source_records, 'toolchain': lock,
               'compiler_flags': FLAGS, 'commands': commands,
               'outputs': {p.name: {'bytes': p.stat().st_size, 'sha256': digest(p)}
                           for p in sorted(out.iterdir()) if p.is_file()},
               'hardware_access': False, 'target_executed_or_emulated': False,
               'network_access_in_build': False, 'flash_write': False,
               'runtime_profile_enabled': False}
    (out / 'build-receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({'output': str(out), 'build_id': receipt['build_id'],
                      'binary': receipt['outputs']['fmr-reader.bin']}))


if __name__ == '__main__':
    main()
