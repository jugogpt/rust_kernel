#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::println;
use bootloader::{BootInfo, entry_point};

entry_point!(kernel_main); // Type checks the start function so that a comilation error occurs when we use a wrong function signature, for example by adding an argument or changing the argument type 
//no longer need the no_mangle or the extern "C" anymore because kernel_main defines the start point at a lower level where this is implied
fn kernel_main(boot_info: &'static BootInfo) -> ! {



    use blog_os::memory;
    use x86_64::{structures::paging::Page, VirtAddr};

    use blog_os::memory::BootFrameAllocator;
   
    println!("Hello World{}", "!");

    blog_os::init();




    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset); // we obtain the physical memory offset from the bootloader
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map)};

    //maps an unused page 

    let page = Page::containing_address(VirtAddr::new(0)); //this is the start of the bootloader, always has (or is able to have) a page already
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);


    //write the string 'New!' to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

     
    
    #[cfg(test)] //testing main for cargo test test_main();
    test_main();
    println!("No crash!");
    blog_os::hlt_loop();
}


//bootinfo passes the BootInfo struct that contains the physical memory offset value and the map_physical_memory (i.e. the amount of physcial memory NOT being used)
/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}








/*






below is an example of a manual address test of translate_addr in the memory.rs:
   /*
        //hardcoded addresses
        let addresses = [
        // the identity-mapped vga buffer page 

        // the vga text buffer is NOTTT regular Ram, it is memory mapped I/O. the display listens for memory access at physical address; in short vga-buffer is not actually RAM so it can be identity mapped; graphics hardware will only repsond at their pysycial adddress'
        0xb8000, //the vga buffer address is usuall
        //some code page 
        0x201008,
        // some stack page
        0x0100_00200_1a10,
        // virtual address mapped to physical address 0 
        boot_info.physical_memory_offset, //the virtual address of the physical address 0 is equal to the physical memory address of 0
    ];

    //using the mapper.translate_addr(virt) to translate the VirtAddr::new(address) by the physical_memory_offset provided by the bootloader
    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);

    }

        
        
        
        */

 //VirtAddr is a struct of x86_64
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };


    

    for (i, entry) in l4_table.iter().enumerate() {

        use x86_64::structures::paging::PageTable;


        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);
        }


        //get the physical address from the entry and convert it 
        let phy = entry.frame().unwrap().start_address();
        let virt = phys.as_u64() + boot_info.physical_memory_offset;
        let ptr = VirtAddr::new(virt).as_mut_ptr();
        let l3_table: &PageTable = unsafe { &*ptr };



        //print non-empty entries of the level 3 table 

        for (i, entry) in l3_table.iter().enumerate() {
            if !entry.is_unused() {
                println!(" L3 Entry {}: {:?}", i, entry);
            }
        }
    }



*/
