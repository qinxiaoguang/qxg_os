// 单独搞个lib.rs是为了让tests也能访问到
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)] // 使用x86处理中断的abi
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod vga_buffer; // 中断处理

extern crate alloc;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::structures::idt::InterruptDescriptorTable;

pub fn init() {
    // 初始化全局描述符，用于处理分段
    gdt::init();
    // 初始化中断, lib.rs中的函数可以让其在项目中任何其他位置调用。只需要使用${proj name}::方法就能进行调用
    // 如果 没初始化中断， 即程序找不到中断处理 函数 ， 则会找二重中断，如果二重中断处理函数也没找到， 则找三重， 三重也没找到一般都只能重启。
    // 操作系统一般有二层中段和是层中断， 其实就是嵌套中断， 在中断程序运行的时候，该程序又导致新的中断， 就是嵌套中断。
    interrupts::init_idt();

    // 初始化中段处理器,
    unsafe { interrupts::PICS.lock().initialize() };

    // 允许中断， 否则中断在cpu中是禁用的， 将会无法收到时钟中断，键盘等相关中断
    // 需要与异常区别开来
    // 开启后， 如果没配置时钟中断的处理函数， 将会导致二次中断异常等。
    x86_64::instructions::interrupts::enable();
}

// hlt指令会让cpu休眠， 直到下次中断到来。
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// 为test_case抽象的trait
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

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

/*
// cargo test --lib的入口
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use x86_64::instructions::hlt;

    init();
    test_main();
    hlt_loop();
}
*/

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

// 处理内存分配失败错误
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
