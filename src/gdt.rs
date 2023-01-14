use lazy_static::lazy_static;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

// 当内核堆栈溢出导致页错误中断的时候，此时有一个异常指针被推入中断栈中， 导致第二次错误，同样的导致，依然会把相关指针推入栈中，导致第三次错误。
// 而x86则是通过InterruptStackTable(ist)表来进行中段时的堆栈切换， 这样在发生堆栈溢出的时候， 中断的堆栈信息就可以推入一个实现准备好的堆栈中，就不会再引发二次中断错误了。
// 而ist就是早期架构中tss中的一部分， 而在32位模式下的tss会保存进程的寄存器信息及硬件的上下文切换， 而在64位模式下， 则会保存特权栈表及ist.
lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // 上面提到的ist,
        // 需要告诉cpu，tss在哪里， 即需要加载tss
        // 但加载tss比较繁琐， 因历史原因, tss用于分段系统中，
        // 而分段系统则需要创建一个gdt表
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };

    // gdt是一个全局描述符表， 即global descriptor table,
    // 只有在x86中有此表,该表可以存在在任何位置，但需要告诉cpu该表的内存地址
    // 用于存储分段信息，虽然在64位模式不再支持分段， 但该结构仍然存在， 处理内核和用户空间及tss加载
    // 分页已经是操作系统的标准实现， 所以一个操作系统即便没有分段也一定会有分页
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    //use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}
