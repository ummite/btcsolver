# Brainwallet Categories - Exhaustive List

## 1. Timestamps (en cours)
- Unix ms: "1231006505000"
- Unix sec: "1231006505"
- ISO 8601: "2009-01-03T18:15:05.000Z"
- DateTime: "2009-01-03 18:15:05.000"
- Windows FILETIME: "128834823050000000"
- MAC: "1231006505.000"
- Birth dates: "1985-06-15", "15/06/1985", "06151985"
- Unix epoch of birth: ~2B combinations (1900-2010)

## 2. Mots simples (dictionnaire)
- Top 100K English words
- Top 50K French words
- BIP39 wordlist (1625 words) - déjà présent
- Single words: "freedom", "bitcoin", "money", "wealth"
- Word + number: "bitcoin1", "money2009"
- Word + year: "bitcoin2009", "freedom2010"
- Word + !: "bitcoin!", "money!!!"

## 3. Passwords courants
- Top 100K passwords (rockyou.txt style)
- "password", "123456", "12345678", "qwerty"
- "abc123", "monkey", "master", "dragon"
- "letmein", "login", "princess", "football"
- "shadow", "sunshine", "trustno1", "iloveyou"
- Variations: uppercase, with !, with year

## 4. Noms propres
- Prénoms courants: "john", "marie", "michael", "sarah"
- Noms de famille: "smith", "martin", "jones"
- Prénom + nom: "johnsmith", "john.smith", "john_smith"
- Noms de célébrités: "elonmusk", "oprah", "bezos"
- Satoshi Nakamoto: "satoshi", "nakamoto", "satoshi nakamoto"

## 5. Bitcoin/Crypto related
- "bitcoin", "btc", "satoshi", "blockchain"
- "private key", "public key", "wallet"
- "my bitcoin wallet", "first bitcoin"
- "satoshi nakamoto", "cypherpunk"
- "trust me im a doctor", "hodl"
- "to the moon", "diamond hands"
- "buy bitcoin", "invest bitcoin"
- "1 bitcoin", "one bitcoin"
- "bitcoin address", "btc address"
- "genesis", "genesis block"
- "halving", "21 million"
- "decentralized", "peer to peer"

## 6. Cités de films
- "May the Force be with you"
- "I'll be back"
- "Here's looking at you kid"
- "You talking to me"
- "Why so serious"
- "I am your father"
- "To infinity and beyond"
- "Just keep swimming"
- "Hasta la vista baby"
- "Elementary my dear Watson"
- "I am Groot"
- "After all this time Always"
- "A martian drinking rum"
- "You shall not pass"
- "I see dead people"
- "Life is like a box of chocolates"
- "My precious"
- "I am iron man"
- "Say hello to my little friend"
- "You can't handle the truth"

## 7. Paroles de chansons
- "Imagine there is no heaven"
- "Let it be"
- "Bohemian Rhapsody"
- "Stairway to heaven"
- "Hotel California"
- "Yesterday once more"
- "Nothing else matters"
- "Enter sandman"
- "Smells like teen spirit"
- "Wonderwall"
- "All you need is love"
- "Hey Jude"
- "Free bird"
- "Sweet child o mine"
- "Back in black"

