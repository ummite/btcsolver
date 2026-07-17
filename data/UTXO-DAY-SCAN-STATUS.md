# UTXO du jour + scan clés oubliées — statut

## UTXO du jour (bloc 935000)
- Source: https://files-vps02.jaonoctus.dev/utxo-935000.dat (8.74 GB)
- Bloc: 0000000000000000000147034958af1652b2b91bba607beacc5e72a56f0fb5ee
- Index redb (btcsolver balance): W:\Temp\btc-index-935000.redb (~4 GB)
- FlatIndex snapshot (scans): W:\Temp\utxo-day-935000.snapshot (3.3 GB, 57.9M scripts)
- Copie projet: Y:\btcsolver\data\

## Hardware utilisé
- CPU: Ultra 9 285K (24 threads)
- GPU: 2x RTX 5090 + 1x RTX 3090 (derive CUDA; index GPU load failed → fallback derive-only)
- RAM: 224 GB (FlatIndex ~4.8 GB en RAM)

## Scans lancés
1. brainwallet_scan — 442k phrases → 14.8M variations, ~594k/s, 0 match sur index full day
2. brute_force sequential 1..100M — ~400k keys/s (legacy+segwit), 0 match
3. Suite sequential + brainwallet_extended en cours

## Astuces perf
- Index FlatIndex local W: (pas NAS)
- Un seul process GPU multi-device (évite contention VRAM)
- addr-types legacy,segwit seulement = ~2x plus rapide (drop wrapped/taproot si besoin)
- batch 256k, 24 threads
- Brainwallets / clés faibles >> random 2^256 (inutile)
- dump_to_flat: nouveau binaire pour dumptxoutset → .snapshot

## Validation
- derive-only key=1 OK
- Index full 164M UTXOs / 58M scripts / ~19.98M BTC supply

## Note
"correct horse battery staple" a matché sur un ANCIEN snapshot partiel (0.01 BTC) —
sur l'index full day 935000: 0 (fonds probablement dépensés, ou faux positif de l'ancien index).
