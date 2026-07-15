#!/usr/bin/env python3
"""Wave 5: reversed words, word+year combos, CamelCase, prefixes, pi slices, fibonacci, etc."""
from pathlib import Path
from itertools import product
import hashlib

OUTPUT = Path(r"Y:\btcsolver\brainwallet-wave5-corpus.txt")
lines: set[str] = set()

def add(*items):
    for x in items:
        if x and str(x).strip():
            lines.add(str(x).strip())

print("1. Reversed common words...")
words = [
    "password","bitcoin","wallet","secret","private","master","monkey","dragon",
    "money","freedom","satoshi","nakamoto","blockchain","crypto","admin","root",
    "login","welcome","hello","test","love","trust","sunshine","shadow","hunter",
    "killer","hacker","letmein","trustno1","iloveyou","qwerty","abc123","god",
    "jesus","angel","baby","computer","market","computer","access","princess",
    "football","baseball","hockey","soccer","batman","superman","starwars",
    "matrix","ninja","samurai","viking","pirate","wizard","phoenix","unicorn",
]
for w in words:
    add(w[::-1], w[::-1].upper(), w[::-1].capitalize(), f"{w[::-1]}!", f"{w[::-1]}123")
print(f"   {len(lines)}")

print("2. Prefix/suffix structural patterns...")
prefixes = ["btc","bitcoin","my","the","wallet","key","secret","private","crypto","x","xx","xxx"]
suffixes = ["btc","bitcoin","wallet","key","secret","private","2009","2010","2011","123","1","!"]
bases = ["password","test","love","god","money","freedom","satoshi","wallet","secret","hello","admin","root"]
for b in bases:
    for p in prefixes:
        add(f"{p}_{b}", f"{p}-{b}", f"{p}.{b}", f"{p}{b}", f"{p} {b}")
    for s in suffixes:
        add(f"{b}_{s}", f"{b}-{s}", f"{b}.{s}", f"{b}{s}", f"{b} {s}")
print(f"   {len(lines)}")

print("3. CamelCase multi-word...")
parts = [
    ("My","Bitcoin","Wallet"),("My","Private","Key"),("My","Secret","Key"),
    ("Bitcoin","Private","Key"),("The","Bitcoin","Wallet"),("First","Bitcoin","Wallet"),
    ("My","First","Bitcoin"),("Satoshi","Nakamoto","Key"),("Brain","Wallet","Key"),
    ("Digital","Gold","Key"),("Crypto","Currency","Key"),("Peer","To","Peer"),
    ("Not","Your","Keys"),("Be","Your","Own"),("Code","Is","Law"),
    ("To","The","Moon"),("Diamond","Hands","Hodl"),("Early","Adopter","Key"),
]
for p in parts:
    add("".join(p), " ".join(p), "_".join(p), "-".join(p), "".join(x.lower() for x in p))
    add(p[0].lower()+"".join(p[1:]))  # camelCase
print(f"   {len(lines)}")

print("4. Pi / e / golden ratio digit windows...")
pi = "31415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679"
e = "27182818284590452353602874713526624977572470936999595749669676277240766303535475945713821785251664274"
phi = "16180339887498948482045868343656381177203091798057628621354486227052604628189024497072072041893911374"
for s, name in [(pi,"pi"),(e,"e"),(phi,"phi")]:
    for length in [8, 16, 32, 64]:
        for start in range(0, min(len(s)-length+1, 40)):
            add(s[start:start+length])
    add(s[:32], s[:64], s)
    add(name, name.upper(), f"{name}!", f"{name}123")
print(f"   {len(lines)}")

print("5. Fibonacci as string...")
fib = [0, 1]
while fib[-1] < 10**20:
    fib.append(fib[-1] + fib[-2])
add("".join(str(x) for x in fib[:20]))
add("".join(str(x) for x in fib[:30]))
add(" ".join(str(x) for x in fib[:15]))
add(",".join(str(x) for x in fib[:15]))
for i in range(len(fib)):
    add(str(fib[i]))
print(f"   {len(lines)}")

print("6. First 200 primes as strings...")
def primes(n):
    sieve = [True]*(n+1)
    sieve[0]=sieve[1]=False
    for i in range(2, int(n**0.5)+1):
        if sieve[i]:
            for j in range(i*i, n+1, i):
                sieve[j]=False
    return [i for i,v in enumerate(sieve) if v]
