//! File and filesystem-related syscalls
use crate::mm::translated_byte_buffer;
use polyhal::debug::DebugConsole;
use crate::task::{current_user_token, suspend_current_and_run_next};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffer = translated_byte_buffer(current_user_token(), buf as _, len);
            print!("{}", core::str::from_utf8(buffer).unwrap());
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read!");
            let c: u8;
            loop {
                if let Some(ch) = DebugConsole::getchar() {
                    c = ch;
                    break;
                }
                suspend_current_and_run_next();
            }
            let buffer = translated_byte_buffer(current_user_token(), buf as _, len);
            unsafe {
                buffer.as_mut_ptr().write_volatile(c);
            }
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}
