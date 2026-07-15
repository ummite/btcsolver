//! FlatIndex: sorted flat array + binary search replacement for HashMap.
//!
//! Memory comparison (20.7M scripts, ~127M UTXOs):
//!   HashMap<Vec<u8>, Vec<(Txid, u32, u64)>>  ~ 39.7 GB  (pointer chasing, hash overhead)
//!   FlatIndex                                 ~  6.5 GB  (contiguous, cache-friendly)

use anyhow::Result;
use bitcoin_hashes::Hash;
use std::io::{BufReader, Read, Write, Cursor};

/// A single UTXO entry — 44 bytes, no padding.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UtxoEntry {
    pub txid: [u8; 32],
    pub vout: u32,
    pub value: u64,
}

impl UtxoEntry {
    pub const SIZE: usize = 32 + 4 + 8; // 44 bytes
}

/// A script entry in the sorted array — 12 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ScriptEntry {
    pub script_offset: u32,
    pub script_len: u16,
    pub utxo_offset: u32,
    pub utxo_count: u32,
}

/// The flat index: sorted scripts + contiguous UTXO data.
pub struct FlatIndex {
    pub script_entries: Vec<ScriptEntry>,
    pub all_data: Vec<u8>,
    pub utxo_data: Vec<u8>,
    pub num_scripts: usize,
    pub total_utxos: usize,
}

impl FlatIndex {
    pub fn new() -> Self {
        Self {
            script_entries: Vec::new(),
            all_data: Vec::new(),
            utxo_data: Vec::new(),
            num_scripts: 0,
            total_utxos: 0,
        }
    }

    /// Build from a HashMap (for migration from v1 format).
    pub fn from_hashmap(
        map: &std::collections::HashMap<Vec<u8>, Vec<(bitcoin::Txid, u32, u64)>>,
        min_value: u64,
    ) -> Self {
        let mut entries: Vec<(Vec<u8>, Vec<UtxoEntry>)> = map
            .iter()
            .filter_map(|(script, utxos)| {
                let filtered: Vec<UtxoEntry> = utxos
                    .iter()
                    .filter(|(_, _, val)| *val >= min_value)
                    .map(|(txid, vout, value)| UtxoEntry {
                        txid: *txid.as_byte_array(),
                        vout: *vout,
                        value: *value,
                    })
                    .collect();
                if filtered.is_empty() {
                    None
                } else {
                    Some((script.clone(), filtered))
                }
            })
            .collect();

        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let num_scripts = entries.len();

        let mut data_size = 0usize;
        let mut utxo_size = 0usize;
        for (script, utxos) in &entries {
            data_size += script.len();
            utxo_size += utxos.len() * UtxoEntry::SIZE;
        }

        let mut all_data = Vec::with_capacity(data_size);
        let mut utxo_data = Vec::with_capacity(utxo_size);
        let mut script_entries = Vec::with_capacity(num_scripts);
        let mut total_utxos = 0usize;

        for (script, utxos) in &entries {
            let s_offset = all_data.len() as u32;
            all_data.extend_from_slice(script);

            let u_offset = utxo_data.len() as u32;
            for utxo in utxos {
                utxo_data.extend_from_slice(&utxo.txid);
                utxo_data.extend_from_slice(&utxo.vout.to_le_bytes());
                utxo_data.extend_from_slice(&utxo.value.to_le_bytes());
            }

            script_entries.push(ScriptEntry {
                script_offset: s_offset,
                script_len: script.len() as u16,
                utxo_offset: u_offset,
                utxo_count: utxos.len() as u32,
            });

            total_utxos += utxos.len();
        }

        Self {
            script_entries,
            all_data,
            utxo_data,
            num_scripts,
            total_utxos,
        }
    }

    /// Compare a script in the flat data (at entry index `i`) with `needle`.
    /// Uses slice references (zero-cost — just pointer + length) with SIMD-optimized cmp.
    #[inline]
    fn cmp_script_at(&self, i: usize, needle: &[u8]) -> std::cmp::Ordering {
        let entry = self.script_entries[i];
        let s_len = entry.script_len as usize;
        let s_start = entry.script_offset as usize;
        let s_end = s_start + s_len;
        self.all_data[s_start..s_end].cmp(needle)
    }

