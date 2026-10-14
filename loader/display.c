/*
 * The unit's display (ST7789, 240 x 240, on SPI1) from the loader: the
 * bootloader's driver (lcd.c) with both boards' pins and setup tables, chosen
 * by the PC (FC43). Text in 12 x 16 cells (20 x 15) with the bootloader's
 * 5 x 7 font drawn twice the size.
 */
#include "display.h"
#include "regs.h"

#define LCD_W 240
#define LCD_COLS 20

/* Font drawn for this project: column bytes, bit 0 = top row. Lower case is
 * shown as upper case; other characters as a blank. */
static const char font_chars[] = " -./:<>+?0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
static const u8 font[][5] = {
    {0x00, 0x00, 0x00, 0x00, 0x00}, /*   */
    {0x08, 0x08, 0x08, 0x08, 0x08}, /* - */
    {0x00, 0x60, 0x60, 0x00, 0x00}, /* . */
    {0x60, 0x10, 0x08, 0x04, 0x03}, /* / */
    {0x00, 0x36, 0x36, 0x00, 0x00}, /* : */
    {0x00, 0x08, 0x14, 0x22, 0x41}, /* < */
    {0x41, 0x22, 0x14, 0x08, 0x00}, /* > */
    {0x08, 0x08, 0x3e, 0x08, 0x08}, /* + */
    {0x02, 0x01, 0x51, 0x09, 0x06}, /* ? */
    {0x3e, 0x51, 0x49, 0x45, 0x3e}, /* 0 */
    {0x00, 0x42, 0x7f, 0x40, 0x00}, /* 1 */
    {0x42, 0x61, 0x51, 0x49, 0x46}, /* 2 */
    {0x21, 0x41, 0x45, 0x4b, 0x31}, /* 3 */
    {0x18, 0x14, 0x12, 0x7f, 0x10}, /* 4 */
    {0x27, 0x45, 0x45, 0x45, 0x39}, /* 5 */
    {0x3c, 0x4a, 0x49, 0x49, 0x30}, /* 6 */
    {0x01, 0x71, 0x09, 0x05, 0x03}, /* 7 */
    {0x36, 0x49, 0x49, 0x49, 0x36}, /* 8 */
    {0x06, 0x49, 0x49, 0x29, 0x1e}, /* 9 */
    {0x7e, 0x09, 0x09, 0x09, 0x7e}, /* A */
    {0x7f, 0x49, 0x49, 0x49, 0x36}, /* B */
    {0x3e, 0x41, 0x41, 0x41, 0x22}, /* C */
    {0x7f, 0x41, 0x41, 0x41, 0x3e}, /* D */
    {0x7f, 0x49, 0x49, 0x49, 0x41}, /* E */
    {0x7f, 0x09, 0x09, 0x09, 0x01}, /* F */
    {0x3e, 0x41, 0x49, 0x49, 0x7a}, /* G */
    {0x7f, 0x08, 0x08, 0x08, 0x7f}, /* H */
    {0x00, 0x41, 0x7f, 0x41, 0x00}, /* I */
    {0x20, 0x40, 0x41, 0x3f, 0x01}, /* J */
    {0x7f, 0x08, 0x14, 0x22, 0x41}, /* K */
    {0x7f, 0x40, 0x40, 0x40, 0x40}, /* L */
    {0x7f, 0x02, 0x0c, 0x02, 0x7f}, /* M */
    {0x7f, 0x04, 0x08, 0x10, 0x7f}, /* N */
    {0x3e, 0x41, 0x41, 0x41, 0x3e}, /* O */
    {0x7f, 0x09, 0x09, 0x09, 0x06}, /* P */
    {0x3e, 0x41, 0x51, 0x21, 0x5e}, /* Q */
    {0x7f, 0x09, 0x19, 0x29, 0x46}, /* R */
    {0x46, 0x49, 0x49, 0x49, 0x31}, /* S */
    {0x01, 0x01, 0x7f, 0x01, 0x01}, /* T */
    {0x3f, 0x40, 0x40, 0x40, 0x3f}, /* U */
    {0x1f, 0x20, 0x40, 0x20, 0x1f}, /* V */
    {0x3f, 0x40, 0x38, 0x40, 0x3f}, /* W */
    {0x63, 0x14, 0x08, 0x14, 0x63}, /* X */
    {0x03, 0x04, 0x78, 0x04, 0x03}, /* Y */
    {0x61, 0x51, 0x49, 0x45, 0x43}, /* Z */
};

