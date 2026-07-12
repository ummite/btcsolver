# BTCSolver - Session Log

## Ce qui a été fait (2026-07-11 ~ 2026-07-12)

### Projet
- **BTCSolver v0.2.0** — Scanner de solde Bitcoin 100% privé en Rust
- GitHub: https://github.com/ummite/btcsolver
- Branch: `master`

### Infrastructure mise en place
- **Bitcoin Core 31.1** installé sur `Y:\bitcoin-31.1\`
- **Datadir Bitcoin**: `Y:\Bitcoin` (config dans `Y:\Bitcoin\bitcoin.conf`)
- **bitcoind** en cours d'exécution, sync en cours (~600 Go)
- RPC actif sur `127.0.0.1:8332` avec cookie auto

### Modifications du code
- `src/main.rs`: `default_bitcoin_datadir()` cherche d'abord `Y:\Bitcoin` puis `%APPDATA%\Bitcoin`
- `Launch-BitcoinCore.ps1`: datadir = `Y:\Bitcoin`, cherche exe dans `Y:\bitcoin-31.1\bin` en priorité
- `Launch-BitcoinCore.bat`: datadir = `Y:\Bitcoin`
- `.gitignore`: exclut target/, .exe, logs, .redb, .cookie, bitcoin.conf

### Points importants
- Données Bitcoin sur **Y:** (RAID DS1821), jamais sur C:
- Le projet compile en release: `cargo build --release` → `target/release/btcsolver.exe`
- 1 warning mineure: ligne 665, variable `n` non lue avant réaffectation
- Le nœud doit être 100% sync pour que `btcsolver balance` fonctionne en mode RPC
- Mode offline (`build-index`) possible avec un snapshot UTXO

### Déplacement
- Projet déplacé de `A:\BTCSolver` → `Y:\BTCSolver` (2026-07-12)
- Travailler dorénavant depuis `Y:\BTCSolver`

### À faire / à discuter
- Vérifier quand le nœud sera sync et tester btcsolver
- Corriger la warning sur la ligne 665
- Considérer un snapshot UTXO pour le mode offline rapide
- Renommer la branche `master` → `main` ?
