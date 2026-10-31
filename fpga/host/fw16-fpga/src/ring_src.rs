//! `RING_CTL.SRC_SEL` (regmap v0.3): what fills the ring in front of C2H.
//! The header gives the field's shift and mask but not its values; they are
//! here, from the field's description in `gateware/regmap/map_v0.ml`.

use crate::regs::{FW16_FPGA_RING_CTL_SRC_SEL_MASK, FW16_FPGA_RING_CTL_SRC_SEL_SHIFT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingSrc {
    /// The shell's test source (SRC block).
    Source = 0,
    /// H2C: the v0.2 `LOOPBACK` bit.
    H2c = 1,
    /// The project slot's `m_axis`; H2C goes to the slot's `s_axis`.
    Slot = 2,
    /// Nothing (reserved): the ring only drains.
    None = 3,
}

impl RingSrc {
    /// The `RING_CTL` value selecting this source.
    pub fn ring_ctl(self) -> u32 {
        (self as u32) << FW16_FPGA_RING_CTL_SRC_SEL_SHIFT
    }

    /// The source a `RING_CTL` value selects.
    pub fn from_ring_ctl(ctl: u32) -> RingSrc {
        match (ctl & FW16_FPGA_RING_CTL_SRC_SEL_MASK) >> FW16_FPGA_RING_CTL_SRC_SEL_SHIFT {
            0 => RingSrc::Source,
            1 => RingSrc::H2c,
            2 => RingSrc::Slot,
            _ => RingSrc::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::regs::FW16_FPGA_RING_CTL_LOOPBACK_MASK;

    #[test]
    fn round_trips_and_keeps_the_loopback_alias() {
        for s in [RingSrc::Source, RingSrc::H2c, RingSrc::Slot, RingSrc::None] {
            assert_eq!(RingSrc::from_ring_ctl(s.ring_ctl()), s);
        }
        // The v0.2 LOOPBACK bit alone still selects H2C.
        assert_eq!(RingSrc::from_ring_ctl(FW16_FPGA_RING_CTL_LOOPBACK_MASK), RingSrc::H2c);
        assert_eq!(RingSrc::H2c.ring_ctl(), FW16_FPGA_RING_CTL_LOOPBACK_MASK);
    }
}
