use polyhal::debug::DebugConsole;

use super::File;
use crate::task::suspend_current_and_run_next;

pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, user_buf: &mut [u8]) -> usize {
        assert_eq!(user_buf.len(), 1);
        // busy loop
        let c: u8;
        loop {
            if let Some(ch) = DebugConsole::getchar() {
                c = ch;
                break;
            }
            suspend_current_and_run_next();
        }
        user_buf[0] = c as u8;
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
        // for buffer in user_buf.buffers.iter() {
        //     print!("{}", core::str::from_utf8(*buffer).unwrap());
        // }
        print!("{}", core::str::from_utf8(user_buf).unwrap());
        user_buf.len()
    }
}
