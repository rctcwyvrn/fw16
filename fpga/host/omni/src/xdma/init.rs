//! Bringing the XDMA to a known state: after open, and again after every
//! resume (the card loses its configuration across a suspend).
//!
//! - Both channel-0 blocks must identify as XDMA AXI-Stream channels of the
//!   right direction (the IP configuration regmap v0.2 §3 asks for).
//! - Both channels stopped, sticky status cleared, channel interrupts off
//!   (completion is polled; see `channel.rs`).
//! - `usr_irq_req[0]` (the shell's IRQ block) enabled and routed to MSI-X
//!   vector [`USER_VECTOR`]. Without this the card never sends the shell's
//!   interrupt, whatever the VFIO side has armed.

use super::port::Port;
use super::regs::*;
use crate::error::{Error, Result};

/// The MSI-X vector the shell's interrupt arrives on: `wait_irq(0)`.
pub const USER_VECTOR: u32 = 0;

pub fn init(port: &mut dyn Port) -> Result<()> {
    for (base, target, name) in [(H2C_CHAN, TARGET_H2C, "H2C"), (C2H_CHAN, TARGET_C2H, "C2H")] {
        let id = port.read32(base + CHAN_IDENTIFIER)?;
        let ok = id >> ID_MAGIC_SHIFT == ID_MAGIC
            && (id & ID_TARGET_MASK) >> ID_TARGET_SHIFT == target
            && id & ID_STREAM != 0;
        if !ok {
            return Err(Error::Device(format!(
                "XDMA {name} channel 0 identifier {id:#010x}: want 0x1FC, target {target}, AXI-Stream"
            )));
        }
        port.write32(base + CHAN_CONTROL, 0)?;
        port.write32(base + CHAN_INTR_ENABLE, 0)?;
        port.read32(base + CHAN_STATUS_RC)?;
    }
    let id = port.read32(IRQ_BLOCK + IRQ_IDENTIFIER)?;
    if id >> ID_MAGIC_SHIFT != ID_MAGIC || (id & ID_TARGET_MASK) >> ID_TARGET_SHIFT != TARGET_IRQ {
        return Err(Error::Device(format!(
            "XDMA IRQ block identifier {id:#010x}"
        )));
    }
    port.write32(IRQ_BLOCK + IRQ_CHAN_INT_EN, 0)?;
    port.write32(IRQ_BLOCK + IRQ_USER_VEC_NUM, USER_VECTOR)?;
    port.write32(IRQ_BLOCK + IRQ_USER_INT_EN, 1)?;
    Ok(())
}
