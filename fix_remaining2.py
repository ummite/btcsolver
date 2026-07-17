#!/usr/bin/env python3
"""Fix remaining French text in the dashboard HTML."""
import re
filepath = r'Y:\btcsolver\static\dashboard\index.html'
with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Line 73 - bloc -> block
content = content.replace(
    'title="Exact UTXO index height (no K shortcut)">bloc \u2014</span>',
    'title="Exact UTXO index height (no K shortcut)">block \u2014</span>',
    1
)

# Line 158 - puis hash. -> then hash.
content = content.replace(
    '\u2192 \u201chelloworld\u201d puis hash.',
    '\u2192 \u201chelloworld\u201d then hash.',
    1
)

# Line 159 - et aussi -> and also
content = content.replace(
    '\u2192 \u201chi world\u201d et aussi',
    '\u2192 \u201chi world\u201d and also',
    1
)

# Line 193 - variantes -> variants
content = content.replace(
    'Each line \u2192 variantes + hashes',
    'Each line \u2192 variants + hashes',
    1
)

# Line 207 - clé privée -> private key
content = content.replace(
    'SHA-256 hash of each phrase \u2192 clé privée.',
    'SHA-256 hash of each phrase \u2192 private key.',
    1
)

# Line 208 - Double SHA-256 : -> Double SHA-256: and clé privée
content = content.replace(
    'Double SHA-256 : SHA256(SHA256(phrase)) \u2192 clé privée.',
    'Double SHA-256: SHA256(SHA256(phrase)) \u2192 private key.',
    1
)

# Line 216 - Enlève -> Removes
content = content.replace(
    "Enlève . , ; ! ? ' &quot; @ # $ % \u2026 Keeps letters/digits/spaces.",
    "Removes . , ; ! ? ' &quot; @ # $ % \u2026 Keeps letters/digits/spaces.",
    1
)

# Line 216 - Sans symboles -> No symbols (use regex to avoid paren issues)
content = re.sub(r'Sans symboles ', 'No symbols ', content)

# Line 217/249 - Sans espaces -> No spaces (in dict section, both occurrences)
content = content.replace('/> Sans espaces</label>', '/> No spaces</label>')

# Line 219 - Pr\u00e9fixes -> Prefixes
content = content.replace('checked /> Pr\u00e9fixes</label>', 'checked /> Prefixes</label>')

# Line 222 - Longueur -> Length
content = content.replace(
    '\u2026 Longueur <strong>2</strong> = 94',
    '\u2026 Length <strong>2</strong> = 94',
    1
)

# Lines 229, 232 - 1 caract\u00e8re / 2 caract\u00e8res -> 1 character / 2 characters
content = content.replace('>1 caract\u00e8re</option>', '>1 character</option>')
content = content.replace('>2 caract\u00e8res</option>', '>2 characters</option>')

# Line 245 - et -> and (in tooltip)
content = content.replace(
    '\u2192 \u00ab alice bob \u00bb et \u00ab bob alice \u00bb.',
    '\u2192 \u00ab alice bob \u00bb and \u00ab bob alice \u00bb.',
    1
)

# Line 304 - ouvre -> opens
content = content.replace(
    '<strong>Check balance \u2197</strong> (ouvre',
    '<strong>Check balance \u2197</strong> (opens',
    1
)

# Lines 321, 329 - Copier -> Copy
content = content.replace('style="display:none">Copier</button>', 'style="display:none">Copy</button>')

# Line 362 - Test\u00e9es -> Tested
content = content.replace(
    '<span class="stat-label">Test\u00e9es</span><span class="stat-value mono" id="keysTested">',
    '<span class="stat-label">Tested</span><span class="stat-value mono" id="keysTested">',
    1
)

# Line 431 - SHA256(cl\u00e9) -> SHA256(key)
content = content.replace('/> SHA256(cl\u00e9)</label>', '/> SHA256(key)</label>')

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)

print('All remaining French text fixed.')
