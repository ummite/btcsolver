# AGENTS.md — Instructions pour agents (Grok / Claude / etc.)

Workspace : `Y:\btcsolver`  
Langue utilisateur : **français**  
Dernière MAJ : **2026-07-16**

---

## Mission du projet

**BTCSolver** = outil **local / privé** (Rust) pour :

1. Scanner des soldes Bitcoin à partir de clés (hex, WIF, BIP39, brainwallet) **sans envoyer les clés sur Internet**
2. Construire / utiliser un **index UTXO offline** (FlatIndex / redb) pour des lookups ultra-rapides
3. Brute-force / scans GPU (CUDA) + **dashboard web local pro**
4. S’appuyer sur un nœud **Bitcoin Core full** quand il faut des données fraîches

### Produit principal : Control Center

```powershell
Y:\btcsolver\Launch-Dashboard.ps1
# → http://localhost:3000
```

Onglets :

| Onglet | Rôle |
|--------|------|
| Bitcoin Core | start/stop/status datadir **W:\Bitcoin** |
| UTXO | rebuild snapshot depuis blocks + reload index RAM |
| Clés manuelles | Hex/WIF/BIP39 multi-chemins + brainwallet (SHA256, reverse, etc.) |
| Dictionnaire | corpus / phrases + transforms |
| Brute-force | scan hex GPU |
| Idées | pistes de recherche (voir aussi `KEY-HUNT-IDEAS.md`) |

Ce n’est **pas** un wallet hot exposé. Dashboard = **localhost only** (`127.0.0.1`).

---

## Bitcoin Core — SOURCE DE VÉRITÉ ACTUELLE = `W:\Bitcoin`

