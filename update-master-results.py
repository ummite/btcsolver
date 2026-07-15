#!/usr/bin/env python3
"""Merge all brainwallet scan results into master results file."""
import json
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path(r"Y:\btcsolver")
MASTER = ROOT / "brainwallet-master-results.json"

result_files = [
    ("wave2-v2", ROOT / "brainwallet-all-v2-results.json"),
    ("wave3", ROOT / "brainwallet-wave3-results.json"),
    ("wave4", ROOT / "brainwallet-wave4-results.json"),
    ("wave5", ROOT / "brainwallet-wave5-results.json"),
]

all_matches = []
seen = set()  # (phrase, address)
scans = []

for scan_id, path in result_files:
    if not path.exists():
        continue
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, list):
        continue
    n = 0
    for m in data:
        key = (m.get("phrase", ""), m.get("address", ""))
        if key in seen:
            continue
        seen.add(key)
        entry = dict(m)
        entry["scan_id"] = scan_id
        all_matches.append(entry)
        n += 1
    scans.append({"id": scan_id, "file": path.name, "unique_matches_added": n, "raw_count": len(data)})

all_matches.sort(key=lambda x: -x.get("value_sats", 0))
total_sats = sum(m.get("value_sats", 0) for m in all_matches)

master = {
    "updated_at": datetime.now(timezone.utc).isoformat(),
    "binary": "brainwallet_extended (UTXO offset fixed)",
    "snapshot": "utxo-index.snapshot",
    "scans": scans,
    "match_count": len(all_matches),
    "total_sats": total_sats,
    "total_btc": total_sats / 1e8,
    "all_matches": all_matches,
}

MASTER.write_text(json.dumps(master, indent=2, ensure_ascii=False), encoding="utf-8")
print(f"Master updated: {len(all_matches)} unique matches, {total_sats} sats ({total_sats/1e8:.8f} BTC)")
print("\nTop by value:")
for m in all_matches[:20]:
    print(f"  {m.get('value_btc',0):12.8f} BTC  {m.get('phrase')!r:20}  {m.get('address')}")
