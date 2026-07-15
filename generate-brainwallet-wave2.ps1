# Wave 2: Additional brainwallet categories
$outputFile = "Y:\btcsolver\brainwallet-wave2-corpus.txt"
$lines = @()

# === 1. Famous book quotes / Literature ===
$bookQuotes = @(
    "It was the best of times it was the worst of times",
    "Call me Ishmael",
    "It was a bright cold day in April and the clocks were striking thirteen",
    "All happy families are alike every unhappy family is unhappy in its own way",
    "In a hole in the ground there lived a hobbit",
    "Alice was beginning to get very tired of sitting by her sister",
    "It is a truth universally acknowledged that a single man in possession of a good fortune",
    "Many years later as he faced the firing squad Colonel Aureliano Buendia",
    "The sky above the port was the color of television tuned to a dead channel",
    "Last night I dreamt I went to Manderley again",
    "Marley was dead to begin with",
    "As Gregor Samsa awoke one morning from uneasy dreams he found himself transformed",
    "The man in black fled across the desert and the man in black was right behind him",
    "A boy is not a burden on your hand but a prayer on your heart",
    "So we beat on boats against the current borne back ceaselessly into the past",
    "You do not understand I could not live without you",
    "All children unless they are Damned are full of grace",
    "I am invisible do you see me do you see me through me",
    "The past is a foreign country they do things differently there",
    "There is no friend as loyal as a book"
)
foreach ($q in $bookQuotes) {
    $lines += $q
    $lines += $q.ToLower()
    $lines += $q.ToUpper()
    $lines += "$q!"
}

# Book titles
$bookTitles = @(
    "1984", "Brave new world", "Fahrenheit 451", "The hobbit",
    "Harry potter", "The great gatsby", "To kill a mockingbird",
    "Lord of the flies", "Animal farm", "The catcher in the rye",
    "Wuthering heights", "Jane eyre", "Pride and prejudice",
    "The lord of the rings", "The silence of the lambs",
    "Gone girl", "The Da Vinci code", "Twilight",
    "The hunger games", "Divergent", "The maze runner",
    "Dune", "Foundation", "Ender game", "Neuromancer",
    "Snow crash", "Ready player one", "The matrix"
)
foreach ($t in $bookTitles) {
    $lines += $t
    $lines += $t.ToUpper()
    $lines += "$t!"
    $lines += "$t123"
    foreach ($y in 2009..2015) { $lines += "$t$y" }
}

# === 2. Video games ===
$videoGames = @(
    "master chief", "link zelda", "mario bross", "sonic hedgehog",
    "get over here mario", "it is dangerous to go alone",
    "wolfenstein", "doom", "quake", "half life",
    "gordon freeman", "glados", "portal", "aperture",
    "warp nine", "respawn", "level 99", "game over",
    "insert coin", "press start", "game over man game over",
    "ight imma head out", "press x to believe",
    "world of warcraft", "ahora se muere", "loot",
    "minecraft", "notch", "creeper", "diamond sword",
    "the quick brown fox jumps over the lazy dog",
    "tetris", "pacman", "space invaders", "galaga",
    "street fighter", "ryu", "ken", "hadouken",
    "mortal kombat", "fatality", "get over here",
    "dark souls", "prepare to die", "bonfire",
    "elden ring", "malenia", "margit",
    "skyrim", "fus ro dah", "dragonborn",
    "fallout", "wasteland", "pip boy",
    "bioshock", "would you kindly", "rapture",
    "red dead redemption", "arthur morgan",
    "god of war", "kratos", "atreus",
    "horizon", "aloy", "shadow of the colossus",
    "the last of us", "joel", "elly",
    "resident evil", "leon kennedy", "claire redfield",
    "silent hill", "alarm", "pyramid head",
    "bloodborne", "hunter", "yamato",
    "dark souls prepare to die",
    "may the odds be ever in your favor",
    "i volunteer as tribute",
    "to be or not to be that is the question",
    "here there be dragons",
    "all your base are belong to us",
    "press f to pay respects",
    "deal with it",
    "this is the way",
    "i am inevitable",
    "winter is coming",
    "you know nothing jon snow",
    "a lannister always pays his debts",
    "the cake is a lie",
    "not all those who wander are lost",
    "one does not simply walk into mordor",
    "i am gin",
    "braveheart",
    "freedom",
    "burn witch burn",
    "the only thing we have to fear is fear itself",
    "after all this time always",
    "i am the bone in your neck",
    "i am the king of the world",
    "show me the money",
    "you had me at hello",
    "just keep swimming",
    "a wise man once said if you want something done right do it yourself",
    "i feel the need the need for speed",
    "nobody puts baby in a corner",
    "go ahead make my day",
    "et phone home",
    "roads where were going we dont need roads",
    "i see dead people",
    "you talking to me",
    "why so serious",
    "houston we have a problem",
    "there is no spoon",
    "follow the white rabbit",
    "red pill blue pill",
    "wake up neo",
    "the matrix has you",
    "knock knock neo",
    "i know kung fu",
    "elementary my dear watson",
    "i am groot",
    "wakanda forever",
    "avengers assemble",
    "with great power comes great responsibility",
    "why do we fall sir because we learn to pick ourselves up",
    "my precious",
    "ash nazg durbatulak",
    "the one ring",
    "fellowship of the ring",
    "aragorn son of arathorn",
    "and lothiriel was her name",
    "a wizard is never late nor is he early he arrives precisely when he means to",
    "i would rather be happy than right",
    "the world is not enough",
    "shaken not stirred",
    "license to kill",
    "goldeneye",
    "casino royale",
    "skyfall",
    "spectre",
    "no time to die",
    "james bond",
    "007",
    "mission impossible",
    "ethan hunt",
    "top gun",
    "maverick",
    "i feel the need"
)
foreach ($g in $videoGames) {
    $lines += $g
    $lines += $g.ToLower()
    $lines += $g.ToUpper()
    $lines += "$g!"
    $lines += "$g123"
}

# === 3. Leet speak variations ===
$leetPasswords = @(
    "p@ssw0rd", "h4ck3r", "br@1nw4ll3t", "b1tc01n",
    "m0n3y", "w4ll3t", "s3cr3t", "k3y",
    "pr1v4t3", "m0nk3y", "dr4g0n", "m4st3r",
    "sunsh1ne", "il0vey0u", "l3tm31n",
    "p@ssword", "p@ssw0rd1", "p@ssw0rd123",
    "h3ll0", "h3ll0 w0rld", "h3ll0 b1tc01n",
    "b1tc01n w4ll3t", "b1tc01n k3y",
    "s3cr3t k3y", "pr1v4t3 k3y",
    "m0n3y m0n3y m0n3y", "r1ch r1ch r1ch",
    "frr33d0m", "j0st1c3", "p34c3",
    "4dm1n", "r00t", "us3r", "gu3st",
    "t3mp", "d3f4ult", "ch4ng3m3",
    "w3lc0m3", "w3lc0m31", "w3lc0m3!",
    "s3cur3", "s4f3", "pr0t3ct3d",
    "3ncrypt3d", "d3crypt3d", "43s256",
    "sh4256", "md5", "h4sh", "bl0ckch41n",
    "cryPt0", "d3c3ntr4l1z3d", "p2p",
    "h0dl", "t0th3m00n", "d14m0ndh4nds",
    "s4t0sh1", "n4k4m0t0", "g3n3s1s",
    "m1n3r", "m1n1ng", "h4shr4t3",
    "n0d3", "w4ll3t", "tx", "utx0",
    "0xf4d3", "0xd34d", "0xc4f3",
    "c4f3b4b3", "d34db33f", "b4dc0d3"
)
foreach ($l in $leetPasswords) { $lines += $l }

