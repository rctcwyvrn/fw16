//! The XDMA descriptor engine (PG195), written from the spec for the
//! card's IP configuration: 1 H2C + 1 C2H AXI-Stream channel, BAR1 = the
//! XDMA register space, `usr_irq` width 1 (regmap v0.2 proposal §3).
//!
//! - [`Channel`] — one channel moving one transfer: SG chain, poll-mode
//!   writeback, C2H result records, stop on every exit.
//! - [`init`] — identify the engine, stop both channels, route the shell's
//!   interrupt to MSI-X vector 0.
//! - [`Port`] — BAR1 plus "let the device work": the card's is a BAR and a
//!   pause, the mock's runs its XDMA model (`mock/xdma.rs`).
//!
//! `desc` lays out chains; `regs` is the register map.

mod channel;
pub mod desc;
mod init;
mod port;
pub mod regs;
#[cfg(test)]
mod tests;

pub use channel::{Channel, Data};
pub use desc::Dir;
pub use init::{USER_VECTOR, init};
pub use port::{DeviceMem, Fault, Port};
