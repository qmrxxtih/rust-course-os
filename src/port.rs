
/// Simple function to read a byte from given input port
#[allow(unused)]
pub fn input_byte(addr: u16) -> u8 {
    let mut port = x86_64::instructions::port::PortReadOnly::<u8>::new(addr);
    unsafe { port.read() }
} 

/// Simple function to output a byte to given output port
#[allow(unused)]
pub fn output_byte(addr: u16, val: u8) {
    let mut port = x86_64::instructions::port::PortWriteOnly::<u8>::new(addr);
    unsafe {
        port.write(val);
    }
}

/// Simple function to read a word from given input port
#[allow(unused)]
pub fn input_word(addr: u16) -> u16 {
    let mut port = x86_64::instructions::port::PortReadOnly::<u16>::new(addr);
    unsafe { port.read() }
}

/// Simple function to write a word to given output port
#[allow(unused)]
pub fn output_word(addr: u16, val: u16) {
    let mut port = x86_64::instructions::port::PortWriteOnly::<u16>::new(addr);
    unsafe { port.write(val); }
}
