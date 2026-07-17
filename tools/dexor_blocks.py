#!/usr/bin/env python3
"""
De-XOR Bitcoin Core block files that are still obfuscated.
Key: 8-byte XOR (same as Core's blocks/xor.dat when non-zero).

Only rewrites files whose first 4 bytes are NOT mainnet magic f9beb4d9.
Safe to re-run (skips already plaintext).
"""
from __future__ import annotations

import argparse
import os
import sys
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

MAINNET_MAGIC = bytes([0xF9, 0xBE, 0xB4, 0xD9])
DEFAULT_KEY = bytes.fromhex("b3a2cd522df3a049")
BUF = 8 * 1024 * 1024  # 8 MiB


def needs_dexor(path: Path, key: bytes) -> bool:
    try:
        with path.open("rb") as f:
            head = f.read(4)
        if len(head) < 4:
            return False
        if head == MAINNET_MAGIC:
            return False
        # If XOR with key yields magic, it's obfuscated with this key
        plain = bytes(a ^ b for a, b in zip(head, key[:4]))
        return plain == MAINNET_MAGIC
    except OSError:
        return False


def dexor_file(path_str: str, key_hex: str) -> tuple[str, str, int]:
    """Returns (path, status, bytes_processed)."""
    path = Path(path_str)
    key = bytes.fromhex(key_hex)
    if len(key) != 8:
        return path_str, "bad-key", 0
    try:
        size = path.stat().st_size
        with path.open("rb") as f:
            head = f.read(4)
        if head == MAINNET_MAGIC:
            return path_str, "skip-plain", 0
        plain_head = bytes(a ^ b for a, b in zip(head, key[:4]))
        if plain_head != MAINNET_MAGIC:
            return path_str, "skip-unknown-magic", 0

        # In-place XOR via temp file then replace
        tmp = path.with_suffix(path.suffix + ".dexor_tmp")
        processed = 0
        with path.open("rb") as src, tmp.open("wb") as dst:
            ki = 0
            while True:
                chunk = src.read(BUF)
                if not chunk:
                    break
                out = bytearray(len(chunk))
                for i, b in enumerate(chunk):
                    out[i] = b ^ key[ki]
                    ki = (ki + 1) & 7
                dst.write(out)
                processed += len(chunk)
        os.replace(tmp, path)
        return path_str, "ok", processed
    except Exception as e:
        try:
            tmp = path.with_suffix(path.suffix + ".dexor_tmp")
            if tmp.exists():
                tmp.unlink()
        except OSError:
            pass
        return path_str, f"err:{e}", 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--blocks-dir", default=r"W:\Bitcoin\blocks")
    ap.add_argument("--key", default=DEFAULT_KEY.hex())
    ap.add_argument("--workers", type=int, default=max(2, (os.cpu_count() or 4) // 2))
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    bdir = Path(args.blocks_dir)
    if not bdir.is_dir():
        print(f"ERROR: blocks dir missing: {bdir}", file=sys.stderr)
        return 1

    key = bytes.fromhex(args.key)
    patterns = ["blk*.dat", "rev*.dat"]
    candidates: list[Path] = []
    for pat in patterns:
        candidates.extend(sorted(bdir.glob(pat)))

    print(f"Scanning {len(candidates)} files in {bdir} ...", flush=True)
    todo = [p for p in candidates if needs_dexor(p, key)]
    print(f"Need de-XOR: {len(todo)} / {len(candidates)}", flush=True)
    if args.dry_run:
        for p in todo[:20]:
            print(f"  would fix {p.name}")
        if len(todo) > 20:
            print(f"  ... +{len(todo)-20} more")
        return 0

    if not todo:
        print("Nothing to do.")
        return 0

    t0 = time.time()
    ok = err = skip = 0
    total_bytes = 0
    workers = max(1, min(args.workers, len(todo)))
    print(f"Workers={workers} key={args.key}", flush=True)

    with ProcessPoolExecutor(max_workers=workers) as ex:
        futs = {ex.submit(dexor_file, str(p), args.key): p for p in todo}
        done = 0
        for fut in as_completed(futs):
            done += 1
            path, status, n = fut.result()
            total_bytes += n
            if status == "ok":
                ok += 1
            elif status.startswith("err"):
                err += 1
                print(f"  FAIL {Path(path).name}: {status}", flush=True)
            else:
                skip += 1
            if done % 25 == 0 or done == len(todo):
                elapsed = max(0.001, time.time() - t0)
                mb = total_bytes / (1024 * 1024)
                print(
                    f"  [{done}/{len(todo)}] ok={ok} err={err} "
                    f"{mb:.0f} MiB in {elapsed:.0f}s ({mb/elapsed:.1f} MiB/s)",
                    flush=True,
                )

    print(f"DONE ok={ok} err={err} skip={skip} seconds={time.time()-t0:.1f}", flush=True)
    return 0 if err == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
