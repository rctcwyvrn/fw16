//! TEMPORARY shim: the IRQ block from `docs/regmap-v0.2-proposal.md` §1 has
//! landed in `omni_regs.h` (regmap v0.2), so these are now re-exports of the
//! generated constants rather than hand-written copies. Callers still name
//! `regs::proposed::*`; switch them to `regs::*` and delete this file.
//!
//! Re-exporting (not redefining) matters: a module that glob-imports both
//! `regs::*` and `regs::proposed::*` sees one item per name, not two.

pub use super::{
    OMNI_IRQ_BASE, OMNI_IRQ_COUNT, OMNI_IRQ_EN, OMNI_IRQ_EN_FORCE_MASK, OMNI_IRQ_EN_FORCE_SHIFT,
    OMNI_IRQ_EN_PPS_MASK, OMNI_IRQ_EN_PPS_SHIFT, OMNI_IRQ_EN_SNK_ERR_MASK,
    OMNI_IRQ_EN_SNK_ERR_SHIFT, OMNI_IRQ_FORCE, OMNI_IRQ_PENDING,
};
