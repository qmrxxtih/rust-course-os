

use alloc::alloc::{GlobalAlloc, Layout};
use crate::multiboot;
use core::ptr::null_mut;
use x86_64::structures::paging as Paging;


pub const KERNEL_HEAP_START: u64 = 0x700000000000;
pub const KERNEL_HEAP_END: u64 = KERNEL_HEAP_START + 1024 * 1024 - 1;
pub const KERNEL_HEAP_SIZE: u64 = KERNEL_HEAP_END - KERNEL_HEAP_START;

#[global_allocator]
static GLOBAL_ALLOC: linked_list_allocator::LockedHeap = linked_list_allocator::LockedHeap::empty();


pub struct NormalFrameAllocator<'a> {
    mem_map: &'a [multiboot::MemoryMapEntry],
    next: u64,
}


#[allow(dead_code)]
impl<'a> NormalFrameAllocator<'a> {
    pub fn new(mem_map: &'a [multiboot::MemoryMapEntry]) -> Self {
        Self {
            mem_map,
            next: 0,
        }
    }

    pub fn frame_iter(&self) -> impl Iterator<Item = Paging::PhysFrame> {
        self
            .mem_map
            .iter()
            .filter(|m| m.typ == multiboot::MemoryMapType::Available)
            .map(|m| m.base_addr..(m.base_addr+m.length))
            .flat_map(|m| m.step_by(4096))
            .map(|m| Paging::PhysFrame::containing_address(x86_64::PhysAddr::new(m)))
    }
}


unsafe impl<'a> Paging::FrameAllocator<Paging::Size4KiB> for NormalFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Paging::PhysFrame> {
        let frame = self
            .frame_iter()
            .nth(self.next as usize);
        self.next += 1;
        frame
    }
}


pub struct Dummy;


/// Initialise heap address space by allocating and mapping required pages and loading initial
/// linked list heap allocator.
/// Heap is allocated on address space from 0x8000.0000.0000, with 100 KiB size.
pub fn heap_init(
    mapper: &mut impl Paging::Mapper<Paging::Size4KiB>,
    frame_alloc: &mut impl Paging::FrameAllocator<Paging::Size4KiB>
) -> Result<(), Paging::mapper::MapToError<Paging::Size4KiB>> {
    // create page range from specified address range
    let range = Paging::Page::range_inclusive(
        Paging::Page::containing_address(x86_64::VirtAddr::new(KERNEL_HEAP_START)),
        Paging::Page::containing_address(x86_64::VirtAddr::new(KERNEL_HEAP_END))
    );

    // flags to use for new mapped pages
    let flags = Paging::PageTableFlags::PRESENT | Paging::PageTableFlags::WRITABLE;

    // map each page in range
    for page in range {
        // allocate new frame
        let frame = match frame_alloc.allocate_frame() {
            Some(f) => f,
            None => return Err(Paging::mapper::MapToError::FrameAllocationFailed),
        };
        unsafe {
            // map the new frame
            match mapper.map_to(page, frame, flags, frame_alloc) {
                Ok(f) => f.flush(),
                Err(e) => return Err(e),
            }
        }
    }
    // loading the linked list allocator
    unsafe {GLOBAL_ALLOC.lock().init(KERNEL_HEAP_START as *mut u8, KERNEL_HEAP_SIZE as usize)};

    Ok(())
}


unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}
