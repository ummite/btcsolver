# BTCSolver — Status & Architecture

## État actuel (2026-07-13)

### Index UTXO — En construction
- **Indexer v3** : fichier 260/3790, 72k txs/sec
- **ETA** : ~193 min (~3h13)
- **Checkpoint** : tous les 200 fichiers
- **UUTXO en mémoire** : 28.6M
- **Blockchain** : 804,897 blocs (août 2023, ~78K blocs de retard)

### Composants compilés
| Binaire | Taille | Statut |
|---------|--------|--------|
| `brute_force.exe` | 2668 KB | ✅ Prêt |
| `cache_manager.exe` | 847 KB | ✅ Prêt |
| `full_utxo_indexer.exe` | 3038 KB | ✅ En cours (v3) |
| `query_balance.exe` | — | ✅ Existant |
| `scan_blocks.exe` | — | ✅ Existant |

### Scripts disponibles
| Script | Usage |
|--------|-------|
| `sync-cache.bat init` | Copier index SAN → disque local |
| `sync-cache.bat sync` | Mettre à jour cache locale |
| `sync-cache.bat status` | État du cache |
| `brute-force-local.bat` | Brute-force avec cache locale |
| `brute-force-random.bat` | Brute-force depuis SAN |
| `build-index.bat` | Construire index UTXO |
| `update-index.bat` | Update incrémental |
| `instant-balance.bat "cle"` | Solde instantané |

### Disques détectés
| Disque | Type | Espace libre |
|--------|------|---|
| C: | Local | 256 GB |
| F: | Local | **7699 GB** ← sélectionné pour cache |
| I: | Local | 2994 GB |
| S: | Local | 3299 GB |
| Y: | SAN/RAID | 20666 GB |

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  SAN (Y:) - Source de vérité                                     │
│                                                                  │
│  Y:\Bitcoin\blocks\       ← Blockchain 472 Go (blk*.dat)         │
│  Y:\btcsolver\            ← Projet + index                       │
│    ├── utxo-index.redb    ← Index UTXO complet (en construction) │
│    └── *.bat              ← Scripts                              │
└──────────────────────────────────────────────────────────────────┘
                            ↕ sync-cache.bat
┌──────────────────────────────────────────────────────────────────┐
│  PC #1 - Cache locale (F:\btcsolver-cache\)                      │
│                                                                  │
│  F:\btcsolver-cache\                                              │
│    ├── utxo-index.redb      ← Copie locale (rapide)              │
│    └── cache-meta.json      ← Métadonnées sync                   │
│                                                                  │
│  brute-force-local.bat → charge depuis F: (zéro I/O réseau)      │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│  PC #2 - Cache locale (disque auto-détecté)                      │
│  Même structure, sync indépendante                               │
└──────────────────────────────────────────────────────────────────┘
```

---

## Flux de travail

### 1. Construction de l'index (une seule fois)
```
build-index.bat  →  full_utxo_indexer.exe build
                     ↓
                 utxo-index.redb (SAN)
```

### 2. Setup d'un nouveau PC
```
sync-cache.bat init
  ↓
Auto-détection disque local (F: 7699GB)
  ↓
Copie Y:\btcsolver\utxo-index.redb → F:\btcsolver-cache\
  ↓
~1-2 minutes pour 4GB
```

### 3. Utilisation quotidienne
```
sync-cache.bat sync    → Vérifie si SAN a des updates
brute-force-local.bat  → Charge depuis F: (instantané)
```

### 4. Update du blockchain
```
update-index.bat  → Parcours des nouveaux blocs seulement (~1-2 min/jour)
sync-cache.bat sync → Propage aux PCs locaux
```

---

## Résultats connus

| Test | Résultat | Date |
|------|----------|------|
| 128 phrases BIP39 (zoo zone...) | **0 BTC** / 512 adresses | 2026-07-13 |
| Clé aléatoire a1b2c3d4... | **0 BTC** / 4 adresses | 2026-07-13 |

---

## Optimisations implémentées

1. **retain() O(n) → HashSet O(1)** : éliminé le bottleneck de filtrage par dépense
2. **script_index incrémental** : maintenu en mémoire, filtré uniquement au checkpoint
3. **Checkpoint tous les 200 fichiers** : équilibre entre sécurité et performance
4. **Dé-obfuscation XOR chunked** : utilisation de chunks_mut() au lieu d'enumerate()
5. **Cache locale multi-PC** : auto-détection disque + sync incrémentale

---

## Prochaines étapes

- [ ] Index UTXO complet (ETA ~3h)
- [ ] Tester sync-cache.bat avec index complet
- [ ] Tester brute-force-local.bat
- [ ] GPU CUDA (planifié, non commencé)
