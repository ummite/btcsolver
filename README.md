# BTCSolver — Scanner de solde BTC ultra-privé et **le plus rapide possible**

Outil en ligne de commande **100 % local et privé** qui permet de connaître le solde d'une (ou plusieurs) clé(s) privée(s) Bitcoin **sans jamais l'envoyer sur internet**.

**Objectif principal** : te donner le solde **le plus vite possible** après avoir accepté de synchroniser (ou d'obtenir) la chaîne.

## Chemin LE PLUS RAPIDE (heures au lieu de jours)

Puisque tu acceptes d'être "quelques jours en retard sur le live" (les données UTXO d'il y a quelques jours suffisent largement pour la plupart des usages), **tu n'as pas besoin de synchroniser 650 Go depuis zéro**.

1. **Télécharge un snapshot UTXO récent** (fichier .dat de quelques Go seulement) :
   - Via torrent (le plus rapide souvent) :
     - Exemples récents (cherche "bitcoin utxo snapshot torrent" pour les plus frais) :
       - Hauteur ~880000 : `magnet:?xt=urn:btih:559bd78170502971e15e97d7572e4c824f033492&dn=utxo-880000.dat`
       - Autres bons miroirs : https://lopp.net/ (James O'Beirne en maintient régulièrement) ou torrents communautaires.
   - Ou téléchargement direct HTTP quand disponible.
   - Temps : **quelques heures** max sur une bonne connexion (beaucoup plus rapide que l'IBD complète).

2. **Construis l'index instantané** (sans même avoir besoin de lancer bitcoind pour les queries) :
   ```powershell
   .\btcsolver.exe build-index --snapshot C:\Downloads\utxo-880000.dat --output C:\Data\btc-index.redb
   ```
   (Ça te dira exactement sur quel bloc les données sont basées + lien mempool.space pour voir la date.)

3. **Obtiens le solde en < 1 seconde** (même pour des centaines de clés) :
   ```powershell
   .\btcsolver.exe balance --index C:\Data\btc-index.redb --key TA_CLE_PRIVEE --sats
   ```

**Résultat** : setup en quelques heures (téléchargement + build), puis **solde instantané** pour toutes tes clés futures, sans nœud qui tourne, sans partage de clé, et avec des données "quelques jours en retard" (ce qui te va).

Tu peux refaire l'opération tous les mois ou quand tu veux des données plus fraîches.

---

Le reste du README documente le mode "nœud complet + RPC" (si tu veux les données en temps réel) et tous les détails.

## Pourquoi "download complet de la chaîne" ?

Oui. Pour être totalement souverain et ne faire confiance à personne :

1. Vous téléchargez et synchronisez la blockchain Bitcoin complète (ou presque, avec `prune`).
2. Le nœud maintient en permanence l'UTXO set (l'état actuel de toutes les pièces non dépensées).
3. `scantxoutset` demande au nœud de scanner cet UTXO set à la recherche des scripts correspondant à vos adresses dérivées de la clé privée.

Stockage requis : ~ 500-650 Go (full) ou beaucoup moins avec prune (quelques Go à 100+ Go selon votre tolérance). Un SSD NVMe est fortement recommandé.

## Installation / Build

