# BTCSolver — Guide d'installation sur un 2e PC

> **Objectif :** Analyser des clés privées Bitcoin depuis un 2e PC en utilisant
> la même blockchain et les mêmes outils que le PC principal.
>
> **Temps total :** ~5 minutes

---

## Prérequis

- **Windows** (x64)
- **Accès au lecteur réseau `Y:`** (RAID DS1821 avec la blockchain)
- **Connexion Internet** (pour installer Rust, une seule fois)

---

## Étape 1 — Mapper le lecteur réseau Y: (1 min)

Le RAID DS1821 doit être accessible depuis ce PC.

### Si Y: est déjà disponible
```
Rien à faire. Passez à l'étape 2.
```

### Si Y: n'existe pas
1. Ouvrer l'Explorateur de fichiers
2. Clic droit sur "Ce PC" → "Mapper un lecteur réseau"
3. Lecteur : `Y:`
4. Dossier : `\\<adresse-du-raid>\`  (demander au propriétaire du réseau)
5. Cocher "Reconnecter à la connexion"
6. Valider

**Vérification :** `Y:\btcsolver\` doit exister et contenir ce fichier.

---

## Étape 2 — Installer Rust (2 min)

1. Télécharger : https://win.rustup.rs/
2. Exécuter `rustup-init.exe`
3. Accepter les paramètres par défaut (Option 1)
4. Fermer et rouvrir PowerShell

**Vérification :**
```powershell
cargo --version
# Doit afficher quelque chose comme "cargo 1.xx.x"
```

---

## Étape 3 — Compiler les outils (30-60 secondes)

```powershell
cd Y:\btcsolver
cargo build --release
```

> **Pourquoi ?** Les `.exe` compilés sur un PC peuvent ne pas fonctionner
> sur un autre PC si les CPU ont des instructions différentes (AVX2, SSE, etc.).
> Cette étape compile les outils pour **le CPU de ce PC**.

---

## Étape 4 — Vérifier que tout fonctionne

```powershell
cd Y:\btcsolver

# Vérifier les outils
.\full_utxo_indexer.exe --help
.\query_balance.exe --help
.\scan_blocks.exe

# Vérifier l'accès à la blockchain
Test-Path "Y:\Bitcoin\blocks\blk00000.dat"
# Doit retourner True
```

---

## Utilisation

### Vérifier le solde d'une clé privée

#### Option A : Scan direct (~2 heures)
```powershell
.\scan-key.bat "votre-cle-privee"
```

Formats acceptés :
- **WIF** : `5HueCGU...`
- **Hex** : `a1b2c3d4e5f6...` (64 caractères)
- **BIP39** : `"mot1 mot2 ... mot12"` (12+ mots)

#### Option B : Index UTXO (< 1 seconde)

1. **Construire l'index** (à faire une seule fois, ~3-5 heures) :
   ```powershell
   .\build-index.bat
   ```

2. **Requêter n'importe quelle clé** (< 1 seconde) :
   ```powershell
   .\instant-balance.bat "votre-cle-privee"
   ```

3. **Mettre à jour l'index** (quand de nouveaux blocs arrivent, ~1-2 min) :
   ```powershell
   .\update-index.bat
   ```

---

## Tous les scripts disponibles

| Script | Usage | Temps |
|--------|-------|-------|
| `scan-key.bat "cle"` | Scanner une clé spécifique | ~2h |
| `scan-all.bat` | Scanner les 128 phrases BIP39 | ~2h |
| `find-12th-word.bat mot1 ... mot11` | Trouver le 12e mot BIP39 | < 1s |
| `build-index.bat` | Construire l'index UTXO complet | ~3-5h (1ère fois) |
| `update-index.bat` | Mettre à jour l'index | ~1-2 min |
| `instant-balance.bat "cle"` | Solde instantané depuis l'index | < 1s |
| `build-cache.bat "cle1" "cle2"` | Construire une cache pour N clés | ~2h |
| `check-cache.bat "cle"` | Vérifier solde depuis une cache | < 1s |
| `index-stats.bat` | Statistiques de l'index | < 1s |

---

## Architecture du projet

```
Y:\btcsolver\
├── src/                          # Code source Rust
│   ├── main.rs                   # BTCSolver principal (RPC + BIP39)
│   └── bin/
│       ├── scan_blocks.rs        # Scanner les 128 phrases BIP39
│       ├── query_balance.rs      # Scanner une clé spécifique
│       ├── full_utxo_indexer.rs  # Index UTXO complet
│       ├── fix_mnemonic.rs       # Trouver le 12e mot BIP39
│       └── debug_blocks.rs       # Outil de debug
│
├── *.bat                         # Scripts d'utilisation
├── *.exe                         # Binaires compilés
│
├── utxo-index.redb               # Index UTXO (généré par build-index.bat)
├── utxo-cache.bin                # Cache UTXO (généré par build-cache.bat)
│
├── bip39-words.txt               # Dictionnaire BIP39 (2048 mots)
└── valid-phrases.txt             # 128 phrases valides trouvées
```

---

## Comment ça marche

### Scanner une clé privée

```
Clé privée (WIF/Hex/BIP39)
    │
    ▼
