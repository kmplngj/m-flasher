#!/usr/bin/env python3
"""File-only FMR1 ELF/raw-image audit. SPDX-License-Identifier: Apache-2.0."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct

BASE, LIMIT = 0x01C20000, 0x01C28000


def audit(elf: bytes, image: bytes, listing: str | None = None) -> dict:
    def require(ok, reason):
        if not ok:
            raise ValueError(reason)

    def span(offset, size):
        require(0 <= offset <= len(elf) and 0 <= size <= len(elf) - offset, 'ELF extent outside file')
        return elf[offset:offset + size]

    def cstring(table, offset):
        require(0 <= offset < len(table), 'string-table offset')
        end = table.find(b'\0', offset)
        require(end >= 0, 'unterminated string')
        return table[offset:end].decode('ascii')

    require(len(elf) >= 52, 'truncated ELF header')
    h = struct.unpack_from('<16sHHIIIIIHHHHHH', elf)
    require(h[0][:7] == b'\x7fELF\x01\x01\x01' and (h[1], h[2], h[3]) == (2, 241, 1), 'wrong ELF kind/architecture')
    require(h[4] == BASE and h[8] == 52 and h[11] == 40 and h[12] > 0 and h[13] < h[12], 'wrong entry/section-header geometry')
    raw_sections = [struct.unpack('<IIIIIIIIII', span(h[6] + n * 40, 40)) for n in range(h[12])]
    names_header = raw_sections[h[13]]
    require(names_header[1] == 3, 'section names must be a string table')
    names = span(names_header[4], names_header[5])
    sections, by_index = {}, []
    for s in raw_sections:
        name = cstring(names, s[0])
        require(name not in sections or name == '', 'duplicate named section')
        d = dict(zip(('name_offset', 'type', 'flags', 'address', 'offset', 'size', 'link', 'info', 'alignment', 'entry_size'), s))
        d['name'] = name
        if s[1] != 8:
            span(s[4], s[5])
        sections[name] = d
        by_index.append(d)
    require(all(n in sections for n in ('.text', '.data', '.bss', '.symtab')), 'missing code/data/state/symbol sections')
    required_sections = {'.text': (1, 6, 2), '.data': (1, 3, 4), '.bss': (8, 3, 4)}
    for name, (kind, flags, alignment) in required_sections.items():
        section = sections[name]
        require((section['type'], section['flags'], section['alignment']) == (kind, flags, alignment),
                'wrong required section type/flags/alignment: ' + name)
        require(section['size'] > 0 and section['address'] % alignment == 0 and
                section['size'] % 4 == 0, 'invalid required section geometry: ' + name)
    require(sections['.text']['address'] == BASE, 'text must start at load base')
    require(sections['.data']['size'] == 4, 'initialized data must contain exactly the entry marker')
    require(sections['.data']['address'] == sections['.text']['address'] + sections['.text']['size'],
            'code/data must be contiguous')
    require(sections['.bss']['address'] == sections['.data']['address'] + sections['.data']['size'],
            'BSS must follow uploaded data')
    populated = []
    for name, s in sections.items():
        if not s['flags'] & 2 or not s['size']:
            continue
        require(name in ('.text', '.data', '.bss'), 'unexpected allocated section: ' + name)
        require(s['type'] == (8 if name == '.bss' else 1), 'unexpected allocated section type')
        require(BASE <= s['address'] < LIMIT and s['size'] <= LIMIT - s['address'], 'allocated extent exceeds reservation')
        require((s['type'] == 8) == (name == '.bss'), 'wrong loaded/NOBITS section kind')
        populated.append(s)
    populated.sort(key=lambda s: s['address'])
    for before, after in zip(populated, populated[1:]):
        require(before['address'] + before['size'] <= after['address'], 'allocated overlap')
    loaded = [s for s in populated if s['type'] != 8]
    end = max(s['address'] + s['size'] for s in loaded)
    require(0 < len(image) == end - BASE < LIMIT - BASE, 'raw image length does not include all loadable data')
    expected = bytearray(end - BASE)
    for s in loaded:
        start = s['address'] - BASE
        expected[start:start + s['size']] = span(s['offset'], s['size'])
    require(image == expected, 'raw image differs from ELF code/data or gap padding')

    table = sections['.symtab']
    require(table['type'] == 2, 'wrong symbol-table type')
    require(table['entry_size'] == 16 and table['size'] % 16 == 0 and table['link'] < len(by_index), 'symbol-table geometry')
    strings = by_index[table['link']]
    require(strings['type'] == 3, 'symbol names must be a string table')
    names = span(strings['offset'], strings['size'])
    symbols, functions = {}, []
    for i in range(table['size'] // 16):
        n, address, size, info, other, index = struct.unpack('<IIIBBH', span(table['offset'] + i * 16, 16))
        name = cstring(names, n)
        if not name:
            continue
        require(index != 0, 'undefined symbol: ' + name)
        require(not re.search(r'flash_(?:erase|program|unprotect|wren)|clock_up|chip_key|p33|display_|fmr_test_', name), 'forbidden/test symbol: ' + name)
        require(name not in symbols, 'duplicate symbol: ' + name)
        symbols[name] = {'address': address, 'size': size, 'section': index, 'type': info & 15}
        if info & 15 == 2:
            require(size > 0, 'empty function: ' + name)
            require(index < len(by_index) and by_index[index]['name'] == '.text', 'function outside text')
            require(BASE <= address and address % 2 == 0 and size % 2 == 0 and
                    address + size <= sections['.text']['address'] + sections['.text']['size'], 'function extent')
            functions.append((address, address + size, name))
    entry = symbols.get('loader_entry', {})
    require(entry.get('address') == BASE and entry.get('type') == 2 and entry.get('size', 0) > 0 and
            entry.get('section', len(by_index)) < len(by_index) and
            by_index[entry['section']]['name'] == '.text', 'entry must be a nonempty text function at load base')
    for name in ('__fmr_start', '__fmr_image_end', '__fmr_bss_start', '__fmr_bss_end', 'entry_marker'):
        require(name in symbols, 'missing geometry/entry-state symbol: ' + name)
    for name, section_name in (('__fmr_start', '.text'), ('__fmr_image_end', '.data'),
                               ('__fmr_bss_start', '.bss'), ('__fmr_bss_end', '.bss')):
        symbol = symbols[name]
        require(symbol['section'] < len(by_index) and by_index[symbol['section']]['name'] == section_name,
                'geometry symbol in wrong section: ' + name)
    require(symbols['__fmr_image_end']['address'] == end, 'image-end symbol mismatch')
    require(symbols['__fmr_start']['address'] == BASE, 'start symbol mismatch')
    require(symbols['__fmr_bss_start']['address'] == sections['.bss']['address'] and
            symbols['__fmr_bss_end']['address'] == sections['.bss']['address'] + sections['.bss']['size'],
            'BSS symbols must match section extent')
    marker = symbols['entry_marker']
    require(marker['section'] < len(by_index), 'entry marker section index')
    require(marker['size'] == 4 and by_index[marker['section']]['name'] == '.data', 'entry-once marker must be uploaded data')
    require(sections['.data']['address'] <= marker['address'] and
            marker['address'] + marker['size'] <= sections['.data']['address'] + sections['.data']['size'],
            'entry marker outside initialized data extent')
    require(image[marker['address'] - BASE:marker['address'] - BASE + 4] == struct.pack('<I', 0x46524d31), 'wrong initial entry marker')

    row_count, direct_calls = 0, []
    if listing is not None:
        row_re = re.compile(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s+(.*)$')
        last = None
        for line in listing.splitlines():
            match = row_re.match(line)
            if not match:
                continue
            pc, data, insn = int(match[1], 16), bytes.fromhex(match[2]), match[3]
            require(BASE <= pc and pc + len(data) <= end, 'listing address outside loaded image')
            require(image[pc - BASE:pc - BASE + len(data)] == data, 'listing/raw byte difference')
            if last is not None:
                require(last == pc, 'listing gap or overlap')
            else:
                require(pc == sections['.text']['address'], 'listing missing text prefix')
            last = pc + len(data)
            row_count += 1
            in_function = any(lo <= pc < hi for lo, hi, _ in functions)
            if in_function:
                require(not re.search(r'<unknown>|<invalid>|\bunknown\b|\(bad\)|^\s*\.(?:byte|short|word|long)\b', insn) and
                        '???' not in insn, 'unrecognized instruction within function')
            call = re.match(r'call\s+(-?\d+)\b', insn)
            if call and in_function:
                target = pc + len(data) + int(call[1])
                require(BASE <= target < sections['.text']['address'] + sections['.text']['size'], 'direct call outside local text')
                direct_calls.append({'site': hex(pc), 'target': hex(target)})
        require(row_count > 0 and last == sections['.text']['address'] + sections['.text']['size'], 'listing does not cover complete text')
    return {'elf_sha256': hashlib.sha256(elf).hexdigest(), 'binary_sha256': hashlib.sha256(image).hexdigest(),
            'image_bytes': len(image), 'entry': hex(BASE), 'image_end': hex(end),
            'bss_start': hex(symbols['__fmr_bss_start']['address']), 'bss_end': hex(symbols['__fmr_bss_end']['address']),
            'initialized_marker': hex(marker['address']), 'function_count': len(functions),
            'functions': {name: {'start': hex(lo), 'end': hex(hi)} for lo, hi, name in functions},
            'listing_rows_byte_matched': row_count, 'local_direct_calls': direct_calls,
            'undefined_symbols': [], 'unexpected_allocated_sections': [], 'source_rebuild_or_physical_proof': False,
            'hardware_access': False, 'target_execution_or_emulation': False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    opts = parser.parse_args()
    d = opts.directory
    result = audit((d / 'fmr-reader.elf').read_bytes(), (d / 'fmr-reader.bin').read_bytes(), (d / 'fmr-reader.dis').read_text())
    (d / 'artifact-audit.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
