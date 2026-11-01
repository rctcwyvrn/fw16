//! Servicing the shell's interrupt (regmap v0.3 IRQ block).
//!
//! `usr_irq_req` is the level `|(IRQ_PENDING & IRQ_EN)` and XDMA sends one
//! MSI-X message per *rising edge* of it. Two cases send no new message
//! (Path 3c, `gateware/shell/irq.ml`):
//! - a source becomes pending while another enabled source still is (the
//!   level is already high);
//! - a source sets again in the same cycle the host's RW1C clears it (set
//!   wins, so the level never falls).
//!
//! So a message means "look at IRQ_PENDING", not "one event": read it,
//! clear (RW1C) what you handled, read again, and only wait on the vector
//! once it reads 0. [`wait_pending`] is that loop's wait step.
//!
//! The shell's sources are FORCE, PPS and SNK_ERR ([`SHELL_SOURCES`]);
//! [`wait_pending`] and [`drain`] touch only those. Bit 3 `SLOT` belongs to
//! the project in the slot (decision 0013) and is serviced through
//! [`slot`]. While SLOT is enabled and the project holds `slot_irq` high,
//! the line cannot fall, so shell sources raise no message at all:
//! [`wait_pending`] then polls `IRQ_PENDING` every [`HELD_POLL`] instead of
//! waiting on the vector.

pub mod slot;

#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use crate::regs::*;
use crate::transport::Transport;

/// The shell's own sources (an `IRQ_EN`-layout mask): everything but SLOT.
pub const SHELL_SOURCES: u32 =
    FW16_FPGA_IRQ_EN_FORCE_MASK | FW16_FPGA_IRQ_EN_PPS_MASK | FW16_FPGA_IRQ_EN_SNK_ERR_MASK;

/// How often [`wait_pending`] re-reads `IRQ_PENDING` while a held SLOT
/// level keeps the line high (no edges, so no messages to wait for).
pub const HELD_POLL: Duration = Duration::from_millis(1);

/// Enabled pending shell sources (a [`SHELL_SOURCES`] mask), waiting on
/// `vector` only while there are none. Returns at once if something is
/// already pending, whether or not its message was consumed. The caller
/// clears what it handles by writing those bits to `IRQ_PENDING` and calls
/// again. SLOT is never reported; see [`slot`].
pub fn wait_pending(t: &dyn Transport, vector: u32, timeout: Duration) -> Result<u32> {
    let deadline = Instant::now() + timeout;
    loop {
        let enabled = enabled_pending(t)?;
        let pending = enabled & SHELL_SOURCES;
        if pending != 0 {
            return Ok(pending);
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(Error::Timeout { done: 0 });
        }
        // An enabled SLOT holds the line high: a shell source going pending
        // is no edge, so poll rather than wait for a message that won't come.
        let held = enabled & FW16_FPGA_IRQ_EN_SLOT_MASK != 0;
        let step = if held { left.min(HELD_POLL) } else { left };
        match t.wait_irq(vector, step) {
            // A message whose cause was already handled (seen pending on an
            // earlier pass) wakes this with nothing pending: loop and wait again.
            Ok(()) => {}
            Err(Error::Timeout { .. }) if held => {}
            Err(e) => return Err(e),
        }
    }
}

/// Clear every enabled pending shell source until none reads pending, so
/// the next source edge sends a message (unless SLOT is holding the line).
/// Returns what was cleared. SLOT is left alone.
pub fn drain(t: &dyn Transport) -> Result<u32> {
    let mut cleared = 0;
    // A source that re-asserts every cycle would never drain; give up
    // rather than spin forever.
    for _ in 0..64 {
        let pending = enabled_pending(t)? & SHELL_SOURCES;
        if pending == 0 {
            return Ok(cleared);
        }
        t.write32(FW16_FPGA_IRQ_PENDING, pending)?;
        cleared |= pending;
    }
    Err(Error::Device(format!(
        "IRQ_PENDING still {:#x} after 64 clears",
        enabled_pending(t)? & SHELL_SOURCES
    )))
}

fn enabled_pending(t: &dyn Transport) -> Result<u32> {
    Ok(t.read32(FW16_FPGA_IRQ_PENDING)? & t.read32(FW16_FPGA_IRQ_EN)?)
}
