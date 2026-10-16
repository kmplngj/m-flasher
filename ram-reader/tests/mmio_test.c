/* SPDX-License-Identifier: Apache-2.0
 * Execute our actual wl82.c through an injected register model. No MMIO occurs.
 * The model's completion/W1C behavior is an assumption, not silicon evidence. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "fmr.h"
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "%s:%d: %s\n", __FILE__, __LINE__, #x); exit(1); } } while (0)
#define BIT(n) (1u << (n))
enum { CON=0x11c00, BAUD=0x11c04, BUF=0x11c08, ADR=0x11c0c, CNT=0x11c10,
       OUT=0x500c0, DIR=0x500c8, HD=0x500dc, MAP0=0x5101c, MAP1=0x51020,
       SFC=0x40200, ENC=0x13000 };
static const uint32_t addresses[] = {CON,BAUD,BUF,ADR,CNT,OUT,DIR,HD,MAP0,MAP1,SFC,ENC};
static uint32_t regs[12], delays[6], saved_dma_address;
static unsigned phase, active, failed_phase, phase_reads, total_reads;
static unsigned byte_count, dma_count, cs_low_count, cs_high_count;
static uint8_t bytes[8];
static int slot(uint32_t a)
{
    unsigned i;
    for (i=0; i<sizeof addresses/sizeof *addresses; ++i) if (addresses[i]==a) return (int)i;
    CHECK(0); return 0; /* Any optional peripheral register is a test failure. */
}
static uint32_t reg(uint32_t a) { return regs[slot(a)]; }
fmr_u32 fmr_test_mmio_read(fmr_u32 a)
{
    if (a==CON && active) {
        ++phase_reads; ++total_reads;
        if (phase != failed_phase && phase_reads >= delays[phase]) regs[slot(CON)] |= BIT(15);
    }
    if (a==BUF) {
        static const uint8_t id[3] = {0x85,0x60,0x14};
        CHECK(phase>=2 && phase<=4 && (reg(CON)&BIT(12)));
        return id[phase-2];
    }
    return reg(a);
}
void fmr_test_mmio_write(fmr_u32 a, fmr_u32 v)
{
    uint32_t old=reg(a);
    if (a==CON) {
        if ((v&BIT(14)) && (old&BIT(15))) active=0;
        /* Model documented pending-clear as W1C, never a sticky read bit. */
        regs[slot(a)] = v & ~(BIT(14)|BIT(15));
        return;
    }
    if (a==OUT) {
        if ((old&1) && !(v&1)) ++cs_low_count;
        if (!(old&1) && (v&1)) ++cs_high_count;
    }
    if (a==BUF) {
        CHECK(byte_count<sizeof bytes); CHECK(!(reg(OUT)&1)); CHECK(reg(CON)&1);
        bytes[byte_count++]=(uint8_t)v; ++phase; active=1; phase_reads=0;
        CHECK(phase<=5); CHECK(v<=255);
    }
    if (a==ADR) saved_dma_address=v;
    if (a==CNT && v) {
        CHECK(v==256); CHECK(reg(ENC)==2); CHECK(reg(CON)&BIT(12));
        CHECK(!(reg(CON)&(BIT(10)|BIT(11)))); CHECK(!(reg(OUT)&1));
        ++dma_count; ++phase; active=1; phase_reads=0; CHECK(phase==5);
    }
    if (a==ENC) CHECK(v==0 || v==2);
    if (a==SFC) CHECK(v==0);
    regs[slot(a)]=v;
}
static void reset(unsigned fail)
{
    unsigned i;
    for (i=0;i<12;++i) regs[i]=0xa5a5a5a5;
    regs[slot(CON)]=0; regs[slot(OUT)]=1; regs[slot(CNT)]=0; regs[slot(ENC)]=0;
    for (i=0;i<6;++i) delays[i]=1;
    memset(bytes,0xee,sizeof bytes);
    failed_phase=fail; phase=active=phase_reads=total_reads=0;
    byte_count=dma_count=cs_low_count=cs_high_count=saved_dma_address=0;
}
static void quiesced(void)
{
    CHECK(!(reg(CON)&1)); CHECK(reg(CNT)==0); CHECK(reg(ENC)==0); CHECK(reg(OUT)&1);
    CHECK(total_reads<=262170); /* Whole operation, never unbounded per phase. */
}
static void jedec_cases(void)
{
    unsigned fail;
    static const uint8_t expected[4]={0x9f,0xff,0xff,0xff};
    for (fail=0;fail<=4;++fail) {
        uint8_t id[5]={0xa7,0x6c,0x6c,0x6c,0xb8};
        reset(fail); CHECK(fmr_hw_init(id+1)==(fail==0));
        CHECK(id[0]==0xa7 && id[4]==0xb8); quiesced();
        CHECK(phase==(fail?fail:4)); CHECK(byte_count==phase && dma_count==0);
        CHECK(memcmp(bytes,expected,byte_count)==0);
        CHECK(reg(SFC)==0 && reg(BAUD)==1);
        CHECK(cs_low_count==1 && cs_high_count>=1);
        if (!fail) CHECK(id[1]==0x85 && id[2]==0x60 && id[3]==0x14);
        else {
            unsigned i; CHECK(total_reads>=262144);
            for (i=fail;i<=3;++i) CHECK(id[i]==0);
        }
    }
}
struct buffer { uint32_t before; uint8_t data[256]; uint32_t after; };
static void read_cases(void)
{
    unsigned fail, i;
    static const uint8_t expected[4]={0x03,0x0f,0xff,0x00};
    struct buffer b;
    for (fail=0;fail<=5;++fail) {
        memset(&b,0x6d,sizeof b); b.before=0xdeadbeef; b.after=0xcafebabe;
        reset(fail); CHECK(fmr_hw_read256(0xfff00,b.data)==(fail==0)); quiesced();
        CHECK(phase==(fail?fail:5)); CHECK(byte_count==(phase<4?phase:4));
        CHECK(memcmp(bytes,expected,byte_count)==0);
        CHECK(dma_count==(fail==0 || fail==5));
        if (dma_count) CHECK(saved_dma_address==(uint32_t)(uintptr_t)b.data);
        CHECK(b.before==0xdeadbeef && b.after==0xcafebabe);
        for(i=0;i<256;++i) CHECK(b.data[i]==0x6d); /* Model never dereferences a DMA address. */
        if (fail) CHECK(total_reads>=262144);
    }
    reset(0); CHECK(fmr_hw_read256(0,b.data)==1); quiesced();
    CHECK(bytes[0]==3 && bytes[1]==0 && bytes[2]==0 && bytes[3]==0);
    reset(0); CHECK(fmr_hw_read256(1,b.data)==0); CHECK(byte_count==0 && dma_count==0); quiesced();
    reset(0); CHECK(fmr_hw_read256(0x100000,b.data)==0); CHECK(byte_count==0 && dma_count==0); quiesced();
    reset(0); CHECK(fmr_hw_read256(0xffffff00,b.data)==0); CHECK(byte_count==0 && dma_count==0); quiesced();
    reset(0); CHECK(fmr_hw_read256(0,b.data+1)==0); CHECK(byte_count==0 && dma_count==0); quiesced();
    reset(0); delays[1]=200000; delays[2]=100000;
    CHECK(fmr_hw_read256(0,b.data)==0); CHECK(phase==2 && dma_count==0); quiesced();
    CHECK(total_reads>=262144);
}
int main(void)
{
    jedec_cases(); read_cases();
    puts("M1 actual HAL with fake MMIO: 9F/03 traces, all 4 JEDEC and 5 read timeout phases, shared budget, raw DMA256 and quiesce passed (model assumptions only)");
    return 0;
}
