//! The project's interrupt: `IRQ_PENDING` bit 3 `SLOT` (decision 0013).
//!
//! SLOT latches the slot's `slot_irq` *level* and re-latches every cycle it
//! stays high, so only the project can end it: a handler reads
//! [`slot_pending`], makes the project drop `slot_irq` through the project's
//! own registers (BAR2), then [`clear_slot`]. A clear while the level is
//! still held does not stick.
//!
//! There is one vector for both owners, and a message is consumed by
//! whoever waits on it. So there is no `wait_slot`: the one thread that
//! waits on the vector checks both [`super::wait_pending`]'s shell sources
//! and [`slot_pending`] after each wake.

use crate::error::Result;
use crate::regs::{FW16_FPGA_IRQ_PENDING, FW16_FPGA_IRQ_PENDING_SLOT_MASK};
use crate::transport::Transport;

/// Whether SLOT is latched in `IRQ_PENDING`, enabled in `IRQ_EN` or not:
/// a project that keeps SLOT disabled (so its level never holds the line
/// high) can still poll for it.
pub fn slot_pending(t: &dyn Transport) -> Result<bool> {
    Ok(t.read32(FW16_FPGA_IRQ_PENDING)? & FW16_FPGA_IRQ_PENDING_SLOT_MASK != 0)
}

/// Clear SLOT (RW1C). Returns `false` if it re-latched because the project
/// still holds `slot_irq`.
pub fn clear_slot(t: &dyn Transport) -> Result<bool> {
    t.write32(FW16_FPGA_IRQ_PENDING, FW16_FPGA_IRQ_PENDING_SLOT_MASK)?;
    Ok(!slot_pending(t)?)
}