struct board {
    void (*pins)(void);
    void (*cs)(int);
    void (*dc)(int);
    const u8 *setup;
};

/* FM-1: SPI1 group B (CLK PC9, MOSI PC10), CS PC7, D/C PC8, backlight PA2 (low = on). */
static void fm1_pins(void)
{
    JL_PORTA->OUT &= ~BIT(2);
    JL_PORTA->DIR &= ~BIT(2);
    JL_IOMAP_CON1 |= BIT(4);
    JL_PORTC->OUT |= BIT(7);
    JL_PORTC->OUT &= ~(BIT(8) | BIT(9) | BIT(10));
    JL_PORTC->DIR &= ~(BIT(7) | BIT(8) | BIT(9) | BIT(10));
}
static void fm1_cs(int v) { if (v) JL_PORTC->OUT |= BIT(7); else JL_PORTC->OUT &= ~BIT(7); }
static void fm1_dc(int v) { if (v) JL_PORTC->OUT |= BIT(8); else JL_PORTC->OUT &= ~BIT(8); }
static const u8 fm1_setup[] = {
    0x01, 0, 0x00, 1, 150, 0x11, 0, 0x00, 1, 120, 0x3a, 1, 0x55, 0x36, 1, 0x00, 0x21, 0, 0x13, 0, 0xff,
};

/* SMK-37: SPI1 group A (CLK PB6, MOSI PB7), CS PC8, D/C PB5, no backlight pin. */
static void smk_pins(void)
{
    JL_IOMAP_CON1 &= ~BIT(4);
    JL_PORTC->OUT |= BIT(8);
    JL_PORTC->DIR &= ~BIT(8);
    JL_PORTB->OUT &= ~(BIT(5) | BIT(6) | BIT(7));
    JL_PORTB->DIR &= ~(BIT(5) | BIT(6) | BIT(7));
}
static void smk_cs(int v) { if (v) JL_PORTC->OUT |= BIT(8); else JL_PORTC->OUT &= ~BIT(8); }
static void smk_dc(int v) { if (v) JL_PORTB->OUT |= BIT(5); else JL_PORTB->OUT &= ~BIT(5); }
static const u8 smk_setup[] = {
    0x00, 1, 100, 0x11, 0, 0x00, 1, 120, 0x36, 1, 0x00, 0x3a, 1, 0x05,
    0xb2, 5, 0x0c, 0x0c, 0x00, 0x33, 0x33, 0xb7, 1, 0x35, 0xbb, 1, 0x32, 0xc2, 1, 0x01,
    0xc3, 1, 0x15, 0xc4, 1, 0x20, 0xc6, 1, 0x0f, 0xd0, 2, 0xa4, 0xa1,
    0xe0, 14, 0xd0, 0x08, 0x0e, 0x09, 0x09, 0x05, 0x31, 0x33, 0x48, 0x17, 0x14, 0x15, 0x31, 0x34,
    0xe1, 14, 0xd0, 0x08, 0x0e, 0x09, 0x09, 0x05, 0x31, 0x33, 0x48, 0x17, 0x14, 0x15, 0x31, 0x34,
    0x21, 0, 0xff,
};

static const struct board boards[2] = {
    {fm1_pins, fm1_cs, fm1_dc, fm1_setup},
    {smk_pins, smk_cs, smk_dc, smk_setup},
};
static const struct board *b;
static u8 lcd_buf[LCD_W * 2];

static void spin_ms(u32 ms)
{
    for (u32 n = ms * 80000; n; n--) {
        asm volatile("nop");
    }
}

static void spi1_wait(void)
{
    for (u32 n = 0; n < 1000000 && !(JL_SPI1->CON & BIT(15)); n++) {
    }
    JL_SPI1->CON |= BIT(14);
}

