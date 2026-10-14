/*
 * AC791N (WL82) peripheral registers used by the bootloader. Addresses and
 * names follow the SDK's include_lib/driver/cpu/wl82/asm/WL82.h; only the
 * registers and fields the bootloader touches are listed.
 */
#ifndef REGS_H
#define REGS_H

#include "types.h"

/* GPIO ports (WL82.h JL_PORT_FLASH_TypeDef, first fields). */
typedef struct {
    volatile u32 OUT, IN, DIR, DIE, PU, PD, HD0, HD;
} jl_port_t;
#define JL_PORTA ((jl_port_t *)0x50000)
#define JL_PORTB ((jl_port_t *)0x50040)
#define JL_PORTC ((jl_port_t *)0x50080)
#define JL_PORTD ((jl_port_t *)0x500c0)
#define JL_PORTE ((jl_port_t *)0x50100)
#define JL_PORTF ((jl_port_t *)0x50140)
#define JL_PORTG ((jl_port_t *)0x50180)
#define JL_PORTH ((jl_port_t *)0x501c0)

/* SPI controllers: CON bit0 enable, bit3 separate DI/DO, bits 11:10 width
 * (0 1-bit, 1 dual, 2 quad), bit12 receive, bit14 clear pending, bit15 pending. */
typedef struct {
    volatile u32 CON, BAUD, BUF, ADR, CNT;
} jl_spi_t;
#define JL_SPI0 ((jl_spi_t *)0x11c00)
#define JL_SPI1 ((jl_spi_t *)0x11d00) /* display */

typedef struct {
    volatile u32 FIFO, REG;
} jl_crc_t;
#define JL_CRC ((jl_crc_t *)0x13500)

typedef struct {
    volatile u32 CON, CNT, PRD, PWM;
} jl_timer_t;
#define JL_TIMER0 ((jl_timer_t *)0x10400)

/* PMU-domain (P33) serial interface. */
typedef struct {
    volatile u32 PMU_CON, RTC_CON, SPI_CON, SPI_DAT;
} jl_p33_t;
#define JL_P33 ((jl_p33_t *)0x13e00)

/* Clock and power control. PWR_CON bit4 = soft reset (SDK CORE_SYSTEM_RESET). */
#define JL_PWR_CON (*(volatile u32 *)0x10000)
#define JL_SYS_DIV (*(volatile u32 *)0x10008) /* bits 17:16 HSB div - 1, 10:8 LSB div - 1 */
#define JL_CLK_CON0 (*(volatile u32 *)0x1000c)
#define JL_CLK_CON2 (*(volatile u32 *)0x10014)
#define JL_PLL_CON0 (*(volatile u32 *)0x119a0)
#define JL_PLL_CON1 (*(volatile u32 *)0x119a4)
#define JL_PLL2_CON0 (*(volatile u32 *)0x119a8)
#define JL_PLL2_CON1 (*(volatile u32 *)0x119ac)

/* Chip ID (JL_INTEST->CHIP_ID). */
#define JL_CHIP_ID (*(volatile u32 *)0x10200)

/*
 * Decryptors: JL_ENC on the SPI0 DMA path (CON 0x83 with decryption, 2
 * without, 0 off; per community docs bit0 = enable, bit7 = reset key, bit1
 * unknown) and JL_SFCENC on the SFC XIP path (CON 1 = on).
 */
#define JL_ENC_CON (*(volatile u32 *)0x13000)
#define JL_ENC_KEY (*(volatile u32 *)0x13004)
#define JL_ENC_ADR (*(volatile u32 *)0x13008)
#define JL_SFCENC_CON (*(volatile u32 *)0x40300)
#define JL_SFCENC_KEY (*(volatile u32 *)0x40304)

/* Pin multiplexing. CON0 bit5 and CON1 bit1 select SPI0 port A (PD0-PD5),
 * CON0 bit16 the SFC boot port. */
#define JL_IOMAP_CON0 (*(volatile u32 *)0x5101c)
#define JL_IOMAP_CON1 (*(volatile u32 *)0x51020)

/* SPI flash controller (memory-mapped XIP at 0x02000000). */
#define JL_SFC_CON (*(volatile u32 *)0x40200)
#define JL_SFC_BAUD (*(volatile u32 *)0x40204)
#define JL_SFC_BASE_ADR (*(volatile u32 *)0x4020c)

/* corex2(0)->CACHE_CON: bit8 RO (flash) cache, bit9 RW (SDRAM) cache, bit14 idle. */
#define CACHE_CON (*(volatile u32 *)0x01eee008)

/* Memory map. */
#define SFC_XIP_BASE 0x02000000 /* flash, through the SFC and its cache */
#define EXT_RAM_BASE 0x04000000 /* PSRAM or SDRAM; not present on the FM-1 */

#endif
