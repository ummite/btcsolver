//! Archive permanente des clés intéressantes.
//!
//! On conserve **toute clé qui a eu de l'action on-chain** :
//! - solde UTXO actuel (`utxo_balance`)
//! - et/ou activité détectée (historique / scanblocks) même si solde = 0
//!
//! Fichiers (append-safe, dédupliqués par privkey_hex) :
//! - `data/keys-archive.jsonl`  — journal append-only (chaque hit)
//! - `data/keys-archive.json`   — vue dédupliquée (mise à jour atomique)
//! - `found-keys.json`         — compat brute_force (soldes)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Raison pour laquelle la clé est archivée
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityReason {
    /// UTXO non dépensé trouvé dans l'index
    UtxoBalance,
    /// Activité historique (reçue/dépensée) même sans solde actuel
    TxHistory,
    /// Les deux
    BalanceAndHistory,
    /// Enregistrement manuel / import
    Manual,
}

impl ActivityReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UtxoBalance => "utxo_balance",
            Self::TxHistory => "tx_history",
            Self::BalanceAndHistory => "balance_and_history",
            Self::Manual => "manual",
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        use ActivityReason::*;
        match (self, other) {
            (BalanceAndHistory, _) | (_, BalanceAndHistory) => BalanceAndHistory,
            (UtxoBalance, TxHistory) | (TxHistory, UtxoBalance) => BalanceAndHistory,
            (UtxoBalance, UtxoBalance) => UtxoBalance,
            (TxHistory, TxHistory) => TxHistory,
            (Manual, x) | (x, Manual) => x.clone(),
        }
    }
}

/// Une entrée d'archive (clé + preuves d'activité)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedKey {
    pub privkey_hex: String,
    /// Clé publique compressée SEC1 (33 octets hex, préfixe 02/03) — utile pour explorers / sites web
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_hex: Option<String>,
    /// Clé publique non compressée (65 octets hex, préfixe 04)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubkey_uncompressed_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wif: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default)]
    pub addresses: Vec<String>,
    /// Solde UTXO au moment de la dernière mise à jour (peut redescendre à 0)
    #[serde(default)]
    pub balance_sats: u64,
    #[serde(default)]
    pub balance_btc: f64,
    /// true si on a détecté une activité on-chain (solde OU historique)
    #[serde(default)]
    pub has_activity: bool,
    /// true si solde UTXO > 0 à la dernière vérif
    #[serde(default)]
    pub has_balance: bool,
    pub reason: ActivityReason,
    /// Première détection
    pub first_seen: String,
    /// Dernière mise à jour
    pub last_seen: String,
    /// Pic de solde observé (ne baisse jamais)
    #[serde(default)]
    pub peak_balance_sats: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Dérive pubkeys SEC1 (comp + uncomp) depuis une priv hex 64.
pub fn pubkeys_from_priv_hex(privkey_hex: &str) -> Option<(String, String)> {
    let hex = privkey_hex.trim().trim_start_matches("0x");
    let bytes = hex::decode(hex).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let sk = bitcoin::secp256k1::SecretKey::from_slice(&bytes).ok()?;
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let pk = bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &sk);
    Some((
        hex::encode(pk.serialize()),
        hex::encode(pk.serialize_uncompressed()),
    ))
}

impl ArchivedKey {
    pub fn now_stamp() -> String {
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }

    pub fn from_utxo_hit(
        privkey_hex: impl Into<String>,
        wif: Option<String>,
        addresses: Vec<String>,
        balance_sats: u64,
        source: &str,
        method: Option<String>,
        input: Option<String>,
    ) -> Self {
        let ts = Self::now_stamp();
        let has_balance = balance_sats > 0;
        let privkey_hex = privkey_hex.into();
        let (pubkey_hex, pubkey_uncompressed_hex) = match pubkeys_from_priv_hex(&privkey_hex) {
            Some((c, u)) => (Some(c), Some(u)),
            None => (None, None),
        };
        Self {
            privkey_hex,
            pubkey_hex,
            pubkey_uncompressed_hex,
            wif,
            input,
            method,
            source: Some(source.to_string()),
            addresses,
            balance_sats,
            balance_btc: balance_sats as f64 / 1e8,
            has_activity: has_balance, // UTXO hit implies activity
            has_balance,
            reason: if has_balance {
                ActivityReason::UtxoBalance
            } else {
                ActivityReason::TxHistory
            },
            first_seen: ts.clone(),
            last_seen: ts,
            peak_balance_sats: balance_sats,
            notes: None,
        }
    }

