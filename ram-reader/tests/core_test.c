/* SPDX-License-Identifier: Apache-2.0
 * Hosted execution of our protocol core with exclusively local fakes.
 * Never link this harness with wl82.c or run a target image. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "fmr.h"
#include "golden_vectors.h"

#ifndef FMR_BUILD_ID
#error Build this test with the core's exact FMR_BUILD_ID.
#endif
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "%s:%d: %s\n", __FILE__, __LINE__, #x); exit(1); } } while (0)
struct guarded { uint32_t before; uint8_t bytes[64]; uint32_t after; };
static struct guarded command, answer;
static struct fmr_rom_io io;
static fmr_handler hook;
static unsigned init_calls, read_calls, send_calls, recv_calls, quiet_calls;
static int init_ok, read_ok, send_result, reenter_send, reenter_read, reenter_init;
static unsigned captured_size;
static uint8_t captured[256], chip[3];
static fmr_u32 last_address;
static const void *denied, *reserved;

static int contains(const void *base, size_t size, const void *p, size_t n)
{
    uintptr_t a = (uintptr_t)base, b = (uintptr_t)p;
    return b >= a && n <= size && b - a <= size - n;
}
static int fake_send(void *buffer, fmr_u32 size)
{
    CHECK(size == 16 || size == 256);
    CHECK(size <= sizeof captured);
    ++send_calls; captured_size = size; memcpy(captured, buffer, size);
    if (reenter_send == 2) { reenter_send = 0; CHECK(loader_entry(&io) != 0); }
    if (reenter_send) { reenter_send = 0; CHECK(hook(command.bytes, answer.bytes) == 1); }
    return send_result;
}
static int fake_recv(void *buffer, fmr_u32 size)
{ (void)buffer; (void)size; ++recv_calls; CHECK(0); return 0; }
int fmr_test_memory_allowed(const void *p, fmr_u32 n, fmr_u32 kind)
{
    if (p == denied) return 0;
    if (kind == FMR_MEMORY_SEND_ROM) return p == (const void *)fake_send;
    if (kind != FMR_MEMORY_SRAM) return 0;
    return contains(&io, sizeof io, p, n) || contains(&hook, sizeof hook, p, n)
        || contains(command.bytes, sizeof command.bytes, p, n)
        || contains(answer.bytes, sizeof answer.bytes, p, n);
}
int fmr_test_reserved_overlap(const void *p, fmr_u32 n)
{ return reserved && contains(p, n, reserved, 1); }
int fmr_hw_init(fmr_u8 id[3])
{
    ++init_calls; memcpy(id, chip, 3);
    if (reenter_init) { reenter_init = 0; CHECK(hook(command.bytes, answer.bytes) == 1); }
    return init_ok;
}
int fmr_hw_read256(fmr_u32 address, fmr_u8 out[256])
{
    unsigned i; ++read_calls; last_address = address;
    /* Fresh deterministic address-sensitive content, unlike zero-filled BSS. */
    for (i = 0; i < 256; ++i) out[i] = (uint8_t)(i ^ (address >> 8) ^ read_calls);
    if (reenter_read) { reenter_read = 0; CHECK(hook(command.bytes, answer.bytes) == 1); }
    return read_ok;
}
void fmr_hw_quiesce(void) { ++quiet_calls; }
static void canaries(void)
{
    CHECK(command.before == 0xdeadbeef && command.after == 0xcafebabe);
    CHECK(answer.before == 0xdeadbeef && answer.after == 0xcafebabe);
    CHECK(recv_calls == 0);
}
static void prepare(void)
{
    fmr_test_reload_image();
    memset(&command, 0xa7, sizeof command); memset(&answer, 0x6d, sizeof answer);
    command.before = answer.before = 0xdeadbeef;
    command.after = answer.after = 0xcafebabe;
    hook = NULL; io.send = fake_send; io.recv = fake_recv; io.handler_slot = &hook;
    chip[0] = 0x85; chip[1] = 0x60; chip[2] = 0x14;
    init_calls = read_calls = send_calls = recv_calls = quiet_calls = captured_size = 0;
    init_ok = read_ok = 1; send_result = -99; reenter_send = reenter_read = reenter_init = 0;
    denied = reserved = NULL; memset(captured, 0xe5, sizeof captured);
}
static void start(void)
{ prepare(); CHECK(loader_entry(&io) == 0); CHECK(hook != NULL); CHECK(init_calls == 1); }
static void dispatch(const uint8_t cbw[31])
{ memcpy(command.bytes, cbw, 31); CHECK(hook(command.bytes, answer.bytes) == 1); canaries(); }
static void identity(unsigned status, unsigned completed)
{
    static const uint8_t prefix[8] = {0xfc,0x40,'F','M','R','1',1,0};
    dispatch(golden_identity_cbw); CHECK(captured_size == 16);
    CHECK(memcmp(captured, prefix, 7) == 0); CHECK(captured[7] == status);
    CHECK(captured[8] == ((FMR_BUILD_ID >> 24) & 255));
    CHECK(captured[9] == ((FMR_BUILD_ID >> 16) & 255));
    CHECK(captured[10] == ((FMR_BUILD_ID >> 8) & 255));
    CHECK(captured[11] == (FMR_BUILD_ID & 255));
    CHECK(captured[12] == 0 && captured[13] == 0);
    CHECK(captured[14] == (completed >> 8) && captured[15] == (completed & 255));
}
static void rejected(uint8_t cbw[31])
{
    unsigned before;
    start(); dispatch(cbw); CHECK(send_calls == 0 && read_calls == 0);
    before = send_calls; dispatch(golden_first_read_cbw);
    CHECK(send_calls == before && read_calls == 0);
    identity(FMR_STATUS_BAD_COMMAND, 0);
    dispatch(golden_first_read_cbw); CHECK(read_calls == 0); canaries();
}
static void protocol_vectors(void)
{
    unsigned i;
    start(); identity(0, 0); CHECK(read_calls == 0);
    dispatch(golden_jedec_cbw); CHECK(captured_size == 16);
    CHECK(memcmp(captured, golden_jedec_reply, 16) == 0);
    dispatch(golden_first_read_cbw); CHECK(read_calls == 1 && last_address == 0);
    CHECK(captured_size == 256);
    for (i = 0; i < 256; ++i) CHECK(captured[i] == (uint8_t)(i ^ 1));
    identity(0, 1);
    dispatch(golden_last_read_cbw); CHECK(last_address == 0xfff00 && read_calls == 2);
    for (i = 0; i < 256; ++i) CHECK(captured[i] == (uint8_t)(i ^ 0xfff ^ 2));
    identity(0, 2); CHECK(quiet_calls >= 1);
    /* Unknown send result must never be reported as wire/CSW completion. */
    send_result = 0; dispatch(golden_first_read_cbw); identity(0, 3);
}
static void malformed(void)
{
    uint8_t b[31]; unsigned i, j;
    static const unsigned header_offsets[] = {0,1,2,3,8,9,10,11,12,13,14};
    for (i = 0; i < sizeof header_offsets / sizeof *header_offsets; ++i) {
        memcpy(b, golden_identity_cbw, 31); b[header_offsets[i]] ^= 1; rejected(b);
    }
    for (i = 0; i < 256; ++i) {
        if (i != 0x80) { memcpy(b, golden_identity_cbw, 31); b[12] = (uint8_t)i; rejected(b); }
        if (i != 16) { memcpy(b, golden_identity_cbw, 31); b[14] = (uint8_t)i; rejected(b); }
    }
    for (j = 0; j < 3; ++j) {
        const uint8_t *v = j == 0 ? golden_identity_cbw : j == 1 ? golden_jedec_cbw : golden_first_read_cbw;
        for (i = j == 2 ? 23 : 17; i < 31; ++i) {
            memcpy(b, v, 31); b[i] = 1; rejected(b);
        }
    }
    /* CBW tags are opaque, including all ones; do not reject valid tags. */
    start(); memcpy(b, golden_identity_cbw, 31); memset(b + 4, 255, 4); dispatch(b); CHECK(send_calls == 1);
}
static void read_bounds(void)
{
    static const uint8_t addresses[][4] = {
        {0,0,0,1}, {0,0,0,255}, {0,0x10,0,0}, {0xff,0xff,0xff,0}, {0xff,0xff,0xff,0xff}
    };
    uint8_t b[31]; unsigned i;
    for (i = 0; i < sizeof addresses / sizeof *addresses; ++i) {
        memcpy(b, golden_first_read_cbw, 31); memcpy(b + 17, addresses[i], 4); rejected(b);
    }
    memcpy(b, golden_first_read_cbw, 31); b[21] = b[22] = 0; rejected(b);
    memcpy(b, golden_first_read_cbw, 31); b[21] = 2; rejected(b);
    memcpy(b, golden_first_read_cbw, 31); b[22] = 1; rejected(b);
    memcpy(b, golden_first_read_cbw, 31); b[9] = 2; rejected(b);
}
static void opcode_allowlist(void)
{
    unsigned op; uint8_t b[31];
    for (op = 0; op <= 65535; ++op) {
        if (op == 0xfc40 || op == 0xfc0b || op == 0xfd05) continue;
        start(); memcpy(b, golden_identity_cbw, 31); b[15] = (uint8_t)(op >> 8); b[16] = (uint8_t)op;
        dispatch(b); CHECK(send_calls == 0 && read_calls == 0);
        dispatch(golden_first_read_cbw); CHECK(send_calls == 0 && read_calls == 0);
    }
}
static void failures(void)
{
    unsigned before;
    prepare(); init_ok = 0; CHECK(loader_entry(&io) == 0); CHECK(hook != NULL);
    identity(FMR_STATUS_INIT_FAILED, 0); before = send_calls;
    dispatch(golden_first_read_cbw); CHECK(read_calls == 0 && send_calls == before);
    prepare(); chip[2] = 0x15; CHECK(loader_entry(&io) == 0);
    identity(FMR_STATUS_UNSUPPORTED, 0); dispatch(golden_jedec_cbw);
    CHECK(captured[5] == 1); CHECK(captured[6] == 0 && captured[7] == 0 && captured[8] == 0 && captured[9] == 0);
    before = send_calls; dispatch(golden_first_read_cbw); CHECK(read_calls == 0 && send_calls == before);
    start(); dispatch(golden_first_read_cbw); read_ok = 0; before = send_calls;
    dispatch(golden_first_read_cbw); CHECK(read_calls == 2 && send_calls == before);
    identity(FMR_STATUS_READ_FAILED, 1); read_ok = 1;
    dispatch(golden_first_read_cbw); CHECK(read_calls == 2);
    start(); reenter_read = 1; dispatch(golden_first_read_cbw);
    CHECK(read_calls == 1 && send_calls == 0); identity(FMR_STATUS_REENTRANT, 0);
    start(); reenter_send = 1; dispatch(golden_first_read_cbw);
    CHECK(read_calls == 1 && send_calls == 1); identity(FMR_STATUS_REENTRANT, 1);
    dispatch(golden_first_read_cbw); CHECK(read_calls == 1);
    start(); reenter_send = 2; dispatch(golden_first_read_cbw);
    CHECK(init_calls == 1 && read_calls == 1); identity(FMR_STATUS_REENTRANT, 1);
    dispatch(golden_first_read_cbw); CHECK(read_calls == 1);
    prepare(); reenter_init = 1; CHECK(loader_entry(&io) == 0);
    identity(FMR_STATUS_REENTRANT, 0); dispatch(golden_first_read_cbw); CHECK(read_calls == 0);
}
static void unknown_chips(void)
{
    unsigned byte, value;
    static const uint8_t expected[3] = {0x85,0x60,0x14};
    for (byte = 0; byte < 3; ++byte) for (value = 0; value < 256; ++value) {
        if (value == expected[byte]) continue;
        prepare(); chip[byte] = (uint8_t)value; CHECK(loader_entry(&io) == 0);
        dispatch(golden_jedec_cbw); CHECK(memcmp(captured + 2, chip, 3) == 0);
        CHECK(captured[5] == FMR_STATUS_UNSUPPORTED);
        CHECK(captured[6] == 0 && captured[7] == 0 && captured[8] == 0 && captured[9] == 0);
        dispatch(golden_first_read_cbw); CHECK(read_calls == 0 && send_calls == 1);
    }
}
static void pointer_and_entry(void)
{
    unsigned before;
    prepare(); CHECK(loader_entry(NULL) != 0); CHECK(init_calls == 0 && hook == NULL);
    prepare(); io.send = NULL; CHECK(loader_entry(&io) != 0); CHECK(init_calls == 0 && hook == NULL);
    prepare(); io.handler_slot = NULL; CHECK(loader_entry(&io) != 0); CHECK(init_calls == 0 && hook == NULL);
    prepare(); denied = &io; CHECK(loader_entry(&io) != 0); CHECK(init_calls == 0);
    prepare(); reserved = &hook; CHECK(loader_entry(&io) != 0); CHECK(init_calls == 0);
    prepare(); CHECK(loader_entry((struct fmr_rom_io *)((uint8_t *)&io + 1)) != 0); CHECK(init_calls == 0);
    prepare(); io.handler_slot = (fmr_handler *)&io; CHECK(loader_entry(&io) != 0); CHECK(init_calls == 0);
    prepare(); denied = (const void *)fake_send; CHECK(loader_entry(&io) != 0); CHECK(init_calls == 0);
    start(); before = init_calls; CHECK(loader_entry(&io) != 0); CHECK(init_calls == before);
    start(); memcpy(command.bytes, golden_first_read_cbw, 31);
    CHECK(hook(NULL, answer.bytes) == 1); CHECK(read_calls == 0); identity(FMR_STATUS_POINTER, 0);
    start(); memcpy(command.bytes, golden_first_read_cbw, 31);
    CHECK(hook(command.bytes, NULL) == 1); CHECK(read_calls == 0); identity(FMR_STATUS_POINTER, 0);
    start(); memcpy(command.bytes + 1, golden_first_read_cbw, 31);
    CHECK(hook(command.bytes + 1, answer.bytes) == 1); CHECK(read_calls == 0); identity(FMR_STATUS_POINTER, 0);
    start(); memcpy(command.bytes, golden_first_read_cbw, 31);
    CHECK(hook(command.bytes, answer.bytes + 1) == 1); CHECK(read_calls == 0); identity(FMR_STATUS_POINTER, 0);
    start(); memcpy(command.bytes, golden_first_read_cbw, 31);
    CHECK(hook(command.bytes, command.bytes) == 1); CHECK(read_calls == 0); identity(FMR_STATUS_POINTER, 0);
    start(); reserved = command.bytes; dispatch(golden_first_read_cbw); CHECK(read_calls == 0 && send_calls == 0);
    reserved = NULL; identity(FMR_STATUS_POINTER, 0);
    canaries();
}
int main(void)
{
    protocol_vectors(); malformed(); read_bounds(); opcode_allowlist(); failures(); unknown_chips(); pointer_and_entry();
    puts("M1 owned core: golden vectors, malformed/bounds/fault/pointer/reentrancy checks and 65,533 forbidden opcodes passed (host fakes only)");
    return 0;
}
