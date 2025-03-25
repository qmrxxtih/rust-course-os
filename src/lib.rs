#![no_std]
#![no_main]

mod vga;

use core::arch::asm;
use core::panic::PanicInfo;
use vga::{VgaTextModeColor, VgaTextModeWriter};

const BIG_MINK: &str = "                   88             88         
                   \"\"             88         
                                  88         
88,dPYba,,adPYba,  88 8b,dPPYba,  88   ,d8   
88P\'   \"88\"    \"8a 88 88P\'   `\"8a 88 ,a8\"    
88      88      88 88 88       88 8888[      
88      88      88 88 88       88 88`\"Yba,   
88      88      88 88 88       88 88   `Y8a
";

// Simple function to output value to given output port
fn output_byte(addr: u16, val: u8) {
    unsafe {
        asm!(
            "out dx,al",
            in("dx") addr,
            in("al") val
        )
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mink_entry() -> ! {
    // Disable blinking cursor
    output_byte(0x3D4, 0x0A);
    output_byte(0x3D5, 0x20);
    // Create new VGA writer
    let mut writer = VgaTextModeWriter::new();
    // Set output text color attributes
    writer.set_attrib(VgaTextModeColor::Black, VgaTextModeColor::LightMagenta);
    // Write to VGA buffer
    writer.write_text(BIG_MINK.as_bytes());

    loop {}
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