## 8. Mathématiques / Constants
- Pi digits: "31415926535897932384626433832795"
- e digits: "27182818284590452353602874713527"
- Golden ratio: "16180339887498948482045868343657"
- Square root of 2: "14142135623730950488016887242097"
- "314159", "271828", "161803"
- "fibonacci", "prime", "collatz"
- "42" (Hitchhiker's Guide)
- "666", "777", "1337", "404", "200"
- "0", "1", "42", "infinity"

## 9. Nombres spéciaux
- Année de naissance: "1985", "1990", "1975"
- Date complète: "19850615", "15061985"
- Numéro de téléphone patterns
- Numéro de sécurité sociale patterns
- Numéro de carte d'identité
- Numéro de plaque d'immatriculation
- Numéro de porte: "42", "13", "7"
- Numéro de maison: "123", "456"
- Combinations: "0000", "1111", "9999", "7777"
- "1234", "12345", "123456", "1234567", "12345678"
- "111111", "222222", "333333"
- "000000", "999999"

## 10. Patterns de clavier
- "qwerty", "asdfgh", "zxcvbn"
- "qwertyuiop", "asdfghjkl", "zxcvbnm"
- "qazwsx", "1qaz2wsx"
- "zaq1xsw2"
- "abcdefghijklmnopqrstuvwxyz"
- "zyxwvutsrqponmlkjihgfedcba"
- "1234567890", "0987654321"
- "!@#$%^&*()", "~`!@#$%^&*()"

## 11. Phrases philosophiques / spirituelles
- "To be or not to be"
- "I think therefore I am"
- "Know thyself"
- "The only thing I know is that I know nothing"
- "Carpe diem"
- "Memento mori"
- "Et tu Brute"
- "Veni vidi vici"
- "Alea iacta est"
- "Timeo danaet et dona ferentes"
- "Gnothi seauton"
- "Nothing is certain except death and taxes"
- "In God we trust"
- "E pluribus unum"
- "Novus ordo seclorum"

## 12. URLs / Sites web
- "brainwallet.org"
- "brainwallet.cn"
- "bitcoin.org"
- "satoshi.nakamotoinstitute.org"
- "bitcointalk.org"
- "mtgox.com"
- "blockchain.info"
- "mywallet地址"
- "http://brainwallet.org"
- "https://bitcoin.org"

## 13. Hashs de choses simples
- MD5("password") = "5f4dcc3b5aa765d61d8327deb882cf99"
- SHA256("a") = shortest hash
- SHA256("") = empty string hash
- SHA256("0") = zero hash
- Double hash: SHA256(SHA256("bitcoin"))
- MD5 then SHA256: SHA256(MD5("password"))

## 14. Clés dérivées de mots de passe
- PBKDF2("password", "bitcoin", 1000)
- scrypt("password", "salt")
- bcrypt hash
- But ces fonctions sont lentes à brute-forcer

## 15. Nombres aléatoires "mémorables"
- 32 bytes de zéros: "0000...0000"
- 32 bytes de FF: "ffff...ffff"
- Pattern répétitif: "123412341234..."
- Alternating: "abab...abab"
- Counter: "00000001", "00000002", ...
- Sequential hex: "0123456789abcdef..."

## 16. Phrases en langues autres
- Espagnol: "bitcoin es el futuro", "dinero digital"
- Allemand: "digitales geld", "kryptowährung"
- Chinois: "比特币", "数字货币"
- Russe: "биткоин", "криптовалюта"
- Japonais: "ビットコイン"
- Arabe: "بتكوين"

## 17. Références pop culture
- "Winter is coming"
- "You know nothing Jon Snow"
- "I am the one who knocks"
- "A Lannister always pays his debts"
- "The cake is a lie"
- "All your base are belong to us"
- "It's dangerous to go alone"
- "Press F to pay respects"
- "Deal with it"
- "I am once again asking"
- "This is the way"
- "I am inevitable"
- "With great power comes great responsibility"
- "Why do we fall sir"
- "Because we learn to pick ourselves up"

## 18. Combinaisons date + mot
- "bitcoin 2009-01-03"
- "my birthday 1985-06-15"
- "first bitcoin 2009"
- "genesis 2009-01-03"
- "birthday bitcoin"
- "wedding bitcoin"

## 19. Wallet software defaults
- "electrum", "armory", "multibit"
- "bitcoin-qt", "bitcoin core"
- Default seed phrases from various wallets

## 20. Things people wrote on paper
- "my bitcoin private key"
- "do not share"
- "keep secret"
- "for my children"
- "inheritance"
- "emergency fund"
- "retirement bitcoin"
- "one million dollars"
- "financial freedom"
- "early adopter"