# === 4. Famous people / Historical figures ===
$famousPeople = @(
    "albert einstein", "isaac newton", "nikola tesla",
    "leonardo da vinci", "william shakespeare", "charlie chaplin",
    "john lennon", "bob dylan", "elvis presley",
    "martin luther king", "mahatma gandhi", "nelson mandela",
    "adolf hitler", "joseph stalin", "cleopatra",
    "alexander the great", "julius caesar", "napoleon",
    "william gates", "steve jobs", "mark zuckerberg",
    "jeff bezos", "bill gates", "warren buffett",
    "elon musk", "jeff bezos", "bernie sanders",
    "oprah winfrey", "barack obama", "donald trump",
    "hillary clinton", "john f kennedy", "roald dahl",
    "agatha christie", "j k rowling", "stephen king",
    "george r r martin", "j r r tolkien", "c s lewis",
    "mark twain", "oscar wilde", "f scott fitzgerald",
    "ernest hemingway", "frank sinatra", "paul mccartney",
    "michael jackson", "prince", "david bowie",
    "freddie mercury", "kurt cobain", "jim morrison",
    "bob marley", "johnny cash", "willie nelson"
)
foreach ($p in $famousPeople) {
    $lines += $p
    $lines += $p.Replace(" ", "")
    $lines += $p.ToUpper()
    $lines += "$p!"
    $lines += "$p123"
}

# === 5. Animals ===
$animals = @(
    "cat", "dog", "horse", "eagle", "wolf",
    "dragon", "phoenix", "unicorn", "griffin",
    "panther", "tiger", "lion", "bear", "shark",
    "butterfly", "dolphin", "penguin", "turtle",
    "falcon", "cobra", "viper", "scorpion",
    "badger", "fox", "raccoon", "squirrel",
    "owl", "raven", "crow", "hawk", "eagle",
    "whale", "octopus", "jellyfish", "crab",
    "lobster", "salmon", "trout", "tuna",
    "elephant", "giraffe", "zebra", "gorilla",
    "chimpanzee", "monkey", "ape", "lemur",
    "kangaroo", "koala", "panda", "red panda",
    "polar bear", "grizzly", "cougar", "jaguar",
    "cheetah", "leopard", "hyena", "jackal",
    "wolves", "husky", "shepherd", "bulldog",
    "tabby", "siamese", "persian", "maine coon"
)
foreach ($a in $animals) {
    $lines += $a
    $lines += $a.ToUpper()
    $lines += "$a!"
    $lines += "$a123"
    $lines += "my $a"
    $lines += "the $a"
    foreach ($y in 2009..2015) { $lines += "$a$y" }
    # Animal + color combos
    foreach ($c in "black", "white", "red", "blue", "golden", "silver", "green") {
        $lines += "$c $a"
        $lines += "$c$a"
    }
}

# === 6. Bitcoin Whitepaper quotes ===
$whitepaperQuotes = @(
    "A purely peer-to-peer version of electronic cash would be possible",
    "The timestamp server works by taking a hash of a block of items",
    "We consider the case of a person wanting to pay another person",
    "Transactions must be public to prevent double spending",
    "We define an electronic coin as a chain of digital signatures",
    "Each owner verifies the previous transactions",
    "Double spending",
    "Proof-of-work",
    "Incentive",
    "Reclaiming disk space",
    "Simplified payment verification",
    "Combining and splitting value",
    "Privacy",
    "Calculations",
    "A purely peer-to-peer version of electronic cash",
    "without relying on trust",
    "electronically signed chain of ownership",
    "unspent transaction output",
    "merkle tree",
    "longest chain",
    "network as proof of processing power",
    "the honest nodes always outnumber the attackers",
    "bitcoin whitepaper",
    "satoshi nakamoto whitepaper",
    "bitcoin paper",
    "peer to peer electronic cash system"
)
foreach ($w in $whitepaperQuotes) {
    $lines += $w
    $lines += $w.ToLower()
    $lines += $w.ToUpper()
    $lines += "$w!"
}

# === 7. Linux / Terminal commands ===
$linuxCommands = @(
    "sudo rm -rf /", "chmod 777", "ls -la",
    "mkdir bitcoin", "cd bitcoin", "touch wallet",
    "cat /etc/passwd", "whoami", "uname -a",
    "git commit", "git push", "git clone",
    "ssh root@localhost", "ping google.com",
    "wget bitcoin.org", "curl bitcoin.org",
    "tar -xzf", "gzip", "zip", "unzip",
    "nano", "vim", "emacs", "gedit",
    "gcc", "make", "cmake", "cargo build",
    "python", "python3", "ruby", "node",
    "npm install", "pip install", "apt install",
    "yum install", "brew install", "choco install",
    "docker", "docker run", "docker build",
    "kubectl", "helm", "terraform",
    "aws s3", "azure", "gcloud"
)
foreach ($c in $linuxCommands) {
    $lines += $c
    $lines += "$c!"
    $lines += "$c123"
}

# === 8. Programming ===
$programming = @(
    "hello world", "hello world!", "Hello World",
    "console.log", "print hello", "printf",
    "def main", "func main", "public static void main",
    "return 0", "exit 0", "exit code",
    "null pointer", "segmentation fault",
    "stackoverflow", "github", "gitlab",
    "for i in range", "while true", "if else",
    "switch case", "try catch", "throw new",
    "import sys", "from os import", "require",
    "class", "interface", "abstract", "enum",
    "array", "list", "map", "set", "dict",
    "string", "integer", "float", "boolean",
    "void", "null", "undefined", "nan",
    "true", "false", "yes", "no",
    "begin", "end", "start", "stop",
    "open", "close", "read", "write",
    "create", "delete", "update", "insert",
    "select", "from", "where", "join",
    "drop table", "alter table", "create table",
    "primary key", "foreign key", "index",
    "sql", "mysql", "postgres", "mongodb",
    "redis", "elasticsearch", "kafka",
    "api", "rest", "graphql", "soap",
    "json", "xml", "yaml", "toml",
    "html", "css", "javascript", "typescript",
    "rust", "go", "python", "java", "c++",
    "c#", "swift", "kotlin", "dart", "flutter",
    "react", "angular", "vue", "svelte",
    "nodejs", "express", "django", "flask",
    "spring", "hibernate", "laravel", "rails",
    "webpack", "babel", "eslint", "prettier",
    "jest", "mocha", "pytest", "unittest"
)
foreach ($p in $programming) {
    $lines += $p
    $lines += $p.ToUpper()
    $lines += "$p!"
    $lines += "$p123"
}

# === 9. Music artists / Albums ===
$musicArtists = @(
    "metallica", "nirvana", "radiohead", "pink floyd",
    "led zeppelin", "the beatles", "queen", "abbey road",
    "dark side of the moon", "rumours", "thriller",
    "back in black", "nevermind", "ok computer",
    "master of puppets", "ride the lightning",
    "kill em all", "metallica black album",
    "enter sandman", "nothing else matters",
    "one metallica", "master of puppets",
    "fade to black", "for whom the bell tolls",
    "the trooper", "seek and destroy",
    "cream", "electric ladyland", "white room",
    "jean luc ponty", "mahavishnu orchestra",
    "deep purple", "machine head", "smoke on the water",
    "black sabbath", "paranoid", "war pigs",
    "iron maiden", "the number of the beast",
    "judas priest", "british steel",
    "slayer", "reign in blood",
    "megadeth", "rust in peace",
    "anthrax", "among the living",
    "gun n roses", "appetite for destruction",
    "sweet child o mine", "paradise city",
    "aerosmith", "dream on", "sweet emmeline",
    "bon jovi", "livin on a prayer", "wont stop believing",
    "journey", "dont stop believin", "open arms",
    "revelation", "carry on wayward son",
    "stoned", "walk like an eagle",
    "tears for fears", "everybody wants to rule the world",
    "talking heads", "once in a lifetime",
    "the police", "every breath you take",
    "sting", "message in a bottle",
    "phil collins", "in the air tonight",
    "genesis", "land of confusion",
    "dire straits", "money for nothing",
    "the rolling stones", "paint it black",
    "satisfaction", "gimme shelter",
    "jim i hear you calling",
    "creedence clearwater revival",
    "fortunate son", "bad moon rising",
    "have you ever seen the rain",
    "the doors", "light my fire",
    "break on through", "ride the lightning",
    "jim morrison", "nightcrawler",
    "the eternal knight",
    "born to be wild", "steppenwolf",
    "free bird", "lynyrd skynyrd",
    "sweet home alabama",
    "all along the watchtower",
    "jimi hendrix", "purple haze",
    "windy", "all along the watchtower",
    "electric ladyland",
    "are you experienced",
    "axis bold as love",
    "little wing",
    "hey joe",
    "the wind cries mary",
    "voodoo child",
    "red house",
    "foxey lady",
    "casting bones",
    "third universe",
    "bold as love"
)
foreach ($m in $musicArtists) {
    $lines += $m
    $lines += $m.ToLower()
    $lines += $m.ToUpper()
    $lines += "$m!"
    $lines += "$m123"
}

