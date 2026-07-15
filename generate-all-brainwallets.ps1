# Generate comprehensive brainwallet corpus
$outputFile = "Y:\btcsolver\brainwallet-all-corpus.txt"
$lines = @()

# === 1. Top passwords ===
$topPasswords = @(
    "password","123456","12345678","123456789","1234567890",
    "qwerty","abc123","monkey","1234567","letmein","trustno1",
    "dragon","baseball","iloveyou","master","sunshine","ashley",
    "bailey","shadow","123123","654321","superman","qazwsx",
    "michael","football","password1","password123","jesus",
    "ninja","mustang","111111","222222","princess","admin",
    "login","starwars","solo","passw0rd","welcome","hello",
    "charlie","donald","test","admin123","love","sex","sexylady",
    "hottie","bandit","jennifer","jessica","thomas","emily",
    "robert","access","thunder","matthew","daniel","password2",
    "000000","1qaz2wsx","zxcvbn","killer","george","hammer",
    "summer","winter","spring","autumn","flower","cookie",
    "butter","cheese","pepper","silver","golden","diamond",
    "freedom","justice","peace","money","rich","wealth",
    "bitcoin","btc","crypto","blockchain","satoshi","nakamoto",
    "wallet","private","public","key","seed","mnemonic",
    "hodl","moon","diamond","hands","tothemoon","cryptocurrency",
    "ethereum","litecoin","ripple","dogecoin","xmr","monero",
    "for my children","inheritance","emergency fund","retirement",
    "financial freedom","early adopter","one million","my secret",
    "keep secret","do not share","my bitcoin","first bitcoin",
    "genesis","genesis block","halving","21 million",
    "decentralized","peer to peer","cypherpunk","trust me"
)
foreach ($p in $topPasswords) { $lines += $p }

# Password variations
foreach ($p in $topPasswords) {
    $lines += $p.ToUpper()
    $lines += $p.Substring(0,1).ToUpper() + $p.Substring(1)
    $lines += "$p!"
    $lines += "$p!!"
    $lines += "$p!!!"
    foreach ($y in 2009..2025) { $lines += "$p$y" }
    $lines += "$p123"
    $lines += "$p1"
    $lines += "$p2009"
    $lines += "$p bitcoin"
    $lines += "$p bitcoin!"
    $lines += "bitcoin $p"
    $lines += "my $p"
    $lines += "the $p"
}

# === 2. BIP39 words (single + pairs + triples) ===
$bip39Words = Get-Content "Y:\btcsolver\bip39-words.txt" | Where-Object { $_ -and $_.Trim() }
foreach ($w in $bip39Words) {
    $w = $w.Trim()
    if ($w) { $lines += $w }
}
# Word + number combos for top 500 words
$top500 = $bip39Words | Select-Object -First 500
foreach ($w in $top500) {
    $w = $w.Trim()
    if (-not $w) { continue }
    foreach ($n in 1..10) { $lines += "$w$n" }
    $lines += "$w!"
    $lines += "$w123"
    foreach ($y in 2009..2015) { $lines += "$w$y" }
}

