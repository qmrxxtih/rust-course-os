use lazy_static::lazy_static;

const VGA_TEXT_MODE_WIDTH: usize = 80;
const VGA_TEXT_MODE_HEIGHT: usize = 25;
const VGA_TEXT_ADDR: usize = 0xb8000;


#[allow(unused)]
pub struct VgaTextModeWriter {
    pos_x: usize,
    pos_y: usize,
    current_attrib: u8,
}

impl core::fmt::Write for VgaTextModeWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_text(s.as_bytes());
        Ok(())
    }
}

lazy_static! {
    static ref VGA:spin::Mutex<VgaTextModeWriter> = spin::Mutex::new(VgaTextModeWriter::new());
}

/// Prints text to VGA buffer using global VGA writer instance.
pub fn vga_print(text: &[u8]) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        VGA.lock().write_text(text);
    })
}

/// Prints single character to VGA buffer using global VGA writer instance.
pub fn vga_print_char(c: u8) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        VGA.lock().write_char(c);
    })
}

/// DO NOT USE: Private print function for the macro
pub fn ghost_print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    x86_64::instructions::interrupts::without_interrupts(|| {
        VGA.lock().write_fmt(args).unwrap();
    });
}

/// Sets global VGA writer's foreground text color.
pub fn vga_set_foreground(color: VgaTextModeColor) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        VGA.lock().set_fg_color(color);
    });
}

/// Sets global VGA writer's background text color.
pub fn vga_set_background(color: VgaTextModeColor) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        VGA.lock().set_bg_color(color);
    });
}

/// Clears screen using global VGA writer.
pub fn vga_clear_screen() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        VGA.lock().clear_screen();
    });
}

/// Sets global VGA writer's cursor position.
pub fn vga_set_cursor_pos(x: Option<usize>, y: Option<usize>) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        VGA.lock().set_cursor(x, y);
    });
}

#[macro_export]
macro_rules! vga_printf {
    ($($arg:tt)*) => {
        (crate::vga::ghost_print(format_args!($($arg)*)))
    };
}

#[allow(unused)]
pub enum VgaTextModeColor {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Orange,
    LightGray,
    Gray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightOrange,
    LightMagenta,
    LightYellow,
    White,
}

#[allow(unused)]
impl VgaTextModeColor {
    pub const fn as_u8(&self) -> u8 {
        match &self {
            Self::Black => 0x00,
            Self::Blue => 0x01,
            Self::Green => 0x02,
            Self::Cyan => 0x03,
            Self::Red => 0x04,
            Self::Magenta => 0x05,
            Self::Orange => 0x06,
            Self::LightGray => 0x07,
            Self::Gray => 0x08,
            Self::LightBlue => 0x09,
            Self::LightGreen => 0x0a,
            Self::LightCyan => 0x0b,
            Self::LightOrange => 0x0c,
            Self::LightMagenta => 0x0d,
            Self::LightYellow => 0x0e,
            Self::White => 0x0f,
        }
    }
}

#[allow(unused)]
impl VgaTextModeWriter {
    /// Create new VGA text mode writer for writing information onto screen.
    /// New writer's position is set to 0:0 (upper left corner).
    /// Default text attribute is white foreground with black background and no blinking cursor.
    fn new() -> Self {
        Self {
            pos_x: 0,
            pos_y: 0,
            current_attrib: 0x0f, // Makes all characters defaultly white foreground + black
                                  // background, no blinking cursor
        }
    }

    /// Clears whole screen.
    fn clear_screen(&self) {
        for y in 0..VGA_TEXT_MODE_HEIGHT {
            for x in 0..VGA_TEXT_MODE_WIDTH {
                unsafe {
                    *((VGA_TEXT_ADDR + 2 * x + y * VGA_TEXT_MODE_WIDTH * 2)
                        as *mut u16) = 0x0000;
                }
            }
        }
    }

    /// Clears given row.
    fn clear_line(&self, row: usize) {
        if row >= VGA_TEXT_MODE_HEIGHT {
            // TODO report error
            return;
        }
        for x in 0..VGA_TEXT_MODE_WIDTH {
            unsafe {
                *((VGA_TEXT_ADDR + 2 * x + row * VGA_TEXT_MODE_WIDTH * 2)
                    as *mut u16) = 0x0000;
            }
        }
    }

