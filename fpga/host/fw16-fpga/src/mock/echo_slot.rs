//! The project slot as the mock fills it (decision 0013): an echo project
//! whose `s_axis` (H2C, when `RING_CTL.SRC_SEL` = 2) is wired straight to
//! its `m_axis` (into the ring), plus the shell-side `SLOT_CTL` register.
//!
//! `SLOT_CTL.RESET` holds the slot in reset; a slot in reset accepts no
//! beats, so H2C stalls (as an AXI-Stream sink with `tready` low would)
//! rather than dropping them. `DECOUPLE` is stored and has no effect, as in
//! v0.3. The project's own BAR2 registers (`SLOT_ID`, `SLOT_VERSION`) are
//! not modelled: [`crate::Transport`] reaches BAR0 only.

use crate::pattern::Beat;
use crate::regs::FW16_FPGA_SLOT_CTL_RESET_MASK;

#[derive(Default)]
pub(super) struct EchoSlot {
    ctl: u32,
}

impl EchoSlot {
    pub fn ctl(&self) -> u32 {
        self.ctl
    }

    pub fn set_ctl(&mut self, value: u32) {
        self.ctl = value;
    }

    fn in_reset(&self) -> bool {
        self.ctl & FW16_FPGA_SLOT_CTL_RESET_MASK != 0
    }

    /// One beat in; the beat it puts out, or `None` if it stalls the input.
    pub fn echo(&self, beat: Beat) -> Option<Beat> {
        (!self.in_reset()).then_some(beat)
    }
}
