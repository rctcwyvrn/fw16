use std::time::Duration;

use super::slot::{clear_slot, slot_pending};
use super::*;
use crate::Mock;

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

#[test]
fn a_held_slot_is_left_to_the_project() {
    let m = Mock::new();
    m.write32(FW16_FPGA_IRQ_EN, FW16_FPGA_IRQ_EN_SLOT_MASK).unwrap();
    m.set_slot_irq(true);
    assert_eq!(drain(&m).unwrap(), 0, "drain neither fails nor clears it");
    assert!(slot_pending(&m).unwrap());
    assert!(matches!(wait_pending(&m, 0, T), Err(Error::Timeout { .. })));
    assert!(!clear_slot(&m).unwrap(), "held: re-latches");
    m.set_slot_irq(false);
    assert!(clear_slot(&m).unwrap());
    assert!(!slot_pending(&m).unwrap());
}

#[test]
fn shell_sources_are_serviced_under_a_held_slot() {
    let m = Mock::new();
    m.write32(
        FW16_FPGA_IRQ_EN,
        FW16_FPGA_IRQ_EN_FORCE_MASK | FW16_FPGA_IRQ_EN_SLOT_MASK,
    )
    .unwrap();
    m.set_slot_irq(true);
    m.wait_irq(0, T).unwrap(); // SLOT's edge
    let count = m.read32(FW16_FPGA_IRQ_COUNT).unwrap();
    m.write32(FW16_FPGA_IRQ_FORCE, 1).unwrap();
    // No edge (the line is already high), so it is found by polling.
    assert_eq!(wait_pending(&m, 0, T).unwrap(), FW16_FPGA_IRQ_EN_FORCE_MASK);
    assert_eq!(m.read32(FW16_FPGA_IRQ_COUNT).unwrap(), count);
    assert_eq!(drain(&m).unwrap(), FW16_FPGA_IRQ_EN_FORCE_MASK);
    assert!(slot_pending(&m).unwrap(), "SLOT untouched");
    assert!(matches!(wait_pending(&m, 0, T), Err(Error::Timeout { .. })));
}

#[test]
fn a_disabled_slot_does_not_hold_the_line() {
    let m = Mock::new();
    m.write32(FW16_FPGA_IRQ_EN, FW16_FPGA_IRQ_EN_FORCE_MASK).unwrap();
    m.set_slot_irq(true);
    assert!(slot_pending(&m).unwrap(), "latched whether or not enabled");
    m.write32(FW16_FPGA_IRQ_FORCE, 1).unwrap();
    m.wait_irq(0, T).unwrap(); // FORCE still makes its own edge
    assert_eq!(wait_pending(&m, 0, T).unwrap(), FW16_FPGA_IRQ_EN_FORCE_MASK);
    assert_eq!(drain(&m).unwrap(), FW16_FPGA_IRQ_EN_FORCE_MASK);
}
