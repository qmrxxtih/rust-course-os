use core::panic::PanicInfo;
use core::fmt::Write;

use crate::vga::{vga_clear_screen, vga_print, vga_print_char, vga_set_cursor_pos, vga_set_foreground};


struct StaticBufferWriter<'a, const N: usize> {
    output_buf: &'a mut [u8;N],
    pos: usize,
}


impl<'a, const N: usize> StaticBufferWriter<'a, N> {
    fn new(buf: &'a mut [u8;N]) -> Self {
        Self {
            output_buf: buf,
            pos: 0,
        }
    }
}


#[allow(unused)]
impl<'a, const N: usize> Write for StaticBufferWriter<'a, N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        for i in 0..N {
            if self.pos == N {
                break;
            }
            let x = bytes.get(i);
            if let Some(c) = x {
                let output_char = if *c == b'\n' {
                    b' '
                } else {
                    *c
                };
                self.output_buf[self.pos] = output_char;
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(())
    }
}


#[allow(unused)]
fn guru_init() {
    const INIT_POS:usize = 6;
    vga_clear_screen();
    vga_set_foreground(crate::vga::VgaTextModeColor::Magenta);

    for i in 7..=72 {
        vga_set_cursor_pos(Some(i), Some(INIT_POS));
        vga_print_char(b'#');
        vga_set_cursor_pos(Some(i), Some(INIT_POS + 4));
        vga_print_char(b'#');
        vga_set_cursor_pos(Some(i), Some(INIT_POS + 13));
        vga_print_char(b'#');
    }

    for i in 7..=18 {
        vga_set_cursor_pos(Some(7), Some(i));
        vga_print_char(b'#');
        vga_set_cursor_pos(Some(72), None);
        vga_print_char(b'#');
    }
    vga_set_cursor_pos(Some(9), Some(8));
    vga_print("Ouch ... this mink is now dead XnX".as_bytes());
}


#[allow(unused)]
pub fn guru_print(message: &[u8]) {
    let mut y = 0;
    'print_loop: loop {
        vga_set_cursor_pos(Some(9), Some(12 + y * 2));
        for i in 0..62 {
            let b = message.get(i + y * 62);
            if let Some(c) = b {
                vga_print_char(*c);
            } else {
                break 'print_loop;
            }
        }
        y += 1;
    }
}


#[allow(unused)]
pub fn guru_panic(info: &PanicInfo) -> ! {
    guru_init();
    // writing the panic message into buffer
    // 186 = 62 * 3 = 3 lines of 62 characters in the guru window
    let mut buf:[u8;186] = [0u8;186];
    let mut wr = StaticBufferWriter::new(&mut buf);
    write!(wr, "PANIC: {}", info.message()).unwrap();
    // writeln!(writer, "{:?}", buf).unwrap();
    // loop {}

    guru_print(&buf[..]);

    loop {}
}


#[allow(unused)]
pub fn guru_error(message: &str) -> ! {
    guru_init();

    guru_print(message.as_bytes());

    loop {}
}
