#!/usr/bin/env python3
"""Wave 3: High-priority systematic brainwallet categories (fast Python)."""
from pathlib import Path
from calendar import monthrange
from itertools import product

OUTPUT = Path(r"Y:\btcsolver\brainwallet-wave3-corpus.txt")
lines: set[str] = set()

def add(*items):
    for x in items:
        if x and str(x).strip():
            lines.add(str(x).strip())

print("1. Single chars + pairs + triples...")
for c in range(97, 123):
    add(chr(c))
for c in range(65, 91):
    add(chr(c))
for c in range(48, 58):
    add(chr(c))
for a, b in product(range(97, 123), repeat=2):
    add(chr(a) + chr(b))
for a, b in product(range(65, 91), repeat=2):
    add(chr(a) + chr(b))
for a, b in product(range(48, 58), repeat=2):
    add(chr(a) + chr(b))
for a, b, c in product(range(97, 123), repeat=3):
    add(chr(a) + chr(b) + chr(c))
for a, b, c in product(range(48, 58), repeat=3):
    add(chr(a) + chr(b) + chr(c))
print(f"   -> {len(lines)} so far")

print("2. Numbers 0-100000 + hex/bin/oct...")
for n in range(0, 100001):
    add(str(n))
for n in range(0, 10001):
    add(format(n, "x"), format(n, "o"))
for n in range(0, 1001):
    add(format(n, "b"))
print(f"   -> {len(lines)} so far")

print("3. Birth dates 1900-2010 (all formats)...")
for y in range(1900, 2011):
    for m in range(1, 13):
        _, max_d = monthrange(y, m)
        for d in range(1, max_d + 1):
            mm, dd = f"{m:02d}", f"{d:02d}"
            yy = f"{y % 100:02d}"
            add(
                f"{dd}{mm}{y}", f"{mm}{dd}{y}", f"{y}{mm}{dd}",
                f"{dd}-{mm}-{y}", f"{mm}-{dd}-{y}", f"{y}-{mm}-{dd}",
                f"{dd}/{mm}/{y}", f"{mm}/{dd}/{y}", f"{y}/{mm}/{dd}",
                f"{dd}.{mm}.{y}",
                f"{dd}{mm}{yy}", f"{mm}{dd}{yy}", f"{yy}{mm}{dd}",
            )
print(f"   -> {len(lines)} so far")

print("4. Rockyou-style passwords + variations...")
rockyou = [
    "123456","password","12345678","qwerty","12345","123456789","1234567",
    "1234567890","dragon","111111","baseball","iloveyou","master","sunshine",
    "ashley","bailey","shadow","123123","654321","superman","qazwsx","michael",
    "football","password1","password123","jesus","ninja","mustang","222222",
    "princess","admin","login","starwars","solo","passw0rd","welcome","hello",
    "charlie","donald","test","admin123","love","letmein","trustno1","access",
    "thunder","matthew","daniel","password2","000000","1qaz2wsx","zxcvbn",
    "killer","george","hammer","summer","winter","spring","flower","cookie",
    "butter","cheese","pepper","silver","golden","diamond","freedom","justice",
    "peace","money","rich","wealth","corvette","austin","thomas","jessie",
    "jordan","hunter","falcon","robert","dallas","yankees","joshua","maggie",
    "ginger","secret","nicole","jason","sexy","orange","taylor","matrix",
    "computer","peanut","whatever","iceman","smokey","gateway","soccer","sparky",
    "dolphin","tigger","eagles","ranger","chelsea","biteme","zxcvbnm","internet",
    "bigdog","andrew","1q2w3e4r","thx1138","55555","aaron","dave","network",
    "bond007","johnny","bigdaddy","1q2w3e","555555","bear","samantha","hockey",
    "summer1","7777777","jaguar","joe","god","angel","baby","sweet","pretty",
    "beautiful","amazing","awesome","cool","nice","great","perfect","happy",
    "lucky","brave","strong","fast","power","force","light","dark","fire",
    "ice","water","earth","wind","storm","rain","snow","cloud","star","moon",
    "sun","sky","sea","ocean","river","lake","mountain","forest","garden",
    "home","house","castle","tower","bridge","road","street","dream","wish",
    "hope","faith","trust","truth","honor","glory","fame","strength","courage",
    "wisdom","knowledge","love","spirit","soul","heart","mind","body","blood",
    "bone","stone","steel","iron","gold","silver","copper","bronze","crystal",
    "ruby","emerald","pearl","ivory","jade","amber","coral","sapphire","topaz",
    "black","white","red","blue","green","yellow","purple","orange","pink","gray",
    "key","lock","safe","vault","chest","box","bag","wallet","coin","cash",
    "king","queen","prince","princess","knight","wizard","witch","mage","ninja",
    "samurai","viking","pirate","captain","soldier","pilot","doctor","teacher",
    "bitcoin","btc","crypto","wallet","private","public","seed","mnemonic","hodl",
    "satoshi","nakamoto","blockchain","genesis","halving","mining","hash","sha256",
]
for p in rockyou:
    add(p, p.upper(), p.capitalize(), f"{p}!", f"{p}!!", f"{p}!!!", f"{p}123", f"{p}1")
    for y in range(2009, 2021):
        add(f"{p}{y}")
    add(f"my {p}", f"the {p}", f"{p} bitcoin", f"bitcoin {p}")
