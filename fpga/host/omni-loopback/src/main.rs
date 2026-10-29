//! `omni-loopback`: stream test patterns over the card's DMA paths and report
//! throughput, pattern errors and interrupt latency. See `args::USAGE`.

use std::process::ExitCode;

use omni_loopback::args::{self, Path};
use omni_loopback::{c2h, h2c, irq_latency, ring_loop};

fn main() -> ExitCode {
    let args = match args::parse(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("omni-loopback: {e}");
            }
            eprintln!("{}", args::USAGE);
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::from(1),
        Err(e) => {
            eprintln!("omni-loopback: {e}");
            ExitCode::from(2)
        }
    }
}

/// Runs every requested path; returns the total error count.
fn run(a: &args::Args) -> omni::Result<u64> {
    let t = omni::open(&a.uri)?;
    let id = t.read32(omni::regs::OMNI_ID)?;
    if id != omni::regs::OMNI_ID_RESET {
        return Err(omni::Error::Device(format!(
            "ID reads {id:#010x}, not 'OMNI'"
        )));
    }
    let version = t.read32(omni::regs::OMNI_VERSION)?;
    println!("{}  map v{}.{}", a.uri, version >> 16, version & 0xFFFF);
    if a.uri.starts_with("mock:") {
        println!("note: mock has no clock; GB/s here only exercises the reporting");
    }
    let mut errors = 0;
    for path in &a.paths {
        let r = match path {
            Path::C2h => c2h::run(&*t, a.pattern, a.duration, a.chunk)?,
            Path::H2c => h2c::run(&*t, a.pattern, a.duration, a.chunk)?,
            Path::Loop => ring_loop::run(&*t, a.pattern, a.duration, a.chunk)?,
        };
        println!("{r}");
        errors += r.errors;
    }
    if a.irqs > 0 {
        match irq_latency::run(&*t, a.irqs)? {
            Some(l) => println!("{l}"),
            None => println!("irq   no interrupt (map without the v0.2 IRQ block?)"),
        }
    }
    println!("{}", if errors == 0 { "PASS" } else { "FAIL" });
    Ok(errors)
}
