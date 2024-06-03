mod inode;
mod pipe;
mod stdio;

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: &mut [u8]) -> usize;
    fn write(&self, buf: &[u8]) -> usize;
}

pub use inode::{list_apps, open_file, OpenFlags};
pub use pipe::make_pipe;
pub use stdio::{Stdin, Stdout};
