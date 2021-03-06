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
// 而定位的方式，则使用一个64位值即可定位(i64)，即64位的值就是一个虚拟地址,其中前12位是偏移地址，后36位是页表地址。
// 因为每个页表存储512个entry， 只需要9位来定位，所以四级页表， 只需要4*9=36位即可定位，

// 四级页表在做地址转换的时候代价昂贵，所以一般会有一个缓存来保存已知的翻译， 叫tlb(translation lookaside buffer).
// 但tlb在页表更新的时候不会自动更新或删除，所以内核在修改页表时需要手动更新tlb.或者可以通过cpu指令invlpg重新加载页表。
// 需要注意的是，开启了分页后，想要直接访问物理地址是不可能的。
