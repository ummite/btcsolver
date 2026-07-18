use anyhow::Result;
use bip39::Mnemonic;
use bitcoin::key::{CompressedPublicKey, PrivateKey, UntweakedPublicKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::{Address, Network};
use md5::Md5;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;

use crate::flat_index::FlatIndex;

/// Result of checking a single private key
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyCheckResult {
    pub input: String,
    pub input_format: String,
    pub method: String,
    pub privkey_hex: String,
    pub pubkey_hex: String,
    pub addresses: KeyAddresses,
    pub matches: Vec<UTXOMatch>,
    pub total_balance_sats: u64,
    pub total_balance_btc: f64,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyAddresses {
    pub legacy: String,
    pub segwit: String,
    pub wrapped: String,
    pub taproot: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UTXOMatch {
    pub address: String,
    pub address_type: String,
    pub value_sats: u64,
    pub value_btc: f64,
    pub script_hex: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum KeyFormat {
    Hex,
    WIF,
    BIP39,
    Brainwallet,
}

impl KeyAddresses {
    /// Look up all 4 addresses in the FlatIndex and return total balance
    pub fn total_sats(&self) -> u64 {
        // This is a placeholder — the actual lookup happens in lookup_key_sync
        0
    }
}

impl std::fmt::Display for KeyFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyFormat::Hex => write!(f, "hex"),
            KeyFormat::WIF => write!(f, "WIF"),
            KeyFormat::BIP39 => write!(f, "BIP39"),
            KeyFormat::Brainwallet => write!(f, "brainwallet"),
        }
    }
}

/// One derived private key with human-readable method label
#[derive(Clone, Debug)]
pub struct DerivedKey {
    pub method: String,
    pub bytes: [u8; 32],
}

/// Transform / hash options for brainwallet-style derivation
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BrainwalletOptions {
    #[serde(default = "default_true")]
    pub sha256: bool,
    #[serde(default)]
    pub double_sha256: bool,
    #[serde(default)]
    pub md5_padded: bool,
    #[serde(default = "default_true")]
    pub reverse_chars: bool,
    #[serde(default = "default_true")]
    pub reverse_words: bool,
    #[serde(default = "default_true")]
    pub lowercase: bool,
    #[serde(default)]
    pub uppercase: bool,
    #[serde(default)]
    pub no_spaces: bool,
    /// Enlever ponctuation / symboles (.,;!?'"…@#$…) — garde lettres, chiffres, espaces
    #[serde(default)]
    pub strip_symbols: bool,
    #[serde(default)]
    pub common_suffixes: bool,
    #[serde(default)]
    pub common_prefixes: bool,
    /// Préfixe = toutes les chaînes de N caractères (ASCII imprimable 0x21–0x7E).
    /// 0 = off, 1 ou 2. Ex. N=1 sur « Patate » → « !Patate », « ?Patate », …
    #[serde(default)]
    pub char_prefix_len: u8,
    /// Suffixe = toutes les chaînes de N caractères (même charset). 0 = off, 1 ou 2.
    /// Ex. préfixe+suffixe 1 → « !Patate! », « !Patate? », …
    #[serde(default)]
    pub char_suffix_len: u8,
    /// Max BIP39 receive addresses to derive (0/1 = first only)
    #[serde(default = "default_bip_count")]
    pub bip39_address_count: u32,
    /// Expand all standard BIP paths (44/49/84/86)
    #[serde(default = "default_true")]
    pub bip39_all_paths: bool,
}

fn default_true() -> bool {
    true
}
fn default_bip_count() -> u32 {
    5
}

impl Default for BrainwalletOptions {
    fn default() -> Self {
        Self {
            sha256: true,
            double_sha256: false,
            md5_padded: false,
            reverse_chars: true,
            reverse_words: true,
            lowercase: true,
            uppercase: false,
            no_spaces: false,
            strip_symbols: false,
            common_suffixes: false,
            common_prefixes: false,
            char_prefix_len: 0,
            char_suffix_len: 0,
            bip39_address_count: 5,
            bip39_all_paths: true,
        }
    }
}

/// ASCII imprimable hors espace (0x21–0x7E) : ! " # … 0-9 A-Z a-z { } ~
pub fn affix_charset() -> Vec<char> {
    (0x21u8..=0x7Eu8).map(|b| b as char).collect()
}

/// Nombre de chaînes de longueur `len` sur un charset de taille `n` (len 0 → 1 = « vide »).
pub fn affix_count(charset_len: usize, len: u8) -> usize {
    match len.min(2) {
        0 => 1,
        1 => charset_len,
        2 => charset_len.saturating_mul(charset_len),
        _ => 1,
    }
}

/// Multiplicateur préfixe×suffixe (sans les bases texte).
pub fn char_affix_multiplier(prefix_len: u8, suffix_len: u8) -> u64 {
    let n = affix_charset().len();
    (affix_count(n, prefix_len) as u64).saturating_mul(affix_count(n, suffix_len) as u64)
}

/// Espace d'affixes indexable (génération à la volée, zéro allocation de 94² strings).
#[derive(Clone, Debug)]
pub struct AffixSpace {
    pub pre_len: u8,
    pub suf_len: u8,
    pub n_pre: usize,
    pub n_suf: usize,
    charset: Vec<char>,
}

impl AffixSpace {
    pub fn from_opts(opts: &BrainwalletOptions) -> Self {
        let charset = affix_charset();
        let n_c = charset.len();
        let pre_len = opts.char_prefix_len.min(2);
        let suf_len = opts.char_suffix_len.min(2);
        Self {
            pre_len,
            suf_len,
            n_pre: affix_count(n_c, pre_len),
            n_suf: affix_count(n_c, suf_len),
            charset,
        }
    }

    pub fn enabled(&self) -> bool {
        self.pre_len > 0 || self.suf_len > 0
    }

    pub fn multiplier(&self) -> u64 {
        if !self.enabled() {
            1
        } else {
            (self.n_pre as u64).saturating_mul(self.n_suf as u64)
        }
    }

    pub fn prefix_at(&self, i: usize) -> String {
        self.affix_at(self.pre_len, i, self.n_pre)
    }

    pub fn suffix_at(&self, i: usize) -> String {
        self.affix_at(self.suf_len, i, self.n_suf)
    }

    fn affix_at(&self, len: u8, i: usize, n: usize) -> String {
        if len == 0 || n == 0 {
            return String::new();
        }
        let i = i % n;
        let n_c = self.charset.len();
        match len {
            1 => self.charset[i].to_string(),
            2 => {
                let mut s = String::with_capacity(2);
                s.push(self.charset[i / n_c]);
                s.push(self.charset[i % n_c]);
                s
            }
            _ => String::new(),
        }
    }
}

/// Nombre de méthodes de hash actives.
pub fn hash_method_count(opts: &BrainwalletOptions) -> u64 {
    (opts.sha256 as u64)
        + (opts.double_sha256 as u64)
        + (opts.md5_padded as u64)
}

/// Keep only letters, digits and whitespace; collapse repeated spaces.
fn strip_symbols_keep_text(s: &str) -> String {
    let filtered: String = s
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    filtered.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub struct KeyChecker;

impl KeyChecker {
    pub fn detect_format(input: &str) -> Result<KeyFormat> {
        let trimmed = input.trim();

        if (trimmed.starts_with('5') || trimmed.starts_with('K') || trimmed.starts_with('L'))
            && trimmed.len() >= 51
            && trimmed.len() <= 52
        {
            return Ok(KeyFormat::WIF);
        }

        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(KeyFormat::Hex);
        }

        let words: Vec<&str> = trimmed.split_whitespace().collect();
        if matches!(words.len(), 12 | 15 | 18 | 21 | 24)
            && words.iter().all(|w| w.chars().all(|c| c.is_ascii_lowercase()))
        {
            return Ok(KeyFormat::BIP39);
        }

        if !trimmed.is_empty() {
            return Ok(KeyFormat::Brainwallet);
        }

        Err(anyhow::anyhow!(
            "Could not detect key format. Expected: hex, WIF, BIP39, or brainwallet text"
        ))
    }

    /// Expand input into all candidate private keys according to format + options
    pub fn expand_keys(
        input: &str,
        format: KeyFormat,
        passphrase: Option<&str>,
        opts: &BrainwalletOptions,
    ) -> Result<Vec<DerivedKey>> {
        let network = Network::Bitcoin;
        let secp = Secp256k1::<All>::new();

        match format {
            KeyFormat::Hex => {
                let bytes = hex::decode(input.trim())?;
                if bytes.len() != 32 {
                    anyhow::bail!("Hex key must be exactly 64 characters (32 bytes)");
                }
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes);
                Ok(vec![DerivedKey {
                    method: "hex".into(),
                    bytes: key,
                }])
            }

            KeyFormat::WIF => {
                let pk = PrivateKey::from_str(input.trim())?;
                Ok(vec![DerivedKey {
                    method: "WIF".into(),
                    bytes: pk.inner.secret_bytes(),
                }])
            }

            KeyFormat::BIP39 => {
                let mnemonic = Mnemonic::parse(input.trim())
                    .map_err(|e| anyhow::anyhow!("Invalid BIP39 mnemonic: {}", e))?;
                let pass = passphrase.unwrap_or("");
                let seed_bytes: [u8; 64] = mnemonic.to_seed(pass);
                let xpriv = bitcoin::bip32::Xpriv::new_master(network, &seed_bytes)
                    .map_err(|e| anyhow::anyhow!("master key: {}", e))?;

                let mut paths: Vec<String> = Vec::new();
                let n = opts.bip39_address_count.max(1);
                let account_paths: &[&str] = if opts.bip39_all_paths {
                    &[
                        "m/44'/0'/0'/0", // legacy
                        "m/49'/0'/0'/0", // nested segwit
                        "m/84'/0'/0'/0", // native segwit
                        "m/86'/0'/0'/0", // taproot
                        "m/44'/0'/0'/1", // change legacy
                        "m/84'/0'/0'/1", // change segwit
                    ]
                } else {
                    &["m/44'/0'/0'/0", "m/84'/0'/0'/0"]
                };

                for base in account_paths {
                    for i in 0..n {
                        paths.push(format!("{}/{}", base, i));
                    }
                }

                let mut out = Vec::new();
                for path_str in paths {
                    let path = bitcoin::bip32::DerivationPath::from_str(&path_str)
                        .map_err(|e| anyhow::anyhow!("path {}: {}", path_str, e))?;
                    let derived = xpriv
                        .derive_priv(&secp, &path)
                        .map_err(|e| anyhow::anyhow!("derive {}: {}", path_str, e))?;
                    out.push(DerivedKey {
                        method: format!("BIP39 {}", path_str),
                        bytes: derived.private_key.secret_bytes(),
                    });
                }
                Ok(out)
            }

            KeyFormat::Brainwallet => Ok(Self::brainwallet_candidates(input, opts)),
        }
    }

    /// Compte les clés (bases × affixes × hashes) **sans** les matérialiser.
    pub fn count_brainwallet_keys(input: &str, opts: &BrainwalletOptions) -> u64 {
        let n_bases = Self::text_variants(input, opts).len() as u64;
        if n_bases == 0 {
            return 0;
        }
        let n_hash = hash_method_count(opts).max(1);
        let aff = AffixSpace::from_opts(opts);
        n_bases
            .saturating_mul(aff.multiplier())
            .saturating_mul(n_hash)
    }

    /// Génère les clés en **boucles** (affixes inclus). `visit` retourne `false` pour arrêter.
    /// Aucun plafond : la RAM reste O(bases + batch) si l’appelant ne stocke pas tout.
    pub fn brainwallet_for_each(
        input: &str,
        opts: &BrainwalletOptions,
        mut visit: impl FnMut(DerivedKey) -> bool,
    ) -> bool {
        let bases = Self::text_variants(input, opts);
        if bases.is_empty() {
            return true;
        }
        let aff = AffixSpace::from_opts(opts);

        let emit = |text: &str, visit: &mut dyn FnMut(DerivedKey) -> bool| -> bool {
            if opts.sha256 {
                let hash = Sha256::digest(text.as_bytes());
                let bytes: [u8; 32] = hash.into();
                if !visit(DerivedKey {
                    method: format!("SHA256(\"{}\")", truncate(text, 48)),
                    bytes,
                }) {
                    return false;
                }
            }
            if opts.double_sha256 {
                let h1 = Sha256::digest(text.as_bytes());
                let h2 = Sha256::digest(&h1);
                let bytes: [u8; 32] = h2.into();
                if !visit(DerivedKey {
                    method: format!("SHA256d(\"{}\")", truncate(text, 48)),
                    bytes,
                }) {
                    return false;
                }
            }
            if opts.md5_padded {
                let md5 = Md5::digest(text.as_bytes());
                let mut bytes = [0u8; 32];
                bytes[..16].copy_from_slice(&md5);
                if !visit(DerivedKey {
                    method: format!("MD5pad(\"{}\")", truncate(text, 48)),
                    bytes,
                }) {
                    return false;
                }
            }
            true
        };

        if !aff.enabled() {
            for base in &bases {
                if !emit(base, &mut visit) {
                    return false;
                }
            }
            return true;
        }

        for base in &bases {
            for pi in 0..aff.n_pre {
                let pre = aff.prefix_at(pi);
                for si in 0..aff.n_suf {
                    let suf = aff.suffix_at(si);
                    let text = if pre.is_empty() && suf.is_empty() {
                        base.clone()
                    } else {
                        format!("{}{}{}", pre, base, suf)
                    };
                    if !emit(&text, &mut visit) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Build text variants + hash methods → private keys (collecte ; OK pour 1 phrase légère).
    /// Pour scans massifs, préférer `brainwallet_for_each` / `count_brainwallet_keys`.
    pub fn brainwallet_candidates(input: &str, opts: &BrainwalletOptions) -> Vec<DerivedKey> {
        let mut keys = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let _ = Self::brainwallet_for_each(input, opts, |k| {
            if seen.insert(k.bytes) {
                keys.push(k);
            }
            true
        });
        keys
    }

    /// Variantes texte **sans** expansion préfixe/suffixe tous-caractères
    /// (les affixes sont appliqués en boucle dans `brainwallet_for_each`).
    pub fn text_variants(input: &str, opts: &BrainwalletOptions) -> Vec<String> {
        let mut v = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let push = |s: String, v: &mut Vec<String>, seen: &mut std::collections::HashSet<String>| {
            if s.is_empty() {
                return;
            }
            if seen.insert(s.clone()) {
                v.push(s);
            }
        };

        let raw = input.to_string();
        let trimmed = input.trim().to_string();
        push(raw.clone(), &mut v, &mut seen);
        push(trimmed.clone(), &mut v, &mut seen);

        // Seeds: original + optional de-symbolized form
        let mut seeds: Vec<String> = vec![trimmed.clone()];
        if opts.strip_symbols {
            let cleaned = strip_symbols_keep_text(&trimmed);
            if !cleaned.is_empty() {
                push(cleaned.clone(), &mut v, &mut seen);
                if cleaned != trimmed {
                    seeds.push(cleaned);
                }
            }
            // Also pure alnum (no spaces) as extra seed when symbols stripped
            let alnum: String = trimmed.chars().filter(|c| c.is_alphanumeric()).collect();
            if !alnum.is_empty() && alnum != trimmed {
                push(alnum.clone(), &mut v, &mut seen);
                if !seeds.iter().any(|s| s == &alnum) {
                    seeds.push(alnum);
                }
            }
        }

        for base in seeds {
            if opts.lowercase {
                let lo = base.to_lowercase();
                push(lo, &mut v, &mut seen);
            }
            if opts.uppercase {
                push(base.to_uppercase(), &mut v, &mut seen);
            }

            if opts.reverse_chars {
                let rev: String = base.chars().rev().collect();
                push(rev.clone(), &mut v, &mut seen);
                if opts.lowercase {
                    push(rev.to_lowercase(), &mut v, &mut seen);
                }
            }

            if opts.reverse_words {
                let words: Vec<&str> = base.split_whitespace().collect();
                if words.len() > 1 {
                    let mut rw: Vec<&str> = words.clone();
                    rw.reverse();
                    let s = rw.join(" ");
                    push(s.clone(), &mut v, &mut seen);
                    if opts.lowercase {
                        push(s.to_lowercase(), &mut v, &mut seen);
                    }
                }
            }

            if opts.no_spaces {
                let ns: String = base.chars().filter(|c| !c.is_whitespace()).collect();
                push(ns.clone(), &mut v, &mut seen);
                if opts.lowercase {
                    push(ns.to_lowercase(), &mut v, &mut seen);
                }
            }

            // Title case
            let title = base
                .to_lowercase()
                .split_whitespace()
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            push(title, &mut v, &mut seen);

            let lower = base.to_lowercase();
            if opts.common_suffixes {
                for s in [
                    " bitcoin",
                    " btc",
                    " wallet",
                    " private key",
                    " key",
                    " 123",
                    " 2024",
                    " 2025",
                    " 2026",
                ] {
                    push(format!("{}{}", lower, s), &mut v, &mut seen);
                }
            }
            if opts.common_prefixes {
                for p in ["my ", "the ", "bitcoin ", "my bitcoin ", "wallet "] {
                    push(format!("{}{}", p, lower), &mut v, &mut seen);
                }
            }
        }

        // Note: préfixe/suffixe « tous caractères » → brainwallet_for_each (boucles), pas ici.
        v
    }

    /// Backward-compatible single-key parse (first candidate only)
    pub fn parse_private_key(
        input: &str,
        format: KeyFormat,
        passphrase: Option<&str>,
        _derivation_path: Option<&str>,
    ) -> Result<Vec<[u8; 32]>> {
        let opts = BrainwalletOptions::default();
        Ok(Self::expand_keys(input, format, passphrase, &opts)?
            .into_iter()
            .map(|d| d.bytes)
            .collect())
    }

    pub async fn check_key(
        index: &FlatIndex,
        privkey_bytes: [u8; 32],
        input_display: String,
        input_format: String,
    ) -> KeyCheckResult {
        Self::check_key_with_method(index, privkey_bytes, input_display, input_format, "direct".into())
            .await
    }

    pub async fn check_key_with_method(
        index: &FlatIndex,
        privkey_bytes: [u8; 32],
        input_display: String,
        input_format: String,
        method: String,
    ) -> KeyCheckResult {
        let secp = Secp256k1::<All>::new();
        let network = Network::Bitcoin;

        let secret_key = match bitcoin::secp256k1::SecretKey::from_slice(&privkey_bytes) {
            Ok(sk) => sk,
            Err(e) => {
                return KeyCheckResult {
                    input: input_display,
                    input_format,
                    method,
                    privkey_hex: hex::encode(privkey_bytes),
                    pubkey_hex: String::new(),
                    addresses: empty_addrs(),
                    matches: Vec::new(),
                    total_balance_sats: 0,
                    total_balance_btc: 0.0,
                    error: Some(format!("Invalid private key: {}", e)),
                };
            }
        };

        let pk = PrivateKey {
            inner: secret_key,
            network: network.into(),
            compressed: true,
        };
        let pubkey = pk.public_key(&secp);
        let compressed = match CompressedPublicKey::from_private_key(&secp, &pk) {
            Ok(c) => c,
            Err(e) => {
                return KeyCheckResult {
                    input: input_display,
                    input_format,
                    method,
                    privkey_hex: hex::encode(privkey_bytes),
                    pubkey_hex: hex::encode(pubkey.to_bytes()),
                    addresses: empty_addrs(),
                    matches: Vec::new(),
                    total_balance_sats: 0,
                    total_balance_btc: 0.0,
                    error: Some(format!("compress pubkey: {}", e)),
                };
            }
        };
        let xonly: UntweakedPublicKey = compressed.into();

        let mut matches = Vec::new();
        let mut total_balance = 0u64;

        let legacy_addr = Address::p2pkh(&pubkey, network);
        push_match(index, &mut matches, &mut total_balance, &legacy_addr, "P2PKH");

        let segwit_addr = Address::p2wpkh(&compressed, network);
        push_match(index, &mut matches, &mut total_balance, &segwit_addr, "P2WPKH");

        let wrapped_addr = Address::p2shwpkh(&compressed, network);
        push_match(
            index,
            &mut matches,
            &mut total_balance,
            &wrapped_addr,
            "P2SH-P2WPKH",
        );

        let taproot_addr = Address::p2tr(&secp, xonly, None, network);
        push_match(index, &mut matches, &mut total_balance, &taproot_addr, "P2TR");

        // Also check uncompressed P2PKH (old brainwallets)
        let pk_u = PrivateKey {
            inner: secret_key,
            network: network.into(),
            compressed: false,
        };
        let pub_u = pk_u.public_key(&secp);
        let legacy_u = Address::p2pkh(&pub_u, network);
        push_match(
            index,
            &mut matches,
            &mut total_balance,
            &legacy_u,
            "P2PKH-uncompressed",
        );

        KeyCheckResult {
            input: input_display,
            input_format,
            method,
            privkey_hex: hex::encode(privkey_bytes),
            pubkey_hex: hex::encode(pubkey.to_bytes()),
            addresses: KeyAddresses {
                legacy: legacy_addr.to_string(),
                segwit: segwit_addr.to_string(),
                wrapped: wrapped_addr.to_string(),
                taproot: taproot_addr.to_string(),
            },
            matches,
            total_balance_sats: total_balance,
            total_balance_btc: total_balance as f64 / 1e8,
            error: None,
        }
    }

    /// Sync lookup: derive addresses from private key + check FlatIndex.
    /// Returns (KeyAddresses, total_balance_sats). No async overhead — for batch processing.
    pub fn lookup_key_sync(index: &FlatIndex, privkey_bytes: [u8; 32]) -> (KeyAddresses, u64) {
        let secp = Secp256k1::<All>::new();
        let network = Network::Bitcoin;

        let secret_key = match bitcoin::secp256k1::SecretKey::from_slice(&privkey_bytes) {
            Ok(sk) => sk,
            Err(_) => return (empty_addrs(), 0),
        };

        let pk = PrivateKey {
            inner: secret_key,
            network: network.into(),
            compressed: true,
        };
        let pubkey = pk.public_key(&secp);
        let compressed = match CompressedPublicKey::from_private_key(&secp, &pk) {
            Ok(c) => c,
            Err(_) => return (empty_addrs(), 0),
        };
        let xonly: UntweakedPublicKey = compressed.into();

        let legacy_addr = Address::p2pkh(&pubkey, network);
        let segwit_addr = Address::p2wpkh(&compressed, network);
        let wrapped_addr = Address::p2shwpkh(&compressed, network);
        let taproot_addr = Address::p2tr(&secp, xonly, None, network);

        let mut total = 0u64;
        for addr in [&legacy_addr, &segwit_addr, &wrapped_addr, &taproot_addr] {
            total += index.lookup(addr.script_pubkey().as_bytes());
        }

        (KeyAddresses {
            legacy: legacy_addr.to_string(),
            segwit: segwit_addr.to_string(),
            wrapped: wrapped_addr.to_string(),
            taproot: taproot_addr.to_string(),
        }, total)
    }
}

fn empty_addrs() -> KeyAddresses {
    KeyAddresses {
        legacy: "error".into(),
        segwit: "error".into(),
        wrapped: "error".into(),
        taproot: "error".into(),
    }
}

fn push_match(
    index: &FlatIndex,
    matches: &mut Vec<UTXOMatch>,
    total: &mut u64,
    addr: &Address,
    label: &str,
) {
    let script = addr.script_pubkey();
    let val = index.lookup(script.as_bytes());
    if val > 0 {
        *total += val;
        matches.push(UTXOMatch {
            address: addr.to_string(),
            address_type: label.to_string(),
            value_sats: val,
            value_btc: val as f64 / 1e8,
            script_hex: hex::encode(script.as_bytes()),
        });
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{}…", t)
    }
}
