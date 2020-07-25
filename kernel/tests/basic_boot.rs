#![no_std]
#![no_main]

#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{BootInfo, entry_point};

use core::panic::PanicInfo;

use kernel::println;

entry_point!(main);

fn main(_boot_info: &'static BootInfo) -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