ps = primes(2000)[:200]
for p in ps:
    add(str(p))
add("".join(str(p) for p in ps[:20]))
add(" ".join(str(p) for p in ps[:20]))
print(f"   {len(lines)}")

print("7. ROT13 of common words...")
def rot13(s):
    out = []
    for c in s:
        if 'a' <= c <= 'z':
            out.append(chr((ord(c)-ord('a')+13)%26 + ord('a')))
        elif 'A' <= c <= 'Z':
            out.append(chr((ord(c)-ord('A')+13)%26 + ord('A')))
        else:
            out.append(c)
    return "".join(out)
for w in words:
    r = rot13(w)
    add(r, r.upper(), f"{r}!", f"{r}123")
print(f"   {len(lines)}")

print("8. Base64 of common words (as passphrase)...")
import base64
for w in words:
    b = base64.b64encode(w.encode()).decode()
    add(b, b.rstrip("="))
print(f"   {len(lines)}")

print("9. IP addresses private ranges...")
for a, b in product(range(0, 5), range(0, 10)):
    add(f"192.168.{a}.{b}", f"10.0.{a}.{b}", f"172.16.{a}.{b}")
add("127.0.0.1","0.0.0.0","255.255.255.255","8.8.8.8","1.1.1.1","8.8.4.4")
print(f"   {len(lines)}")

print("10. Coordinates memorable...")
coords = [
    "40.7128,-74.0060","51.5074,-0.1278","48.8566,2.3522","35.6762,139.6503",
    "0,0","90,0","-90,0","0,180","0,-180",
    "37.7749,-122.4194","34.0522,-118.2437","41.8781,-87.6298",
    "55.7558,37.6173","39.9042,116.4074","28.6139,77.2090",
    "40.7128N74.0060W","51.5074N0.1278W",
]
for c in coords:
    add(c, c.replace(",", " "), c.replace(",", ""), c.replace(".", ""))
print(f"   {len(lines)}")

print("11. BIP39 top-200 pairs (space and concat)...")
bip39_path = Path(r"Y:\btcsolver\bip39-words.txt")
if bip39_path.exists():
    bip = [w.strip() for w in bip39_path.read_text(encoding="utf-8", errors="ignore").splitlines() if w.strip()]
    top = bip[:200]
    for a, b in product(top, top):
        if a != b:
            add(f"{a} {b}", f"{a}{b}")
print(f"   {len(lines)}")

print("12. Correct horse battery staple variants...")
eff = [
    "correct horse battery staple",
    "correcthorsebatterystaple",
    "correct-horse-battery-staple",
    "correct_horse_battery_staple",
    "Correct Horse Battery Staple",
    "CORRECT HORSE BATTERY STAPLE",
    "correct horse battery staple!",
    "correct horse battery staple 2009",
    "horse battery staple correct",
    "battery staple correct horse",
    "staple correct horse battery",
]
for e in eff:
    add(e)
print(f"   {len(lines)}")

print("13. Whitepaper key sentences...")
wp = [
    "A purely peer-to-peer version of electronic cash would allow online payments to be sent directly from one party to another without going through a financial institution",
    "We propose a solution to the double-spending problem using a peer-to-peer network",
    "The network timestamps transactions by hashing them into an ongoing chain of hash-based proof-of-work",
    "forming a record that cannot be changed without redoing the proof-of-work",
    "The longest chain not only serves as proof of the sequence of events witnessed",
    "but proof that it came from the largest pool of CPU power",
    "As long as a majority of CPU power is controlled by nodes that are not cooperating to attack the network",
    "they will generate the longest chain and outpace attackers",
    "The system is secure as long as honest nodes collectively control more CPU power than any cooperating group of attacker nodes",
    "Transactions that are computationally impractical to reverse would protect sellers from fraud",
    "and routine escrow mechanisms could easily be implemented to protect buyers",
    "In this paper we propose a solution to the double-spending problem using a peer-to-peer distributed timestamp server",
    "to generate computational proof of the chronological order of transactions",
    "The system is secure as long as honest nodes collectively control more CPU power than any cooperating group of attacker nodes",
]
for w in wp:
    add(w, w.lower(), w[:50], w[:64], w[:100])
