//! TEMPORARY: the IRQ block from `docs/regmap-v0.2-proposal.md` §1, which
//! `map_v0.ml` does not have yet. Names are what `to_c.ml` would emit (fields
//! declared on `IRQ_EN`; `IRQ_PENDING` uses the same bits), so when Path 3
//! lands the block these move into `omni_regs.h` and this file is deleted.
//!
//! Only the mock backend implements these; the card ignores them until then.

pub const OMNI_IRQ_BASE: u32 = 0x5000;
/// RW: per-source enable.
pub const OMNI_IRQ_EN: u32 = 0x5000;
pub const OMNI_IRQ_EN_FORCE_SHIFT: u32 = 0;
pub const OMNI_IRQ_EN_FORCE_MASK: u32 = 0x0000_0001;
pub const OMNI_IRQ_EN_PPS_SHIFT: u32 = 1;
pub const OMNI_IRQ_EN_PPS_MASK: u32 = 0x0000_0002;
pub const OMNI_IRQ_EN_SNK_ERR_SHIFT: u32 = 2;
pub const OMNI_IRQ_EN_SNK_ERR_MASK: u32 = 0x0000_0004;
/// RW1C: latched sources, same bits as `IRQ_EN`; write 1 to clear.
pub const OMNI_IRQ_PENDING: u32 = 0x5004;
/// RO: interrupts raised since reset (wraps).
pub const OMNI_IRQ_COUNT: u32 = 0x5008;
/// WO (pulse): bit 0 sets `IRQ_PENDING.FORCE`.
pub const OMNI_IRQ_FORCE: u32 = 0x500C;
