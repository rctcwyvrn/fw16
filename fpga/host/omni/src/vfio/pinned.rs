//! Page-aligned anonymous memory mapped into the device's IOMMU domain: the
//! storage behind a vfio [`DmaBuf`](crate::DmaBuf).
//!
//! `VFIO_IOMMU_MAP_DMA` pins the pages (counted against `RLIMIT_MEMLOCK`) and
//! gives the device an IOVA for them. Drop unmaps the IOVA first, then the
//! memory, then returns the IOVA range — never the other way round, or the
//! device could write into pages the process has already given back.

use std::ptr::NonNull;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use vfio_ioctls::VfioContainer;

use super::iova::{IovaAlloc, PAGE};
use crate::error::{Error, Result};

/// What every pinned buffer of one device shares: the IOMMU domain and its
/// IOVA allocator. Buffers hold an `Arc` to it, so they may outlive the
/// device handle and still unmap cleanly.
pub struct Domain {
    pub container: Arc<VfioContainer>,
    pub iova: Mutex<IovaAlloc>,
}

impl Domain {
    // The allocator's state stays consistent across a panic elsewhere, so a
    // poisoned lock is still usable.
    fn iova(&self) -> MutexGuard<'_, IovaAlloc> {
        self.iova.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

pub struct Pinned {
    ptr: NonNull<u64>,
    /// Mapped bytes: the requested length rounded up to whole pages.
    mapped: usize,
    iova: u64,
    domain: Arc<Domain>,
}

// SAFETY: `Pinned` owns its mapping outright; access goes through
// `&self`/`&mut self` like a `Vec`.
unsafe impl Send for Pinned {}
unsafe impl Sync for Pinned {}

impl Pinned {
    pub fn new(domain: &Arc<Domain>, len: usize) -> Result<Pinned> {
        let mapped = len.next_multiple_of(PAGE as usize);
        // SAFETY: a fresh private anonymous mapping, populated so the pages
        // exist before they are pinned.
        let p = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                mapped,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_POPULATE,
                -1,
                0,
            )
        };
        if p == libc::MAP_FAILED {
            return Err(Error::last_os(format!("mmap {mapped} bytes for DMA")));
        }
        let ptr = NonNull::new(p.cast::<u64>()).expect("mmap returned null");
        // SAFETY (both uses): unmapping the mapping made just above.
        let unmap = || unsafe { libc::munmap(p, mapped) };

        let Some(iova) = domain.iova().alloc(mapped as u64) else {
            unmap();
            return Err(Error::Device(format!(
                "IOVA window has no {mapped}-byte hole"
            )));
        };
        // SAFETY: the range is ours and stays mapped until `drop`, which
        // unmaps the IOVA before the memory. Rust references to it exist only
        // between transfers (see `DmaBuf`), when the device is not using it.
        if let Err(e) = unsafe { domain.container.vfio_dma_map(iova, mapped, p.cast()) } {
            domain.iova().free(iova, mapped as u64);
            unmap();
            return Err(e.into());
        }
        Ok(Pinned {
            ptr,
            mapped,
            iova,
            domain: domain.clone(),
        })
    }

    pub fn iova(&self) -> u64 {
        self.iova
    }

    /// Byte 0 of the mapping, for device-shared access without a slice.
    pub fn as_ptr(&self) -> *mut u64 {
        self.ptr.as_ptr()
    }

    /// The first `words` u64s. `words * 8` must not exceed the mapping.
    pub fn words(&self, words: usize) -> &[u64] {
        assert!(words * 8 <= self.mapped);
        // SAFETY: in bounds, aligned (page-aligned mapping), initialised
        // (anonymous pages read as zero).
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), words) }
    }

    pub fn words_mut(&mut self, words: usize) -> &mut [u64] {
        assert!(words * 8 <= self.mapped);
        // SAFETY: as in `words`, and `&mut self` makes it exclusive.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), words) }
    }
}

impl Drop for Pinned {
    fn drop(&mut self) {
        let unmapped = self
            .domain
            .container
            .vfio_dma_unmap(self.iova, self.mapped)
            .is_ok();
        // SAFETY: unmapping exactly what `new` mapped; the IOVA is gone.
        unsafe { libc::munmap(self.ptr.as_ptr().cast(), self.mapped) };
        // An IOVA the kernel would not unmap is leaked rather than handed to
        // the next buffer while it may still point at these pages.
        if unmapped {
            self.domain.iova().free(self.iova, self.mapped as u64);
        }
    }
}