Dérivation de 4 adresses Bitcoin
    │  Legacy   (1A1zP1eP5...)
    │  SegWit   (bc1qar0s...)
    │  Wrapped  (3J98t1W...)
    │  Taproot  (bc1p...)
    │
    ▼
Lecture des fichiers blockchain Y:\Bitcoin\blocks\blk*.dat
    │
    ├─ Déobfuscation XOR (clé: b3a2cd522df3a049)
    ├─ Parsing des blocs (magic 0xd9b4bef9)
    ├─ Extraction des 884 millions de transactions
    │
    └─ Suivi des UTXO :
        ├─ Crédit  → sortie vers notre adresse → +solde
        └─ Dépense → input qui dépense notre UTXO → -solde
    │
    ▼
Solde final (correct jusqu'au bloc 804 897, août 2023)
```

### Index UTXO (requête instantanée)

```
build-index.bat (1ère fois, ~3-5h)
    │
    ▼
Scan de TOUS les blocs → Index UTXO COMPLET
    │
    ├─ Table "utxos"     : outpoint → (script, valeur)
    └─ Table "by_script" : script → liste de (txid, vout, valeur)
    │
    ▼
Sauvegarde dans utxo-index.redb (~2-3 Go)
    │
    │
instant-balance.bat "cle" (< 1 seconde)
    │
    ▼
Clé → 4 adresses → 4 scripts → Lookup dans "by_script"
    │
    ▼
Solde instantané
```

---

## Clé d'obfuscation

La clé `b3a2cd522df3a049` est **spécifique à cette installation** de
Bitcoin Core sur le RAID DS1821. Elle est utilisée pour déobfusquer
les fichiers `blk*.dat` (chiffrement XOR).

**Si les fichiers blockchain sont copiés depuis ce RAID** → cette clé
fonctionne partout.

**Si un autre PC synchronise sa propre blockchain** → il aura sa propre
clé. La trouver dans le fichier `database/obfuscate` du dossier Bitcoin
de ce PC, ou dans les logs de `bitcoind`.

---

## Limitations

- **Blockchain incomplète** : s'arrête au bloc ~804 971 (août 2023)
  Les soldes sont corrects jusqu'à cette date. ~78 000 blocs de retard
  sur le réseau actuel.

- **Bitcoin Core incompatible avec le RAID** : le RAID DS1821 est trop
  lent pour les I/O aléatoires de LevelDB. Les outils BTCSolver utilisent
  un I/O séquentiel et fonctionnent correctement.

---

## Dépannage

### "cargo : terme non reconnu"
→ Rust n'est pas installé. Retournez à l'étape 2.

### "Test-Path retourne False pour blk00000.dat"
→ Le lecteur Y: n'est pas mappé correctement. Retournez à l'étape 1.

### "Cannot parse key"
→ Le format de la clé n'est pas reconnu. Utilisez WIF, hex (64 caractères),
   ou une phrase BIP39 de 12+ mots.

### L'.exe plante au démarrage
→ Les binaires sont compilés pour un autre CPU. Refaire `cargo build --release`.

### "Database does not exist" pour instant-balance.bat
→ L'index n'a pas été construit. Lancer `build-index.bat` d'abord.

---

## Résultats connus

| Test | Résultat | Date |
|------|----------|------|
| 128 phrases BIP39 (`zoo zone zoo zone zoo zone zoo zone zoo zone zoo` + 12e mot) | **0 BTC** sur 512 adresses | 2026-07-13 |
| Clé aléatoire `a1b2c3d4...` | **0 BTC** sur 4 adresses (en cours de vérification) | 2026-07-13 |

---

## Contact

Projet local. Aucune dépendance externe. Tout fonctionne offline.
