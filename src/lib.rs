#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod guru;
mod interrupts;
mod multiboot;
mod pic;
mod port;
mod vga;

use core::panic::PanicInfo;

use interrupts::init_idt;

use multiboot::Multiboot2;
use port::output_byte;
use vga::{vga_set_foreground, VgaTextModeColor};

const BIG_MINK: &str = "                   88             88         
                   \"\"             88         
                                  88         
88,dPYba,,adPYba,  88 8b,dPPYba,  88   ,d8   
88P\'   \"88\"    \"8a 88 88P\'   `\"8a 88 ,a8\"    
88      88      88 88 88       88 8888[      
88      88      88 88 88       88 88`\"Yba,   
88      88      88 88 88       88 88   `Y8a
";


#[unsafe(no_mangle)]
pub extern "C" fn mink_entry(multiboot_addr: usize) -> ! {
    let mbi = Multiboot2::from_ptr(multiboot_addr as *const u32);

    // Initialising interrupt vector by loading IDT (Interrupt Descriptor Table)
    init_idt();
    // Initialising PIC8259 interrupt chain
    pic::init();
    // Enabling external interrupts by calling STI (set interrupt) instruction
    x86_64::instructions::interrupts::enable();

    // Disable blinking cursor by writing VGA control registers
    output_byte(0x3D4, 0x0A);
    output_byte(0x3D5, 0x20);

    vga_set_foreground(VgaTextModeColor::LightMagenta);
    vga_printf!("{}", BIG_MINK);
    vga_set_foreground(VgaTextModeColor::White);
    vga_printf!("\nMBI LOADED WITH : SIZE {}, RESERVED {} ::", mbi.total_size, mbi.reserved);

    vga_printf!("LOADED {} TAGS", mbi.into_iter().count());

    // test of interrupts :3
    // x86_64::instructions::interrupts::int3();
    
    // triggering very bad stuff
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // }

    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    guru::guru_panic(&info)
}
