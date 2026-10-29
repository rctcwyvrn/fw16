//! `mock:` — an in-process model of the card: shell regmap v0.2 (with its
//! IRQ block) behind an XDMA model, behind the same [`Transport`] as the
//! real card.
//!
//! DMA runs the real engine ([`crate::xdma::Channel`]) against the XDMA
//! model (`xdma.rs`, `xdma_chan.rs`): the same chains, register sequence
//! and writebacks the card sees, with buffers known to the model by made-up
//! bus addresses. `alloc` hands those out.
//!
//! Everything is synchronous, so data can only arrive inside a call: when a
//! transfer cannot finish (empty ring, loopback ring full, no interrupt
//! pending) the mock returns `Error::Timeout` at once instead of sleeping
//! out the timeout. As on the card, a timed-out transfer's `done` counts
//! whole descriptors (4 KiB). The whole model sits behind one `Mutex`, so
//! calls from several threads serialise; each transfer holds the lock for
//! its whole length.

mod irq;
mod pattern_state;
mod shell;
#[cfg(test)]
mod tests;
mod xdma;
mod xdma_chan;
#[cfg(test)]
mod xdma_tests;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use crate::dma_buf::DmaBuf;
use crate::error::{Error, Result};
use crate::pattern::Beat;
use crate::regs::{FW16_FPGA_IRQ_BASE, FW16_FPGA_IRQ_EN_SNK_ERR_MASK};
use crate::transport::Transport;
use crate::xdma::{Channel, Data, DeviceMem, Dir, Port};
use irq::Irq;
use shell::Shell;
use xdma::Xdma;
use xdma_chan::Streams;

pub use shell::RING_DEPTH;

/// Where the mock's made-up bus addresses start (above 4 GiB, so the
/// engine's `_HI` register writes are exercised).
const IOVA_BASE: u64 = 0x1_0000_0000;
const PAGE: u64 = 4096;

pub struct Mock {
    state: Mutex<State>,
    next_iova: AtomicU64,
}

struct State {
    card: CardSide,
    xdma: Xdma,
    h2c: Channel,
    c2h: Channel,
}

/// The shell and what sits on its stream ports.
struct CardSide {
    shell: Shell,
    irq: Irq,
    /// Fault injection: flip bit 0 of every n-th C2H beat delivered.
    corrupt_c2h_every: Option<u64>,
    c2h_beats: u64,
}

impl Mock {
    pub fn new() -> Mock {
        let mut m = Mock {
            state: Mutex::new(State {
                card: CardSide {
                    shell: Shell::new(),
                    irq: Irq::default(),
                    corrupt_c2h_every: None,
                    c2h_beats: 0,
                },
                xdma: Xdma::new(),
                h2c: Channel::new(Dir::H2c),
                c2h: Channel::new(Dir::C2h),
            }),
            next_iova: AtomicU64::new(IOVA_BASE),
        };
        // What `Card::open` does: without it the XDMA forwards no interrupt.
        let s = m.state_mut();
        crate::xdma::init(&mut MockPort {
            xdma: &mut s.xdma,
            card: &mut s.card,
        })
        .expect("the XDMA model identifies as XDMA");
        m
    }

    /// Corrupt every `n`-th beat the host receives over C2H (n ≥ 1), to prove
    /// a checker counts what it should.
    pub fn corrupt_c2h_every(mut self, n: u64) -> Mock {
        self.state_mut().card.corrupt_c2h_every = Some(n.max(1));
        self
    }

    // A panic inside the model (a bug) poisons the lock; the model's state
    // is still whatever it was, so carry on rather than panic every caller.
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn state_mut(&mut self) -> &mut State {
        self.state.get_mut().unwrap_or_else(PoisonError::into_inner)
    }

    fn is_irq_block(offset: u32) -> bool {
        offset & !0xFFF == FW16_FPGA_IRQ_BASE
    }

