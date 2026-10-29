//! `mock:` — an in-process model of the card: shell v0 plus the proposed v0.2
//! IRQ block, behind the same [`Transport`] as the real card.
//!
//! Everything is synchronous and single-threaded, so data can only arrive
//! inside a call: when a transfer cannot finish (empty ring, loopback ring
//! full, no interrupt pending) the mock returns `Error::Timeout` at once
//! instead of sleeping out the timeout. There is no XDMA model yet (Path 4
//! step 5); DMA moves beats straight in and out of the shell model.

mod irq;
mod pattern_state;
mod shell;
#[cfg(test)]
mod tests;

use std::time::Duration;

use crate::dma_buf::DmaBuf;
use crate::error::{Error, Result};
use crate::regs::proposed::{OMNI_IRQ_BASE, OMNI_IRQ_EN_SNK_ERR_MASK};
use crate::transport::Transport;
use irq::Irq;
use shell::Shell;

pub use shell::RING_DEPTH;

pub struct Mock {
    shell: Shell,
    irq: Irq,
    /// Fault injection: flip bit 0 of every n-th C2H beat delivered.
    corrupt_c2h_every: Option<u64>,
    c2h_beats: u64,
}

impl Mock {
    pub fn new() -> Mock {
        Mock {
            shell: Shell::new(),
            irq: Irq::default(),
            corrupt_c2h_every: None,
            c2h_beats: 0,
        }
    }

    /// Corrupt every `n`-th beat the host receives over C2H (n ≥ 1), to prove
    /// a checker counts what it should.
    pub fn corrupt_c2h_every(mut self, n: u64) -> Mock {
        self.corrupt_c2h_every = Some(n.max(1));
        self
    }

    fn is_irq_block(offset: u32) -> bool {
        offset & !0xFFF == OMNI_IRQ_BASE
    }
}

impl Default for Mock {
    fn default() -> Self {
        Mock::new()
    }
}

fn check_aligned(offset: u32) -> Result<()> {
    if !offset.is_multiple_of(4) {
        return Err(Error::InvalidArgument(format!(
            "register offset {offset:#x} not 4-byte aligned"
        )));
    }
    Ok(())
}

impl Transport for Mock {
    fn read32(&mut self, offset: u32) -> Result<u32> {
        check_aligned(offset)?;
        Ok(if Mock::is_irq_block(offset) {
            self.irq.read(offset)
        } else {
            self.shell.read(offset)
        })
    }

    fn write32(&mut self, offset: u32, value: u32) -> Result<()> {
        check_aligned(offset)?;
        if Mock::is_irq_block(offset) {
            self.irq.write(offset, value);
        } else {
            self.shell.write(offset, value);
        }
        Ok(())
    }

    fn alloc(&mut self, len: usize) -> Result<DmaBuf> {
        DmaBuf::new_host(len)
    }

    fn dma_read(&mut self, buf: &mut DmaBuf, len: usize, _timeout: Duration) -> Result<()> {
        buf.check_len(len)?;
        for i in 0..len / crate::pattern::BEAT_BYTES {
            let Some(mut beat) = self.shell.c2h_pop() else {
                return Err(Error::Timeout {
                    done: i * crate::pattern::BEAT_BYTES,
                });
            };
            self.c2h_beats += 1;
            if self
                .corrupt_c2h_every
                .is_some_and(|n| self.c2h_beats.is_multiple_of(n))
            {
                beat[0] ^= 1;
            }
            buf.set_beat(i, beat);
        }
        Ok(())
    }

    fn dma_write(&mut self, buf: &DmaBuf, len: usize, _timeout: Duration) -> Result<()> {
        buf.check_len(len)?;
        for i in 0..len / crate::pattern::BEAT_BYTES {
            let had_errors = self.shell.snk_errs() != 0;
            if !self.shell.h2c_push(buf.beat(i)) {
                return Err(Error::Timeout {
                    done: i * crate::pattern::BEAT_BYTES,
                });
            }
            // SNK_ERR interrupt source: the 0→1 edge of STATUS.SNK_ERR.
            if !had_errors && self.shell.snk_errs() != 0 {
                self.irq.raise(OMNI_IRQ_EN_SNK_ERR_MASK);
            }
        }
        Ok(())
    }

    fn wait_irq(&mut self, vector: u32, _timeout: Duration) -> Result<()> {
        if vector != 0 {
            return Err(Error::InvalidArgument(format!(
                "mock has one interrupt vector, not {vector}"
            )));
        }
        if self.irq.take() {
            Ok(())
        } else {
            Err(Error::Timeout { done: 0 })
        }
    }
}
