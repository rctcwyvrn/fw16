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
use crate::pattern::{BEAT_BYTES, WORDS_PER_BEAT};
use crate::vfio::Pinned;

pub struct DmaBuf {
    /// Length in u64 words; the pinned mapping may be longer (whole pages).
    words: usize,
    storage: Storage,
}

enum Storage {
    // u64 storage keeps the buffer 8-byte aligned for `words()`.
    Host(Vec<u64>),
    Pinned(Pinned),
}

impl DmaBuf {
    pub(crate) fn new_host(len: usize) -> Result<Self> {
        check_beats(len)?;
        Ok(DmaBuf {
            words: len / 8,
            storage: Storage::Host(vec![0; len / 8]),
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

    /// The bus address the device uses for byte 0, for buffers from the
    /// vfio backend; `None` for host-only (mock) buffers.
    pub fn iova(&self) -> Option<u64> {
        match &self.storage {
            Storage::Host(_) => None,
            Storage::Pinned(p) => Some(p.iova()),
        }
    }

    /// The buffer as little-endian 64-bit words, the way the shell's counter
    /// pattern reads.
    pub fn words(&self) -> &[u64] {
        match &self.storage {
            Storage::Host(v) => v,
            Storage::Pinned(p) => p.words(self.words),
        }
    }

    pub fn words_mut(&mut self) -> &mut [u64] {
        match &mut self.storage {
            Storage::Host(v) => v,
            Storage::Pinned(p) => p.words_mut(self.words),
        }
    }

    /// Beat `i` as four words.
    pub(crate) fn beat(&self, i: usize) -> [u64; WORDS_PER_BEAT] {
        self.words()[i * WORDS_PER_BEAT..][..WORDS_PER_BEAT]
            .try_into()
            .unwrap()
    }

    pub(crate) fn set_beat(&mut self, i: usize, beat: [u64; WORDS_PER_BEAT]) {
        self.words_mut()[i * WORDS_PER_BEAT..][..WORDS_PER_BEAT].copy_from_slice(&beat);
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