    fn new_buf(&self, len: usize) -> Result<DmaBuf> {
        let span = (len as u64).next_multiple_of(PAGE);
        let iova = self.next_iova.fetch_add(span, Ordering::Relaxed);
        DmaBuf::new_host(len, iova)
    }

    /// A BAR1 (XDMA) register, for tests of the model itself.
    #[cfg(test)]
    fn bar1_write(&self, offset: u32, value: u32) {
        self.state().xdma.write(offset, value);
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

impl Streams for CardSide {
    fn h2c(&mut self, beat: Beat) -> bool {
        let had_errors = self.shell.snk_errs() != 0;
        if !self.shell.h2c_push(beat) {
            return false;
        }
        // SNK_ERR interrupt source: the 0→1 edge of STATUS.SNK_ERR.
        if !had_errors && self.shell.snk_errs() != 0 {
            self.irq.raise(FW16_FPGA_IRQ_EN_SNK_ERR_MASK);
        }
        true
    }

    fn c2h(&mut self) -> Option<Beat> {
        let mut beat = self.shell.c2h_pop()?;
        self.c2h_beats += 1;
        if self
            .corrupt_c2h_every
            .is_some_and(|n| self.c2h_beats.is_multiple_of(n))
        {
            beat[0] ^= 1;
        }
        Some(beat)
    }
}

/// BAR1 is the model; waiting runs it until it stalls or stops.
struct MockPort<'a> {
    xdma: &'a mut Xdma,
    card: &'a mut CardSide,
}

impl Port for MockPort<'_> {
    fn read32(&mut self, offset: u32) -> Result<u32> {
        Ok(self.xdma.read(offset))
    }

    fn write32(&mut self, offset: u32, value: u32) -> Result<()> {
        self.xdma.write(offset, value);
        Ok(())
    }

    fn wait(&mut self, mem: &DeviceMem<'_>, _deadline: Instant) -> Result<bool> {
        self.xdma.run(mem, self.card);
        Ok(false)
    }
}

impl Transport for Mock {
    fn read32(&self, offset: u32) -> Result<u32> {
        check_aligned(offset)?;
        let s = &mut self.state().card;
        Ok(if Mock::is_irq_block(offset) {
            s.irq.read(offset)
        } else {
            s.shell.read(offset)
        })
    }

    fn write32(&self, offset: u32, value: u32) -> Result<()> {
        check_aligned(offset)?;
        let s = &mut self.state().card;
        if Mock::is_irq_block(offset) {
            s.irq.write(offset, value);
        } else {
            s.shell.write(offset, value);
        }
        Ok(())
    }

    fn alloc(&self, len: usize) -> Result<DmaBuf> {
        self.new_buf(len)
    }

    fn dma_read(&self, buf: &mut DmaBuf, len: usize, timeout: Duration) -> Result<()> {
        let s = &mut *self.state();
        let mut port = MockPort {
            xdma: &mut s.xdma,
            card: &mut s.card,
        };
        s.c2h.transfer(
            &mut port,
            &|n| self.new_buf(n),
            Data::C2h(buf),
            len,
            timeout,
        )
    }

    fn dma_write(&self, buf: &DmaBuf, len: usize, timeout: Duration) -> Result<()> {
        let s = &mut *self.state();
        let mut port = MockPort {
            xdma: &mut s.xdma,
            card: &mut s.card,
        };
        s.h2c.transfer(
            &mut port,
            &|n| self.new_buf(n),
            Data::H2c(buf),
            len,
            timeout,
        )
    }

    fn wait_irq(&self, vector: u32, _timeout: Duration) -> Result<()> {
        if vector != 0 {
            return Err(Error::InvalidArgument(format!(
                "mock has one interrupt vector, not {vector}"
            )));
        }
        let s = &mut *self.state();
        // A message the XDMA does not forward (USER_INT_EN clear, or routed
        // to another vector) is lost, as on the card.
        let delivered = s.card.irq.take();
        if delivered && s.xdma.user_vector() == Some(vector) {
            Ok(())
        } else {
            Err(Error::Timeout { done: 0 })
        }
    }
}
