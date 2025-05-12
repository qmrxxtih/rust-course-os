section .bss
  ; initial setup of paging (OSdev wiki)
  align 4096
pmap_l4_table:
  resb 4096
pdp_table:
  resb 4096
pd_table:
  resb 4096
stack_end:
  resb 0x10000
stack_start:


section .rodata
; GDT table entries
longmode_gdt:
  dq 0 ; zero entry
gdt_code: equ $ - longmode_gdt
  dq 0x20980000000000 ; code segment
.gdt_pointer:
  dw $ - longmode_gdt - 1
  dq longmode_gdt

section .text
  global _start
  extern start_lm 
  bits 32

_start:
  ; Multiboot2 specification sets up processor into protected mode automatically,
  ; including enabling of A20 address line.
  ; However, it is up to programmer to create their own stack, set up paging, 
  ; set up GDT (global descriptor table) for segments and IDT (interrupt descriptor table)
  ; for interrupts.
  ; Basic stack, initial paging, and GDT setup is done here.
  ; IDT setup and additional initialisation work is done in Rust.
  mov esp,stack_start   ; INITIALISE STACK
  mov edi, ebx          ; 1st argument for Rust program = Multiboot Boot Information

  cmp eax,0x36d76289    ; check if we really are booted in multiboot
  jnz invalid_boot      ; if not, report error

  ; CHECKING IF CPUID IS AVAILABLE (from OSdev wiki)
  pushfd ; copy flags into eax via stack
  pop eax
  mov ecx,eax ;copy to ECX for later
  xor eax,0x200000 ; flip ID bit
  push eax
  popfd ; copy EAX back to flags
  pushfd ; ... and then back to EAX (bit 21 will be flipped if CPUID is supported)
  pop eax
  push ecx ; restore flags
  popfd

  cmp eax,ecx ; compare EAX and ECX - if equal, bit was not flipped = CPUID not supported
  jz cpuid_unavailable 

  ; CHECKING IF LONG MODE IS AVAILABLE (from OSdev wiki)
  mov eax,0x80000000 ; cpuid argument (extended info availability)
  cpuid
  cmp eax,0x80000001 ; returned value has to be >= 0x80000001
  jb longmode_unavailable
  mov eax,0x80000001 ; cpuid argument (extended info - long mode availability)
  cpuid
  and edx,0x20000000 ; test LONG MODE bit
  jz longmode_unavailable

  ; SETTING UP PAGE ENTRIES
  mov eax,pdp_table ; loads address of PDP
  or eax,0x03 ; set PDP's present and writable flags
  mov [pmap_l4_table],eax ; maps first PMAP entry to PDP
  mov eax,pd_table ; loads address of PD
  or eax,0x03 ; set PD's present and writable flags
  mov [pdp_table],eax ; maps first PDP entry to PD

  ; SETTING UP PAGE TABLES
  ; each P2 entry -> 4MiB page
  mov ecx,0 ; counter
_map_pd:
  mov eax,0x200000 ; EAX = 2 MiB
  mul ecx ; offset by ECX
  or eax,0x83 ; huge, writable, present
  mov [pd_table + ecx * 8],eax ; map entry according to ECX
  inc ecx
  cmp ecx,512 ; we want to map whole PD (1 GiB)
  jnz _map_pd

  ; ENABLE PAGING
  mov eax,cr4
  or eax,(1 << 5)
  mov cr4,eax ; enable PAE flag in Control Register 4 (physical adress ext.)

  mov ecx,0xc0000080
  rdmsr
  or eax,(1 << 8)
  wrmsr ; set long mode bit in MSR (model specific register)

  mov eax,pmap_l4_table 
  mov cr3,eax ; load page map to cr3 (the page table register)

  mov eax,cr0
  or eax,(1 << 31) | (1 << 0)
  mov cr0,eax ; enable paging in Control Register 0

  ; LOAD NEW GDT
  lgdt [longmode_gdt.gdt_pointer]

  ; JUMP TO NEW CODE ADDRESS
  jmp gdt_code:start_lm


longmode_unavailable:
cpuid_unavailable:
invalid_boot:
  hlt ; halt the processor
  jmp invalid_boot ;loop on error
  
