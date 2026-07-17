//! Alertes sonores quand une clé avec solde est trouvée.
//! Non bloquant (thread dédié). Windows: Beep API ; sinon BEL terminal.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Anti-spam : min intervalle entre rafales de bip
static LAST_BEEP: OnceLock<std::sync::Mutex<Option<Instant>>> = OnceLock::new();
static ENABLED: AtomicBool = AtomicBool::new(true);

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Bip immédiat (thread séparé) — pattern « jackpot » ~3 notes.
pub fn alert_balance_found() {
    if !is_enabled() {
        return;
    }
    // Throttle 1.5s pour ne pas saturer si 50 hits d'un coup
    let lock = LAST_BEEP.get_or_init(|| std::sync::Mutex::new(None));
    {
        let mut g = lock.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        if let Some(prev) = *g {
            if now.duration_since(prev) < Duration::from_millis(1500) {
                return;
            }
        }
        *g = Some(now);
    }

    let _ = std::thread::Builder::new()
        .name("btc-hit-beep".into())
        .spawn(|| {
            play_hit_sequence();
        });
}

fn play_hit_sequence() {
    #[cfg(windows)]
    {
        // Frequencies (Hz), durations (ms) — clairement audible
        // Beep() = haut-parleur PC / sortie audio Windows (kernel32)
        let notes: &[(u32, u32)] = &[
            (880, 180),  // A5
            (1175, 180), // D6
            (1568, 350), // G6
            (1568, 200),
        ];
        for &(freq, ms) in notes {
            unsafe {
                Beep(freq, ms);
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        return;
    }

    #[cfg(not(windows))]
    {
        // BEL + prints
        eprint!("\x07\x07\x07");
        let _ = std::io::Write::flush(&mut std::io::stderr());
        eprintln!("\n*** HIT SOLDE — bip ***\n");
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn Beep(dwFreq: u32, dwDuration: u32) -> i32;
}
