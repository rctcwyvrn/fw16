//! The engine end to end against the mock's XDMA model: chains that span
//! several fetch blocks and pages, progress on timeout, and recovery.

use std::time::Duration;

use super::desc::DATA_PER_DESC;
use super::regs::*;
use super::{Channel, Data, DeviceMem, Dir, Port};
use crate::mock::RING_DEPTH;
use crate::pattern::{BEAT_BYTES, Checker, Generator, Mode, counter_beat};
use crate::regs::*;
use crate::{Error, Mock, Transport};

const T: Duration = Duration::from_millis(10);

fn source_on(m: &Mock, mode: Mode) {
    m.write32(
        OMNI_SRC_CTL,
        OMNI_SRC_CTL_EN_MASK | mode.bit() << OMNI_SRC_CTL_MODE_SHIFT,
    )
    .unwrap();
}

#[test]
fn c2h_chain_of_300_descriptors() {
    // 300 descriptors: 10 fetch blocks over 3 pages of chain.
    let m = Mock::new();
    source_on(&m, Mode::Prbs31);
    let len = 300 * DATA_PER_DESC;
    let mut buf = m.alloc(len).unwrap();
    m.dma_read(&mut buf, len, T).unwrap();
    let mut c = Checker::new(Mode::Prbs31);
    c.check(buf.words());
    assert_eq!(c.errors(), 0);
}

#[test]
fn h2c_chain_with_a_short_last_descriptor() {
    let m = Mock::new();
    m.write32(OMNI_SNK_CTL, OMNI_SNK_CTL_EN_MASK).unwrap();
    let len = 70 * DATA_PER_DESC + 5 * BEAT_BYTES;
    let mut buf = m.alloc(len).unwrap();
    Generator::new(Mode::Counter).fill(buf.words_mut());
    m.dma_write(&buf, len, T).unwrap();
    let beats = crate::counters::read_u64(&m, OMNI_SNK_BEATS_LO, OMNI_SNK_BEATS_HI).unwrap();
    assert_eq!(beats, (len / BEAT_BYTES) as u64);
    assert_eq!(m.read32(OMNI_SNK_ERRS).unwrap(), 0);
}

#[test]
fn partial_length_uses_the_front_of_the_buffer() {
    let m = Mock::new();
    source_on(&m, Mode::Counter);
    let mut buf = m.alloc(4 * DATA_PER_DESC).unwrap();
    m.dma_read(&mut buf, DATA_PER_DESC, T).unwrap();
    let per_desc = DATA_PER_DESC / 8;
    assert_eq!(buf.words()[per_desc - 4..per_desc], counter_beat(127));
    assert!(buf.words()[per_desc..].iter().all(|&w| w == 0));
}

#[test]
fn timeout_reports_whole_descriptors_then_the_channel_recovers() {
    let m = Mock::new();
    source_on(&m, Mode::Counter); // fills the ring: RING_DEPTH beats
    m.write32(OMNI_SRC_CTL, 0).unwrap();
    // One beat more than the ring holds plus half a descriptor: the 33rd
    // descriptor starts, takes nothing, and never completes.
    let len = RING_DEPTH * BEAT_BYTES + DATA_PER_DESC / 2;
    let mut buf = m.alloc(len).unwrap();
    let r = m.dma_read(&mut buf, len, T);
    assert!(
        matches!(r, Err(Error::Timeout { done }) if done == RING_DEPTH * BEAT_BYTES),
        "{r:?}"
    );
    // The stopped channel takes the next transfer cleanly; the stream picks
    // up where the ring left off.
    source_on(&m, Mode::Counter);
    let mut next = m.alloc(DATA_PER_DESC).unwrap();
    m.dma_read(&mut next, DATA_PER_DESC, T).unwrap();
    assert_eq!(next.words()[..4], counter_beat(RING_DEPTH as u64));
}

/// A port that fails every transfer: on the first wait it flags the poll
/// writeback with WB_ERR and STATUS reports a read error.
#[derive(Default)]
struct FailingPort {
    regs: std::collections::HashMap<u32, u32>,
    failed: bool,
}

impl Port for FailingPort {
    fn read32(&mut self, offset: u32) -> crate::Result<u32> {
        let chan_status = [CHAN_STATUS, CHAN_STATUS_RC].map(|r| H2C_CHAN + r);
        Ok(if self.failed && chan_status.contains(&offset) {
            first_read_error()
        } else {
            self.regs.get(&offset).copied().unwrap_or(0)
        })
    }

    fn write32(&mut self, offset: u32, value: u32) -> crate::Result<()> {
        self.regs.insert(offset, value);
        Ok(())
    }

    fn wait(&mut self, mem: &DeviceMem<'_>, _: std::time::Instant) -> crate::Result<bool> {
        let wb = u64::from(self.regs[&(H2C_CHAN + CHAN_POLL_WB_LO)])
            | u64::from(self.regs[&(H2C_CHAN + CHAN_POLL_WB_HI)]) << 32;
        mem.write(wb, &WB_ERR.to_le_bytes()).unwrap();
        self.failed = true;
        Ok(true)
    }
}

fn first_read_error() -> u32 {
    STAT_READ_ERROR.isolate_lowest_one()
}

#[test]
fn an_engine_error_is_reported_with_its_status() {
    let m = Mock::new(); // only for buffers
    let buf = m.alloc(DATA_PER_DESC).unwrap();
    let mut ch = Channel::new(Dir::H2c);
    let mut port = FailingPort::default();
    let r = ch.transfer(&mut port, &|n| m.alloc(n), Data::H2c(&buf), buf.len(), T);
    match r {
        Err(Error::Device(why)) => assert!(why.contains("read error"), "{why}"),
        other => panic!("{other:?}"),
    }
    assert_eq!(port.regs[&(H2C_CHAN + CHAN_CONTROL)], 0, "left stopped");
}

#[test]
fn growing_transfers_reallocate_the_chain() {
    let m = Mock::new();
    source_on(&m, Mode::Counter);
    let mut c = Checker::new(Mode::Counter);
    for descs in [1, 40, 2, 200] {
        let len = descs * DATA_PER_DESC;
        let mut buf = m.alloc(len).unwrap();
        m.dma_read(&mut buf, len, T).unwrap();
        c.check(buf.words());
    }
    assert_eq!(c.errors(), 0);
}
