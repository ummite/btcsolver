from pathlib import Path

p = Path(__file__).with_name("dict_scan.rs")
t = p.read_text(encoding="utf-8")

if "building = Arc::new(Mutex::new(GpuLaunchChunk" in t:
    print("already shared")
    raise SystemExit(0)

# Find the fill thread creation block by unique marker
a = t.find("let (tx, rx) = mpsc::sync_channel::<GpuLaunchChunk>(2);")
if a < 0:
    raise SystemExit("channel not found")
b = t.find("drop(tx); // consumer gets EOF when all fillers done", a)
if b < 0:
    # alternate
    b = t.find("drop(tx);", a)
if b < 0:
    raise SystemExit("drop tx not found")
# include drop line
b_end = t.find("\n", b) + 1

new = r'''let (tx, rx) = mpsc::sync_channel::<GpuLaunchChunk>(2);
                                    let phrase_cur = Arc::new(AtomicUsize::new(p_lo));
                                    let building = Arc::new(Mutex::new(GpuLaunchChunk::new(batch)));
                                    let mut fill_handles = Vec::new();

                                    for fi in 0..n_fill {
                                        let tx = tx.clone();
                                        let phrases = Arc::clone(&phrases);
                                        let opts = Arc::clone(&opts);
                                        let stop = Arc::clone(&stop);
                                        let phrase_cur = Arc::clone(&phrase_cur);
                                        let last_phrase = Arc::clone(&last_phrase);
                                        let building = Arc::clone(&building);
                                        let batch_cap = batch;
                                        fill_handles.push(
                                            std::thread::Builder::new()
                                                .name(format!("dict-fill-{}-{}", hi, fi))
                                                .spawn(move || {
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
                                                                *lp = format!(
                                                                    "[GPU×{} thr] {}",
                                                                    batch_cap, ph
                                                                );
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
                                                                let mid = if k
                                                                    .method
                                                                    .starts_with("SHA256d")
                                                                {
                                                                    1u8
                                                                } else if k.method.starts_with("MD5")
                                                                {
                                                                    2u8
                                                                } else {
                                                                    0u8
                                                                };
                                                                let mut to_send = None;
                                                                {
                                                                    let mut chunk = building
                                                                        .lock()
                                                                        .unwrap_or_else(|e| {
                                                                            e.into_inner()
                                                                        });
                                                                    if !chunk.push(
                                                                        &k.bytes, pi_u, mid,
                                                                    ) {
                                                                        return false;
                                                                    }
                                                                    if chunk.is_full() {
                                                                        let full = std::mem::replace(
                                                                            &mut *chunk,
                                                                            GpuLaunchChunk::new(
                                                                                batch_cap,
                                                                            ),
                                                                        );
                                                                        to_send = Some(full);
                                                                    }
                                                                }
                                                                if let Some(full) = to_send {
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
                                                })
                                                .expect("spawn fill"),
                                        );
                                    }
                                    drop(tx);
'''

t = t[:a] + new + t[b_end:]

# After all fillers join, flush remaining building chunk — insert before join loop
join_marker = "for h in fill_handles {"
ji = t.find(join_marker)
if ji < 0:
    raise SystemExit("join marker missing")
flush_rest = r'''// dernier chunk partiel
                                    {
                                        let mut chunk =
                                            building.lock().unwrap_or_else(|e| e.into_inner());
                                        if chunk.n > 0 {
                                            // channel already closed — process inline via local
                                            // (consumer done). Re-open not possible; run GPU here.
                                            let n = chunk.n;
                                            let gi = my_gpu_indices[0];
                                            let dev_id = gpu_ids_all[gi];
                                            let dev_ids_one = [dev_id];
                                            let lock_slot = (dev_id as usize)
                                                .min(device_locks.len().saturating_sub(1));
                                            let _guard = device_locks.get(lock_slot).map(|m| {
                                                m.lock().unwrap_or_else(|e| e.into_inner())
                                            });
                                            if full_mode {
                                                let mut values = vec![0u64; n];
                                                let rc = crate::gpu::gpu_derive_lookup(
                                                    &chunk.priv_buf[..n * 32],
                                                    &mut values[..n],
                                                    n,
                                                    addr_types,
                                                    &dev_ids_one,
                                                );
                                                drop(_guard);
                                                if rc == 0 {
                                                    batches_ok += 1;
                                                    if let Some(a) = gpu_tested.get(gi) {
                                                        a.fetch_add(n as u64, Ordering::Relaxed);
                                                    }
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
                                                }
                                            }
                                            keys_tested.fetch_add(n as u64, Ordering::Relaxed);
                                            chunk.n = 0;
                                        }
                                    }
                                    '''

# Actually order is wrong: we need to join fillers FIRST then flush building, but consumer already exited.
# Better: fillers flush partial via a final send before exit - need channel open.
# Simpler: don't drop(tx) until after join; consumer runs in parallel; fillers on exit flush building with lock and send.
# Revert complex flush - have each filler not own partial; one designated flusher after join with tx still open.

# Simpler approach: after join fillers, building may have remainder - but rx already EOF.
# Process remainder inline (code above) AFTER join.

# Move join before remainder flush - currently join is after while recv. Correct order:
# 1. drop(tx) - wait NO if we drop tx before join, fillers may still send
# Correct: join fillers first WITHOUT dropping tx - they hold clones. When they finish, drop original tx... 
# Actually: drop original at start, fillers have clones, when all fillers end all clones dropped, EOF, recv ends, then join is noop.

# Remainder in building: fillers must send partial on exit.
# Add to filler end: 
# after loop, lock building, if n>0 and fi==0 only, send partial once.

# For patch simplicity: after while recv, join, then inline remainder on building.

if "dernier chunk partiel" not in t:
    t = t[:ji] + flush_rest + t[ji:]

p.write_text(t, encoding="utf-8", newline="\n")
print("OK", p.stat().st_size)
