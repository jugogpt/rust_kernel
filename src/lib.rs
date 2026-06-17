#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

//make public to make them usable outside our library; this is also requiired for making our println and serial)println macros usable since the y use the _print function of these  modules 
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
use core::fmt;
use core::panic::PanicInfo;

pub mod serial;
pub mod vga_buffer;

pub mod gdt;

pub mod interrupts;

pub mod memory;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    vga_buffer::_print(args);
}



#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where T: Fn(), {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}


pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {  // this is the start function of the cargo test
    //only thing that changed is that we need no longer the extern "C" or the #[unsafe(no_mangle)]
    //we were able to do this because of the entry_point! macro provided by the bootloader; this kernel entry point is only used here in test
    init(); // new
    test_main();
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() { // this function essentionally just uses the x86_64 library to call a breakpoint exception to test the interrupt_mod we made
    //invoke a breakpoiint exception 
    x86_64::instructions::interrupts::int3();
}



#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10, 
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe {
        interrupts::PICS.lock().initialize();
        // Unmask timer (IRQ 0) and keyboard (IRQ 1): clear bits 0 and 1 → 0xFC
        interrupts::PICS.lock().write_masks(0xFC, 0xFF);
    }
    x86_64::instructions::interrupts::enable();
}


pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

