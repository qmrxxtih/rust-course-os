section .text
  global _start
  extern mink_enter
  bits 32
_start:
  ;mov dword [0xb8000], 0x2f4b2f4f
  call mink_enter
  hlt

