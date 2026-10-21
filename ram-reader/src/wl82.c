/* SPDX-License-Identifier: Apache-2.0
 * Copyright (c) 2026 Julius Vitkauskas.
 * SPI pattern and register layout from M-Flasher/fm1-bootloader (Apache-2.0).
 * Modified 2026-10-16: only fixed 9F/03 operations, bounded polling, one256
 * byte DMA buffer; no NOR write/status opcode, clock, display or P33 support.
 * See ../NOTICE and ../LICENSE. This is not physically qualified.
 */
#include "fmr.h"
#define BIT(n) (1u << (n))
#ifdef FMR_HOST_MMIO_TEST
#ifndef FMR_HOST_TEST
#error "Fake MMIO requires the explicit hosted test build"
#endif
extern fmr_u32 fmr_test_mmio_read(fmr_u32 address);
extern void fmr_test_mmio_write(fmr_u32 address, fmr_u32 value);
#define READ32(a) fmr_test_mmio_read(a)
#define WRITE32(a, v) fmr_test_mmio_write((a), (v))
#else
#define READ32(a) (*(volatile fmr_u32 *)(fmr_uptr)(a))
#define WRITE32(a, v) (*(volatile fmr_u32 *)(fmr_uptr)(a) = (v))
#endif
#define SPI_CON 0x00011c00ul
#define SPI_BAUD 0x00011c04ul
#define SPI_BUF 0x00011c08ul
#define SPI_ADR 0x00011c0cul
#define SPI_CNT 0x00011c10ul
#define PORTD_OUT 0x000500c0ul
#define PORTD_DIR 0x000500c8ul
#define PORTD_HD 0x000500dcul
#define IOMAP_CON0 0x0005101cul
#define IOMAP_CON1 0x00051020ul
#define SFC_CON 0x00040200ul
#define ENC_CON 0x00013000ul
/* One shared finite polling budget for the whole selected operation, including
 * byte transfers and DMA. Iterations are not a qualified wall-clock deadline. */
#define POLL_BUDGET 262144u

static void set_bits(fmr_u32 reg, fmr_u32 bits)
{
    WRITE32(reg, READ32(reg) | bits);
}
static void clear_bits(fmr_u32 reg, fmr_u32 bits)
{
    WRITE32(reg, READ32(reg) & ~bits);
}
static void cs(fmr_u32 high)
{
    fmr_u32 i;
    clear_bits(PORTD_DIR, BIT(0));
    if (high) { set_bits(PORTD_OUT, BIT(0)); } else { clear_bits(PORTD_OUT, BIT(0)); }
    for (i = 0; i < 40u; ++i) { __asm__ volatile("nop"); }
}
void fmr_hw_quiesce(void)
{
    /* Stop the controller before disabling ENC/deasserting chip select.
     * MMIO completion/cancellation semantics still require physical evidence. */
    clear_bits(SPI_CON, BIT(0));
    WRITE32(SPI_CNT, 0u);
    WRITE32(ENC_CON, 0u);
    cs(1u);
    set_bits(SPI_CON, BIT(14));
}
static int wait_pending(fmr_u32 *budget)
{
    while (*budget != 0u) {
        --*budget;
        if ((READ32(SPI_CON) & BIT(15)) != 0u) {
            set_bits(SPI_CON, BIT(14));
            return 1;
        }
    }
    fmr_hw_quiesce();
    return 0;
}
static int transmit(fmr_u8 byte, fmr_u32 *budget)
{
    clear_bits(SPI_CON, BIT(12));
    WRITE32(SPI_BUF, byte);
    return wait_pending(budget);
}
static int receive(fmr_u8 *byte, fmr_u32 *budget)
{
    set_bits(SPI_CON, BIT(12));
    WRITE32(SPI_BUF, 0xffu);
    if (!wait_pending(budget)) { return 0; }
    *byte = (fmr_u8)READ32(SPI_BUF);
    return 1;
}
int fmr_hw_init(fmr_u8 jedec[3])
{
    fmr_u32 budget = POLL_BUDGET;
    jedec[0] = 0u; jedec[1] = 0u; jedec[2] = 0u;
    WRITE32(SFC_CON, 0u);
    WRITE32(ENC_CON, 0u);
    WRITE32(SPI_CON, 0x28u);
    WRITE32(SPI_BAUD, 1u);
    clear_bits(IOMAP_CON1, BIT(1));
    clear_bits(IOMAP_CON0, BIT(5));
    clear_bits(PORTD_DIR, (BIT(4) | BIT(5)));
    set_bits(PORTD_HD, BIT(4));
    set_bits(PORTD_DIR, BIT(1));
    set_bits(SPI_CON, BIT(14));
    set_bits(SPI_CON, BIT(0));
    cs(1u);
    cs(0u);
    /* The only startup NOR command is JEDEC READ ID. */
    if (!transmit(0x9fu, &budget) || !receive(jedec, &budget) ||
        !receive(jedec + 1, &budget) || !receive(jedec + 2, &budget)) {
        fmr_hw_quiesce();
        return 0;
    }
    fmr_hw_quiesce();
    return 1;
}
int fmr_hw_read256(fmr_u32 address, fmr_u8 out[FMR_READ_SIZE])
{
    fmr_u32 budget = POLL_BUDGET;
    /* Defense in depth: core cannot request other sizes, opcodes or addresses. */
    if ((address & 255u) != 0u || address > FMR_CAPACITY - FMR_READ_SIZE ||
        (((fmr_uptr)out & 3u) != 0u)) {
        fmr_hw_quiesce();
        return 0;
    }
    set_bits(SPI_CON, BIT(14));
    set_bits(SPI_CON, BIT(0));
    cs(0u);
    if (!transmit(0x03u, &budget) || !transmit((fmr_u8)(address >> 16), &budget) ||
        !transmit((fmr_u8)(address >> 8), &budget) || !transmit((fmr_u8)address, &budget)) {
        fmr_hw_quiesce();
        return 0;
    }
    WRITE32(ENC_CON, 2u); /* Existing upstream SPI-path raw mode, decryption disabled. */
    set_bits(SPI_CON, BIT(12));
    clear_bits(SPI_CON, (BIT(10) | BIT(11)));
    WRITE32(SPI_ADR, (fmr_u32)(fmr_uptr)out);
    WRITE32(SPI_CNT, FMR_READ_SIZE);
    if (!wait_pending(&budget)) {
        fmr_hw_quiesce();
        return 0;
    }
    fmr_hw_quiesce();
    return 1;
}
