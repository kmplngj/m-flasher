#!/usr/bin/env python3
"""File-only mutation tests for a supplied offline FMR1 build.

SPDX-License-Identifier: Apache-2.0
Requires Python 3.11+.
Run: python3 ram-reader/tests/test_artifact.py --artifact target/fmr-reader/build-4
Or set FMR_TEST_ARTIFACT before unittest discovery. No target code is executed.
These checks establish validator behavior, not source or physical certification.
"""
import argparse
import importlib.util
import os
from pathlib import Path
import re
import struct
import sys
import unittest

if sys.version_info < (3, 11):
    raise SystemExit('Artifact tests require Python 3.11 or newer; select that interpreter explicitly.')

READER = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location('fmr_artifact_validator', READER / 'verify_artifact.py')
_validator = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_validator)
ARTIFACT = None
ROW = re.compile(r'^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s+(.*)$')


class ArtifactMutationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        supplied = ARTIFACT or os.environ.get('FMR_TEST_ARTIFACT')
        if not supplied:
            raise RuntimeError('Supply --artifact <build-directory> or FMR_TEST_ARTIFACT; '
                               'for example target/fmr-reader/build-4. A real offline build is required.')
        directory = Path(supplied).expanduser().resolve()
        try:
            cls.elf = (directory / 'fmr-reader.elf').read_bytes()
            cls.image = (directory / 'fmr-reader.bin').read_bytes()
            cls.listing = (directory / 'fmr-reader.dis').read_text()
        except OSError as exc:
            raise RuntimeError('Missing FMR1 artifact files in %s; supply an existing offline '
                               'build containing fmr-reader.elf/.bin/.dis: %s' % (directory, exc)) from exc
        # Independent ELF field extraction for mutation locations. Do not reuse
        # the validator's section/symbol helpers or its audit receipt as an oracle.
        cls.sh_offset = struct.unpack_from('<I', cls.elf, 32)[0]
        count, names_index = struct.unpack_from('<HH', cls.elf, 48)
        headers = [struct.unpack_from('<10I', cls.elf, cls.sh_offset + 40 * i) for i in range(count)]
        names_header = headers[names_index]
        names = cls.elf[names_header[4]:names_header[4] + names_header[5]]
        cls.sections = {}
        for index, fields in enumerate(headers):
            name = names[fields[0]:].split(b'\0', 1)[0].decode('ascii')
            cls.sections[name] = (index, fields)
        sym_index, sym = cls.sections['.symtab']
        strings = headers[sym[6]]
        strings_data = cls.elf[strings[4]:strings[4] + strings[5]]
        cls.symbols = {}
        for offset in range(sym[4], sym[4] + sym[5], 16):
            fields = struct.unpack_from('<IIIBBH', cls.elf, offset)
            name = strings_data[fields[0]:].split(b'\0', 1)[0].decode('ascii')
            if name:
                cls.symbols[name] = (offset, fields, strings[4] + fields[0])
        cls.row_indexes = [i for i, line in enumerate(cls.listing.splitlines()) if ROW.match(line)]

    def reject(self, elf=None, image=None, listing=None, reason=None, omit_listing=False):
        with self.assertRaises(ValueError) as caught:
            _validator.audit(self.elf if elf is None else bytes(elf),
                             self.image if image is None else bytes(image),
                             None if omit_listing else (self.listing if listing is None else listing))
        if reason:
            self.assertIn(reason, str(caught.exception))

    def section_field(self, name, field, value):
        result = bytearray(self.elf)
        struct.pack_into('<I', result, self.sh_offset + self.sections[name][0] * 40 + field * 4, value)
        return result

    def symbol_field(self, name, field_offset, format_code, value):
        result = bytearray(self.elf)
        struct.pack_into('<' + format_code, result, self.symbols[name][0] + field_offset, value)
        return result

    def test_section_access_flags(self):
        # Correct file bytes with altered ELF permissions must not be accepted.
        for name, flags in (('.text', 3), ('.text', 7), ('.text', 2),
                            ('.data', 6), ('.data', 2), ('.bss', 6), ('.bss', 2)):
            with self.subTest(section=name, flags=flags):
                self.reject(elf=self.section_field(name, 2, flags))

    def test_text_must_start_at_entry_base(self):
        # Preserve a byte-exact image with a zero leading gap. This would satisfy
        # generic ELF/raw equality; the selected entry-first profile forbids it.
        text = self.sections['.text'][1]
        elf = bytearray(self.elf)
        base = self.sh_offset + self.sections['.text'][0] * 40
        struct.pack_into('<III', elf, base + 12, text[3] + 4, text[4] + 4, text[5] - 4)
        image = bytearray(self.image); image[:4] = bytes(4)
        self.reject(elf=elf, image=image, omit_listing=True)

    def test_bss_geometry_symbols_match_actual_section(self):
        for name, delta in (('__fmr_bss_start', 4), ('__fmr_bss_end', -4)):
            with self.subTest(symbol=name):
                value = self.symbols[name][1][1] + delta
                self.reject(elf=self.symbol_field(name, 4, 'I', value))

    def test_bss_geometry_symbols_belong_to_bss(self):
        for name in ('__fmr_bss_start', '__fmr_bss_end'):
            with self.subTest(symbol=name):
                self.reject(elf=self.symbol_field(name, 14, 'H', self.sections['.data'][0]))

    def test_load_geometry_symbols_belong_to_their_sections(self):
        for name in ('__fmr_start', '__fmr_image_end'):
            with self.subTest(symbol=name):
                self.reject(elf=self.symbol_field(name, 14, 'H', self.sections['.bss'][0]))

    def test_marker_extent_must_be_inside_data(self):
        # Keep the fresh marker magic present in matching ELF/raw bytes, but lie
        # about its section membership by relocating it into the end of .text.
        text = self.sections['.text'][1]
        address = text[3] + text[5] - 4
        elf = self.symbol_field('entry_marker', 4, 'I', address)
        image = bytearray(self.image)
        magic = struct.pack('<I', 0x46524d31)
        elf[text[4] + text[5] - 4:text[4] + text[5]] = magic
        image[address - 0x01c20000:address - 0x01c20000 + 4] = magic
        self.reject(elf=elf, image=image, omit_listing=True)

    def test_entry_is_nonempty_function(self):
        entry = self.symbols['loader_entry'][1]
        # STT_NOTYPE and STT_OBJECT cannot certify executable entry geometry.
        for kind in (0, 1):
            with self.subTest(kind=kind):
                self.reject(elf=self.symbol_field('loader_entry', 12, 'B', (entry[3] & 0xf0) | kind))
        self.reject(elf=self.symbol_field('loader_entry', 8, 'I', 0))

    def test_symbol_names_require_string_table_type(self):
        symbol_strings_index = self.sections['.symtab'][1][6]
        strings_name = next(name for name, (index, _) in self.sections.items() if index == symbol_strings_index)
        self.reject(elf=self.section_field(strings_name, 1, 1))

    def test_section_alignment_matches_profile(self):
        for name, alignment in (('.text', 4), ('.data', 2), ('.bss', 2)):
            with self.subTest(section=name):
                self.reject(elf=self.section_field(name, 8, alignment))

    def test_required_sections_are_not_empty(self):
        for name in ('.text', '.data', '.bss'):
            with self.subTest(section=name):
                self.reject(elf=self.section_field(name, 5, 0))

    def test_start_symbol_matches_load_base(self):
        self.reject(elf=self.symbol_field('__fmr_start', 4, 'I', 0x01c20004))

    def test_duplicate_named_symbol(self):
        elf = self.symbol_field('fmr_hw_read256', 0, 'I', self.symbols['loader_entry'][1][0])
        self.reject(elf=elf, reason='duplicate symbol')

    def test_nonentry_function_geometry(self):
        function = self.symbols['fmr_hw_read256'][1]
        for offset, value in ((4, function[1] + 1), (8, 0), (8, function[2] + 1)):
            with self.subTest(field=offset, value=value):
                self.reject(elf=self.symbol_field('fmr_hw_read256', offset, 'I', value))

    def test_undecoded_instruction_in_function(self):
        lines = self.listing.splitlines()
        at = self.row_indexes[0]; match = ROW.match(lines[at])
        for instruction in ('<unknown>', '<invalid>', '(bad)', '.byte 0x00',
                            '.short 0x0000', '.word 0x00000000', '.long 0', '???'):
            with self.subTest(instruction=instruction):
                changed = list(lines)
                changed[at] = '%x:    %s    %s' % (int(match[1], 16), match[2].strip(), instruction)
                self.reject(listing='\n'.join(changed), reason='unrecognized instruction within function')

    def test_baseline_passes_with_complete_listing(self):
        result = _validator.audit(self.elf, self.image, self.listing)
        self.assertEqual(result['entry'], '0x1c20000')
        self.assertEqual(result['image_bytes'], len(self.image))
        self.assertEqual(result['listing_rows_byte_matched'], len(self.row_indexes))
        self.assertGreater(result['function_count'], 0)
        self.assertFalse(result['source_rebuild_or_physical_proof'])
        self.assertFalse(result['hardware_access'])
        self.assertFalse(result['target_execution_or_emulation'])

    def test_truncated_elf(self):
        for length in (0, 16, 51):
            with self.subTest(length=length):
                self.reject(elf=self.elf[:length], reason='truncated ELF header')

    def test_wrong_architecture_and_elf_class(self):
        for offset, value in ((4, 2), (5, 2), (18, 0)):
            with self.subTest(offset=offset):
                elf = bytearray(self.elf); elf[offset] = value
                self.reject(elf=elf, reason='wrong ELF kind/architecture')

    def test_wrong_entry(self):
        elf = bytearray(self.elf)
        struct.pack_into('<I', elf, 24, 0x01c20004)
        self.reject(elf=elf, reason='wrong entry/section-header geometry')

    def test_section_headers_outside_file(self):
        elf = bytearray(self.elf)
        struct.pack_into('<I', elf, 32, len(elf) - 1)
        self.reject(elf=elf, reason='ELF extent outside file')

    def test_section_payload_outside_file(self):
        self.reject(elf=self.section_field('.text', 4, len(self.elf)), reason='ELF extent outside file')

    def test_drop_initialized_data_from_raw_image(self):
        text = self.sections['.text'][1]
        text_end = text[3] - 0x01c20000 + text[5]
        self.assertLess(text_end, len(self.image), 'fixture needs initialized data beyond code')
        self.reject(image=self.image[:text_end], reason='raw image length')

    def test_append_bss_to_raw_image(self):
        self.reject(image=self.image + bytes(self.sections['.bss'][1][5]), reason='raw image length')

    def test_raw_code_corruption(self):
        image = bytearray(self.image); image[0] ^= 1
        self.reject(image=image, reason='raw image differs')

    def test_zero_marker_in_both_elf_and_image(self):
        marker = self.symbols['entry_marker'][1]
        data = self.sections['.data'][1]
        marker_offset = marker[1] - data[3]
        elf, image = bytearray(self.elf), bytearray(self.image)
        elf[data[4] + marker_offset:data[4] + marker_offset + 4] = bytes(4)
        image[marker[1] - 0x01c20000:marker[1] - 0x01c20000 + 4] = bytes(4)
        self.reject(elf=elf, image=image, reason='wrong initial entry marker')

    def test_wrong_allocated_section_types(self):
        for name, kind in (('.text', 8), ('.data', 8), ('.bss', 1)):
            with self.subTest(section=name):
                self.reject(elf=self.section_field(name, 1, kind))

    def test_allocated_sections_overlap(self):
        self.reject(elf=self.section_field('.bss', 3, self.sections['.text'][1][3]))

    def test_allocated_section_outside_reservation(self):
        self.reject(elf=self.section_field('.bss', 3, 0x01c27ffc))

    def test_unexpected_allocated_section(self):
        sym = self.sections['.symtab'][1]
        self.reject(elf=self.section_field('.symtab', 2, sym[2] | 2), reason='unexpected allocated section')

    def test_undefined_entry_symbol(self):
        elf = bytearray(self.elf)
        struct.pack_into('<H', elf, self.symbols['loader_entry'][0] + 14, 0)
        self.reject(elf=elf, reason='undefined symbol: loader_entry')

    def test_test_hook_symbol_in_target(self):
        elf = bytearray(self.elf)
        offset = self.symbols['loader_entry'][2]
        self.assertEqual(len('loader_entry'), len('fmr_test_xxx'))
        elf[offset:offset + len('loader_entry')] = b'fmr_test_xxx'
        self.reject(elf=elf, reason='forbidden/test symbol: fmr_test_xxx')

    def test_wrong_symbol_table_record_size(self):
        self.reject(elf=self.section_field('.symtab', 9, 8), reason='symbol-table geometry')

    def test_listing_bytes_disagree(self):
        lines = self.listing.splitlines()
        at = self.row_indexes[0]; match = ROW.match(lines[at])
        changed = bytearray.fromhex(match[2]); changed[0] ^= 1
        lines[at] = '%x:    %s    %s' % (int(match[1], 16), changed.hex(' '), match[3])
        self.reject(listing='\n'.join(lines), reason='listing/raw byte difference')

    def test_listing_missing_middle_row(self):
        lines = self.listing.splitlines(); del lines[self.row_indexes[len(self.row_indexes) // 2]]
        self.reject(listing='\n'.join(lines), reason='listing gap or overlap')

    def test_listing_missing_prefix(self):
        lines = self.listing.splitlines(); del lines[self.row_indexes[0]]
        self.reject(listing='\n'.join(lines), reason='listing missing text prefix')

    def test_listing_missing_tail(self):
        lines = self.listing.splitlines(); del lines[self.row_indexes[-1]]
        self.reject(listing='\n'.join(lines), reason='listing does not cover complete text')

    def test_empty_listing(self):
        self.reject(listing='', reason='listing does not cover complete text')

    def test_listing_direct_call_outside_text(self):
        lines = self.listing.splitlines()
        for at in self.row_indexes:
            match = ROW.match(lines[at])
            if re.match(r'call\s+-?\d+\b', match[3]):
                lines[at] = '%x:    %s    call 99999999' % (int(match[1], 16), match[2].strip())
                break
        else:
            self.fail('fixture needs a decoded direct call')
        self.reject(listing='\n'.join(lines), reason='direct call outside local text')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__, add_help=False)
    parser.add_argument('--artifact', type=Path)
    opts, rest = parser.parse_known_args()
    ARTIFACT = opts.artifact
    unittest.main(argv=[sys.argv[0], *rest])
