// in tests/basic_boot.rs

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
// 此处如果填crate::test_runner则找的是tests目录下的test_runner
#![test_runner(qxg_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    qxg_os::test_panic_handler(info)
}

use qxg_os::println;

#[test_case]
fn test_println() {
    println!("test_println output");
}
