import re

with open(r'Y:\btcsolver\static\dashboard\app.js', 'r', encoding='utf-8') as f:
    content = f.read()

# Replace the explorer link title - the French text with smart quotes
content = content.replace(
    "Ouvre blockchain.com avec l\u2019adresse PUBLIQUE uniquement \u2014 aucune cl\u00e9 priv\u00e9e n\u2019est envoy\u00e9e",
    "Opens blockchain.com with PUBLIC address only \u2014 no private key is sent"
)

with open(r'Y:\btcsolver\static\dashboard\app.js', 'w', encoding='utf-8') as f:
    f.write(content)

print("Done - explorer link title translated")
