
/// ATA PIO module.
pub mod pio {
    use crate::{port::{input_byte, input_word, output_byte, output_word}, vga_printf};

    // [R/W] data register offset
    const DISK_DATA_REGISTER: u16 = 0;
    // [R] error register offset
    const DISK_ERROR_REGISTER: u16 = 1;
    // [W] features register offset
    const DISK_FEATURES_REGISTER: u16 = 1;
    // [R/W] sector count register offset
    const DISK_SECTOR_COUNT_REGISTER: u16 = 2;
    // [R/W] low byte of LBA (sector number) register offset
    const DISK_LBA_LOW_REGISTER: u16 = 3;
    // [R/W] middle byte of LBA (cylinder low) register offset
    const DISK_LBA_MID_REGISTER: u16 = 4;
    // [R/W] high byte of LBA (cylinder high) register offset
    const DISK_LBA_HIGH_REGISTER: u16 = 5;
    // [R/W] drive/head select register, highest order bits of LBA addressing
    const DISK_DRIVE_HEAD_REGISTER: u16 = 6;
    // [R] disk status register offset
    const DISK_STATUS_REGISTER: u16 = 7;
    // [W] disk command register offset
    const DISK_COMMAND_REGISTER: u16 = 7;

    // [R] alternate status register offset (does not affect interrupts)
    const CONTROL_ALTERNATE_STATUS_REGISTER: u16 = 0;
    // [W] device control register offset
    const CONTROL_CONTROL_REGISTER: u16 = 0;
    // [R] drive address information register offset
    const CONTROL_ADDRESS_REGISTER: u16 = 1;



    const AddressMarkNotFound: u8 = 1;
    const TrackZeroNotFound: u8 = 2;
    const Aborted: u8 = 4;
    const MediaChangeRequest: u8 = 8;
    const IDNotFound: u8 = 16;
    const MediaChanged: u8 = 32;
    const UncorrectableDataError: u8 = 64;
    const BadBlock: u8 = 128;


    const LBA_HIGH: u8 = 0x07;
    const DRIVE_SELECT: u8 = 16;
    const USE_LBA: u8 = 64;


    const STATUS_ERROR: u8 = 1;
    // ALWAYS 0
    const STATUS_INDEX: u8 = 2;
    // ALWAYS 0
    const STATUS_CORRECTED: u8 = 4;
    const STATUS_DRQ: u8 = 8;
    const STATUS_OVERLAPPED_SERVICE_MODE: u8 = 16;
    const STATUS_DRIVE_FAULT: u8 = 32;
    const STATUS_READY: u8 = 64;
    const STATUS_BUSY: u8 = 128;


    /// Stores information about disk's I/O port addresses.
    #[derive(Clone, Copy, Debug)]
    pub struct DiskPort {
        base: u16,
        ctrl: u16,
    }


    /// Disk information structure.
    #[derive(Clone, Copy, Debug)]
    pub struct DiskInfo {
        identify_buffer: [u16;256],
    }


    impl DiskInfo {
        /// Tries to create a new disk information structure using ATA IDENTIFY command.
        pub fn identify(disk_port: DiskPort, use_slave: bool) -> Option<Self> {
            let mut buf = [0;256];
            if identify(disk_port, use_slave, &mut buf) {
                Some(Self {identify_buffer: buf})
            } else {
                None
            }
        }

        /// Returns whether identified device is a disk or not.
        pub fn is_disk(&self) -> bool {
            self.identify_buffer[0] != 0
        }

        /// Returns whether identified device supports LBA addressing or not.
        pub fn lba_support(&self) -> bool {
            self.identify_buffer[83] & (1 << 10) != 0
        }

        /// Returns device's maximum addressable sectors using LBA28 addressing.
        pub fn lba28_sectors(&self) -> u32 {
            unsafe { *([self.identify_buffer[60], self.identify_buffer[61]].as_ptr() as *const u32) }
        }

        /// Returns device's maximum addressable sectors using LBA48 addressing.
        pub fn lba48_sectors(&self) -> u64 {
            unsafe { *([self.identify_buffer[100], self.identify_buffer[101], self.identify_buffer[102], self.identify_buffer[103]].as_ptr() as *const u64) }
        }
    }


    impl Default for DiskPort {
        /// Returns disk port information referencing primary ATA bus.
        fn default() -> Self {
            Self {
                base: 0x1f0,
                ctrl: 0x3f6,
            }
        }
    }


    impl DiskPort {
        /// Returns disk port information referencing secondary ATA bus.
        pub fn secondary() -> Self {
            Self {
                base: 0x170,
                ctrl: 0x376,
            }
        }
    }


