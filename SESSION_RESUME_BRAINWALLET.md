# Session Resume — Brainwallet / UTXO (2026-07-15)

## ⚠️ CRITIQUE — À LIRE EN PREMIER

**On N’A PAS la blockchain à jour. Les scans brainwallet actuels ne sont PAS fiables pour des soldes dépensables.**

| Élément | État |
|--------|------|
| Tip mainnet | ~**958 196** |
| bitcoind local | **Éteint** (RPC `127.0.0.1:8332` inaccessible) |
| Blocks locaux `Y:\Bitcoin\blocks` | 3790 `blk*.dat`, **~472 GB** (chaîne complète ~754 GB → **~60–65 %**) |
| Dernier blk récent | ~**13 juil. 2026** |
| `utxo-index.snapshot` **actuel** | **PARTIEL / CASSÉ** — rebuild stoppé au checkpoint file **50/3790** ≈ bloc **111 191** (début 2011) |
| Scripts dans snapshot actuel | **~10,9 M** (vagues 4–5) |
| Ancien index (vague 3) | **~28,6 M** scripts (format legacy, plus complet mais pas tip) |
| Backup | `utxo-index.snapshot.bak` **2,89 GB** (14 juil.) — mieux que l’actuel, **pas la tip** |

### Conséquence
- Matchs trouvés = surtout **UTXO historiques / dust / déjà vidés** (`final_balance = 0` on-chain pour les gros).
- **Ne pas conclure “fonds récupérables”** tant que l’index n’est pas rebuild jusqu’à la tip.
- **Garder les phrases candidates** (utile) ; **ignorer les montants du snapshot actuel**.

### Avant de re-scanner (ordre obligatoire)
1. Relancer Bitcoin Core / bitcoind  
2. Sync jusqu’à tip (~958k) + tous les `blk*.dat`  
3. Rebuild complet UTXO :
   ```bat
   update-utxo-full.bat
   ```
   ou `full_utxo_indexer` sur **tous** les fichiers (pas s’arrêter à checkpoint 50)  
4. Vérifier snapshot : scripts ~28M+, hauteur ≈ tip  
5. Relancer scans brainwallet  

Option temporaire : restaurer `utxo-index.snapshot.bak` → meilleur que le partiel, **pas suffisant**.

---

## Fichiers importants

| Fichier | Rôle |
|---------|------|
| `brainwallet-master-list.md` | Liste maîtresse ~100 catégories d’idées de clés |
| `brainwallet-master-results.json` | Agrégat matches (phrases + adresses) |
| `brainwallet-wave3-corpus.txt` | ~757K patterns |
| `brainwallet-wave4-corpus.txt` | ~64K patterns |
| `brainwallet-wave5-corpus.txt` | ~94K patterns |
| `brainwallet-all-corpus-v2.txt` | Wave 1+2 ~71K |
| `brainwallet-wave{3,4,5}-results.json` | Résultats par vague |
| `update-master-results.py` | Merge des `*-results.json` → master |
| `generate-brainwallet-wave{3,4,5}.py` | Générateurs de corpus |
| `target/release/brainwallet_extended.exe` | Scanner (SHA256+MD5, comp+uncomp, 4 addr types) — **bug offset UTXO corrigé** |
| `target/release/timestamp_scan.exe` | Timestamps (pas relancé sérieusement) |
| `utxo-index.snapshot` | **PARTIEL — ne pas faire confiance** |
| `utxo-index.snapshot.bak` | Backup plus gros (14 juil.) |
| `utxo-index-build.log` | Prouve rebuild stoppé à file 50 |
| `update-utxo-full.bat` | Rebuild index depuis `Y:\Bitcoin\blocks` |

---

## Bug FlatIndex (corrigé)

- **Problème** : lecture UTXO `chunk[32..40]` au lieu de `chunk[36..44]` → soldes ×2³² ou absurdes  
- **Fix** : `src/flat_index.rs` + `src/bin/flat_index.rs`  
- Toujours rebuild/re-scan avec binaire post-fix

---

## Scans déjà faits (phrases utiles ; soldes snapshot non fiables)

### Waves 1–2 (~71K)
- Bible EN/FR, passwords, BIP39, films, chansons, math, clavier, philo, URLs, langues, pop culture…

### Wave 3 (~757K) — ~6.5 min
- Chars, nombres 0–100k, dates 1900–2010, noms, color×animal, phones, dice, wallets, pays…

### Wave 4 (~64K)
- Paroles étendues, dialogues films, leet, adj+nom, 3-words, MD5-as-key, sports, emails…

### Wave 5 (~94K)
- Reversed, ROT13, base64, BIP39 pairs top-200, pi windows, cypherpunk, silk road, CHBS, whitepaper…

### Matches notables (adresses réelles historiquement ; live souvent 0)
- `correct horse battery staple` — uncomp `1JwSSubhmg6iPtRjtyqhUYYH7bZg3Lfy1T` + comp `1C7zdTfnkzmr13HfA2vNm5SJYRK6nEKyq8`  
- `1728` — `1EyTFVBL44aGuybFbXZdgueoSLKMJAorui`  
- `love`, `test1`, `TEST`, `dog`, `you`, `very`, `root`, `test wallet`  
- `password`, `god`, `market`, `test`, `computer`, `satoshinakamoto`, `to be or not to be`  
- `02011980`, `fff`, `NSA`, `hi`, `abc`, `abcdefg`, `1`, `""`, `Satoshi Nakamoto`  

~**29** matches uniques dans `brainwallet-master-results.json`  
→ **re-vérifier tous on-chain + re-scanner après index à jour**

---

## Backlog (après index à jour)

1. Rebuild UTXO complet + vérifier hauteur  
2. Re-scan waves 1–5 (ou corpus fusionné) avec index frais  
3. Vérifier chaque match sur mempool.space / blockchain.info (`final_balance`)  
4. Timestamps (`timestamp_scan`) — long  
5. Rockyou 100K si dispo  
6. BIP39 3-words top-N  
7. Encodages (UTF-16, newline trailing)  
8. Rotations de bits (×512, très cher)  

---

## Commandes utiles

```bat
REM Rebuild index (bitcoind sync + blocks complets d'abord)
update-utxo-full.bat

REM Scan corpus
target\release\brainwallet_extended.exe --texts brainwallet-wave5-corpus.txt --snapshot utxo-index.snapshot --threads 22 --hash both --output brainwallet-wave5-results.json

REM Merger résultats
python update-master-results.py
```

RPC bitcoind (si tourne) :
```
cookie: Y:\Bitcoin\.cookie
RPC: http://127.0.0.1:8332
```

---

## Décision utilisateur en fin de session

- Conserver en mémoire / reprendre plus tard  
- Priorité au retour : **sync blockchain + rebuild index**, pas plus de scans sur snapshot partiel  

*Dernière mise à jour : 2026-07-15 (session brainwallet waves 1–5 + audit fraîcheur UTXO)*