print(f"   -> {len(lines)} so far")

print("5. Top English words + variations...")
english = [
    "the","be","to","of","and","a","in","that","have","it","for","not","on","with",
    "he","as","you","do","at","this","but","his","by","from","they","we","say","her",
    "she","or","an","will","my","one","all","would","there","their","what","so","up",
    "out","if","about","who","get","which","go","me","when","make","can","like","time",
    "no","just","him","know","take","people","into","year","your","good","some","could",
    "them","see","other","than","then","now","look","only","come","its","over","think",
    "also","back","after","use","two","how","our","work","first","well","way","even",
    "new","want","because","any","these","give","day","most","us","great","between",
    "need","large","end","under","never","city","tree","cross","carry","born","every",
    "white","house","right","boy","old","too","mean","before","through","story","off",
    "member","against","move","night","point","find","long","both","little","keep",
    "head","word","begin","life","hand","eye","picture","change","next","small","form",
    "real","home","school","program","idea","children","air","death","father","open",
    "line","free","busy","dark","full","empty","clean","dirty","hot","cold","warm",
    "cool","fast","slow","hard","soft","high","low","deep","wide","narrow","thick",
    "thin","short","tall","big","huge","tiny","early","late","young","fresh","sweet",
    "sour","bitter","salty","spicy","mild","strong","weak","loud","quiet","bright",
    "dim","clear","smooth","rough","sharp","dull","wet","dry","light","heavy","easy",
    "difficult","simple","complex","common","rare","normal","strange","funny","serious",
    "calm","angry","sad","glad","proud","afraid","sure","alone","together","near","far",
    "love","hate","hope","fear","trust","dream","wish","peace","war","fight","dance",
    "sing","play","work","rest","sleep","wake","run","walk","stop","start","begin",
    "finish","open","close","break","build","create","destroy","heal","kill","save",
    "lose","win","find","seek","hide","show","tell","ask","answer","learn","teach",
    "read","write","speak","hear","listen","watch","feel","touch","taste","smell",
    "breathe","live","die","grow","change","turn","fall","rise","fly","swim","climb",
    "jump","push","pull","throw","catch","hold","drop","carry","give","take","send",
    "receive","buy","sell","pay","cost","worth","value","price","money","power","force",
    "energy","speed","sound","noise","music","color","shape","size","weight","number",
    "letter","name","place","space","world","earth","nature","water","fire","air",
    "spirit","god","devil","angel","demon","ghost","soul","mind","heart","brain",
    "body","blood","bone","skin","hair","face","hand","foot","head","arm","leg",
    "king","queen","prince","princess","knight","warrior","hero","villain","master",
    "servant","friend","enemy","lover","stranger","neighbor","family","parent","child",
    "brother","sister","mother","father","son","daughter","husband","wife","baby","pet",
    "dog","cat","bird","fish","horse","cow","pig","sheep","chicken","rabbit","mouse",
    "snake","frog","bear","wolf","fox","deer","eagle","shark","whale","lion","tiger",
    "elephant","monkey","dragon","phoenix","unicorn","griffin","giant","dwarf","elf",
    "troll","ogre","goblin","fairy","witch","wizard","mage","ninja","samurai","viking",
    "pirate","knight","soldier","captain","chief","leader","boss","teacher","student",
    "doctor","nurse","police","driver","pilot","sailor","farmer","hunter","builder",
    "artist","musician","singer","writer","thinker","dreamer","believer","follower",
]
for w in english:
    add(w, w.upper(), w.capitalize(), f"{w}!", f"{w}123")
    for y in range(2009, 2016):
        add(f"{w}{y}")
