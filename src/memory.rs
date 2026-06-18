use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};



use bootloader::bootinfo::MemoryMap;


//returns a mutable reference to the active level 4 table.
//
//this function is unsafe because the caller must guarentee that the complete
//physical memory is mapped to virtual memory at the passed 'physical_memory_offset'  Also, this functio must be only called once to avoid 
//aliasing '&mut' references (which is undefined behavior).


//keep this init, although our BootFrameAllocator has its own init that overrides this one in its implementation

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}






//unsafe bc we are reading raw CPU registers + constructing raw pointers and referencing them,

//CR3 gives us a PHYSICAL address of theh L4 table, but the CPU lets us only accesss the virtual addresses
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;


    let (level_4_table_frame, _) = Cr3::read(); // Cr3::read() outputs a tuple with the level table frame as well as the flag, which we don't need

    let phys = level_4_table_frame.start_address(); //physical memory address of the starting (pointed to by cr3) physical space frame, no offset
    let virt = physical_memory_offset + phys.as_u64(); //virutal memory address, offset applied, this is the address to the page (memory area chunk of virtual memory)
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr } //dereferencing and returning 

}



//in order to jsut have a fucntion to quickly obtain the mapped physical address given a virtual memory address
// AKA TRANSLATING ADDRESS

//translates the given virtual address to the mapped physical addres, or 'None' if the address is NOT mapped
//complete physical memory is mapped to virtual memory at the passed  'physical_memory _offset.'

pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> { //return if it exists or not
    translate_addr_inner(addr, physical_memory_offset) //we will be using this to alter addresses, so this function's use is just to cover the translate_addr_inner(addr, physical_memory_offset) into unsafe version
}

//private function that is called by 'translate_addr', which is supposed to turn a virtual address into its mapped py=hysical address equivalent, without having to manually dfs the pages

fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> 
{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;


    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];

    let mut frame = level_4_table_frame;

    for &index in &table_indexes {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };


        //now we need to read the page table entry and update the 'frame'
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None, //this is a valid response because not all virtual addresses are mapped, so we could type random stuff and would get this match-return
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    //calculate the physica; address by adding the page offset to the frame start address
    Some(frame.start_address() + u64::from(addr.page_offset()))
}


//need our own function to call the map_to function from the mapper trait 
//recall that the Mapper trait comes from x86_64:: ...

pub fn create_example_mapping (
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000)); // we do not know whether this address is already in use or not, so this is very unsafe 

    let flags = Flags::PRESENT | Flags::WRITABLE; //this is the equivalent of saying that the variable flags can be either PRESENT or  WRITABLE

    let map_to_result = unsafe {
        //FIXME: this is not safe, we do it only for the sake of testing...
        mapper.map_to(page, frame, flags, frame_allocator)
    };


    map_to_result.expect("map_to failed").flush();
}


// need to create a dummy type that implements the FrameAllocator trait in order to test our 'create_example_mapping(page: Page, mapper: &mut OffsetPageTable, frame_allocator: &mut impl FrameAllocator())'
// recall that the OffsetPageTable hodls the mapper functionality, and strcuts with the FrameAllocator<Size4KiB> trait can allocate frames, and page is stored in the Page struct

pub struct EmptyFrameAllocator;


//in order to implement a trait to a struct 
//in order to map pages that don't have a level 1 page table yet, we need to createe a proper FrameAllocator --> Problem: how do we know which frames are unused and how much physical memory is avaiable???
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}



//remember that the memory map is passed by the bootloader but provided by the BIOS/UEFI firmware. It can only be queired bery early in the boot process, we we store it early in the bootloader
//the memory map consistes of a list of Memory Region structs, which contain the start address, the length, and the type (e.g. unused, reserved, etc.) of each memory region...
pub struct BootInfoFrameAllocator {
    memory_map: &'static  MemoryMap,
    next: usize, //keeps track of the number of the next frame that the frame_allocator should return HEHE
}

impl BootInfoFrameAllocator {
    //need to add an auxillary method that converts the memory map into an iterator of usable framezs 

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> { //this is supposed to return some struct that implements Iterator of the parameter Item which is the PhysFrame address
        //get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range

        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))

    }


    //Create a Frame Allocator from the passed memory map
    //
    // This function is unsafe because the caller must guarantee that the passed memory map is valid.
    //memory map is valid the main requirement is that all frames that are makred as USABLE in it are really unused

    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { //pub unsafe (bc we actually do not know if the memory map is valid) fn init instantiates with the given memory map and default page of 0, which will be incrase for every frame allocation to acoid returng the same frame twice
            memory_map,
            next: 0,
        }
    }
}   