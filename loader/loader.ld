/* The M-Flasher loader, run in RAM from the mask ROM's USB download mode.
 * Loaded at 0x01C20000 (well above JieLi's loader at 0x01C02000 and its
 * buffers up to 0x01C0B9D0), so it never overwrites a loader still in use. */
MEMORY { ram : ORIGIN = LOAD_ADDR, LENGTH = 0x8000 }
ENTRY(loader_entry)

SECTIONS
{
    .text : {
        KEEP(*(.entry))
        *(.text .text.*)
        *(.rodata .rodata.*)
        *(.data .data.*)
        . = ALIGN(4);
    } > ram
    .bss (NOLOAD) : { *(.bss .bss.*) *(COMMON) } > ram
}
