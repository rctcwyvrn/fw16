//! Cross-check against the gateware: `host/include/fw16_fpga_patterns.txt` is
//! printed by `gateware/shell/bin/patterns.exe` from the same `Pattern.Make`
//! functor the RTL source and sink instantiate (refreshed under
//! `dune build @regmap`). If `pattern.ml` changes and this port does not,
//! this test fails.

use super::{Beat, Generator, Mode};

const GOLDEN: &str = include_str!("../../../include/fw16_fpga_patterns.txt");
const BEATS: u64 = 64;

/// The golden's beats for `name`, in file order, each checked to carry
/// the beat index it sits at.
fn beats(name: &str) -> Vec<Beat> {
    let mut out = Vec::new();
    for line in GOLDEN.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(f.len(), 6, "malformed golden line {line:?}");
        if f[0] != name {
            continue;
        }
        let index: usize = f[1].parse().unwrap();
        assert_eq!(index, out.len(), "{name} beats out of order");
        let word = |s: &str| u64::from_str_radix(s, 16).unwrap();
        out.push([word(f[2]), word(f[3]), word(f[4]), word(f[5])]);
    }
    out
}

fn check(name: &str, mode: Mode) {
    let golden = beats(name);
    assert_eq!(golden.len() as u64, BEATS, "{name}: want {BEATS} beats");
    let mut g = Generator::new(mode);
    for (k, want) in golden.iter().enumerate() {
        assert_eq!(&g.next_beat(), want, "{name} beat {k}");
    }
}

#[test]
fn counter_matches_the_gateware() {
    check("counter", Mode::Counter);
}

#[test]
fn prbs31_matches_the_gateware() {
    check("prbs31", Mode::Prbs31);
}

#[test]
fn only_known_patterns_in_the_file() {
    for line in GOLDEN
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let name = line.split_whitespace().next().unwrap();
        assert!(
            matches!(name, "counter" | "prbs31"),
            "unknown pattern {name:?}"
        );
    }
}
