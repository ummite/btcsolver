# Liste maîtresse — Alternatives de clés privées brainwallet

Objectif : inventaire exhaustif des idées qu'une personne aurait pu utiliser comme passphrase → SHA256/MD5 → clé privée Bitcoin.

**Règle d'opération** : chaque vague est générée puis scannée automatiquement. Tout match avec solde est stocké dans `brainwallet-master-results.json` + fichiers `*-results.json`.

---

## Déjà testé (Wave 1 + 2)

| # | Catégorie | Statut | Patterns approx. |
|---|-----------|--------|------------------|
| 1 | Bible (EN + FR) | ✅ 0 match | ~41K |
| 2 | Top passwords + variations | ✅ 8 dust matches | inclus corpus |
| 3 | BIP39 words + suffixes | ✅ | ~1.6K base |
| 4 | Phrases Bitcoin/crypto | ✅ | |
| 5 | Citations films (courtes) | ✅ | |
| 6 | Paroles chansons (titres) | ✅ | |
| 7 | Constantes math (π, e, φ…) | ✅ | |
| 8 | Nombres célèbres | ✅ | |
| 9 | Patterns clavier | ✅ | |
| 10 | Citations philosophiques | ✅ | |
| 11 | URLs / sites | ✅ | |
| 12 | Hashs MD5/SHA256 de mots | ✅ | |
| 13 | Langues étrangères (phrases) | ✅ | |
| 14 | Pop culture | ✅ | |
| 15 | Dates de naissance partielles | ✅ | |
| 16 | Double/triple hash | ✅ | |

**Wave 2 matches (dust)** : `""`, `1`, `password`, `market`, `computer`, `test`, `to be or not to be`, `satoshinakamoto` — tous SHA256 uncompressed P2PKH, total ~97K sats.

---

## Wave 3 — en cours de scan (~757K patterns)

| # | Catégorie | Contenu |
|---|-----------|---------|
| 17 | Caractères seuls a–z, A–Z, 0–9 | + paires + triples |
| 18 | Nombres 0–100000 | + hex + binary + octal |
| 19 | Dates de naissance 1900–2010 | 13 formats (DDMMYYYY, ISO, etc.) |
| 20 | Rockyou-style + variations | case, !, années, my/the |
| 21 | Mots anglais courants | + variations |
| 22 | Couleur × Animal | red dragon, black panther… |
| 23 | Prénom × Nom | top ~120×120 |
| 24 | Dates Bitcoin | genesis, halvings, pizza day… |
| 25 | Prénoms + numéros | 1–99, années |
| 26 | Noms d'animaux de compagnie | Max, Bella, Luna… |
| 27 | Acronymes | USA, NASA, SHA256, BIP39… |
| 28 | Slogans de marques | Just do it, Think different… |
| 29 | BIP39 + suffixes étendus | ! 123 wallet key seed |
| 30 | Phrases FR / DE / ES | |
| 31 | Patterns téléphone US | area+prefix+line |
| 32 | Dés RPG (D&D) | 4d6, 2d20, stats |
| 33 | Defaults wallets | Electrum, Armory, Ledger… |
| 34 | Pays + villes | |
| 35 | Hex mémorables | deadbeef, cafebabe, 0×64… |
| 36 | Keyboard walks | qwerty, 1qaz2wsx… |

---

## Wave 4 — généré, scan en attente (~64K patterns)

| # | Catégorie | Contenu |
|---|-----------|---------|
| 37 | Paroles de chansons (versets) | Bohemian Rhapsody, Stairway, Hotel California, Imagine… |
| 38 | Dialogues de films étendus | Matrix, LOTR, Batman, Portal… |
| 39 | Leet speak exhaustif | p@ssw0rd, b1tc01n, h4ck3r… |
| 40 | Paires adjectif+nom | ~60×80 combos |
| 41 | Triplets my/the + adj + nom | my red dragon… |
| 42 | MD5/SHA256/double-SHA comme phrase | hash hex du mot |
| 43 | Citations étrangères | FR, DE, ES, Latin, IT |
| 44 | Sports équipes + joueurs | Yankees, Lakers, Messi… |
| 45 | Usernames / emails | user@gmail.com, admin1… |
| 46 | Mois + années | january2009… |

---

## Wave 5 — à faire (priorité haute)

