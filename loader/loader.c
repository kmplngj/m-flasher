/*
 * M-Flasher's own loader for the JieLi WL82 (AC791N) mask ROM's USB download
 * mode.
 *
 * Like JieLi's loader it is a plug-in for the ROM's USB mass-storage stack:
 * the ROM's JUMP command calls loader_entry with a pointer to {send, recv,
 * handler_slot}; the loader puts its handler in the slot and returns. The
 * ROM then calls handler(cbw, answer) for every command: the 31-byte USB
 * command block (the 16-byte command from offset 15: a big-endian opcode and
 * big-endian arguments) and a 16-byte answer buffer. The handler moves data
 * with send()/recv() and returns 1 if it handled the command, 0 to leave it
 * to the ROM (which keeps its own memory read/write and jump).
 *
 * Commands, compatible with JieLi's loader (so the app works with either):
 *   FB00 erase 64 KiB block   FB01 erase 4 KiB sector   FB04 write flash
 *   FD05 read flash           FC09 chip key (scrambled)  FC0A online device
 *   FC0B JEDEC ID             FC0E flash CRC16            FC14 buffer size
 * New:
 *   FC40 identify: "MFLR", version (4 chars), its address (BE), features
 *   FC41 CRC32 of a flash range (addr u32, len u32), computed here
 *   FC42 the flash chip's factory-unique ID (4Bh), 8 bytes
 *   FC43 display on (board), FC44 a line of text, FC45 a progress bar
 * Not implemented, on purpose: burning the chip key efuse, chip erase.
 */
#include "display.h"
#include "p33.h"
#include "regs.h"

#define VERSION "0.3"
#define BUF_SIZE 4096           /* data per command */
#define CHUNK 512               /* per send/recv call */

struct rom_io {
    int (*send)(void *buf, u32 len);
    int (*recv)(void *buf, u32 len);
    void **handler_slot;
};

static struct rom_io io;
static u8 buf[BUF_SIZE];
static u32 last_error;
static u32 read_chunk = CHUNK;  /* bytes per send() when reading */          /* FC46: what the last refused command was */
static u32 flash_id;

/* ---- helpers ---- */

static void spin(u32 n)
{
    while (n--) {
        asm volatile("nop");
    }
}

static u32 be32(const u8 *p)
{
    return (u32)p[0] << 24 | p[1] << 16 | p[2] << 8 | p[3];
}

static u32 be16(const u8 *p)
{
    return p[0] << 8 | p[1];
}

static void put_be32(u8 *p, u32 v)
{
    p[0] = v >> 24;
    p[1] = v >> 16;
    p[2] = v >> 8;
    p[3] = v;
}

/* CRC16/XMODEM, as JieLi's tools use everywhere. */
static u16 crc16_update(u16 crc, const u8 *p, u32 n)
{
    while (n--) {
        crc ^= *p++ << 8;
        for (int i = 0; i < 8; i++) {
            crc = crc & 0x8000 ? crc << 1 ^ 0x1021 : crc << 1;
        }
    }
    return crc;
}

/* CRC32 (IEEE, reflected), table-driven. */
static u32 crc32_table[256];

static void crc32_init(void)
{
    for (u32 i = 0; i < 256; i++) {
        u32 c = i;
        for (int k = 0; k < 8; k++) {
            c = c & 1 ? 0xEDB88320 ^ (c >> 1) : c >> 1;
        }
        crc32_table[i] = c;
    }
}

static u32 crc32_update(u32 crc, const u8 *p, u32 n)
{
    while (n--) {
        crc = crc32_table[(crc ^ *p++) & 0xff] ^ (crc >> 8);
    }
    return crc;
}