# === 3. Bitcoin-related phrases ===
$bitcoinPhrases = @(
    "bitcoin","btc","satoshi","nakamoto","satoshi nakamoto",
    "my bitcoin","my bitcoin wallet","first bitcoin","one bitcoin",
    "buy bitcoin","invest bitcoin","to the moon","diamond hands",
    "hodl","hODL","HODL","bitcoin forever","bitcoin is money",
    "sound money","digital gold","peer to peer","decentralized",
    "trustless","censorship resistant","permissionless",
    "open source","proof of work","hash rate","mining",
    "blockchain","distributed ledger","consensus","protocol",
    "private key","public key","seed phrase","recovery phrase",
    "wallet address","bitcoin address","btc address",
    "my private key","my secret key","my wallet key",
    "keep it secret","do not share","for my children",
    "emergency fund","retirement plan","financial freedom",
    "early adopter","bitcoin investor","bitcoin holder",
    "millionaire","billionaire","get rich","be rich",
    "money printer","free money","easy money",
    "trust no one","trust the code","code is law",
    "not your keys not your coins",
    "be your own bank","self custody","cold storage",
    "paper wallet","hardware wallet","brain wallet",
    "brainwallet","brainwallet.org","brainwallet.cn",
    "bitcointalk.org","mtgox.com","blockchain.info",
    "electrum","armory","multibit","bitcoin-qt",
    "bitcoin core","bitcoind","full node","lightning",
    "lightning network","bitcoin cash","bitcoin sv",
    "segwit","taproot","ordinals","inscriptions",
    "genesis block","halving","21 million","21000000",
    "block 0","block 1","block 478558","block 840000",
    "difficulty","nonce","merkle","hash","sha256",
    "double spend","51 percent attack","fork","soft fork",
    "hard fork","UTXO","script","op return",
    "p2pkh","p2sh","p2wpkh","p2tr","bech32",
    "wif","compressed","uncompressed","legacy",
    "testnet","mainnet","regtest","signet"
)
foreach ($p in $bitcoinPhrases) { $lines += $p }
foreach ($p in $bitcoinPhrases) {
    $lines += $p.ToUpper()
    $lines += "$p!"
    $lines += "$p123"
    foreach ($y in 2009..2015) { $lines += "$p$y" }
    $lines += "my $p"
    $lines += "the $p"
    $lines += "$p wallet"
    $lines += "$p key"
    $lines += "$p private"
    $lines += "$p bitcoin"
}

# === 4. Movie quotes ===
$movieQuotes = @(
    "May the Force be with you","I'll be back","Here's looking at you kid",
    "You talking to me","Why so serious","I am your father",
    "To infinity and beyond","Just keep swimming","Hasta la vista baby",
    "Elementary my dear Watson","I am Groot","After all this time Always",
    "You shall not pass","I see dead people",
    "Life is like a box of chocolates","My precious",
    "I am iron man","Say hello to my little friend",
    "You can't handle the truth","Here I go again",
    "I feel the need the need for speed","Nobody puts Baby in a corner",
    "Go ahead make my day","E.T. phone home",
    "Roads? Where we're going we don't need roads",
    "I'm the king of the world","Show me the money",
    "Just keep swimming find out who you are",
    "With great power comes great responsibility",
    "Why do we fall sir","Because we learn to pick ourselves up",
    "A martian drinking rum","I am the one who knocks",
    "A Lannister always pays his debts","Winter is coming",
    "You know nothing Jon Snow","The cake is a lie",
    "All your base are belong to us",
    "It's dangerous to go alone take this",
    "Press F to pay respects","Deal with it",
    "I am once again asking","This is the way",
    "I am inevitable","TARDIS","Gandalf the grey",
    "One ring to rule them all","Frodo baggins",
    "Gollum","my precious","aragorn","legolas","gimli",
    "Luke skywalker","darth vader","han solo","princess leia",
    "chewbacca","r2d2","c3po","yoda","obi wan",
    "jon snow","daenerys","tyrion","cersei","jaime",
    "ned stark","arya","sansa","bran","theon",
    "iron man","thor","hulk","captain america",
    "black widow","hawkeye","spider man","batman",
    "superman","wonder woman","flash","aquaman"
)
foreach ($q in $movieQuotes) { $lines += $q }
foreach ($q in $movieQuotes) {
    $lines += $q.ToUpper()
    $lines += $q.ToLower()
    $lines += "$q!"
    $lines += "$q123"
}

