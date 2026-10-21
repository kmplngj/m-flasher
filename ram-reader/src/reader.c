/* SPDX-License-Identifier: Apache-2.0
 * Copyright (c) 2026 Julius Vitkauskas (upstream ROM-interface pattern).
 * Modified 2026-10-16: minimal fail-closed read-only derivative; all writer,
 * fallback, optional feature and pipelining code removed. See ../NOTICE.
 */
#include "fmr.h"
#ifndef FMR_BUILD_ID
#error "FMR_BUILD_ID must bind this artifact's source/build inputs"
#endif
#ifdef FMR_HOST_TEST
#define FMR_SECTION(name)
#else
#define FMR_SECTION(name) __attribute__((section(name)))
#endif
#ifndef FMR_HOST_TEST
_Static_assert(sizeof(fmr_uptr) == 4, "target pointers must be 32-bit");
_Static_assert(sizeof(struct fmr_rom_io) == 12, "ROM ABI is exactly three words");
#endif
_Static_assert(sizeof(fmr_u32) == 4, "32-bit words required");

struct reader_state {
    struct fmr_rom_io io;
    fmr_uptr interface_address;
    fmr_u32 completed_dma;
    fmr_u8 jedec[3];
    volatile fmr_u8 status;
    volatile fmr_u8 busy;
};
static struct reader_state state;
static fmr_u8 read_buffer[FMR_READ_SIZE] __attribute__((aligned(4)));
/* Loadable initialized marker; no assumption about BSS before entry. */
static volatile fmr_u32 entry_marker FMR_SECTION(".data.entry_marker") = 0x46524d31u;

static int overlap(fmr_uptr a, fmr_u32 an, fmr_uptr b, fmr_u32 bn)
{
    return a <= b ? b - a < an : a - b < bn;
}
static int memory_allowed(const void *p, fmr_u32 n, fmr_u32 kind)
{
#ifdef FMR_HOST_TEST
    return fmr_test_memory_allowed(p, n, kind);
#else
    fmr_uptr a = (fmr_uptr)p;
    fmr_uptr lo = kind == FMR_MEMORY_SEND_ROM ? 0x01e00000ul : 0x01c00000ul;
    fmr_uptr hi = kind == FMR_MEMORY_SEND_ROM ? 0x01e20000ul : 0x01c80000ul;
    return a >= lo && a < hi && n <= hi - a;
#endif
}
static int external_memory(const void *p, fmr_u32 n, fmr_u32 alignment)
{
    fmr_uptr a = (fmr_uptr)p;
    if ((a & (alignment - 1u)) != 0u || !memory_allowed(p, n, FMR_MEMORY_SRAM)) {
        return 0;
    }
#ifdef FMR_HOST_TEST
    return !fmr_test_reserved_overlap(p, n);
#else
    return !overlap(a, n, FMR_LOAD_BASE, FMR_RESERVED_END - FMR_LOAD_BASE);
#endif
}
static void poison(fmr_u8 reason)
{
    /* Preserve the first failure, except a reentrant invocation is always visible. */
    if (state.status == FMR_STATUS_OK || reason == FMR_STATUS_REENTRANT) {
        state.status = reason;
    }
    fmr_hw_quiesce();
}
static fmr_u32 be32(const fmr_u8 *p)
{
    return (fmr_u32)p[0] << 24 | (fmr_u32)p[1] << 16 |
           (fmr_u32)p[2] << 8 | (fmr_u32)p[3];
}
static void put_be32(fmr_u8 *p, fmr_u32 v)
{
    p[0] = (fmr_u8)(v >> 24); p[1] = (fmr_u8)(v >> 16);
    p[2] = (fmr_u8)(v >> 8); p[3] = (fmr_u8)v;
}
static int command_pointers(fmr_u8 *cbw, fmr_u8 *answer)
{
    fmr_uptr c = (fmr_uptr)cbw, a = (fmr_uptr)answer;
    return external_memory(cbw, 31u, 4u) && external_memory(answer, 16u, 4u) &&
        !overlap(c, 31u, a, 16u) &&
        !overlap(c, 31u, state.interface_address, sizeof(struct fmr_rom_io)) &&
        !overlap(a, 16u, state.interface_address, sizeof(struct fmr_rom_io)) &&
        !overlap(c, 31u, (fmr_uptr)state.io.handler_slot, sizeof(fmr_handler)) &&
        !overlap(a, 16u, (fmr_uptr)state.io.handler_slot, sizeof(fmr_handler));
}
static int reader_handler(fmr_u8 *cbw, fmr_u8 *answer)
{
    const fmr_u8 *cmd;
    fmr_u32 op, want, address, i, reserved_from;
    if (state.busy) {
        poison(FMR_STATUS_REENTRANT);
        return 1;
    }
    state.busy = 1u;
    if (!command_pointers(cbw, answer)) {
        poison(FMR_STATUS_POINTER);
        goto done;
    }
    cmd = cbw + 15;
    op = (fmr_u32)cmd[0] << 8 | (fmr_u32)cmd[1];
    want = (fmr_u32)cbw[8] | (fmr_u32)cbw[9] << 8 |
           (fmr_u32)cbw[10] << 16 | (fmr_u32)cbw[11] << 24;
    if (cbw[0] != 'U' || cbw[1] != 'S' || cbw[2] != 'B' || cbw[3] != 'C' ||
        cbw[12] != 0x80u || cbw[13] != 0u || cbw[14] != 16u ||
        (op != 0xfc40u && op != 0xfc0bu && op != 0xfd05u) ||
        want != (op == 0xfd05u ? FMR_READ_SIZE : 16u)) {
        poison(FMR_STATUS_BAD_COMMAND);
        goto done;
    }
    reserved_from = op == 0xfd05u ? 8u : 2u;
    for (i = reserved_from; i < 16u; ++i) {
        if (cmd[i] != 0u) {
            poison(FMR_STATUS_BAD_COMMAND);
            goto done;
        }
    }
    if (op == 0xfd05u) {
        address = be32(cmd + 2);
        if (cmd[6] != 1u || cmd[7] != 0u || (address & 255u) != 0u ||
            address > FMR_CAPACITY - FMR_READ_SIZE) {
            poison(FMR_STATUS_BAD_COMMAND);
            goto done;
        }
        if (state.status != FMR_STATUS_OK) {
            goto done;
        }
        if (!fmr_hw_read256(address, read_buffer)) {
            poison(FMR_STATUS_READ_FAILED);
            goto done;
        }
        /* A nested invocation during HAL activity must not release stale bytes. */
        if (state.status != FMR_STATUS_OK) {
            goto done;
        }
        if (state.completed_dma != 0xffffffffu) {
            ++state.completed_dma;
        }
        /* No success interpretation is known for this opaque ROM callback. */
        (void)state.io.send(read_buffer, FMR_READ_SIZE);
        goto done;
    }
    for (i = 0; i < 16u; ++i) { answer[i] = 0u; }
    answer[0] = (fmr_u8)(op >> 8); answer[1] = (fmr_u8)op;
    if (op == 0xfc40u) {
        answer[2] = 'F'; answer[3] = 'M'; answer[4] = 'R'; answer[5] = '1';
        answer[6] = 1u; answer[7] = state.status;
        put_be32(answer + 8, (fmr_u32)FMR_BUILD_ID);
        put_be32(answer + 12, state.completed_dma);
    } else {
        answer[2] = state.jedec[0]; answer[3] = state.jedec[1];
        answer[4] = state.jedec[2]; answer[5] = state.status;
        if (state.jedec[0] == 0x85u && state.jedec[1] == 0x60u && state.jedec[2] == 0x14u) {
            put_be32(answer + 6, FMR_CAPACITY);
        }
        answer[10] = 1u; /* fixed read size BE16 = 256 */
    }
    (void)state.io.send(answer, 16u);
done:
    state.busy = 0u;
    return 1; /* ALL commands are consumed; never delegate to the ROM. */
}

