//! In-process dictionary / phrase scan against FlatIndex

use anyhow::Result;
use bitcoin::key::{CompressedPublicKey, PrivateKey};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::{Address, Network};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::dashboard::key_checker::{BrainwalletOptions, KeyChecker};
use crate::flat_index::FlatIndex;
use crate::key_archive::{ArchivedKey, KeyArchive};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DictScanRequest {
    #[serde(default)]
    pub phrases: String,
    #[serde(default)]
    pub corpus_path: Option<String>,
    #[serde(default)]
    pub options: BrainwalletOptions,
    #[serde(default)]
    pub threads: Option<usize>,
    #[serde(default)]
    pub min_value: u64,
    #[serde(default = "default_max_phrases")]
    pub max_phrases: usize,
    /// Toutes les permutations des mots (ordre)
    #[serde(default)]
    pub word_permutations: bool,
    /// Toutes les combinaisons non vides (sous-ensembles) de mots
    #[serde(default)]
    pub word_combinations: bool,
    /// Joindre avec espace (ex: "a b c")
    #[serde(default = "default_true_join")]
    pub join_with_space: bool,
    /// Joindre sans espace (ex: "abc")
    #[serde(default = "default_true_join")]
    pub join_no_space: bool,
    /// Plafond mots pris dans le sac (n! explose vite)
    #[serde(default = "default_max_perm_words")]
    pub max_perm_words: usize,
    /// Utiliser les GPU CUDA (derive secp multi-carte) — défaut true si dispo
    #[serde(default = "default_true_gpu")]
    pub use_gpu: bool,
    /// Charger FlatIndex en VRAM + lookup on-device (FULL). Défaut true.
    #[serde(default = "default_true_gpu")]
    pub gpu_full: bool,
}

fn default_true_gpu() -> bool {
    true
}

fn default_max_phrases() -> usize {
    50_000_000
}
fn default_true_join() -> bool {
    true
}
fn default_max_perm_words() -> usize {
    7
}

/// Génère phrases à partir d'un sac de mots (permutations / combinaisons).
/// - combinations: tous les sous-ensembles non vides (ordre d'apparition conservé pour le sous-ensemble)
/// - permutations: tous les ordres de chaque sous-ensemble (ou du set complet)
/// - join_with_space / join_no_space: "a b" et/ou "ab"
pub fn expand_word_bag(
    words: &[String],
    combinations: bool,
    permutations: bool,
    join_with_space: bool,
    join_no_space: bool,
    max_words: usize,
    max_out: usize,
) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut seen_tok = std::collections::HashSet::new();
    for w in words {
        let t = w.trim();
        if t.is_empty() {
            continue;
        }
        let key = t.to_lowercase();
        if seen_tok.insert(key) {
            tokens.push(t.to_string());
        }
        if tokens.len() >= max_words {
            break;
        }
    }
    if tokens.is_empty() {
        return Vec::new();
    }
    // At least one join mode
    let with_sp = join_with_space || (!join_with_space && !join_no_space);
    let no_sp = join_no_space || (!join_with_space && !join_no_space);

    let n = tokens.len();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let push = |s: String, out: &mut Vec<String>, seen: &mut std::collections::HashSet<String>| {
        if out.len() >= max_out {
            return;
        }
        if s.is_empty() {
            return;
        }
        if seen.insert(s.clone()) {
            out.push(s);
        }
    };

    // Bitmask subsets: if combinations → all non-empty; else → full set only
    let mask_end = 1u32 << n;
    let masks: Vec<u32> = if combinations {
        (1..mask_end).collect()
    } else {
        vec![mask_end - 1]
    };

    for mask in masks {
        let mut subset: Vec<String> = Vec::new();
        for i in 0..n {
            if mask & (1u32 << i) != 0 {
                subset.push(tokens[i].clone());
            }
        }
        if subset.is_empty() {
            continue;
        }

        if permutations {
            let mut idxs: Vec<usize> = (0..subset.len()).collect();
            loop {
                let ordered: Vec<&str> = idxs.iter().map(|&i| subset[i].as_str()).collect();
                if with_sp {
                    push(ordered.join(" "), &mut out, &mut seen);
                }
                if no_sp {
                    push(ordered.join(""), &mut out, &mut seen);
                }
                if out.len() >= max_out || !next_permutation(&mut idxs) {
                    break;
                }
            }
        } else {
            let ordered: Vec<&str> = subset.iter().map(|s| s.as_str()).collect();
            if with_sp {
                push(ordered.join(" "), &mut out, &mut seen);
            }
            if no_sp {
                push(ordered.join(""), &mut out, &mut seen);
            }
        }
        if out.len() >= max_out {
            break;
        }
    }
    out
}

fn next_permutation(a: &mut [usize]) -> bool {
    // standard next_permutation
    if a.len() < 2 {
        return false;
    }
    let mut i = a.len() - 1;
    while i > 0 && a[i - 1] >= a[i] {
        i -= 1;
    }
    if i == 0 {
        return false;
    }
    let mut j = a.len() - 1;
    while a[j] <= a[i - 1] {
        j -= 1;
    }
    a.swap(i - 1, j);
    a[i..].reverse();
    true
}

