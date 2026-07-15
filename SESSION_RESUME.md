# Session resume — BTC Solver Dashboard

**Date:** 2026-07-15  
**Workspace:** `Y:\btcsolver`  
**Langue préférée:** français

## Ce qui est fait

### Dashboard web (principal de cette session)
- **Binaire:** `C:\btcsolver-bin\btcsolver_dashboard.exe` (+ `target\release\btcsolver_dashboard.exe`)
- **Frontend:** `static/dashboard/` (`index.html`, `style.css`, `app.js`) — thème dark pro
- **Backend modules:** `src/dashboard/`
  - `scan_manager.rs` — start/stop `brute_force.exe`, stats GPU/RAM
  - `bitcoind.rs` — start/stop/sync Bitcoin Core via RPC
  - `key_checker.rs` — check Hex / WIF / BIP39 / brainwallet vs FlatIndex
- **Serveur:** `src/bin/btcsolver_dashboard.rs` — Axum + REST + WebSocket `/ws`

### Lancer le dashboard
```powershell
cd Y:\btcsolver
C:\btcsolver-bin\btcsolver_dashboard.exe --port 3000
```
Ouvrir: http://localhost:3000

### Autres (sessions précédentes)
- Brute-force GPU CUDA ~31M keys/s, FlatIndex UTXO, position file resume
- Freshness check snapshot: max 24h par défaut (`--max-snapshot-age`)
- Scan **arrêté** pour mise à jour binaire; position sauvée (~2465T keys, hex `…ced079813`)
- Snapshot était **> 24h** au moment de la session → régénérer ou `--max-snapshot-age 0` pour relancer

## Chemins importants
| Rôle | Chemin |
|------|--------|
| Projet | `Y:\btcsolver` |
| Binaires | `C:\btcsolver-bin\` |
| Cache / snapshot | `C:\btcsolver-cache\` (snapshot: `utxo-index.snapshot`) |
| Frontend | `Y:\btcsolver\static\dashboard\` |

## Prochaines étapes possibles
1. Lancer et tester le dashboard en live
2. Régénérer le snapshot UTXO (si bitcoind synchro)
3. Relancer le scan exhaustif avec freshness check
4. Optionnel: OpenCL multi-device (todo `gpu3` encore pending)
5. Polish UI (auth locale, onglets, reload index après refresh snapshot sans redémarrer)

## Notes utilisateur
- Interface demandée: status temps réel, clés manuelles, départ de scan custom, bitcoind, intervalle refresh UTXO
- Choix validés: web dashboard, formats Hex+WIF+BIP39, full Bitcoin Core mgmt, snapshot auto + on-restart + manuel
- **Ne pas** relancer les tâches de monitoring planifiées (1 min / 15 min) — volontairement arrêtées

## Sécurité
Dashboard **local only** — ne pas exposer sur Internet (clés privées dans l’UI/API).
