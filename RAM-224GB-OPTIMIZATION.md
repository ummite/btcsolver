# Optimisation BTCSolver pour 224 Go de RAM

## Architecture actuelle

```
Pipeline par thread (16 threads en parallèle) :
┌─────────────┐    ┌──────────────┐    ┌──────────────────┐    ┌────────────┐
│ Générer clé │───▶│ GPU: pubkey  │───▶│ CPU: SHA256+RIPE │───▶│ FlatIndex  │
│ 32 octets   │    │ 33 octets    │    │ MD160 → adresse  │    │ lookup     │
└─────────────┘    └──────────────┘    └──────────────────┘    └────────────┘
     CPU                 GPU                  CPU                    CPU
```

## Ce qui change avec 224 Go de RAM

### FlatIndex : ~3.9 Go utilisé vs 224 Go disponible

Le FlatIndex occupe seulement **3.9 Go** pour 28.7M scripts / 65.6M UTXOs.
Les **220 Go restants** permettent :

| Optimisation | Impact | RAM supplémentaire |
|---|---|---|
| **Batch size ×4** (256k → 1M) | Moins d'appels GPU/CPU par clé | ~33 Mo par thread |
| **Threads ×4** (16 → 64) | Saturation totale du CPU | ~1 Go |
| **Redb en RAM directe** | Lecture DB sans snapshot | ~30-40 Go |
| **Cache adresse→script** | Éviter SHA256+RIPEMD160 inverse | ~10-20 Go |
| **Multi-instance** | 4 processus en parallèle | ~16 Go chacun |

## Configuration recommandée

### Option 1 : Max performance (recommandé)

```batch
:: build-gpu.bat  (compile pour les RTX 5090)
:: Puis :

cd target\release

brute_force.exe ^
  --db-path ..\..\utxo-index.redb ^
  --threads 32 ^
  --random ^
  --use-gpu ^
  --batch-size 512000 ^
  --stop-on-match ^
  --output-file found-keys.json ^
  --stats-interval 10
```

**Pourquoi ces valeurs :**
- `--threads 32` : un thread par petit noyau CPU (PNW) sur les CPU récents
- `--batch-size 512000` : réduit l'overhead CPU/GPU par 2 (256k → 512k)
- `--use-gpu` : distribue automatiquement sur tous les GPU détectés

### Option 2 : Multi-instance (si 2+ GPU)

```batch
:: Instance 1 - GPU 0
start brute_force.exe --db-path ..\..\utxo-index.redb --threads 16 --random --use-gpu --batch-size 512000 --gpus 0

:: Instance 2 - GPU 1
start brute_force.exe --db-path ..\..\utxo-index.redb --threads 16 --random --use-gpu --batch-size 512000 --gpus 1
```

### Option 3 : Redb direct (RAM quasi illimitée)

Avec 224 Go, on peut charger le fichier redb directement sans snapshot :
- Le snapshot actuel fait 3 Go (compressé) → 3.6 Go (décompressé)
- Le redb fait ~30-40 Go mais avec 224 Go de RAM, c'est trivial
- Gain : données toujours à jour, pas de délai de snapshot

## Estimation de performance

| Configuration | Clés/sec | Facteur vs GTX 1080 |
|---|---|---|
| GTX 1080 actuel | ~260M | 1× |
| 1× RTX 5090 | ~2-3B | ~10× |
| 2× RTX 5090 | ~4-6B | ~20× |
| 2× RTX 5090 + 64 threads | ~6-8B | ~25× |

## Ce que 224 Go de RAM permet de plus

### 1. Index complet en RAM
Le FlatIndex binaire peut être mappé entièrement en mémoire avec `mmap`-like,
éliminant toute E/S disque pendant la recherche.

### 2. Cache de hachage inversé
Pré-calculer un HashMap `adresse → [scripts]` en RAM :
- Construction : ~30 min une seule fois
- Lookup : O(1) au lieu de O(log n) binary search
- RAM : ~15-20 Go pour 65M entrées

### 3. Multiple processus sans swap
Avec 32 Go de RAM, chaque processus brute_force utilise ~4 Go.
224 Go = **6 instances simultanées** sans aucun swapping.

## Script de lancement optimisé

Voir `run-max-performance.bat` pour la config auto-détectée.
