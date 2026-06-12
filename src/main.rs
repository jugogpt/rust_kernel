#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
mod serial;


use core::fmt;
use core::panic::PanicInfo;

mod vga_buffer;
use crate::vga_buffer::WRITER;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {

    serial_println!("Running {} tests", tests.len());
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    //exit the qemu by providing the exit code type of QemuExitCode, which is then prpovided to the port 0xf64 and wrote, which exits the QEMU
    exit_qemu(QemuExitCode::Success);
    //exit after all tests are ran
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}


#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    test_main();
    loop {}
}

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    print!("trivial assertion... ");
    assert_eq!(1,1);
    println!("[ok]")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode { //0 is to exit, 1 is to stay?
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port; //making a port object use x86_64::instruction::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);  //setting up port reference as mutable
        port.write(exit_code as u32); //now we port.write(exit_code as u32)
    }
}









/*

cargo bootimage
qemu-system-x86_64 -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin

*/