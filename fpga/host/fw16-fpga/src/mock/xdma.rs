//! The mock's XDMA: BAR1 as PG195 lays it out (channel 0 of H2C and C2H,
//! their SGDMA blocks, the IRQ block), with the descriptor walks in
//! `xdma_chan.rs`. Other channels and blocks read 0 and drop writes.
//!
//! The IRQ block gates the shell's interrupt the way the card does: a
//! `usr_irq_req[0]` edge becomes a message only while `USER_INT_EN[0]` is
//! set, and it goes to the vector in `USER_VEC_NUM[4:0]`. So the mock
//! interrupts only after `xdma::init` has run, as the card would.

use super::xdma_chan::{ChanModel, Streams};
use crate::xdma::DeviceMem;
use crate::xdma::desc::Dir;
use crate::xdma::regs::*;

pub(super) struct Xdma {
    h2c: ChanModel,
    c2h: ChanModel,
    user_int_en: u32,
    chan_int_en: u32,
    user_vec: u32,
}

impl Xdma {
    pub fn new() -> Xdma {
        Xdma {
            h2c: ChanModel::new(Dir::H2c),
            c2h: ChanModel::new(Dir::C2h),
            user_int_en: 0,
            chan_int_en: 0,
            user_vec: 0,
        }
    }

    /// The vector a shell interrupt goes to, or `None` while it is masked.
    pub fn user_vector(&self) -> Option<u32> {
        (self.user_int_en & 1 != 0).then_some(self.user_vec & 0x1F)
    }

    pub fn read(&mut self, offset: u32) -> u32 {
        let (block, chan, off) = split(offset);
        match (block, chan) {
            (0, 0) => self.h2c.read(off),
            (1, 0) => self.c2h.read(off),
            (4, 0) => self.h2c.read_sgdma(off),
            (5, 0) => self.c2h.read_sgdma(off),
            (2, _) => match offset & 0xFFF {
                IRQ_IDENTIFIER => ID_MAGIC << ID_MAGIC_SHIFT | TARGET_IRQ << ID_TARGET_SHIFT | 6,
                IRQ_USER_INT_EN => self.user_int_en,
                IRQ_CHAN_INT_EN => self.chan_int_en,
                IRQ_USER_VEC_NUM => self.user_vec,
                _ => 0,
            },
            _ => 0,
        }
    }

    pub fn write(&mut self, offset: u32, v: u32) {
        let (block, chan, off) = split(offset);
        match (block, chan) {
            (0, 0) => self.h2c.write(off, v),
            (1, 0) => self.c2h.write(off, v),
            (4, 0) => self.h2c.write_sgdma(off, v),
            (5, 0) => self.c2h.write_sgdma(off, v),
            // Each enable is RW, with W1S and W1C aliases at +4 and +8.
            (2, _) => match offset & 0xFFF {
                IRQ_USER_INT_EN => self.user_int_en = v,
                0x08 => self.user_int_en |= v,
                0x0C => self.user_int_en &= !v,
                IRQ_CHAN_INT_EN => self.chan_int_en = v,
                0x14 => self.chan_int_en |= v,
                0x18 => self.chan_int_en &= !v,
                IRQ_USER_VEC_NUM => self.user_vec = v,
                _ => {}
            },
            _ => {}
        }
    }

    /// Let both channels walk as far as the streams allow.
    pub fn run(&mut self, mem: &DeviceMem<'_>, streams: &mut dyn Streams) {
        self.h2c.run(mem, streams);
        self.c2h.run(mem, streams);
    }
}

/// (target block, channel, register offset within the channel block).
fn split(offset: u32) -> (u32, u32, u32) {
    ((offset >> 12) & 0xF, (offset >> 8) & 0xF, offset & 0xFF)
}
