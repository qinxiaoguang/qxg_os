use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::gdt;

// InterruptStackFrame 是中断栈中的栈帧信息， 其比函数调用多一些信息
// 需要设置为静态的， 因为idt表在os运行期间经常访问
// 当处理中段的时候， 会将中断的堆栈帧推到堆栈中。
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); // 当二层中断发生时，切换到gdt中表示的堆栈中。
        }
        idt
    };
}

// idt为中断表， 需要处理中断表， 中断才会找到合适的函数去执行相应的中断函数
// 找中段处理程序是引导程序自动写好的， 所以我们不需要关心， 比如内存缺页异常，访问异常等。
pub fn init_idt() {
    IDT.load()
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
