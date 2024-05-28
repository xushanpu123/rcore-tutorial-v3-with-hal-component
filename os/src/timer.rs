use polyhal::time::Time;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;

/// read the `mtime` register
pub fn get_time() -> usize {
        Time::now().to_msec()/MSEC_PER_SEC
}
/// get current time in milliseconds
pub fn get_time_ms() -> usize {
    Time::now().to_msec() 
}