# === 5. Song lyrics ===
$songLyrics = @(
    "Imagine there is no heaven","Let it be","Bohemian Rhapsody",
    "Stairway to heaven","Hotel California","Yesterday once more",
    "Nothing else matters","Enter sandman","Smells like teen spirit",
    "Wonderwall","All you need is love","Hey Jude",
    "Free bird","Sweet child o mine","Back in black",
    "We will rock you","We are the champions","Is this the real life",
    "Somebody to love","Under pressure","Killer queen",
    "Another brick in the wall","Comfortably numb","Money money money",
    "Like a rolling stone","Blowin in the wind","What a wonderful world",
    "Imagine john lennon","The sound of silence","Hallelujah leonard",
    "Hotel california eagles","Peaceful easy feeling",
    "Born to run","dancing in the dark","glory days",
    "thriller","billie jean","beat it","smooth criminal",
    "thriller michael jackson","bad michael jackson",
    "rolling in the deep","rolling stones","pink floyd",
    "led zeppelin","queen band","beatles","abbey road",
    "lettuce be","come together","help me","twist and shout",
    "yesterday beatles","blackbird","here comes the sun",
    "happy xmas war is over","real love","free as a bird"
)
foreach ($s in $songLyrics) { $lines += $s }
foreach ($s in $songLyrics) {
    $lines += $s.ToUpper()
    $lines += "$s!"
    $lines += "$s123"
}

# === 6. Mathematical constants ===
$mathConstants = @(
    "31415926535897932384626433832795",
    "27182818284590452353602874713527",
    "16180339887498948482045868343657",
    "14142135623730950488016887242097",
    "314159","271828","161803","141421",
    "pi","euler","golden ratio","fibonacci",
    "42","666","777","1337","404","200",
    "0","1","42","infinity","null","void",
    "00000000000000000000000000000000",
    "ffffffffffffffffffffffffffffffff",
    "0123456789abcdef0123456789abcdef",
    "deadbeef","cafebabe","face","babe",
    "1234567890abcdef1234567890abcdef",
    "abcdef1234567890abcdef1234567890",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "cccccccccccccccccccccccccccccccc"
)
foreach ($m in $mathConstants) { $lines += $m }

# === 7. Famous numbers ===
$famousNumbers = @(
    "42","666","777","1337","404","200","500",
    "314","271","161","141","911","999","000",
    "111","222","333","444","555","888",
    "1234","12345","123456","1234567","12345678","123456789","1234567890",
    "1111","2222","3333","4444","5555","6666","7777","8888","9999","0000",
    "111111","222222","333333","444444","555555",
    "10000","20000","50000","100000","1000000",
    "21000000","2100000000000000",
    "100","200","300","500","1000","2000","3000","5000","10000"
)
foreach ($n in $famousNumbers) { $lines += $n }

# === 8. Keyboard patterns ===
$keyboardPatterns = @(
    "qwerty","asdfgh","zxcvbn","qwertyuiop","asdfghjkl","zxcvbnm",
    "qazwsx","1qaz2wsx","zaq1xsw2","qweasdzxc",
    "abcdefghijklmnopqrstuvwxyz","zyxwvutsrqponmlkjihgfedcba",
    "1234567890","0987654321","12345678901234567890",
    "!@#$%^&*()","~`!@#$%^&*()",
    "qwer","asdf","zxcv","qwertz","azerty",
    "dvorak","colemak","workman",
    "1q2w3e","1q2w3e4r","1q2w3e4r5t",
    "a1b2c3","a1s2d3","z1x2c3",
    "qwerty123","asdfgh123","zxcvbn123",
    "password1","qwerty1","abc1234",
    "iloveyou","iloveyou1","iloveyou123",
    "trustno1","letmein1","admin123",
    "changeme","change","default","test123",
    "temp","tmp","guest","user","root"
)
foreach ($k in $keyboardPatterns) { $lines += $k }
foreach ($k in $keyboardPatterns) {
    $lines += $k.ToUpper()
    $lines += "$k!"
    $lines += "$k123"
}