# === 10. Religion / Spirituality (non-Bible) ===
$religion = @(
    "bismillah", "in the name of Allah",
    "allah akbar", "subhanallah", "alhamdulillah",
    "om mani padme hum", "namaste", "karma",
    "om shanti", "ganesha", "krishna",
    "om namah shivaya", "sat nam wahe guru",
    "the lord is my shepherd", "hail mary",
    "rosary", "crucifix", "vatican",
    "dalai lama", "tibetan buddhism",
    "zen", "koan", "satori",
    "enlightenment", "nirvana", "dharma",
    "samsara", "reincarnation", "chakra",
    "third eye", "third eye", "pineal gland",
    "crystal healing", "aura", "chakra",
    "meditation", "mindfulness", "yoga",
    "pranayama", "asana", "namaste",
    "the power of now", " Eckhart tolle",
    "the secret", "law of attraction",
    "abundance", "manifestation", "visualization",
    "positive thinking", "affirmation",
    "i am that i am", "the divine",
    "universal consciousness", "source energy",
    "higher self", "spirit guide",
    "angel numbers", "1111", "444", "777",
    "twin flame", "soul mate", "karmic debt",
    "past life", "akasha", "akashic records",
    "reiki", "pranayama", "kundalini",
    "shambhala", "atlantis", "lemuria",
    "rosetta stone", "dead sea scrolls",
    "book of dead", "book of kells",
    "talmud", "torah", "zohar",
    "quran", "hadith", "sunnah",
    "vedas", "upanishads", "gita",
    "tripitaka", "sutras", "dhammapada",
    "tao te ching", "taoism", "feng shui",
    "yin yang", "bagua", "i ching"
)
foreach ($r in $religion) {
    $lines += $r
    $lines += $r.ToLower()
    $lines += $r.ToUpper()
    $lines += "$r!"
}

# === 11. Historical dates ===
$historicalDates = @(
    "1776", "1492", "1914", "1918",
    "1939", "1945", "1969", "1989",
    "2001", "2008", "2009",
    "january 3rd 2009", "1/3/2009", "01/03/2009", "2009-01-03",
    "september 11 2001", "9/11/2001", "09/11/2001", "2001-09-11",
    "december 7 1941", "12/7/1941", "07/12/1941", "1941-12-07",
    "july 4 1776", "7/4/1776", "04/07/1776", "1776-07-04",
    "november 9 1989", "11/9/1989", "09/11/1989", "1989-11-09",
    "august 6 1945", "8/6/1945", "06/08/1945", "1945-08-06",
    "august 9 1945", "8/9/1945", "09/08/1945", "1945-08-09",
    "november 11 1918", "11/11/1918", "1918-11-11",
    "june 6 1944", "6/6/1944", "06/06/1944", "1944-06-06",
    "april 12 1961", "4/12/1961", "1961-04-12",
    "july 20 1969", "7/20/1969", "1969-07-20",
    "january 20 2009", "1/20/2009", "2009-01-20",
    "october 29 2008", "10/29/2008", "2008-10-29",
    "march 10 2013", "3/10/2013", "2013-03-10",
    "may 22 2010", "5/22/2010", "2010-05-22",
    "october 5 2009", "10/5/2009", "2009-10-05",
    "bitcoin genesis", "genesis block",
    "the chicago tribune 03/jan/2009",
    "chancellor on brink of second bailout for banks",
    "times 03jan2009 chancellor on brink of second bailout for banks"
)
foreach ($d in $historicalDates) {
    $lines += $d
    $lines += $d.ToUpper()
    $lines += "$d!"
}

# === 12. File paths / Filenames ===
$filePaths = @(
    "readme.txt", "todo.txt", "notes.txt",
    "private.key", "wallet.dat", "bitcoin.wallet",
    "home/user/bitcoin", "users/bitcoin",
    "desktop/wallet", "documents/keys",
    "my private key", "my wallet",
    "secret key", "private key",
    "bitcoin key", "btc key",
    "wallet backup", "wallet recovery",
    "seed phrase", "recovery phrase",
    "mnemonic phrase", "backup phrase",
    "twelve words", "twenty four words",
    "wallet import", "wallet export",
    "key import", "key export",
    "wif key", "wif private key",
    "compressed key", "uncompressed key",
    "hex key", "base58 key",
    "bech32 address", "legacy address",
    "p2pkh address", "p2sh address",
    "p2wpkh address", "p2tr address",
    "taproot address", "segwit address",
    "native segwit", "wrapped segwit"
)
foreach ($f in $filePaths) {
    $lines += $f
    $lines += $f.ToUpper()
    $lines += "$f!"
}

# === 13. "I remember" type phrases ===
$memoryPhrases = @(
    "the password is", "my key is",
    "remember this", "never forget",
    "i will never forget this",
    "this is my key", "this is my secret",
    "only i know this", "nobody knows this",
    "secret password 2009",
    "my secret is", "my password is",
    "dont forget this", "write this down",
    "keep this safe", "store this safely",
    "this changes everything",
    "the future is here",
    "digital revolution",
    "money will never be the same",
    "the end of banks",
    "freedom from banks",
    "no more banks",
    "banking 2.0",
    "internet money",
    "online currency",
    "virtual currency",
    "virtual money",
    "cyber money",
    "e-cash", "ecash",
    "digital cash",
    "electronic money",
    "internet cash",
    "web money",
    "net money",
    "crypto money",
    "crypto cash",
    "bitcoin cash",
    "litecoin", "dogecoin",
    "namecoin", "peercoin",
    "primcoin", "feathercoin",
    "nova coin", "worldcoin",
    "freecoin", "memecoin",
    "yacoin", "auroracoin",
    "vertcoin", "blackcoin",
    "dash", "zcash", "monero",
    "stealth coin", "bytecoin",
    "potcoin", "gamblecoin",
    "reddcoin", "ixcoin",
    "nxt", "bitshares",
    "ripple", "xrp", "stellar",
    "ethereum", "neo", "eos",
    "cardano", "polkadot",
    "solana", "avalanche",
    "chainlink", "uniswap",
    "compound", "aave",
    "maker", "sushi", "pancakeswap"
)
foreach ($m in $memoryPhrases) {
    $lines += $m
    $lines += $m.ToLower()
    $lines += $m.ToUpper()
    $lines += "$m!"
}

# === 14. Idioms ===
$idioms = @(
    "a piece of cake", "break a leg", "hit the road",
    "once in a blue moon", "the ball is in your court",
    "actions speak louder than words",
    "better late than never", "every cloud has a silver lining",
    "dont count your chickens", "easy does it",
    "fortune favors the bold", "good things come to those who wait",
    "the early bird catches the worm",
    "when pigs fly", "beat around the bush",
    "cut corners", "spill the beans",
    "let the cat out of the bag",
    "bite the bullet", "burn the midnight oil",
    "cost an arm and a leg", "devils advocate",
    "elephant in the room", "face the music",
    "get out of hand", "go the extra mile",
    "hit the nail on the head", "jump on the bandwagon",
    "kill two birds with one stone", "last but not least",
    "miss the boat", "on the fence",
    "pull your weight", "raining cats and dogs",
    "save for a rainy day", "the best of both worlds",
    "the straw that broke the camels back",
    "to each his own", "under the weather",
    "wrap your head around", "yours truly"
)
foreach ($i in $idioms) {
    $lines += $i
    $lines += $i.ToLower()
    $lines += $i.ToUpper()
    $lines += "$i!"
}

