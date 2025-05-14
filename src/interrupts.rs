use lazy_static::lazy_static;
use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode
};

use crate::{keyboard::Key, pic::{end_of_interrupt, IRQ}, vga_printf};


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[IRQ::Timer as u8].set_handler_fn(timer_interrupt);
        idt[IRQ::Keyboard as u8].set_handler_fn(keyboard_interrupt);
        idt[IRQ::PrimaryATA as u8].set_handler_fn(ata_prim_handler);
        idt[IRQ::SecondaryATA as u8].set_handler_fn(ata_sec_handler);
        idt
    };
}


pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn ata_sec_handler(_stack_frame: InterruptStackFrame) {
    end_of_interrupt(IRQ::SecondaryATA);
}

extern "x86-interrupt" fn ata_prim_handler(_stack_frame: InterruptStackFrame) {
    end_of_interrupt(IRQ::PrimaryATA);
}


extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
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


extern "x86-interrupt" fn timer_interrupt(_stack_frame: InterruptStackFrame) {
    // vga_print_char(b'.');
    end_of_interrupt(IRQ::Timer);
}


extern "x86-interrupt" fn keyboard_interrupt(_stack_frame: InterruptStackFrame) {
    let scancode: u8 = unsafe {
        let mut p = x86_64::instructions::port::Port::new(0x60);
        p.read()
    };
    crate::keyboard::_push_key(scancode);
    end_of_interrupt(IRQ::Keyboard);
}
