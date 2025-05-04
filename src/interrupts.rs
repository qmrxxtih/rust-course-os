use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{pic::{end_of_interrupt, IRQ}, vga::vga_print_char, vga_printf};


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt[IRQ::Timer as u8].set_handler_fn(timer_interrupt);
        idt[IRQ::Keyboard as u8].set_handler_fn(keyboard_interrupt);
        idt
    };
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
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
    // retrieving character scancode
    let scancode: u8 = unsafe {
        let mut p = x86_64::instructions::port::Port::new(0x60);
        p.read()
    };
    vga_printf!("{}", scancode);
    end_of_interrupt(IRQ::Keyboard);
}
