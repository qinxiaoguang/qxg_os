// also known as stack allocator
// 线性的进行内存分配， 每次分配完成后，next指向的都是下一个未被分配内存的边界
// 同时记录内存分配的次数，当次数为0，则将整块内存回收
// 缺点明显，就是只有在所有内存都释放的时候才能被复用

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size - 1;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    // 因为内存分配器定义的方法参数是self，而我们需要在内部改变结构体对应的内容，所以需要内存可变性。
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.inner.lock();
        // 将bump.next以layout.align()的大小方式对齐，其中layout.align返回的是一个size
        let alloc_start = align_up(bump.next, layout.align());
        // checked_add 是越界检查， 如果越界，则返回None
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };
        if alloc_end > bump.heap_end {
            ptr::null_mut()
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut bump = self.inner.lock();

        bump.allocations -= 1;
        if bump.allocations == 0 {
            // 可以重新开始分配内存
            bump.next = bump.heap_start;
        }
    }
}
