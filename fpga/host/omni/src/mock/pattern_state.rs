//! The pattern state inside the shell's source and sink. The RTL keeps a beat
//! index and a PRBS register that both advance on every beat and muxes the
//! output by the *current* MODE bit, so flipping MODE mid-run without a CLEAR
//! continues the other pattern at the same position. The mock does the same.

use crate::pattern::{Beat, Prbs31, counter_beat};

#[derive(Clone, Debug, Default)]
pub(super) struct PatternState {
    index: u64,
    prbs: Prbs31,
}

impl PatternState {
    pub fn clear(&mut self) {
        *self = PatternState::default();
    }

    /// Jump the beat counter (tests of the 64-bit counter registers only;
    /// the PRBS position is left at reset).
    #[cfg(test)]
    pub fn at(index: u64) -> PatternState {
        PatternState {
            index,
            prbs: Prbs31::new(),
        }
    }

    /// Beats so far (the source's / sink's 64-bit beat counter).
    pub fn beats(&self) -> u64 {
        self.index
    }

    /// The next beat for MODE bit `mode`, advancing both patterns.
    pub fn next(&mut self, mode: u32) -> Beat {
        let counter = counter_beat(self.index);
        let prbs = self.prbs.next_beat();
        self.index = self.index.wrapping_add(1);
        if mode & 1 == 0 { counter } else { prbs }
    }
}
