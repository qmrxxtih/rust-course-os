use lazy_static::lazy_static;
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode
};

use crate::{keyboard::Key, pic::{end_of_interrupt, IRQ}, vga::vga_print_char, vga_printf};


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[IRQ::Timer as u8].set_handler_fn(timer_interrupt);
        idt[IRQ::Keyboard as u8].set_handler_fn(keyboard_interrupt);
        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, err: PageFaultErrorCode) {
    // Control Register 2 stores address that was accessed and caused a page fault
    let cr2 = {
        x86_64::registers::control::Cr2::read()
    };
    vga_printf!("PAGE FAULT CAUSED BY {:?}, error {:?}\n", cr2, err);
    vga_printf!("Stack trace : {:?}", stack_frame);
    loop {}
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT {} :: {:#?}", error_code, stack_frame);
}

pub fn init_idt() {
    IDT.load();
}


extern "x86-interrupt" fn timer_interrupt(stack_frame: InterruptStackFrame) {
    vga_print_char(b'.');
    end_of_interrupt(IRQ::Timer);
}

extern "x86-interrupt" fn keyboard_interrupt(stack_frame: InterruptStackFrame) {
    // retrieving character scancode from PS/2 keyboard port
    let scancode: u8 = unsafe {
        let mut p = x86_64::instructions::port::Port::new(0x60);
        p.read()
    };
    // adding key to key buffer
    crate::keyboard::_push_key(scancode);
    // try translating key buffer into key
    if let Some(k) = crate::keyboard::translate_key() {
        if let Key::Char(c) = k {
            vga_printf!("{}", c as char);
        } else {
            vga_printf!("Special {:?}", k);
        }
    }
    // signal end of interrupt
    end_of_interrupt(IRQ::Keyboard);
}
