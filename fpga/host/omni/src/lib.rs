//! `omni` — the host library for the omnimeter card.
//!
//! [`open`] a [`Transport`] by URI (`mock:` now; `vfio:<BDF>` once Path 4
//! step 4 lands; `sim:` once the shell RTL builds), then drive the shell
//! through its registers ([`regs`]) and its two DMA streams. [`pattern`]
//! generates and checks the shell's test patterns host-side.
//! Design: `docs/decisions/0009-host-driver.md`.

pub mod counters;
mod dma_buf;
mod error;
pub mod mock;
mod open;
pub mod pattern;
pub mod regs;
mod transport;

pub use dma_buf::DmaBuf;
pub use error::{Error, Result};
pub use mock::Mock;
pub use open::open;
pub use transport::Transport;