# === 15. Cybersecurity / Hacking ===
$cybersec = @(
    "cypherpunks", "weasel", "adam back",
    "pgp", "pretty good privacy", "openpgp",
    "darknet", "deepweb", "tor", "onion",
    "anonymous", "lulzsec", "hackers",
    "exploit", "zero day", "backdoor",
    "encryption", "decryption", "aes256",
    "rsa", "elliptic curve", "ecdsa",
    "secp256k1", "nist curve", "brainpool",
    "ed25519", "curve25519", "salsa20",
    "chacha20", "poly1305", "nacl",
    "libsodium", "openssl", "gpg",
    "keybase", "signal", "wire",
    "protonmail", "tutanota", "cryptpad",
    "zeronet", "freenet", "i2p",
    "darkode", "dark market", "silk road",
    "ross ulbricht", "dream", "alphabay",
    "hanse", "wall street market",
    "bitcoin mixer", "wasabi wallet",
    "samourai wallet", "joinmarket",
    "coinjoin", "lightning network",
    "liquid network", "fedimint",
    "cashu", "vault privacy",
    "tornado cash", "razor network",
    "tumbler", "mixer", "privacy coin",
    "monero privacy", "zcash shielded",
    "stealth address", "ring signature",
    "zk snark", "zk proof", "zero knowledge",
    "bulletproofs", "halting problem",
    "p versus np", "turing complete",
    "halting", "undecidable", "godel",
    "incompleteness", "chinese remainder",
    "euler totient", "fermat little",
    "diffie hellman", "ellgamal",
    "merkle damgard", "sponge function",
    "keccak", "blake2", "blake3",
    "argon2", "bcrypt", "scrypt",
    "pbkdf2", "hkdf", "hmac",
    "cmac", "gmac", "pmac"
)
foreach ($c in $cybersec) {
    $lines += $c
    $lines += $c.ToLower()
    $lines += $c.ToUpper()
    $lines += "$c!"
}

# === 16. Space / Planets ===
$space = @(
    "mercury", "venus", "mars", "jupiter", "saturn",
    "uranus", "neptune", "pluto",
    "apollo", "voyager", "hubble",
    "nasa", "space shuttle", "international space station",
    "black hole", "supernova", "nebula",
    "andromeda", "milky way", "cosmos",
    "pulsar", "quasar", "magnetar",
    "exoplanet", "kepler", "tegra",
    "mars rover", "curiosity", "perseverance",
    "opal", "ingenuity", "phoenix",
    "spirit", "opportunity", "sojourner",
    "pathfinder", "mars explorer",
    "new horizons", "juno", "cassini",
    "galileo", "magellan", "pioneer",
    "park probe", "mercury", "mariner",
    "accretion", "deep impact",
    "stardust", "rossetta", "philae",
    "hayabusa", "osiris rex",
    "artemis", "artemis 1", "artemis 2",
    "space x", "spacex", "starship",
    "falcon 9", "falcon heavy", "dragon",
    "crew dragon", "cargo dragon",
    "blue origin", "new shepard", "new gladden",
    "virgin galactic", "spaceship one",
    "spaceship two", "unity", "vss unity",
    "boeing starliner", "cstv", "cft",
    "sso", "leo", "geo", "mbo",
    "hellas planitia", "olympus mons",
    "valles marineris", "galle crater",
    "titan", "enceladus", "europa",
    "ganymede", "callisto", "io",
    "amater", "miranda", "ariel",
    "oberon", "titania", "oberon",
    "triton", "nereid", "phoebe",
    "hyperion", "iapetus", "rhea",
    "dione", "tethys", "mimas",
    "janus", "epimetheus", "prometheus",
    "pandora", "pan", "atlas"
)
foreach ($s in $space) {
    $lines += $s
    $lines += $s.ToUpper()
    $lines += "$s!"
    $lines += "$s123"
}

# === 17. Greek letters / Mathematical symbols ===
$greekLetters = @(
    "alpha", "beta", "gamma", "delta", "epsilon",
    "zeta", "eta", "theta", "iota", "kappa",
    "lambda", "mu", "nu", "xi", "omicron",
    "pi", "rho", "sigma", "tau", "upsilon",
    "phi", "chi", "psi", "omega",
    "alpha beta", "alpha omega",
    "omega point", "final omega",
    "big omega", "little omega",
    "sigma algebra", "sigma bond",
    "lambda calculus", "lambda function",
    "delta function", "dirac delta",
    "gamma function", "beta function",
    "zeta function", "riemann zeta",
    "phi function", "euler phi",
    "pi function", "prime counting",
    "alpha particle", "beta decay",
    "gamma ray", "delta wave",
    "theta brain wave", "sleep",
    "golden mean", "divine proportion",
    "phi ratio", "1.6180339887498948482045868343656",
    "phi number", "golden number"
)
foreach ($g in $greekLetters) {
    $lines += $g
    $lines += $g.ToUpper()
    $lines += "$g!"
}

# === 18. Mythology ===
$mythology = @(
    "zeus", "hermes", "apollo", "athena",
    "odin", "thor", "loki", "freya",
    "anubis", "ra", "osiris", "isis",
    "hades", "persephone", "medusa",
    "hercules", "perseus", "achilles",
    "odysseus", "ulysses", "aeneas",
    "titan", "prometheus", "epimetheus",
    "atlantis", "elysium", "olympus",
    "asgard", "valhalla", "midgard",
    "yggdrasil", "ragnarok", "fenrir",
    "jormungandr", "surtr", "heimgall",
    "niflheim", "muspelheim", "alfheim",
    "svartalfheim", "vanaheim", "jotunheim",
    "nine worlds", "world tree",
    "fate", "destiny", "prophecy",
    "oracle", "delphi", "sphinx",
    "minotaur", "labyrinth", "theseus",
    "ariadne", "thread", "maze",
    "centaur", "chiron", "satyr",
    "faun", "nymph", "dryad",
    "siren", "mermaid", "triton",
    "poseidon", "trident", "sea god",
    "demeter", "persephone", "kore",
    "hades", "pluto", "dis pater",
    "cerberus", "three headed dog",
    "scylla", "charybdis", "cyclops",
    "polyphemus", "polyphemus", "polyphemus"
)
foreach ($m in $mythology) {
    $lines += $m
    $lines += $m.ToUpper()
    $lines += "$m!"
    $lines += "$m123"
}

# === 19. Chinese / Japanese / Korean phrases ===
$cjkPhrases = @(
    "bitcoin zhongwen", "shuzi huo bi", "kuai lie shan",
    "ji mi huo bi", "xu ni huo bi", "dian zi huo bi",
    "wo de bitcoin", "di yi ge bitcoin",
    "bitcoin qian bao", "si yao", "gong yao",
    "mi ma", "mi ma xue", "jia mi",
    "zi you", "cai fu", "jin qian",
    "bit coin nihongo", "ango tsu ka", "ka so tsu ka",
    "bit coin wallet", "himitsu kagi", "koukai kagi",
    "pass word", "ango ka", "block chain",
    "bit coin korean", "am ho hwa pe", "ga sang hwa pe",
    "bit coin ji gab", "gae in ki", "gae gae ki",
    "bi mil beon ho", "am ho hwa", "beu reo chei en"
)
foreach ($c in $cjkPhrases) { $lines += $c }

# === 20. Tech products ===
$techProducts = @(
    "iphone", "ipad", "ipod", "macbook",
    "kindle", "playstation", "xbox", "wii",
    "nokia", "blackberry", "htc",
    "galaxy s", "pixel", "oneplus",
    "surface", "thinkpad", "dell xps",
    "mac pro", "mac mini", "imac",
    "studio display", "pro display",
    "airpod", "airpods pro", "beats",
    "sony wm1", "bose", "sennheiser",
    "jbl", "harman kardon", "bang olufsen",
    "dyson", "roomba", "nest",
    "echo", "alexa", "google home",
    "homepod", "hub", "smart home",
    "iot", "internet of things",
    "smart watch", "fitness tracker",
    "garmin", "fitbit", "apple watch",
    "pixel watch", "galaxy watch",
    "vr", "ar", "mr", "xr",
    "oculus", "quest", "rift",
    "vive", "index", "valve index",
    "ps vr", "psvr2", "playstation vr",
    "apple vr", "apple ar", "vision pro",
    "hololens", "magic leap",
    "neuralink", "brain computer interface",
    "bci", "emg", "eeg", "fnci"
)
foreach ($t in $techProducts) {
    $lines += $t
    $lines += $t.ToUpper()
    $lines += "$t!"
    $lines += "$t123"
}

