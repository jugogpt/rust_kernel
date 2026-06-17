use x86_64:: { // new instance of the CPU class
    structures::paging::PageTable, 
    VirtAddr,
};

//returns a mutable reference to the active level 4 table.
//
//this function is unsafe because the caller must guarentee that the complete
//physical memory is mapped to virtual memory at the passed 'physical_memory_offset'  Also, this functio must be only called once to avoid 
//aliasing '&mut' references (which is undefined behavior).

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


