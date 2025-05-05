
use lazy_static::lazy_static;

#[derive(Clone, Copy, Debug)]
pub enum Key {
    Backspace,
    Tab,
    Enter,
    Escape,
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    CapsLock,
    LeftAlt,
    RightAlt,
    Char(u8),
}

lazy_static! {
    static ref KEY_BUFFER: spin::Mutex<[u8;2]> = spin::Mutex::new([0u8;2]);
}

pub fn _push_key(c: u8) {
    let mut lock = KEY_BUFFER.lock();
    if lock[0] == 0xe0 {
        lock[1] = c;
    }
    else {
        lock[0] = c;
    }
}

pub fn translate_key() -> Option<Key> {
    let mut lock = KEY_BUFFER.lock();
    let k = match lock.as_slice() {
        // we AND with 0x7F, which will disable the most significant bit (press / release
        // indication)
        &[0xe0, x] => match x & 0x7f {
            // escaped key not registered, return without further action
            0x00 => return None,
            // TODO match escaped characters
            _ => None,
        },
        // we AND with 0x7F, which will disable the most significant bit (press / release
        // indication)
        &[x, _] => match x & 0x7f {
            0x01 => Some(Key::Escape),
            0x02 => Some(Key::Char(b'1')),
            0x03 => Some(Key::Char(b'2')),
            0x04 => Some(Key::Char(b'3')),
            0x05 => Some(Key::Char(b'4')),
            0x06 => Some(Key::Char(b'5')),
            0x07 => Some(Key::Char(b'6')),
            0x08 => Some(Key::Char(b'7')),
            0x09 => Some(Key::Char(b'8')),
            0x0A => Some(Key::Char(b'9')),
            0x0B => Some(Key::Char(b'0')),
            0x0C => Some(Key::Char(b'-')),
            0x0D => Some(Key::Char(b'=')),
            0x0E => Some(Key::Backspace),
            0x0F => Some(Key::Tab),
            0x10 => Some(Key::Char(b'q')),
            0x11 => Some(Key::Char(b'w')),
            0x12 => Some(Key::Char(b'e')),
            0x13 => Some(Key::Char(b'r')),
            0x14 => Some(Key::Char(b't')),
            0x15 => Some(Key::Char(b'y')),
            0x16 => Some(Key::Char(b'u')),
            0x17 => Some(Key::Char(b'i')),
            0x18 => Some(Key::Char(b'o')),
            0x19 => Some(Key::Char(b'p')),
            0x1A => Some(Key::Char(b'[')),
            0x1B => Some(Key::Char(b']')),
            0x1C => Some(Key::Enter),
            0x1D => Some(Key::LeftCtrl),
            0x1E => Some(Key::Char(b'a')),
            0x1F => Some(Key::Char(b's')),
            0x20 => Some(Key::Char(b'd')),
            0x21 => Some(Key::Char(b'f')),
            0x22 => Some(Key::Char(b'g')),
            0x23 => Some(Key::Char(b'h')),
            0x24 => Some(Key::Char(b'j')),
            0x25 => Some(Key::Char(b'k')),
            0x26 => Some(Key::Char(b'l')),
            0x27 => Some(Key::Char(b';')),
            0x28 => Some(Key::Char(b'\'')),
            0x29 => Some(Key::Char(b'`')),
            0x2A => Some(Key::LeftShift),
            0x2B => Some(Key::Char(b'\\')),
            0x2C => Some(Key::Char(b'z')),
            0x2D => Some(Key::Char(b'x')),
            0x2E => Some(Key::Char(b'c')),
            0x2F => Some(Key::Char(b'v')),
            0x30 => Some(Key::Char(b'b')),
            0x31 => Some(Key::Char(b'n')),
            0x32 => Some(Key::Char(b'm')),
            0x33 => Some(Key::Char(b',')),
            0x34 => Some(Key::Char(b'.')),
            0x35 => Some(Key::Char(b'/')),
            0x36 => Some(Key::RightShift),
            0x37 => Some(Key::Char(b'*')),
            0x38 => Some(Key::LeftAlt),
            0x39 => Some(Key::Char(b' ')),
            0x3A => Some(Key::CapsLock),
            _ => None,
        },
        _ => None,
    };
    lock[0] = 0x00;
    lock[1] = 0x00;
    k
}

