"""SCSI pass-through to the chip's USB download mode, which shows up as a
USB disk (vendor "WL82", product "UBOOT1.00", USB 4C4A:8057).

Linux: the SCSI generic driver (/dev/sgN, SG_IO). Windows: \\\\.\\PhysicalDriveN
with IOCTL_SCSI_PASS_THROUGH_DIRECT, which needs administrator rights.
Ported from jl-uboot-tool's scsiio (MIT, (c) 2023 Andrey Grigoryev).
"""
import ctypes
import os
import sys

VENDOR, PRODUCT = 'WL82', 'UBOOT1.00'
TIMEOUT_S = 10


class SCSIError(Exception):
    pass


class NeedsAdmin(SCSIError):
    pass


def _inquiry_strings(data):
    return (bytes(data[8:16]).decode('ascii', 'replace').strip(),
            bytes(data[16:32]).decode('ascii', 'replace').strip())


INQUIRY = b'\x12\x00\x00\x00\x24\x00'


class _Base:
    def __enter__(self):
        return self

    def __exit__(self, *a):
        self.close()

    def inquiry(self):
        data = bytearray(36)
        self.execute(INQUIRY, None, data)
        return _inquiry_strings(data)


# ---------------------------------------------------------------- Linux ---

if sys.platform.startswith('linux'):
    import fcntl

    class _sg_io_hdr(ctypes.Structure):
        _fields_ = [
            ('interface_id', ctypes.c_int), ('dxfer_direction', ctypes.c_int),
            ('cmd_len', ctypes.c_ubyte), ('mx_sb_len', ctypes.c_ubyte),
            ('iovec_count', ctypes.c_short), ('dxfer_len', ctypes.c_uint),
            ('dxferp', ctypes.c_void_p), ('cmdp', ctypes.c_void_p), ('sbp', ctypes.c_void_p),
            ('timeout', ctypes.c_uint), ('flags', ctypes.c_uint), ('pack_id', ctypes.c_int),
            ('usr_ptr', ctypes.c_void_p), ('status', ctypes.c_ubyte),
            ('masked_status', ctypes.c_ubyte), ('msg_status', ctypes.c_ubyte),
            ('sb_len_wr', ctypes.c_ubyte), ('host_status', ctypes.c_ushort),
            ('driver_status', ctypes.c_ushort), ('resid', ctypes.c_int),
            ('duration', ctypes.c_uint), ('info', ctypes.c_uint),
        ]

    SG_IO = 0x2285

    class SCSIDevice(_Base):
        def __init__(self, path):
            self.path = path
            try:
                self.fd = os.open(path, os.O_RDWR)
            except PermissionError as e:
                raise NeedsAdmin(f'no permission to open {path}: run as root, or install '
                                 'the udev rule from the README') from e

        def close(self):
            if self.fd is not None:
                os.close(self.fd)
                self.fd = None

        def execute(self, cdb, data_out, data_in):
            h = _sg_io_hdr()
            h.interface_id = ord('S')
            h.timeout = TIMEOUT_S * 1000
            sense = ctypes.create_string_buffer(32)
            h.mx_sb_len = 32
            h.sbp = ctypes.addressof(sense)
            cmd = ctypes.create_string_buffer(bytes(cdb), len(cdb))
            h.cmd_len = len(cdb)
            h.cmdp = ctypes.addressof(cmd)
            if data_out is not None:
                buf = ctypes.create_string_buffer(bytes(data_out), len(data_out))
                h.dxfer_direction, h.dxfer_len = -2, len(data_out)
                h.dxferp = ctypes.addressof(buf)
            elif data_in is not None:
                buf = ctypes.create_string_buffer(len(data_in))
                h.dxfer_direction, h.dxfer_len = -3, len(data_in)
                h.dxferp = ctypes.addressof(buf)
            else:
                h.dxfer_direction = -1
            try:
                fcntl.ioctl(self.fd, SG_IO, h)
            except OSError as e:
                raise SCSIError(f'SG_IO failed: {e}') from e
            if h.info & 1:
                raise SCSIError(f'command 0x{cdb[0]:02x}{cdb[1]:02x} failed: status 0x{h.status:02x}, '
                                f'host 0x{h.host_status:02x}, driver 0x{h.driver_status:02x}')
            if data_in is not None:
                data_in[:] = buf.raw[:len(data_in)]

    def find_devices():
        """[(path, description)] of the download-mode disks."""
        out = []
        base = '/sys/class/scsi_generic'
        if not os.path.isdir(base):
            return out
        for name in sorted(os.listdir(base)):
            dev = os.path.join(base, name, 'device')
            try:
                vendor = open(os.path.join(dev, 'vendor')).read().strip()
                model = open(os.path.join(dev, 'model')).read().strip()
            except OSError:
                continue
            if vendor == VENDOR and model == PRODUCT:
                out.append(('/dev/' + name, f'{vendor} {model} (/dev/{name})'))
        return out


# -------------------------------------------------------------- Windows ---