/* JieLi's "CrcDecode" stream cipher (key 0xFFFFFFFF), for the chip key answer. */
static void crc_cipher(u8 *p, u32 n)
{
    static const u8 magic[16] = {0xC3, 0xCF, 0xC0, 0xE8, 0xCE, 0xD2, 0xB0, 0xAE,
                                 0xC4, 0xE3, 0xA3, 0xAC, 0xD3, 0xF1, 0xC1, 0xD6};
    u8 k[2] = {0xFF, 0xFF};
    u16 c = crc16_update(0xFFFF, k, 2);
    for (u32 i = 0; i < n; i++) {
        c = crc16_update(c, &magic[i % 16], 1);
        p[i] ^= (u8)c;
    }
}

/* ---- SPI NOR flash on SPI0 (CS on PD0), 1-bit mode ---- */

static void cs(int level)
{
    JL_PORTD->DIR &= ~BIT(0);
    if (level) {
        JL_PORTD->OUT |= BIT(0);
    } else {
        JL_PORTD->OUT &= ~BIT(0);
    }
    spin(20);
}

static void spi_wait(void)
{
    while (!(JL_SPI0->CON & BIT(15))) {
    }
    JL_SPI0->CON |= BIT(14);
}

static void spi_tx(u32 b)
{
    JL_SPI0->CON &= ~BIT(12);
    JL_SPI0->BUF = b;
    spi_wait();
}

static u8 spi_rx(void)
{
    JL_SPI0->CON |= BIT(12);
    JL_SPI0->BUF = 0xff;
    spi_wait();
    return JL_SPI0->BUF;
}

static void flash_cmd_addr(u8 cmd, u32 addr)
{
    cs(0);
    spi_tx(cmd);
    spi_tx(addr >> 16);
    spi_tx(addr >> 8);
    spi_tx(addr);
}

static u8 flash_status(u8 cmd)
{
    cs(0);
    spi_tx(cmd);
    u8 s = spi_rx();
    cs(1);
    return s;
}

static void flash_wait(void)
{
    while (flash_status(0x05) & 1) {
        spin(200);
    }
}

static void flash_wren(void)
{
    cs(0);
    spi_tx(0x06);
    cs(1);
}

static void flash_init(void)
{
    JL_SFC_CON = 0;
    JL_SPI0->CON = 0x28;
    JL_SPI0->BAUD = 1;                  /* 0..2 read correctly; USB is the limit below 2 */
    JL_IOMAP_CON1 &= ~BIT(1);           /* SPI0 port A pins (PD1..PD5) */
    JL_IOMAP_CON0 &= ~BIT(5);
    JL_PORTD->DIR &= ~(BIT(4) | BIT(5));
    JL_PORTD->HD |= BIT(4);
    JL_PORTD->DIR |= BIT(1);            /* DI in */
    JL_SPI0->CON |= BIT(14);
    JL_SPI0->CON |= BIT(0);
    cs(1);
    cs(0);
    spi_tx(0x9f);
    flash_id = spi_rx() << 16;
    flash_id |= spi_rx() << 8;
    flash_id |= spi_rx();
    cs(1);
}

/* Read with the SPI0 DMA (1-bit), the data straight into RAM; the decryptor
 * on the SPI path off. */
static void flash_read(u32 addr, u8 *p, u32 n)
{
    flash_cmd_addr(0x03, addr);
    JL_ENC_CON = 2;
    JL_SPI0->CON |= BIT(12);
    JL_SPI0->CON &= ~(BIT(10) | BIT(11));
    JL_SPI0->ADR = (u32)p;
    JL_SPI0->CNT = n;
    spi_wait();
    JL_ENC_CON = 0;
    cs(1);
}

/* Clear the block-protect bits (SR1 BP0-BP2, TB, SEC; SR2 CMP), keeping QE. */
static void flash_unprotect(void)
{
    u8 sr1 = flash_status(0x05), sr2 = flash_status(0x35);
    if ((sr1 & 0x7c) || (sr2 & 0x40)) {
        flash_wren();
        cs(0);
        spi_tx(0x01);
        spi_tx(sr1 & ~0x7c);
        spi_tx(sr2 & ~0x40);
        cs(1);
        flash_wait();
    }
}