print(f"   {len(lines)}")

print("14. Cypherpunk / early crypto phrases...")
cypher = [
    "cypherpunks write code","privacy is necessary for an open society",
    "we must defend our own privacy if we expect to have any",
    "we cannot expect governments corporations or other large faceless organizations to grant us privacy",
    "out of their own beneficence","cypherpunks write code",
    "we know that someone has to write software to defend privacy",
    "and since we cant get privacy unless we all do we are going to write it",
    "we publish our code so that our fellow cypherpunks may practice and play with it",
    "our code is free for all to use worldwide","we dont much care if you dont approve of the software we write",
    "we know that software cant be destroyed and that a widely dispersed system cant be shut down",
    "cypherpunks deplore regulations on cryptography for cryptography is fundamentally a private act",
    "the engine of freedom","crypto anarchy","the crypto anarchist manifesto",
    "a specter is haunting the modern world the specter of crypto anarchy",
    "computer technology is on the verge of providing the ability for individuals and groups to communicate",
    "and interact with each other in a totally anonymous manner",
    "two persons may exchange messages conduct business and negotiate electronic contracts",
    "without ever knowing the true name or legal identity of the other",
    "interactions over networks will be untraceable","via extensive re routing of encrypted packets",
    "and tamper proof boxes which implement cryptographic protocols with nearly perfect assurance",
    "against any tampering","reputations will be of central importance much more important in dealings",
    "than even the credit ratings of today","these developments will alter completely the nature of government regulation",
    "the ability to tax and control economic interactions","the nature of trust and reputation",
    "and the nature of privacy","the technology for this revolution and perhaps this revolution itself",
    "has awaited a computer revolution of the kind that is now bursting forth",
    "the internet","and the world of networked computers",
    "hal finney","adam back","wei dai","nick szabo","david chaum","tim may",
    "eric hughes","john gilmore","jude milhon","st jude","phil zimmermann","pgp",
    "pretty good privacy","hashcash","b money","bit gold","remailer","mixmaster",
]
for c in cypher:
    add(c, c.lower(), c.upper() if len(c)<40 else c)
print(f"   {len(lines)}")

print("15. Silk Road / MtGox related (historical brainwallet era)...")
hist = [
    "silk road","silkroad","ross ulbricht","dread pirate roberts","dpr",
    "mtgox","mt gox","mark karpeles","magicaltux","tibanne",
    "bitcoinica","tradefortress","bitfloor","bitcoinica hack",
    "pirateat40","mybitcoin","instawallet","inputs.io",
    "bitomat","bitcoin market","new liberty standard",
    "laszlo hanyecz","bitcoin pizza","10000 btc pizza",
    "hal finney first transaction","block 170",
    "bitcoin faucet","gavin andresen","mike hearn",
    "amir taaki","coderrr","theymos","nullc","adam back",
    "gregory maxwell","pieter wuille","wladimir van der laan",
    "satoshi","satoshi nakamoto","satoshin","satoshin@gmx.com",
    "from: satoshi nakamoto","bitcoin v0.1","bitcoin 0.1",
]
for h in hist:
    add(h, h.upper() if len(h)<30 else h, f"{h}!", f"{h}123", f"{h}2009", f"{h}2010", f"{h}2011")
print(f"   {len(lines)}")

print("16. Password1! style patterns...")
base_pw = ["Password","Bitcoin","Welcome","Admin","Master","Secret","Private","Wallet","Monkey","Dragon","Love","Test","God","Money","Freedom","Satoshi"]
for b in base_pw:
    for n in ["1","12","123","1234","12345","123456","!","1!","123!","!1","2009","2010"]:
        add(f"{b}{n}")
    add(f"{b}1!", f"{b}@1", f"{b}#1", f"{b}$1", f"P@{b[1:].lower()}1" if len(b)>1 else b)
print(f"   {len(lines)}")

print("17. Season + year...")
seasons = ["spring","summer","autumn","fall","winter","Spring","Summer","Autumn","Fall","Winter"]
for s in seasons:
    for y in range(2008, 2026):
        add(f"{s}{y}", f"{s} {y}", f"{s}!{y}", f"{s}{y}!")
