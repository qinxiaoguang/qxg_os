// 早期的内存访问为16位格式的， 只能访问64k的内存空间， 所以加上了分段方式
// cpu运行在保护模式下后， 段描述符指向本地或全局的描述符表(L/GDT， local/global descrpitor table)，
// 该描述符表包含了偏移地址，段大小，和访问权限。每个程序都会有自己的LDT，这样在访问的时候， 就起到了程序间隔离的效果。

// 一个虚拟地址会通过转换函数映射到对应的物理地址， 转换前的地址就被称为虚拟地址，转换后的则是物理地址。

// 分段的缺点就是， 容易产生内存碎片
// 分页是将虚拟内存和物理内存都分隔为同样大小的小块， 注意是小块， 而虚拟内存的小块是页，物理内存的小块则是帧。

// 分段的段大小是要比分页大的。 分段容易产生碎片，而分片也容易产生， 比如分片大小10k,而仅使用1k都要分配一个页。

// 一般每个程序都会有对应的页表， 用于内存间的转换， 该表每行保存一个页地址对应的帧地址，以及相应的读写权限 如page:0-> frame:300, r/w。
// 行之间都是连续的
// 但如果一个程序的内存只使用了0内存和100000000内存， 就需要很大的页表来保存，浪费空间。
// 这个时候就需要二级页表， 二级页表与一级页表的关系就类似一级页表与内存间的关系。即二级页表指向一级页表， 一级页表指向物理内存。
// 这个时候，只需要一个二级页表， 两个一级页表将内存映射保存下来。
// 同样的道理，可以将这种关系扩展到三级四级等，这种关系就叫多极/分层页表。

// x86_64使用四级页表，每页4kb内存。每级表512*8字节，所以四级表一共4kb，刚好存到一页上。
// 而定位的方式，则使用一个64位值即可定位(i64)，即64位的值就是一个虚拟地址,其中最后12位是偏移地址，接着36位是页表地址。
// 那么页表指是定位一个页，偏移地址则是用来定位具体地址的。
// 因为每个页表存储512个entry， 只需要9位来定位，所以四级页表， 只需要4*9=36位即可定位，加上前12位一共48位
// 每个进程的四级页表分配的位置并不是固定的，可能分开存储。

// 页表的entry保存在cr3寄存器上，所以cr3指向四级页表地址，而四级页表的每个entry指向三级页表地址，以此类推。

// 四级页表在做地址转换的时候代价昂贵，所以一般会有一个缓存来保存已知的翻译， 叫tlb(translation lookaside buffer).
// 但tlb在页表更新的时候不会自动更新或删除，所以内核在修改页表时需要手动更新tlb.或者可以通过cpu指令invlpg重新加载页表。
// 需要注意的是，开启了分页后，想要直接访问物理地址是不可能的。

// 引导程序(bootloader)已经为我们处理好了相应的页表，所以内核也已经运行在了虚拟内存上，而内核这个时候就无法直接访问物理内存。
// 而在页表中的每一项都存储的页对应的物理地址，我们找到了一个物理地址，内核无法直接访问，就无法找到下一个页表的位置。

// 为了解决这个问题，我们一般需要借助于bootloader的支持。
// 但这里有几个方法来解决他们， 核心就是当我们知道页表的物理内存时，需要找到能访问这个物理内存的虚拟内存。
// 第一种是，恒等映射， 即页表中对应的物理内存地址，在虚拟内存也表示为同样的地址。
// 缺点就是容易发生内存碎片，导致很难去创建一个大内存块，且创建一个新的页表，需要在两个内存地址中找到同样没被使用的地址。

// 第二种方法是，避免第一种内存互相影响的问题， 使用一个offset，如10Tb，那么虚拟地址10Tb+10kb就映射到了10kb的物理地址。但只映射页表间的内存。
// 缺点就是，每次创建一个页表时，都需要创建一个映射关系，即在虚拟内存中创建一个map映射。

// 第三种方法是在第二种方法的基础上， 不是仅映射页表间的内存，而是映射整个物理内存。缺点是需要用一个额外的地方来保存这个映射表
// 在x86_64上，可以开启huge_page,一个页有2M,而不是4k,这样保存一个32G的映射只需要132kb(3个一级表，32个二级表)。huge-page也会在tlb上进行相关的缓存。

// 第四种方法是创建临时的恒等映射，当访问的时候，才去找内存来创建恒等映射， 再通过恒等映射来找到对应的虚拟内存。缺点是比较麻烦。

