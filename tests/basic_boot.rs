#![no_std]
#![no_main]

use blog_os::{serial_println, test_panic_handler, test_runner, Testable};
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_runner(&[&test_println as &dyn Testable]);
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

fn test_println() {
    serial_println!("test_println output");
}
