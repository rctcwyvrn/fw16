//! Servicing the shell's interrupt (regmap v0.2 IRQ block).
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

use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use crate::regs::{FW16_FPGA_IRQ_EN, FW16_FPGA_IRQ_PENDING};
use crate::transport::Transport;

/// Enabled pending sources (an `IRQ_EN`-layout mask), waiting on `vector`
/// only while there are none. Returns at once if something is already
/// pending, whether or not its message was consumed. The caller clears
/// what it handles by writing those bits to `IRQ_PENDING` and calls again.
pub fn wait_pending(t: &dyn Transport, vector: u32, timeout: Duration) -> Result<u32> {
    let deadline = Instant::now() + timeout;
    loop {
        let pending = enabled_pending(t)?;
        if pending != 0 {
            return Ok(pending);
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(Error::Timeout { done: 0 });
        }
        // A message whose cause was already handled (seen pending on an
        // earlier pass) wakes this with nothing pending: loop and wait again.
        t.wait_irq(vector, left)?;
    }
}

/// Clear every enabled pending source until `IRQ_PENDING & IRQ_EN` reads 0,
/// so the next source edge sends a message. Returns what was cleared.
pub fn drain(t: &dyn Transport) -> Result<u32> {
    let mut cleared = 0;
    // A source that re-asserts every cycle would never drain; give up
    // rather than spin forever.
    for _ in 0..64 {
        let pending = enabled_pending(t)?;
        if pending == 0 {
            return Ok(cleared);
        }
        t.write32(FW16_FPGA_IRQ_PENDING, pending)?;
        cleared |= pending;
    }
    Err(Error::Device(format!(
        "IRQ_PENDING still {:#x} after 64 clears",
        enabled_pending(t)?
    )))
}

fn enabled_pending(t: &dyn Transport) -> Result<u32> {
    Ok(t.read32(FW16_FPGA_IRQ_PENDING)? & t.read32(FW16_FPGA_IRQ_EN)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mock;
    use crate::regs::*;

    const T: Duration = Duration::from_millis(10);

    #[test]
    fn a_source_raised_under_another_is_seen_without_a_message() {
        let m = Mock::new();
        m.write32(
            FW16_FPGA_IRQ_EN,
            FW16_FPGA_IRQ_EN_FORCE_MASK | FW16_FPGA_IRQ_EN_SNK_ERR_MASK,
        )
        .unwrap();
        m.write32(FW16_FPGA_SNK_CTL, FW16_FPGA_SNK_CTL_EN_MASK).unwrap();
        m.write32(FW16_FPGA_IRQ_FORCE, 1).unwrap();
        assert_eq!(wait_pending(&m, 0, T).unwrap(), FW16_FPGA_IRQ_EN_FORCE_MASK);
        m.wait_irq(0, T).unwrap(); // the one message, for FORCE
        // SNK_ERR goes pending while FORCE still is: the level is already
        // high, so no second message.
        let zeros = m.alloc(32).unwrap();
        m.dma_write(&zeros, 32, T).unwrap();
        assert!(m.wait_irq(0, T).is_err());
        m.write32(FW16_FPGA_IRQ_PENDING, FW16_FPGA_IRQ_EN_FORCE_MASK).unwrap();
        // Only re-reading finds it.
        assert_eq!(wait_pending(&m, 0, T).unwrap(), FW16_FPGA_IRQ_EN_SNK_ERR_MASK);
        assert_eq!(drain(&m).unwrap(), FW16_FPGA_IRQ_EN_SNK_ERR_MASK);
        assert!(matches!(wait_pending(&m, 0, T), Err(Error::Timeout { .. })));
    }

    #[test]
    fn disabled_sources_are_not_reported() {
        let m = Mock::new();
        m.write32(FW16_FPGA_IRQ_FORCE, 1).unwrap();
        assert!(wait_pending(&m, 0, T).is_err());
        assert_eq!(drain(&m).unwrap(), 0);
        assert_eq!(m.read32(FW16_FPGA_IRQ_PENDING).unwrap(), FW16_FPGA_IRQ_EN_FORCE_MASK);
    }

    #[test]
    fn after_a_drain_the_next_edge_sends_a_message() {
        let m = Mock::new();
        m.write32(FW16_FPGA_IRQ_EN, FW16_FPGA_IRQ_EN_FORCE_MASK).unwrap();
        for _ in 0..3 {
            m.write32(FW16_FPGA_IRQ_FORCE, 1).unwrap();
            m.wait_irq(0, T).unwrap();
            assert_eq!(drain(&m).unwrap(), FW16_FPGA_IRQ_EN_FORCE_MASK);
        }
        assert_eq!(m.read32(FW16_FPGA_IRQ_COUNT).unwrap(), 3);
    }
}
