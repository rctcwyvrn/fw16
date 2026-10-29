//! Every mode passes clean against `mock:`, and the error paths count what
//! the fault-injecting mock does to the data.

use std::time::Duration;

use omni::pattern::Mode;
use omni::regs::*;
use omni::{Mock, Transport};
use omni_loopback::{c2h, h2c, irq_latency, ring_loop};

const D: Duration = Duration::from_millis(50);
const CHUNK: usize = 64 << 10;

#[test]
fn every_mode_is_clean_on_the_mock() {
    for pattern in [Mode::Counter, Mode::Prbs31] {
        let m = Mock::new();
        for r in [
            c2h::run(&m, pattern, D, CHUNK).unwrap(),
            h2c::run(&m, pattern, D, CHUNK).unwrap(),
            ring_loop::run(&m, pattern, D, CHUNK).unwrap(),
        ] {
            assert_eq!(r.errors, 0, "{r}");
            assert!(r.bytes >= CHUNK as u64, "{r}");
        }
    }
}

#[test]
fn c2h_counts_corrupt_beats() {
    let m = Mock::new().corrupt_c2h_every(1000);
    let r = c2h::run(&m, Mode::Prbs31, D, CHUNK).unwrap();
    let beats = r.bytes / 32;
    assert_eq!(r.errors, beats / 1000, "{r}");
}

#[test]
fn loop_counts_corruption_on_the_way_back_only() {
    let m = Mock::new().corrupt_c2h_every(512);
    let r = ring_loop::run(&m, Mode::Counter, D, CHUNK).unwrap();
    assert_eq!(r.errors, r.bytes / 32 / 512, "{r}");
    assert!(r.notes.iter().any(|n| n.contains("host checker")), "{r}");
}

#[test]
fn loop_clamps_the_chunk_to_the_ring() {
    let m = Mock::new();
    let r = ring_loop::run(&m, Mode::Counter, D, 4 << 20).unwrap();
    assert_eq!(r.errors, 0, "{r}");
    assert!(r.notes.iter().any(|n| n.contains("clamped")), "{r}");
}

#[test]
fn h2c_flags_a_sink_that_saw_the_wrong_beat_count() {
    // A mock whose sink was disabled behind our back: it counts nothing.
    struct SinkOff(Mock);
    impl Transport for SinkOff {
        fn read32(&self, o: u32) -> omni::Result<u32> {
            self.0.read32(o)
        }
        fn write32(&self, o: u32, v: u32) -> omni::Result<()> {
            self.0.write32(o, if o == OMNI_SNK_CTL { 0 } else { v })
        }
        fn alloc(&self, l: usize) -> omni::Result<omni::DmaBuf> {
            self.0.alloc(l)
        }
        fn dma_read(&self, b: &mut omni::DmaBuf, l: usize, t: Duration) -> omni::Result<()> {
            self.0.dma_read(b, l, t)
        }
        fn dma_write(&self, b: &omni::DmaBuf, l: usize, t: Duration) -> omni::Result<()> {
            self.0.dma_write(b, l, t)
        }
        fn wait_irq(&self, v: u32, t: Duration) -> omni::Result<()> {
            self.0.wait_irq(v, t)
        }
    }
    let r = h2c::run(&SinkOff(Mock::new()), Mode::Counter, D, CHUNK).unwrap();
    assert_eq!(r.errors, 1, "{r}");
}

#[test]
fn irq_latency_on_the_mock() {
    let m = Mock::new();
    let l = irq_latency::run(&m, 10)
        .unwrap()
        .expect("mock implements IRQ_FORCE");
    assert_eq!(l.samples.len(), 10);
    assert_eq!(m.read32(omni::regs::proposed::OMNI_IRQ_COUNT).unwrap(), 10);
}
