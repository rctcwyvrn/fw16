//! The logind delay inhibitor, checked from outside: `systemd-inhibit
//! --list` must show it while held and not after drop. With `--suspend`,
//! also a real suspend: an RTC alarm, `systemctl suspend`, and both
//! `PrepareForSleep` edges seen.

use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use omni::sleep_inhibit::{SleepEvent, SleepInhibitor};

const WHY: &str = "omni edu_check";

/// `Ok(note)` or `Err(why it failed)`.
pub fn check(suspend: bool) -> Result<String, String> {
    let hooks = Arc::new(AtomicU64::new(0));
    let h = hooks.clone();
    let hook = Box::new(move |e: SleepEvent| {
        println!("      sleep hook: {e:?}");
        h.fetch_add(1, Ordering::SeqCst);
    });
    let inhibitor = SleepInhibitor::start(WHY, hook).map_err(|e| e.to_string())?;
    if !inhibitor.held() || !listed()? {
        return Err("inhibitor not held / not in systemd-inhibit --list".into());
    }
    let mut note = "held and listed".to_string();

    if suspend {
        run("rtcwake", &["-m", "no", "-s", "20"])?;
        run("systemctl", &["suspend"])?;
        let t0 = Instant::now();
        while inhibitor.events() != (1, 1) && t0.elapsed() < Duration::from_secs(90) {
            std::thread::sleep(Duration::from_millis(200));
        }
        let (s, r) = inhibitor.events();
        if (s, r) != (1, 1) || hooks.load(Ordering::SeqCst) != 2 {
            return Err(format!(
                "suspend cycle: saw {s} suspend / {r} resume events"
            ));
        }
        // Re-taken after resume.
        std::thread::sleep(Duration::from_millis(500));
        if !inhibitor.held() || !listed()? {
            return Err("inhibitor not re-taken after resume".into());
        }
        note += &format!(
            ", suspend cycle took {:.1} s, re-taken",
            t0.elapsed().as_secs_f64()
        );
    }

    drop(inhibitor);
    std::thread::sleep(Duration::from_millis(500));
    if listed()? {
        return Err("inhibitor still listed after drop".into());
    }
    Ok(note + ", released on drop")
}

fn listed() -> Result<bool, String> {
    let out = Command::new("systemd-inhibit")
        .args(["--list", "--no-pager"])
        .output()
        .map_err(|e| format!("systemd-inhibit: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text.lines().any(|l| l.contains(WHY) && l.contains("delay")))
}

fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    let st = Command::new(cmd)
        .args(args)
        .status()
        .map_err(|e| format!("{cmd}: {e}"))?;
    st.success()
        .then_some(())
        .ok_or(format!("{cmd} {args:?}: {st}"))
}
