/* SPDX-License-Identifier: Apache-2.0
 * FM-1 minimal RAM reader. New interface; modified from the M-Flasher ROM ABI.
 * Copyright (c) 2026 Julius Vitkauskas (upstream ABI/SPI pattern).
 * See ../NOTICE and ../LICENSE.
 */
#ifndef FMR_H
#define FMR_H

typedef unsigned char fmr_u8;
typedef unsigned short fmr_u16;
typedef unsigned int fmr_u32;
typedef unsigned long fmr_uptr;
typedef int (*fmr_handler)(fmr_u8 *cbw, fmr_u8 *answer);
typedef int (*fmr_transfer)(void *buffer, fmr_u32 size);
struct fmr_rom_io {
    fmr_transfer send;
    fmr_transfer recv; /* Second ABI word, copied but NEVER invoked; hook slot is third. */
    fmr_handler *handler_slot;
};
#define FMR_READ_SIZE 256u
#define FMR_CAPACITY 0x100000u
#define FMR_LOAD_BASE 0x01c20000ul
#define FMR_RESERVED_END 0x01c28000ul
#define FMR_STATUS_OK 0u
#define FMR_STATUS_UNSUPPORTED 1u
#define FMR_STATUS_INIT_FAILED 2u
#define FMR_STATUS_BAD_COMMAND 3u
#define FMR_STATUS_READ_FAILED 4u
#define FMR_STATUS_REENTRANT 5u
#define FMR_STATUS_POINTER 6u
#define FMR_MEMORY_SRAM 1u
#define FMR_MEMORY_SEND_ROM 2u

/* HAL: 1 success / 0 failure. No retry. quiesce deasserts CS and disables ENC.
 * A successful read is exactly one complete 256-byte DMA; send's unknown return
 * semantics are deliberately ignored. completed counter counts DMA, not USB.
 */
int fmr_hw_init(fmr_u8 jedec[3]);
int fmr_hw_read256(fmr_u32 address, fmr_u8 out[FMR_READ_SIZE]);
void fmr_hw_quiesce(void);
int loader_entry(struct fmr_rom_io *rom);

#ifdef FMR_HOST_TEST
/* Dedicated hosted-only pointer seam. Production always applies numeric SRAM /
 * ROM bounds. Tests must whitelist exact allocated objects and function targets.
 * The reserved-overlap callback models the otherwise fixed target image range.
 */
int fmr_test_memory_allowed(const void *pointer, fmr_u32 length, fmr_u32 kind);
int fmr_test_reserved_overlap(const void *pointer, fmr_u32 length);
/* Model a fresh image with dirty BSS, then restore its entry-once marker. */
void fmr_test_reload_image(void);
#endif
#endif
