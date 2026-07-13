# BTCSolver - Résumé de synchronisation

## État actuel (2026-07-12 ~18:04)

| Métrique | Valeur |
|---|---|
| **Bloc actuel** | 567 629 (59.27%) |
| **Date estimée** | ~2019-10-15 |
| **Disque utilisé** | 221.7 Go |
| **bitcoind** | ✅ Redémarré (arrêt à 18:00) |

## Historique des jalons

| Heure | Bloc | % | Disque | Note |
|---|---|---|---|---|
| 08:00 | 0 | 0% | 0 | Démarrage bitcoind |
| 09:37 | 224 627 | 23.38% | 6.5 Go | Premier scan RPC (clé 000000...0001 = 0 sat) |
| 10:16 | 287 542 | 30.03% | 16.9 Go | 30% |
| 11:39 | 383 483 | 40.04% | 53.3 Go | 40% |
| 12:55 | 430 903 | 45.0% | 89.9 Go | 45% |
| 14:47 | 479 224 | 50.04% | 136.1 Go | **50% - LA MOITIÉ** |
| 16:13 | 519 065 | 54.2% | 175.7 Go | Flush UTXO long (12 min) |
| 17:05 | 547 143 | 57.13% | 200 Go | Disque à 200 Go |
| 17:27 | 552 095 | 57.65% | 205.5 Go | Flush UTXO long (9 min) |
| 18:00 | 567 629 | 59.27% | 221.7 Go | **bitcoind arrêté (wrapper timeout)** |
| 18:04 | — | — | — | **bitcoind redémarré** |

## Vitesse moyenne

- **~70 000 blocs/heure** (moyenne sur les 10 heures, avant ralentissement)
- **~22 Go/heure** de données téléchargées
- **Ralentissement** : la vitesse baisse significativement après 2018 (blocs plus gros)
- **Estimation restante** : ~10-15 heures (vitesse en baisse + flush UTXO)

## Notes importantes

- Les flush UTXO périodiques (tous les ~5-10%) ralentissent la sync sur le RAID réseau
- Chaque flush prend 3-12 minutes sur le DS1821
- bitcoind a été arrêté à 18:00 (timeout du wrapper PowerShell à 10h) et redémarré
- La vitesse continuera de baisser (les blocs 2020+ sont encore plus gros)
