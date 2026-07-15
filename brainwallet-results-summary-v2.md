# Brainwallet Scan Results - Wave 2 (Corpus étendu v2)

## Configuration
- **Corpus** : 71,207 phrases uniques (53,912 Wave 1 + 18,365 Wave 2)
- **Variations testées** : 5,808,800 (SHA256+MD5 × compressed+uncompressed × 4 address types)
- **Vitesse** : 186,390 phrases/sec
- **Temps** : 31.2 secondes
- **Threads** : 22
- **FlatIndex** : 28.6M scripts, 65.5M UTXOs, 3.8 GB RAM
- **Binaire** : brainwallet_extended (corrigé du bug UTXO offset)

## Catégories Wave 2 (18,365 nouveaux patterns)

| # | Catégorie | Exemples |
|---|-----------|----------|
| 1 | Citations de livres célèbres | "Call me Ishmael", "It was the best of times..." |
| 2 | Jeux vidéo | "master chief", "fus ro dah", "prepare to die" |
| 3 | Leet speak | "p@ssw0rd", "b1tc01n", "h4ck3r" |
| 4 | Personnes célèbres | "albert einstein", "steve jobs", "elon musk" |
| 5 | Animaux | "dragon", "phoenix", "black panther" |
| 6 | Citations du Bitcoin Whitepaper | "A purely peer-to-peer version..." |
| 7 | Commandes Linux/Terminal | "sudo rm -rf /", "chmod 777" |
| 8 | Programmation | "hello world", "return 0", "null pointer" |
| 9 | Artistes/Albums musicaux | "dark side of the moon", "master of puppets" |
| 10 | Religions/Spiritualité | "om mani padme hum", "bismillah", "namaste" |
| 11 | Dates historiques | "1776", "9/11/2001", "genesis block" |
| 12 | Noms de fichiers/chemins | "private.key", "wallet.dat" |
| 13 | Phrases "je me souviens" | "this is my secret", "never forget" |
| 14 | Expressions idiomatiques | "a piece of cake", "break a leg" |
| 15 | Cybersécurité/Hacking | "cypherpunks", "zero day", "darknet" |
| 16 | Espace/Planètes | "mars rover", "black hole", "voyager" |
| 17 | Lettres grecques/Math | "alpha omega", "golden mean", "phi ratio" |
| 18 | Mythologie | "zeus", "odin", "ragnarok", "valhalla" |
| 19 | Phrases CJK (translittérées) | "bit coin nihongo", "ji mi huo bi" |
| 20 | Produits tech | "iphone", "playstation", "vision pro" |
| 21 | Réseaux sociaux | "twitter", "reddit", "4chan" |
| 22 | Livres SF | "dune", "neuromancer", "three body problem" |
| 23 | Séries TV | "breaking bad", "game of thrones", "dark" |
| 24 | Paires de mots (EFF style) | "correct horse", "blue guitar" |
| 25 | Éléments chimiques | "uranium", "plutonium", "oganesson" |
| 26 | Formules scientifiques | "e=mc2", "quantum entanglement" |
| 27 | Devises | "dollar", "euro", "bitcoin dollar" |
| 28 | Marques automobiles | "ferrari", "lamborghini", "cybertruck" |
| 29 | Cartes/Jeu | "ace of spades", "royal flush", "natural 20" |
| 30 | Météo/Saisons | "thunder storm", "rainbow" |
| 31 | Couleurs | "crimson", "azure", "cerulean" |
| 32 | Nourriture | "pizza", "sushi", "tiramisu" |
| 33 | Variations séparateurs | "my-bitcoin-key", "my_bitcoin_key" |
| 34 | Phrases "je veux" | "i want bitcoin", "i want freedom" |
| 35 | OS/Logiciels | "ubuntu", "firefox", "arch" |
| 36 | Exchanges crypto | "coinbase wallet", "yield farming" |
| 37 | Patterns hex mémorables | "0xdeadbeef×4", "0xcafebabe×4" |
| 38 | Genesis block | "chancellor on brink...", "50 bitcoin" |
| 39 | Passwords de breaches | 200+ mots de passe réels de fuites |
| 40 | Brainwallet.org spécifique | "brainwallet sha256", "brainwallet seed" |

## Résultats : 8 correspondances trouvées

| # | Phrase | Hash | Key Type | Address | Sats | BTC |
|---|--------|------|----------|---------|------|-----|
| 1 | *(vide)* | SHA256 | Uncompressed | `1HZwkjkeaoZfTSaJxDw6aKkxp45agDiEzN` | 11,116 | 0.00011116 |
| 2 | `1` | SHA256 | Uncompressed | `12AKRNHpFhDSBDD9rSn74VAzZSL3774PxQ` | 10,000 | 0.0001 |
| 3 | `satoshinakamoto` | SHA256 | Uncompressed | `1K9qgN3H2wB2v3LwJEBDbRRJ3znHXEQP4Y` | 1,000 | 0.00001 |
| 4 | `password` | SHA256 | Uncompressed | `16ga2uqnF1NqpAuQeeg7sTCAdtDUwDyJav` | 30,000 | 0.0003 |
| 5 | `to be or not to be` | SHA256 | Uncompressed | `1J3m4nneGFppRjx6qv92qyz7EsMVdLfr8R` | 7,000 | 0.00007 |
| 6 | `market` | SHA256 | Uncompressed | `1AYxhQmNm6FVfcRkY23TWTgcsZz7o7bWXe` | 30,000 | 0.0003 |
| 7 | `test` | SHA256 | Uncompressed | `1HKqKTMpBTZZ8H5zcqYEWYBaaWELrDEXeE` | 3,053 | 0.00003053 |
| 8 | `computer` | SHA256 | Uncompressed | `1PfW1aNW4NQ9JgKqcGeB6u4GjW9hCDZhBw` | 5,000 | 0.00005 |

**Total** : 97,169 sats ≈ 0.00097169 BTC ≈ ~$0.61 (à ~$63K/BTC)

## Analyse

### Ce qui est notable
1. **Toutes en SHA256 + Uncompressed** : Personne n'a utilisé MD5 ou compressed keys pour les brainwallets simples
2. **Toutes en P2PKH legacy** : Aucun segwit ou taproot
3. **Montants minuscules** : Ce sont du "dust" - probablement des tests ou du dusting
4. **`password`** : 45,028 transactions sur blockchain.info - cette adresse a été massivement dustée
5. **`market`** : 2 transactions seulement, déjà dépensé depuis le snapshot
6. **`satoshinakamoto`** : 1,000 sats - le minimum UTXO non-poussière
7. **`to be or not to be`** : Shakespeare - classique brainwallet
8. **String vide** : SHA256("") - cas limite testé par curiosité

### Ce qui n'a PAS été trouvé
- **Aucun brainwallet avec un solde significatif** (> 0.001 BTC)
- **Aucune citation de film/chanson** avec un solde
- **Aucune phrase BIP39** avec un solde
- **Aucune URL** avec un solde
- **Aucune phrase en langue étrangère** avec un solde
- **Aucune variation compressed** avec un solde
- **Aucun MD5** avec un solde
- **Aucun segwit/taproot** avec un solde

### Conclusion
Les brainwallets "évidents" (mots simples, citations connues) ont tous été testés et ne contiennent que de la poussière. Personne n'a stocké de vrais bitcoins sur des brainwallets prévisibles - ou alors ils les ont déjà dépensés.