# === 21. Social media ===
$socialMedia = @(
    "twitter", "facebook", "instagram",
    "reddit", "tumblr", "pinterest",
    "telegram", "whatsapp", "discord",
    "twitch", "youtube", "tiktok",
    "snapchat", "linkedin", "github",
    "stackoverflow", "medium", "devto",
    "hashnode", "mirror", "substack",
    "newsletter", "blog", "vlog",
    "podcast", "stream", "live",
    "meme", "dank meme", "wholesome meme",
    "shitpost", "copypasta", "greentext",
    "4chan", "8chan", "9chan",
    "420", "69", "1337", "l33t",
    "h4x0r", "n00b", "p00n", "f4g",
    "w0w", "lol", "lmao", "rofl",
    "brb", "gtg", "afk", "btw",
    "idk", "imho", "imo", "fyi",
    "tbf", "tbh", "smh", "fr",
    "ngl", "ikr", "ngl", "frfr",
    "no cap", "cap", "bet", "period",
    "slay", "periodt", "dead", "im dead",
    "cant even", "on god", "say less",
    "its giving", "understood the assignment",
    "main character energy", "villain era",
    "romantic era", "god era", "glow up"
)
foreach ($s in $socialMedia) {
    $lines += $s
    $lines += $s.ToUpper()
    $lines += "$s!"
}

# === 22. Science fiction books ===
$scifiBooks = @(
    "dune", "foundation", "ender game",
    "neuromancer", "snow crash", "ready player one",
    "the matrix", "blade runner",
    "neon genesis evangelion",
    "2001 a space odyssey",
    "contact", "carbon",
    "the left hand of darkness",
    "the dispossessed", "the forever war",
    "old man war", "ancillary justice",
    "three body problem", "dark forest",
    "death mask", "remembrance of earths past",
    "project hail mary", "we are legion",
    "the Martian", "arthur c clarke",
    "isaac asimov", "robert heinlein",
    "ursula le guin", "octavia butler",
    "greg bear", "alastair reynolds",
    "lisa gold", "martha wells",
    "jo haldeman", "joanne robinson",
    "n k jemisin", "the fifth season",
    "the obelisk gate", "the storm scale",
    "broken earth trilogy",
    "the name of the wind",
    "the kingkiller chronicle",
    "the way of kings", "words of radiance",
    "the brilliant throne", "oathbringer",
    "rhythm of war", "the dust of ambition",
    "the furthest shore", "the justice of gods",
    "the wind's truth", "the secret history",
    "the lying detective", "the wise mans fear"
)
foreach ($s in $scifiBooks) {
    $lines += $s
    $lines += $s.ToLower()
    $lines += $s.ToUpper()
    $lines += "$s!"
}

# === 23. TV Shows ===
$tvShows = @(
    "breaking bad", "game of thrones",
    "stranger things", "the office",
    "friends", "seinfeld", "lost",
    "sherlock", "house of cards",
    "the wire", "the sopranos",
    "better call saul", "fargo",
    "true detective", "mindhunter",
    "dark", "1899", "squid game",
    "the mandalorian", "the book of bobba fett",
    "andor", "obican quen", "resistance",
    "vision", "falcon", "winter soldier",
    "loki", "what if", "multiverse",
    "wanda vision", "hawk eye",
    "moon knight", "she hulk",
    "ms marvel", "secret invasion",
    "echo", "daredevil", "born again",
    "punisher", "defenders", "judicators",
    "agents of shield", "inhumans",
    "runaways", "cloak and dagger",
    "the gifted", "legion",
    "doom patrol", "titans",
    "watchmen", "v for vendetta",
    "the boys", "gen v",
    "invincible", "paper girls",
    "the boys", "homelander",
    "butcher", "starlight", "frenchie",
    "mm", "the deep", "black noah",
    "cher-ho", "rainbow", "siege",
    "anthony", "sisterhood", "vought",
    "speedster", "compound v", "compound blue",
    "translucent", "electric", "terror",
    "blacklion", "phoenix", "payback",
    "railgun", "dean crimp", "emcee",
    "june", "donnie", "chloe",
    "herman", "all american",
    "chess", "cassie", "amber",
    "translucent", "golden globe",
    "golden boy", "golden girl",
    "the golden compass", "dark materials",
    "lyra", "will", "pantalaimon",
    "resterly", "seraph", "dust",
    "alethiometer", "armillary sphere",
    "bolvangryst", "gyptians",
    "mrovi", "i-of-o", "coulter",
    "michael", "jeremy", "roger",
    "macosta", "costa", "fiona",
    "fanny", "tanya", "jeroboam",
    "magnus", "asriel", "aurora",
    "the arctic", "the antarctic",
    "the north pole", "the south pole",
    "the equator", "the prime meridian",
    "greenwich", "utc", "gmt",
    "daylight saving", "summer time",
    "winter time", "standard time",
    "leap year", "leap second",
    "bissextile", "intercalary",
    "julian calendar", "gregorian calendar",
    "islamic calendar", "hebrew calendar",
    "chinese calendar", "hindu calendar",
    "persian calendar", "coptic calendar",
    "ethiopian calendar", "maya calendar",
    "aztec calendar", "babylonian calendar",
    "roman calendar", "greek calendar",
    "easter", "christmas", "hanukkah",
    "diwali", "ramadan", "eid",
    "vesak", "nowruz", "yom kippur",
    "pessach", "shavuot", "sukkot",
    "purim", "tu bi shvat", "lag ba omer",
    "counting the omer", "shemitta",
    "yovel", "jubilee", "sabbatical"
)
foreach ($t in $tvShows) {
    $lines += $t
    $lines += $t.ToLower()
    $lines += $t.ToUpper()
    $lines += "$t!"
}

# === 24. Two-word passphrases (EFF style) ===
$effWords = @(
    "correct", "horse", "battery", "staple",
    "blue", "guitar", "ocean", "wave",
    "purple", "monkey", "dishwasher",
    "silver", "lightning", "thunder",
    "golden", "sunset", "mountain",
    "red", "robot", "kitchen",
    "green", "planet", "keyboard",
    "black", "coffee", "window",
    "white", "rabbit", "garden",
    "yellow", "submarine", "bridge",
    "orange", "jungle", "castle",
    "pink", "elephant", "dolphin",
    "brown", "bear", "forest",
    "gray", "wolf", "river",
    "violet", "rainbow", "flower",
    "indigo", "sky", "cloud",
    "teal", "mountain", "lake",
    "cyan", "crystal", "diamond",
    "magenta", "star", "moon",
    "salmon", "penguin", "turtle",
    "lavender", "butterfly", "dragonfly",
    "turquoise", "parrot", "toucan",
    "maroon", "panther", "jaguar",
    "olive", "owl", "falcon",
    "peach", "peacock", "phoenix"
)
# Generate all pairs
for ($i = 0; $i -lt $effWords.Length; $i++) {
    for ($j = $i + 1; $j -lt $effWords.Length; $j++) {
        $lines += "$($effWords[$i]) $($effWords[$j])"
        $lines += "$($effWords[$i])-$($effWords[$j])"
        $lines += "$($effWords[$i])_$($effWords[$j])"
    }
}

# === 25. Chemical elements ===
$elements = @(
    "hydrogen", "helium", "lithium", "beryllium",
    "boron", "carbon", "nitrogen", "oxygen",
    "fluorine", "neon", "sodium", "magnesium",
    "aluminum", "silicon", "phosphorus", "sulfur",
    "chlorine", "argon", "potassium", "calcium",
    "scandium", "titanium", "vanadium", "chromium",
    "manganese", "iron", "cobalt", "nickel",
    "copper", "zinc", "gallium", "germanium",
    "arsenic", "selenium", "bromine", "krypton",
    "rubidium", "strontium", "yttrium", "zirconium",
    "niobium", "molybdenum", "technetium", "ruthenium",
    "rhodium", "palladium", "silver", "cadmium",
    "indium", "tin", "antimony", "tellurium",
    "iodine", "xenon", "cesium", "barium",
    "lanthanum", "cerium", "praseodymium", "neodymium",
    "promethium", "samarium", "europium", "gadolinium",
    "terbium", "dysprosium", "holmium", "erbium",
    "thulium", "ytterbium", "lutetium", "hafnium",
    "tantalum", "tungsten", "rhenium", "osmium",
    "iridium", "platinum", "gold", "mercury",
    "thallium", "lead", "bismuth", "polonium",
    "astatine", "radon", "francium", "radium",
    "actinium", "thorium", "protactinium", "uranium",
    "neptunium", "plutonium", "americium", "curium",
    "berkelium", "californium", "einsteinium", "fermium",
    "mendelevium", "nobelium", "lawrencium", "rutherfordium",
    "dubnium", "seaborgium", "bohrium", "hassium",
    "meitnerium", "darmstadtium", "roentgenium", "copernicium",
    "nihonium", "flerovium", "moscovium", "livermorium",
    "tennessine", "oganesson"
)
foreach ($e in $elements) {
    $lines += $e
    $lines += $e.ToUpper()
    $lines += "$e!"
}

