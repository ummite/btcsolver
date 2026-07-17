# Corpus listes (Scan listes)

Fichiers générés pour brainwallet / dict scan (1 entrée par ligne).

| Fichier | Contenu (approx.) |
|---------|-------------------|
| `../corpus-cities-world.txt` | ~41 000 villes (world-cities / GeoNames, + villes JP) |
| `../corpus-names-first.txt` | ~22 000 prénoms (monde + JP) |
| `../corpus-names-surnames.txt` | ~84 000 noms de famille |
| `../corpus-names-japanese.txt` | prénoms/noms JP romaji + combos famille+prénom |
| `../corpus-names-cities-all.txt` | **fusion** ~145 000 lignes (villes + prénoms + noms) |

## Usage

Dashboard → **2. Scan listes** → menu « Liste prête » → choisir un `corpus-*`.

Ou en CLI / API :
```
corpus_path: "corpus-names-cities-all.txt"
```

## Limites

- Pas « tous les noms de l’humanité » (impossible / illégal en exhaustif).
- Sources ouvertes : datasets world-cities, NameDatabases, listes romaji JP.
- Régénérer : voir script de session ou re-télécharger les sources raw GitHub.