print(f"   -> {len(lines)} so far")

print("6. Color + Animal combos...")
colors = [
    "red","blue","green","yellow","purple","orange","black","white","silver","gold",
    "pink","brown","gray","cyan","magenta","violet","indigo","teal","maroon","navy",
    "crimson","scarlet","amber","ivory","pearl","ruby","emerald","sapphire","bronze",
    "copper","obsidian","cobalt","azure","lavender","coral","salmon","olive","lime",
    "turquoise","chocolate","vanilla","cherry","lemon","mint","honey","sand","stone",
    "ash","smoke","cloud","snow","ice","flame","storm","thunder","lightning","shadow",
    "dark","bright","neon","electric","cosmic","galactic","stellar","lunar","solar",
    "midnight","sunrise","sunset","twilight","dawn","dusk",
]
animals = [
    "dragon","phoenix","unicorn","panther","tiger","lion","bear","wolf","eagle",
    "falcon","hawk","owl","raven","crow","fox","deer","horse","whale","shark",
    "dolphin","serpent","cobra","viper","scorpion","spider","butterfly","snake",
    "lizard","turtle","frog","fish","salmon","octopus","seal","penguin","swan",
    "parrot","hawk","cat","dog","rabbit","mouse","elephant","monkey","gorilla",
    "cheetah","leopard","jaguar","cougar","hyena","badger","raccoon","squirrel",
    "beast","monster","spirit","ghost","phantom","wraith","angel","demon","god",
    "king","queen","knight","warrior","mage","wizard","ninja","samurai","viking",
]
for c, a in product(colors, animals):
    add(f"{c} {a}", f"{c}{a}", f"{c}-{a}", f"{c}_{a}")
print(f"   -> {len(lines)} so far")

print("7. First + Last name combos (top sets)...")
first_names = [
    "james","john","robert","michael","william","david","richard","joseph","thomas",
    "charles","christopher","daniel","matthew","anthony","mark","donald","steven",
    "paul","andrew","joshua","kenneth","kevin","brian","george","timothy","jason",
    "jeffrey","ryan","jacob","gary","nicholas","eric","jonathan","stephen","larry",
    "justin","scott","brandon","benjamin","samuel","frank","alexander","jack",
    "dennis","jerry","tyler","aaron","jose","adam","nathan","henry","walter",
    "arthur","lawrence","jennifer","linda","barbara","patricia","jessica","sarah",
    "karen","nancy","lisa","betty","margaret","sandra","ashley","emily","donna",
    "melissa","deborah","stephanie","rebecca","sharon","lauren","cynthia","amy",
    "angela","anna","brenda","emma","nicole","helen","samantha","katherine",
    "christine","marie","amanda","rachel","catherine","heather","diana","ruth",
    "olivia","julie","kelly","megan","amber","sophia","isabella","mia","charlotte",
    "amelia","harper","abigail","ella","avery","camila","aria","scarlett","victoria",
    "madison","luna","grace","chloe","penelope","layla","riley","zoey","nora","lily",
]
last_names = [
    "smith","johnson","williams","brown","jones","garcia","miller","davis",
    "rodriguez","martinez","hernandez","lopez","gonzalez","wilson","anderson",
    "thomas","taylor","moore","jackson","martin","lee","perez","thompson","white",
    "harris","sanchez","clark","ramirez","lewis","robinson","walker","young",
    "allen","king","wright","scott","torres","nguyen","hill","flores","green",
    "adams","nelson","baker","hall","rivera","campbell","mitchell","carter",
    "roberts","gomez","phillips","evans","turner","diaz","parker","cruz","edwards",
    "collins","reyes","stewart","morris","morales","murphy","cook","rogers",
    "gutierrez","ortiz","murray","ward","cox","howard","peterson","gray","watson",
    "brooks","kelly","sanders","price","bennett","wood","barnes","ross","henderson",
    "coleman","jenkins","perry","powell","long","patterson","hughes","butler",
    "simmons","foster","bryant","alexander","russell","griffin","hayes","chavez",
]
for fn, ln in product(first_names, last_names):
    add(f"{fn}{ln}", f"{fn} {ln}", f"{fn}.{ln}", f"{fn}_{ln}", f"{fn}-{ln}")
    add(f"{fn}{ln}2009", f"{fn}{ln}!", f"{fn}{ln}123")
