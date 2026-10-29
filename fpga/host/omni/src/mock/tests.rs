//! The mock against the shell semantics it claims (`shell_top.ml`, map v0.1,
//! v0.2 proposal).

use std::time::Duration;

use super::*;
use crate::counters::read_u64;
use crate::pattern::{Checker, Generator, Mode, WORDS_PER_BEAT, counter_beat};
use crate::regs::proposed::*;
use crate::regs::*;

const T: Duration = Duration::from_millis(10);
const BEAT: usize = crate::pattern::BEAT_BYTES;

fn beats(m: &Mock, lo: u32, hi: u32) -> u64 {
    read_u64(m, lo, hi).unwrap()
}

#[test]
fn identity_and_access_kinds() {
    let m = Mock::new();
    assert_eq!(m.read32(OMNI_ID).unwrap(), 0x4F4D_4E49);
    assert_eq!(m.read32(OMNI_VERSION).unwrap(), OMNI_VERSION_RESET);
    // RW
    m.write32(OMNI_SCRATCH, 0xDEAD_BEEF).unwrap();
    assert_eq!(m.read32(OMNI_SCRATCH).unwrap(), 0xDEAD_BEEF);
    assert_eq!(m.read32(OMNI_SRC_RATE).unwrap(), OMNI_SRC_RATE_RESET);
    // RO writes are dropped; pulse and unmapped read 0.
    m.write32(OMNI_ID, 0).unwrap();
    assert_eq!(m.read32(OMNI_ID).unwrap(), OMNI_ID_RESET);
    m.write32(OMNI_SRC_CLEAR, 1).unwrap();
    assert_eq!(m.read32(OMNI_SRC_CLEAR).unwrap(), 0);
    assert_eq!(m.read32(0x0F00).unwrap(), 0);
    assert!(m.read32(0x0002).is_err());
    let status = m.read32(OMNI_STATUS).unwrap();
    assert_eq!(
        status,
        OMNI_STATUS_LINK_UP_MASK | OMNI_STATUS_MEM_READY_MASK
    );
}

#[test]
fn c2h_delivers_the_source_pattern_through_the_ring() {
    for mode in [Mode::Counter, Mode::Prbs31] {
        let m = Mock::new();
        m.write32(
            OMNI_SRC_CTL,
            OMNI_SRC_CTL_EN_MASK | mode.bit() << OMNI_SRC_CTL_MODE_SHIFT,
        )
        .unwrap();
        // The source fills the ring before anyone reads.
        assert_eq!(m.read32(OMNI_RING_FILL).unwrap(), RING_DEPTH as u32);
        let mut buf = m.alloc(3 * RING_DEPTH * BEAT).unwrap();
        m.dma_read(&mut buf, 3 * RING_DEPTH * BEAT, T).unwrap();
        let mut c = Checker::new(mode);
        c.check(buf.words());
        assert_eq!(c.errors(), 0, "{mode:?}");
        let src = beats(&m, OMNI_SRC_BEATS_LO, OMNI_SRC_BEATS_HI);
        assert_eq!(src, 4 * RING_DEPTH as u64, "delivered + ring refilled");
    }
}

#[test]
fn c2h_with_source_off_times_out_with_progress() {
    let m = Mock::new();
    m.write32(OMNI_SRC_CTL, OMNI_SRC_CTL_EN_MASK).unwrap();
    m.write32(OMNI_SRC_CTL, 0).unwrap();
    let mut buf = m.alloc((RING_DEPTH + 8) * BEAT).unwrap();
    let r = m.dma_read(&mut buf, (RING_DEPTH + 8) * BEAT, T);
    assert!(matches!(r, Err(Error::Timeout { done }) if done == RING_DEPTH * BEAT));
    assert_eq!(buf.words()[..WORDS_PER_BEAT], counter_beat(0));
}

