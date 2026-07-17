#!/usr/bin/env python3
filepath = r'Y:\btcsolver\static\dashboard\index.html'
with open(filepath, 'r', encoding='utf-8') as f:
    content = f.read()

# Fix the two remaining strings with guillemet quotes
content = content.replace(
    'coffre \u00ab Cl\u00e9s trouv\u00e9es \u00bb',
    'vault "Found keys"'
)
content = content.replace(
    'Sauver les hits dans \u00ab Cl\u00e9s trouv\u00e9es \u00bb',
    'Save hits to "Found keys"'
)

with open(filepath, 'w', encoding='utf-8') as f:
    f.write(content)
print('Fixed 2 remaining strings')