# === 26. Formulas / Science ===
$formulas = @(
    "e=mc2", "e equals mc squared",
    "f=ma", "f equals ma",
    "pv=nrt", "pv equals nrt",
    "entropy", "relativity", "quantum",
    "quantum mechanics", "quantum physics",
    "quantum computing", "quantum entanglement",
    "quantum supremacy", "quantum tunneling",
    "quantum field theory", "quantum chromodynamics",
    "quantum electrodynamics", "standard model",
    "higgs boson", "god particle",
    "string theory", "m theory", "loop quantum gravity",
    "general relativity", "special relativity",
    "time dilation", "length contraction",
    "mass energy equivalence", "photoelectric effect",
    "compton effect", "de broglie wavelength",
    "schrodinger equation", "heisenberg uncertainty",
    "planck constant", "boltzmann constant",
    "avogadro number", "faraday constant",
    "gas constant", "gravitational constant",
    "speed of light", "fine structure constant",
    "riemann hypothesis", "p vs np",
    "collatz conjecture", "goldbach conjecture",
    "twin prime conjecture", "fermat last theorem",
    "four color theorem", "pythagorean theorem",
    "euler identity", "euler formula",
    "bayes theorem", "central limit theorem",
    "law of large numbers", "law of averages",
    "normal distribution", "gaussian distribution",
    "poisson distribution", "binomial distribution",
    "exponential distribution", "uniform distribution",
    "chi squared", "t distribution",
    "f distribution", "wilcoxon test",
    "anova", "regression", "correlation",
    "covariance", "variance", "standard deviation",
    "mean", "median", "mode", "outlier",
    "skewness", "kurtosis", "percentile",
    "quartile", "interquartile range",
    "box plot", "histogram", "scatter plot",
    "line graph", "bar chart", "pie chart",
    "venn diagram", "tree diagram",
    "flow chart", "mind map", "concept map"
)
foreach ($f in $formulas) {
    $lines += $f
    $lines += $f.ToLower()
    $lines += $f.ToUpper()
    $lines += "$f!"
}

# === 27. Currency names ===
$currencies = @(
    "dollar", "euro", "pound", "yen", "yuan", "rupee",
    "bitcoin dollar", "digital dollar", "crypto dollar",
    "usd", "eur", "gbp", "jpy", "cny", "inr",
    "aud", "cad", "chf", "sek", "nok", "dkk",
    "pln", "czk", "huf", "ron", "bgn", "hrk",
    "rub", "uah", "gel", "amd", "azn", "kzt",
    "uzs", "kgs", "tjs", "mnd", "lbp", "syp",
    "jod", "iqd", "irr", "sar", "aed", "qar",
    "kwd", "bhd", "omr", "yER", "sdg", "egp",
    "mad", "tnd", "dzm", "lyd", "mro", "mru",
    "xof", "xaf", "xaf", "xpf", "wst", "top",
    "vuv", "nuf", "sjp", "xdr", "xts", "xxx",
    "brl", "mxn", "ars", "clp", "cop", "pen",
    "uyu", "pyg", "bob", "vef", "veb", "gyd",
    "srD", "awg", "ang", "bzd", "hnd", "nio",
    "crc", "pab", "gtq", "svc", "cus", "cuc"
)
foreach ($c in $currencies) {
    $lines += $c
    $lines += $c.ToUpper()
    $lines += "$c!"
}

# === 28. Car brands ===
$carBrands = @(
    "ferrari", "lamborghini", "porsche",
    "bmw", "mercedes", "audi",
    "tesla model s", "ford mustang",
    "chevrolet", "dodge", "jeep",
    "ram", "gmc", "cadillac",
    "lincoln", "buick", "oldsmobile",
    "pontiac", "plymouth", "datsun",
    "subaru", "mazda", "toyota",
    "honda", "nissan", "infiniti",
    "acura", "lexus", "genesis",
    "volvo", "saab", "rover",
    "lotus", "bentley", "rolls royce",
    "aston martin", "mclaren", "bugatti",
    "koenigsegg", "pagani", "rimac",
    "karma", "faraday future", "lucid",
    "rivian", "polestar", "cybertruck",
    "model 3", "model y", "model x",
    "roadster", "plaid", "performance",
    "turbo", "s", "rs", "gt",
    "st", "ttype", "rs6", "rs7",
    "gt3", "gt4", "gt2rs",
    "911", "718", "cayenne", "macan",
    "panamera", "taycan", "boxster", "cayman"
)
foreach ($c in $carBrands) {
    $lines += $c
    $lines += $c.ToUpper()
    $lines += "$c!"
    $lines += "$c123"
}

# === 29. Playing cards / Board games ===
$games = @(
    "ace of spades", "king of hearts",
    "queen of diamonds", "jack of clubs",
    "monopoly", "scrabble", "chess",
    "checkmate", "royal flush", "full house",
    "straight flush", "four of a kind",
    "three of a kind", "two pair",
    "one pair", "high card",
    "poker", "blackjack", "roulette",
    "baccarat", "craps", "slots",
    "keno", "bingo", "lottery",
    "powerball", "megamillions", "eurojackpot",
    "scratch card", "instant win",
    "dice", "d20", "d6", "d4",
    "d8", "d10", "d12", "d100",
    "critical hit", "natural 20", "critical fail",
    "initiative", "hit points", "armor class",
    "saving throw", "attack roll",
    "damage dice", "healing potion",
    "mana", "stamina", "experience",
    "level up", "quest", "dungeon",
    "boss fight", "loot box",
    "rare", "epic", "legendary", "mythic",
    "common", "uncommon", "unique",
    "crafting", "enchanting", "smithing",
    "alchemy", "potions", "scrolls",
    "runes", "sigils", "wards",
    "blessing", "curse", "hex",
    "spell", "cantrip", "incantation",
    "ritual", "ceremony", "chant",
    "mantra", "prayer", "invocation",
    "summoning", "banishment", "protection",
    "offense", "defense", "support",
    "tank", "healer", "damage dealer",
    "main damage", "off damage", "sub damage",
    "support", "utility", "crowd control",
    "burst", "sustain", "mobility",
    "range", "melee", "hybrid"
)
foreach ($g in $games) {
    $lines += $g
    $lines += $g.ToLower()
    $lines += $g.ToUpper()
    $lines += "$g!"
}

# === 30. Weather / Seasons ===
$weather = @(
    "sunny day", "rainy night", "snowy morning",
    "thunder storm", "lightning bolt",
    "rainbow", "sunshine", "moonlight",
    "starlight", "twilight", "dawn",
    "dusk", "noon", "midnight",
    "spring", "summer", "autumn", "winter",
    "equinox", "solstice", "perihelion",
    "aphelion", "precession", "nutation",
    "aberration", "parallax", "refraction",
    "diffraction", "interference", "polarization",
    "dispersion", "scattering", "absorption",
    "emission", "fluorescence", "phosphorescence",
    "incandescence", "luminescence", "bioluminescence",
    "chemiluminescence", "electroluminescence",
    "thermoluminescence", "triboluminescence",
    "sonoluminescence", "radioluminescence",
    "crystalloluminescence", "fractoluminescence"
)
foreach ($w in $weather) {
    $lines += $w
    $lines += $w.ToLower()
    $lines += $w.ToUpper()
    $lines += "$w!"
}

# === 31. Colors ===
$colors = @(
    "red", "blue", "green", "yellow", "purple", "orange",
    "black", "white", "silver", "gold", "crimson",
    "scarlet", "maroon", "burgundy", "wine",
    "navy", "azure", "cobalt", "cerulean",
    "teal", "turquoise", "aqua", "cyan",
    "emerald", "jade", "olive", "lime",
    "magenta", "violet", "indigo", "lavender",
    "lilac", "rose", "pink", "coral",
    "salmon", "peach", "apricot", "amber",
    "honey", "bronze", "copper", "rust",
    "ochre", "sienna", "umber", "tan",
    "beige", "ivory", "cream", "ecru",
    "khaki", "sand", "fawn", "buff",
    "charcoal", "slate", "graphite", "ash",
    "smoke", "mist", "fog", "haze",
    "pearl", "opalescent", "iridescent", "metallic",
    "chrome", "platinum", "pewter", "nickel"
)
foreach ($c in $colors) {
    $lines += $c
    $lines += $c.ToUpper()
    $lines += "$c!"
    $lines += "$c123"
    # Color + animal combos
    foreach ($a in "dragon", "whale", "eagle", "panther", "fox", "tiger", "bear", "wolf") {
        $lines += "$c $a"
        $lines += "$c$a"
    }
}

