//! One modelled XDMA channel: its registers and its descriptor walk.
//!
//! The walk follows what the engine would do, and is stricter than the
//! hardware where the hardware would silently go wrong: a descriptor fetched
//! as "adjacent" whose predecessor's `next` points elsewhere, or a burst
//! crossing 4 KiB, is a descriptor error here (on the card it is a wrong
//! fetch). Every device access goes through [`DeviceMem`], so an address
//! outside the transfer's buffers is a DMA fault.

use crate::pattern::Beat;
use crate::xdma::desc::{Desc, Dir};
use crate::xdma::regs::*;
use crate::xdma::{DeviceMem, Fault};

/// The shell's side of the two streams.
pub(super) trait Streams {
    /// Offer one H2C beat; false if the shell stalls it.
    fn h2c(&mut self, beat: Beat) -> bool;
    /// Take one C2H beat; `None` if there is none (yet).
    fn c2h(&mut self) -> Option<Beat>;
}

pub(super) struct ChanModel {
    dir: Dir,
    control: u32,
    status: u32,
    completed: u32,
    wb: u64,
    int_mask: u32,
    sg: u64,
    sg_adj: u32,
    // The walk: fetching from `cur`, `adj_left` more adjacent descriptors
    // known, `desc` in progress with `offset` bytes moved.
    active: bool,
    cur: u64,
    adj_left: u32,
    desc: Option<Desc>,
    offset: u32,
}

impl ChanModel {
    pub fn new(dir: Dir) -> ChanModel {
        ChanModel {
            dir,
            control: 0,
            status: 0,
            completed: 0,
            wb: 0,
            int_mask: 0,
            sg: 0,
            sg_adj: 0,
            active: false,
            cur: 0,
            adj_left: 0,
            desc: None,
            offset: 0,
        }
    }

    pub fn read(&mut self, off: u32) -> u32 {
        let target = match self.dir {
            Dir::H2c => TARGET_H2C,
            Dir::C2h => TARGET_C2H,
        };
        match off {
            CHAN_IDENTIFIER => {
                ID_MAGIC << ID_MAGIC_SHIFT | target << ID_TARGET_SHIFT | ID_STREAM | 6
            }
            CHAN_CONTROL => self.control,
            CHAN_STATUS => self.status,
            CHAN_STATUS_RC => {
                let s = self.status;
                self.status &= STAT_BUSY;
                s
            }
            CHAN_COMPLETED_DESC => self.completed,
            CHAN_POLL_WB_LO => self.wb as u32,
            CHAN_POLL_WB_HI => (self.wb >> 32) as u32,
            CHAN_INTR_ENABLE => self.int_mask,
            _ => 0,
        }
    }

    pub fn write(&mut self, off: u32, v: u32) {
        let lo = |x: u64| x & !0xFFFF_FFFF | u64::from(v);
        let hi = |x: u64| x & 0xFFFF_FFFF | u64::from(v) << 32;
        match off {
            CHAN_CONTROL => self.set_control(v),
            CHAN_STATUS => self.status &= !v | STAT_BUSY,
            CHAN_POLL_WB_LO => self.wb = lo(self.wb),
            CHAN_POLL_WB_HI => self.wb = hi(self.wb),
            CHAN_INTR_ENABLE => self.int_mask = v,
            _ => {}
        }
    }

    pub fn read_sgdma(&self, off: u32) -> u32 {
        match off {
            SGDMA_DESC_LO => self.sg as u32,
            SGDMA_DESC_HI => (self.sg >> 32) as u32,
            SGDMA_DESC_ADJ => self.sg_adj,
            _ => 0,
        }
    }

    pub fn write_sgdma(&mut self, off: u32, v: u32) {
        match off {
            SGDMA_DESC_LO => self.sg = self.sg & !0xFFFF_FFFF | u64::from(v),
            SGDMA_DESC_HI => self.sg = self.sg & 0xFFFF_FFFF | u64::from(v) << 32,
            SGDMA_DESC_ADJ => self.sg_adj = v & 0x3F,
            _ => {}
        }
    }

    fn set_control(&mut self, v: u32) {
        let rose = v & CTRL_RUN != 0 && self.control & CTRL_RUN == 0;
        let fell = v & CTRL_RUN == 0 && self.control & CTRL_RUN != 0;
        self.control = v;
        if rose {
            self.completed = 0;
            self.active = true;
            self.status |= STAT_BUSY;
            self.cur = self.sg;
            self.adj_left = self.sg_adj;
            self.desc = None;
            if !burst_fits(self.cur, self.adj_left) {
                self.halt(first_cause(STAT_DESC_ERROR), None);
            }
        }
        if fell {
            // Abandons a descriptor in progress, as the engine does.
            self.active = false;
            self.status &= !STAT_BUSY;
        }
    }

