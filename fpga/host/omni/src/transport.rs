//! The one interface between tools and a card (real, mocked or simulated).
//!
//! Registers are BAR0 byte offsets from [`crate::regs`]. DMA is the shell's
//! two streams: `dma_read` is C2H (card → host, the ring's output) and
//! `dma_write` is H2C (host → card, the sink and, in loopback, the ring).
//! Closing is `Drop`.

use std::time::Duration;

use crate::dma_buf::DmaBuf;
use crate::error::Result;

pub trait Transport: Send {
    /// Read a 32-bit register. `offset` must be 4-byte aligned.
    fn read32(&mut self, offset: u32) -> Result<u32>;

    /// Write a 32-bit register. Writes to read-only or unmapped offsets are
    /// dropped by the shell, not reported.
    fn write32(&mut self, offset: u32, value: u32) -> Result<()>;

    /// Allocate a DMA buffer of `len` bytes (whole beats).
    fn alloc(&mut self, len: usize) -> Result<DmaBuf>;

    /// C2H: fill `buf[..len]` from the card's stream. On timeout returns
    /// `Error::Timeout { done }` with the bytes that did arrive.
    fn dma_read(&mut self, buf: &mut DmaBuf, len: usize, timeout: Duration) -> Result<()>;

    /// H2C: send `buf[..len]` to the card. On timeout returns
    /// `Error::Timeout { done }` with the bytes the card accepted.
    fn dma_write(&mut self, buf: &DmaBuf, len: usize, timeout: Duration) -> Result<()>;

    /// Block until interrupt `vector` fires (one MSI-X message), or time out.
    fn wait_irq(&mut self, vector: u32, timeout: Duration) -> Result<()>;
}
