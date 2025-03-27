#![no_std]
#![no_main]

mod multiboot;
mod port;
mod vga;

use core::fmt::Write;
use core::panic::PanicInfo;

use multiboot::{Multiboot2, Tag};
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
pub extern "C" fn mink_entry(multiboot_addr: usize) -> ! {
    let mut writer = VgaTextModeWriter::new();
    let mbi = Multiboot2::from_ptr(multiboot_addr as *const u32);

    // Disable blinking cursor
    output_byte(0x3D4, 0x0A);
    output_byte(0x3D5, 0x20);

    writer.set_fg_color(VgaTextModeColor::LightMagenta);
    writer.write_text(BIG_MINK.as_bytes());
    writer.set_fg_color(VgaTextModeColor::White);

    writeln!(
        writer,
        "\nMBI LOADED WITH : SIZE {}, RESERVED {} ::",
        mbi.total_size, mbi.reserved
    )
    .unwrap();

    for tag in mbi {
        writeln!(writer, "{:?}", tag).unwrap();
        if let Tag::ElfSymbols(mut es) = tag {
            for section in es {
                if (section.size != 0) {
                    writeln!(writer, "{:?}", section);
                }
            }
        }
    }

    loop {}
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    let mut panic_writer = VgaTextModeWriter::new();
    // panic_writer.clear_screen();
    panic_writer.set_fg_color(VgaTextModeColor::Red);
    panic_writer.write_text(b"[ERROR] PANIC ENCOUNTERED");

    loop {}
}