    /// Look up a script and return the total value of all UTXOs for that script.
    /// Uses SIMD-optimized slice comparison via cmp_script_at.
    #[inline]
    pub fn lookup(&self, script: &[u8]) -> u64 {
        if self.script_entries.is_empty() {
            return 0;
        }

        let mut lo: usize = 0;
        let mut hi: usize = self.script_entries.len();

        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            match self.cmp_script_at(mid, script) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => {
                    let mut total = 0u64;

                    // Scan backward for earlier matches (shouldn't happen with unique scripts, but safe)
                    let mut idx = mid;
                    while idx > 0 {
                        if self.cmp_script_at(idx - 1, script) != std::cmp::Ordering::Equal {
                            break;
                        }
                        idx -= 1;
                    }

                    // Scan forward including the match at `mid`
                    while idx < self.script_entries.len() {
                        if self.cmp_script_at(idx, script) != std::cmp::Ordering::Equal {
                            break;
                        }

                        let cur = self.script_entries[idx];
                        let u_start = cur.utxo_offset as usize;
                        let u_end = u_start + (cur.utxo_count as usize) * UtxoEntry::SIZE;
                        let utxo_slice = &self.utxo_data[u_start..u_end];
                        for chunk in utxo_slice.chunks_exact(UtxoEntry::SIZE) {
                            let value = u64::from_le_bytes(chunk[36..44].try_into().unwrap());
                            total += value;
                        }

                        idx += 1;
                    }

                    return total;
                }
            }
        }

        0
    }

    /// Look up a script and return total value + UTXO count.
    #[inline]
    pub fn lookup_with_count(&self, script: &[u8]) -> (u64, usize) {
        if self.script_entries.is_empty() {
            return (0, 0);
        }

        let mut lo: usize = 0;
        let mut hi: usize = self.script_entries.len();

        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            match self.cmp_script_at(mid, script) {
                std::cmp::Ordering::Less => lo = mid + 1,
                std::cmp::Ordering::Greater => hi = mid,
                std::cmp::Ordering::Equal => {
                    let mut total = 0u64;
                    let mut count = 0usize;

                    let mut idx = mid;
                    while idx < self.script_entries.len() {
                        if self.cmp_script_at(idx, script) != std::cmp::Ordering::Equal {
                            break;
                        }

                        let cur = self.script_entries[idx];
                        let u_start = cur.utxo_offset as usize;
                        let u_end = u_start + (cur.utxo_count as usize) * UtxoEntry::SIZE;
                        let utxo_slice = &self.utxo_data[u_start..u_end];
                        for chunk in utxo_slice.chunks_exact(UtxoEntry::SIZE) {
                            let value = u64::from_le_bytes(chunk[36..44].try_into().unwrap());
                            total += value;
                            count += 1;
                        }

                        idx += 1;
                    }

                    return (total, count);
                }
            }
        }

        (0, 0)
    }

    /// Get memory usage in bytes.
    pub fn memory_usage_bytes(&self) -> usize {
        self.script_entries.len() * std::mem::size_of::<ScriptEntry>()
            + self.all_data.len()
            + self.utxo_data.len()
    }

    /// Print stats.
    pub fn print_stats(&self) {
        let mb = self.memory_usage_bytes() as f64 / 1_048_576.0;
        eprintln!(
            "  FlatIndex: {} scripts, {} UTXOs, {:.1} MB RAM",
            self.num_scripts, self.total_utxos, mb
        );
    }

    // ─── Snapshot I/O ─────────────────────────────────────────────────────

    /// Load from a snapshot file (supports both v1 legacy and v2 flat formats).
    pub fn load_from_snapshot(snapshot_path: &str, min_value: u64) -> Result<Self> {
        let file = std::fs::File::open(snapshot_path)?;
        let file_len = file.metadata()?.len();
        let mut f = BufReader::new(file);

        let mut magic = [0u8; 4];
        f.read_exact(&mut magic)?;

        if magic == *b"BTCS" {
            let mut ver_buf = [0u8; 1];
            f.read_exact(&mut ver_buf)?;
            let version = ver_buf[0];

            let mut comp_size_buf = [0u8; 8];
            f.read_exact(&mut comp_size_buf)?;
            // Skip compressed_size

            let mut raw_size_buf = [0u8; 8];
            f.read_exact(&mut raw_size_buf)?;

            let mut compressed = Vec::new();
            f.read_to_end(&mut compressed)?;

            let decompressed = zstd::decode_all(Cursor::new(&compressed))?;
            let compressed_mb = file_len as f64 / 1_048_576.0;
            let raw_mb = decompressed.len() as f64 / 1_048_576.0;
            println!(
                "  Decompressed snapshot: {:.1} MB -> {:.1} MB ({:.1}:1 ratio)",
                compressed_mb, raw_mb, raw_mb / compressed_mb
            );

            if version == 2 {
                Self::parse_flat_v2(&decompressed, min_value, snapshot_path)
            } else {
                Self::parse_legacy(&decompressed, min_value, snapshot_path)
            }
        } else {
            let mut raw = magic.to_vec();
            f.read_to_end(&mut raw)?;
            println!(
                "  Legacy snapshot format (uncompressed, {:.1} MB)",
                file_len as f64 / 1_048_576.0
            );
            Self::parse_legacy(&raw, min_value, snapshot_path)
        }
    }

    /// Parse v2 flat format: [num_scripts:4][total_utxos:4][entries+data...]
    fn parse_flat_v2(data: &[u8], min_value: u64, snapshot_path: &str) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        let mut header = [0u8; 8];
        cursor.read_exact(&mut header)?;
        let num_scripts = u32::from_le_bytes(header[..4].try_into().unwrap());
        let total_utxos_header = u32::from_le_bytes(header[4..8].try_into().unwrap());

        let mut index = Self::new();
        index.script_entries.reserve(num_scripts as usize);

        let remaining = data.len() - 8;
        index.all_data.reserve(remaining / 2);
        index.utxo_data.reserve(remaining / 2);

        let mut actual_utxos = 0usize;

        for _ in 0..num_scripts {
            let mut slen_buf = [0u8; 2];
            cursor.read_exact(&mut slen_buf)?;
            let script_len = u16::from_le_bytes(slen_buf) as usize;

            let s_offset = index.all_data.len() as u32;
            {
                let data_len = index.all_data.len();
                index.all_data.resize(data_len + script_len, 0);
                cursor.read_exact(&mut index.all_data[data_len..])?;
            }

            let mut count_buf = [0u8; 4];
            cursor.read_exact(&mut count_buf)?;
            let utxo_count = u32::from_le_bytes(count_buf);

            let u_offset = index.utxo_data.len() as u32;
            let utxo_bytes = utxo_count as usize * UtxoEntry::SIZE;
            {
                let utxo_len = index.utxo_data.len();
                index.utxo_data.resize(utxo_len + utxo_bytes, 0);
                cursor.read_exact(&mut index.utxo_data[utxo_len..])?;
            }

            // Apply dust filter: compact UTXOs >= min_value in place
            let filtered_count = if min_value > 0 {
                let u_start = u_offset as usize;
                let mut write_pos = u_start;
                let mut kept = 0u32;
                for i in 0..utxo_count {
                    let read_pos = u_start + (i as usize) * UtxoEntry::SIZE;
                    let value = u64::from_le_bytes(
                        index.utxo_data[read_pos + 36..read_pos + 44]
                            .try_into()
                            .unwrap(),
                    );
                    if value >= min_value {
                        if write_pos != read_pos {
                            for byte in 0..UtxoEntry::SIZE {
                                index.utxo_data[write_pos + byte] = index.utxo_data[read_pos + byte];
                            }
                        }
                        write_pos += UtxoEntry::SIZE;
                        kept += 1;
                    }
                }
                actual_utxos += kept as usize;
                kept
            } else {
                actual_utxos += utxo_count as usize;
                utxo_count
            };

            index.script_entries.push(ScriptEntry {
                script_offset: s_offset,
                script_len: script_len as u16,
                utxo_offset: u_offset,
                utxo_count: filtered_count,
            });
        }

        index.num_scripts = num_scripts as usize;
        index.total_utxos = actual_utxos;

        if min_value > 0 {
            println!(
                "  Dust filter (>= {} sats): {} -> {} UTXOs",
                min_value, total_utxos_header, actual_utxos
            );
        }

        println!(
            "  Loaded {} scripts, {} UTXOs from {}",
            num_scripts, actual_utxos, snapshot_path
        );
        Ok(index)
    }

    /// Parse legacy v1 format into HashMap, then convert to FlatIndex.
    fn parse_legacy(data: &[u8], min_value: u64, snapshot_path: &str) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let mut header = [0u8; 4];
        cursor.read_exact(&mut header)?;
        let num_scripts = u32::from_le_bytes(header);

        let mut map = std::collections::HashMap::new();

        for _ in 0..num_scripts {
            let mut slen_buf = [0u8; 2];
            cursor.read_exact(&mut slen_buf)?;
            let script_len = u16::from_le_bytes(slen_buf) as usize;

            let mut script = vec![0u8; script_len];
            cursor.read_exact(&mut script)?;

            let mut count_buf = [0u8; 4];
            cursor.read_exact(&mut count_buf)?;
            let count = u32::from_le_bytes(count_buf);

            let mut entries = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let mut txid = [0u8; 32];
                cursor.read_exact(&mut txid)?;

                let mut vout_buf = [0u8; 4];
                cursor.read_exact(&mut vout_buf)?;
                let vout = u32::from_le_bytes(vout_buf);

                let mut val_buf = [0u8; 8];
                cursor.read_exact(&mut val_buf)?;
                let value = u64::from_le_bytes(val_buf);

                entries.push((bitcoin::Txid::from_byte_array(txid), vout, value));
            }

            if !entries.is_empty() {
                map.insert(script, entries);
            }
        }

        println!("  Converted legacy format: {} scripts", map.len());
        let index = Self::from_hashmap(&map, min_value);
        println!(
            "  FlatIndex from {}: {} scripts, {} UTXOs",
            snapshot_path, index.num_scripts, index.total_utxos
        );
        Ok(index)
    }
}

