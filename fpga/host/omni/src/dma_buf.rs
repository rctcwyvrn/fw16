//! Library-owned DMA buffers. The transport allocates them so the vfio
//! backend can hand out pinned, IOMMU-mapped memory and DMA straight into it
//! (zero copy); callers only ever see the bytes or the 64-bit words.
//!
//! Lengths are whole 256-bit beats: the shell's streams have no byte
//! granularity, so a partial beat is a caller bug, not a short transfer.

use crate::error::{Error, Result};
use crate::pattern::{BEAT_BYTES, WORDS_PER_BEAT};

pub struct DmaBuf {
    // u64 storage keeps the buffer 8-byte aligned for `words()`. The vfio
    // backend will replace this with page-aligned pinned memory; the fields
    // are private so that change is invisible to callers.
    words: Vec<u64>,
}

impl DmaBuf {
    pub(crate) fn new_host(len: usize) -> Result<Self> {
        check_beats(len)?;
        Ok(DmaBuf {
            words: vec![0; len / 8],
        })
    }

    /// Length in bytes (always a multiple of [`BEAT_BYTES`]).
    pub fn len(&self) -> usize {
        self.words.len() * 8
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// The buffer as little-endian 64-bit words, the way the shell's counter
    /// pattern reads.
    pub fn words(&self) -> &[u64] {
        &self.words
    }

    pub fn words_mut(&mut self) -> &mut [u64] {
        &mut self.words
    }

    /// Beat `i` as four words.
    pub(crate) fn beat(&self, i: usize) -> [u64; WORDS_PER_BEAT] {
        self.words[i * WORDS_PER_BEAT..][..WORDS_PER_BEAT]
            .try_into()
            .unwrap()
    }

    pub(crate) fn set_beat(&mut self, i: usize, beat: [u64; WORDS_PER_BEAT]) {
        self.words[i * WORDS_PER_BEAT..][..WORDS_PER_BEAT].copy_from_slice(&beat);
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
