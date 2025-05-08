#![allow(unused_parens)]
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod guru;
mod interrupts;
mod keyboard;
mod multiboot;
mod paging;
mod pic;
mod port;
mod vga;

use core::panic::PanicInfo;

use interrupts::init_idt;

use multiboot::{Multiboot2, Tag};
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
\n";

const BIG_MINK_2: &str = "
           _       _                 
 _ __ ___ (_)_ __ | | __    ___  ___ 
| '_ ` _ \\| | '_ \\| |/ /   / _ \\/ __|
| | | | | | | | | |   <   | (_) \\__ \\
|_| |_| |_|_|_| |_|_|\\_\\___\\___/|___/
                      |_____|        
\n";


pub struct EmptyFrameAllocator;

use x86_64::structures::paging as Paging;


struct NormalFrameAllocator<'a> {
    mem_map: &'a [multiboot::MemoryMapEntry],
    next: u64,
}


#[allow(dead_code)]
impl<'a> NormalFrameAllocator<'a> {
    fn frame_iter(&self) -> impl Iterator<Item = Paging::PhysFrame> {
        self
            .mem_map
            .iter()
            .filter(|m| m.typ == multiboot::MemoryMapType::Available)
            .map(|m| m.base_addr..(m.base_addr+m.length))
            .flat_map(|m| m.step_by(4096))
            .map(|m| Paging::PhysFrame::containing_address(x86_64::PhysAddr::new(m)))
    }
}


unsafe impl<'a> Paging::FrameAllocator<Paging::Size4KiB> for NormalFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Paging::PhysFrame> {
        let frame = self
            .frame_iter()
            .nth(self.next as usize);
        self.next += 1;
        frame
    }
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

    // Logo print
    vga_set_foreground(VgaTextModeColor::LightMagenta);
    vga_printf!("{}", BIG_MINK_2);
    vga_set_foreground(VgaTextModeColor::White);

    vga_printf!("\nMBI LOADED WITH : SIZE {}, RESERVED {}\n", mbi.total_size, mbi.reserved);

    // maybe for later use: retrieve kernel base address
    let _load_addr = Multiboot2::from_ptr(multiboot_addr as *const u32)
        .into_iter()
        .filter_map(|x| if let Tag::ImgLoadBaseAddr(k) = x { Some(k) } else { None })
        .next()
        .expect("Kernel base address not found!");

    // retrieve memory areas identified by underlying bootloader
    let mem_map = Multiboot2::from_ptr(multiboot_addr as *const u32)
        .into_iter()
        .filter_map(|x| if let Tag::MemoryMap(m) = x { Some(m) } else { None })
        .next()
        .expect("Memory map not found!");

    // retrieve largest memory area available for use
    let largest_area = mem_map
        .iter()
        .filter(|x| multiboot::MemoryMapType::Available == x.typ)
        .max_by(|x, y| x.length.cmp(&y.length))
        .unwrap()
        ;
    vga_printf!("Largest available memory area : {:?}\n", largest_area);

    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    guru::guru_panic(&info)
}
