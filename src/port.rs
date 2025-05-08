
/// Simple function to output a byte to given output port
#[allow(unused)]
pub fn output_byte(addr: u16, val: u8) {
    let mut port = x86_64::instructions::port::PortWriteOnly::<u8>::new(addr);
    unsafe {
        port.write(val);
    }
}

/// Simple function to read a byte from given input port
#[allow(unused)]
pub fn input_byte(addr: u16) -> u8 {
    let mut port = x86_64::instructions::port::PortReadOnly::<u8>::new(addr);
    unsafe { port.read() }
} 