elif sys.platform == 'win32':
    from ctypes import wintypes

    _k32 = ctypes.WinDLL('kernel32', use_last_error=True)
    _k32.CreateFileW.restype = wintypes.HANDLE
    _k32.CreateFileW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD, wintypes.DWORD, ctypes.c_void_p,
                                 wintypes.DWORD, wintypes.DWORD, wintypes.HANDLE]
    _k32.DeviceIoControl.restype = wintypes.BOOL
    _k32.DeviceIoControl.argtypes = [wintypes.HANDLE, wintypes.DWORD, ctypes.c_void_p, wintypes.DWORD,
                                     ctypes.c_void_p, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD),
                                     ctypes.c_void_p]
    _k32.CloseHandle.argtypes = [wintypes.HANDLE]
    _INVALID = wintypes.HANDLE(-1).value

    GENERIC_READ, GENERIC_WRITE = 0x80000000, 0x40000000
    SHARE_RW, OPEN_EXISTING = 3, 3
    IOCTL_SCSI_PASS_THROUGH_DIRECT = 0x4D014
    IOCTL_STORAGE_QUERY_PROPERTY = 0x2D1400
    ERROR_ACCESS_DENIED = 5

    class _SPTD(ctypes.Structure):
        _fields_ = [
            ('Length', wintypes.USHORT), ('ScsiStatus', ctypes.c_ubyte), ('PathId', ctypes.c_ubyte),
            ('TargetId', ctypes.c_ubyte), ('Lun', ctypes.c_ubyte), ('CdbLength', ctypes.c_ubyte),
            ('SenseInfoLength', ctypes.c_ubyte), ('DataIn', ctypes.c_ubyte),
            ('DataTransferLength', wintypes.ULONG), ('TimeOutValue', wintypes.ULONG),
            ('DataBuffer', ctypes.c_void_p), ('SenseInfoOffset', wintypes.ULONG),
            ('Cdb', ctypes.c_ubyte * 16),
        ]

    class _SPTDWithSense(ctypes.Structure):
        _fields_ = [('sptd', _SPTD), ('sense', ctypes.c_ubyte * 32)]

    def _open(path, access):
        h = _k32.CreateFileW(path, access, SHARE_RW, None, OPEN_EXISTING, 0, None)
        if h == _INVALID or h is None:
            err = ctypes.get_last_error()
            if err == ERROR_ACCESS_DENIED:
                raise NeedsAdmin(f'access to {path} denied: run as administrator')
            raise SCSIError(f'cannot open {path}: {ctypes.FormatError(err)}')
        return h

    class SCSIDevice(_Base):
        def __init__(self, path):
            self.path = path
            self.h = _open(path, GENERIC_READ | GENERIC_WRITE)

        def close(self):
            if self.h is not None:
                _k32.CloseHandle(self.h)
                self.h = None

        def execute(self, cdb, data_out, data_in):
            s = _SPTDWithSense()
            p = s.sptd
            p.Length = ctypes.sizeof(_SPTD)
            p.TargetId = 1
            p.CdbLength = len(cdb)
            p.SenseInfoLength = 32
            p.SenseInfoOffset = _SPTDWithSense.sense.offset
            p.TimeOutValue = TIMEOUT_S
            for i, b in enumerate(cdb):
                p.Cdb[i] = b
            if data_out is not None:
                buf = ctypes.create_string_buffer(bytes(data_out), len(data_out))
                p.DataIn, p.DataTransferLength = 0, len(data_out)
            elif data_in is not None:
                buf = ctypes.create_string_buffer(len(data_in))
                p.DataIn, p.DataTransferLength = 1, len(data_in)
            else:
                buf = None
                p.DataIn = 2
            p.DataBuffer = ctypes.addressof(buf) if buf is not None else None
            got = wintypes.DWORD()
            ok = _k32.DeviceIoControl(self.h, IOCTL_SCSI_PASS_THROUGH_DIRECT, ctypes.byref(s),
                                      ctypes.sizeof(s), ctypes.byref(s), ctypes.sizeof(s),
                                      ctypes.byref(got), None)
            if not ok:
                raise SCSIError(f'command 0x{cdb[0]:02x}{cdb[1]:02x} failed: '
                                f'{ctypes.FormatError(ctypes.get_last_error())}')
            if p.ScsiStatus != 0:
                raise SCSIError(f'command 0x{cdb[0]:02x}{cdb[1]:02x} failed: status 0x{p.ScsiStatus:02x}')
            if data_in is not None:
                data_in[:] = buf.raw[:len(data_in)]

    def _query_ids(path):
        """(vendor, product) from the storage driver, without any SCSI command
        and without administrator rights; None if it cannot be asked."""
        try:
            h = _open(path, 0)
        except SCSIError:
            return None
        try:
            query = (ctypes.c_ubyte * 12)()          # StorageDeviceProperty, PropertyStandardQuery
            out = ctypes.create_string_buffer(1024)
            got = wintypes.DWORD()
            if not _k32.DeviceIoControl(h, IOCTL_STORAGE_QUERY_PROPERTY, query, 12, out, 1024,
                                        ctypes.byref(got), None):
                return None
            raw = out.raw[:got.value]

            def field(at):
                off = int.from_bytes(raw[at:at + 4], 'little')
                if not off or off >= len(raw):
                    return ''
                return raw[off:raw.index(b'\0', off)].decode('ascii', 'replace').strip()
            return field(12), field(16)
        finally:
            _k32.CloseHandle(h)

    def find_devices():
        out = []
        for n in range(64):
            path = rf'\\.\PhysicalDrive{n}'
            ids = _query_ids(path)
            if ids and ids[0] == VENDOR and ids[1] == PRODUCT:
                out.append((path, f'{ids[0]} {ids[1]} (disk {n})'))
        return out

else:
    def find_devices():
        return []

    class SCSIDevice(_Base):
        def __init__(self, path):
            raise SCSIError(f'{sys.platform} is not supported: use Windows or Linux')