# === 9. Philosophical quotes ===
$philosophicalQuotes = @(
    "To be or not to be","I think therefore I am","Know thyself",
    "The only thing I know is that I know nothing","Carpe diem",
    "Memento mori","Et tu Brute","Veni vidi vici","Alea iacta est",
    "Gnothi seauton","Nothing is certain except death and taxes",
    "In God we trust","E pluribus unum","Novus ordo seclorum",
    "Liberty equality fraternity","Life liberty pursuit of happiness",
    "Give me liberty or give me death","I have a dream",
    "That government governs best which governs least",
    "Power tends to corrupt and absolute power corrupts absolutely",
    "The unexamined life is not worth living","Man is condemned to be free",
    "Existence precedes essence","God is dead","Will to power",
    "Eternal recurrence","Ubermensch","Nietzsche",
    "Descartes","Kant","Hegel","Marx","Plato","Aristotle",
    "Socrates","Confucius","Buddha","Lao tzu",
    "Be here now","The art of war","Sun tzu",
    "Know yourself and know your enemy",
    "The journey of a thousand miles begins with one step",
    "When I let go of what I am I become what I might be",
    "The mind is everything what you think you become",
    "Turn your wounds into wisdom","What does not kill me makes me stronger"
)
foreach ($q in $philosophicalQuotes) { $lines += $q }
foreach ($q in $philosophicalQuotes) {
    $lines += $q.ToUpper()
    $lines += $q.ToLower()
    $lines += "$q!"
}

# === 10. URLs ===
$urls = @(
    "brainwallet.org","brainwallet.cn","bitcoin.org",
    "satoshi.nakamotoinstitute.org","bitcointalk.org",
    "mtgox.com","blockchain.info","blockchain.com",
    "electrum.org","armory","multibit.org",
    "coinbase.com","kraken.com","binance.com",
    "gemini.com","bitfinex.com","bittrex.com",
    "poloniex.com","hitbtc.com","kucoin.com",
    "mywallet","mywallet.com","bitcoinwallet",
    "http://brainwallet.org","https://bitcoin.org",
    "www.bitcoin.org","www.blockchain.info",
    "github.com/bitcoin/bitcoin","bitcoin.it",
    "bitcoinwhitepaper","bitcoin pdf","nakamoto pdf"
)
foreach ($u in $urls) { $lines += $u }
foreach ($u in $urls) {
    $lines += "$u!"
    $lines += "$u123"
    $lines += "$u wallet"
    $lines += "$u key"
}

# === 11. Simple hashes of common strings ===
# MD5 of common strings (used as brainwallets)
$md5Hashes = @(
    "5f4dcc3b5aa765d61d8327deb882cf99",  # MD5("password")
    "e10adc3949ba59abbe56e057f20f883e",  # MD5("123456")
    "7c6a180b36896a65c4a28dd41833be9e",  # MD5("1234567")
    "25d55ad283aa400af464c76d713c07ad",  # MD5("12345678")
    "e99a18c428cb38d5f260853678922e03",  # MD5("abc123")
    "b1b3773a05c0ed0176787a4f1574ff00",  # MD5("test")
    "098f6bcd4621d373cade4e832627b4f6",  # MD5("test" with different case)
    "5d41402abc4b2a76b9719d911017c592",  # MD5("hello")
    "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8",  # SHA256("password")
    "65e84be33532fb784c48129675f9eff3a682b27168c0ea744b2cf58ee02337c5",  # SHA256("123456")
    "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b",  # SHA256("")
    "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",  # SHA256("1")
    "6c32683756220e679677781568521845f0a5c4c28bfbf2d4f9b4d8c0b0a5e123"   # SHA256("a")
)
foreach ($h in $md5Hashes) { $lines += $h }

