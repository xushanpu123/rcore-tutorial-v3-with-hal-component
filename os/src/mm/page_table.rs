//! Implementation of [`PageTableEntry`] and [`PageTable`].
use _core::{slice, str::from_utf8_unchecked};
use polyhal::{ pagetable::PageTable};
use alloc::string::String;
use bitflags::*;
use crate::alloc::string::ToString;

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

/// doc
pub fn translated_byte_buffer(_token: PageTable, ptr: *mut u8, len: usize) -> &'static mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(ptr, len) }
}

unsafe fn str_len(ptr: *const u8) -> usize {
    let mut i = 0;
    loop {
        if *ptr.add(i) == 0 {
            break i;
        }
        i += 1;
    }
}

/// Load a string from other address spaces into kernel space without an end `\0`.
pub fn translated_str(_token: PageTable, ptr: *const u8) -> String {
    unsafe {
        let len = str_len(ptr);
        from_utf8_unchecked(slice::from_raw_parts(ptr, len)).to_string()
    }
}


pub fn translated_ref<T>(_token: PageTable, ptr: *const T) -> &'static T {
    unsafe { ptr.as_ref().unwrap() }
}

/// doc
pub fn translated_refmut<T>(_token: PageTable, ptr: *mut T) -> &'static mut T {
    unsafe { ptr.as_mut().unwrap() }
}