FMR_SECTION(".entry") __attribute__((used))
int loader_entry(struct fmr_rom_io *rom)
{
    fmr_u32 i;
    if (entry_marker != 0x46524d31u) {
        entry_marker = 0x706f6973u;
        poison(FMR_STATUS_REENTRANT);
        return -1;
    }
    entry_marker = 0u;
    state.io.send = (fmr_transfer)0;
    state.io.recv = (fmr_transfer)0;
    state.io.handler_slot = (fmr_handler *)0;
    state.interface_address = 0u;
    state.completed_dma = 0u;
    state.jedec[0] = 0u; state.jedec[1] = 0u; state.jedec[2] = 0u;
    state.status = FMR_STATUS_OK;
    state.busy = 1u;
    for (i = 0; i < FMR_READ_SIZE; ++i) { read_buffer[i] = 0u; }
    if (!external_memory(rom, sizeof(*rom), sizeof(fmr_uptr))) {
        state.status = FMR_STATUS_POINTER;
        state.busy = 0u;
        return -1;
    }
    /* Exactly the three ABI words; no vendor's unrelated 32-byte debug read. */
    state.io.send = rom->send;
    state.io.recv = rom->recv;
    state.io.handler_slot = rom->handler_slot;
    state.interface_address = (fmr_uptr)rom;
    if (!external_memory(state.io.handler_slot, sizeof(fmr_handler), sizeof(fmr_uptr)) ||
        overlap((fmr_uptr)rom, sizeof(*rom), (fmr_uptr)state.io.handler_slot, sizeof(fmr_handler)) ||
        (((fmr_uptr)state.io.send & 3u) != 0u) ||
        !memory_allowed((const void *)state.io.send, 4u, FMR_MEMORY_SEND_ROM)) {
        state.status = FMR_STATUS_POINTER;
        state.busy = 0u;
        return -1;
    }
    /* A repeated entry during our initialization must survive the explicit BSS
     * assignments above. This does not qualify arbitrary concurrent ROM/IRQ use. */
    if (entry_marker != 0u) {
        poison(FMR_STATUS_REENTRANT);
        state.busy = 0u;
        return -1;
    }
    /* Install before peripheral setup, so init failure cannot expose old dispatch. */
    *state.io.handler_slot = reader_handler;
    if (!fmr_hw_init(state.jedec)) {
        poison(FMR_STATUS_INIT_FAILED);
    } else if (state.status == FMR_STATUS_OK &&
               (state.jedec[0] != 0x85u || state.jedec[1] != 0x60u || state.jedec[2] != 0x14u)) {
        state.status = FMR_STATUS_UNSUPPORTED;
    }
    fmr_hw_quiesce();
    state.busy = 0u;
    return 0;
}
#ifdef FMR_HOST_TEST
void fmr_test_reload_image(void)
{
    fmr_u32 i;
    volatile fmr_u8 *dirty = (volatile fmr_u8 *)&state;
    for (i = 0u; i < sizeof(state); ++i) { dirty[i] = 0xa5u; }
    for (i = 0u; i < FMR_READ_SIZE; ++i) { read_buffer[i] = 0xa5u; }
    entry_marker = 0x46524d31u;
}
#endif
