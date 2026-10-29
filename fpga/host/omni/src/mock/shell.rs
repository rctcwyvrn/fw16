//! Transaction-level model of shell v0 (`gateware/shell/shell_top.ml`): the
//! CTL/SRC/SNK/RING/TS register blocks and the dataflow
//!
//!   source ─> ring ─> C2H        (RING_CTL.LOOPBACK = 0)
//!   H2C ───> ring ─> C2H         (LOOPBACK = 1; source held off)
//!   H2C ───> sink                (always; counts only while SNK_CTL.EN)
//!
//! There is no clock: the source is infinitely fast (it tops the ring up to
//! RING_DEPTH whenever there is room) and SRC_RATE is stored but has no
//! effect. 64-bit counters follow the v0.2 proposal: reading `*_LO` latches
//! the live high word, which `*_HI` returns.

use std::collections::VecDeque;

use super::pattern_state::PatternState;
use crate::pattern::Beat;
use crate::regs::*;

/// Ring capacity in beats: the BRAM build (`Shell_top.Bram`, depth 4096).
pub const RING_DEPTH: usize = 4096;
/// SYSMON raw code for about 45 °C (UltraScale+ formula, 10 bits in [15:6]).
const TEMP_RAW: u32 = 0xA340;

pub(super) struct Shell {
    scratch: u32,
    src_ctl: u32,
    src_rate: u32,
    src: PatternState,
    src_hi: u32,
    snk_ctl: u32,
    snk: PatternState,
    snk_errs: u32,
    snk_hi: u32,
    ring_ctl: u32,
    ring: VecDeque<Beat>,
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            scratch: 0,
            src_ctl: 0,
            src_rate: OMNI_SRC_RATE_RESET,
            src: PatternState::default(),
            src_hi: 0,
            snk_ctl: 0,
            snk: PatternState::default(),
            snk_errs: 0,
            snk_hi: 0,
            ring_ctl: 0,
            ring: VecDeque::with_capacity(RING_DEPTH),
        }
    }

    fn loopback(&self) -> bool {
        self.ring_ctl & OMNI_RING_CTL_LOOPBACK_MASK != 0
    }

    pub fn snk_errs(&self) -> u32 {
        self.snk_errs
    }

    pub fn read(&mut self, offset: u32) -> u32 {
        match offset {
            OMNI_ID => OMNI_ID_RESET,
            OMNI_VERSION => OMNI_VERSION_RESET,
            OMNI_SCRATCH => self.scratch,
            OMNI_STATUS => {
                OMNI_STATUS_LINK_UP_MASK
                    | OMNI_STATUS_MEM_READY_MASK
                    | if self.snk_errs != 0 {
                        OMNI_STATUS_SNK_ERR_MASK
                    } else {
                        0
                    }
                // RING_OVERFLOW stays 0: the ring backpressures, never drops.
            }
            OMNI_TEMP => TEMP_RAW,
            OMNI_SRC_CTL => self.src_ctl,
            OMNI_SRC_RATE => self.src_rate,
            OMNI_SRC_BEATS_LO => {
                self.src_hi = (self.src.beats() >> 32) as u32;
                self.src.beats() as u32
            }
            OMNI_SRC_BEATS_HI => self.src_hi,
            OMNI_SNK_CTL => self.snk_ctl,
            OMNI_SNK_ERRS => self.snk_errs,
            OMNI_SNK_BEATS_LO => {
                self.snk_hi = (self.snk.beats() >> 32) as u32;
                self.snk.beats() as u32
            }
            OMNI_SNK_BEATS_HI => self.snk_hi,
            OMNI_RING_CTL => self.ring_ctl,
            OMNI_RING_DEPTH => RING_DEPTH as u32,
            OMNI_RING_FILL => self.ring.len() as u32,
            // PWR_MW (no INA), pulse registers, the TS block (zero in the
            // RTL until Path 3 step 4) and unmapped offsets.
            _ => 0,
        }
    }

    pub fn write(&mut self, offset: u32, value: u32) {
        let pulse = value & 1 != 0;
        match offset {
            OMNI_SCRATCH => self.scratch = value,
            OMNI_SRC_CTL => self.src_ctl = value,
            OMNI_SRC_RATE => self.src_rate = value,
            OMNI_SRC_CLEAR if pulse => self.src.clear(),
            OMNI_SNK_CTL => self.snk_ctl = value,
            OMNI_SNK_CLEAR if pulse => {
                self.snk.clear();
                self.snk_errs = 0;
            }
            OMNI_RING_CTL => self.ring_ctl = value,
            OMNI_RING_CLEAR if pulse => self.ring.clear(),
            // RO, TS_LATCH (no timestamp unit yet) and unmapped: dropped.
            _ => {}
        }
        self.settle();
    }

    /// Let the source fill the ring as far as it would by now.
    fn settle(&mut self) {
        let src_on = self.src_ctl & OMNI_SRC_CTL_EN_MASK != 0;
        if src_on && !self.loopback() {
            let mode = (self.src_ctl & OMNI_SRC_CTL_MODE_MASK) >> OMNI_SRC_CTL_MODE_SHIFT;
            while self.ring.len() < RING_DEPTH {
                let beat = self.src.next(mode);
                self.ring.push_back(beat);
            }
        }
    }

    /// One C2H beat out of the ring, or `None` if it is empty and nothing
    /// can refill it.
    pub fn c2h_pop(&mut self) -> Option<Beat> {
        let beat = self.ring.pop_front();
        self.settle();
        beat
    }

    /// One H2C beat in. Returns false if the shell stalls it (loopback with
    /// the ring full) — the beat is then not taken by the sink either.
    pub fn h2c_push(&mut self, beat: Beat) -> bool {
        let loopback = self.loopback();
        if loopback && self.ring.len() >= RING_DEPTH {
            return false;
        }
        if self.snk_ctl & OMNI_SNK_CTL_EN_MASK != 0 {
            let mode = (self.snk_ctl & OMNI_SNK_CTL_MODE_MASK) >> OMNI_SNK_CTL_MODE_SHIFT;
            if self.snk.next(mode) != beat {
                self.snk_errs = self.snk_errs.saturating_add(1);
            }
        }
        if loopback {
            self.ring.push_back(beat);
        }
        true
    }

    #[cfg(test)]
    pub fn set_beat_counters(&mut self, src: u64, snk: u64) {
        self.src = PatternState::at(src);
        self.snk = PatternState::at(snk);
    }
}
