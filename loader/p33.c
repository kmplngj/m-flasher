/*
 * P33 register access. A transfer is three bytes: opcode (bits 7:5) with
 * address bits 9:8, address bits 7:0, then data (SDK p33_buf protocol).
 */
#include "p33.h"
#include "regs.h"

#define P33_OP_WRITE 0x00
#define P33_OP_OR 0x20
#define P33_OP_AND 0x40
#define P33_OP_READ 0x80

/* One byte transfer: SPI_CON bits 3:2 = clock divider - 1, bit4 starts, bit1 = busy. */
static u8 p33_buf(u8 buf)
{
    JL_P33->SPI_CON |= 0xc;
    JL_P33->SPI_DAT = buf;
    JL_P33->SPI_CON |= BIT(4);
    while (JL_P33->SPI_CON & BIT(1)) {
    }
    return JL_P33->SPI_DAT;
}

static u8 p33_xfer(u8 op, u16 addr, u8 data)
{
    u8 ret;
    JL_P33->SPI_CON |= BIT(0);
    p33_buf(op | ((addr >> 8) & 3));
    p33_buf(addr & 0xff);
    ret = p33_buf(data);
    JL_P33->SPI_CON &= ~BIT(0);
    return ret;
}

/* SDK p33_rx_1byte: read, clocked out with a dummy byte. */
u8 p33_rx_1byte(u16 addr)
{
    return p33_xfer(P33_OP_READ, addr, 0x5e);
}

/* SDK p33_tx_1byte. */
void p33_tx_1byte(u16 addr, u8 data)
{
    p33_xfer(P33_OP_WRITE, addr, data);
}

/* Register |= data (SDK P33_OR). */
void p33_or_1byte(u16 addr, u8 data)
{
    p33_xfer(P33_OP_OR, addr, data);
}

/* Register &= data (SDK P33_AND). */
void p33_and_1byte(u16 addr, u8 data)
{
    p33_xfer(P33_OP_AND, addr, data);
}

/* Replace the low len bits of a register (SDK P33_CON_SET with start 0). */
void p33_set_low_bits(u16 addr, u32 len, u8 data)
{
    u8 reg = p33_rx_1byte(addr);
    reg = (reg & ~(~(0xffffffff << len))) | (data & ~(0xffffffff << len));
    p33_tx_1byte(addr, reg);
}

/*
 * A second channel: one byte through SPI_DAT, completion polled on
 * JL_P33.RTC_CON bit1, RTC_CON bit0 held during a command. Inferred from this
 * code only (the SDK uses RTC_CON as an IRQ enable/pending register), so
 * what the channel reaches is unknown; clock_init uses registers 0 and 6 for
 * the 12 MHz RTC oscillator.
 */
static u8 p33_rtccon_xfer(u8 data)
{
    JL_P33->SPI_DAT = data;
    asm volatile("csync");
    asm volatile("csync");
    asm volatile("csync");
    asm volatile("csync");
    asm volatile("csync");
    while (!(JL_P33->RTC_CON & BIT(1))) {
    }
    return JL_P33->SPI_DAT;
}

/* Read: command 0xa0 + reg, then clock the value back. */
u8 p33_rtccon_read(u8 reg)
{
    u8 val;
    JL_P33->RTC_CON |= BIT(0);
    p33_rtccon_xfer(reg + 0xa0);
    val = p33_rtccon_xfer(0);
    JL_P33->RTC_CON &= ~BIT(0);
    return val;
}

/* Write: command 0x20 + reg, then the data byte. */
void p33_rtccon_write(u8 reg, u8 data)
{
    JL_P33->RTC_CON |= BIT(0);
    p33_rtccon_xfer(reg + 0x20);
    p33_rtccon_xfer(data);
    JL_P33->RTC_CON &= ~BIT(0);
}
