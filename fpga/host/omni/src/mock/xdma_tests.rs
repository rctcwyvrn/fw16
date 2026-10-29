//! The XDMA model on its own: hand-made chains it must refuse, and the
//! IRQ block's gate on the shell interrupt.

use std::time::{Duration, Instant};

use super::*;
use crate::regs::*;
use crate::xdma::desc::Desc;
use crate::xdma::regs::*;

const T: Duration = Duration::from_millis(10);

/// Run on H2C the chain `build(base)` returns, written at `offset` into a
/// fresh page whose first descriptor sits at bus address `base`, with
/// `first_adj`; `data` is the only other buffer. Returns STATUS.
fn run_h2c(
    m: &Mock,
    data: &DmaBuf,
    offset: usize,
    first_adj: u32,
    build: impl Fn(u64) -> Vec<[u64; 4]>,
) -> u32 {
    let mut descs = m.alloc(4096).unwrap();
    let base = descs.iova().unwrap() + offset as u64;
    for (i, w) in build(base).iter().enumerate() {
        descs.words_mut()[offset / 8 + 4 * i..][..4].copy_from_slice(w);
    }
    let s = &mut *m.state();
    let mut port = MockPort {
        xdma: &mut s.xdma,
        card: &mut s.card,
    };
    port.write32(H2C_SGDMA + SGDMA_DESC_LO, base as u32)
        .unwrap();
    port.write32(H2C_SGDMA + SGDMA_DESC_HI, (base >> 32) as u32)
        .unwrap();
    port.write32(H2C_SGDMA + SGDMA_DESC_ADJ, first_adj).unwrap();
    port.write32(H2C_CHAN + CHAN_CONTROL, CTRL_RUN).unwrap();
    let mem = DeviceMem::new().rw(&mut descs).ro(data);
    port.wait(&mem, Instant::now()).unwrap();
    port.read32(H2C_CHAN + CHAN_STATUS).unwrap()
}

/// A one-beat descriptor reading `data`.
fn desc(data: &DmaBuf, next: u64, last: bool) -> [u64; 4] {
    Desc {
        flags: if last { DESC_STOP } else { 0 },
        adj: 0,
        bytes: 32,
        src: data.iova().unwrap(),
        dst: 0,
        next,
    }
    .encode()
}

#[test]
fn a_good_two_descriptor_chain_stops_clean() {
    let m = Mock::new();
    let data = m.alloc(32).unwrap();
    let s = run_h2c(&m, &data, 0, 1, |base| {
        vec![desc(&data, base + 32, false), desc(&data, 0, true)]
    });
    assert_eq!(s, STAT_DESC_STOPPED);
}

#[test]
fn bad_magic_stops_the_engine() {
    let m = Mock::new();
    let data = m.alloc(32).unwrap();
    let s = run_h2c(&m, &data, 0, 0, |_| {
        let mut d = desc(&data, 0, true);
        d[0] ^= 1 << 16;
        vec![d]
    });
    assert_eq!(s & (STAT_MAGIC_STOPPED | STAT_BUSY), STAT_MAGIC_STOPPED);
}

#[test]
fn an_adjacent_count_that_lies_is_a_descriptor_error() {
    let m = Mock::new();
    let data = m.alloc(32).unwrap();
    // first_adj = 1 says descriptor 1 follows 0 contiguously, but 0's next
    // points two slots on.
    let s = run_h2c(&m, &data, 0, 1, |base| {
        vec![
            desc(&data, base + 64, false),
            desc(&data, 0, true),
            desc(&data, 0, true),
        ]
    });
    assert_ne!(s & STAT_DESC_ERROR, 0, "{s:#x}");
}

#[test]
fn a_burst_across_a_page_is_a_descriptor_error() {
    let m = Mock::new();
    let data = m.alloc(32).unwrap();
    // The first descriptor is the page's last slot; first_adj = 1 would
    // fetch past the page.
    let s = run_h2c(&m, &data, 4096 - 32, 1, |_| vec![desc(&data, 0, true)]);
    assert_ne!(s & STAT_DESC_ERROR, 0, "{s:#x}");
}

#[test]
fn data_outside_the_transfer_buffers_is_a_read_error() {
    let m = Mock::new();
    let data = m.alloc(32).unwrap();
    let s = run_h2c(&m, &data, 0, 0, |_| {
        let mut d = desc(&data, 0, true);
        d[1] = 0xDEAD_0000; // src: nothing lives there
        vec![d]
    });
    assert_ne!(s & STAT_READ_ERROR, 0, "{s:#x}");
    assert_eq!(s & STAT_BUSY, 0);
}

#[test]
fn shell_interrupt_needs_the_xdma_user_enable() {
    let m = Mock::new();
    m.write32(OMNI_IRQ_EN, OMNI_IRQ_EN_FORCE_MASK).unwrap();
    m.bar1_write(IRQ_BLOCK + IRQ_USER_INT_EN, 0);
    m.write32(OMNI_IRQ_FORCE, 1).unwrap();
    assert!(m.wait_irq(0, T).is_err(), "masked at the XDMA: lost");
    crate::irq::drain(&m).unwrap();
    m.bar1_write(IRQ_BLOCK + IRQ_USER_INT_EN + 4, 1); // W1S
    m.write32(OMNI_IRQ_FORCE, 1).unwrap();
    m.wait_irq(0, T).unwrap();
}
