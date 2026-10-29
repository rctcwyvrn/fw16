//! Pattern checker with the shell sink's semantics (`stream_sink.ml`), so a
//! host-side count from C2H data and the card's `SNK_ERRS` mean the same
//! thing: the expected sequence advances on every beat, so one corrupt beat
//! is one error and a dropped or duplicated beat is an error on every beat
//! after it; the error count saturates at `u32::MAX`.

use super::{Beat, Generator, Mode, WORDS_PER_BEAT};

#[derive(Clone, Debug)]
pub struct Checker {
    mode: Mode,
    expected: Generator,
    beats: u64,
    errors: u32,
}

impl Checker {
    pub fn new(mode: Mode) -> Checker {
        Checker {
            mode,
            expected: Generator::new(mode),
            beats: 0,
            errors: 0,
        }
    }

    /// SNK_CLEAR: counters and pattern back to reset.
    pub fn clear(&mut self) {
        *self = Checker::new(self.mode);
    }

    /// Check one beat; returns whether it matched.
    pub fn check_beat(&mut self, beat: &Beat) -> bool {
        self.beats = self.beats.wrapping_add(1);
        let ok = self.expected.next_beat() == *beat;
        if !ok {
            self.errors = self.errors.saturating_add(1);
        }
        ok
    }

    /// Check a whole number of beats laid out as words.
    pub fn check(&mut self, words: &[u64]) {
        assert_eq!(words.len() % WORDS_PER_BEAT, 0, "check: partial beat");
        for chunk in words.as_chunks::<WORDS_PER_BEAT>().0 {
            self.check_beat(chunk);
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn beats(&self) -> u64 {
        self.beats
    }

    pub fn errors(&self) -> u32 {
        self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(mode: Mode, beats: usize) -> Vec<u64> {
        let mut words = vec![0; beats * WORDS_PER_BEAT];
        Generator::new(mode).fill(&mut words);
        words
    }

    #[test]
    fn clean_stream_has_no_errors() {
        for mode in [Mode::Counter, Mode::Prbs31] {
            let mut c = Checker::new(mode);
            c.check(&stream(mode, 100));
            assert_eq!((c.beats(), c.errors()), (100, 0), "{mode:?}");
        }
    }

    #[test]
    fn one_corrupt_beat_is_one_error() {
        for mode in [Mode::Counter, Mode::Prbs31] {
            let mut words = stream(mode, 100);
            words[10 * WORDS_PER_BEAT + 2] ^= 1 << 17;
            let mut c = Checker::new(mode);
            c.check(&words);
            assert_eq!(c.errors(), 1, "{mode:?}");
        }
    }

    #[test]
    fn a_dropped_beat_cascades() {
        let mut words = stream(Mode::Prbs31, 100);
        words.drain(50 * WORDS_PER_BEAT..51 * WORDS_PER_BEAT);
        let mut c = Checker::new(Mode::Prbs31);
        c.check(&words);
        assert_eq!((c.beats(), c.errors()), (99, 49));
    }

    #[test]
    fn wrong_mode_fails_every_beat() {
        let mut c = Checker::new(Mode::Counter);
        c.check(&stream(Mode::Prbs31, 10));
        assert_eq!(c.errors(), 10);
    }

    #[test]
    fn clear_restarts_the_sequence() {
        let mut c = Checker::new(Mode::Counter);
        c.check(&stream(Mode::Counter, 5));
        c.clear();
        c.check(&stream(Mode::Counter, 5));
        assert_eq!((c.beats(), c.errors()), (5, 0));
    }
}
