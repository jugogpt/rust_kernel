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

    use blog_os::memory::active_level_4_table; //use (current_folder)::(file_we_want_to_access_the_function_of)::(function_name)
    use x86_64::VirtAddr;
    println!("Hello World{}", "!");

    blog_os::init();
    //VirtAddr is a struct of x86_64
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);
        }
    }

    
    #[cfg(test)]
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


