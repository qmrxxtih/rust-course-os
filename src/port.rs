use core::arch::asm;

// Simple function to output value to given output port
pub fn output_byte(addr: u16, val: u8) {
    unsafe {
        asm!(
            "out dx,al",
            in("dx") addr,
            in("al") val
        )
    }
}

pub fn input_byte(addr: u16) -> u8 {
    let mut value:u8;
    unsafe {
        asm!(
            "in al,dx",
            in("dx") addr,
            out("al") value
        )
    }
    value
} 
