# BTCSolver — Fichier de reprise

**Sauvegardé le :** 2026-07-13 ~07:40

---

## État actuel de la synchro Bitcoin Core

| Métrique | Valeur |
|---|---|
| Bloc vérifié | ~771 516 |
| Date du bloc | ~août 2023 |
| Pointe réelle (tip) | ~957 779 |
| Blocs restants | ~186 263 |
| % blocs vérifiés | ~80,6% |
| % travail de vérification | ~54% (est.) |
| Disque utilisé | ~475 Go |
| `initialblockdownload` | `true` (encore en IBD) |

**Vitesse moyenne :** ~460-570 blocs/min (variable selon la taille des blocs et les UTXO flushes)

---

## Ce qui a été fait

1. **BTCSolver v0.2.0** — Projet lu et compris (scanner de solde Bitcoin en Rust)
2. **Commande `history`** — Implémentée dans `src/main.rs` (via sous-agent). Fonctionne mais nécessite `addrindex=1` pour l'historique complet
3. **Test de la clé `...0001`** — Solde = 0 sat, aucune transaction trouvée (sur les blocs synchro jusqu'alors)
4. **Monitor de synchro** — `monitor_sync.ps1` en place, affiche bloc + % + date estimée + disque chaque minute
5. **Scripts utilitaires** — `check_key.ps1` pour tester n'importe quelle clé (solde + historique)
6. **Logs ignorés** — `*.log` + scripts temp ajoutés au `.gitignore`
7. **Synchro Bitcoin Core** — Lancée de 0% et avancée jusqu'à ~80,6% (bloc 771k)

---

## Problèmes rencontrés

- **bitcoind tombe tous les ~10h** : Le wrapper de tâche de fond impose une limite de 10h. Solution : relancer avec `cmd /c start /b` + tâche background `timeout:0`
- **UTXO flushes lents** : Sur le RAID DS1821, chaque flush prend 5-15 min (blocage temporaire de la synchro). Normal, inévitable.
- **Replay au redémarrage** : ~35k blocs à rejouer = 30-40 min sur ce RAID
- **`-daemon` non supporté** sur Windows → utiliser `cmd /c start /b` ou `Start-Process`

---

## Ce qui reste à faire

### 1. Attendre la fin de la synchro Bitcoin Core
- ~186 000 blocs restants (août 2023 → présent)
- La vitesse va ralentir car les blocs récents sont plus gros
- Attendre `initialblockdownload: false` + `blocks == headers`

### 2. Retester la clé `...0001` une fois la synchro complète
```powershell
powershell -ExecutionPolicy Bypass -File Y:\btcsolver\check_key.ps1
```
Ou manuellement :
```
btcsolver.exe balance --key 0000000000000000000000000000000000000000000000000000000000000001 --cookie-file Y:\Bitcoin\.cookie --sats
btcsolver.exe history --key 0000000000000000000000000000000000000000000000000000000000000001 --cookie-file Y:\Bitcoin\.cookie --sats
```

### 3. Tester d'autres clés (optionnel)
- Clés toutes zéros : `0000000000000000000000000000000000000000000000000000000000000000`
- Clés mnémoniques BIP39 (12-24 mots)
- Clés WIF

### 4. Activer `addrindex=1` (optionnel)
Pour l'historique complet des transactions, ajouter dans `Y:\Bitcoin\bitcoin.conf` :
```
addrindex=1
```
⚠️ Nécessite un reindex complet (`-reindex`) = très long sur ce RAID. À faire seulement si vraiment nécessaire.

---

## Commandes utiles

### Vérifier l'état de la synchro
```powershell
& 'Y:\bitcoin-31.1\bin\bitcoin-cli.exe' -datadir=Y:\Bitcoin getblockchaininfo
```

### Redémarrer bitcoind (si tombé)
```powershell
cmd /c "start /b Y:\bitcoin-31.1\bin\bitcoind.exe -datadir=Y:\Bitcoin -server"
```

### Lancer le monitor
```powershell
powershell -ExecutionPolicy Bypass -File Y:\btcsolver\monitor_sync.ps1
```

### Vérifier si bitcoind tourne
```powershell
tasklist | findstr bitcoin
```

### Consulter le debug log
```powershell
Get-Content Y:\Bitcoin\debug.log -Tail 20
```

---

## Fichiers importants

| Fichier | Description |
|---|---|
| `Y:\btcsolver\src\main.rs` | Code source BTCSolver (avec commande `history` ajoutée) |
| `Y:\btcsolver\monitor_sync.ps1` | Script de monitoring de la synchro |
| `Y:\btcsolver\check_key.ps1` | Script de test solde + historique d'une clé |
| `Y:\Bitcoin\bitcoin.conf` | Configuration Bitcoin Core |
| `Y:\Bitcoin\debug.log` | Log Bitcoin Core |
| `Y:\Bitcoin\.cookie` | Cookie d'authentification RPC |
