//! The mock against regmap v0.3's slot additions (decision 0013):
//! `RING_CTL.SRC_SEL`, the echo slot and `SLOT_CTL`, IRQ bit 3 `SLOT`.

use std::time::Duration;

use super::*;
use crate::counters::read_u64;
use crate::pattern::{Generator, Mode};
use crate::regs::*;
use crate::ring_src::RingSrc;

const T: Duration = Duration::from_millis(10);
const BEAT: usize = crate::pattern::BEAT_BYTES;

fn snk_beats(m: &Mock) -> u64 {
    read_u64(m, FW16_FPGA_SNK_BEATS_LO, FW16_FPGA_SNK_BEATS_HI).unwrap()
}

#[test]
fn slot_source_echoes_h2c_back_and_the_sink_still_checks() {
    let m = Mock::new();
    m.write32(FW16_FPGA_SRC_CTL, FW16_FPGA_SRC_CTL_EN_MASK).unwrap();
    m.write32(FW16_FPGA_RING_CTL, RingSrc::Slot.ring_ctl()).unwrap();
    m.write32(FW16_FPGA_RING_CLEAR, 1).unwrap();
    m.write32(FW16_FPGA_SNK_CTL, FW16_FPGA_SNK_CTL_EN_MASK).unwrap();
    let mut out = m.alloc(64 * BEAT).unwrap();
    Generator::new(Mode::Counter).fill(out.words_mut());
    m.dma_write(&out, out.len(), T).unwrap();
    assert_eq!(m.read32(FW16_FPGA_RING_FILL).unwrap(), 64, "the source stays off");
    let mut back = m.alloc(64 * BEAT).unwrap();
    m.dma_read(&mut back, 64 * BEAT, T).unwrap();
    assert_eq!(back.words(), out.words());
    assert_eq!(snk_beats(&m), 64);
    assert_eq!(m.read32(FW16_FPGA_SNK_ERRS).unwrap(), 0);
}

#[test]
fn slot_in_reset_stalls_h2c() {
    let m = Mock::new();
    m.write32(FW16_FPGA_RING_CTL, RingSrc::Slot.ring_ctl()).unwrap();
    m.write32(FW16_FPGA_SNK_CTL, FW16_FPGA_SNK_CTL_EN_MASK).unwrap();
    m.write32(FW16_FPGA_SLOT_CTL, FW16_FPGA_SLOT_CTL_RESET_MASK).unwrap();
    assert_eq!(m.read32(FW16_FPGA_SLOT_CTL).unwrap(), FW16_FPGA_SLOT_CTL_RESET_MASK);
    let buf = m.alloc(8 * BEAT).unwrap();
    let r = m.dma_write(&buf, buf.len(), T);
    assert!(matches!(r, Err(Error::Timeout { done: 0 })));
    assert_eq!(snk_beats(&m), 0, "a stalled beat reaches neither side");
    assert_eq!(m.read32(FW16_FPGA_RING_FILL).unwrap(), 0);
    // Out of reset, the same transfer goes through.
    m.write32(FW16_FPGA_SLOT_CTL, 0).unwrap();
    m.dma_write(&buf, buf.len(), T).unwrap();
    assert_eq!(m.read32(FW16_FPGA_RING_FILL).unwrap(), 8);
}

#[test]
fn h2c_reaches_the_ring_only_from_its_own_selects() {
    for (src, fill) in [
        (RingSrc::Source, 0),
        (RingSrc::H2c, 8),
        (RingSrc::Slot, 8),
        (RingSrc::None, 0),
    ] {
        let m = Mock::new();
        m.write32(FW16_FPGA_RING_CTL, src.ring_ctl()).unwrap();
        m.write32(FW16_FPGA_SNK_CTL, FW16_FPGA_SNK_CTL_EN_MASK).unwrap();
        let mut buf = m.alloc(8 * BEAT).unwrap();
        Generator::new(Mode::Counter).fill(buf.words_mut());
        m.dma_write(&buf, buf.len(), T).unwrap();
        assert_eq!(m.read32(FW16_FPGA_RING_FILL).unwrap(), fill, "{src:?}");
        assert_eq!(snk_beats(&m), 8, "{src:?}: the sink sees H2C in every mode");
    }
}

#[test]
fn src_sel_none_holds_the_source_off() {
    let m = Mock::new();
    m.write32(FW16_FPGA_RING_CTL, RingSrc::None.ring_ctl()).unwrap();
    m.write32(FW16_FPGA_SRC_CTL, FW16_FPGA_SRC_CTL_EN_MASK).unwrap();
    assert_eq!(m.read32(FW16_FPGA_RING_FILL).unwrap(), 0);
}

#[test]
fn slot_irq_is_a_level_that_relatches_until_dropped() {
    let m = Mock::new();
    m.write32(FW16_FPGA_IRQ_EN, FW16_FPGA_IRQ_EN_SLOT_MASK).unwrap();
    m.set_slot_irq(true);
    m.wait_irq(0, T).unwrap();
    // Cleared while the project holds the level: it re-latches, no new edge.
    m.write32(FW16_FPGA_IRQ_PENDING, FW16_FPGA_IRQ_PENDING_SLOT_MASK).unwrap();
    assert_eq!(m.read32(FW16_FPGA_IRQ_PENDING).unwrap(), FW16_FPGA_IRQ_PENDING_SLOT_MASK);
    assert!(m.wait_irq(0, T).is_err());
    // The crate's drain leaves SLOT to the project (0013).
    assert_eq!(crate::irq::drain(&m).unwrap(), 0);
    // Dropped: the clear sticks, and the next raise is a new edge.
    m.set_slot_irq(false);
    m.write32(FW16_FPGA_IRQ_PENDING, FW16_FPGA_IRQ_PENDING_SLOT_MASK).unwrap();
    assert_eq!(m.read32(FW16_FPGA_IRQ_PENDING).unwrap(), 0);
    m.set_slot_irq(true);
    m.wait_irq(0, T).unwrap();
    assert_eq!(m.read32(FW16_FPGA_IRQ_COUNT).unwrap(), 2);
}
