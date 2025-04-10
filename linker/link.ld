ENTRY(_start)
SECTIONS {
  . = 0x100000;
  .mb_header : {
    *(.multiboot_header)
  }
  .text : {
    *(.text .text.*)
  }
  .data : {
    *(.data .data.*)
  }
  .bss : {
    *(.bss .bss.*)
    *(COMMON)
    _bss_end = .;
  }
  .stack (NOLOAD) : {
    . = ALIGN(8);
    stack_start = .;
    . += 0x10000;
    stack_end = .;
  }
  .rodata : {
    *(.rodata .rodata.*)
  }
}
