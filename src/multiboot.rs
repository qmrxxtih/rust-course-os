


pub enum MultibootTagType {
    BootCommandLine, // 1
    Modules,
    ElfSymbols,
    MemoryMap,
    BootLoaderName,
    APMTable,
    VBEInfo,
    FramebufferInfo,
}

/// Multiboot Tag, one entry in Multiboot Information Structure.
#[derive(Clone, Copy)]
pub struct MultibootTag {
    pub typ: u32,
    pub size: u32,
    data: Option<*const u8>,
}

/// Structure for iteration through Multiboot Information Structure tags.
pub struct MultibootTagIterator {
    ptr: *const MultibootTag,
    finished: bool,
}

impl Iterator for MultibootTagIterator {
    type Item = *const MultibootTag;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            None
        } else {
            let result = self.ptr;
            let size = unsafe { (*result).size }; // get size of tag
            let typ = unsafe { (*result).typ }; // get type of tag

            // check if tag is Terminating Tag
            if typ == 0 && size == 8 {
                self.finished = true;
            } else {
                // tags are padded on 8 byte boundaries (size >> 3 = size / 8)
                let advance = if size % 8 == 0 { size >> 3 } else { (size >> 3) + 1 } as usize;
                let ptr = self.ptr as *const u64;
                self.ptr = ptr.wrapping_add(advance) as *const MultibootTag;
            }
            Some(result)
        }
    }
}

/// Multiboot Information Structure, passed by GRUB to our program.
/// It contains important system information such as memory map,
/// ISO ELF sections, etc.
pub struct MultibootMBI {
    total_size: u32,
    reserved: u32,
    first_tag: *const MultibootTag,
}


impl MultibootMBI {
    /// Loads Multiboot Information Structure from specified address. Note that this function does not check
    /// if given address points to valid MIS.
    pub fn load_from_addr(addr: *const u32) -> Self {
        let mut addr = addr;
        let total_size;
        let reserved;
        let first_tag;
        unsafe {
            // tomfoolery with pointers
            total_size = *addr;
            // advances 1 * sizeof(u32) bytes
            addr = addr.add(1);
            reserved = *addr;
            // advances 1 * sizeof(u32) bytes
            addr = addr.add(1);
            first_tag = addr as *const MultibootTag;
        }
        Self {
            total_size,
            reserved,
            first_tag,
        }
    }

    pub fn iter(&self) -> MultibootTagIterator {
        MultibootTagIterator {
            ptr: self.first_tag,
            finished: false,
        }
    }

    pub fn total_size(&self) -> u32 {
        self.total_size
    }

    pub fn reserved(&self) -> u32 {
        self.reserved
    }
}
