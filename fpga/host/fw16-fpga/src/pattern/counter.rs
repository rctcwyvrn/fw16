//! Counter pattern: beat k carries the 64-bit words 4k, 4k+1, 4k+2, 4k+3, so
//! a whole C2H buffer reads as an incrementing u64 array.

use super::Beat;

pub fn counter_beat(k: u64) -> Beat {
    // pattern.ml: `sll (uresize beat_index 64) 2` — wraps mod 2^64.
    let base = k.wrapping_shl(2);
    [
        base,
        base.wrapping_add(1),
        base.wrapping_add(2),
        base.wrapping_add(3),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_count_up_across_beats() {
        assert_eq!(counter_beat(0), [0, 1, 2, 3]);
        assert_eq!(counter_beat(1), [4, 5, 6, 7]);
        assert_eq!(counter_beat(1000), [4000, 4001, 4002, 4003]);
    }

    #[test]
    fn wraps_like_the_rtl() {
        // The top two bits of the beat index fall off the 64-bit shift.
        assert_eq!(counter_beat(1 << 62), [0, 1, 2, 3]);
        assert_eq!(counter_beat(u64::MAX)[3], u64::MAX);
    }
}
