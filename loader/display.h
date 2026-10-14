#ifndef DISPLAY_H
#define DISPLAY_H
#include "types.h"
int display_init(u32 board);
void display_fill(u32 x, u32 y, u32 w, u32 h, u16 color);
void display_text(u32 row, const char *s, u32 n, u16 fg, u16 bg);
void display_progress(u32 percent, u16 color);
#endif
