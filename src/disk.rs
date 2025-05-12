

/// ATA PIO module.
pub mod ata {
    use x86_64::instructions::port::Port;

    use crate::{port::{input_byte, output_byte}, vga_printf};

    /// Type of device connected on ATA bus.
    #[derive(Debug)]
    pub enum DevType {
        ATADEV_PATAPI,
        ATADEV_SATAPI,
        ATADEV_PATA,
        ATADEV_SATA,
        Unknown,
    }

    /// Structure with ports for PIO ATA device control.
    #[allow(unused)]
    pub struct DeviceControl {
        // base device register
        base: u16,
        // device control register
        dev_ctrl: u16,
        // device select register
        reg_dev_sel: u16,
    }

    impl Default for DeviceControl {
        fn default() -> Self {
            Self {
                base: 0x1f0,
                dev_ctrl: 0x3f6,
                reg_dev_sel: 0x06,
            }
        }
    }

    pub fn identify(slave: bool, out_buf: &mut [u16;256]) -> Option<()> {
        // write to drive select register
        output_byte(0x1f6, if slave { 0xb0 } else { 0xa0 });
        // reset registers 0x1f2 to 0x1f5 (sector count, LBA lower, LBA middle, LBA higher)
        for i in 0..=3 {
            output_byte(0x1f2 + i, 0x00);
        }
        // write IDENTIFY command to status register port
        output_byte(0x1f7, 0xec);
        // read status register port
        let result = input_byte(0x1f7);
        if result == 0x00 {
            None
        } else {
            // await clearing of bit 7 (BUSY)
            loop {
                // check the BUSY bit
                if (input_byte(0x1f7) & 0x80 == 0x00) { break; }
            }
            // if LBA middle and LBA higher are set, drive is not ATA - end
            let mid = input_byte(0x1f4);
            let high = input_byte(0x1f5);
            if mid != 0x00 || high != 0x00 {
                return None;
            }
            loop {
                let b = input_byte(0x1f7);
                // wait until DRQ bit (or ERR bit) is set
                if b & 0x08 != 0x00 {
                    // data is ready at this point, read it
                    let mut port = Port::<u16>::new(0x1f0);
                    for i in 0..256 {
                        unsafe {
                            out_buf[i] = port.read();
                        }
                    }
                    return Some(())
                }
                // if error is set, IDENTIFY failed 
                if b & 0x01 != 0x00 {
                    vga_printf!("ERR set!\n");
                    return None;
                }
            }
        }
    }

    pub fn get_device_type(slave: bool, dev_ctrl: DeviceControl) -> DevType {
        // wait until master is ready
        soft_reset(dev_ctrl.dev_ctrl);

        output_byte(dev_ctrl.base + dev_ctrl.reg_dev_sel, if slave { 0xb0 } else { 0xa0 });
        // wait 4x (circa 400 ns) for drive select to work
        _ = input_byte(dev_ctrl.dev_ctrl);
        _ = input_byte(dev_ctrl.dev_ctrl);
        _ = input_byte(dev_ctrl.dev_ctrl);
        _ = input_byte(dev_ctrl.dev_ctrl);

        // 0x04 = REGISTER_CYLINDER_LOW
        let cl = input_byte(dev_ctrl.base + 0x04);
        // 0x05 = REGISTER_CYLINDER_HIGH
        let ch = input_byte(dev_ctrl.base + 0x05);

        match (cl, ch) {
            (0x14, 0xeb) => DevType::ATADEV_PATAPI,
            (0x69, 0x96) => DevType::ATADEV_SATAPI,
            (0x00, 0x00) => DevType::ATADEV_PATA,
            (0x3c, 0xc3) => DevType::ATADEV_SATA,
            _ => DevType::Unknown,
        }
    }


    /// Reads data from ATA disk, single tasking way.
    pub fn read(sectors: u32, dev_ctrl: DeviceControl, out_buf: &mut [u8]) {
        // reads higher than 2 GiB not allowed
        if sectors > 0x3fffff {
            return;
        }
    }


    /// Software reset of ATA PIO bus.
    #[allow(unused)]
    pub fn soft_reset(dcr: u16) {
        let mut port = Port::<u8>::new(dcr);

        unsafe {
            // software reset 
            port.write(0x04);
            // bus reset to normal operation
            port.write(0x00);
            // 4 tries to wait for status bits to reset
            _ = port.read();
            _ = port.read();
            _ = port.read();
            _ = port.read();

            // wait for ATA PIO to report READY status
            loop {
                // read byte from the status register
                let x = port.read();
                // check BUSY and READY bits (0x80 = BUSY, 0x40 = READY)
                if x & 0xc0 != 0x40 {
                    break;
                }
            }
        }
    }
}