    pub fn from_history_hit(
        privkey_hex: impl Into<String>,
        wif: Option<String>,
        addresses: Vec<String>,
        balance_sats: u64,
        source: &str,
        method: Option<String>,
        input: Option<String>,
        notes: Option<String>,
    ) -> Self {
        let ts = Self::now_stamp();
        let has_balance = balance_sats > 0;
        let reason = if has_balance {
            ActivityReason::BalanceAndHistory
        } else {
            ActivityReason::TxHistory
        };
        let privkey_hex = privkey_hex.into();
        let (pubkey_hex, pubkey_uncompressed_hex) = match pubkeys_from_priv_hex(&privkey_hex) {
            Some((c, u)) => (Some(c), Some(u)),
            None => (None, None),
        };
        Self {
            privkey_hex,
            pubkey_hex,
            pubkey_uncompressed_hex,
            wif,
            input,
            method,
            source: Some(source.to_string()),
            addresses,
            balance_sats,
            balance_btc: balance_sats as f64 / 1e8,
            has_activity: true,
            has_balance,
            reason,
            first_seen: ts.clone(),
            last_seen: ts,
            peak_balance_sats: balance_sats,
            notes,
        }
    }

    /// Remplit pubkey_hex / uncomp si absents (anciennes entrées archive).
    pub fn ensure_pubkeys(&mut self) {
        if self.pubkey_hex.is_some() && self.pubkey_uncompressed_hex.is_some() {
            return;
        }
        if let Some((c, u)) = pubkeys_from_priv_hex(&self.privkey_hex) {
            if self.pubkey_hex.is_none() {
                self.pubkey_hex = Some(c);
            }
            if self.pubkey_uncompressed_hex.is_none() {
                self.pubkey_uncompressed_hex = Some(u);
            }
        }
    }

    fn merge_with(&mut self, other: &ArchivedKey) {
        // Keep earliest first_seen
        if other.first_seen < self.first_seen {
            self.first_seen = other.first_seen.clone();
        }
        self.last_seen = other.last_seen.clone();
        if other.balance_sats > self.balance_sats {
            self.balance_sats = other.balance_sats;
            self.balance_btc = other.balance_btc;
        }
        if other.peak_balance_sats > self.peak_balance_sats {
            self.peak_balance_sats = other.peak_balance_sats;
        }
        // Peak never decreases from either side
        self.peak_balance_sats = self.peak_balance_sats.max(self.balance_sats);
        self.has_balance = self.balance_sats > 0;
        self.has_activity = self.has_activity || other.has_activity || self.has_balance;
        self.reason = self.reason.merge(&other.reason);
        if self.pubkey_hex.is_none() {
            self.pubkey_hex = other.pubkey_hex.clone();
        }
        if self.pubkey_uncompressed_hex.is_none() {
            self.pubkey_uncompressed_hex = other.pubkey_uncompressed_hex.clone();
        }
        if self.wif.is_none() {
            self.wif = other.wif.clone();
        }
        if self.input.is_none() {
            self.input = other.input.clone();
        }
        if self.method.is_none() {
            self.method = other.method.clone();
        }
        if self.source.is_none() {
            self.source = other.source.clone();
        }
        // Union addresses
        for a in &other.addresses {
            if !self.addresses.iter().any(|x| x == a) {
                self.addresses.push(a.clone());
            }
        }
        if self.notes.is_none() {
            self.notes = other.notes.clone();
        }
        self.ensure_pubkeys();
    }
}

/// Chemins d'archive sous le projet
pub struct KeyArchive {
    project_dir: PathBuf,
    lock: Mutex<()>,
}

impl KeyArchive {
    pub fn new(project_dir: impl AsRef<Path>) -> Self {
        Self {
            project_dir: project_dir.as_ref().to_path_buf(),
            lock: Mutex::new(()),
        }
    }

    pub fn data_dir(&self) -> PathBuf {
        self.project_dir.join("data")
    }

    pub fn jsonl_path(&self) -> PathBuf {
        self.data_dir().join("keys-archive.jsonl")
    }

    pub fn json_path(&self) -> PathBuf {
        self.data_dir().join("keys-archive.json")
    }

