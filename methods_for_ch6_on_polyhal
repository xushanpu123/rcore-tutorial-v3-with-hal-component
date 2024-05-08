基于polyhal的rcore tutorial ch6支持

我们需要先分析哪些部分需要用到底层硬件相关的功能，将其替换为利用polyhal接口实现即可。

1、fs模块

fs/inode.rs是定义OSInode的代码，只在内核的内存中构建，与硬件层无关，因此不需要有任何改变，fs/mod.rs同理。

fs/stdio.rs涉及到获取标准输入，这在polyhal中有接口实现，因此使用接口来完成这个功能：

```rust
        let c: u8;
        loop {
            if let Some(ch) = DebugConsole::getchar() {
                c = ch;
                break;
            }
            suspend_current_and_run_next();
        }
```

这里借鉴了杨金博的实现，在没有获取输入数据时把自己调度走。



2、drivers模块：

与只支持riscv64的drivers模块不同，ch6 on polyhal必须依据不同的架构使用不同的底层BlockDevice：

```rust
#[cfg(any(target_arch = "x86_64", target_arch = "loongarch64"))]
mod ram_blk;

#[cfg(any(target_arch = "riscv64", target_arch = "aarch64"))]
mod virtio_blk;
#[cfg(any(target_arch = "riscv64", target_arch = "aarch64"))]
pub use virtio_blk::VirtIOBlock;

use alloc::sync::Arc;
use easyfs::BlockDevice;
use lazy_static::*;

#[cfg(any(target_arch = "x86_64", target_arch = "loongarch64"))]
use ram_blk::RamDiskBlock;

#[cfg(any(target_arch = "riscv64", target_arch = "aarch64"))]
lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(VirtIOBlock::new());
}

#[cfg(any(target_arch = "x86_64", target_arch = "loongarch64"))]
lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(RamDiskBlock::new());
}
```

virtio_blk.rs驱动提供上层接口的代码与hal无关，因此不做修改，在virtio_blk.rs中涉及到了连续页帧分配的工作，由于polyhal有自己独立的Page定义方式，因此必须做一定的修改：

```rust
       let mut ppn_base = PhysPage::new(0);
        for i in 0..pages {
            let frame = frame_alloc().unwrap();
            debug!("alloc paddr: {:?}", frame);
            if i == 0 {
                ppn_base = frame.ppn
            };
            assert_eq!(frame.ppn.as_num(), ppn_base.as_num() + i);
            QUEUE_FRAMES.exclusive_access().push(frame);
        }
        let pa: usize = ppn_base.to_addr();
        unsafe {
            (
                pa,
                NonNull::new_unchecked((pa | VIRT_ADDR_START) as *mut u8),
            )
        }
    }

    unsafe fn dma_dealloc(paddr: usize, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        // trace!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
        log::error!("dealloc paddr: {:?}", paddr);
        let pa = PhysAddr::new(paddr);
        let mut ppn_base: PhysPage = pa.into();
        for _ in 0..pages {
            frame_dealloc(ppn_base);
            ppn_base = ppn_base + 1;
        }
        0
    }
```

ram_blk.rs为新引入的BlockDevice：RamDiskBlock的驱动接口，借鉴了杨金博的ch7实现。