| Élément | Chemin |
|--------|--------|
| **Datadir (actif)** | `W:\Bitcoin` |
| **Disque** | `W:` local **SwordFish x2 3.6TB** (~3.6 TB, SSD/rapide) |
| **Binaires** | `W:\Bitcoin\bin\` et `W:\Bitcoin\bin\daemon\` |
| **bitcoind** | `W:\Bitcoin\bin\daemon\bitcoind.exe` (v31.1.0) |
| **bitcoin-cli** | `W:\Bitcoin\bin\daemon\bitcoin-cli.exe` |
| **GUI** | `W:\Bitcoin\bin\bitcoin-qt.exe` |
| **Config** | `W:\Bitcoin\bitcoin.conf` |
| **RPC** | `127.0.0.1:8332` — user `btcsolver` / pass `btcsolver_rpc_2026` (aussi cookie `.cookie`) |
| **Blocks** | `W:\Bitcoin\blocks\` — ~3790 `blk*.dat`, **plaintext** (`blocksxor=0`) |
| **NAS (legacy)** | `Y:\Bitcoin` — partage RAID DS1821 ; **ne plus lancer bitcoind ici par défaut** |

### Pourquoi W: et pas Y: ?

- `Y:` = NAS RAID lent pour LevelDB / reindex / flushes UTXO
- `W:` = disque local rapide, install **portable** complète (exe + data)
- Les blocs sur **W:** sont déjà **dé-XOR** (magic `f9beb4d9`). Config : `blocksxor=0`
- **Ne pas** recréer `blocks\xor.dat` tant que les blocs sont en clair (sinon Core refuse de démarrer avec `blocksxor=0`)

### Lancer Bitcoin Core (commande standard)

```powershell
# Depuis le repo
.\Launch-BitcoinCore.bat
# ou
powershell -ExecutionPolicy Bypass -File .\Launch-BitcoinCore.ps1
# ou direct
Start-Process "W:\Bitcoin\bin\daemon\bitcoind.exe" -ArgumentList "-datadir=W:\Bitcoin" -WindowStyle Hidden
```

Scripts natifs du datadir :

```bat
W:\Bitcoin\Launch-BitcoinCore.bat
W:\Bitcoin\Launch-BitcoinCore-GUI.bat
W:\Bitcoin\Stop-BitcoinCore.bat
W:\Bitcoin\bitcoin-cli.bat getblockchaininfo
```

### Vérifier le statut

```powershell
Get-Process bitcoind -ErrorAction SilentlyContinue
& "W:\Bitcoin\bin\daemon\bitcoin-cli.exe" -datadir=W:\Bitcoin getblockchaininfo
& "W:\Bitcoin\bin\daemon\bitcoin-cli.exe" -datadir=W:\Bitcoin getblockcount
Get-Content W:\Bitcoin\debug.log -Tail 30
```

### Arrêter proprement

```bat
W:\Bitcoin\Stop-BitcoinCore.bat
```

ou :

```powershell
& "W:\Bitcoin\bin\daemon\bitcoin-cli.exe" -datadir=W:\Bitcoin stop
```

### Règles critiques bitcoind

1. **Un seul bitcoind à la fois** sur un datadir (fichier `.lock`)
2. Ne **jamais** lancer en parallèle `Y:\Bitcoin` **et** `W:\Bitcoin` sur le même port 8332
3. Si crash / arrêt sale : supprimer `.lock` / `blocks\.lock` **seulement** si aucun process `bitcoind` n’existe
4. Sur Windows : **pas** de flag `-daemon` fiable → `Start-Process` / `start /B`
5. Si erreur  
   `The blocksdir XOR-key can not be disabled when a random key was already stored`  
   → blocs en clair + `xor.dat` orphelin : **renommer/supprimer** `W:\Bitcoin\blocks\xor.dat` (backup d’abord), garder `blocksxor=0`
6. Chainstate vide / reindex interrompu → au redémarrage Core reprend validation / IBD ; **patience** (centaines de Go)
7. Espace libre W: à surveiller (~chainstate + croissance blocks)

---

## Projet btcsolver (Y:\btcsolver)

### Rôles des composants

| Composant | Rôle |
|-----------|------|
| `btcsolver.exe` / `src/main.rs` | CLI solde / index / history via RPC ou index offline |
| `full_utxo_indexer*.exe` | Build index UTXO depuis `blk*.dat` |
| `brute_force*.exe` | Scan exhaustif / GPU keys |
| `btcsolver_dashboard.exe` | UI web Axum (port 3000) |
| `query_balance.exe` | Lookup solde sur index |
| `data/` | Index / snapshots intermédiaires |
| `utxo-index.redb` / `.snapshot` | Index UTXO (souvent partiel — **vérifier hauteur**) |
| `static/dashboard/` | Frontend dashboard |

### Binaires souvent copiés hors repo

- `C:\btcsolver-bin\` — binaires “stables” pour le dashboard
- `C:\btcsolver-cache\` — cache / snapshot local rapide

### Chemins blockchain pour les indexers

Priorité quand on indexe / scanne les blocks :

1. **`W:\Bitcoin\blocks`** (local, plaintext) — **préféré**
2. `Y:\Bitcoin\blocks` (NAS, encore XOR-obfusqué côté Y si non déchiffré)

### RPC depuis les outils

```powershell
# Cookie
.\btcsolver.exe balance --key <KEY> --cookie-file W:\Bitcoin\.cookie --sats

