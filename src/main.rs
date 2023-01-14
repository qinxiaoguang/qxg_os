#![no_std]
#![no_main]
// 自定义测试库， cargo test默认会依赖标准库，而对于no_std来说， 需要自定义测试库
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod memory;
mod serial;
mod vga_buffer;
extern crate alloc;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use qxg_os::allocator;
use x86_64::VirtAddr;

// 非测试时调用此函数处理panic
#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use qxg_os::hlt_loop;

    println!("{}", info);
    hlt_loop();
}

// 测试的时候调用此函数处理panic
// 目的是为了在console上打印测试的panic信息
// 虽然lib.rs也定义了panic_handler,但是main和lib可以被当成是两个独立的包
// lib.rs只是向外暴露一些函数/方法,在执行cargo test --lib的时候使用的lib.rs中的panic_handler
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    qxg_os::test_panic_handler(info);
}

entry_point!(kernal_main);

// 之前是通过no_mangle来定义入口函数_start,但该函数无法传递相关参数
// 而通过entry_point!来指定函数，该函数可以传递参数，但不能乱传参数。
// 其中参数就是BootInfo,该参数是bootloader初始化后提供的相关信息，包含了页表的信息等。
// 其中memory_map包含了哪些物理内存可用，哪些物理内存给相关设备来使用，因为这些信息只有在bios中进行查询，所以只能由bootloader来提供。
// 而physical_memory_offset 则是物理内存映射到虚拟内存的映射关系表的存储位置，通过该变量，可以访问该映射关系，进而可以通过物理地址来访问虚拟地址，主要用于四级页表中向下级页表寻址时的定位。
fn kernal_main(boot_info: &'static BootInfo) -> ! {
    // 调用println就会将数据打印到模拟器屏幕上
    println!("qxg_os starting");
    qxg_os::init();

    // 测试中断， 在这添加breakpoint
    // x86_64::instructions::interrupts::int3(); // int3就是breakpoint中断

    // 测试page fault
    // 写入page错误
    /*let ptr = 0xdeadbeaf as *mut u32;
    unsafe {
        *ptr = 42;
    }*/

    // 初始化分页,分页是bootloader已经支持了的
    // 此处的分页需要支持翻译虚拟内存到物理内存
    // 以及物理内存到虚拟内存(页表存储的是物理内存)
    // 以及页表的创建等功能
    // 这是后边内存使用的基础
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // 初始化堆内存分配器, 要在分页初始化之后,依赖于分页
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");
    // 在初始化完allocator就可以使用Box, Vec, Rc等等相关方法，因为这些都依赖于堆内存分配器

    // 不管是执行cargo test还是cargo run,入口函数都是这个
    // 为了能正确执行test,需要指定cargo test的入口函数是什么
    #[cfg(test)]
    test_main();

    qxg_os::hlt_loop();
}

/*
// no_mangle是禁止编译器为函数生成唯一名字
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    // 调用println就会将数据打印到模拟器屏幕上
    println!("qxg_os starting");
    qxg_os::init();

    // 测试中断， 在这添加breakpoint
    // x86_64::instructions::interrupts::int3(); // int3就是breakpoint中断

    // 测试page fault
    // 写入page错误
    /*let ptr = 0xdeadbeaf as *mut u32;
    unsafe {
        *ptr = 42;
    }*/

    // 不管是执行cargo test还是cargo run,入口函数都是这个
    // 为了能正确执行test,需要指定cargo test的入口函数是什么
    #[cfg(test)]
    test_main();

    qxg_os::hlt_loop();
}
*/

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
