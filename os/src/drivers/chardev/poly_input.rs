///! Ref: https://www.lammertbies.nl/comm/info/serial-uart
///! Ref: ns16550a datasheet: https://datasheetspdf.com/pdf-file/605590/NationalSemiconductor/NS16550A/1
///! Ref: ns16450 datasheet: https://datasheetspdf.com/pdf-file/1311818/NationalSemiconductor/NS16450/1
use super::CharDevice;
use crate::{sync::UPIntrFreeCell, task::suspend_current_and_run_next};
use alloc::collections::VecDeque;
use polyhal::debug::DebugConsole;

pub struct PolyInput {
    buffer: UPIntrFreeCell<VecDeque<u8>>
}

impl PolyInput {
    pub fn new() -> Self {
        PolyInput {
            buffer: unsafe { UPIntrFreeCell::new(VecDeque::new()) }
        }
    }

    pub fn read_buffer_is_empty(&self) -> bool {
        // self.inner
        //     .exclusive_session(|inner| inner.read_buffer.is_empty())
        if let Some(c) = DebugConsole::getchar() {
            self.buffer.exclusive_access().push_back(c);
            true
        } else {
            false
        }
    }
}

impl CharDevice for PolyInput {
    fn init(&self) {}

    fn read(&self) -> u8 {
        loop {
            if let Some(c) = self.buffer.exclusive_access().pop_front() {
                return c;
            }
            if let Some(c) = DebugConsole::getchar() {
                return c;
            }
            suspend_current_and_run_next();
        }
    }

    fn write(&self, ch: u8) {
        DebugConsole::putchar(ch);
    }
}