static void flash_erase(u32 addr, int block)
{
    flash_unprotect();
    flash_wren();
    flash_cmd_addr(block ? 0xd8 : 0x20, addr);
    cs(1);
    flash_wait();
}

/* Program n bytes from addr, in page-sized (256-byte) pieces. */
static void flash_program(u32 addr, const u8 *p, u32 n)
{
    flash_unprotect();
    while (n) {
        u32 k = 256 - (addr & 255);
        if (k > n) {
            k = n;
        }
        flash_wren();
        flash_cmd_addr(0x02, addr);
        for (u32 i = 0; i < k; i++) {
            spi_tx(p[i]);
        }
        cs(1);
        flash_wait();
        addr += k;
        p += k;
        n -= k;
    }
}

/* ---- efuse chip key (bits 23:8 of efuse word 1), as the bootloader reads it ---- */

static u16 chip_key(void)
{
    static const u32 mask[2] = {0xffffffff, 0xffffffff};
    u32 n = (JL_CHIP_ID & 0xff) ? 4 : 3;
    u32 value = 0;
    for (u32 i = 1; i < 2; i++) {
        u32 tmp = 0;
        p33_tx_1byte(P3_EFUSE_CON0, 1 << 6);
        for (u32 j = 0; j < n; j++) {
            p33_tx_1byte(P3_EFUSE_CON1, 3 << 6 | j * 4 | 2);
            spin(2000);
            tmp |= p33_rx_1byte(P3_EFUSE_RDAT) << (j * 8);
            p33_tx_1byte(P3_EFUSE_CON1, 0);
        }
        value |= mask[i - 1] & tmp;
    }
    return value >> 8;
}

/* ---- commands ---- */

static void send_answer(u8 *answer, u32 want)
{
    io.send(answer, want < 16 ? want : 16);
}

int loader_entry(struct rom_io *rom);