    pub fn found_keys_path(&self) -> PathBuf {
        self.project_dir.join("found-keys.json")
    }

    fn ensure_data_dir(&self) -> Result<()> {
        fs::create_dir_all(self.data_dir()).context("create data/")?;
        Ok(())
    }

    /// Charge la vue dédupliquée (ou reconstruit depuis jsonl)
    pub fn load_all(&self) -> Result<Vec<ArchivedKey>> {
        let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.load_all_unlocked()
    }

    fn load_all_unlocked(&self) -> Result<Vec<ArchivedKey>> {
        let path = self.json_path();
        if path.exists() {
            let raw = fs::read_to_string(&path)?;
            if let Ok(v) = serde_json::from_str::<Vec<ArchivedKey>>(&raw) {
                return Ok(v);
            }
            // maybe wrapped object
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(arr) = v.get("keys").and_then(|x| x.as_array()) {
                    let mut out = Vec::new();
                    for item in arr {
                        if let Ok(k) = serde_json::from_value::<ArchivedKey>(item.clone()) {
                            out.push(k);
                        }
                    }
                    return Ok(out);
                }
            }
        }
        // Rebuild from jsonl
        self.rebuild_from_jsonl_unlocked()
    }

    fn rebuild_from_jsonl_unlocked(&self) -> Result<Vec<ArchivedKey>> {
        let path = self.jsonl_path();
        let mut map: HashMap<String, ArchivedKey> = HashMap::new();
        if path.exists() {
            let raw = fs::read_to_string(&path)?;
            for line in raw.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Ok(k) = serde_json::from_str::<ArchivedKey>(line) {
                    let hex = k.privkey_hex.to_lowercase();
                    map.entry(hex)
                        .and_modify(|e| e.merge_with(&k))
                        .or_insert(k);
                }
            }
        }
        let mut keys: Vec<_> = map.into_values().collect();
        keys.sort_by(|a, b| b.peak_balance_sats.cmp(&a.peak_balance_sats));
        Ok(keys)
    }

    /// Enregistre une clé avec activité. No-op si ni solde ni has_activity.
    /// Retourne true si une nouvelle entrée a été ajoutée/mise à jour.
    pub fn record(&self, mut entry: ArchivedKey) -> Result<bool> {
        // Normalise
        entry.privkey_hex = entry.privkey_hex.to_lowercase();
        entry.ensure_pubkeys();
        entry.has_balance = entry.balance_sats > 0;
        if entry.has_balance {
            entry.has_activity = true;
        }
        if !entry.has_activity && !entry.has_balance {
            return Ok(false);
        }
        if entry.peak_balance_sats < entry.balance_sats {
            entry.peak_balance_sats = entry.balance_sats;
        }

        let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.ensure_data_dir()?;

        // Append jsonl event
        {
            let line = serde_json::to_string(&entry)?;
            let mut f = OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.jsonl_path())?;
            writeln!(f, "{}", line)?;
        }

        // Merge into deduped view
        let mut all = self.load_all_unlocked().unwrap_or_default();
        let mut found = false;
        for existing in &mut all {
            if existing.privkey_hex.eq_ignore_ascii_case(&entry.privkey_hex) {
                existing.merge_with(&entry);
                found = true;
                break;
            }
        }
        if !found {
            all.push(entry.clone());
        }
        all.sort_by(|a, b| b.peak_balance_sats.cmp(&a.peak_balance_sats));

        let wrapper = serde_json::json!({
            "updated_at": ArchivedKey::now_stamp(),
            "count": all.len(),
            "with_balance": all.iter().filter(|k| k.has_balance).count(),
            "activity_only": all.iter().filter(|k| k.has_activity && !k.has_balance).count(),
            "keys": all,
        });
        let tmp = self.json_path().with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_string_pretty(&wrapper)?)?;
        fs::rename(&tmp, self.json_path())?;

        // Also maintain found-keys.json (balance > 0 only, for brute_force compat)
        self.sync_found_keys_unlocked(&all)?;

        Ok(true)
    }

    fn sync_found_keys_unlocked(&self, all: &[ArchivedKey]) -> Result<()> {
        let with_bal: Vec<serde_json::Value> = all
            .iter()
            .filter(|k| k.peak_balance_sats > 0 || k.has_balance)
            .map(|k| {
                serde_json::json!({
                    "key_hex": k.privkey_hex,
                    "privkey_hex": k.privkey_hex,
                    "pubkey_hex": k.pubkey_hex,
                    "pubkey_uncompressed_hex": k.pubkey_uncompressed_hex,
                    "wif": k.wif,
                    "sats": k.peak_balance_sats.max(k.balance_sats),
                    "btc": (k.peak_balance_sats.max(k.balance_sats) as f64) / 1e8,
                    "addresses": k.addresses,
                    "timestamp": k.last_seen,
                    "has_activity": k.has_activity,
                    "reason": k.reason.as_str(),
                    "source": k.source,
                    "method": k.method,
                })
            })
            .collect();
        // Write project root + data/
        for path in [
            self.found_keys_path(),
            self.data_dir().join("found-keys.json"),
        ] {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let _ = fs::write(&path, serde_json::to_string_pretty(&with_bal)?);
        }
        Ok(())
    }

    /// Stats for dashboard
    pub fn stats(&self) -> Result<serde_json::Value> {
        let all = self.load_all()?;
        Ok(serde_json::json!({
            "count": all.len(),
            "with_balance": all.iter().filter(|k| k.has_balance).count(),
            "activity_only": all.iter().filter(|k| k.has_activity && !k.has_balance).count(),
            "peak_sats_total": all.iter().map(|k| k.peak_balance_sats).sum::<u64>(),
            "jsonl": self.jsonl_path().display().to_string(),
            "json": self.json_path().display().to_string(),
        }))
    }

    pub fn list_json(&self) -> Result<serde_json::Value> {
        let mut all = self.load_all()?;
        let mut changed = false;
        for k in &mut all {
            let before = (k.pubkey_hex.is_none(), k.pubkey_uncompressed_hex.is_none());
            k.ensure_pubkeys();
            if (before.0 && k.pubkey_hex.is_some())
                || (before.1 && k.pubkey_uncompressed_hex.is_some())
            {
                changed = true;
            }
        }
        if changed {
            // Persiste les pubkeys rétro-remplies (vue dédupliquée)
            let _g = self.lock.lock().unwrap_or_else(|e| e.into_inner());
            if let Ok(()) = self.ensure_data_dir() {
                let wrapper = serde_json::json!({
                    "updated_at": ArchivedKey::now_stamp(),
                    "count": all.len(),
                    "with_balance": all.iter().filter(|k| k.has_balance).count(),
                    "activity_only": all.iter().filter(|k| k.has_activity && !k.has_balance).count(),
                    "keys": all,
                });
                let tmp = self.json_path().with_extension("json.tmp");
                if fs::write(&tmp, serde_json::to_string_pretty(&wrapper).unwrap_or_default()).is_ok()
                {
                    let _ = fs::rename(&tmp, self.json_path());
                }
                let _ = self.sync_found_keys_unlocked(&all);
            }
        }
        Ok(serde_json::json!({
            "success": true,
            "count": all.len(),
            "with_balance": all.iter().filter(|k| k.has_balance).count(),
            "activity_only": all.iter().filter(|k| k.has_activity && !k.has_balance).count(),
            "keys": all,
        }))
    }
}

/// Helper path-based record used by binaries that don't hold KeyArchive long-lived
pub fn record_utxo_hit(
    project_dir: &Path,
    privkey_hex: &str,
    wif: Option<String>,
    addresses: Vec<String>,
    balance_sats: u64,
    source: &str,
    method: Option<String>,
    input: Option<String>,
) -> Result<bool> {
    if balance_sats == 0 {
        return Ok(false);
    }
    let arch = KeyArchive::new(project_dir);
    arch.record(ArchivedKey::from_utxo_hit(
        privkey_hex,
        wif,
        addresses,
        balance_sats,
        source,
        method,
        input,
    ))
}

pub fn record_activity_hit(
    project_dir: &Path,
    privkey_hex: &str,
    wif: Option<String>,
    addresses: Vec<String>,
    balance_sats: u64,
    source: &str,
    method: Option<String>,
    input: Option<String>,
    notes: Option<String>,
) -> Result<bool> {
    let arch = KeyArchive::new(project_dir);
    arch.record(ArchivedKey::from_history_hit(
        privkey_hex,
        wif,
        addresses,
        balance_sats,
        source,
        method,
        input,
        notes,
    ))
}
