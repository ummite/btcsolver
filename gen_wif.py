import hashlib, binascii

ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'

def base58encode(data):
    n = int.from_bytes(data, 'big')
    result = ''
    while n > 0:
        n, r = divmod(n, 58)
        result = ALPHABET[r] + result
    for byte in data:
        if byte == 0:
            result = '1' + result
        else:
            break
    return result

def privkey_to_wif(priv_bytes, compressed=True):
    wif_bytes = b'\x80' + priv_bytes
    if compressed:
        wif_bytes += b'\x01'
    checksum = hashlib.sha256(hashlib.sha256(wif_bytes).digest()).digest()[:4]
    return base58encode(wif_bytes + checksum)

phrases = [
    ("fff", "12cxZXUiWFTa9w7fEvTjtQ2CurYKFfmJwj", "247191 sats"),
    ("god", "1KxmSmcMTmPvU1qSLYpJLrqnSzBoQ53NXN", "61084 sats"),
    ("God", "1JJopWpJ5ZmXazk3kiisS8iVencznpnWrc", "1000 sats"),
    ("1",   "12AKRNHpFhDSBDD9rSn74VAzZSL3774PxQ", "10000 sats"),
]

print("=" * 80)
print("  CLES PRIVEES WIF - Adresses avec solde trouvees")
print("=" * 80)
print()

for phrase, addr, balance in phrases:
    priv_bytes = hashlib.sha256(phrase.encode()).digest()
    wif_uncomp = privkey_to_wif(priv_bytes, compressed=False)
    wif_comp = privkey_to_wif(priv_bytes, compressed=True)
    priv_hex = binascii.hexlify(priv_bytes).decode()

    print(f"Phrase: \"{phrase}\"")
    print(f"  Adresse:       {addr}")
    print(f"  Solde:         {balance}")
    print(f"  Cle privee:    {priv_hex}")
    print(f"  WIF (uncomp):  {wif_uncomp}  <-- A UTILISER")
    print(f"  WIF (comp):    {wif_comp}")
    print()
    print(f"  Commande Bitcoin Core:")
    print(f"    bitcoin-cli importprivkey \"{wif_uncomp}\" \"{phrase}\" true")
    print()
    print("-" * 80)
    print()