static int handler(u8 *cbw, u8 *answer)
{
    const u8 *cmd = cbw + 15;
    u32 op = be16(cmd);
    u32 want = cbw[8] | cbw[9] << 8 | cbw[10] << 16 | (u32)cbw[11] << 24;
    u32 addr = be32(cmd + 2);
    u32 n, done, k;

    for (int i = 0; i < 16; i++) {
        answer[i] = 0;
    }
    answer[0] = op >> 8;
    answer[1] = op;
    switch (op) {
    case 0xFB00:                        /* erase block */
    case 0xFB01:                        /* erase sector */
        flash_erase(addr, op == 0xFB00);
        send_answer(answer, want);
        return 1;
    case 0xFB04:                        /* write: addr, len u16, 0, crc16 LE of the data */
        n = be16(cmd + 6);
        /* The ROM cannot fail a command whose data it has passed on: a
         * refusal is "handled" without writing, recorded for FC46; the PC
         * reads back what it wrote anyway. */
        for (done = 0; done < n; done += k) {
            k = n - done < CHUNK ? n - done : CHUNK;
            io.recv(buf + (done % BUF_SIZE), k);
        }
        if (n > BUF_SIZE) {
            last_error = 1;             /* too long */
        } else if (crc16_update(0, buf, n) != (cmd[9] | cmd[10] << 8)) {
            last_error = 2;             /* damaged transfer: nothing written */
        } else {
            flash_program(addr, buf, n);
        }
        return 1;
    case 0xFD05:                        /* read: addr, len u16 */
        n = be16(cmd + 6);
        for (done = 0; done < n; done += k) {
            k = n - done < read_chunk ? n - done : read_chunk;
            flash_read(addr + done, buf, k);
            io.send(buf, k);
        }
        return 1;
    case 0xFC09: {                      /* chip key, scrambled as JieLi's loader does */
        u16 key = chip_key();
        u8 c[2] = {key, key >> 8};
        crc_cipher(c, 2);
        answer[7] = c[0];
        answer[6] = c[1];
        send_answer(answer, want);
        return 1;
    }
    case 0xFC0A:                        /* online device: SPI0 NOR + JEDEC ID */
        answer[2] = 3;
        answer[4] = flash_id;
        answer[5] = flash_id >> 8;
        answer[6] = flash_id >> 16;
        send_answer(answer, want);
        return 1;
    case 0xFC0B:                        /* JEDEC ID */
        answer[2] = flash_id >> 16;
        answer[3] = flash_id >> 8;
        answer[4] = flash_id;
        send_answer(answer, want);
        return 1;
    case 0xFC0E: {                      /* CRC16 of addr, len u16 */
        u16 crc = 0;
        n = be16(cmd + 6);
        for (done = 0; done < n; done += k) {
            k = n - done < BUF_SIZE ? n - done : BUF_SIZE;
            flash_read(addr + done, buf, k);
            crc = crc16_update(crc, buf, k);
        }
        answer[2] = crc >> 8;
        answer[3] = crc;
        send_answer(answer, want);
        return 1;
    }
    case 0xFC14:                        /* buffer size */
        put_be32(answer + 2, BUF_SIZE);
        send_answer(answer, want);
        return 1;
    case 0xFC40: {                      /* identify */
        static const char id[] = "MFLR" VERSION;
        for (int i = 0; id[i]; i++) {
            answer[2 + i] = id[i];
        }
        /* answer: "MFLR" and the version at 2..8, where it runs at 10..13 (the PC
         * loads a new one elsewhere), features at 14 */
        put_be32(answer + 10, (u32)loader_entry);
        answer[14] = 0x0f;              /* features: CRC32, unique ID, display, status */
        send_answer(answer, want);
        return 1;
    }
    case 0xFC41: {                      /* CRC32 of addr u32, len u32 */
        u32 crc = 0xffffffff;
        n = be32(cmd + 6);
        for (done = 0; done < n; done += k) {
            k = n - done < BUF_SIZE ? n - done : BUF_SIZE;
            flash_read(addr + done, buf, k);
            crc = crc32_update(crc, buf, k);
        }
        put_be32(answer + 2, ~crc);
        send_answer(answer, want);
        return 1;
    }
    case 0xFC43:                        /* display on: board 1 = FM-1, 2 = SMK-37 */
        answer[2] = display_init(cmd[2]);
        send_answer(answer, want);
        return 1;
    case 0xFC44:                        /* text: row, fg u16, bg u16; data-out: up to 20 characters */
        n = be16(cmd + 7);
        io.recv(buf, n > 32 ? 32 : n);
        display_text(cmd[2], (const char *)buf, n > 32 ? 32 : n, be16(cmd + 3), be16(cmd + 5));
        return 1;
    case 0xFC45:                        /* progress bar: percent, color u16 */
        display_progress(cmd[2], be16(cmd + 3));
        send_answer(answer, want);
        return 1;
    case 0xFC46:                        /* status: the last refusal (0 none, 1 too long, 2 CRC), cleared */
        answer[2] = last_error;
        last_error = 0;
        send_answer(answer, want);
        return 1;
    case 0xFC4F:                        /* (testing) SPI0 clock divider */
        JL_SPI0->BAUD = cmd[2];
        if (cmd[3] >= 1 && cmd[3] <= BUF_SIZE / 256) {
            read_chunk = cmd[3] * 256;
        }
        send_answer(answer, want);
        return 1;
    case 0xFC42:                        /* factory-unique ID: 4Bh, 4 dummy bytes, 8 bytes */
        cs(0);
        spi_tx(0x4b);
        for (int i = 0; i < 4; i++) {
            spi_tx(0);
        }
        for (int i = 0; i < 8; i++) {
            answer[2 + i] = spi_rx();
        }
        cs(1);
        send_answer(answer, want);
        return 1;
    }
    return 0;
}

__attribute__((section(".entry"), used))
int loader_entry(struct rom_io *rom)
{
    io.send = rom->send;
    io.recv = rom->recv;
    io.handler_slot = rom->handler_slot;
    crc32_init();
    flash_init();
    *rom->handler_slot = (void *)handler;
    return 0;
}
