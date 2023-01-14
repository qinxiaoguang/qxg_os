// in tests/basic_boot.rs
// 每个文件都是独立的可执行程序

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
// 此处如果填crate::test_runner则找的是tests目录下的test_runner
#![test_runner(qxg_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use qxg_os::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    qxg_os::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