Prérequis : [Rust](https://rustup.rs/) (rustup).

```powershell
cd C:\Programmation\BTCSolver
cargo build --release
```

Le binaire prêt à l'emploi est copié à la racine : `btcsolver.exe`

Vous pouvez le copier dans un dossier du PATH (ex: `C:\Tools\`) pour l'utiliser de n'importe où.

**Note** : les commandes utilisent maintenant des sous-commandes explicites (`balance` et `build-index`).

## Configuration Bitcoin Core (Windows)

1. Téléchargez la version officielle : https://bitcoincore.org/en/download/
2. Installez et lancez **Bitcoin Core** (ou `bitcoind.exe` en ligne de commande).
3. **Important** : pendant l'installation ou via "Settings > Data Directory", choisissez un dossier sur un disque avec **au moins 600 Go libres** (idéalement un NVMe rapide).
4. Laissez tourner jusqu'à ce que la synchronisation soit à 100 % (ça peut prendre des heures ou des jours selon votre connexion et disque — la première fois c'est long).

Créez / éditez le fichier de configuration `bitcoin.conf` dans votre datadir (ex: `C:\Users\VotreNom\AppData\Roaming\Bitcoin\bitcoin.conf`) :

```ini
# Active le serveur RPC (obligatoire)
server=1

# Laissez le nœud créer automatiquement le fichier .cookie (recommandé pour l'auth)
# Vous pouvez aussi mettre :
# rpcuser=votreuser
# rpcpassword=votremotdepasseultralong

# Optionnel : réduire l'espace disque (le set UTXO reste complet de toute façon)
# prune=10000   # ~10 Go de blocs + UTXO set

# Limite l'accès RPC au localhost (sécurité)
rpcallowip=127.0.0.1
```

Redémarrez Bitcoin Core / bitcoind.

Vérifiez que ça marche :
Ouvrez la console dans Bitcoin Core GUI ou utilisez `bitcoin-cli getblockchaininfo`.

## Utilisation

### Clé unique (la plus courante)

Mode nœud local (RPC) :

```powershell
btcsolver.exe balance --key L1aW4aubDFB7yfras2S1mN3bqg9nwySY8nkoLmJebSLD5BWv3ENZ
```

Ou avec l'index offline ultra-rapide (après `build-index`) :

```powershell
btcsolver.exe balance --index C:\Data\btc-balances.redb --key L1aW4aubDFB7yfras2S1mN3bqg9nwySY8nkoLmJebSLD5BWv3ENZ --sats
```

En hex (les deux modes) :

```powershell
btcsolver.exe balance --key 0000000000000000000000000000000000000000000000000000000000000001
```

Options utiles :

- `--network main|test|signet|regtest` (défaut: main)
- `--sats` → affiche en satoshis
- `--show-all` → affiche toutes les adresses même à 0
- `--verbose` → infos sur l'état du nœud
- `--derive-only` → dérive seulement les adresses (aucun appel RPC, utile pour vérifier ou airgap)

### Lots de clés (très efficace)

Créez un fichier `mes-cles.txt` :

```
L1aW4aubDFB7yfras2S1mN3bqg9nwySY8nkoLmJebSLD5BWv3ENZ
5HpHagT65TZzG1PH3CSu63k8DbpvD8s5ip4nEB3kEsreAnchuDf
...
```

Puis :

```powershell
btcsolver.exe --file mes-cles.txt --sats
```

Le programme collecte **toutes** les adresses dérivées (legacy, segwit natif, wrapped, taproot), déduplique, et fait **un seul appel** `scantxoutset`. C'est optimal.

Vous pouvez aussi utiliser `--stdin` pour pipe :

```powershell
type mes-cles.txt | btcsolver.exe --stdin
```

### Authentification RPC (si besoin)

Le plus simple = laisser Bitcoin Core gérer le `.cookie` (fichier auto-généré dans le datadir). Le programme le trouve automatiquement.

Autres possibilités :

```powershell
# Fichier cookie explicite
btcsolver.exe --key XXXXX --cookie-file "C:\Users\...\Bitcoin\.cookie"

# User / pass
btcsolver.exe --key XXXXX --rpc-user admin --rpc-password SuperMotDePasse123

# URL complète
btcsolver.exe --key XXXXX --rpc-url http://admin:pass@127.0.0.1:8332
```

## Ce que l'outil vérifie exactement

À partir d'une clé privée, il dérive les 4 types d'adresses standards qu'une personne peut utiliser :

- `legacy (P2PKH)` → adresses commençant par `1` (ou `m` en testnet)
- `native segwit (P2WPKH)` → `bc1q...`
- `wrapped segwit (P2SH-P2WPKH)` → `3...`
- `taproot (P2TR)` → `bc1p...`

Il scanne l'UTXO set pour **tous ces scripts**. Si des bitcoins ont été reçus sur l'une de ces adresses, le solde apparaîtra (même si les pièces n'ont jamais bougé).

**Note** : le solde retourné est le solde **confirmé** (UTXO set = pièces non dépensées dans la chaîne actuelle).

## Mode "index offline ultra-rapide" (recommandé quand tu acceptes de synchroniser la chaîne complète)

Puisque tu as confirmé que synchroniser la blockchain complète ne te dérange pas, voici **la façon la plus efficace** à long terme :

1. Synchronise Bitcoin Core complètement (full ou avec prune raisonnable).
2. Génère un snapshot officiel (Core fait tout le travail de décodage correctement) :
   ```powershell
   bitcoin-cli dumptxoutset C:\Temp\utxos-latest.dat latest
   ```
   (Ça prend du temps et de l'espace — plusieurs Go.)

3. Construis l'index compact et rapide avec btcsolver :
   ```powershell
   .\btcsolver.exe build-index --snapshot C:\Temp\utxos-latest.dat --output C:\Data\btc-balances.redb
   ```

4. Ensuite, **plus jamais besoin de lancer le nœud** pour checker des clés :
   ```powershell
   .\btcsolver.exe balance --index C:\Data\btc-balances.redb --key L1aW4aubDFB7yfras2S1mN3bqg9nwySY8nkoLmJebSLD5BWv3ENZ --sats
   ```

Avantages :
- Requêtes **instantanées** (même pour des milliers de clés).
- 100% offline et privé.
- Tu contrôles quand mettre à jour (refais dumptxoutset + build-index quand tu veux les données les plus récentes).

Le fichier `.redb` est un index agrégé par scriptPubKey (les 4 types dérivés de ta clé sont testés exactement).

Tu peux aussi continuer à utiliser le mode RPC classique (sans `--index`) tant que ton nœud est synchronisé et tourne — il donnera toujours les données les plus fraîches sans rebuild.

## Pour aller encore plus loin (usage très intensif / solver)

L'index ci-dessus est déjà excellent pour des scans massifs de clés que tu possèdes ou génères.

Si tu as besoin de fonctionnalités supplémentaires (filtre Bloom pré-filtre, stockage par adresse lisible en plus du script, historique complet des réceptions au lieu de seulement le solde UTXO actuel, etc.), dis-le — on peut l'enrichir.

## Sécurité & bonnes pratiques

- Ne partagez **jamais** le binaire + vos clés sur la même machine que vous utilisez pour autre chose si vous êtes parano.
- Pour les clés très sensibles : exécutez tout sur une machine air-gappée (ou au moins sans réseau pendant l'opération).
- Vérifiez toujours le binaire (`cargo build` vous-même depuis les sources est le plus sûr).
- Le programme n'a **aucun** appel réseau en dehors de votre nœud local.

## Crédits & technique

- Dérivation d'adresses : [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin)
- RPC : appel JSON basique + `ureq`
- Méthode de scan : `scantxoutset` (Bitcoin Core)

Ce projet a été créé pour répondre précisément à : "je veux scanner le solde de **ma** clé privée très rapidement, sans la partager, avec le full chain si nécessaire".

Bonne chasse (responsable) !

---

Si vous trouvez des bugs ou voulez des améliorations (support descriptors, xpriv, export CSV, mode index offline, etc.), ouvrez une issue ou une PR.