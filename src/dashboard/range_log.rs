//! Journal des plages hex déjà scannées (éviter de retester) + fenêtres De→À.
//! Fichier : `{project}/data/scan-ranges-log.json`

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Taille de fenêtre par défaut : 2^30 clés (~1.07 milliard)
pub const DEFAULT_RANGE_STEP: u64 = 1u64 << 30;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RangeEntry {
    /// Hex 64, inclusif
    pub start: String,
    /// Hex 64, exclusif (première clé non testée de la plage)
    pub end: String,
    pub keys: u64,
    pub completed_at: String,
    #[serde(default)]
    pub mode: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrentWindow {
    pub start: String,
    pub end: String,
    #[serde(default)]
    pub cursor: Option<String>,
    pub started_at: String,
    #[serde(default)]
    pub keys_target: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RangeLog {
    pub version: u32,
    /// Pas d'extension quand la fin de fenêtre (À) est atteinte
    #[serde(default = "default_step")]
    pub range_step: u64,
    /// Départ manuel souhaité (hex 64). Vide = auto après dernière plage.
    #[serde(default)]
    pub manual_start: Option<String>,
    #[serde(default)]
    pub ranges: Vec<RangeEntry>,
    #[serde(default)]
    pub current: Option<CurrentWindow>,
}

fn default_step() -> u64 {
    DEFAULT_RANGE_STEP
}

impl Default for RangeLog {
    fn default() -> Self {
        Self {
            version: 1,
            range_step: DEFAULT_RANGE_STEP,
            manual_start: None,
            ranges: Vec::new(),
            current: None,
        }
    }
}

impl RangeLog {
    pub fn path(project_dir: &str) -> PathBuf {
        Path::new(project_dir).join("data").join("scan-ranges-log.json")
    }

    pub fn load(project_dir: &str) -> Self {
        let p = Self::path(project_dir);
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Ok(mut log) = serde_json::from_str::<RangeLog>(&content) {
                if log.range_step == 0 {
                    log.range_step = DEFAULT_RANGE_STEP;
                }
                return log;
            }
        }
        Self::default()
    }

    pub fn save(&self, project_dir: &str) -> Result<()> {
        let p = Self::path(project_dir);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        // écriture atomique simple
        let tmp = p.with_extension("json.tmp");
        std::fs::write(&tmp, &json)?;
        if std::fs::rename(&tmp, &p).is_err() {
            std::fs::write(&p, &json)?;
            let _ = std::fs::remove_file(&tmp);
        }
        Ok(())
    }

    /// Normalise un hex 64 (lowercase, pad left). Erreur si invalide.
    pub fn normalize_hex(s: &str) -> Result<[u8; 32]> {
        let t = s.trim().to_lowercase().replace("0x", "");
        if t.len() > 64 || !t.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow!("hex invalide (attendu jusqu'à 64 chars)"));
        }
        let padded = format!("{:0>64}", t);
        let bytes = hex::decode(&padded).map_err(|e| anyhow!("hex: {}", e))?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow!("clé doit faire 32 octets"))?;
        if arr.iter().all(|&b| b == 0) {
            return Err(anyhow!("clé nulle interdite"));
        }
        Ok(arr)
    }

    pub fn hex_encode(k: &[u8; 32]) -> String {
        hex::encode(k)
    }

    /// k + offset (big-endian)
    pub fn add_u64(key: &[u8; 32], offset: u64) -> [u8; 32] {
        let mut out = *key;
        let mut carry = offset as u128;
        for i in (0..32).rev() {
            if carry == 0 {
                break;
            }
            let sum = out[i] as u128 + carry;
            out[i] = (sum & 0xff) as u8;
            carry = sum >> 8;
        }
        out
    }

    pub fn cmp_hex(a: &str, b: &str) -> std::cmp::Ordering {
        let na = Self::normalize_hex(a).ok();
        let nb = Self::normalize_hex(b).ok();
        match (na, nb) {
            (Some(aa), Some(bb)) => aa.cmp(&bb),
            _ => a.to_lowercase().cmp(&b.to_lowercase()),
        }
    }

    /// True si [start, end) chevauche une plage déjà loggée
    pub fn overlaps_done(&self, start: &str, end: &str) -> bool {
        for r in &self.ranges {
            // overlap if start < r.end && end > r.start
            if Self::cmp_hex(start, &r.end) == std::cmp::Ordering::Less
                && Self::cmp_hex(end, &r.start) == std::cmp::Ordering::Greater
            {
                return true;
            }
        }
        false
    }

    /// Avance `start` juste après toute plage déjà couverte (évite re-test).
    pub fn skip_done(&self, mut start: [u8; 32]) -> [u8; 32] {
        loop {
            let start_h = Self::hex_encode(&start);
            let mut advanced = false;
            for r in &self.ranges {
                // si start est dans [r.start, r.end) → sauter à r.end
                if Self::cmp_hex(&start_h, &r.start) != std::cmp::Ordering::Less
                    && Self::cmp_hex(&start_h, &r.end) == std::cmp::Ordering::Less
                {
                    if let Ok(e) = Self::normalize_hex(&r.end) {
                        start = e;
                        advanced = true;
                        break;
                    }
                }
            }
            if !advanced {
                break;
            }
        }
        start
    }

    /// Plus grande fin de plage loggée (pour enchaîner)
    pub fn max_logged_end(&self) -> Option<[u8; 32]> {
        let mut max: Option<[u8; 32]> = None;
        for r in &self.ranges {
            if let Ok(e) = Self::normalize_hex(&r.end) {
                max = Some(match max {
                    Some(m) if e > m => e,
                    Some(m) => m,
                    None => e,
                });
            }
        }
        max
    }

    /// Calcule la prochaine fenêtre [start, end) de `step` clés.
    /// Si une fenêtre courante est encore ouverte, reprend depuis le curseur jusqu'à son À.
    pub fn next_window(
        &self,
        preferred_start: Option<&str>,
        step: u64,
    ) -> Result<(String, String, u64)> {
        let step = step.max(1);

        // Reprise fenêtre en cours (crash / stop) — ne pas re-créer une autre plage
        if preferred_start.map(|s| s.trim().is_empty()).unwrap_or(true) {
            if let Some(cur) = &self.current {
                if let (Ok(ws), Ok(we)) = (
                    Self::normalize_hex(&cur.start),
                    Self::normalize_hex(&cur.end),
                ) {
                    let start = cur
                        .cursor
                        .as_ref()
                        .and_then(|c| Self::normalize_hex(c).ok())
                        .filter(|c| c.as_slice() >= ws.as_slice() && c.as_slice() < we.as_slice())
                        .unwrap_or(ws);
                    // count approximatif = step (stop réel via --end)
                    return Ok((
                        Self::hex_encode(&start),
                        Self::hex_encode(&we),
                        cur.keys_target.max(1),
                    ));
                }
            }
        }

        let mut start = if let Some(ps) = preferred_start.filter(|s| !s.trim().is_empty()) {
            Self::normalize_hex(ps)?
        } else if let Some(m) = self.max_logged_end() {
            m
        } else if let Some(ms) = &self.manual_start {
            Self::normalize_hex(ms)?
        } else {
            let mut k = [0u8; 32];
            k[31] = 1;
            k
        };

        start = self.skip_done(start);
        let end = Self::add_u64(&start, step);
        Ok((Self::hex_encode(&start), Self::hex_encode(&end), step))
    }

    pub fn open_window(&mut self, start: &str, end: &str, keys: u64) {
        self.current = Some(CurrentWindow {
            start: start.to_lowercase(),
            end: end.to_lowercase(),
            cursor: Some(start.to_lowercase()),
            started_at: chrono::Local::now().to_rfc3339(),
            keys_target: keys,
        });
    }

    pub fn update_cursor(&mut self, cursor: &str) {
        if let Some(ref mut c) = self.current {
            c.cursor = Some(cursor.to_lowercase());
        }
    }

    /// Marque la fenêtre courante comme terminée et l'ajoute au journal.
    pub fn complete_current(&mut self, mode: &str) -> Option<RangeEntry> {
        let cur = self.current.take()?;
        let keys = if cur.keys_target > 0 {
            cur.keys_target
        } else {
            // estimation : end - start si possible
            0
        };
        let entry = RangeEntry {
            start: cur.start,
            end: cur.end,
            keys,
            completed_at: chrono::Local::now().to_rfc3339(),
            mode: mode.to_string(),
        };
        // éviter doublons exacts
        let exists = self
            .ranges
            .iter()
            .any(|r| r.start == entry.start && r.end == entry.end);
        if !exists {
            self.ranges.push(entry.clone());
        }
        Some(entry)
    }

    /// Force un départ manuel (prochaine fenêtre partira d'ici, après skip des plages déjà faites).
    pub fn set_manual_start(&mut self, hex: &str) -> Result<String> {
        let k = Self::normalize_hex(hex)?;
        let h = Self::hex_encode(&k);
        self.manual_start = Some(h.clone());
        // abandonne la fenêtre courante sans la marquer done (user override)
        self.current = None;
        Ok(h)
    }
}
