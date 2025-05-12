use pic8259::ChainedPics;


const PIC1_OFFSET: u8 = 32;
const PIC2_OFFSET: u8 = 40;

static CHAINED_PICS: spin::Mutex<pic8259::ChainedPics> = spin::Mutex::new(unsafe {ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET)});

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum IRQ {
    Timer = PIC1_OFFSET,
    Keyboard,
    Cascade,
    COM2,
    COM1,
    LPT2,
    FloppyDisk,
    LPT1,
    CMOSClock,
    Nic1,
    Nic2,
    Nic3,
    Mouse,
    Coprocessor,
    PrimaryATA,
    SecondaryATA,
}

pub fn init() {
    unsafe {
        CHAINED_PICS.lock().initialize();
    }
}

pub fn end_of_interrupt(irq: IRQ) {
    unsafe {
        CHAINED_PICS.lock().notify_end_of_interrupt(irq as u8);
    }
}
