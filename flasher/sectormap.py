"""The flash as 256 sectors of 4 KiB (a 16 x 16 map), each labelled with the
region it belongs to, from the unit's own file tables."""
from .boards import FM1, SMK37, KEY_MAC
from .formats import BOOT_AREA, SECTOR, app_table

REGIONS = ['boot', 'firmware', 'settings', 'pairings', 'calibration', 'presets', 'sounds', 'mac', 'free']
_NAMES = {'VM': 'settings', 'BTIF': 'pairings', 'USRTRIM': 'calibration', 'USRFLASH': 'presets', 'USR': 'sounds'}

# Stock layouts, for a unit whose firmware table cannot be read.
_STOCK = {
    'smk37': [('firmware', 0x04000, 0x9D000), ('settings', 0x9D000, 0xC0000), ('pairings', 0xC0000, 0xC1000),
              ('calibration', 0xC1000, 0xC2000), ('presets', 0xC2000, 0xEB000), ('sounds', 0xF4000, 0xFE000)],
    'fm1': [('firmware', 0x04000, 0x93000), ('settings', 0x93000, 0xE9000), ('pairings', 0xE9000, 0xEA000),
            ('sounds', 0xEA000, 0xFC000)],
}


def sector_map(flash, key, board=None):
    """[region name] * 256 for a 1 MiB image."""
    n = len(flash) // SECTOR
    out = ['free'] * n

    def mark(region, start, end):
        for s in range(max(start, 0) // SECTOR, min((end + SECTOR - 1) // SECTOR, n)):
            out[s] = region

    table = app_table(flash, key)
    files = [e for e in table if e.attr == 0x82]
    regions = [e for e in table if e.attr in (0x12, 0x92) and e.name in _NAMES]
    if files:
        end = BOOT_AREA + max(e.addr + e.len for e in files)
        for e in table:                 # PRCT: the code area the firmware protects
            if e.name == 'PRCT' and e.addr == 0 and e.len <= len(flash):
                end = max(end, e.len)
        mark('firmware', BOOT_AREA, end)
        for e in regions:
            if e.len and e.addr + e.len <= len(flash):
                mark(_NAMES[e.name], e.addr, e.addr + e.len)
    elif board is not None:
        for region, a, b in _STOCK[board.key]:
            mark(region, a, b)
    if board is SMK37:
        mark('calibration', 0xC1000, 0xC2000)       # factory data, whatever the firmware declares
    mark('boot', 0, BOOT_AREA)
    mark('mac', *KEY_MAC)
    return out


def region_ranges(smap):
    """{region: 'C2-EA, F4-FD'} (sector numbers in hex) for the legend."""
    out = {}
    s = 0
    while s < len(smap):
        e = s
        while e + 1 < len(smap) and smap[e + 1] == smap[s]:
            e += 1
        txt = f'{s:02X}' if s == e else f'{s:02X}-{e:02X}'
        out.setdefault(smap[s], []).append(txt)
        s = e + 1
    return {k: ', '.join(v) for k, v in out.items()}
