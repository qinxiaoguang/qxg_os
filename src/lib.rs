// 单独搞个lib.rs是为了让tests也能访问到
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)] // 使用x86处理中断的abi

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer; // 中断处理

use core::panic::PanicInfo;
use x86_64::structures::idt::InterruptDescriptorTable;

pub fn init() {
    // 初始化全局描述符，用于处理分段
    gdt::init();
    // 初始化中断, lib.rs中的函数可以让其在项目中任何其他位置调用。只需要使用${proj name}::方法就能进行调用
    // 如果 没初始化中断， 即程序找不到中断处理 函数 ， 则会找二重中断，如果二重中断处理函数也没找到， 则找三重， 三重也没找到一般都只能重启。
    // 操作系统一般有二层中段和是层中断， 其实就是嵌套中断， 在中断程序运行的时候，该程序又导致新的中断， 就是嵌套中断。
    interrupts::init_idt();
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
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    // new
    exit_qemu(QemuExitCode::Success);
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

// cargo test --lib的入口
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
