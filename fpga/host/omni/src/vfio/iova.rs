//! IO virtual addresses for DMA buffers: first-fit over a free list, so
//! buffers freed in a long benchmark loop give their range back.
//!
//! The window starts at 1 MiB and ends below 0xFEE0_0000, the x86 MSI
//! doorbell range that the IOMMU reserves. Staying low also keeps the first
//! 255 MiB of buffers inside QEMU `edu`'s default 28-bit DMA mask, which the
//! VM test relies on. The card's XDMA engine is 64-bit, so the window is a
//! choice, not a hardware limit.

use std::collections::BTreeMap;

pub const PAGE: u64 = 4096;
pub const WINDOW_START: u64 = 0x10_0000;
pub const WINDOW_END: u64 = 0xFEE0_0000;

pub struct IovaAlloc {
    /// Free ranges, start → length. Never adjacent: `free` merges them.
    free: BTreeMap<u64, u64>,
}

impl IovaAlloc {
    pub fn new(start: u64, end: u64) -> IovaAlloc {
        assert!(start.is_multiple_of(PAGE) && end.is_multiple_of(PAGE) && start < end);
        IovaAlloc {
            free: BTreeMap::from([(start, end - start)]),
        }
    }

    /// A page-aligned range of `len` bytes (rounded up to whole pages), or
    /// `None` when the window has no hole that big.
    pub fn alloc(&mut self, len: u64) -> Option<u64> {
        let len = len.max(1).next_multiple_of(PAGE);
        let (&start, &hole) = self.free.iter().find(|&(_, &hole)| hole >= len)?;
        self.free.remove(&start);
        if hole > len {
            self.free.insert(start + len, hole - len);
        }
        Some(start)
    }

    /// Return a range from [`alloc`](Self::alloc), with the same `len`.
    pub fn free(&mut self, start: u64, len: u64) {
        let mut start = start;
        let mut len = len.max(1).next_multiple_of(PAGE);
        if let Some((&prev, &prev_len)) = self.free.range(..start).next_back()
            && prev + prev_len == start
        {
            self.free.remove(&prev);
            start = prev;
            len += prev_len;
        }
        if let Some(next_len) = self.free.remove(&(start + len)) {
            len += next_len;
        }
        self.free.insert(start, len);
    }
}

impl Default for IovaAlloc {
    fn default() -> IovaAlloc {
        IovaAlloc::new(WINDOW_START, WINDOW_END)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_to_pages_and_packs_from_the_bottom() {
        let mut a = IovaAlloc::default();
        assert_eq!(a.alloc(1), Some(WINDOW_START));
        assert_eq!(a.alloc(PAGE + 1), Some(WINDOW_START + PAGE));
        assert_eq!(a.alloc(PAGE), Some(WINDOW_START + 3 * PAGE));
    }

    #[test]
    fn freed_ranges_merge_and_are_reused() {
        let mut a = IovaAlloc::new(0, 8 * PAGE);
        let x = a.alloc(2 * PAGE).unwrap();
        let y = a.alloc(2 * PAGE).unwrap();
        let z = a.alloc(2 * PAGE).unwrap();
        a.free(x, 2 * PAGE);
        a.free(z, 2 * PAGE);
        // x's hole is too small for 4 pages; z merged with the tail fits it.
        assert_eq!(a.alloc(4 * PAGE), Some(z));
        a.free(z, 4 * PAGE);
        a.free(y, 2 * PAGE);
        // Everything merged back into one range.
        assert_eq!(a.alloc(8 * PAGE), Some(0));
    }

    #[test]
    fn exhaustion_is_none() {
        let mut a = IovaAlloc::new(0, 2 * PAGE);
        assert!(a.alloc(3 * PAGE).is_none());
        assert!(a.alloc(2 * PAGE).is_some());
        assert!(a.alloc(1).is_none());
    }
}