print(f"   -> {len(lines)} so far")

print("8. Bitcoin-specific dates...")
btc_dates = [
    "20090103","01032009","03012009","2009-01-03","01-03-2009","03-01-2009",
    "2009/01/03","01/03/2009","03/01/2009",
    "january 3 2009","jan 3 2009","3 january 2009","3 jan 2009",
    "20090112","01122009","12012009","2009-01-12","january 12 2009",
    "20081031","10312008","31102008","2008-10-31","october 31 2008",
    "20100701","07012010","2010-07-01","july 1 2010",
    "20100717","07172010","2010-07-17","july 17 2010",
    "20100522","05222010","2010-05-22","may 22 2010","bitcoin pizza day","bitcoin pizza",
    "20121128","11282012","2012-11-28","november 28 2012","first hallving","first halving",
    "20160709","07092016","2016-07-09","july 9 2016","second halving",
    "20200511","05112020","2020-05-11","may 11 2020","third hallving","third halving",
    "20240420","04202024","2024-04-20","april 20 2024","fourth hallving","fourth halving",
    "210000","336000","480000","600000","780000","840000",
    "the times 03/jan/2009 chancellor on brink of second bailout for banks",
    "chancellor on brink of second bailout for banks",
    "chancellor on brink","second bailout for banks",
    "genesis block","block 0","block zero","first block","coinbase",
    "50 bitcoin","50 btc","fifty bitcoin","block reward",
]
for d in btc_dates:
    add(d, f"{d}!", f"{d} bitcoin", f"bitcoin {d}")
print(f"   -> {len(lines)} so far")

print("9. First names + numbers...")
firsts = [
    "james","john","robert","michael","william","david","richard","joseph","thomas",
    "charles","daniel","matthew","anthony","mark","steven","paul","andrew","joshua",
    "kevin","brian","george","jason","jeffrey","ryan","jacob","nicholas","eric",
    "jonathan","justin","scott","brandon","benjamin","samuel","frank","alexander",
    "jack","tyler","aaron","jose","adam","nathan","henry","jennifer","linda",
    "barbara","patricia","jessica","sarah","karen","nancy","lisa","betty","emily",
    "ashley","donna","melissa","stephanie","rebecca","lauren","amy","angela","anna",
    "emma","nicole","helen","samantha","amanda","rachel","heather","diana","olivia",
    "julie","kelly","megan","sophia","isabella","mia","charlotte","amelia","grace",
    "chloe","lily","hannah","zoe","ava","ella","riley","nora","luna","aria",
]
for n in firsts:
    add(n, n.upper(), n.capitalize(), f"{n}!", f"{n}123")
    for y in range(2009, 2021):
        add(f"{n}{y}")
    for nn in range(1, 100):
        add(f"{n}{nn}")
print(f"   -> {len(lines)} so far")

print("10. Pet names...")
pets = [
    "max","bella","charlie","lucky","cooper","sadie","tucker","annie","bailey",
    "lexi","bear","jack","lola","molly","pepper","toby","rover","teddy","lucy",
    "maggie","rocky","daisy","milo","chloe","winston","sasha","mocha","simba",
    "grace","shadow","gizmo","nala","titan","penelope","sunny","ginger","scooby",
    "spirit","nemo","patches","midnight","muffin","princess","king","duke","prince",
    "diamond","star","jewel","goldie","silver","rusty","spot","freckles","bubbles",
    "fuzzy","fluffy","fido","rex","shep","buddy","coco","oreo","zeus","luna",
    "buster","duke","zeus","ace","bandit","harley","riley","oliver","jasper",
    "willow","cleo","mittens","whiskers","tiger","smokey","boots","felix","garfield",
]
for p in pets:
    add(p, p.upper(), p.capitalize(), f"{p}!", f"{p}123", f"my {p}")
    for y in range(2009, 2021):
        add(f"{p}{y}")
