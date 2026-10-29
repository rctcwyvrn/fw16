//! PRBS-31 (x^31 + x^28 + 1), 256 output bits per beat, as `pattern.ml`:
//! state seeded to 0x7FFFFFFF; each step outputs `fb = s[30] ^ s[27]` and
//! shifts it in at bit 0; bit 0 of the beat is the first bit out.
//!
//! The RTL unrolls 256 steps per cycle; bit-serial in software would cost 256
//! steps per 32 bytes, too slow to check a card at GB/s. Instead: the output
//! stream obeys o[n] = o[n-31] ^ o[n-28], so with a window `w` holding the
//! last 31 outputs (oldest at bit 0), the next 28 outputs are
//! `(w ^ (w >> 3)) & 0xFFFFFFF` in one step. Tests hold this to the
//! bit-serial reference.

use super::{Beat, WORDS_PER_BEAT};

pub const SEED: u32 = 0x7FFF_FFFF;
const MASK31: u64 = (1 << 31) - 1;
const MASK28: u64 = (1 << 28) - 1;

#[derive(Clone, Debug)]
pub struct Prbs31 {
    /// Last 31 outputs, oldest at bit 0.
    w: u64,
    /// Generated but not yet emitted output bits, first at bit 0.
    acc: u128,
    nacc: u32,
}

impl Prbs31 {
    pub fn new() -> Prbs31 {
        // RTL state bit j is the output j+1 steps ago, i.e. the window reversed.
        Prbs31 {
            w: u64::from(SEED.reverse_bits() >> 1),
            acc: 0,
            nacc: 0,
        }
    }

    fn next28(&mut self) -> u64 {
        let new = (self.w ^ (self.w >> 3)) & MASK28;
        self.w = ((self.w >> 28) | (new << 3)) & MASK31;
        new
    }

    pub fn next_word(&mut self) -> u64 {
        while self.nacc < 64 {
            self.acc |= u128::from(self.next28()) << self.nacc;
            self.nacc += 28;
        }
        let word = self.acc as u64;
        self.acc >>= 64;
        self.nacc -= 64;
        word
    }

    pub fn next_beat(&mut self) -> Beat {
        let mut beat = [0; WORDS_PER_BEAT];
        for w in &mut beat {
            *w = self.next_word();
        }
        beat
    }
}

impl Default for Prbs31 {
    fn default() -> Self {
        Prbs31::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `pattern.ml`'s `prbs_step`, literally: (output bit, next state).
    fn step(s: u32) -> (u32, u32) {
        let fb = ((s >> 30) ^ (s >> 27)) & 1;
        (fb, ((s << 1) | fb) & 0x7FFF_FFFF)
    }

    /// `prbs_beat`: 256 steps, bit i of the beat = i-th bit out.
    fn reference_beat(mut s: u32) -> (Beat, u32) {
        let mut beat = [0u64; 4];
        for i in 0..256 {
            let (bit, next) = step(s);
            beat[i / 64] |= u64::from(bit) << (i % 64);
            s = next;
        }
        (beat, s)
    }

    #[test]
    fn first_word_by_hand() {
        // From all-ones: both taps read 1 for 28 steps (outputs 0..27 = 0),
        // then outputs 28..30 = 1, 31..55 = 0, 56..61 = 1, 62..63 = 0.
        assert_eq!(Prbs31::new().next_word(), 0x3F00_0000_7000_0000);
        assert_eq!(reference_beat(SEED).0[0], 0x3F00_0000_7000_0000);
    }

    #[test]
    fn fast_matches_bit_serial() {
        let mut fast = Prbs31::new();
        let mut s = SEED;
        for k in 0..4096 {
            let (want, next) = reference_beat(s);
            assert_eq!(fast.next_beat(), want, "beat {k}");
            s = next;
        }
    }

    #[test]
    fn not_stuck() {
        let mut p = Prbs31::new();
        let ones: u32 = (0..1000).map(|_| p.next_word().count_ones()).sum();
        // A maximal-length sequence is balanced; 64000 bits → about 32000 ones.
        assert!((31000..33000).contains(&ones), "{ones} ones");
    }
}
