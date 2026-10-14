/* Basic types and helpers shared by every module. */
#ifndef TYPES_H
#define TYPES_H

#include <stddef.h>

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef signed int s32;

#define BIT(n) (1u << (n))

/* Replace bits [start, start + len) of a register with dat (vendor SFR macro). */
#define SFR(sfr, start, len, dat) \
    (sfr = ((sfr) & ~((~(0xffffffff << (len))) << (start))) | (((dat) & (~(0xffffffff << (len)))) << (start)))

/* Little-endian word at an unaligned address. */
struct unaligned_u32 {
    u32 v;
} __attribute__((packed));

/* From the toolchain's libc.a (the vendor bootloader links the same library). */
void *memcpy(void *dst, const void *src, size_t n);
void *memmove(void *dst, const void *src, size_t n);
void *memset(void *dst, int c, size_t n);
int memcmp(const void *a, const void *b, size_t n);
size_t strlen(const char *s);
int strcmp(const char *a, const char *b);

#endif
