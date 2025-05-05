#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod guru;
mod interrupts;
mod keyboard;
mod multiboot;
mod pic;
mod port;
mod vga;

use core::panic::PanicInfo;

use interrupts::init_idt;

use multiboot::{Multiboot2, Tag};
use port::output_byte;
use vga::{vga_set_foreground, VgaTextModeColor};
use x86_64::VirtAddr;

const BIG_MINK: &str = "                   88             88         
                   \"\"             88         
                                  88         
88,dPYba,,adPYba,  88 8b,dPPYba,  88   ,d8   
88P\'   \"88\"    \"8a 88 88P\'   `\"8a 88 ,a8\"    
88      88      88 88 88       88 8888[      
88      88      88 88 88       88 88`\"Yba,   
88      88      88 88 88       88 88   `Y8a
\n";

fn get_active_l4_table(offset: x86_64::VirtAddr) -> &'static mut x86_64::structures::paging::PageTable {
    let (cr3, _) = {
        x86_64::registers::control::Cr3::read()
    };
    let virtual_addr = offset + cr3.start_address().as_u64();
    let table_addr: *mut x86_64::structures::paging::PageTable = virtual_addr.as_mut_ptr();
    unsafe {&mut *table_addr}
}

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

    vga_printf!("\nMBI LOADED WITH : SIZE {}, RESERVED {}\n", mbi.total_size, mbi.reserved);
    // vga_printf!("LOADED {} TAGS\n", mbi.into_iter().count());
    let load_addr = mbi.into_iter()
        .filter_map(|x| if let Tag::ImgLoadBaseAddr(k) = x { Some(k) } else { None }).next();

    if let Some(x) = load_addr {
        vga_printf!("Image base address : {:x}\n", x);
        let ph = get_active_l4_table(VirtAddr::new(x.into()));
        let mut c = 0;
        let tabcount = ph.iter().count();
        vga_printf!("Detected {} table entries\n", tabcount);
        for (i, e) in ph.iter().enumerate() {
            vga_printf!("L4 Entry #{}: {:?}\n", i, e);
            c += 1;
            if c == 10 {
                break;
            }
        }
    } else {
        vga_printf!("IMAGE BASE NOT FOUND!\n");
    }


    // for x in mbi.into_iter() {
    //     vga_printf!("{:?}\n", x);
    // }

    // test of interrupts :3
    // x86_64::instructions::interrupts::int3();
    
    // triggering very bad stuff
    // unsafe {
    //     *(0xdeadbeef as *mut u8) = 42;
    // }
    unsafe {
        let _:u8 = *(0x1002e5 as *const u8);
        vga_printf!("read ok");
        *(0x1002e5 as *mut u8) = 10;
        vga_printf!("write ok?!");
    }

    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    guru::guru_panic(&info)
}
