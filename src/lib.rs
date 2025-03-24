#![no_std]
#![no_main]

use core::panic::PanicInfo;


#[unsafe(no_mangle)]
pub extern "C" fn mink_entry() -> ! {
    clear_vga();
    write_vga(0, b'h', 0x04);
    write_vga(2, b'e', 0x0c);
    write_vga(4, b'l', 0x0e);
    write_vga(6, b'l', 0x0a);
    write_vga(8, b'o', 0x03);
    write_vga(10, b'_', 0x01);
    write_vga(12, b'w', 0x05);
    write_vga(14, b'o', 0x0d);
    write_vga(16, b'r', 0x01);
    write_vga(18, b'l', 0x0f);
    write_vga(20, b'd', 0x0f);
    write_vga(22, b' ', 0x00);
    write_vga(24, b':', 0x0b);
    write_vga(26, b'3', 0x0b);

    loop {}
}



#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}

fn write_vga(offset: usize, character: u8, attrib: u8) {
    let attr_ptr:*mut u8 = (0xb8001 + offset) as *mut u8;
    let char_ptr:*mut u8 = (0xb8000 + offset) as *mut u8;

    unsafe {
        *attr_ptr = attrib;
        *char_ptr = character;
    }
}

fn clear_vga() {
    let ptr:*mut u8 = 0xb8000 as *mut u8;
    for i in 0..25 {
        for j in 0..80 {
            unsafe {
                let offset = j * 2 + i * 80;
                *ptr.wrapping_add(offset) = 0x00;
                *ptr.wrapping_add(offset + 1) = 0x00;
            }
        }
    }
}

