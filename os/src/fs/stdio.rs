use log::warn;

use super::File;
use crate::drivers::chardev::CharDevice;
use crate::drivers::chardev::UART;

use log::info;
use polyhal::{addr::VirtAddr, debug::DebugConsole, pagetable::PageTable};

use crate::task::{current_process, suspend_current_and_run_next};


pub struct Stdin;
pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, mut user_buf: &mut [u8]) -> usize {
        let ch = UART.read();
        unsafe {
            user_buf.as_mut_ptr().write_volatile(ch);
        }
        1
    }
    fn write(&self, _user_buf: &mut [u8]) -> usize {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }
    fn read(&self, _user_buf: &mut [u8]) -> usize {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: &mut [u8]) -> usize {
        print!("{}", core::str::from_utf8(user_buf).unwrap());
        1
    }
}
