
use x86_64::structures::paging::PageTable;
use x86_64::registers::control::Cr3;


/// Returns new page mapper structure, which can be used to retrieve information about pages or
/// mapping new pages to memory.
#[allow(unused)]
pub fn get_page_mapper(phys_offset: Option<x86_64::VirtAddr>) -> Paging::OffsetPageTable<'static> {
    let (cr3, _) = Cr3::read();
    unsafe { 
        Paging::OffsetPageTable::new(
            &mut *(cr3.start_address().as_u64() as *mut PageTable),
            phys_offset.unwrap_or(x86_64::VirtAddr::zero()))
    }
}

use x86_64::structures::paging as Paging;

use crate::vga_printf;


/// Map given huge page to given physical memory frame using given mapper with given flags.
#[allow(unused)]
pub fn map_huge_page(
    page: Paging::Page<Paging::Size2MiB>,
    memory_frame: Paging::PhysFrame::<Paging::Size2MiB>,
    mapper: &mut Paging::OffsetPageTable,
    flags: Paging::PageTableFlags,
    _frame_allocator: &mut impl Paging::FrameAllocator<Paging::Size2MiB>
) {
    // enable required flags if they are not enabled
    let flags = flags | Paging::PageTableFlags::PRESENT | Paging::PageTableFlags::HUGE_PAGE;
    // Retrieve page map (level 4)
    let page_map = mapper.level_4_table_mut();
    let page_map_entry = &mut page_map[page.p4_index()];
    // Retrieve PDP (level 3)
    let page_dir_pointer = unsafe { &mut *(page_map_entry
        .frame()
        .expect("Failed to dereference page map entry!")
        .start_address()
        .as_u64()
        as *mut PageTable)
    };
    let pdp_entry = &mut page_dir_pointer[page.p3_index()];
    // Retrieve PD (level 2)
    let page_dir = unsafe {
        &mut *(pdp_entry
            .frame()
            .expect("Failed to dereference page directory pointer!")
            .start_address()
            .as_u64() as *mut PageTable)
    };
    let pd_entry = &mut page_dir[page.p2_index()];
    pd_entry.set_addr(memory_frame.start_address(), flags);
}


/// BROKEN: Debug prints page mappings currently in use.
/// BROKEN: This function does not check poiner validity - null pointer panics
#[allow(unused)]
pub fn debug_print_huge_pages(mapper: &mut Paging::OffsetPageTable) {
    let mut counter = 0usize;
    // Retrieve page map (level 4)
    let page_map = mapper.level_4_table_mut();
    for x in page_map.iter() {
        if !x.flags().contains(Paging::PageTableFlags::PRESENT) || x.is_unused() { continue; }
        vga_printf!("Page Map Entry : {:?}\n", x.addr());
        // Retrieve PDP (level 3)
        let page_dir_pointer = unsafe { &mut *(x
            .frame()
            .expect("Failed to dereference page map entry!")
            .start_address()
            .as_u64()
            as *mut PageTable)
        };
        for y in page_dir_pointer.iter() {
            if !y.flags().contains(Paging::PageTableFlags::PRESENT) || y.is_unused() { continue; }
            vga_printf!("Page Directory Pointer Entry : {:?}\n", y.addr());
            // Retrieve PD (level 2)
            let page_dir = unsafe {
                &mut *(y
                    .frame()
                    .expect("Failed to dereference page directory pointer!")
                    .start_address()
                    .as_u64() as *mut PageTable)
            };
            let flags = Paging::PageTableFlags::PRESENT | Paging::PageTableFlags::HUGE_PAGE;
            let entries = page_dir
                .iter()
                .filter(|x| x.flags().contains(flags));

            for z in entries.take(10) {
                if !z.flags().contains(flags) || z.is_unused() { continue; }
                vga_printf!("Page Directory Entry : {:?}\n", z.addr());
            }
            counter += page_dir.iter().filter(|x| x.flags().contains(flags)).count();
        }
    }
    vga_printf!("TOTAL NUMBER OF PAGES : {}", counter);
}

