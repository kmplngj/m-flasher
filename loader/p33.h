/* P33 (power-domain) register access over the JL_P33 serial interface. */
#ifndef P33_H
#define P33_H

#include "types.h"

/* P33 register addresses (SDK asm/p33.h). */
#define P3_ANA_CON5 0x05    /* bits 2:0 VDDIO level */
#define P3_ANA_CON6 0x06    /* bits 2:0 DCDC14 level */
#define P3_ANA_CON9 0x09    /* bits 3:0 DVDD level */
#define P3_WDT_CON 0x80     /* watchdog; bit4 = enable */
#define P3_P33_LAT 0xad
#define P3_OTH_LAT 0xae
#define P3_USB_LAT 0xaf
#define P3_EFUSE_CON0 0xb0
#define P3_EFUSE_CON1 0xb1
#define P3_EFUSE_RDAT 0xb2
#define P3_PORTA_LAT0 0xd0  /* PORTA..H_LAT0 at 0xd0..0xd7: pins 0-7 */
#define P3_PORTA_LAT1 0xd8  /* PORTA..H_LAT1 at 0xd8..0xdf: pins 8-15 */
#define P3_PORTH_LAT1 0xdf

u8 p33_rx_1byte(u16 addr);
void p33_tx_1byte(u16 addr, u8 data);
void p33_or_1byte(u16 addr, u8 data);
void p33_and_1byte(u16 addr, u8 data);
void p33_set_low_bits(u16 addr, u32 len, u8 data);

u8 p33_rtccon_read(u8 reg);
void p33_rtccon_write(u8 reg, u8 data);

#endif
