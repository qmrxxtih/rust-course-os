#![allow(unused_parens)]
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod allocator;
mod guru;
mod interrupts;
mod keyboard;
mod multiboot;
mod paging;
mod pic;
mod port;
mod vga;
mod shell;

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


#[unsafe(no_mangle)]
pub extern "C" fn mink_entry(multiboot_addr: usize) -> ! {
    // getting basic information using multiboot2 standard
    let mbi = Multiboot2::from_ptr(multiboot_addr as *const u32);
    // retrieve memory areas identified by underlying bootloader
    let mem_map = Multiboot2::from_ptr(multiboot_addr as *const u32)
        .into_iter()
        .filter_map(|x| if let Tag::MemoryMap(m) = x { Some(m) } else { None })
        .next()
        .expect("Memory map not found!");


    // Initialising interrupt vector by loading IDT (Interrupt Descriptor Table)
    init_idt();
    // Initialising PIC8259 interrupt chain
    pic::init();
    // Enabling external interrupts by calling STI (set interrupt) instruction
    x86_64::instructions::interrupts::enable();
    // initialise heap memory
    let mut mapper = paging::get_page_mapper(None);
    let mut frame_alloc = allocator::NormalFrameAllocator::new(&mem_map);
    allocator::heap_init(&mut mapper, &mut frame_alloc).expect("heap memory init failed!");

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


    // retrieve largest memory area available for use
    let largest_area = mem_map
        .iter()
        .filter(|x| multiboot::MemoryMapType::Available == x.typ)
        .max_by(|x, y| x.length.cmp(&y.length))
        .unwrap()
        ;
    vga_printf!("Largest available memory area : {:?}\n", largest_area);

    let mut vec = alloc::vec::Vec::new();
    for i in 0..5 {
        vec.push(i * 10);
    }
    vga_printf!("CONTENT OF HEAP VECTOR : {:?}\n", vec);

    vga::vga_clear_screen();
    vga_set_foreground(VgaTextModeColor::LightGreen);
    vga_printf!("{}", BIG_MINK_2);
    vga_set_foreground(VgaTextModeColor::White);
    vga_printf!("MinkOS ready. Starting shell...\n\n");

    // Initialize and run shell
    let mut shell = shell::Shell::new();
    shell.run();


    loop {}
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    vga::vga_clear_screen();
    vga_printf!("RECEIVED PANIC SIGNAL : {:?}", info);
    loop {}
    // guru::guru_panic(&info)
}