# User/pass (bitcoin.conf W:)
# rpcuser=btcsolver / rpcpassword=btcsolver_rpc_2026
```

### Dashboard

```powershell
cd Y:\btcsolver
# préférer binaire installé si présent
C:\btcsolver-bin\btcsolver_dashboard.exe --port 3000
# sinon
.\target\release\btcsolver_dashboard.exe --port 3000
```

Ouvrir : http://localhost:3000  
**Ne pas exposer sur Internet.**

### Always-On (Core + UTXO + Dashboard) — OBLIGATOIRE

```bat
Y:\btcsolver\Install-AlwaysOn.bat          # installe taches + demarre
Y:\btcsolver\Uninstall-AlwaysOn.bat        # desinstalle
Y:\btcsolver\START-BTC-SOLVER.bat          # manuel + verif HTTP
```

| Fichier / tache | Rôle |
|-----------------|------|
| `Keep-Core-And-Utxo.ps1` | **bitcoind toujours vivant** ; dès tip → `dumptxoutset` + FlatIndex auto |
| `Watch-BtcSolver.ps1` | Dashboard :3000 ; brute GPU **sauf** si PRIORITY-SYNC |
| Tache `BTCSolver-Core-Utxo` | Toutes les **3 min** |
| Tache `BTCSolver-Watchdog` | Toutes les **2 min** |
| Status | `data\CORE-UTXO-STATUS.json` |

### PRIORITÉ SYNC (Core tip + UTXO tip AVANT les clés)

```powershell
.\Enable-PrioritySync.ps1    # flag + stop brute + focus Core/UTXO
.\Disable-PrioritySync.ps1   # reautorise brute (après tip+UTXO frais)
```

- Flag : `data\PRIORITY-SYNC.flag` → **aucune chasse aux clés** (watchdog ne relance pas `brute_force`)
- Ordre obligatoire : **1) Core tip** → **2) UTXO auto &lt;24h** → **3) seulement ensuite** scans / clés
- **Ne jamais tuer bitcoind** pendant IBD

**Règle UTXO tests** : valable si retard tip **&lt; 24 h** (après auto-refresh au tip).  
**Tant que IBD** (`blocks` &lt; `headers`) : Core travaille normalement ; UTXO tip impossible localement.

Logs : `C:\btcsolver-cache\core-utxo.log`, `C:\btcsolver-cache\watchdog.log`  
UI : http://127.0.0.1:3000/

---

## Workflow agent — au démarrage d’une session ici

1. **Lire ce fichier** (`AGENTS.md`) + éventuellement `SESSION_RESUME*.md` / `STATUS.md` s’ils sont plus récents
2. **Vérifier bitcoind** sur `W:\Bitcoin` :
   - process vivant ?
   - `getblockchaininfo` → `blocks`, `headers`, `initialblockdownload`, `verificationprogress`
3. Si bitcoind est **arrêté** et que la session a besoin de RPC / sync : **le relancer** (section ci-dessus)
4. Ne **pas** relancer des tâches de monitoring planifiées (1 min / 15 min) sauf demande explicite
5. Avant de conclure qu’un solde brainwallet est “récupérable” : vérifier que l’**index UTXO est à jour** jusqu’à la tip (sinon soldes historiques / dust trompeurs — voir `SESSION_RESUME_BRAINWALLET.md`)
6. Modifications code : Rust via `cargo build --release` dans `Y:\btcsolver`
7. Commits / push / force : **demander confirmation** avant actions destructives ou visibles (push, etc.)

---

## État connu (2026-07-16 — PRIORITY-SYNC)

- **Ordre fixe** : Core tip → UTXO tip (&lt;24h) → **ensuite seulement** chasse aux clés
- **PRIORITY-SYNC actif** (`data\PRIORITY-SYNC.flag`) : `brute_force` **OFF** ; watchdog ne le relance pas
- **Always-On** : taches `BTCSolver-Core-Utxo` (3 min) + `BTCSolver-Watchdog` (2 min) + Startup Windows
- bitcoind **vivant** `W:\Bitcoin` — IBD (~433k+ / ~958k headers) ; **ne pas tuer**
- UTXO `utxo-day-935000` **stale** → auto `dumptxoutset`+`dump_to_flat` dès tip
- Status : `data\CORE-UTXO-STATUS.json` · logs `C:\btcsolver-cache\core-utxo.log`
- Runtime Bitcoin = **W:** (pas Y:)
- **Archive clés actives** : `data\keys-archive.json` (+ `.jsonl`) — toute clé avec **solde UTXO** ou **activité on-chain** (historique via `scanblocks` quand blockfilter prêt) ; pic de solde conservé même si dépensé plus tard. API `GET /api/keys/archive`

---

## Fichiers de session utiles

| Fichier | Contenu |
|---------|---------|
| `AGENTS.md` | **Ce fichier** — procédure agent (à jour) |
| `SESSION_RESUME.md` | Dashboard + chemins binaires |
| `SESSION_RESUME_BRAINWALLET.md` | État scans brainwallet + pièges index partiel |
| `STATUS.md` | Architecture cache multi-PC (peut être daté) |
| `SETUP-2E-PC.md` | Install 2e PC sur le NAS Y: |
| `W:\Bitcoin\README-PORTABLE.md` | Doc install portable Bitcoin |

Quand tu changes le runtime Bitcoin (lettre de disque, version, RPC), **mets à jour ce `AGENTS.md` en premier**.