    /// Sets foreground color of next printed text.
    fn set_fg_color(&mut self, color: VgaTextModeColor) {
        let b = color.as_u8();
        // Apply to current attribute value, keeping upper part (background color) and upper bit (blinking)
        self.current_attrib = (self.current_attrib & 0xf0) | b;
    }

    /// Sets background color of next printed text.
    fn set_bg_color(&mut self, color: VgaTextModeColor) {
        let b = color.as_u8() << 4;
        // Apply to current attribute value, keeping the lower part (foreground color) and upper bit (blinking)
        self.current_attrib = (self.current_attrib & 0x8f) | b;
    }

    /// Sets all attributes to new values.
    fn set_attrib(
        &mut self,
        bg_color: VgaTextModeColor,
        fg_color: VgaTextModeColor,
    ) {
        self.set_fg_color(fg_color);
        self.set_bg_color(bg_color);
    }

    /// Scrolls text by given ammount.
    fn scroll_by(&self, count: usize) {
        // If count is more than or equal to VGA text mode height, simply clear screen and exit.
        if count >= VGA_TEXT_MODE_HEIGHT {
            self.clear_screen();
        } else {
            // Calculate how many rows should be shifted
            let num_rows_shifted = VGA_TEXT_MODE_HEIGHT - count;
            for y in 0..num_rows_shifted {
                for x in 0..VGA_TEXT_MODE_WIDTH {
                    // Copy character from following line into current line.
                    unsafe {
                        let c = *((VGA_TEXT_ADDR
                            + (y + count) * VGA_TEXT_MODE_WIDTH * 2
                            + x * 2)
                            as *const u16);
                        *((VGA_TEXT_ADDR + y * VGA_TEXT_MODE_WIDTH * 2 + x * 2)
                            as *mut u16) = c;
                    }
                }
            }
            // Clear following lines
            for y in num_rows_shifted..VGA_TEXT_MODE_HEIGHT {
                self.clear_line(y);
            }
        }
    }

    /// Puts character onto screen on writer's position.
    /// If new character would go out of row (current X position >= VGA_TEXT_MODE_WIDTH),
    /// cursor is moved to beginning of next line.
    /// If new character would go out of column (current Y position >= VGA_TEXT_MODE_HEIGHT),
    /// entire screen is scrolled.
    fn write_char(&mut self, c: u8) {
        if self.pos_x >= VGA_TEXT_MODE_WIDTH {
            // Go to next line's beginning
            self.pos_y += 1;
            self.pos_x = 0;
        }
        if self.pos_y >= VGA_TEXT_MODE_HEIGHT {
            // Scroll by number of columns out of range
            let scroll = self.pos_y - VGA_TEXT_MODE_HEIGHT + 1;
            self.scroll_by(scroll);
            self.pos_y -= scroll;
        }
        let offset = 2 * self.pos_y * VGA_TEXT_MODE_WIDTH + 2 * self.pos_x;
        let attr_ptr: *mut u8 = (VGA_TEXT_ADDR + 1 + offset) as *mut u8;
        let char_ptr: *mut u8 = (VGA_TEXT_ADDR + offset) as *mut u8;

        if c == b'\n' {
            self.pos_y += 1;
            self.pos_x = 0;
        } else {
            unsafe {
                *attr_ptr = self.current_attrib;
                *char_ptr = c;
            }
            self.pos_x += 1;
        }
    }

    fn write_text(&mut self, text: &[u8]) {
        for c in text {
            self.write_char(*c);
        }
    }

    /// Moves cursor to specified X and Y if provided.
    fn set_cursor(&mut self, x: Option<usize>, y: Option<usize>) -> bool {
        if let Some(px) = x {
            if px < VGA_TEXT_MODE_WIDTH {
                self.pos_x = px;
            } else {
                return false;
            }
        }
        if let Some(py) = y {
            if py < VGA_TEXT_MODE_HEIGHT {
                self.pos_y = py;
            } else {
                return false;
            }
        }
        return true;
    }
}