#[test]
fn h2c_sink_counts_only_while_enabled() {
    let m = Mock::new();
    let mut buf = m.alloc(64 * BEAT).unwrap();
    Generator::new(Mode::Prbs31).fill(buf.words_mut());
    m.dma_write(&buf, buf.len(), T).unwrap(); // sink off: accepted, not counted
    assert_eq!(beats(&m, OMNI_SNK_BEATS_LO, OMNI_SNK_BEATS_HI), 0);
    m.write32(OMNI_SNK_CTL, OMNI_SNK_CTL_EN_MASK | OMNI_SNK_CTL_MODE_MASK)
        .unwrap();
    m.dma_write(&buf, buf.len(), T).unwrap();
    assert_eq!(beats(&m, OMNI_SNK_BEATS_LO, OMNI_SNK_BEATS_HI), 64);
    assert_eq!(m.read32(OMNI_SNK_ERRS).unwrap(), 0);
    assert_eq!(
        m.read32(OMNI_RING_FILL).unwrap(),
        0,
        "H2C reaches the ring only in loopback"
    );
}

#[test]
fn h2c_errors_set_status_and_clear() {
    let m = Mock::new();
    m.write32(OMNI_SNK_CTL, OMNI_SNK_CTL_EN_MASK).unwrap();
    let mut buf = m.alloc(16 * BEAT).unwrap();
    Generator::new(Mode::Counter).fill(buf.words_mut());
    buf.words_mut()[5 * WORDS_PER_BEAT] ^= 0x80;
    m.dma_write(&buf, buf.len(), T).unwrap();
    assert_eq!(m.read32(OMNI_SNK_ERRS).unwrap(), 1);
    assert_ne!(m.read32(OMNI_STATUS).unwrap() & OMNI_STATUS_SNK_ERR_MASK, 0);
    m.write32(OMNI_SNK_CLEAR, 1).unwrap();
    assert_eq!(m.read32(OMNI_SNK_ERRS).unwrap(), 0);
    assert_eq!(beats(&m, OMNI_SNK_BEATS_LO, OMNI_SNK_BEATS_HI), 0);
}

#[test]
fn loopback_returns_h2c_data_and_stalls_when_full() {
    let m = Mock::new();
    m.write32(OMNI_SRC_CTL, OMNI_SRC_CTL_EN_MASK).unwrap(); // fills the ring...
    m.write32(OMNI_RING_CTL, OMNI_RING_CTL_LOOPBACK_MASK)
        .unwrap(); // ...then is held off
    m.write32(OMNI_RING_CLEAR, 1).unwrap();
    m.write32(OMNI_SNK_CTL, OMNI_SNK_CTL_EN_MASK).unwrap();
    let mut out = m.alloc((RING_DEPTH + 1) * BEAT).unwrap();
    Generator::new(Mode::Counter).fill(out.words_mut());
    let r = m.dma_write(&out, out.len(), T);
    assert!(matches!(r, Err(Error::Timeout { done }) if done == RING_DEPTH * BEAT));
    // The stalled beat was not taken by the sink either.
    assert_eq!(
        beats(&m, OMNI_SNK_BEATS_LO, OMNI_SNK_BEATS_HI),
        RING_DEPTH as u64
    );
    let mut back = m.alloc(RING_DEPTH * BEAT).unwrap();
    m.dma_read(&mut back, RING_DEPTH * BEAT, T).unwrap();
    assert_eq!(back.words(), &out.words()[..back.words().len()]);
    // Only the fill from before LOOPBACK was set; none since.
    assert_eq!(
        beats(&m, OMNI_SRC_BEATS_LO, OMNI_SRC_BEATS_HI),
        RING_DEPTH as u64
    );
}

#[test]
fn fault_injection_corrupts_c2h() {
    let m = Mock::new().corrupt_c2h_every(100);
    m.write32(OMNI_SRC_CTL, OMNI_SRC_CTL_EN_MASK).unwrap();
    let mut buf = m.alloc(1000 * BEAT).unwrap();
    m.dma_read(&mut buf, 1000 * BEAT, T).unwrap();
    let mut c = Checker::new(Mode::Counter);
    c.check(buf.words());
    assert_eq!(c.errors(), 10);
}