    /// Stop the walk with error `bits`, flagging the poll writeback.
    fn halt(&mut self, bits: u32, mem: Option<&DeviceMem<'_>>) {
        self.status = (self.status | bits) & !STAT_BUSY;
        self.active = false;
        if let Some(mem) = mem
            && self.control & CTRL_POLL_MODE_WB != 0
        {
            let _ = mem.write(self.wb, &(self.completed | WB_ERR).to_le_bytes());
        }
    }

    /// Walk as far as the streams allow.
    pub fn run(&mut self, mem: &DeviceMem<'_>, streams: &mut dyn Streams) {
        let read_err = first_cause(STAT_READ_ERROR);
        let write_err = first_cause(STAT_WRITE_ERROR);
        let desc_err = first_cause(STAT_DESC_ERROR);
        while self.active {
            let d = match self.desc {
                Some(d) => d,
                None => {
                    let mut raw = [0u8; DESC_BYTES];
                    if mem.read(self.cur, &mut raw) == Err(Fault) {
                        return self.halt(desc_err, Some(mem));
                    }
                    let words: [u64; 4] = std::array::from_fn(|i| {
                        u64::from_le_bytes(raw[i * 8..][..8].try_into().unwrap())
                    });
                    let (d, magic) = Desc::decode(words);
                    if magic != DESC_MAGIC {
                        return self.halt(STAT_MAGIC_STOPPED, Some(mem));
                    }
                    if d.bytes == 0 || !d.bytes.is_multiple_of(32) {
                        return self.halt(STAT_INVALID_LENGTH, Some(mem));
                    }
                    self.desc = Some(d);
                    self.offset = 0;
                    d
                }
            };
            while self.offset < d.bytes {
                let at = u64::from(self.offset);
                let mut raw = [0u8; 32];
                match self.dir {
                    Dir::H2c => {
                        if mem.read(d.src + at, &mut raw).is_err() {
                            return self.halt(read_err, Some(mem));
                        }
                        if !streams.h2c(beat_from(&raw)) {
                            return; // stalled
                        }
                    }
                    Dir::C2h => {
                        let Some(beat) = streams.c2h() else {
                            return; // stalled
                        };
                        if mem.write(d.dst + at, &beat_bytes(&beat)).is_err() {
                            return self.halt(write_err, Some(mem));
                        }
                    }
                }
                self.offset += 32;
            }
            if self.complete(&d, mem).is_err() {
                return self.halt(write_err, Some(mem));
            }
            if d.flags & DESC_STOP != 0 {
                self.status = (self.status | STAT_DESC_STOPPED) & !STAT_BUSY;
                self.active = false;
                return;
            }
            self.desc = None;
            if self.adj_left > 0 {
                if d.next != self.cur + DESC_BYTES as u64 {
                    return self.halt(desc_err, Some(mem));
                }
                self.cur += DESC_BYTES as u64;
                self.adj_left -= 1;
            } else {
                self.cur = d.next;
                self.adj_left = d.adj;
                if !burst_fits(self.cur, self.adj_left) {
                    return self.halt(desc_err, Some(mem));
                }
            }
        }
    }

    /// Descriptor `d` has moved all its bytes: count it, write back.
    fn complete(&mut self, d: &Desc, mem: &DeviceMem<'_>) -> Result<(), Fault> {
        self.completed += 1;
        if self.dir == Dir::C2h && self.control & CTRL_C2H_STREAM_WB_DISABLE == 0 {
            let mut rec = [0u8; RESULT_BYTES];
            rec[..4].copy_from_slice(&(RESULT_MAGIC << 16).to_le_bytes());
            rec[4..8].copy_from_slice(&d.bytes.to_le_bytes());
            mem.write(d.src, &rec)?;
        }
        if self.control & CTRL_POLL_MODE_WB != 0 {
            mem.write(self.wb, &self.completed.to_le_bytes())?;
        }
        if d.flags & DESC_COMPLETED != 0 {
            self.status |= STAT_DESC_COMPLETED;
        }
        Ok(())
    }
}

/// The lowest bit of a multi-bit error field (which cause is not modelled).
fn first_cause(field: u32) -> u32 {
    field.isolate_lowest_one()
}

/// A burst of `1 + adj` descriptors from `at` stays inside one 4 KiB page.
fn burst_fits(at: u64, adj: u32) -> bool {
    at % ADJ_BOUNDARY + (u64::from(adj) + 1) * DESC_BYTES as u64 <= ADJ_BOUNDARY
}

fn beat_from(raw: &[u8; 32]) -> Beat {
    std::array::from_fn(|i| u64::from_le_bytes(raw[i * 8..][..8].try_into().unwrap()))
}

fn beat_bytes(beat: &Beat) -> [u8; 32] {
    let mut raw = [0u8; 32];
    for (i, w) in beat.iter().enumerate() {
        raw[i * 8..][..8].copy_from_slice(&w.to_le_bytes());
    }
    raw
}