# === 12. Foreign language phrases ===
$foreignPhrases = @(
    # Spanish
    "bitcoin es el futuro","dinero digital","libertad financiera",
    "criptomoneda","billetera digital","clave privada",
    "mi bitcoin","primer bitcoin","bitcoin para siempre",
    "el futuro del dinero","fondo de emergencia",
    # German
    "digitales geld","kryptowährung","bitcoin wallet",
    "privater schlüssel","finanzielle freiheit",
    "die zukunft des geldes","kryptowährung investieren",
    # Italian
    "denaro digitale","libertà finanziaria","bitcoin per sempre",
    "il futuro del denaro","chiave privata",
    # Portuguese
    "dinheiro digital","liberdade financeira","criptomoeda",
    "carteira digital","chave privada",
    # Russian (transliterated)
    "bitkoin","kriptovalyuta","privatnyy klyuch",
    "tsifrovye dengi","finansovaya svoboda",
    # Dutch
    "digitaal geld","cryptocurrency","bitcoin portemonnee",
    "financiële vrijheid","privésleutel"
)
foreach ($f in $foreignPhrases) { $lines += $f }
foreach ($f in $foreignPhrases) {
    $lines += $f.ToUpper()
    $lines += "$f!"
    $lines += "$f123"
}

# === 13. Pop culture ===
$popCulture = @(
    "Winter is coming","You know nothing Jon Snow",
    "I am the one who knocks","A Lannister always pays his debts",
    "The cake is a lie","All your base are belong to us",
    "It's dangerous to go alone","Press F to pay respects",
    "Deal with it","I am once again asking",
    "This is the way","I am inevitable",
    "The internet is for porn","I am Groot",
    "Wakanda forever","Avengers assemble",
    "May the odds be ever in your favor",
    "I volunteer as tribute","Katniss everdeen",
    "Hodor","Hodor hodor","Moreado",
    "Valar morghulis","Valar dohaeris","Azor ahai",
    "Fire and blood","Dracarys","Targaryen",
    "Stark","Lannister","Baratheon","Tully","Tyrell",
    "Game of thrones","Breaking bad","Better call Saul",
    "Walter white","Jesse pinkman","Heisenberg",
    "Say my name","I am the danger",
    "The name is white","Plutonium","Crystal meth",
    "Stranger things","Eleven","Mike wheeler",
    "Friends","Ross geller","Rachel green",
    "How you doin","Pivot pivot","The one where",
    "The office","Michael scott","Dwight schrute",
    "That's what she said","Bears strikes and cars",
    "Parkour","Just do it","Think different",
    "I'm lovin it","Finger lickin good",
    "Just because you're paranoid","doesn't mean","they're not after you"
)
foreach ($p in $popCulture) { $lines += $p }
foreach ($p in $popCulture) {
    $lines += $p.ToUpper()
    $lines += "$p!"
    $lines += "$p123"
}

# === 14. Common date patterns ===
# Birth year + common numbers
foreach ($y in 1950..2010) {
    $lines += "$y"
    $lines += "bitcoin$y"
    $lines += "btc$y"
    $lines += "born$y"
    $lines += "birthday$y"
    # Common dates
    foreach ($m in @(1,2,3,4,5,6,7,8,9,10,11,12)) {
        foreach ($d in @(1,2,3,4,5,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28)) {
            $lines += "$y${m}1${d}"
            $lines += "$y$($m.ToString('D2'))$($d.ToString('D2'))"
        }
    }
}

# === 15. Double/triple hashes ===
$doubleHashBases = @("password","bitcoin","123456","hello","test","abc","god","love","freedom","money")
foreach ($base in $doubleHashBases) {
    # SHA256(SHA256(base))
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($base)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $hash1 = $sha.ComputeHash($bytes)
    $hash2 = $sha.ComputeHash($hash1)
    $hex = -join ($hash2 | ForEach-Object { $_.ToString("x2") })
    $lines += $hex

    # MD5 then SHA256
    $md5 = [System.Security.Cryptography.MD5]::Create()
    $md5hash = $md5.ComputeHash($bytes)
    $shaMd5 = $sha.ComputeHash($md5hash)
    $hex2 = -join ($shaMd5 | ForEach-Object { $_.ToString("x2") })
    $lines += $hex2
}

# === Deduplicate and write ===
$unique = $lines | Where-Object { $_ -and $_.Trim() } | Sort-Object -Unique
Write-Host "Total unique patterns: $($unique.Count)"
$unique | Out-File -FilePath $outputFile -Encoding UTF8
Write-Host "Written to $outputFile"
