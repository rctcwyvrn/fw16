//! What the engine needs from whoever hosts the XDMA: its BAR1 registers,
//! and a way to let the device work.
//!
//! On the card the device runs by itself and [`Port::wait`] only paces the
//! polling. The mock's XDMA model has no thread of its own, so `wait` is
//! where it runs, and it reaches host memory through the [`DeviceMem`] the
//! engine hands it: the transfer's buffers, by bus address, and nothing
//! else. An address outside them is a DMA fault, which is how the model
//! catches a wrong chain.

use std::marker::PhantomData;
use std::time::Instant;

use crate::dma_buf::DmaBuf;
use crate::error::Result;

pub trait Port {
    /// A BAR1 register.
    fn read32(&mut self, offset: u32) -> Result<u32>;
    fn write32(&mut self, offset: u32, value: u32) -> Result<()>;

    /// Give the device a moment, up to `deadline`. Returns `Ok(false)` when
    /// the device can make no more progress whatever the wait (the mock,
    /// once it stalls); an error aborts the transfer (e.g. a suspend).
    fn wait(&mut self, mem: &DeviceMem<'_>, deadline: Instant) -> Result<bool>;
}

struct Region {
    iova: u64,
    ptr: *mut u8,
    len: usize,
    writable: bool,
}

/// The buffers one transfer lets the device touch. Built fresh for each
/// [`Port::wait`] from borrows the engine holds, so the pointers are live
/// for its lifetime `'a`.
pub struct DeviceMem<'a> {
    regions: Vec<Region>,
    _borrows: PhantomData<&'a mut DmaBuf>,
}

/// The device touched memory outside the transfer's buffers, or wrote a
/// read-only one.
#[derive(Debug, PartialEq, Eq)]
pub struct Fault;

impl<'a> DeviceMem<'a> {
    pub fn new() -> Self {
        DeviceMem {
            regions: Vec::new(),
            _borrows: PhantomData,
        }
    }

    /// A buffer the device may read and write. Buffers without a bus
    /// address are skipped (the device cannot name them).
    pub fn rw(mut self, buf: &'a mut DmaBuf) -> Self {
        if let Some(iova) = buf.iova() {
            let len = buf.len();
            let ptr = buf.as_mut_ptr().cast();
            self.regions.push(Region {
                iova,
                ptr,
                len,
                writable: true,
            });
        }
        self
    }

    /// A buffer the device may only read.
    pub fn ro(mut self, buf: &'a DmaBuf) -> Self {
        if let Some(iova) = buf.iova() {
            self.regions.push(Region {
                iova,
                ptr: buf.as_ptr().cast_mut().cast(),
                len: buf.len(),
                writable: false,
            });
        }
        self
    }

    fn find(&self, iova: u64, len: usize, write: bool) -> std::result::Result<*mut u8, Fault> {
        let r = self
            .regions
            .iter()
            .find(|r| iova >= r.iova && iova + len as u64 <= r.iova + r.len as u64)
            .ok_or(Fault)?;
        if write && !r.writable {
            return Err(Fault);
        }
        // SAFETY: the range was just checked to lie inside the region.
        Ok(unsafe { r.ptr.add((iova - r.iova) as usize) })
    }

    /// A device read of `out.len()` bytes at `iova`.
    pub fn read(&self, iova: u64, out: &mut [u8]) -> std::result::Result<(), Fault> {
        let p = self.find(iova, out.len(), false)?;
        // SAFETY: `find` bounds-checked it; the region is borrowed for 'a.
        unsafe { std::ptr::copy_nonoverlapping(p, out.as_mut_ptr(), out.len()) };
        Ok(())
    }

    /// A device write of `data` at `iova`.
    pub fn write(&self, iova: u64, data: &[u8]) -> std::result::Result<(), Fault> {
        let p = self.find(iova, data.len(), true)?;
        // SAFETY: as in `read`, and the region was lent mutably.
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), p, data.len()) };
        Ok(())
    }
}

impl Default for DeviceMem<'_> {
    fn default() -> Self {
        DeviceMem::new()
    }
}
