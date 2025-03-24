#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}


#[unsafe(no_mangle)]
pub extern "C" fn write_vga(offset: usize, character: u8, attrib: u8) {
    let ptr:*mut u8 = 0xb8000 as *mut u8;

    unsafe {
        *ptr.wrapping_add(offset) = attrib;
        *ptr.wrapping_add(offset + 1) = character;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn clear_vga() {
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

#[unsafe(no_mangle)]
pub extern "C" fn mink_enter() -> ! {
    //clear_vga();
    write_vga(0, b'H', 0x0b);
    write_vga(2, b'e', 0x0d);
    write_vga(4, b'l', 0x02);
    write_vga(6, b'l', 0x01);
    write_vga(8, b'o', 0x05);
    write_vga(10, b' ', 0x00);
    write_vga(12, b':', 0x0b);
    write_vga(14, b'3', 0x0b);

    loop {}
}

