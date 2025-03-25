const VGA_TEXT_MODE_WIDTH: usize = 80;
const VGA_TEXT_MODE_HEIGHT: usize = 25;
const VGA_TEXT_ADDR: usize = 0xb8000;

#[allow(unused)]
pub struct VgaTextModeWriter {
    pos_x: usize,
    pos_y: usize,
    current_attrib: u8,
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
    pub const fn new() -> Self {
        Self {
            pos_x: 0,
            pos_y: 0,
            current_attrib: 0x0f, // Makes all characters defaultly white foreground + black
                                  // background, no blinking cursor
        }
    }

    /// Clears whole screen.
    pub fn clear_screen(&self) {
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
    pub fn clear_line(&self, row: usize) {
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
    pub fn set_fg_color(&mut self, color: VgaTextModeColor) {
        let b = color.as_u8();
        // Apply to current attribute value, keeping upper part (background color) and upper bit (blinking)
        self.current_attrib = (self.current_attrib & 0xf0) | b;
    }

    /// Sets background color of next printed text.
    pub fn set_bg_color(&mut self, color: VgaTextModeColor) {
        let b = color.as_u8() << 4;
        // Apply to current attribute value, keeping the lower part (foreground color) and upper bit (blinking)
        self.current_attrib = (self.current_attrib & 0x8f) | b;
    }

    /// Sets all attributes to new values.
    pub fn set_attrib(
        &mut self,
        bg_color: VgaTextModeColor,
        fg_color: VgaTextModeColor,
    ) {
        self.set_fg_color(fg_color);
        self.set_bg_color(bg_color);
    }

    /// Scrolls text by given ammount.
    pub fn scroll_by(&self, count: usize) {
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
    pub fn write_char(&mut self, c: u8) {
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

    pub fn write_text(&mut self, text: &[u8]) {
        for c in text {
            self.write_char(*c);
        }
    }
}