# === 32. Foods ===
$foods = @(
    "pizza", "sushi", "taco", "burger",
    "chocolate", "vanilla", "strawberry",
    "coffee", "tea", "beer", "wine",
    "pasta", "risotto", "couscous",
    "curry", "ramen", "pho",
    "pad thai", "biryani", "tandoori",
    "sake", "soy sauce", "miso",
    "tofu", "tempeh", "seitan",
    "kimchi", "sauerkraut", "kombucha",
    "kefir", "yogurt", "cheese",
    "brie", "camembert", "cheddar",
    "gouda", "parmesan", "mozzarella",
    "feta", "goat cheese", "blue cheese",
    "caviar", "truffle", "foie gras",
    "lobster", "crab", "shrimp",
    "oyster", "clam", "mussel",
    "scallop", "squid", "octopus",
    "steak", "chicken", "pork",
    "lamb", "beef", "venison",
    "duck", "goose", "turkey",
    "bread", "baguette", "sourdough",
    "croissant", "bagel", "muffin",
    "donut", "cookie", "cake",
    "pie", "tart", "brownie",
    "ice cream", "sorbet", "gelato",
    "sherbet", "mousse", "creme brulee",
    "tiramisu", "panna cotta", "flan",
    "macaron", "eclair", "croquembouche",
    "macaroon", "meringue", "souffle"
)
foreach ($f in $foods) {
    $lines += $f
    $lines += $f.ToUpper()
    $lines += "$f!"
    $lines += "$f123"
}

# === 33. Separated variations (my-bitcoin-key style) ===
$separatorBases = @(
    "my bitcoin key", "my bitcoin wallet",
    "my private key", "my secret key",
    "bitcoin private key", "bitcoin secret key",
    "bitcoin wallet key", "btc private key",
    "digital wallet key", "crypto private key",
    "my bitcoin address", "my btc address",
    "bitcoin wallet address", "btc wallet address",
    "my inheritance", "my emergency fund",
    "my retirement fund", "my financial freedom",
    "bitcoin is money", "bitcoin is freedom",
    "bitcoin is the future", "money is freedom",
    "digital gold", "sound money",
    "peer to peer", "decentralized money",
    "trust no one", "trust the code",
    "code is law", "not your keys not your coins",
    "be your own bank", "self custody",
    "cold storage", "paper wallet",
    "brain wallet", "brainwallet"
)
foreach ($b in $separatorBases) {
    $lines += $b.Replace(" ", "-")
    $lines += $b.Replace(" ", "_")
    $lines += $b.Replace(" ", ".")
    $lines += $b.Replace(" ", "")
    $lines += $b.Replace(" ", " ")
}

# === 34. "I want" phrases ===
$iWantPhrases = @(
    "i want bitcoin", "i want money",
    "i want to be rich", "i want freedom",
    "i want financial independence",
    "i want to retire early",
    "i want passive income",
    "i want to travel the world",
    "i want to buy a house",
    "i want to be free",
    "i want peace", "i want love",
    "i want happiness", "i want success",
    "i want power", "i want knowledge",
    "i want wisdom", "i want truth",
    "i want justice", "i want equality",
    "i want liberty", "i want democracy",
    "i want revolution", "i want change",
    "i want the future", "i want tomorrow",
    "i want everything", "i want nothing",
    "i want to disappear", "i want to vanish",
    "i want to be invisible", "i want to fly",
    "i want to be immortal", "i want to live forever",
    "i want to know everything", "i want to see everything",
    "i want to go to space", "i want to explore",
    "i want to discover", "i want to create",
    "i want to build", "i want to destroy",
    "i want to conquer", "i want to rule",
    "i want to lead", "i want to follow",
    "i want to serve", "i want to protect",
    "i want to defend", "i want to attack"
)
foreach ($w in $iWantPhrases) {
    $lines += $w
    $lines += $w.ToUpper()
    $lines += $w.ToLower()
    $lines += "$w!"
}

# === 35. Operating systems / Software ===
$osSoft = @(
    "windows", "linux", "ubuntu",
    "android", "ios", "macos",
    "firefox", "chrome", "safari",
    "opera", "edge", "brave",
    "vivaldi", "waterfox", "librewolf",
    "epic", "surf", "qute browser",
    "netscape", "internet explorer",
    "mozilla", "seamonkey", "k-meleon",
    "debian", "fedora", "arch",
    "gentoo", "slackware", "mint",
    "elementary os", "pop os", "manjaro",
    "endeavouros", "artix", "void",
    "nixos", "guix", "solus",
    "clear linux", "openSUSE", "tumbleweed",
    "freebsd", "openbsd", "netbsd",
    "dragonfly", "minix", "plan 9",
    "inferno", "amiga", "atari",
    "dos", "os2", "beos",
    "haiku", "reactos", "freeDOS",
    "chromeos", "fuchsia", "harmonyos",
    "tvos", "watchos", "visionos",
    "xr os", "automotive os", "iot os"
)
foreach ($o in $osSoft) {
    $lines += $o
    $lines += $o.ToUpper()
    $lines += "$o!"
}

# === 36. Crypto exchanges + keywords ===
$exchangeCombos = @(
    "mtgox bitcoin", "bitstamp key",
    "coinbase wallet", "kraken trade",
    "poloniex api", "binance deposit",
    "huobi wallet", "okex trade",
    "bitfinex key", "gemini wallet",
    "bittrex api", "kucoin deposit",
    "ftx wallet", "crypto.com key",
    "bybit trade", "mexc wallet",
    "gate.io key", "bitget trade",
    "bitmart wallet", "phemex key",
    "deribit trade", "bitmex key",
    "liquid wallet", "bitso key",
    "coinmama trade", "coinmama buy",
    "localbitcoins trade", "localbitcoins key",
    "bter wallet", "zb.com key",
    "bitz trade", "hotbit wallet",
    "crex24 key", "tradeogre trade",
    "b2bx wallet", "cex.io key",
    "exmo trade", "livecoin wallet",
    "bl3p key", "lykke trade",
    "bitbay wallet", "crypto market key",
    "btc exchange", "crypto exchange",
    "bitcoin exchange", "btc trading",
    "crypto trading", "bitcoin trading",
    "decentralized exchange", "dex",
    "centralized exchange", "cex",
    "automated market maker", "amm",
    "liquidity pool", "yield farming",
    "liquidity mining", "staking",
    "proof of stake", "pos",
    "proof of work", "pow",
    "proof of authority", "poa",
    "proof of history", "poh",
    "proof of space", "pospace",
    "proof of capacity", "poc",
    "proof of elapsed time", "poet",
    "proof of burn", "pob",
    "proof of importance", "poi",
    "proof of activity", "poa",
    "delegated proof of stake", "dpos",
    "leap proof of stake", "lpos",
    "bonded proof of stake", "bpos",
    "proof of utility", "pou",
    "proof of reputation", "por",
    "proof of contribution", "poc",
    "proof of existence", "poe"
)
foreach ($e in $exchangeCombos) {
    $lines += $e
    $lines += $e.ToLower()
    $lines += $e.ToUpper()
    $lines += "$e!"
}

# === 37. Random memorable hex patterns ===
$hexPatterns = @(
    "0000000000000000000000000000000000000000000000000000000000000000",
    "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    "0101010101010101010101010101010101010101010101010101010101010101",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    "1111111111111111111111111111111111111111111111111111111111111111",
    "2222222222222222222222222222222222222222222222222222222222222222",
    "3333333333333333333333333333333333333333333333333333333333333333",
    "4444444444444444444444444444444444444444444444444444444444444444",
    "5555555555555555555555555555555555555555555555555555555555555555",
    "6666666666666666666666666666666666666666666666666666666666666666",
    "7777777777777777777777777777777777777777777777777777777777777777",
    "8888888888888888888888888888888888888888888888888888888888888888",
    "9999999999999999999999999999999999999999999999999999999999999999",
    "abcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabca",
    "1231231231231231231231231231231231231231231231231231231231231231",
    "1234123412341234123412341234123412341234123412341234123412341234",
    "abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd",
    "0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f",
    "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0",
    "0ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00f",
    "f00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff0",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
    "cafebabecafebabecafebabecafebabe",
    "deadbeefdeadbeefdeadbeefdeadbeef",
    "baddcafebaddcafebaddcafebaddcafe",
    "facefacefacefacefacefacefaceface",
    "baadbaadbaadbaadbaadbaadbaadbaad",
    "13371337133713371337133713371337",
    "4242424242424242424242424242424242424242424242424242424242424242",
    "6666666666666666666666666666666666666666666666666666666666666666",
    "7777777777777777777777777777777777777777777777777777777777777777"
)
foreach ($h in $hexPatterns) { $lines += $h }