static void lcd_cmd(u8 cmd)
{
    b->dc(0);
    b->cs(0);
    JL_SPI1->CON |= BIT(14);
    JL_SPI1->BUF = cmd;
    spi1_wait();
    b->cs(1);
}

static void lcd_data(const u8 *p, u32 n)
{
    if (n == 0) {
        return;
    }
    b->dc(1);
    b->cs(0);
    JL_SPI1->CON |= BIT(14);
    JL_SPI1->ADR = (u32)p;
    JL_SPI1->CNT = n;
    spi1_wait();
    b->cs(1);
}

static void lcd_window(u32 x, u32 y, u32 w, u32 h)
{
    u8 a[4] = {x >> 8, x, (x + w - 1) >> 8, x + w - 1};
    lcd_cmd(0x2a);
    lcd_data(a, 4);
    a[0] = y >> 8;
    a[1] = y;
    a[2] = (y + h - 1) >> 8;
    a[3] = y + h - 1;
    lcd_cmd(0x2b);
    lcd_data(a, 4);
    lcd_cmd(0x2c);
}

void display_fill(u32 x, u32 y, u32 w, u32 h, u16 color)
{
    u32 n = w * h;
    if (!b || n == 0) {
        return;
    }
    for (u32 i = 0; i < LCD_W; i++) {
        lcd_buf[2 * i] = color >> 8;
        lcd_buf[2 * i + 1] = color;
    }
    lcd_window(x, y, w, h);
    for (; n > LCD_W; n -= LCD_W) {
        lcd_data(lcd_buf, sizeof(lcd_buf));
    }
    lcd_data(lcd_buf, 2 * n);
}

int display_init(u32 board)
{
    if (board < 1 || board > 2) {
        return 0;
    }
    b = &boards[board - 1];
    b->pins();
    JL_SPI1->CON = 0x4021;
    JL_SPI1->BAUD = 4;
    for (const u8 *s = b->setup; s[0] != 0xff; s += 2 + s[1]) {
        if (s[0] == 0) {
            spin_ms(s[2]);
        } else {
            lcd_cmd(s[0]);
            lcd_data(s + 2, s[1]);
        }
    }
    display_fill(0, 0, LCD_W, LCD_W, 0);
    lcd_cmd(0x29);
    return 1;
}

static const u8 *glyph(char c)
{
    if (c >= 'a' && c <= 'z') {
        c -= 'a' - 'A';
    }
    for (u32 i = 0; font_chars[i]; i++) {
        if (font_chars[i] == c) {
            return font[i];
        }
    }
    return font[0];
}

/* Text in row (0..14) from column 0; the rest of the row cleared to bg. */
void display_text(u32 row, const char *s, u32 n, u16 fg, u16 bg)
{
    if (!b) {
        return;
    }
    for (u32 col = 0; col < LCD_COLS; col++) {
        const u8 *g = glyph(col < n && s[col] ? s[col] : ' ');
        u8 *p = lcd_buf;
        for (u32 y = 0; y < 16; y++) {
            for (u32 x = 0; x < 12; x++) {
                u16 c = x < 10 && y < 14 && (g[x >> 1] >> (y >> 1)) & 1 ? fg : bg;
                *p++ = c >> 8;
                *p++ = c;
            }
        }
        lcd_window(col * 12, row * 16, 12, 16);
        lcd_data(lcd_buf, 12 * 16 * 2);
    }
}

/* A progress bar in rows 9 and 10: frame, filled to percent. */
void display_progress(u32 percent, u16 color)
{
    if (!b) {
        return;
    }
    if (percent > 100) {
        percent = 100;
    }
    u32 w = 216 * percent / 100;
    display_fill(10, 148, 220, 2, 0xffff);
    display_fill(10, 172, 220, 2, 0xffff);
    display_fill(10, 148, 2, 26, 0xffff);
    display_fill(228, 148, 2, 26, 0xffff);
    display_fill(12, 152, w, 18, color);
    display_fill(12 + w, 152, 216 - w, 18, 0);
}
