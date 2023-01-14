pub mod bump;
pub mod linked_list;

use alloc::alloc::{GlobalAlloc, Layout};
use core::{arch::x86_64::_MM_HINT_NTA, ptr::null_mut};

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use self::bump::BumpAllocator;

//use linked_list_allocator::LockedHeap;
// 指定堆内存分配器
//#[global_allocator]
//static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

// 定义对内存的大小及开始位置
// 该初地址为虚拟内存
// 实际物理内存地址可以不是连续的(具体看实现)，但是虚拟地址需要是连续的
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

// 需要初始化堆空间，
// 因为不初始化，堆空间没有在页表中注册，且相应的内存没有被标记为已使用
// 所以无法使用
// 所以堆分配器是依赖于页表的
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        // 获取包含heap_start的页面
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        // 将状态置为已分配及可写
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // 为page进行内存空间分配及映射, 通过fluash方法，会将对应的映射刷新到tlb中
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() }
    }

    // 初始完heap后，需要初始allocator,allocator从heap中分配内存
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    Ok(())
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

// 因为无法直接为Mutex实现GlobalAlloc,所以自定义一个lock
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

pub struct Dummy;

// 自定义内存分配,只需要实现GlobalAlloc即可
unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}
