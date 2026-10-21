/* SPDX-License-Identifier: Apache-2.0
 * Modified 2026-10-16 from M-Flasher's simple RAM-link layout: fixed own
 * 32KiB reservation, entry-first assertions, explicit data/BSS geometry.
 */
MEMORY { ram (rwx) : ORIGIN = 0x01C20000, LENGTH = 0x8000 }
ENTRY(loader_entry)
SECTIONS
{
    .text : {
        __fmr_start = .;
        KEEP(*(.entry))
        *(.text .text.*)
        *(.rodata .rodata.*)
        . = ALIGN(4);
    } > ram
    .data : {
        *(.data .data.*)
        . = ALIGN(4);
        __fmr_image_end = .;
    } > ram
    .init_array : {
        KEEP(*(.preinit_array .preinit_array.* .init_array .init_array.* .ctors .ctors.*))
    } > ram
    .fini_array : { KEEP(*(.fini_array .fini_array.* .dtors .dtors.*)) } > ram
    .forbidden : { KEEP(*(.init .fini .jcr .tdata .tdata.* .tbss .tbss.*)) } > ram
    .bss (NOLOAD) : {
        __fmr_bss_start = .;
        *(.bss .bss.*) *(COMMON)
        . = ALIGN(4);
        __fmr_bss_end = .;
    } > ram
    /DISCARD/ : { *(.comment) *(.note*) *(.eh_frame*) *(.llvm_addrsig) }
}
ASSERT(loader_entry == ORIGIN(ram), "entry must be the first image byte")
ASSERT(__fmr_image_end <= ORIGIN(ram) + LENGTH(ram), "load image exceeds reservation")
ASSERT(__fmr_bss_end <= ORIGIN(ram) + LENGTH(ram), "state exceeds reservation")
ASSERT(SIZEOF(.init_array) == 0, "constructors forbidden")
ASSERT(SIZEOF(.fini_array) == 0, "destructors forbidden")
ASSERT(SIZEOF(.forbidden) == 0, "runtime initialization/TLS forbidden")
