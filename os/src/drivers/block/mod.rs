#[cfg(any(target_arch = "x86_64", target_arch = "loongarch64"))]
mod ram_blk;

#[cfg(any(target_arch = "riscv64", target_arch = "aarch64"))]
mod virtio_blk;
#[cfg(any(target_arch = "riscv64", target_arch = "aarch64"))]
pub use virtio_blk::VirtIOBlock;

use alloc::sync::Arc;
use easy_fs::BlockDevice;
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

#[allow(unused)]
pub fn block_device_test() {
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    for i in 0..512 {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as usize, &write_buffer);
        block_device.read_block(i as usize, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    println!("block device test passed!");
}
