// 串口模块，通过串口向console打印内容
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

// 创建串口实例,使用uart_16550::SerialPort
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        // 与print类似，需要使用锁来提供sync及内部可变性
        Mutex::new(serial_port)
    };
}

// 以下为了提高易用性，所提供的相应的宏，使用方法类似println
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    // 与println相应的，中断时打印容易造成死锁。
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