    /// Sends an IDENTIFY command to the ATA controller. Returns true if IDENTIFY executed
    /// successfully, returning contents into the output buffer.
    pub fn identify(disk_port: DiskPort, use_slave: bool, output_buffer: &mut [u16]) -> bool {
        let select = disk_port.base + DISK_DRIVE_HEAD_REGISTER;
        let cmd_status = disk_port.base + DISK_COMMAND_REGISTER;
        output_byte(select, if use_slave { 0xf0 } else { 0xe0 });
        // set sector count, LBA low, LBA mid, LBA high to 0
        output_byte(disk_port.base + DISK_SECTOR_COUNT_REGISTER, 0x00);
        output_byte(disk_port.base + DISK_LBA_LOW_REGISTER, 0x00);
        output_byte(disk_port.base + DISK_LBA_MID_REGISTER, 0x00);
        output_byte(disk_port.base + DISK_LBA_HIGH_REGISTER, 0x00);
        // send the IDENTIFY command
        output_byte(cmd_status, 0xec);
        // read result of the operation
        let result = input_byte(cmd_status);
        // if 0, drive does not exist - return
        if result == 0x00 {
            return false;
        }
        // poll status register until BUSY flag clears.
        loop {
            let b = input_byte(cmd_status);
            if b & STATUS_BUSY == 0x00 { break; }
        }
        // check values in LBA  mid and LBA high registers, if non-0, drive is not ATA - return
        let m = input_byte(disk_port.base + DISK_LBA_MID_REGISTER);
        let h = input_byte(disk_port.base + DISK_LBA_HIGH_REGISTER);
        if m != 0x00 || h != 0x00 {
            return false;
        }
        // wait until READY (or in bad case ERR) flag goes high
        loop {
            let b = input_byte(cmd_status);
            let flags = STATUS_READY | STATUS_ERROR;
            if b & flags == STATUS_READY {
                // data is ready - read it into output buffer
                for i in 0..256 {
                    let data = input_word(disk_port.base + DISK_DATA_REGISTER);
                    // safety first - check if index is not out of range before writing
                    if let Some(out) = output_buffer.get_mut(i) {
                        *out = data;
                    }
                }
                return true;
            }
            else if b & flags == STATUS_ERROR {
                return false;
            }
        }
    }


    pub fn poll_status(disk_port: DiskPort) {
        loop {
            let s = input_byte(disk_port.ctrl + CONTROL_ALTERNATE_STATUS_REGISTER);
            let mask = STATUS_READY | STATUS_BUSY;
            // check if READY flag is the only one set
            if s & mask == STATUS_READY { break; }
            if s & STATUS_ERROR != 0x00 {
                panic!("DRIVE ERROR!\n");
            }
            if s & STATUS_DRIVE_FAULT != 0x00 {
                panic!("DRIVE FAULT!!!\n");
            }
        }
    }


    /// Performs "software reset" of ATA bus.
    pub unsafe fn soft_reset(disk_port: DiskPort) {
        poll_status(disk_port);
        let control = disk_port.ctrl + CONTROL_CONTROL_REGISTER;
        // send software reset command to the disk
        output_byte(control, 0x04);
        // perform reset on the bus
        output_byte(control, 0x00);
        // invoke "fake" delay for disk registers to reset
        for _ in 0..4 { _ = input_byte(control); }
        // wait until BUSY flag goes low and READY flag goes high
        poll_status(disk_port);
    }


    /// Reads data from given LBA28 address.
    /// Returns true if read was successful.
    pub unsafe fn read_sector_lba28(disk_port: DiskPort, use_slave: bool, address: u32, output_buffer: &mut [u16]) -> bool {
        let op = |y: &mut u16| {
            *y = input_word(disk_port.base + DISK_DATA_REGISTER);
        };
        unsafe {
            operate_sector_lba28(0x20, op, disk_port, use_slave, address, output_buffer)
        }
    }


    /// Write data to give LBA28 address.
    /// Returns true if write was successful.
    pub unsafe fn write_sector_lba28(disk_port: DiskPort, use_slave: bool, address: u32, input_buffer: &mut [u16]) -> bool {
        let op = |y: &mut u16| {
            output_word(disk_port.base + DISK_DATA_REGISTER, *y);
        };
        unsafe {
            operate_sector_lba28(0x30, op, disk_port, use_slave, address, input_buffer)
        }
    }

    /// Universal function for PIO R/W operation with LBA28 addressing.
    unsafe fn operate_sector_lba28<F: Fn(&mut u16)>(op: u8, op_func: F, disk_port: DiskPort, use_slave: bool, address: u32, output_buffer: &mut [u16]) -> bool {
        // check if disk is ready, otherwise reset before operation
        let stat = input_byte(disk_port.ctrl + CONTROL_ALTERNATE_STATUS_REGISTER);
        let flags = STATUS_DRQ | STATUS_BUSY;
        if stat & flags != 0x00 {
            unsafe { soft_reset(disk_port); }
        }
        // set sector count
        output_byte(disk_port.base + DISK_SECTOR_COUNT_REGISTER, 0x01);
        // set LBA address, byte by byte
        output_byte(disk_port.base + DISK_LBA_LOW_REGISTER, (address & 0xff) as u8);
        output_byte(disk_port.base + DISK_LBA_MID_REGISTER, ((address & 0xff00) >> 8) as u8);
        output_byte(disk_port.base + DISK_LBA_HIGH_REGISTER, ((address & 0xff0000) >> 16) as u8);
        // last 4 bits of LBA address go to the select register
        let high_bits = ((address & 0xf000000) >> 24) as u8;
        let select = if use_slave { 0xf0 } else { 0xe0 };
        output_byte(disk_port.base + DISK_DRIVE_HEAD_REGISTER, high_bits | select);
        // send read command
        output_byte(disk_port.base + DISK_COMMAND_REGISTER, op);

        // skip first 4 error reports, wait until READY
        let mut x = 0;

        loop {
            let b = input_byte(disk_port.base + DISK_STATUS_REGISTER);
            // skip if BUSY is set
            if b & STATUS_DRIVE_FAULT == 0x00 { 
                // if DRQ is set, data is ready to read
                if b & STATUS_READY != 0x00 {
                    break;
                }
            }
            // first 4 cycles are over, we can now also check error bit
            if x >= 4 {
                if b & STATUS_ERROR != 0x00 || b & STATUS_DRIVE_FAULT != 0x00 {
                    return false;
                }
            }
            x += 1;
        }
        // data is ready to be read - read 2B * 256 = 512B (1 sector)
        for i in 0..256 {
            if let Some(out) = output_buffer.get_mut(i) {
                op_func(out);
            }
        }
        // wait for status registers
        poll_status(disk_port);
        // we are finished!
        true
    }
}
