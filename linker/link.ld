ENTRY(mink_enter)
SECTIONS {
  . = 1M;
  .text :
  {
    *(.header)
    *(.text)
  }
  .rodata :
  {
    *(.rodata)
  }
  .data :
  {
    *(.data)
  }
  .bss : 
  {
    *(.bss)
    *(COMMON)
    _bss_end = .;
  }
}
