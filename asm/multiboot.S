section .header
_start_header:
  magic_number    dd 0xe85250d6
  architecture    dd 0x00000000
  header_length   dd _end_header - _start_header
  checksum        dd 0x100000000 - (0xe85250d6 + 0 + (_end_header - _start_header))
tags:
end_tag:
  dw 0x0000
  dw 0x0000
  dd 0x00000008
_end_header:
