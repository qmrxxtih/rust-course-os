section .text
  global start_lm
  extern mink_entry
start_lm:
  ; RESET ALL SEGMENT REGISTERS (THEY ARE USELESS NOW)
  mov ax,0
  mov ss,ax
  mov ds,ax
  mov es,ax
  mov fs,ax
  mov gs,ax

  ; PRINT OKAY
  ; mov rax,0x2f592f412f4b2f4f
  ; mov qword [0xb8000],rax
  call mink_entry
  hlt