# === 38. Bitcoin block 0 genesis coinbase ===
$genesisRelated = @(
    "the chicago tribune 03jan2009 chancellor on brink of second bailout for banks",
    "chancellor on brink of second bailout for banks",
    "chancellor on brink",
    "second bailout for banks",
    "genesis block coinbase",
    "block 0 coinbase",
    "genesis coinbase",
    "41046a8080",
    "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
    "genesis", "block 0", "block zero",
    "first block", "genesis transaction",
    "coinbase transaction", "coinbase reward",
    "50 bitcoin", "50 btc", "fifty bitcoin",
    "block reward", "mining reward",
    "subsidy", "transaction fees",
    "unspendable", "lost bitcoin",
    "satoshi bitcoin", "genesis satoshi",
    "5000000000 sats", "first reward"
)
foreach ($g in $genesisRelated) {
    $lines += $g
    $lines += $g.ToLower()
    $lines += $g.ToUpper()
    $lines += "$g!"
}

# === 39. Famous passwords from breaches ===
$breachPasswords = @(
    "123456", "password", "12345678", "qwerty",
    "12345", "123456789", "1234567", "1234567890",
    "dragon", "111111", "baseball", "iloveyou",
    "master", "sunshine", "ashley", "bailey",
    "shadow", "123123", "654321", "superman",
    "qazwsx", "michael", "football", "password1",
    "password123", "jesus", "ninja", "mustang",
    "222222", "princess", "admin", "login",
    "starwars", "solo", "passw0rd", "welcome",
    "hello", "charlie", "donald", "test",
    "admin123", "love", "letmein", "trustno1",
    "access", "thunder", "matthew", "daniel",
    "password2", "000000", "1qaz2wsx", "zxcvbn",
    "killer", "george", "hammer", "summer",
    "winter", "spring", "autumn", "flower",
    "cookie", "butter", "cheese", "pepper",
    "silver", "golden", "diamond", "freedom",
    "justice", "peace", "money", "rich", "wealth",
    "corvette", "austin", "thomas", "jessie",
    "jordan", "hunter", "falcon", "robert",
    "dallas", "yankees", "joshua", "maggie",
    "ginger", "secret", "nicole", "jason",
    "sexy", "orange", "taylor", "matrix",
    "mollie", "keeping", "tamara", "joseph",
    "hardcore", "keepout", "scott", "costello",
    "banana", "javier", "barcelona", "jennifer",
    "hottie", "amanda", "computer", "peanut",
    "whatever", "iceman", "smokey", "gateway",
    "soccer", "sparky", "dolphin", "tigger",
    "eagles", "ranger", "chelsea", "biteme",
    "zxcvbnm", "harder", "internet", "bigdog",
    "andrew", "1q2w3e4r", "thx1138", "55555",
    "aaron", "dave", "network", "bond007",
    "johnny", "bigdaddy", "1q2w3e", "555555",
    "bear", "samantha", "hockey", "summer1",
    "7777777", "jaguar", "joe", "city",
    "la", "newyork", "philadelphia", "sanfrancisco",
    "losangeles", "chicago", "houston", "phoenix",
    "philly", "boston", "seattle", "denver",
    "miami", "detroit", "atlanta", "portland",
    "minneapolis", "dallas", "texas", "california",
    "newjersey", "pennsylvania", "ohio", "florida",
    "illinois", "texas", "newmexico", "arizona"
)
foreach ($b in $breachPasswords) {
    $lines += $b
    $lines += $b.ToUpper()
    $lines += $b.Substring(0,1).ToUpper() + $b.Substring(1)
    $lines += "$b!"
    $lines += "$b123"
    foreach ($y in 2009..2015) { $lines += "$b$y" }
}

# === 40. Brainwallet.org specific patterns ===
$brainwalletSpecific = @(
    "brainwallet", "brain wallet",
    "brainwallet.org", "brainwallet.cn",
    "brainwallet.org my key",
    "brainwallet private key",
    "brainwallet bitcoin",
    "brainwallet btc",
    "generated by brainwallet",
    "brainwallet generated",
    "brainwallet passphrase",
    "brainwallet seed",
    "brainwallet entropy",
    "brainwallet hash",
    "brainwallet sha256",
    "brainwallet md5",
    "brainwallet ripemd",
    "brainwallet compressed",
    "brainwallet uncompressed",
    "brainwallet legacy",
    "brainwallet segwit",
    "brainwallet taproot",
    "brainwallet wif",
    "brainwallet hex",
    "brainwallet base58",
    "brainwallet bech32",
    "brainwallet address",
    "brainwallet p2pkh",
    "brainwallet p2sh",
    "brainwallet p2wpkh",
    "brainwallet p2tr",
    "brainwallet private",
    "brainwallet public",
    "brainwallet key pair",
    "brainwallet wallet",
    "brainwallet import",
    "brainwallet export",
    "brainwallet backup",
    "brainwallet restore",
    "brainwallet recover",
    "brainwallet generate",
    "brainwallet create",
    "brainwallet new",
    "brainwallet old",
    "brainwallet first",
    "brainwallet last",
    "brainwallet test",
    "brainwallet demo",
    "brainwallet example",
    "brainwallet sample",
    "brainwallet tutorial",
    "brainwallet guide",
    "brainwallet help",
    "brainwallet faq",
    "brainwallet about",
    "brainwallet contact",
    "brainwallet support",
    "brainwallet forum",
    "brainwallet reddit",
    "brainwallet bitcointalk",
    "brainwallet github",
    "brainwallet bitbucket",
    "brainwallet gitlab",
    "brainwallet source",
    "brainwallet code",
    "brainwallet javascript",
    "brainwallet node",
    "brainwallet browser",
    "brainwallet client",
    "brainwallet server",
    "brainwallet offline",
    "brainwallet online",
    "brainwallet secure",
    "brainwallet insecure",
    "brainwallet safe",
    "brainwallet unsafe",
    "brainwallet strong",
    "brainwallet weak",
    "brainwallet random",
    "brainwallet deterministic",
    "brainwallet predictable",
    "brainwallet unpredictable",
    "brainwallet entropy",
    "brainwallet randomness",
    "brainwallet prng",
    "brainwallet trng",
    "brainwallet csrng",
    "brainwallet hwrng",
    "brainwallet urandom",
    "brainwallet random",
    "brainwallet devrandom",
    "brainwallet devurandom",
    "brainwallet getrandom",
    "brainwallet randombytes",
    "brainwallet sodium",
    "brainwallet libsodium",
    "brainwallet nacl",
    "brainwallet tweetnacl"
)
foreach ($b in $brainwalletSpecific) {
    $lines += $b
    $lines += $b.ToLower()
    $lines += $b.ToUpper()
    $lines += "$b!"
}

# === Deduplicate and write ===
$unique = $lines | Where-Object { $_ -and $_.Trim() } | Sort-Object -Unique
Write-Host "Total unique patterns (Wave 2): $($unique.Count)"
$unique | Out-File -FilePath $outputFile -Encoding UTF8
Write-Host "Written to $outputFile"

# === Merge with existing corpus ===
$existing = Get-Content "Y:\btcsolver\brainwallet-all-corpus.txt" | Where-Object { $_ -and $_.Trim() }
$combined = @($existing) + @($unique)
$finalUnique = $combined | Where-Object { $_ -and $_.Trim() } | Sort-Object -Unique
Write-Host "Total unique patterns (combined): $($finalUnique.Count)"
$finalUnique | Out-File -FilePath "Y:\btcsolver\brainwallet-all-corpus-v2.txt" -Encoding UTF8
Write-Host "Written combined to brainwallet-all-corpus-v2.txt"
