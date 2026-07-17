# Runtime rapide (résumé une page)

> Copie compacte de `AGENTS.md` pour lecture ultra-rapide.

## Bitcoin Core = W:\Bitcoin (local)

```powershell
# Start
.\Launch-BitcoinCore.ps1
# ou
Start-Process "W:\Bitcoin\bin\daemon\bitcoind.exe" -ArgumentList "-datadir=W:\Bitcoin" -WindowStyle Hidden

# Status
& "W:\Bitcoin\bin\daemon\bitcoin-cli.exe" -datadir=W:\Bitcoin getblockchaininfo
Get-Content W:\Bitcoin\debug.log -Tail 20

# Stop
W:\Bitcoin\Stop-BitcoinCore.bat
```

| | |
|--|--|
| Datadir | `W:\Bitcoin` |
| Version | 31.1.0 portable |
| RPC | `127.0.0.1:8332` user `btcsolver` |
| Blocks | plaintext, `blocksxor=0` |
| NAS legacy | `Y:\Bitcoin` — ne pas utiliser par défaut |

## Projet

- Repo : `Y:\btcsolver`
- Build : `cargo build --release --bin btcsolver_dashboard`
- **Dashboard pro** : `.\Launch-Dashboard.ps1` → http://localhost:3000
- Index UTXO : vérifier hauteur / scripts avant de croire un solde
- Idées chasse de clés : `KEY-HUNT-IDEAS.md` + onglet Idées

## Au démarrage agent

1. Lire `AGENTS.md`
2. Check / start bitcoind sur **W:**
3. Si besoin UI : lancer le dashboard
4. Ne pas lancer moniteurs planifiés sans demande