/// Export a script_index HashMap to a v2 flat snapshot file (zstd compressed, sorted).
pub fn export_flat_snapshot(
    script_index: &std::collections::HashMap<Vec<u8>, Vec<([u8; 32], u32, u64)>>,
    snapshot_path: &str,
) -> Result<()> {
    // Build sorted entries
    let mut entries: Vec<(Vec<u8>, Vec<([u8; 32], u32, u64)>)> = script_index
        .iter()
        .filter_map(|(script, utxos)| {
            if utxos.is_empty() {
                None
            } else {
                Some((script.clone(), utxos.clone()))
            }
        })
        .collect();

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let num_scripts = entries.len() as u32;
    let total_utxos: u32 = entries.iter().map(|e| e.1.len() as u32).sum();

    // Write raw data: [num_scripts:4][total_utxos:4][script_len:2][script][utxo_count:4][utxos...]
    let mut raw_data = Vec::new();
    raw_data.extend_from_slice(&num_scripts.to_le_bytes());
    raw_data.extend_from_slice(&total_utxos.to_le_bytes());

    for (script, utxos) in &entries {
        raw_data.extend_from_slice(&(script.len() as u16).to_le_bytes());
        raw_data.extend_from_slice(script);
        raw_data.extend_from_slice(&(utxos.len() as u32).to_le_bytes());
        for (txid, vout, value) in utxos {
            raw_data.extend_from_slice(txid);
            raw_data.extend_from_slice(&vout.to_le_bytes());
            raw_data.extend_from_slice(&value.to_le_bytes());
        }
    }

    let raw_size_mb = raw_data.len() as f64 / 1_048_576.0;

    // Compress with zstd
    let compressed = zstd::encode_all(Cursor::new(&raw_data), 3)?;
    let compressed_size_mb = compressed.len() as f64 / 1_048_576.0;

    // Write with v2 header: [magic:4][version:1][compressed_size:8][raw_size:8][zstd_data...]
    let mut f = std::io::BufWriter::new(std::fs::File::create(snapshot_path)?);
    f.write_all(b"BTCS")?;
    f.write_all(&2u8.to_le_bytes())?; // version 2 = flat format
    f.write_all(&(compressed.len() as u64).to_le_bytes())?;
    f.write_all(&(raw_data.len() as u64).to_le_bytes())?;
    f.write_all(&compressed)?;
    f.flush()?;

    eprintln!(
        "    Snapshot v2: {:.1} MB -> {:.1} MB ({:.2}:1, {}% size) [{} scripts, {} UTXOs]",
        raw_size_mb, compressed_size_mb,
        raw_size_mb / compressed_size_mb,
        (compressed_size_mb / raw_size_mb * 100.0).floor() as u64,
        num_scripts, total_utxos
    );

    Ok(())
}
