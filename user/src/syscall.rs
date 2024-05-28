use core::arch::asm;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_SBRK: usize = 214;

#[cfg(target_arch = "riscv64")]
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

#[cfg(target_arch = "aarch64")]
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "svc #0",
            inlateout("x0") args[0] => ret,
            in("x1") args[1],
            in("x2") args[2],
            in("x8") id
        );
    }
    ret
}

#[cfg(target_arch = "x86_64")]
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "
                push r11
                push rcx
                syscall
                pop  rcx
                pop  r11
            ",
            in("rdi") args[0],
            in("rsi") args[1],
            in("rdx") args[2],
            inlateout("rax") id => ret
        );
    }
    ret
}

#[cfg(target_arch = "loongarch64")]
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "syscall 0",
            inlateout("$r4") args[0] => ret,
            in("$r5") args[1],
            in("$r6") args[2],
            in("$r11") id
        );
    }
    ret
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    syscall(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_sbrk(size: i32) -> isize {
    syscall(SYSCALL_SBRK, [size as usize, 0, 0])
}
