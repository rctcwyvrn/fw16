//! The shell's two test patterns, hand-ported from
//! `gateware/shell/pattern.ml` so the host generates and checks the same
//! beats as the card's source and sink. A beat is 256 bits = four
//! little-endian u64 words; bit 0 of the beat is bit 0 of word 0.

mod checker;
mod counter;
mod prbs31;

pub use checker::Checker;
pub use counter::counter_beat;
pub use prbs31::Prbs31;

pub const BEAT_BYTES: usize = 32;
pub const WORDS_PER_BEAT: usize = 4;

pub type Beat = [u64; WORDS_PER_BEAT];

/// `SRC_CTL.MODE` / `SNK_CTL.MODE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Counter,
    Prbs31,
}

impl Mode {
    /// The MODE bit as the CTL registers encode it.
    pub fn bit(self) -> u32 {
        match self {
            Mode::Counter => 0,
            Mode::Prbs31 => 1,
        }
    }

    pub fn from_bit(bit: u32) -> Mode {
        if bit & 1 == 0 {
            Mode::Counter
        } else {
            Mode::Prbs31
        }
    }
}

/// A pattern stream from its reset state (what SRC_CLEAR / SNK_CLEAR restore).
#[derive(Clone, Debug)]
pub enum Generator {
    Counter { next: u64 },
    Prbs31(Prbs31),
}

impl Generator {
    pub fn new(mode: Mode) -> Generator {
        match mode {
            Mode::Counter => Generator::Counter { next: 0 },
            Mode::Prbs31 => Generator::Prbs31(Prbs31::new()),
        }
    }

    pub fn next_beat(&mut self) -> Beat {
        match self {
            Generator::Counter { next } => {
                let beat = counter_beat(*next);
                *next = next.wrapping_add(1);
                beat
            }
            Generator::Prbs31(p) => p.next_beat(),
        }
    }

    /// Fill `words` (a whole number of beats) with the next beats.
    pub fn fill(&mut self, words: &mut [u64]) {
        assert_eq!(words.len() % WORDS_PER_BEAT, 0, "fill: partial beat");
        for chunk in words.as_chunks_mut::<WORDS_PER_BEAT>().0 {
            chunk.copy_from_slice(&self.next_beat());
        }
    }
}
