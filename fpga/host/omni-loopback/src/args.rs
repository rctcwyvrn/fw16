//! Command line. std-only: the flag set is small and fixed.

use std::time::Duration;

use omni::pattern::Mode;

pub const USAGE: &str = "\
usage: omni-loopback [--uri mock:|vfio:<BDF>] [--mode c2h|h2c|loop|all]
                     [--pattern counter|prbs31] [--seconds S] [--chunk BYTES[K|M]]
                     [--irqs N]

  c2h   card source -> ring -> C2H; the host checks the pattern
  h2c   host pattern -> H2C -> card sink; the card counts errors
  loop  H2C -> ring -> C2H (RING_CTL.LOOPBACK); both sides check
  all   the three in turn (default)
Then times N forced interrupts (IRQ_FORCE -> wait_irq), default 100.
Exit status: 0 clean, 1 pattern errors, 2 transport or usage error.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    C2h,
    H2c,
    Loop,
}

#[derive(Clone, Debug)]
pub struct Args {
    pub uri: String,
    pub paths: Vec<Path>,
    pub pattern: Mode,
    pub duration: Duration,
    pub chunk: usize,
    pub irqs: u32,
}

impl Default for Args {
    fn default() -> Args {
        Args {
            uri: "mock:".into(),
            paths: vec![Path::C2h, Path::H2c, Path::Loop],
            pattern: Mode::Prbs31,
            duration: Duration::from_secs(1),
            chunk: 1 << 20,
            irqs: 100,
        }
    }
}

pub fn parse(argv: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = argv.into_iter();
    while let Some(flag) = it.next() {
        let mut value = || it.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--uri" => a.uri = value()?,
            "--mode" => {
                a.paths = match value()?.as_str() {
                    "c2h" => vec![Path::C2h],
                    "h2c" => vec![Path::H2c],
                    "loop" => vec![Path::Loop],
                    "all" => vec![Path::C2h, Path::H2c, Path::Loop],
                    m => return Err(format!("unknown mode {m}")),
                }
            }
            "--pattern" => {
                a.pattern = match value()?.as_str() {
                    "counter" => Mode::Counter,
                    "prbs31" => Mode::Prbs31,
                    p => return Err(format!("unknown pattern {p}")),
                }
            }
            "--seconds" => {
                let s: f64 = value()?.parse().map_err(|e| format!("--seconds: {e}"))?;
                a.duration =
                    Duration::try_from_secs_f64(s).map_err(|e| format!("--seconds: {e}"))?;
            }
            "--chunk" => a.chunk = parse_size(&value()?)?,
            "--irqs" => a.irqs = value()?.parse().map_err(|e| format!("--irqs: {e}"))?,
            "-h" | "--help" => return Err(String::new()),
            f => return Err(format!("unknown flag {f}")),
        }
    }
    if a.chunk == 0 || a.chunk % omni::pattern::BEAT_BYTES != 0 {
        return Err(format!(
            "--chunk must be a non-zero multiple of {} bytes",
            omni::pattern::BEAT_BYTES
        ));
    }
    Ok(a)
}

fn parse_size(s: &str) -> Result<usize, String> {
    let (num, mult) = match s.as_bytes().last() {
        Some(b'K') => (&s[..s.len() - 1], 1 << 10),
        Some(b'M') => (&s[..s.len() - 1], 1 << 20),
        _ => (s, 1),
    };
    num.parse::<usize>()
        .map(|n| n * mult)
        .map_err(|e| format!("--chunk {s}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Result<Args, String> {
        parse(s.split_whitespace().map(String::from))
    }

    #[test]
    fn defaults_and_flags() {
        let a = p("").unwrap();
        assert_eq!(
            (a.uri.as_str(), a.paths.len(), a.chunk),
            ("mock:", 3, 1 << 20)
        );
        let a = p("--mode loop --pattern counter --seconds 0.5 --chunk 64K --irqs 3").unwrap();
        assert_eq!(a.paths, vec![Path::Loop]);
        assert_eq!(a.pattern, Mode::Counter);
        assert_eq!(a.duration, Duration::from_millis(500));
        assert_eq!((a.chunk, a.irqs), (65536, 3));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(p("--chunk 33").is_err());
        assert!(p("--mode sideways").is_err());
        assert!(p("--seconds").is_err());
        assert!(p("--bogus").is_err());
    }
}