/// Estimate count without allocating (approx upper bound)
pub fn estimate_word_bag_count(
    n_words: usize,
    combinations: bool,
    permutations: bool,
    join_modes: usize,
) -> u64 {
    let n = n_words.min(12) as u64;
    if n == 0 {
        return 0;
    }
    let joins = join_modes.max(1) as u64;
    if combinations && permutations {
        // sum_{k=1..n} P(n,k) = floor(n! * e) - 1
        let mut fact = 1u64;
        let mut sum = 0u64;
        for k in 1..=n {
            // P(n,k) = n!/(n-k)!
            fact = fact.saturating_mul(n - k + 1);
            sum = sum.saturating_add(fact);
        }
        return sum.saturating_mul(joins);
    }
    if permutations && !combinations {
        let mut fact = 1u64;
        for k in 1..=n {
            fact = fact.saturating_mul(k);
        }
        return fact.saturating_mul(joins);
    }
    if combinations && !permutations {
        // 2^n - 1 subsets
        return ((1u64 << n) - 1).saturating_mul(joins);
    }
    joins // single phrase * joins
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DictMatch {
    pub phrase: String,
    pub method: String,
    pub address: String,
    pub address_type: String,
    pub value_sats: u64,
    pub value_btc: f64,
    pub privkey_hex: String,
    /// Clé publique compressée (hex SEC1) pour explorers / sites web
    #[serde(default)]
    pub pubkey_hex: String,
    /// Clé publique non compressée (hex SEC1 04…)
    #[serde(default)]
    pub pubkey_uncompressed_hex: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GpuRateStat {
    pub id: i32,
    pub keys_tested: u64,
    pub keys_per_sec: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DictScanStatus {
    pub running: bool,
    pub phrases_total: u64,
    pub variants_total: u64,
    pub keys_tested: u64,
    pub matches_found: u64,
    /// Débit total (GPU + CPU)
    pub keys_per_sec: f64,
    pub elapsed_seconds: f64,
    pub progress_pct: f64,
    pub last_phrase: String,
    pub error: Option<String>,
    pub matches: Vec<DictMatch>,
    pub done: bool,
    /// "cpu" | "gpu:0,1,2+cpu"
    #[serde(default)]
    pub engine: String,
    #[serde(default)]
    pub gpu_util: Option<f64>,
    /// Vitesse par carte GPU
    #[serde(default)]
    pub gpu_rates: Vec<GpuRateStat>,
    /// Clés traitées par les workers CPU
    #[serde(default)]
    pub cpu_keys_tested: u64,
    #[serde(default)]
    pub cpu_keys_per_sec: f64,
    #[serde(default)]
    pub cpu_threads: u32,
}

#[derive(Clone)]
pub struct DictScanState {
    pub running: Arc<AtomicBool>,
    pub stop: Arc<AtomicBool>,
    pub keys_tested: Arc<AtomicU64>,
    pub status: Arc<Mutex<DictScanStatus>>,
}

impl DictScanState {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            keys_tested: Arc::new(AtomicU64::new(0)),
            status: Arc::new(Mutex::new(DictScanStatus::default())),
        }
    }
}

pub struct DictScanManager;

impl DictScanManager {
    pub fn list_corpora(project_dir: &str) -> Vec<serde_json::Value> {
        let mut out = Vec::new();
        let roots = [
            Path::new(project_dir).to_path_buf(),
            Path::new(project_dir).join("corpora"),
            Path::new(project_dir).join("data"),
        ];
        for dir in &roots {
            if !dir.is_dir() {
                continue;
            }
            let Ok(rd) = std::fs::read_dir(dir) else {
                continue;
            };
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                let lower = name.to_lowercase();
                if !lower.ends_with(".txt") {
                    continue;
                }
                let keep = lower.contains("brain")
                    || lower.contains("bible")
                    || lower.contains("corpus")
                    || lower.contains("phrase")
                    || lower.contains("bip39")
                    || lower.contains("word")
                    || lower.contains("valid")
                    || lower.contains("name")
                    || lower.contains("city")
                    || lower.contains("cities")
                    || lower.contains("japanese")
                    || lower.contains("surname")
                    || lower.contains("prenom")
                    || lower.contains("ville");
                if !keep {
                    continue;
                }
                if let Ok(meta) = e.metadata() {
                    // Avoid listing the same basename twice
                    if out.iter().any(|x: &serde_json::Value| {
                        x.get("name").and_then(|v| v.as_str()) == Some(name.as_str())
                    }) {
                        continue;
                    }
                    out.push(serde_json::json!({
                        "name": name,
                        "path": e.path().to_string_lossy(),
                        "size_mb": ((meta.len() as f64 / 1_048_576.0) * 100.0).round() / 100.0,
                    }));
                }
            }
        }
        out.sort_by(|a, b| {
            a["name"]
                .as_str()
                .unwrap_or("")
                .cmp(b["name"].as_str().unwrap_or(""))
        });
        out
    }

    pub fn status(state: &DictScanState) -> DictScanStatus {
        state
            .status
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn stop(state: &DictScanState) {
        state.stop.store(true, Ordering::SeqCst);
    }

    pub fn start(
        state: &DictScanState,
        index: Arc<FlatIndex>,
        project_dir: String,
        req: DictScanRequest,
    ) -> Result<()> {
        if state.running.load(Ordering::SeqCst) {
            anyhow::bail!("Dictionary scan already running");
        }

        let mut phrases: Vec<String> = req
            .phrases
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();

        if let Some(ref path) = req.corpus_path {
            let p = if Path::new(path).is_absolute() {
                path.clone()
            } else {
                format!("{}\\{}", project_dir, path)
            };
            let text = std::fs::read_to_string(&p)
                .map_err(|e| anyhow::anyhow!("Cannot read corpus {}: {}", p, e))?;
            for line in text.lines() {
                let t = line.trim();
                if !t.is_empty() && !t.starts_with('#') {
                    phrases.push(t.to_string());
                }
            }
        }

        phrases.sort();
        phrases.dedup();

        // ── Expansion permutations / combinaisons de mots ──
        let mut expanded_from_bag = Vec::new();
        if req.word_permutations || req.word_combinations {
            // Sac de mots = tokens de toutes les lignes collées (+ phrases multi-mots splitées)
            let mut bag: Vec<String> = Vec::new();
            for ph in &phrases {
                for tok in ph.split_whitespace() {
                    if !tok.is_empty() {
                        bag.push(tok.to_string());
                    }
                }
            }
            let max_w = req.max_perm_words.clamp(1, 10);
            let max_out = req.max_phrases.min(200_000);
            let join_modes = {
                let j = (req.join_with_space as usize) + (req.join_no_space as usize);
                if j == 0 {
                    2
                } else {
                    j
                }
            };
            let est = estimate_word_bag_count(
                bag.len().min(max_w),
                req.word_combinations,
                req.word_permutations,
                join_modes,
            );
            if est > 2_000_000 {
                anyhow::bail!(
                    "Trop de permutations/combinaisons estimées ({}). Réduis le nombre de mots (max {}) ou désactive combinaisons. n≈{}",
                    est,
                    max_w,
                    bag.len().min(max_w)
                );
            }
            expanded_from_bag = expand_word_bag(
                &bag,
                req.word_combinations,
                req.word_permutations,
                req.join_with_space,
                req.join_no_space,
                max_w,
                max_out,
            );
            // Keep original lines too (already in phrases)
            for e in &expanded_from_bag {
                phrases.push(e.clone());
            }
            phrases.sort();
            phrases.dedup();
        }

        if phrases.len() > req.max_phrases {
            phrases.truncate(req.max_phrases);
        }
        if phrases.is_empty() {
            anyhow::bail!("No phrases to scan");
        }

        let opts = req.options.clone();
        if !opts.sha256 && !opts.double_sha256 && !opts.md5_padded {
            anyhow::bail!("Active au moins une méthode de hash (SHA256 / SHA256d / MD5)");
        }

        // Total calculé (affixes inclus) — AUCUNE matérialisation des clés en RAM.
        // Le scan itère en boucles : phrase → bases texte → préfixe × suffixe → hashes.
        let mut variants_total: u64 = 0;
        for ph in &phrases {
            variants_total = variants_total
                .saturating_add(KeyChecker::count_brainwallet_keys(ph, &opts));
        }
        if variants_total == 0 {
            anyhow::bail!("Aucune variante à tester (phrases / options vides ?)");
        }
        let affix_m = crate::dashboard::key_checker::char_affix_multiplier(
            opts.char_prefix_len,
            opts.char_suffix_len,
        );
        eprintln!(
            "[dict] plan streaming: {} phrases · ~{} clés (affix_mult={} pref={} suf={}) — génération en boucles, pas de liste jobs",
            phrases.len(),
            variants_total,
            affix_m,
            opts.char_prefix_len,
            opts.char_suffix_len
        );

        let phrases_total = phrases.len() as u64;
        let _bag_expanded = expanded_from_bag.len();
        let min_value = req.min_value;
        let logical = num_cpus::get().max(1);

        // GPU d'abord : tenter CUDA avant de décider des threads CPU
        let mut engine = "cpu".to_string();
        let mut gpu_ids: Vec<i32> = Vec::new();
        if req.use_gpu {
            let n = crate::gpu::gpu_device_count();
            if n > 0 {
                let n_init = crate::gpu::gpu_init();
                if n_init > 0 {
                    gpu_ids = (0..n_init).collect();
                }
            }
        }
        let use_gpu_path = !gpu_ids.is_empty();

        // Max GPU / min CPU :
        // - GPU dispo → threads CPU = 0 par défaut (seulement si req.threads > 0 explicitement)
        // - pas de GPU → 50 % des cœurs
        let cpu_threads = if use_gpu_path {
            req.threads.filter(|&t| t > 0).unwrap_or(0).min(logical)
        } else {
            req.threads
                .filter(|&t| t > 0)
                .unwrap_or_else(|| (logical / 2).max(1))
                .max(1)
                .min(logical)
        };

        if use_gpu_path {
            engine = if cpu_threads == 0 {
                format!(
                    "gpu:{} ONLY",
                    gpu_ids
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            } else {
                format!(
                    "gpu:{}+cpu×{}",
                    gpu_ids
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    cpu_threads
                )
            };
        } else {
            engine = format!("cpu×{}", cpu_threads);
        }

        state.running.store(true, Ordering::SeqCst);
        state.stop.store(false, Ordering::SeqCst);
        state.keys_tested.store(0, Ordering::SeqCst);

        if let Ok(mut st) = state.status.lock() {
            *st = DictScanStatus {
                running: true,
                phrases_total,
                variants_total,
                keys_tested: 0,
                matches_found: 0,
                keys_per_sec: 0.0,
                elapsed_seconds: 0.0,
                progress_pct: 0.0,
                last_phrase: String::new(),
                error: None,
                matches: Vec::new(),
                done: false,
                engine: engine.clone(),
                gpu_util: None,
                gpu_rates: gpu_ids
                    .iter()
                    .map(|&id| GpuRateStat {
                        id,
                        keys_tested: 0,
                        keys_per_sec: 0.0,
                    })
                    .collect(),
                cpu_keys_tested: 0,
                cpu_keys_per_sec: 0.0,
                cpu_threads: cpu_threads as u32,
            };
        }

        let running = state.running.clone();
        let stop = state.stop.clone();
        let keys_tested = state.keys_tested.clone();
        let status = state.status.clone();

        std::thread::Builder::new()
            .name("dict-scan-orchestrator".into())
            .spawn(move || {
                let start = Instant::now();
                let secp = Secp256k1::<All>::new();
                let network = Network::Bitcoin;
                let matches_std = Arc::new(Mutex::new(Vec::<DictMatch>::new()));
                let last_phrase = Arc::new(Mutex::new(String::new()));

                // Phrases seulement en RAM — les clés sont générées en boucles (affixes inclus).
                let phrases = Arc::new(phrases);
                let opts = Arc::new(opts);
                let n_phrases = phrases.len();
                let cpu_tested = Arc::new(AtomicU64::new(0));
                let gpu_tested: Arc<Vec<AtomicU64>> = Arc::new(
                    gpu_ids
                        .iter()
                        .map(|_| AtomicU64::new(0))
                        .collect(),
                );

                // FULL = index UTXO copié en VRAM sur chaque carte + lookup on-device.
                // API: gpu_full=true (défaut). Env: BTC_GPU_DERIVE_ONLY=1 force DERIVE.
                let mut full_mode = false;
                // Max GPU : FULL VRAM forcé dès que possible (lookup on-device → zéro check CPU sauf hits)
                let want_full = std::env::var("BTC_GPU_DERIVE_ONLY")
                    .map(|v| v != "1")
                    .unwrap_or(true)
                    && (req.gpu_full || use_gpu_path);
                if use_gpu_path {
                    if !want_full {
                        eprintln!("[dict] FULL VRAM off (BTC_GPU_DERIVE_ONLY=1) — DERIVE+CPU (plus de CPU)");
                    } else {
                        let packed = index.serialize_script_entries_for_gpu();
                        let need_mb = index.gpu_index_bytes() as f64 / (1024.0 * 1024.0);
                        eprintln!(
                            "[dict] GPU FULL: chargement index ≈ {:.0} MB/GPU ({} scripts)…",
                            need_mb,
                            index.script_entries.len()
                        );
                        let t0 = Instant::now();
                        let rc = crate::gpu::gpu_load_index(
                            &packed,
                            &index.all_data,
                            &index.utxo_data,
                            index.script_entries.len() as u32,
                        );
                        full_mode = rc == 0;
                        eprintln!(
                            "[dict] GPU FULL load {} in {:.1}s",
                            if full_mode {
                                "OK — lookup on-device sur toutes les cartes"
                            } else {
                                "FAILED → fallback DERIVE+CPU"
                            },
                            t0.elapsed().as_secs_f64()
                        );
                        if let Ok(mut st) = status.lock() {
                            if full_mode {
                                st.engine = format!(
                                    "{} · FULL VRAM {:.0}MB/GPU",
                                    engine, need_mb
                                );
                            } else {
                                st.error = Some(
                                    "GPU FULL load failed — DERIVE+CPU".into(),
                                );
                            }
                        }
                    }
                }

                // Threads CUDA = 1 par clé dans le kernel.
                // BTC_GPU_LAUNCH : taille d'un lancement (défaut 8M FULL / max 32M).
                // Double-buffer (canal depth 2) + fill threads : le GPU ne attend pas le hash host.
                let n_cards_plan = gpu_ids.len().max(1);
                let per_gpu_batch: usize = std::env::var("BTC_GPU_LAUNCH")
                    .ok()
                    .or_else(|| std::env::var("BTC_DICT_GPU_BATCH").ok())
                    .or_else(|| std::env::var("BTC_GPU_THREADS").ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(if full_mode { 8_388_608 } else { 2_097_152 })
                    .clamp(524_288, 33_554_432);
                // Peu de fill threads (défaut 2) : chaque un tient un chunk en construction
                // (RAM ≈ (fill + channel_depth) × launch × 32 o). Monter BTC_GPU_FILL_THREADS si CPU libre.
                let fill_threads: usize = std::env::var("BTC_GPU_FILL_THREADS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(2)
                    .clamp(1, 8);
                let gpu_host_threads: usize = std::env::var("BTC_GPU_HOST_THREADS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(n_cards_plan)
                    .clamp(1, n_cards_plan);
                // Max GPU : 100 % des phrases aux GPU si CPU workers = 0 ; sinon 95/5
                let gpu_phrase_end: usize = if use_gpu_path {
                    if cpu_threads == 0 {
                        n_phrases
                    } else if n_phrases <= gpu_host_threads {
                        n_phrases
                    } else {
                        ((n_phrases * 95) / 100).max(gpu_host_threads).min(n_phrases)
                    }
                } else {
                    0
                };
                let phrase_cursor_cpu = Arc::new(AtomicUsize::new(gpu_phrase_end));
                eprintln!(
                    "[dict] MAX-GPU: phrases GPU[0..{}) CPU[{}..{}) keys≈{} launch_threads/GPU={} fill_thr={} hosts={} full={} cpu_workers={}",
                    gpu_phrase_end,
                    gpu_phrase_end,
                    n_phrases,
                    variants_total,
                    per_gpu_batch,
                    fill_threads,
                    gpu_host_threads,
                    full_mode,
                    cpu_threads
                );
                let addr_types: u32 = 0x01 | 0x02 | 0x04 | 0x08; // legacy+segwit+wrapped+taproot on-device
                let mode_label = if !use_gpu_path {
                    "CPU"
                } else if full_mode && cpu_threads == 0 {
                    "FULL GPU-only"
                } else if full_mode {
                    "FULL+CPU"
                } else {
                    "GPU+CPU"
                };
                // Un mutex par device CUDA — chaque host thread ne touche qu'une carte
                let device_locks: Arc<Vec<Mutex<()>>> = Arc::new(
                    (0..crate::gpu::gpu_device_count().max(1) as usize)
                        .map(|_| Mutex::new(()))
                        .collect(),
                );

                let publish_status = |keys_tested: &AtomicU64,
                                      cpu_tested: &AtomicU64,
                                      gpu_tested: &Vec<AtomicU64>,
                                      gpu_ids: &[i32],
                                      matches_std: &Arc<Mutex<Vec<DictMatch>>>,
                                      last_phrase: &Arc<Mutex<String>>,
                                      status: &Arc<Mutex<DictScanStatus>>,
                                      engine: &str,
                                      mode_label: &str,
                                      cpu_threads: usize,
                                      start: Instant,
                                      variants_total: u64| {
                    let tested = keys_tested.load(Ordering::Relaxed);
                    let c_test = cpu_tested.load(Ordering::Relaxed);
                    let elapsed = start.elapsed().as_secs_f64().max(0.001);
                    let mf = matches_std.lock().map(|g| g.len() as u64).unwrap_or(0);
                    let lp = last_phrase.lock().map(|g| g.clone()).unwrap_or_default();
                    let mut rates: Vec<GpuRateStat> = Vec::new();
                    for (i, &id) in gpu_ids.iter().enumerate() {
                        let k = gpu_tested
                            .get(i)
                            .map(|a| a.load(Ordering::Relaxed))
                            .unwrap_or(0);
                        rates.push(GpuRateStat {
                            id,
                            keys_tested: k,
                            keys_per_sec: k as f64 / elapsed,
                        });
                    }
                    if let Ok(mut st) = status.lock() {
                        st.keys_tested = tested;
                        st.matches_found = mf;
                        st.keys_per_sec = tested as f64 / elapsed;
                        st.elapsed_seconds = elapsed;
                        st.progress_pct = if variants_total > 0 {
                            (tested as f64 / variants_total as f64 * 100.0).min(99.9)
                        } else {
                            0.0
                        };
                        st.last_phrase = lp;
                        st.engine = format!(
                            "{} · {} · CPU×{}",
                            engine, mode_label, cpu_threads
                        );
                        st.gpu_rates = rates;
                        st.cpu_keys_tested = c_test;
                        st.cpu_keys_per_sec = c_test as f64 / elapsed;
                        st.cpu_threads = cpu_threads as u32;
                        if let Ok(m) = matches_std.lock() {
                            let take = m.len().saturating_sub(50);
                            st.matches = m[take..].to_vec();
                        }
                    }
                };

                let mut handles = Vec::new();

                // ── GPU MAX THREADS: multi-fill + double-buffer ──
                // 1 lancement CUDA = per_gpu_batch threads (jusqu'à 32M).
                // fill_threads hashe en parallèle → canal depth 2 = GPU toujours alimenté.
                if use_gpu_path {
                    let n_cards = gpu_ids.len().max(1);
                    let n_hosts = gpu_host_threads.min(n_cards);
                    for hi in 0..n_hosts {
                        let p_lo = gpu_phrase_end * hi / n_hosts;
                        let p_hi = gpu_phrase_end * (hi + 1) / n_hosts;
                        if p_lo >= p_hi {
                            continue;
                        }
                        let my_gpu_indices: Vec<usize> = (0..n_cards)
                            .filter(|gi| gi % n_hosts == hi)
                            .collect();
                        if my_gpu_indices.is_empty() {
                            continue;
                        }

                        let phrases = Arc::clone(&phrases);
                        let opts = Arc::clone(&opts);
                        let index = Arc::clone(&index);
                        let stop = Arc::clone(&stop);
                        let keys_tested = Arc::clone(&keys_tested);
                        let cpu_tested = Arc::clone(&cpu_tested);
                        let gpu_tested = Arc::clone(&gpu_tested);
                        let matches_std = Arc::clone(&matches_std);
                        let last_phrase = Arc::clone(&last_phrase);
                        let status = Arc::clone(&status);
                        let engine = engine.clone();
                        let secp = secp.clone();
                        let gpu_ids_all = gpu_ids.clone();
                        let device_locks = Arc::clone(&device_locks);
                        let batch = per_gpu_batch;
                        let n_fill = fill_threads.max(1);
                        let start = start;

                        handles.push(
                            std::thread::Builder::new()
                                .name(format!("dict-gpu-h{}", hi))
                                .spawn(move || {
                                    // depth 2 = double buffer (un chunk en compute, un en fill)
                                    let (tx, rx) = mpsc::sync_channel::<GpuLaunchChunk>(2);
                                    let phrase_cur = Arc::new(AtomicUsize::new(p_lo));
                                    let mut fill_handles = Vec::new();

                                    for fi in 0..n_fill {
                                        let tx = tx.clone();
                                        let phrases = Arc::clone(&phrases);
                                        let opts = Arc::clone(&opts);
                                        let stop = Arc::clone(&stop);
                                        let phrase_cur = Arc::clone(&phrase_cur);
                                        let last_phrase = Arc::clone(&last_phrase);
                                        fill_handles.push(
                                            std::thread::Builder::new()
                                                .name(format!("dict-fill-{}-{}", hi, fi))
                                                .spawn(move || {
                                                    let mut chunk = GpuLaunchChunk::new(batch);
                                                    loop {
                                                        if stop.load(Ordering::Relaxed) {
                                                            break;
                                                        }
                                                        let pi = phrase_cur
                                                            .fetch_add(1, Ordering::Relaxed);
                                                        if pi >= p_hi {
                                                            break;
                                                        }
                                                        let ph = &phrases[pi];
                                                        if fi == 0 {
                                                            if let Ok(mut lp) = last_phrase.lock() {
                                                                *lp = format!("[GPU-fill] {}", ph);
                                                            }
                                                        }
                                                        let pi_u = pi as u32;
                                                        let cont = KeyChecker::brainwallet_for_each(
                                                            ph,
                                                            &opts,
                                                            |k| {
                                                                if stop.load(Ordering::Relaxed) {
                                                                    return false;
                                                                }
                                                                let mid =
                                                                    if k.method.starts_with("SHA256d")
                                                                    {
                                                                        1u8
                                                                    } else if k
                                                                        .method
                                                                        .starts_with("MD5")
                                                                    {
                                                                        2u8
                                                                    } else {
                                                                        0u8
                                                                    };
                                                                if !chunk.push(&k.bytes, pi_u, mid) {
                                                                    // should not happen
                                                                    return false;
                                                                }
                                                                if chunk.is_full() {
                                                                    let full = std::mem::replace(
                                                                        &mut chunk,
                                                                        GpuLaunchChunk::new(batch),
                                                                    );
                                                                    if tx.send(full).is_err() {
                                                                        return false;
                                                                    }
                                                                }
                                                                true
                                                            },
                                                        );
                                                        if !cont {
                                                            break;
                                                        }
                                                    }
                                                    if chunk.n > 0 {
                                                        let _ = tx.send(chunk);
                                                    }
                                                })
                                                .expect("spawn fill"),
                                        );
                                    }
                                    drop(tx); // consumer gets EOF when all fillers done

                                    let mut values = vec![0u64; batch];
                                    let mut out_buf = if full_mode {
                                        Vec::new()
                                    } else {
                                        vec![0u8; batch * 85]
                                    };
                                    let mut batches_ok = 0u64;
                                    let mut batches_fail = 0u64;
                                    let mut rr = 0usize;

                                    while let Ok(chunk) = rx.recv() {
                                        if stop.load(Ordering::Relaxed) {
                                            break;
                                        }
                                        let n = chunk.n;
                                        if n == 0 {
                                            continue;
                                        }
                                        let gi = my_gpu_indices[rr % my_gpu_indices.len()];
                                        rr = rr.wrapping_add(1);
                                        let dev_id = gpu_ids_all[gi];
                                        let dev_ids_one = [dev_id];
                                        let lock_slot = (dev_id as usize)
                                            .min(device_locks.len().saturating_sub(1));
                                        let _guard = device_locks.get(lock_slot).map(|m| {
                                            m.lock().unwrap_or_else(|e| e.into_inner())
                                        });

                                        let mut used_gpu = false;
                                        if full_mode {
                                            let rc = crate::gpu::gpu_derive_lookup(
                                                &chunk.priv_buf[..n * 32],
                                                &mut values[..n],
                                                n,
                                                addr_types,
                                                &dev_ids_one,
                                            );
                                            drop(_guard);
                                            if rc == 0 {
                                                used_gpu = true;
                                                batches_ok += 1;
                                                for i in 0..n {
                                                    if values[i] == 0 || values[i] < min_value {
                                                        continue;
                                                    }
                                                    let pii = chunk.meta_phrase[i] as usize;
                                                    let phr = phrases
                                                        .get(pii)
                                                        .map(|s| s.as_str())
                                                        .unwrap_or("?");
                                                    let method = match chunk.meta_mid[i] {
                                                        1 => "SHA256d+affix",
                                                        2 => "MD5pad+affix",
                                                        _ => "SHA256+affix",
                                                    };
                                                    let mut kb = [0u8; 32];
                                                    kb.copy_from_slice(
                                                        &chunk.priv_buf[i * 32..i * 32 + 32],
                                                    );
                                                    cpu_check_one(
                                                        &index,
                                                        &secp,
                                                        network,
                                                        phr,
                                                        method,
                                                        &kb,
                                                        min_value,
                                                        &matches_std,
                                                    );
                                                }
                                            } else {
                                                batches_fail += 1;
                                                for i in 0..n {
                                                    let pii = chunk.meta_phrase[i] as usize;
                                                    let phr = phrases
                                                        .get(pii)
                                                        .map(|s| s.as_str())
                                                        .unwrap_or("?");
                                                    let method = match chunk.meta_mid[i] {
                                                        1 => "SHA256d+affix",
                                                        2 => "MD5pad+affix",
                                                        _ => "SHA256+affix",
                                                    };
                                                    let mut kb = [0u8; 32];
                                                    kb.copy_from_slice(
                                                        &chunk.priv_buf[i * 32..i * 32 + 32],
                                                    );
                                                    cpu_check_one(
                                                        &index,
                                                        &secp,
                                                        network,
                                                        phr,
                                                        method,
                                                        &kb,
                                                        min_value,
                                                        &matches_std,
                                                    );
                                                }
                                            }
                                        } else {
                                            let rc = crate::gpu::gpu_derive_multi(
                                                &chunk.priv_buf[..n * 32],
                                                &mut out_buf[..n * 85],
                                                n,
                                                &dev_ids_one,
                                            );
                                            drop(_guard);
                                            if rc == 0 {
                                                used_gpu = true;
                                                batches_ok += 1;
                                            } else {
                                                batches_fail += 1;
                                            }
                                            for i in 0..n {
                                                let pii = chunk.meta_phrase[i] as usize;
                                                let phr = phrases
                                                    .get(pii)
                                                    .map(|s| s.as_str())
                                                    .unwrap_or("?");
                                                let method = match chunk.meta_mid[i] {
                                                    1 => "SHA256d+affix",
                                                    2 => "MD5pad+affix",
                                                    _ => "SHA256+affix",
                                                };
                                                let mut kb = [0u8; 32];
                                                kb.copy_from_slice(
                                                    &chunk.priv_buf[i * 32..i * 32 + 32],
                                                );
                                                cpu_check_one(
                                                    &index,
                                                    &secp,
                                                    network,
                                                    phr,
                                                    method,
                                                    &kb,
                                                    min_value,
                                                    &matches_std,
                                                );
                                            }
                                        }
                                        if used_gpu {
                                            if let Some(a) = gpu_tested.get(gi) {
                                                a.fetch_add(n as u64, Ordering::Relaxed);
                                            }
                                        }
                                        keys_tested.fetch_add(n as u64, Ordering::Relaxed);
                                        if batches_ok > 0 && batches_ok % 2 == 0 {
                                            if let Some(u) = sample_nvidia_gpu_util() {
                                                if let Ok(mut st) = status.lock() {
                                                    st.gpu_util = Some(u);
                                                }
                                            }
                                        }
                                        publish_status(
                                            &keys_tested,
                                            &cpu_tested,
                                            &gpu_tested,
                                            &gpu_ids_all,
                                            &matches_std,
                                            &last_phrase,
                                            &status,
                                            &engine,
                                            mode_label,
                                            cpu_threads,
                                            start,
                                            variants_total,
                                        );
                                    }

                                    for h in fill_handles {
                                        let _ = h.join();
                                    }
                                    eprintln!(
                                        "[dict] GPU host{} phrases=[{}..{}) launch_threads={} ok_batches={} fail={} fill_thr={}",
                                        hi, p_lo, p_hi, batch, batches_ok, batches_fail, n_fill
                                    );
                                })
                                .expect("spawn dict-gpu-host"),
                        );
                    }
                }

                // ── CPU : curseur de phrases [gpu_phrase_end, n) — expansion en boucles ──
                for ti in 0..cpu_threads {
                    let phrases = Arc::clone(&phrases);
                    let opts = Arc::clone(&opts);
                    let phrase_cursor = Arc::clone(&phrase_cursor_cpu);
                    let index = Arc::clone(&index);
                    let stop = Arc::clone(&stop);
                    let keys_tested = Arc::clone(&keys_tested);
                    let cpu_tested = Arc::clone(&cpu_tested);
                    let gpu_tested = Arc::clone(&gpu_tested);
                    let matches_std = Arc::clone(&matches_std);
                    let last_phrase = Arc::clone(&last_phrase);
                    let status = Arc::clone(&status);
                    let engine = engine.clone();
                    let gpu_ids_c = gpu_ids.clone();
                    let secp = secp.clone();
                    let start = start;

                    handles.push(
                        std::thread::Builder::new()
                            .name(format!("dict-cpu-{}", ti))
                            .spawn(move || {
                                let mut since_pub = 0u64;
                                loop {
                                    if stop.load(Ordering::Relaxed) {
                                        break;
                                    }
                                    let pi = phrase_cursor.fetch_add(1, Ordering::Relaxed);
                                    if pi >= n_phrases {
                                        break;
                                    }
                                    let ph = &phrases[pi];
                                    if let Ok(mut lp) = last_phrase.lock() {
                                        *lp = format!("[CPU{}] {}", ti, ph);
                                    }
                                    let cont = KeyChecker::brainwallet_for_each(ph, &opts, |k| {
                                        if stop.load(Ordering::Relaxed) {
                                            return false;
                                        }
                                        cpu_check_one(
                                            &index,
                                            &secp,
                                            network,
                                            ph,
                                            &k.method,
                                            &k.bytes,
                                            min_value,
                                            &matches_std,
                                        );
                                        keys_tested.fetch_add(1, Ordering::Relaxed);
                                        cpu_tested.fetch_add(1, Ordering::Relaxed);
                                        since_pub += 1;
                                        if since_pub >= 256 {
                                            since_pub = 0;
                                            publish_status(
                                                &keys_tested,
                                                &cpu_tested,
                                                &gpu_tested,
                                                &gpu_ids_c,
                                                &matches_std,
                                                &last_phrase,
                                                &status,
                                                &engine,
                                                mode_label,
                                                cpu_threads,
                                                start,
                                                variants_total,
                                            );
                                        }
                                        true
                                    });
                                    if !cont {
                                        break;
                                    }
                                }
                            })
                            .expect("spawn dict-cpu"),
                    );
                }

                for h in handles {
                    let _ = h.join();
                }
                if use_gpu_path {
                    crate::gpu::gpu_unload_index();
                    crate::gpu::gpu_cleanup();
                }

                let tested = keys_tested.load(Ordering::Relaxed);
                let c_test = cpu_tested.load(Ordering::Relaxed);
                let elapsed = start.elapsed().as_secs_f64().max(0.001);
                let matches_final = matches_std.lock().map(|g| g.clone()).unwrap_or_default();
                let mf = matches_final.len() as u64;
                let mut rates: Vec<GpuRateStat> = Vec::new();
                for (i, &id) in gpu_ids.iter().enumerate() {
                    let k = gpu_tested
                        .get(i)
                        .map(|a| a.load(Ordering::Relaxed))
                        .unwrap_or(0);
                    rates.push(GpuRateStat {
                        id,
                        keys_tested: k,
                        keys_per_sec: k as f64 / elapsed,
                    });
                }
                if let Ok(mut st) = status.lock() {
                    st.running = false;
                    st.done = true;
                    st.keys_tested = tested;
                    st.matches_found = mf;
                    st.keys_per_sec = tested as f64 / elapsed;
                    st.elapsed_seconds = elapsed;
                    st.engine = engine;
                    st.progress_pct = 100.0;
                    st.matches = matches_final;
                    st.last_phrase = last_phrase.lock().map(|g| g.clone()).unwrap_or_default();
                    st.gpu_rates = rates;
                    st.cpu_keys_tested = c_test;
                    st.cpu_keys_per_sec = c_test as f64 / elapsed;
                    st.cpu_threads = cpu_threads as u32;
                }
                running.store(false, Ordering::SeqCst);
            })
            .map_err(|e| anyhow::anyhow!("spawn dict scan: {}", e))?;

        Ok(())
    }
}

/// Moyenne d'utilisation GPU via nvidia-smi (échantillonnage léger pour le dashboard).

/// Un lancement CUDA = `cap` threads (1 clé / thread). Double-buffer via canal.
struct GpuLaunchChunk {
    priv_buf: Vec<u8>,
    meta_phrase: Vec<u32>,
    meta_mid: Vec<u8>,
    n: usize,
    cap: usize,
}

impl GpuLaunchChunk {
    fn new(cap: usize) -> Self {
        Self {
            priv_buf: vec![0u8; cap * 32],
            meta_phrase: vec![0u32; cap],
            meta_mid: vec![0u8; cap],
            n: 0,
            cap,
        }
    }

    fn push(&mut self, key: &[u8; 32], phrase_idx: u32, mid: u8) -> bool {
        if self.n >= self.cap {
            return false;
        }
        let i = self.n;
        self.priv_buf[i * 32..i * 32 + 32].copy_from_slice(key);
        self.meta_phrase[i] = phrase_idx;
        self.meta_mid[i] = mid;
        self.n += 1;
        true
    }

    fn is_full(&self) -> bool {
        self.n >= self.cap
    }
}

fn sample_nvidia_gpu_util() -> Option<f64> {
    let out = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=utilization.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut sum = 0.0f64;
    let mut n = 0u32;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Ok(v) = t.parse::<f64>() {
            sum += v;
            n += 1;
        }
    }
    if n == 0 {
        None
    } else {
        Some(sum / n as f64)
    }
}

/// CPU path: derive addresses + FlatIndex lookup for one private key.
fn cpu_check_one(
    fi: &FlatIndex,
    secp: &Secp256k1<All>,
    network: Network,
    phrase: &str,
    method: &str,
    key_bytes: &[u8; 32],
    min_value: u64,
    matches_std: &Arc<Mutex<Vec<DictMatch>>>,
) {
    let Ok(sk) = bitcoin::secp256k1::SecretKey::from_slice(key_bytes) else {
        return;
    };
    let pk = PrivateKey {
        inner: sk,
        network: network.into(),
        compressed: true,
    };
    let (pub_comp_hex, pub_uncomp_hex) =
        crate::key_archive::pubkeys_from_priv_hex(&hex::encode(key_bytes))
            .unwrap_or_default();

    if let Ok(comp) = CompressedPublicKey::from_private_key(secp, &pk) {
        let pubk = pk.public_key(secp);
        let candidates = [
            (Address::p2pkh(pubk, network), "P2PKH"),
            (Address::p2wpkh(&comp, network), "P2WPKH"),
            (Address::p2shwpkh(&comp, network), "P2SH-P2WPKH"),
            (Address::p2tr(secp, comp.into(), None, network), "P2TR"),
        ];
        for (addr, atype) in candidates {
            let val = fi.lookup(addr.script_pubkey().as_bytes());
            if val > min_value.max(0) && val > 0 {
                if let Ok(mut m) = matches_std.lock() {
                    m.push(DictMatch {
                        phrase: phrase.to_string(),
                        method: method.to_string(),
                        address: addr.to_string(),
                        address_type: atype.into(),
                        value_sats: val,
                        value_btc: val as f64 / 1e8,
                        privkey_hex: hex::encode(key_bytes),
                        pubkey_hex: pub_comp_hex.clone(),
                        pubkey_uncompressed_hex: pub_uncomp_hex.clone(),
                    });
                }
                crate::alert_beep::alert_balance_found();
                let arch = KeyArchive::new(r"Y:\btcsolver");
                let _ = arch.record(ArchivedKey::from_utxo_hit(
                    hex::encode(key_bytes),
                    None,
                    vec![addr.to_string()],
                    val,
                    "dict_scan",
                    Some(method.to_string()),
                    Some(phrase.to_string()),
                ));
            }
        }
    }
    let pk_u = PrivateKey {
        inner: sk,
        network: network.into(),
        compressed: false,
    };
    let pub_u = pk_u.public_key(secp);
    let addr_u = Address::p2pkh(pub_u, network);
    let val = fi.lookup(addr_u.script_pubkey().as_bytes());
    if val > 0 && val >= min_value {
        if let Ok(mut m) = matches_std.lock() {
            m.push(DictMatch {
                phrase: phrase.to_string(),
                method: format!("{} +uncomp", method),
                address: addr_u.to_string(),
                address_type: "P2PKH-uncompressed".into(),
                value_sats: val,
                value_btc: val as f64 / 1e8,
                privkey_hex: hex::encode(key_bytes),
                pubkey_hex: pub_comp_hex.clone(),
                pubkey_uncompressed_hex: pub_uncomp_hex.clone(),
            });
        }
        crate::alert_beep::alert_balance_found();
        let arch = KeyArchive::new(r"Y:\btcsolver");
        let _ = arch.record(ArchivedKey::from_utxo_hit(
            hex::encode(key_bytes),
            None,
            vec![addr_u.to_string()],
            val,
            "dict_scan",
            Some(format!("{} +uncomp", method)),
            Some(phrase.to_string()),
        ));
    }
}
