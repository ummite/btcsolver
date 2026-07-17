"""Replace GPU host loop with multi-fill + double-buffer high-thread launches."""
from pathlib import Path

p = Path(__file__).with_name("dict_scan.rs")
text = p.read_text(encoding="utf-8")

start = text.find("                // ── GPU: 1 host / carte")
end = text.find("                // ── CPU : curseur de phrases")
assert start > 0 and end > start, (start, end)

new = r'''                // ── GPU MAX THREADS: multi-fill + double-buffer ──
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

'''

text = text[:start] + new + text[end:]

# Insert GpuLaunchChunk struct before sample_nvidia
marker = "fn sample_nvidia_gpu_util()"
idx = text.find(marker)
assert idx > 0
struct_def = r'''
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

'''
text = text[:idx] + struct_def + text[idx:]

p.write_text(text, encoding="utf-8", newline="\n")
print("OK", p.stat().st_size)
print("GpuLaunchChunk", "GpuLaunchChunk" in text)
print("sync_channel", "sync_channel" in text)