print(f"   -> {len(lines)} so far")

print("11. Acronyms...")
acronyms = [
    "USA","UK","UN","EU","NATO","CIA","FBI","NSA","BBC","CNN","IBM","NASA","ESA",
    "FIFA","NBA","NFL","MLB","NHL","CEO","COO","CFO","CTO","AI","ML","TCP","IP",
    "HTTP","HTTPS","FTP","SSH","SSL","TLS","DNS","API","URL","RAM","ROM","CPU",
    "GPU","SSD","HDD","USB","WiFi","GPS","PDF","PNG","JPEG","GIF","MP3","MP4",
    "HTML","CSS","JS","JSON","XML","SQL","PHP","AWS","GCP","AES","RSA","ECC",
    "SHA1","SHA256","SHA512","MD5","HMAC","PBKDF2","WIF","P2PKH","P2SH","P2WPKH",
    "P2TR","BIP32","BIP39","BIP44","BIP84","HD","SECP256K1","BTC","ETH","XRP",
    "LTC","XMR","DOGE","ADA","SOL","DOT","AVAX","LINK","UNI","USDT","USDC",
]
for a in acronyms:
    add(a, a.lower(), f"{a}!", f"{a}123")
print(f"   -> {len(lines)} so far")

print("12. Brand slogans...")
slogans = [
    "just do it","think different","impossible is nothing",
    "the happiest place on earth","im lovin it","finger lickin good",
    "have it your way","because youre worth it","obey your thirst",
    "taste the rainbow","the best a man can get","a diamond is forever",
    "the ultimate driving machine","built ford tough","the real thing",
    "put a tiger in your tank","good to the last drop",
    "when you care enough to send the very best","unleash the power",
    "dare to be great","the power of choice","driving pleasure",
    "the future is electric","accelerating the world",
    "zero emissions","carbon neutral","eco friendly",
]
for s in slogans:
    add(s, s.lower(), s.upper(), f"{s}!")
print(f"   -> {len(lines)} so far")

print("13. BIP39 extended variations...")
bip39_path = Path(r"Y:\btcsolver\bip39-words.txt")
if bip39_path.exists():
    for w in bip39_path.read_text(encoding="utf-8", errors="ignore").splitlines():
        w = w.strip()
        if not w:
            continue
        add(f"{w}!", f"{w}!!", f"{w}!!!", f"{w}1", f"{w}12", f"{w}123",
            f"{w}1234", f"{w}12345", f"{w}123456",
            f"{w} bitcoin", f"{w} wallet", f"{w} key", f"{w} seed",
            f"my {w}", f"the {w}")
        for y in range(2009, 2016):
            add(f"{w}{y}")
print(f"   -> {len(lines)} so far")

print("14. French / German / Spanish phrases...")
foreign = [
    # French
    "liberte egalite fraternite","la vie est belle","je taime","cest la vie",
    "bonjour","bonsoir","merci beaucoup","vive la france","la tour eiffel",
    "bitcoin france","bitcoin paris","monnaie numerique","liberte financiere",
    "cle privee","cle publique","portefeuille bitcoin","mon bitcoin","ma vie",
    "paris","marseille","lyon","toulouse","nice","bordeaux","lille","strasbourg",
    # German
    "einigkeit und recht und freiheit","deutschland","berlin","munchen","hamburg",
    "digitales geld","kryptowahrung","privater schlussel","mein bitcoin","freiheit",
    "bitcoin deutschland","btc deutschland","geheimnis","sicherheit","geld",
    # Spanish
    "te quiero","te amo","la vida es bella","bitcoin espana","dinero digital",
    "libertad financiera","criptomoneda","clave privada","billetera bitcoin",
    "mi bitcoin","madrid","barcelona","valencia","sevilla","mexico","bogota",
    "buenos aires","lima","santiago","caracas",
]
for f in foreign:
    add(f, f.lower(), f.upper(), f"{f}!")
print(f"   -> {len(lines)} so far")

