# Idées de recherche de clés (brainwallets, seeds, entropy faible)

> Usage éthique : clés oubliées personnelles, corpus publics, recherche historique.
> Ne pas cibler des personnes. Dashboard **localhost only**.

## Classiques qui marchent encore (parfois)

1. **SHA256(phrase)** → clé privée (brainwallet 2011–2014)
2. Variantes texte : lower/upper, reverse chars, reverse words, sans espaces
3. **SHA256d**, MD5 paddé à 32 bytes (outils amateurs)
4. BIP39 + passphrase oubliée (dictionnaire court)
5. 11 mots BIP39 + bruteforce du 12e

## Plus novateur / sous-exploité

| Idée | Pourquoi |
|------|----------|
| Electrum v1 seeds | Dérivation ≠ BIP39 ; beaucoup de vieux wallets |
| Minikeys Casascius | Format court, encore du dust on-chain |
| Timestamps → SHA256 | Dates de naissance, epochs « lancement Bitcoin » |
| Warpwallet / scrypt | Coûteux → moins scanné massivement |
| Chemins non-std `m/0`, `m/0'/0'` | Wallets custom / scripts basiques |
| UTF-16LE hash | Apps Windows/Android |
| P2PK uncompressed early | 2009–2012 |
| Leet + multi-langue FR/EN | Phrases humaines réelles |
| Username + année | Corpus public type rockyou-style, pas doxxing |
| Partial key bits | Si un backup révèle N bits, brute le reste |
| Tip exact via `dumptxoutset` | Une fois Core sync, snapshot UTXO vrai tip |

## Pipeline pro (ce repo)

```
bitcoind W:\Bitcoin → IBD false
  → full_utxo_indexer build (blocks plaintext, xor 0)
  → reload FlatIndex
  → clés manuelles / dict waves / GPU hex
```

## Priorisation

1. Corriger la **fraîcheur UTXO** (sinon faux positifs historiques)
2. Phrases **haute intention** avant rockyou complet
3. Garder les **candidates** même si solde 0 sur index partiel
4. Vérifier on-chain (mempool.space / nœud tip) avant toute conclusion