print(f"   {len(lines)}")

print("18. Single digits repeated (4-32)...")
for d in "0123456789abcdef":
    for n in [4, 6, 8, 12, 16, 20, 24, 32, 40, 48, 56, 64]:
        add(d * n)
print(f"   {len(lines)}")

print("19. Dictionary top words if available...")
# Use bip39 + common english already; add more from a simple built-in list
extra = [
    "about","above","across","action","active","actual","advice","afraid","again",
    "agency","agree","ahead","album","alive","allow","almost","alone","along","already",
    "also","always","among","amount","animal","annual","answer","anyone","appear","apple",
    "apply","approach","area","argue","around","arrive","article","artist","assume","attack",
    "attend","author","avoid","away","baby","back","ball","bank","base","basic","basis",
    "battle","beach","beauty","become","before","begin","behind","believe","below","benefit",
    "best","better","between","beyond","bill","birth","bit","black","blood","blue","board",
    "boat","body","book","born","both","box","boy","break","bring","brother","budget","build",
    "building","business","call","camera","campaign","cancer","candidate","capital","card",
    "care","career","carry","case","catch","cause","cell","center","central","century","certain",
    "certainly","chair","challenge","chance","change","character","charge","check","child",
    "choice","choose","church","citizen","city","civil","claim","class","clear","clearly",
    "close","coach","cold","collection","college","color","come","commercial","common","community",
    "company","compare","computer","concern","condition","conference","congress","consider",
    "consumer","contain","continue","control","cost","could","country","couple","course","court",
    "cover","create","crime","cultural","culture","cup","current","customer","cut","dark","data",
    "daughter","day","dead","deal","death","debate","decade","decide","decision","deep","defense",
    "degree","democrat","democratic","describe","design","despite","detail","determine","develop",
    "development","die","difference","different","difficult","dinner","direction","director",
    "discover","discuss","discussion","disease","do","doctor","dog","door","down","draw","dream",
    "drive","drop","drug","during","each","early","east","easy","eat","economic","economy",
    "edge","education","effect","effort","eight","either","election","else","employee","end",
    "energy","enjoy","enough","enter","entire","environment","environmental","especially",
    "establish","even","evening","event","ever","every","everybody","everyone","everything",
    "evidence","exactly","example","executive","exist","expect","experience","expert","explain",
    "eye","face","fact","factor","fail","fall","family","far","farm","fast","father","fear",
    "federal","feel","feeling","few","field","fight","figure","fill","film","final","finally",
    "financial","find","fine","finger","finish","fire","firm","first","fish","five","floor",
    "fly","focus","follow","food","foot","for","force","foreign","forget","form","former",
    "forward","four","free","friend","from","front","full","fund","future","game","garden",
    "gas","general","generation","get","girl","give","glass","go","goal","good","government",
    "great","green","ground","group","grow","growth","guess","gun","guy","hair","half","hand",
    "hang","happen","happy","hard","have","he","head","health","hear","heart","heat","heavy",
    "help","her","here","herself","high","him","himself","his","history","hit","hold","home",
    "hope","hospital","hot","hotel","hour","house","how","however","huge","human","hundred",
    "husband","i","idea","identify","if","image","imagine","impact","important","improve",
    "in","include","including","increase","indeed","indicate","individual","industry",
    "information","inside","instead","institution","interest","interesting","international",
    "interview","into","investment","involve","issue","it","item","its","itself","job","join",
    "just","keep","key","kid","kill","kind","kitchen","know","knowledge","land","language",
    "large","last","late","later","laugh","law","lawyer","lay","lead","leader","learn","least",
    "leave","left","leg","legal","less","let","letter","level","lie","life","light","like",
    "likely","line","list","listen","little","live","local","long","look","lose","loss","lot",
    "love","low","machine","magazine","main","maintain","major","majority","make","man",
    "manage","management","manager","many","market","marriage","material","matter","may",
    "maybe","me","mean","measure","media","medical","meet","meeting","member","memory","mention",
    "message","method","middle","might","military","million","mind","minute","miss","mission",
    "model","modern","moment","money","month","more","morning","most","mother","mouth","move",
    "movement","movie","mr","mrs","much","music","must","my","myself","name","nation","national",
    "natural","nature","near","nearly","necessary","need","network","never","new","news",
    "newspaper","next","nice","night","no","none","nor","north","not","note","nothing","notice",
    "now","number","occur","of","off","offer","office","officer","official","often","oh","oil",
    "ok","old","on","once","one","only","onto","open","operation","opportunity","option","or",
    "order","organization","other","others","our","out","outside","over","own","owner","page",
    "pain","painting","paper","parent","part","participant","particular","particularly",
    "partner","party","pass","past","patient","pattern","pay","peace","people","per","perform",
    "performance","perhaps","period","person","personal","phone","physical","pick","picture",
    "piece","place","plan","plant","play","player","pm","point","police","policy","political",
    "politics","poor","popular","population","position","positive","possible","power","practice",
    "prepare","present","president","pressure","pretty","prevent","price","private","probably",
    "problem","process","produce","product","production","professional","professor","program",
    "project","property","protect","prove","provide","public","pull","purpose","push","put",
    "quality","question","quickly","quite","race","radio","raise","range","rate","rather",
    "reach","read","ready","real","reality","realize","really","reason","receive","recent",
    "recently","recognize","record","red","reduce","reflect","region","relate","relationship",
    "religious","remain","remember","remove","report","represent","republican","require",
    "research","resource","respond","response","responsibility","rest","result","return",
    "reveal","rich","right","rise","risk","road","rock","role","room","rule","run","safe",
    "same","save","say","scene","school","science","scientist","score","sea","season","seat",
    "second","section","security","see","seek","seem","sell","send","senior","sense","series",
    "serious","serve","service","set","seven","several","sex","sexual","shake","share","she",
    "shoot","short","shot","should","shoulder","show","side","sign","significant","similar",
    "simple","simply","since","sing","single","sister","sit","site","situation","six","size",
    "skill","skin","small","smile","so","social","society","soldier","some","somebody","someone",
    "something","sometimes","son","song","soon","sort","sound","source","south","southern",
    "space","speak","special","specific","speech","spend","sport","spring","staff","stage",
    "stand","standard","star","start","state","statement","station","stay","step","still",
    "stock","stop","store","story","strategy","street","strong","structure","student","study",
    "stuff","style","subject","success","successful","such","suddenly","suffer","suggest",
    "summer","support","sure","surface","system","table","take","talk","task","tax","teach",
    "teacher","team","technology","television","tell","ten","tend","term","test","than","thank",
    "that","the","their","them","themselves","then","theory","there","these","they","thing",
    "think","third","this","those","though","thought","thousand","threat","three","through",
    "throughout","throw","thus","time","to","today","together","tonight","too","top","total",
    "tough","toward","town","trade","traditional","training","travel","treat","treatment",
    "tree","trial","trip","trouble","true","truth","try","turn","tv","two","type","under",
    "understand","unit","until","up","upon","us","use","usually","value","various","very",
    "victim","view","violence","visit","voice","vote","wait","walk","wall","want","war","watch",
    "water","way","we","weapon","wear","week","weight","well","west","western","what","whatever",
    "when","where","whether","which","while","white","who","whole","whom","whose","why","wide",
    "wife","will","win","wind","window","wish","with","within","without","woman","wonder","word",
    "work","worker","world","worry","would","write","writer","wrong","yard","yeah","year","yes",
    "yet","you","young","your","yourself",
]
for w in extra:
    add(w, w.upper(), w.capitalize(), f"{w}!", f"{w}123", f"{w}1")
    for y in (2009, 2010, 2011, 2012):
        add(f"{w}{y}")
print(f"   {len(lines)}")

print("20. Encoding variants (trailing newline, space)...")
for w in ["password","bitcoin","test","god","love","hello","satoshi","wallet","secret","1","abc"]:
    add(w + "\n", w + "\r\n", w + " ", " " + w, w + "\t")
print(f"   {len(lines)}")

# Write
print(f"\nWriting {len(lines)} unique patterns...")
sorted_lines = sorted(lines)
OUTPUT.write_text("\n".join(sorted_lines) + "\n", encoding="utf-8")
print(f"Done. Total Wave 5: {len(sorted_lines)}")