print("15. Phone number patterns (common)...")
for area in ["212","213","310","312","415","503","617","702","713","718","800","888","877","866"]:
    for prefix in ["555","100","200","300","400","500","600","700","800","900"]:
        for line in ["0000","0100","0111","0123","1111","1212","1234","1337","2222",
                     "3333","4444","5555","6666","7777","8888","9999","0001","1000",
                     "2000","4200","6969","8000","9000"]:
            add(f"{area}{prefix}{line}", f"{area}-{prefix}-{line}")
print(f"   -> {len(lines)} so far")

print("16. Dice / RPG patterns...")
for a, b, c, d in product(range(1, 7), repeat=4):
    add(f"{a}{b}{c}{d}")
for a, b in product(range(1, 21), repeat=2):
    add(f"{a}{b}")
for a, b, c in product(range(3, 19), repeat=3):
    add(f"{a}{b}{c}")
print(f"   -> {len(lines)} so far")

print("17. Wallet defaults...")
wallets = [
    "electrum","armory","multibit","bitcoin-qt","bitcoin core","bitcoin wallet",
    "coinbase","blockchain.info","exodus","trust wallet","ledger","trezor",
    "keepkey","coldcard","jade","blue wallet","samourai","wasabi","sparrow",
    "mycelium","bread wallet","blockstream green","electrum123","default password",
    "changeme","password","admin","root","test","wallet","seed","mnemonic",
]
for w in wallets:
    add(w, w.lower(), w.upper(), f"{w}!", f"{w}123", f"{w} password",
        f"{w} default", f"{w} key", f"{w} seed", f"{w} wallet")
print(f"   -> {len(lines)} so far")

print("18. Countries + Cities...")
places = [
    "united states","america","usa","united kingdom","britain","uk","france",
    "germany","italy","spain","portugal","greece","turkey","russia","china",
    "japan","korea","india","australia","canada","brazil","mexico","argentina",
    "new york","london","paris","tokyo","berlin","rome","madrid","moscow",
    "beijing","mumbai","shanghai","seoul","bangkok","dubai","hong kong",
    "singapore","sydney","toronto","vancouver","mexico city","buenos aires",
    "sao paulo","cairo","lagos","nairobi","istanbul","athens","amsterdam",
    "stockholm","oslo","helsinki","lisbon","milan","munich","barcelona",
    "mumbai","delhi","bangalore","osaka","kyoto","hanoi","jakarta",
]
for p in places:
    add(p, p.upper(), f"{p}!", f"{p} bitcoin", f"bitcoin {p}")
print(f"   -> {len(lines)} so far")

print("19. Memorable hex patterns...")
hex_patterns = [
    "0" * 64, "f" * 64, "a" * 64, "1" * 64, "2" * 64, "5" * 64, "6" * 64, "7" * 64, "8" * 64, "9" * 64,
    "deadbeef" * 8, "cafebabe" * 8, "baadf00d" * 8, "facefeed" * 8,
    "0123456789abcdef" * 4, "fedcba9876543210" * 4,
    "0f" * 32, "f0" * 32, "aa" * 32, "55" * 32, "ff00" * 16, "00ff" * 16,
    "1337" * 16, "4242" * 16, "6969" * 16, "0001" * 16, "1000" * 16,
]
for h in hex_patterns:
    add(h)
print(f"   -> {len(lines)} so far")

print("20. Short sequences / keyboard walks...")
kb = [
    "qwerty","qwertyuiop","asdfgh","asdfghjkl","zxcvbn","zxcvbnm",
    "qazwsx","1qaz2wsx","zaq1xsw2","1q2w3e","1q2w3e4r","1q2w3e4r5t",
    "qweasdzxc","asdzxc","qweasd","poiuyt","lkjhgf","mnbvcx",
    "azerty","azertyuiop","qwertz","abcdef","abcdefg","abcdefgh",
    "abcdefghijklmnopqrstuvwxyz","zyxwvutsrqponmlkjihgfedcba",
    "1234567890","0987654321","!@#$%^&*()","~!@#$%^&*()",
]
for k in kb:
    add(k, k.upper(), f"{k}!", f"{k}123", f"{k}1")
print(f"   -> {len(lines)} so far")

# Write
print(f"\nWriting {len(lines)} unique patterns to {OUTPUT}...")
sorted_lines = sorted(lines)
OUTPUT.write_text("\n".join(sorted_lines) + "\n", encoding="utf-8")
print(f"Done. Total: {len(sorted_lines)}")
