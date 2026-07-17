# Accélérer les validations — guide BTCSolver

Deux « validations » différentes :

| Type | Où | Objectif |
|------|-----|--------|
| **A. Bitcoin Core IBD** | `bitcoind` | blocs validés → tip réseau |
| **B. Soldes / clés** | dashboard, brute, dict | lookup UTXO fiables |

---

## A. Bitcoin Core plus rapide

### État typique
- Blocs déjà sur `W:\Bitcoin\blocks` (pas le download le goulot)
- Goulot = **CPU** (scripts/signatures) + **RAM/cache LevelDB** (chainstate)
- W: calme = normal

### Leviers (du plus fort au plus faible)

#### 1. `dbcache` très haut (déjà préparé : 96 Go)
Dans `W:\Bitcoin\bitcoin.conf` :
```
dbcache=98304
```
Avec ~224 Go RAM, 64–96 Go de cache accélère fortement les flushes UTXO.

**Prend effet au redémarrage** de bitcoind (le cache RAM se reconstruit).

#### 2. Couper le travail inutile pendant l’IBD
```
blocksonly=1          # déjà
txindex=0             # déjà
# blockfilterindex=1  # OFF pendant IBD (réactiver après tip)
```
`blockfilterindex` construit un index en plus → ralentit.

#### 3. Ne pas tuer bitcoind
Chaque restart vide une partie du cache → perte de vitesse.

#### 4. `assumevalid` (défaut Core)
Par défaut Core **ne re-vérifie pas toutes les signatures** jusqu’à un bloc de confiance récent.  
`assumevalid=0` = validation totale = **beaucoup plus lent**. Ne l’active pas pour aller plus vite.

#### 5. Bombardière : `loadtxoutset` (assumeutxo)
Core v26+ :
```
bitcoin-cli -datadir=W:\Bitcoin loadtxoutset "chemin\vers\utxo.dat"
```
Charge un snapshot UTXO → nœud **utilisable près du tip en minutes**, pendant qu’un 2e chainstate valide l’historique en fond.

**Conditions :**
- Fichier au format `dumptxoutset` (ex. `W:\Temp\utxo-935000.dat` ~8,7 Go)
- Hauteur reconnue / hash accepté par ta version Core (sinon refus)
- Opération **lourde** : demander confirmation avant de lancer

Une fois au tip « snapshot » : `dumptxoutset` → FlatIndex frais pour les scans.

#### 6. Ordre de grandeur
| Mesure | Impact |
|--------|--------|
| dbcache 24→96 Go | fort |
| stop blockfilterindex | moyen |
| leave PC 24/7 | fort |
| loadtxoutset OK | **énorme** (saut de hauteur) |
| plus de peers | faible (blocs locaux) |

---

## B. Scans de clés plus rapides (et corrects)

### Règle d’or après les faux positifs GPU
**Tout solde archivé = confirmation FlatIndex CPU** (adresses dérivées de la clé).  
Le GPU peut accélérer la dérivation, pas mentir sur le solde.

### Leviers

| Action | Gain |
|--------|------|
| Index UTXO en RAM (dashboard chargé) | déjà |
| Index en VRAM + lookup GPU (FULL) **après** kernel corrigé + re-vérif CPU | élevé si fiable |
| Multi-GPU 1 worker / carte | élevé |
| Batchs 0,75–1M clés | moyen |
| Moins de transforms (mode Rapide) | fort sur dict |
| Max mots permutations bas (5–6) | fort |
| PRIORITY-SYNC jusqu’au tip Core | libère CPU pour Core |

### Pipeline cible (fiable + rapide)
```
GPU : privkeys → pubkeys/hash160  (3 cartes en parallèle)
CPU : scripts + FlatIndex.lookup  (seul juge du solde)
Hit  : bip + archive + export
```

---

## Checklist opérationnelle maintenant

1. **Laisser bitcoind tourner** (PID stable sur W:)
2. Conf déjà accélérée → **un seul restart propre** quand tu veux appliquer `dbcache=98304` :
   ```bat
   W:\Bitcoin\Stop-BitcoinCore.bat
   W:\Bitcoin\Launch-BitcoinCore.bat
   ```
3. Après tip Core :
   - réactiver `blockfilterindex=1` si besoin d’historique
   - `Keep-Core-And-Utxo` → UTXO tip
   - `Disable-PrioritySync` → scans GPU
4. **Ne pas faire confiance** aux `found-keys` du scan GPU du 16/07 (faux positifs) — dossier `data\FALSE-POSITIVES-*`

---

## Commandes utiles

```powershell
# Progression Core
& "W:\Bitcoin\bin\daemon\bitcoin-cli.exe" -datadir=W:\Bitcoin getblockcount
& "W:\Bitcoin\bin\daemon\bitcoin-cli.exe" -datadir=W:\Bitcoin getblockchaininfo

# Vitesse (2 mesures)
# ... getblockcount ; sleep 30 ; getblockcount

# Mémoire bitcoind
Get-Process bitcoind | Select WS,CPU
```
