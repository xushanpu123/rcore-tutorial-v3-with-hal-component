use polyhal::addr::VirtPage;

#[derive(Copy, Clone, Debug)]
pub struct VPNRange {
    l: VirtPage,
    r: VirtPage,
}
impl VPNRange {
    pub fn new(start: VirtPage, end: VirtPage) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }
    pub fn get_start(&self) -> VirtPage {
        self.l
    }
    pub fn get_end(&self) -> VirtPage {
        self.r
    }
}
impl IntoIterator for VPNRange {
    type Item = VirtPage;
    type IntoIter = SimpleRangeIterator;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}
pub struct SimpleRangeIterator {
    current: VirtPage,
    end: VirtPage,
}
impl SimpleRangeIterator {
    pub fn new(l: VirtPage, r: VirtPage) -> Self {
        Self { current: l, end: r }
    }
}
impl Iterator for SimpleRangeIterator {
    type Item = VirtPage;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current = self.current + 1;
            // let t = self.current;
            // self.current.step();
            Some(t)
        }
    }
}
