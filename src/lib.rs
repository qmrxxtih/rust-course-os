#![no_std]
#![no_main]

mod port;
mod vga;


use core::panic::PanicInfo;
use port::output_byte;
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