#[test]
fn lo_read_latches_hi() {
    let m = Mock::new();
    m.state()
        .shell
        .set_beat_counters(0x1_FFFF_FFFF, 0x2_0000_0005);
    assert_eq!(m.read32(OMNI_SRC_BEATS_LO).unwrap(), 0xFFFF_FFFF);
    m.state()
        .shell
        .set_beat_counters(0x2_0000_0000, 0x2_0000_0005); // carry after the LO read
    assert_eq!(
        m.read32(OMNI_SRC_BEATS_HI).unwrap(),
        1,
        "HI is the shadow from the LO read"
    );
    assert_eq!(
        beats(&m, OMNI_SNK_BEATS_LO, OMNI_SNK_BEATS_HI),
        0x2_0000_0005
    );
}

#[test]
fn forced_irq_is_edge_triggered_and_rearms_on_clear() {
    let m = Mock::new();
    m.write32(OMNI_IRQ_FORCE, 1).unwrap();
    assert!(
        m.wait_irq(0, T).is_err(),
        "disabled source does not interrupt"
    );
    assert_eq!(m.read32(OMNI_IRQ_PENDING).unwrap(), OMNI_IRQ_EN_FORCE_MASK);
    m.write32(OMNI_IRQ_EN, OMNI_IRQ_EN_FORCE_MASK).unwrap(); // enabling a pending source fires
    m.wait_irq(0, T).unwrap();
    m.write32(OMNI_IRQ_FORCE, 1).unwrap(); // still pending: no new edge
    assert!(m.wait_irq(0, T).is_err());
    m.write32(OMNI_IRQ_PENDING, OMNI_IRQ_EN_FORCE_MASK).unwrap(); // RW1C
    assert_eq!(m.read32(OMNI_IRQ_PENDING).unwrap(), 0);
    m.write32(OMNI_IRQ_FORCE, 1).unwrap();
    m.wait_irq(0, T).unwrap();
    assert_eq!(m.read32(OMNI_IRQ_COUNT).unwrap(), 2);
    assert!(m.wait_irq(1, T).is_err());
}

#[test]
fn one_handle_serves_h2c_and_c2h_threads() {
    let m = Mock::new();
    m.write32(OMNI_SNK_CTL, OMNI_SNK_CTL_EN_MASK).unwrap();
    m.write32(OMNI_SRC_CTL, OMNI_SRC_CTL_EN_MASK).unwrap();
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut buf = m.alloc(64 * BEAT).unwrap();
            Generator::new(Mode::Counter).fill(buf.words_mut());
            for _ in 0..100 {
                m.dma_write(&buf, buf.len(), T).unwrap();
                m.write32(OMNI_SNK_CLEAR, 1).unwrap();
            }
        });
        s.spawn(|| {
            let mut buf = m.alloc(64 * BEAT).unwrap();
            let mut c = Checker::new(Mode::Counter);
            for _ in 0..100 {
                m.dma_read(&mut buf, 64 * BEAT, T).unwrap();
                c.check(buf.words());
            }
            assert_eq!(c.errors(), 0);
        });
    });
    assert_eq!(m.read32(OMNI_SNK_ERRS).unwrap(), 0);
}

#[test]
fn snk_err_interrupt_fires_once_per_error_run() {
    let m = Mock::new();
    m.write32(OMNI_IRQ_EN, OMNI_IRQ_EN_SNK_ERR_MASK).unwrap();
    m.write32(OMNI_SNK_CTL, OMNI_SNK_CTL_EN_MASK).unwrap();
    let buf = m.alloc(8 * BEAT).unwrap(); // zeros: no beat matches the counter
    m.dma_write(&buf, buf.len(), T).unwrap();
    assert_eq!(m.read32(OMNI_SNK_ERRS).unwrap(), 8);
    m.wait_irq(0, T).unwrap();
    assert!(m.wait_irq(0, T).is_err());
}
