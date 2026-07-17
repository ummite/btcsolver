#!/usr/bin/env python3
"""Translate French user-facing text in static/dashboard/index.html to English."""

filepath = r'Y:\btcsolver\static\dashboard\index.html'
with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

replacements = [
    # Line 65 - infoHitsSub
    ('aucune clé avec activité on-chain pour l\u2019instant', 'no key with on-chain activity yet'),

    # Line 71 - badgeUtxo title
    ('Fraîcheur de l\u2019index UTXO vs tip', 'UTXO index freshness vs tip'),

    # Line 73 - infoUtxoBlock title
    ('Hauteur exacte de l\u2019index UTXO (pas de raccourci K)', 'Exact UTXO index height (no K shortcut)'),

    # Line 75 - date bloc
    ('date bloc:', 'block date:'),

    # Line 76 - généré
    ('généré:', 'generated:'),

    # Line 80 - Scan en cours
    ('Scan en cours', 'Scan in progress'),

    # Line 81 - testées
    ('testées:', 'tested:'),

    # Lines 87-89 - 24/7 banner
    ('le scan de fond (brute) tourne tout seul.', 'the background scan (brute) runs on its own.'),
    ('Tu peux coller d\u2019autres mots ici', 'You can paste other words here'),
    ('sans l\u2019arrêter', 'without stopping it'),
    ('onglet <strong>1. Tester une clé</strong>', 'tab <strong>1. Test a key</strong>'),
    ('Hits communs :', 'Common hits:'),
    ('coffre \u201c Clés trouvées \u201d', 'vault "Found keys"'),

    # Lines 92-95 - UTXO freshness warning
    ('UTXO : évaluation fraîcheur\u2026', 'UTXO: freshness check\u2026'),
    ('Règle : index UTXO', 'Rule: UTXO index'),
    ('valable pour les tests', 'valid for testing'),
    ('s\u2019il a moins de', 'if it is less than'),
    ('de retard sur le tip.', 'behind the tip.'),
    ('Au-delà : hits = candidats seulement (pas de solde garanti).', 'Beyond that: hits = candidates only (no guaranteed balance).'),

    # Lines 103-107 - Tabs
    ('1. Tester une clé', '1. Test a key'),
    ('2. Scan listes', '2. Scan lists'),
    ('3. Clés trouvées', '3. Found keys'),
    ('4. Scans auto', '4. Auto scans'),
    ('Système</button>', 'System</button>'),

    # Line 115 - hunt heading
    ('Colle n\u2019importe quoi', 'Paste anything'),
    ('on teste le max de dérivations', 'we test max derivations'),

    # Line 116 - hint
    ('plusieurs lignes = lot', 'multiple lines = batch'),

    # Line 120 - placeholder
    ('Exemples:', 'Examples:'),
    ('mon mot de passe 2013', 'my password 2013'),

    # Line 125 - option
    ('Auto (recommandé)', 'Auto (recommended)'),

    # Line 132 - passphrase label
    ('Passphrase BIP39 (si tu l\u2019as oubliée, teste des variantes en lot)', 'BIP39 passphrase (if you forgot it, test variants in batch)'),

    # Line 133 - placeholder
    ('vide = aucune', 'empty = none'),

    # Line 140 - buttons
    ('Mode MAX (toutes méthodes)', 'MAX Mode (all methods)'),
    ('Mode rapide', 'Quick Mode'),

    # Line 142 - checkbox
    ('Seulement soldes', 'Only balances'),

    # Line 145 - summary
    ('Méthodes activées (Mode MAX coche tout)', 'Methods enabled (MAX Mode checks all)'),

    # Lines 150-162 - method tooltips (title attributes)
    ('Hash SHA-256 de la phrase', 'SHA-256 hash of the phrase'),
    ('32 octets = clé privée', '32 bytes = private key'),
    ('Méthode brainwallet classique.', 'Classic brainwallet method.'),
    ('Double SHA-256 : SHA256(SHA256(phrase))', 'Double SHA-256: SHA256(SHA256(phrase))'),
    ('clé privée. Variante moins courante que SHA256 simple.', 'private key. Less common variant than simple SHA256.'),
    ('sur 16 octets, puis padding de 16 zéros', 'on 16 bytes, then padded with 16 zeros'),
    ('Ancienne variante brainwallet peu sûre mais parfois vue.', 'Old brainwallet variant, insecure but sometimes seen.'),
    ('MD5 paddé', 'Padded MD5'),
    ('Inverse l\u2019ordre des caractères', 'Reverses the order of characters'),
    ('puis hash.', 'then hash.'),
    ('Reverse caractères', 'Reverse characters'),
    ('Inverse l\u2019ordre des mots (séparés par espaces)', 'Reverses word order (space-separated)'),
    ('Reverse mots', 'Reverse words'),
    ('Met toute la phrase en minuscules avant le hash', 'Converts entire phrase to lowercase before hashing'),
    ('Met toute la phrase en MAJUSCULES avant le hash', 'Converts entire phrase to UPPERCASE before hashing'),
    ('Supprime tous les espaces', 'Removes all spaces'),
    ('Sans espaces', 'No spaces'),
    ('Enlève ponctuation et symboles', 'Removes punctuation and symbols'),
    ('Garde lettres, chiffres et espaces', 'Keeps letters, digits and spaces'),
    ('Sans symboles (.,;!?', 'No symbols (.,;!?'),
    ('Ajoute des suffixes à la phrase (minuscules) puis hash chaque variante', 'Adds suffixes to the phrase (lowercase) then hashes each variant'),
    ('Suffixes testés :', 'Suffixes tested:'),
    ('Ajoute des préfixes à la phrase (minuscules) puis hash', 'Adds prefixes to the phrase (lowercase) then hashes'),
    ('Préfixes :', 'Prefixes:'),
    ('Préfixes (my, the', 'Prefixes (my, the'),
    ('Pour une seed BIP39 : dérive plusieurs chemins standards BIP44', 'For a BIP39 seed: derives multiple standard BIP44 paths'),
    ('pas seulement le premier chemin.', 'not just the first path.'),
    ('BIP39 multi-chemins', 'BIP39 multi-path'),

    # Line 166 - bipCount label
    ('Nb adresses BIP39 par chemin', 'Number of BIP39 addresses per path'),

    # Line 172 - buttons
    ('Chercher un solde', 'Check balance'),
    ('Effacer', 'Clear'),

    # Line 182 - heading
    ('Résultats', 'Results'),

    # Line 183 - save button
    ('Sauver les hits dans \u201c Clés trouvées \u201d', 'Save hits to "Found keys"'),

    # Line 184 - keyResult
    ('Aucun test pour l\u2019instant.', 'No tests yet.'),

    # Line 192 - dict heading
    ('Scan une liste de phrases', 'Scan a list of phrases'),

    # Line 193 - hint
    ('Corpus du repo ou colle ta liste.', 'Repo corpus or paste your list.'),
    ('Chaque ligne', 'Each line'),

    # Line 196 - label
    ('Liste prête (projet)', 'Ready list (project)'),

    # Line 197 - option
    ('\u2014 coller des phrases ou choisir \u2014', '\u2014 paste phrases or choose \u2014'),

    # Line 199 - label
    ('Ou colle ici (1 phrase / ligne)', 'Or paste here (1 phrase / line)'),

    # Line 203 - buttons
    ('Max méthodes', 'Max methods'),
    ('Rapide', 'Quick'),

    # Lines 206-219 - dict method tooltips
    ('Hash SHA-256 de chaque phrase', 'SHA-256 hash of each phrase'),
    ('MD5(phrase) 16 octets + 16 zéros de padding', 'MD5(phrase) 16 bytes + 16 zero-padding'),
    ('32 octets (clé).', '32 bytes (key).'),
    ('Inverse les caractères.', 'Reverses characters.'),
    ('Inverse l\u2019ordre des mots.', 'Reverses word order.'),
    ('Minuscules avant hash.', 'Lowercase before hash.'),
    ('Garde lettres/chiffres/espaces.', 'Keeps letters/digits/spaces.'),
    ('Supprime les espaces.', 'Removes spaces.'),
    ('Ajoute des suffixes puis hash.', 'Adds suffixes then hashes.'),
    ('Suffixes :', 'Suffixes:'),
    ('Ajoute des préfixes puis hash.', 'Adds prefixes then hashes.'),
    ('Préfixes :', 'Prefixes:'),
    ('Dérivation + lookup sur toutes les cartes CUDA.', 'Derivation + lookup on all CUDA cards.'),
    ('Threads CPU à 0 = GPU only.', 'CPU threads at 0 = GPU only.'),
    ('Index UTXO en VRAM', 'UTXO index in VRAM'),
    ('lookup on-device (min CPU). Recommandé pour max GPU.', 'on-device lookup (min CPU). Recommended for max GPU.'),
    ('Index FULL en VRAM', 'FULL index in VRAM'),

    # Line 221 - heading
    ('Préfixe / suffixe \u2014 tous caractères', 'Prefix / suffix \u2014 all characters'),

    # Line 222 - hint
    ('(94 car.)', '(94 chars)'),
    ('préfixes en', 'prefixes in'),
    ('boucle', 'loop'),
    ('pas de limite artificielle', 'no artificial limit'),
    ('le total est calculé, les essais sont générés à la volée sans tout stocker en RAM.', 'the total is calculated, attempts are generated on-the-fly without storing everything in RAM.'),

    # Lines 225-239 - prefix/suffix labels and options
    ('Ajoute devant la phrase toutes les chaînes de N caractères du charset. 0 = désactivé.', 'Adds all strings of N characters from the charset before the phrase. 0 = disabled.'),
    ('Préfixe (tous car.)', 'Prefix (all chars)'),
    ('1 caractère', '1 character'),
    ('2 caractères', '2 characters'),
    ('Ajoute derrière la phrase toutes les chaînes de N caractères du charset. 0 = désactivé.', 'Adds all strings of N characters from the charset after the phrase. 0 = disabled.'),
    ('Suffixe (tous car.)', 'Suffix (all chars)'),

    # Line 243 - heading
    ('Mots \u2192 permutations / combinaisons', 'Words \u2192 permutations / combinations'),

    # Line 244 - hint
    ('Les tokens de ta zone (1 mot/ligne ou phrases splitées) forment un sac.', 'Tokens from your area (1 word/line or split phrases) form a bag.'),
    ('ordres + sous-ensembles, avec/sans espaces, puis hashes.', 'orderings + subsets, with/without spaces, then hashes.'),

    # Lines 246-249 - perms/checks
    ('Tous les ordres possibles des mots (n!).', 'All possible word orderings (n!).'),
    ('Permutations (tous les ordres)', 'Permutations (all orderings)'),
    ('Tous les sous-ensembles non vides', 'All non-empty subsets'),
    ('Combinaisons (tous sous-ensembles)', 'Combinations (all subsets)'),
    ('Joint les mots avec un espace.', 'Joins words with a space.'),
    ('Avec espaces', 'With spaces'),
    ('Joint les mots sans espace.', 'Joins words without spaces.'),

    # Line 253 - label
    ('Max mots dans le sac', 'Max words in the bag'),

    # Line 256 - hint
    ('\u2014 active les options pour estimer', '\u2014 enable options to estimate'),

    # Line 260 - title
    ('Avec GPU : 0 = GPU only (recommandé, min CPU). Sans GPU : 0 = 50 % des cœurs.', 'With GPU: 0 = GPU only (recommended, min CPU). Without GPU: 0 = 50% of cores.'),

    # Line 264 - label
    ('Ignorer dust sous (sats)', 'Ignore dust below (sats)'),

    # Line 270 - buttons
    ('Lancer', 'Start'),

    # Line 277 - heading
    ('Progression', 'Progress'),

    # Lines 279-282 - stats labels
    ('Testées', 'Tested'),
    ('Variantes', 'Variants'),
    ('Vitesse totale', 'Total speed'),

    # Line 284 - title
    ('Débit par GPU et par workers CPU', 'Throughput per GPU and per CPU workers'),

    # Line 299 - heading
    ('Coffre local \u2014 clés candidates / hits', 'Local vault \u2014 candidate keys / hits'),

    # Line 301 - buttons
    ('Exporter JSON', 'Export JSON'),
    ('Vider le coffre', 'Clear vault'),

    # Line 304 - hint paragraph
    ('Stocké dans ce navigateur (localStorage).', 'Stored in this browser (localStorage).'),
    ('Boutons :', 'Buttons:'),
    ('Copier PRIV', 'Copy PRIV'),
    ('rouge,', 'red,'),
    ('Copier ADDR', 'Copy ADDR'),
    ('vert)', 'green)'),
    ('Vérifier solde', 'Check balance'),
    ('avec l\u2019<strong>adresse publique uniquement</strong>', 'with the <strong>public address only</strong>'),
    ('Copier PUB hex', 'Copy PUB hex'),
    ('Ne jamais coller une clé privée / WIF / seed dans un site web.', 'Never paste a private key / WIF / seed into a website.'),

    # Line 305 - foundList
    ('Aucune clé sauvée pour l\u2019instant.', 'No keys saved yet.'),

    # Line 314 - heading
    ('Scan auto \u2014 plage en cours', 'Auto scan \u2014 current range'),

    # Line 317 - hint
    ('Tourne en GPU dès que le scan listes est arrêté.', 'Runs on GPU as soon as list scan is stopped.'),
    ('Un scan listes (GPU) met le brute en pause pour libérer les cartes.', 'A list scan (GPU) pauses brute to free up cards.'),

    # Line 319 - range label
    ('De (départ fenêtre \u2014 hex complet)', 'From (window start \u2014 full hex)'),

    # Line 325 - range label
    ('À (fin fenêtre \u2014 hex complet)', 'To (window end \u2014 full hex)'),

    # Line 330 - label
    ('Modifier le départ (hex 64) à la main', 'Manually edit start (hex 64)'),

    # Line 332 - buttons
    ('Appliquer départ', 'Apply start'),
    ('Applique et relance le scan sur ce départ', 'Apply and restart scan at this start'),
    ('Appliquer + relancer', 'Apply + restart'),

    # Line 336 - label
    ('Pas d\u2019extension à la fin (À atteint) \u2014 défaut 2^30', 'Extension step at end (when To is reached) \u2014 default 2^30'),

    # Line 340 - button
    ('Enregistrer le pas', 'Save step'),

    # Line 343 - label
    ('Plages déjà testées (journal)', 'Ranges already tested (log)'),

    # Line 344 - hint
    ('0 plage(s)', '0 range(s)'),

    # Line 347 - range-label
    ('Journal des plages testées (ne seront pas retestées)', 'Log of tested ranges (will not be retested)'),

    # Line 352 - hint
    ('Séquentiel = fenêtre De\u2192À (pas 2^30) \u00b7 curseur live ci-dessous \u00b7 journal = pas de retest', 'Sequential = window From\u2192To (step 2^30) \u00b7 live cursor below \u00b7 log = no retest'),

    # Lines 354-359 - stats labels
    ('Clés actives (solde ou historique)', 'Active keys (balance or history)'),
    ('Dernière MAJ vitesse', 'Last speed update'),
    ('Moyenne k/s', 'Average k/s'),

    # Lines 370-376 - UTXO meta labels
    ('Bloc UTXO (index)', 'UTXO block (index)'),
    ('Retard (blocs)', 'Lag (blocks)'),
    ('Date du bloc', 'Block date'),
    ('Index généré le', 'Index generated on'),
    ('Hash du bloc', 'Block hash'),

    # Line 380 - range-label
    ('Position / curseur (hex complet)', 'Position / cursor (full hex)'),

    # Line 383 - range-label
    ('Échantillon clés threads (live)', 'Thread key sample (live)'),

    # Line 387 - range-label
    ('Variantes actives', 'Active variants'),

    # Line 394 - hint
    ('Scan de l\u2019espace des clés privées.', 'Scans the private key space.'),
    ('Chaque base key peut générer plusieurs variantes (checkboxes).', 'Each base key can generate multiple variants (checkboxes).'),

    # Line 397 - label
    ('Départ hex séquentiel (vide = 00', 'Sequential hex start (empty = 00'),
    ('ou resume)', 'or resume)'),

    # Line 399 - label
    ('% des cœurs (défaut 50 %)', '% of cores (default 50%)'),

    # Line 404 - hint
    ('\u2192 workers calculés au chargement\u2026', '\u2192 workers calculated on load\u2026'),

    # Line 407 - title
    ('0 = utiliser le % ci-dessus (recommandé). N > 0 force un nombre exact.', '0 = use the % above (recommended). N > 0 forces an exact number.'),

    # Line 408 - label
    ('Threads fixes (0 = auto %)', 'Fixed threads (0 = auto %)'),

    # Line 416 - title
    ('Décoché = séquentiel (De\u2192À clair). Coché = random (fenêtre threads).', 'Unchecked = sequential (From\u2192To clear). Checked = random (thread window).'),

    # Line 421 - label
    ('Variantes appliquées à chaque clé de base', 'Variants applied to each base key'),

    # Line 423 - checkbox
    ('identity (clé brute)', 'identity (raw key)'),

    # Lines 429-430 - checkboxes
    ('rotation gauche 8 bits', 'rotate left 8 bits'),
    ('rotation droite 8 bits', 'rotate right 8 bits'),

    # Line 434 - hint
    ('Plusieurs cases = N tests par clé (ralentit d\u2019autant). Requiert un redémarrage du scan.', 'Multiple checkboxes = N tests per key (slows down proportionally). Requires scan restart.'),

    # Line 446 - heading
    ('Idées qui maximisent les chances', 'Ideas that maximize chances'),

    # Lines 450-456 - ideas list
    ('suffixes année', 'year suffixes'),
    ('multi-chemins + plusieurs index + passphrase oubliée', 'multi-path + several indexes + forgotten passphrase'),
    ('11 mots connus + bruteforce du 12e', '11 known words + bruteforce the 12th'),
    ('Phrases perso: emails, pets, dates, villes,', 'Personal phrases: emails, pets, dates, cities,'),
    ('récupérés de vieux backups / screenshots', 'recovered from old backups / screenshots'),
    ('outils CLI du repo)', 'repo CLI tools)'),
    ('Toujours vérifier on-chain quand Core sera à jour', 'Always verify on-chain when Core is up to date'),

    # Line 473 - stats label
    ('Blocs', 'Blocks'),

    # Line 481-484 - buttons
    ('Relancer', 'Restart'),
    ('Démarrer', 'Start'),
    ('Arrêter', 'Stop'),
    ('Rafraîchir', 'Refresh'),

    # Line 532 - heading
    ('Index UTXO (pour savoir s\u2019il y a un solde)', 'UTXO Index (to check for a balance)'),

    # Lines 534-537 - stats labels
    ('Âge', 'Age'),
    ('Taille', 'Size'),
    ('Chargé', 'Loaded'),

    # Line 541 - buttons
    ('Recharger index', 'Reload index'),

    # Line 553 - footer
    ('ne partage jamais tes clés', 'never share your keys'),

    # Line 358 - keys hint
    ('= 1\u202f073\u202f741\u202f824 clés', '= 1\u202f073\u202f741\u202f824 keys'),
]

count = 0
not_found = []
for fr, en in replacements:
    if fr in content:
        content = content.replace(fr, en, 1)
        count += 1
    else:
        not_found.append(fr[:80])

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)

print(f'Done. {count} replacements made.')
if not_found:
    print(f'\n{len(not_found)} strings NOT found:')
    for s in not_found:
        print(f'  - {s}')
