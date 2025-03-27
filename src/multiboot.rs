#![allow(dead_code)]

use core::slice;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TagType {
    End = 0,
    BootCommandLine,
    BootLoaderName,
    Modules,
    MemoryInfo,
    BootDevice,
    MemoryMap,
    VbeInfo,
    FramebufferInfo,
    ElfSymbols,
    ApmTable,
    Efi32Addr,
    Efi64Addr,
    SmbiosTables,
    AcpiOldRspd,
    AcpiNewRspd,
    NetInfo,
    EfiMemoryMap,
    EfiBootNotTerminated,
    Efi32ImgHandle,
    Efi64ImgHandle,
    ImgLoadBaseAddr,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
// TODO: undo pub once `Unimplemented` is not necessary
pub struct TagInfo {
    typ: TagType,
    size: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryInfo {
    lower: u32,
    upper: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BootDevice {
    biosdev: u32,
    partition: u32,
    sub_partition: u32,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MemoryMapType {
    Invalid = 0,
    /// available RAM
    Available = 1,
    Reserved = 2,
    /// usable memory holding ACPI information
    AcpiInfo = 3,
    /// memory that needs to be preserved on hibernation
    HiberPreserve = 4,
    /// ram in use by defective ram sticks
    Defective = 5,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    base_addr: u64,
    length: u64,
    typ: MemoryMapType,
    reserved: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VbeInfo {
    mode: u16,
    interface_seg: u16,
    interface_off: u16,
    interface_len: u16,
    control_info: [u8; 512],
    mode_info: [u8; 256],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FramebufferInfoBase {
    addr: u64,
    pitch: u32,
    width: u32,
    height: u32,
    bpp: u8,
    typ: u8,
    reserved: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ApmTable {
    version: u16,
    cseg: u16,
    offset: u32,
    cseg_16: u16,
    dseg: u16,
    flags: u16,
    cseg_len: u16,
    cseg_16_len: u16,
    dseg_len: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AcpiOldRspd {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    addr: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AcpiNewRspd {
    old: AcpiOldRspd,
    len: u32,
    addr: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[derive(Debug)]
pub enum Tag {
    /// string representing the command line
    BootCommandLine(&'static [u8]),

    /// string representing the boot loader name
    BootLoaderName(&'static [u8]),

    /// This tag indicates to the kernel what boot module was loaded along with
    /// the kernel image, and where it can be found.

    /// The ‘mod_start’ and ‘mod_end’ contain the start and end physical
    /// addresses of the boot module itself. The ‘string’ field provides an
    /// arbitrary string to be associated with that particular boot module; it
    /// is a zero-terminated UTF-8 string, just like the kernel command line.
    /// Typically the string might be a command line (e.g. if the operating
    /// system treats boot modules as executable programs), or a pathname (e.g.
    /// if the operating system treats boot modules as files in a file system),
    /// but its exact use is specific to the operating system.

    /// One tag appears per module. This tag type may appear multiple times.
    Modules {
        mod_start: u32,
        mod_end: u32,
        string: &'static [u8],
    },

    /// ‘mem_lower’ and ‘mem_upper’ indicate the amount of lower and upper
    /// memory, respectively, in kilobytes. Lower memory starts at address 0,
    /// and upper memory starts at address 1 megabyte. The maximum possible
    /// value for lower memory is 640 kilobytes. The value returned for upper
    /// memory is maximally the address of the first upper memory hole minus
    /// 1 megabyte. It is not guaranteed to be this value.
    MemoryInfo(&'static MemoryInfo),

    /// This tag indicates which BIOS disk device the boot loader loaded the OS
    /// image from. If the OS image was not loaded from a BIOS disk, then this
    /// tag must not be present. The operating system may use this field as
    /// a hint for determining its own root device, but is not required to.

    /// The ‘biosdev’ contains the BIOS drive number as understood by the BIOS
    /// INT 0x13 low-level disk interface: e.g. 0x00 for the first floppy disk
    /// or 0x80 for the first hard disk.

    /// The three remaining bytes specify the boot partition. ‘partition’
    /// specifies the top-level partition number, ‘sub_partition’ specifies
    /// a sub-partition in the top-level partition, etc. Partition numbers
    /// always start from zero. Unused partition bytes must be set to
    /// 0xFFFFFFFF. For example, if the disk is partitioned using a simple
    /// one-level DOS partitioning scheme, then ‘partition’ contains the DOS
    /// partition number, and ‘sub_partition’ if 0xFFFFFF. As another example,
    /// if a disk is partitioned first into DOS partitions, and then one of
    /// those DOS partitions is subdivided into several BSD partitions using
    /// BSD’s disklabel strategy, then ‘partition’ contains the DOS partition
    /// number and ‘sub_partition’ contains the BSD sub-partition within that
    /// DOS partition.

    /// DOS extended partitions are indicated as partition numbers starting
    /// from 4 and increasing, rather than as nested sub-partitions, even
    /// though the underlying disk layout of extended partitions is
    /// hierarchical in nature. For example, if the boot loader boots from the
    /// second extended partition on a disk partitioned in conventional DOS
    /// style, then ‘partition’ will be 5, and ‘sub_partiton’ will be
    /// 0xFFFFFFFF.
    BootDevice(&'static BootDevice),

    /// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Memory-map
    MemoryMap(&'static [MemoryMapEntry]),

    /// The fields ‘vbe_control_info’ and ‘vbe_mode_info’ contain VBE control
    /// information returned by the VBE Function 00h and VBE mode information
    /// returned by the VBE Function 01h, respectively.

    /// The field ‘vbe_mode’ indicates current video mode in the format specified
    /// in VBE 3.0.

    /// The rest fields ‘vbe_interface_seg’, ‘vbe_interface_off’, and
    /// ‘vbe_interface_len’ contain the table of a protected mode interface defined
    /// in VBE 2.0+. If this information is not available, those fields contain
    /// zero. Note that VBE 3.0 defines another protected mode interface which is
    /// incompatible with the old one. If you want to use the new protected mode
    /// interface, you will have to find the table yourself.
    VbeInfo(&'static VbeInfo),

    /// https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html#Framebuffer-info
    FramebufferInfo {
        base: &'static FramebufferInfoBase,
    },

    /// The fields ‘version’, ‘cseg’, ‘offset’, ‘cseg_16’, ‘dseg’, ‘flags’,
    /// ‘cseg_len’, ‘cseg_16_len’, ‘dseg_len’ indicate the version number, the
    /// protected mode 32-bit code segment, the offset of the entry point, the
    /// protected mode 16-bit code segment, the protected mode 16-bit data
    /// segment, the flags, the length of the protected mode 32-bit code
    /// segment, the length of the protected mode 16-bit code segment, and the
    /// length of the protected mode 16-bit data segment, respectively. Only
    /// the field ‘offset’ is 4 bytes, and the others are 2 bytes. See
    /// http://www.microsoft.com/hwdev/busbios/amp_12.htm, for more
    /// information.
    ApmTable(&'static ApmTable),

    /// This tag contains pointer to i386 EFI system table.
    Efi32Addr(u64),

    /// This tag contains pointer to amd64 EFI system table.
    Efi64Addr(u64),

    /// https://wiki.osdev.org/RSDP
    AcpiOldRspd(&'static AcpiOldRspd),

    /// https://wiki.osdev.org/RSDP
    AcpiNewRspd(&'static AcpiNewRspd),

    /// This tag indicates ExitBootServices wasn’t called
    EfiBootNotTerminated,

    /// This tag contains pointer to EFI i386 image handle. Usually it is boot
    /// loader image handle.
    Efi32ImgHandle(u32),

    /// This tag contains pointer to EFI amd64 image handle. Usually it is boot
    /// loader image handle.
    Efi64ImgHandle(u64),

    /// This tag contains image load base physical address. It is provided only
    /// if image has relocatable header tag.
    ImgLoadBaseAddr(u32),

    // TODO: remove
    Unimplemented(TagInfo),
}

impl Tag {
    fn from_ptr(tag_info_ptr: *const TagInfo) -> Self {
        unsafe {
            let tag_info = *tag_info_ptr;
            // skip the tag info and get the address after it
            let ptr = tag_info_ptr.add(1).cast::<u32>();

            match (tag_info.typ) {
                TagType::BootCommandLine | TagType::BootLoaderName => {
                    let string = slice::from_raw_parts(
                        ptr.cast(),
                        // 8 is ize of the tag_info
                        (tag_info.size as usize) - 8,
                    );

                    if (matches!(tag_info.typ, TagType::BootCommandLine)) {
                        Self::BootCommandLine(string)
                    } else {
                        Self::BootLoaderName(string)
                    }
                }
                TagType::Modules => {
                    let string = slice::from_raw_parts(
                        ptr.add(2).cast(),
                        // tag_info (8) + 2 * u32 (8) = 16
                        tag_info.size as usize - 16 ,
                    );
                    Self::Modules {
                        mod_start: *ptr,
                        mod_end: *ptr.add(1),
                        string,
                    }
                }
                TagType::MemoryInfo => {
                    Self::MemoryInfo(&*(ptr as *const MemoryInfo))
                }
                TagType::BootDevice => {
                    Self::BootDevice(&*(ptr as *const BootDevice))
                }
                TagType::MemoryMap => {
                    if (*ptr.add(1) != 0) {
                        unimplemented!("unsupported memory map entry_version");
                    }

                    Self::MemoryMap(slice::from_raw_parts(
                        ptr.add(2).cast(),
                        // tag info (8) + 2 * u32 (8) = 16
                        (tag_info.size as usize - 16)
                            / size_of::<MemoryMapEntry>(),
                    ))
                }
                TagType::VbeInfo => Self::VbeInfo(&*(ptr as *const VbeInfo)),
                // TODO: rest of the fields
                TagType::FramebufferInfo => Self::FramebufferInfo {
                    base: &*(ptr as *const FramebufferInfoBase),
                },
                // TODO: TagType::ElfSymbols
                TagType::ApmTable => {
                    Self::ApmTable(&*(ptr as *const ApmTable))
                }
                TagType::Efi32Addr => Self::Efi32Addr(*(ptr as *const u64)),
                TagType::Efi64Addr => Self::Efi64Addr(*(ptr as *const u64)),
                // TODO: TagType::SmbiosTables
                TagType::AcpiOldRspd => {
                    Self::AcpiOldRspd(&*(ptr as *const AcpiOldRspd))
                }
                TagType::AcpiNewRspd => {
                    Self::AcpiNewRspd(&*(ptr as *const AcpiNewRspd))
                }
                // TODO: TagType::NetInfo
                // TODO: TagType::EfiMemoryMap
                TagType::EfiBootNotTerminated => Self::EfiBootNotTerminated,
                TagType::Efi32ImgHandle => Self::Efi32ImgHandle(*ptr),
                TagType::Efi64ImgHandle => {
                    Self::Efi64ImgHandle(*(ptr as *const u64))
                }
                TagType::ImgLoadBaseAddr => Self::ImgLoadBaseAddr(*ptr),
                _ => Self::Unimplemented(tag_info),
            }
        }
    }
}

pub struct Multiboot2 {
    pub total_size: u32,
    pub reserved: u32,
    ptr: *const TagInfo,
}

impl Multiboot2 {
    pub fn from_ptr(ptr: *const u32) -> Self {
        unsafe {
            Self {
                total_size: *ptr,
                reserved: *ptr.add(1),
                ptr: ptr.add(2) as *const TagInfo,
            }
        }
    }
}

impl Iterator for Multiboot2 {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        let tag_info = unsafe { *self.ptr };
        if (matches!(tag_info.typ, TagType::End)) {
            return None;
        }

        let result = Tag::from_ptr(self.ptr);

        // aligning to 8
        let addr = (self.ptr as usize) + (tag_info.size as usize) + 7;
        self.ptr = (addr & !7) as *const TagInfo;

        Some(result)
    }
}
