//! The one interface between tools and a card (real, mocked or simulated).
//!
//! Registers are BAR0 byte offsets from [`crate::regs`]. DMA is the shell's
//! two streams: `dma_read` is C2H (card → host, the ring's output) and
//! `dma_write` is H2C (host → card, the sink and, in loopback, the ring).
//! Closing is `Drop`.
//!
//! Every method takes `&self` and transports are `Sync`: each backend locks
//! what it must inside, so one handle can run H2C and C2H on two threads.

use std::time::Duration;

use crate::dma_buf::DmaBuf;
use crate::error::Result;

pub trait Transport: Send + Sync {
    /// Read a 32-bit register. `offset` must be 4-byte aligned.
    fn read32(&self, offset: u32) -> Result<u32>;

    /// Write a 32-bit register. Writes to read-only or unmapped offsets are
    /// dropped by the shell, not reported.
    fn write32(&self, offset: u32, value: u32) -> Result<()>;

    /// Allocate a DMA buffer of `len` bytes (whole beats).
    fn alloc(&self, len: usize) -> Result<DmaBuf>;

    /// C2H: fill `buf[..len]` from the card's stream. On timeout returns
    /// `Error::Timeout { done }` with the bytes that did arrive.
    fn dma_read(&self, buf: &mut DmaBuf, len: usize, timeout: Duration) -> Result<()>;

    /// H2C: send `buf[..len]` to the card. On timeout returns
    /// `Error::Timeout { done }` with the bytes the card accepted.
    fn dma_write(&self, buf: &DmaBuf, len: usize, timeout: Duration) -> Result<()>;

    /// Block until interrupt `vector` fires (one MSI-X message), or time out.
    fn wait_irq(&self, vector: u32, timeout: Duration) -> Result<()>;
}
