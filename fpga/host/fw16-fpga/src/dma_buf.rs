//! Library-owned DMA buffers. The transport allocates them so the vfio
//! backend can hand out pinned, IOMMU-mapped memory and DMA straight into it
//! (zero copy); callers only ever see the bytes or the 64-bit words.
//!
//! Lengths are whole 256-bit beats: the shell's streams have no byte
//! granularity, so a partial beat is a caller bug, not a short transfer.
//!
//! A pinned buffer is reachable by the device for its whole life, but the
//! device only touches it during a transfer, and a transfer borrows the
//! buffer (`&mut` for C2H), so the slices handed out here are never live
//! while the device writes. Code that starts DMA by hand (the `edu` VM test)
//! must keep to the same rule.

use crate::error::{Error, Result};
use crate::pattern::BEAT_BYTES;
use crate::vfio::Pinned;

pub struct DmaBuf {
    /// Length in u64 words; the pinned mapping may be longer (whole pages).
    words: usize,
    storage: Storage,
}

enum Storage {
    // u64 storage keeps the buffer 8-byte aligned for `words()`. `iova` is
    // the bus address the mock's XDMA model knows it by.
    Host { mem: Vec<u64>, iova: u64 },
    Pinned(Pinned),
}

impl DmaBuf {
    /// Plain memory with a made-up bus address `iova` (the mock's).
    pub(crate) fn new_host(len: usize, iova: u64) -> Result<Self> {
        check_beats(len)?;
        Ok(DmaBuf {
            words: len / 8,
            storage: Storage::Host {
                mem: vec![0; len / 8],
                iova,
            },
        })
    }

    /// A buffer backed by `pin(len)`, after checking `len`.
    pub(crate) fn new_pinned(
        len: usize,
        pin: impl FnOnce(usize) -> Result<Pinned>,
    ) -> Result<Self> {
        check_beats(len)?;
        Ok(DmaBuf {
            words: len / 8,
            storage: Storage::Pinned(pin(len)?),
        })
    }

    /// Length in bytes (always a multiple of [`BEAT_BYTES`]).
    pub fn len(&self) -> usize {
        self.words * 8
    }

    pub fn is_empty(&self) -> bool {
        self.words == 0
    }

    /// The bus address the device uses for byte 0: the IOMMU address for
    /// vfio buffers, a made-up one the mock's XDMA model resolves for mock
    /// buffers. `Option` because a transport may one day hand out buffers
    /// no device can name.
    pub fn iova(&self) -> Option<u64> {
        match &self.storage {
            Storage::Host { iova, .. } => Some(*iova),
            Storage::Pinned(p) => Some(p.iova()),
        }
    }

    /// The buffer as little-endian 64-bit words, the way the shell's counter
    /// pattern reads.
    pub fn words(&self) -> &[u64] {
        match &self.storage {
            Storage::Host { mem, .. } => mem,
            Storage::Pinned(p) => p.words(self.words),
        }
    }

    pub fn words_mut(&mut self) -> &mut [u64] {
        match &mut self.storage {
            Storage::Host { mem, .. } => mem,
            Storage::Pinned(p) => p.words_mut(self.words),
        }
    }

    /// Byte 0, without making a slice: for memory the device may write
    /// while the pointer is in use (see [`read_volatile_u32`]).
    ///
    /// [`read_volatile_u32`]: Self::read_volatile_u32
    pub(crate) fn as_ptr(&self) -> *const u64 {
        match &self.storage {
            Storage::Host { mem, .. } => mem.as_ptr(),
            Storage::Pinned(p) => p.as_ptr(),
        }
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut u64 {
        match &mut self.storage {
            Storage::Host { mem, .. } => mem.as_mut_ptr(),
            Storage::Pinned(p) => p.as_ptr(),
        }
    }

    /// The u32 at byte `offset`, read the way a polled writeback must be:
    /// volatile, so each call really looks at memory the device writes.
    pub(crate) fn read_volatile_u32(&self, offset: usize) -> u32 {
        assert!(offset.is_multiple_of(4) && offset + 4 <= self.len());
        // SAFETY: in bounds and aligned (just checked; storage is 8-aligned).
        unsafe {
            self.as_ptr()
                .cast::<u8>()
                .add(offset)
                .cast::<u32>()
                .read_volatile()
        }
    }

    /// Checks a transfer length against this buffer.
    pub(crate) fn check_len(&self, len: usize) -> Result<()> {
        check_beats(len)?;
        if len > self.len() {
            return Err(Error::InvalidArgument(format!(
                "transfer of {len} bytes from a {}-byte buffer",
                self.len()
            )));
        }
        Ok(())
    }
}

fn check_beats(len: usize) -> Result<()> {
    if len == 0 || !len.is_multiple_of(BEAT_BYTES) {
        return Err(Error::InvalidArgument(format!(
            "{len} bytes is not a whole, non-zero number of {BEAT_BYTES}-byte beats"
        )));
    }
    Ok(())
}
