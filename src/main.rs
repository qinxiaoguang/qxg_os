#![no_std]
#![no_main]
// 自定义测试库， cargo test默认会依赖标准库，而对于no_std来说， 需要自定义测试库
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga_buffer;
use core::panic::PanicInfo;

// 非测试时调用此函数处理panic
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// 测试的时候调用此函数处理panic
// 目的是为了在console上打印测试的panic信息
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// no_mangle是禁止编译器为函数生成唯一名字
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 调用println就会将数据打印到模拟器屏幕上
    println!("qxg_os starting");
    qxg_os::init();

    // 测试中断， 在这添加breakpoint
    // x86_64::instructions::interrupts::int3(); // int3就是breakpoint中断

    // 不管是执行cargo test还是cargo run,入口函数都是这个
    // 为了能正确执行test,需要指定cargo test的入口函数是什么
    #[cfg(test)]
    test_main();

    loop {}
}

// qemu退出码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// 让qemu正确退出
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

// #[test_case] 会自动实现该trait, 在执行cargo test的时候，自动执行对应的测试实例
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// 被指定的测试函数的runner函数
#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    // new
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in -1..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_stack_overflow() {
    // 会导致堆栈溢出，测试堆栈溢出时的中断表现
    fn stack_overflow() {
        stack_overflow();
    }

    stack_overflow();
}