// 第五种方法是递归，即页表中自动保存物理内存到虚拟内存的映射。在四级页表中的最后一个entry中，保存的是自身的页表地址，即四级页表中拿到的还是四级页表的地址
// 但是cpu会认为来到了三级表，就这样层层递归， 具体实现还是看文档吧。

// 代码中需要bootloader来支持页表映射，其中开启了map_physical_memory的feature,对应的是第三种方法。

use x86_64::structures::paging::OffsetPageTable;
use x86_64::PhysAddr;
use x86_64::{structures::paging::PageTable, VirtAddr};

// 初始化一个offset_page_table
// 后续可以通过OffsetPageTable的相关方法来计算
// table会通过physical_memory_offset来计算物理地址
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

// 访问cr3指向的四级页表， 该页表信息是x86_64的bootloader程序启动的时候就已经创建好的，因为os本身就要运行在虚拟内存之上，
// 所以在进行entry_point之前就已经初始化好。
// 而cr3就是指向的最顶层的页表信息及四级页表。
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    // 四级页表的物理地址
    let phys = level_4_table_frame.start_address();
    // 转换为虚拟地址，因为访问的时候需要通过虚拟地址访问
    let virt = physical_memory_offset + phys.as_u64();
    // 转换为对应的page_table
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

use bootloader::bootinfo::MemoryMap;

// BootInfo Frame内存分配器
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// 通过memroy_map来创建一个frame内存分配器
    /// 在创建页表的时候用得上
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }
}

use bootloader::bootinfo::MemoryRegionType;

impl BootInfoFrameAllocator {
    /// 在memory_map中可被使用的frames,已被os分配的也是可被使用的
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // 获取所有的region
        let regions = self.memory_map.iter();
        // 过滤出可用的region
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        // 将region映射为起始地址
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        // 该range 以4096的步数来获取对应的地址，因为4096就是4k,一个页面，也就获取到了地址对应的页起始地址
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // 将对应地址转换为物理frame地址(虚拟地址中为page,对应到物理地址就是frame,即page为虚拟地址中的一页， frame是物理地址中的一页)
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    // 分配frame
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

// 因为有了OffsetPageTable,已经包含了以下功能，所以不需要了
// // 将虚拟地址转换为物理地址
// pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
//     translate_addr_inner(addr, physical_memory_offset)
// }

// // 虚拟地址转换为物理地址的具体实现,包含很多unsafe代码
// fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
//     use x86_64::registers::control::Cr3;
//     use x86_64::structures::paging::page_table::FrameError;

//     // 首先获取cr3处存储的level 4页表
//     let (level_4_table_frame, _) = Cr3::read();

//     // 获取虚拟地址的页表的各级地址， 其中后12位为偏移地址，接着的就是4-3-2-1级地址，每个地址9位
//     // 即虚拟地址: |..| 9            |9          |9           |9          |12     |
//     // 依次表示:  |..| 四级页表index |三级页表index|二级页表index|一级页表index|偏移地址|
//     // 那么虚拟地址转为物理地址的过程就是，先从cr3中找到四级页表， 从四级页表中找到四级页表index对应的三级页表地址
//     // 再根据三级页表地址找到三级页表， 从三级页表中找到三级页表index中对应的二级页表地址，依次类推
//     // 最终找到了一级页表index在一级页表中对应的地址， 该地址对应的页面就是目标页面，以该地址为基础， 加上偏移量就是最终的物理地址。
//     let table_indexes = [
//         addr.p4_index(),
//         addr.p3_index(),
//         addr.p2_index(),
//         addr.p1_index(),
//     ];

//     // 因为需要从四级找到一级，所以定义一个临时变量，每个循环都会降到下一级
//     // 目前在四级，注意frame仅仅是一个物理地址，指向对应的页表
//     let mut frame = level_4_table_frame;

//     // 开始寻找从四级寻找
//     for &index in &table_indexes {
//         // 将页表的物理地址转换为虚拟地址
//         let virt = physical_memory_offset + frame.start_address().as_u64();
//         let table_ptr: *const PageTable = virt.as_ptr();

//         // 将虚拟地址墙转为对应的页表
//         let table = unsafe { &*table_ptr };

//         // 获取每级页表对应的entry项，通过该项获取到下级页表地址
//         let entry = &table[index];
//         frame = match entry.frame() {
//             Ok(frame) => frame,
//             Err(FrameError::FrameNotPresent) => return None,
//             Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
//         };
//     }

//     // 最终获取到一级页表中对应的地址，再加上偏移地址即获取到对应的物理地址
//     Some(frame.start_address() + u64::from(addr.page_offset()))
// }