| # | Catégorie | Pourquoi | Complexité |
|---|-----------|----------|------------|
| 47 | Top 100K Rockyou complet | Vrais mots de passe | Moyen (fichier) |
| 48 | Dictionnaire anglais 10K–50K | Mots simples oubliés | Moyen |
| 49 | Dictionnaire français 10K | Public FR early adopter | Moyen |
| 50 | BIP39 2-word combos | 2048² trop gros → top 200² | Grand |
| 51 | BIP39 3-word (top 50³) | Style seed court | Moyen |
| 52 | Correct horse battery staple + variants | Phrase EFF célèbre | Petit |
| 53 | Phone numbers complets (US common area) | 555-xxxx | Moyen |
| 54 | SSN patterns XXX-XX-XXXX plausibles | Rare mais possible | Moyen |
| 55 | Cartes de crédit patterns (Luhn dump) | Peu probable | Bas |
| 56 | ISBN / numéros de livres | Niche | Bas |
| 57 | Coordinates lat/long mémorables | 40.7128,-74.0060 | Moyen |
| 58 | IP addresses privées | 192.168.1.1 | Petit |
| 59 | MAC addresses patterns | aa:bb:cc… | Petit |
| 60 | UUID v4 "fake" mémorables | 00000000-… | Petit |

---

## Wave 6 — timestamps & formats spéciaux

| # | Catégorie | Notes |
|---|-----------|-------|
| 61 | Unix seconds genesis→now | Binary `timestamp_scan` |
| 62 | Unix milliseconds | Très long (~30+ jours) |
| 63 | ISO 8601 strings | 2009-01-03T18:15:05 |
| 64 | Windows FILETIME | |
| 65 | Human datetime strings | "January 3, 2009 18:15:05" |
| 66 | Bit rotations (256+256) | ×512 overhead |

---

## Wave 7 — culture / niche

| # | Catégorie |
|---|-----------|
| 67 | Coran / Torah / textes religieux non-Bible |
| 68 | Constitution US / Déclaration d'indépendance (phrases) |
| 69 | Whitepaper Bitcoin (paragraphes complets) |
| 70 | IRC / early cypherpunk quotes |
| 71 | Bitcointalk early post titles |
| 72 | MtGox / Silk Road related phrases |
| 73 | Anime / manga quotes |
| 74 | Sports scores mémorables |
| 75 | Lottery numbers / sequences |
| 76 | Pi / e digits windows (100+ digit slices) |
| 77 | Fibonacci sequence as string |
| 78 | Prime numbers list as passphrase |
| 79 | DNA / protein sequences (ATG CGA…) |
| 80 | Morse code of common words |
| 81 | Base64 of common words |
| 82 | ROT13 of common words |
| 83 | Reversed common words (drowssap) |
| 84 | Keyboard shift of common words (qazwsx style) |

---

## Wave 8 — dérivations cryptographiques (lentes)

| # | Catégorie | Note |
|---|-----------|------|
| 85 | PBKDF2(password, salt="bitcoin", N=1000) | Lent |
| 86 | scrypt common passwords | Très lent |
| 87 | SHA256(UTF-16LE / UTF-16BE) | Encoding variants |
| 88 | SHA256 with BOM / trailing newline | CRLF vs LF |
| 89 | Keccak / SHA3 as key | Rare |
| 90 | RIPEMD160 as 20-byte → pad to 32 | Rare |

---

## Wave 9 — combos structurels

| # | Catégorie |
|---|-----------|
| 91 | word + special + number + year (Password1!2009) |
| 92 | CamelCase multi-word (MyBitcoinWallet) |
| 93 | snake_case / kebab-case / dot.case |
| 94 | Prefix "btc_" / "wallet_" / "key_" + word |
| 95 | Suffix "_btc" / "_key" / "_wallet" |
| 96 | Year in middle (bit2009coin) |
| 97 | Repeated word (bitcoinbitcoin) |
| 98 | Word × 3 (lol lol lol) |
| 99 | Empty + whitespace variants (space, tab) |
| 100 | Unicode lookalikes (о vs o) — careful encoding |

---

## Fichiers

| Fichier | Rôle |
|---------|------|
| `brainwallet-all-corpus.txt` | Wave 1 (~54K) |
| `brainwallet-wave2-corpus.txt` | Wave 2 (~18K) |
| `brainwallet-all-corpus-v2.txt` | Wave 1+2 (~71K) |
| `brainwallet-wave3-corpus.txt` | Wave 3 (~757K) |
| `brainwallet-wave4-corpus.txt` | Wave 4 (~64K) |
| `brainwallet-all-v2-results.json` | Matches wave 1+2 |
| `brainwallet-wave3-results.json` | Matches wave 3 (en cours) |
| `brainwallet-master-results.json` | Agrégat de tous les matches |
| `brainwallet-master-list.md` | Ce document |

---

## Pipeline auto

1. Générer corpus (Python)
2. Scanner avec `brainwallet_extended --hash both`
3. Si matches > 0 → merge dans `brainwallet-master-results.json`
4. Enchaîner la vague suivante sans attendre d'instruction
)
