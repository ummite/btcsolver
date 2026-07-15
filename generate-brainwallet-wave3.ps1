# Wave 3: High-priority systematic categories
$outputFile = "Y:\btcsolver\brainwallet-wave3-corpus.txt"
$lines = @()

# === 1. Single characters a-z, A-Z, 0-9 ===
foreach ($c in 97..122) { $lines += [char]$c }       # a-z
foreach ($c in 65..90) { $lines += [char]$c }       # A-Z
foreach ($c in 48..57) { $lines += [char]$c }       # 0-9
# Pairs of single chars (aa, ab, ... zz, AA, AB, ... ZZ, 00, 01, ... 99)
foreach ($c1 in 97..122) { foreach ($c2 in 97..122) { $lines += ([char]$c1 + [char]$c2) } }
foreach ($c1 in 65..90) { foreach ($c2 in 65..90) { $lines += ([char]$c1 + [char]$c2) } }
foreach ($c1 in 48..57) { foreach ($c2 in 48..57) { $lines += ([char]$c1 + [char]$c2) } }
# Triple chars (aaa, aab, ... zzz)
foreach ($c1 in 97..122) { foreach ($c2 in 97..122) { foreach ($c3 in 97..122) { $lines += ([char]$c1 + [char]$c2 + [char]$c3) } } }
foreach ($c1 in 48..57) { foreach ($c2 in 48..57) { foreach ($c3 in 48..57) { $lines += ([char]$c1 + [char]$c2 + [char]$c3) } } }

Write-Host "Single chars + pairs + triples done"

# === 2. Numbers 0-100000 ===
foreach ($n in 0..100000) { $lines += "$n" }
# Hex representations
foreach ($n in 0..10000) { $lines += ([Convert]::ToString($n, 16)) }
# Binary representations
foreach ($n in 0..1000) { $lines += ([Convert]::ToString($n, 2)) }
# Octal representations
foreach ($n in 0..10000) { $lines += ([Convert]::ToString($n, 8)) }

Write-Host "Numbers done"

# === 3. Birth dates ALL formats 1900-2010 ===
foreach ($y in 1900..2010) {
    foreach ($m in 1..12) {
        $maxD = 31
        if ($m -eq 2) {
            if (($y % 4 -eq 0 -and $y % 100 -ne 0) -or $y % 400 -eq 0) { $maxD = 29 } else { $maxD = 28 }
        } elseif ($m -in 4,6,9,11) { $maxD = 30 }
        foreach ($d in 1..$maxD) {
            $mm = $m.ToString('D2')
            $dd = $d.ToString('D2')
            # DDMMYYYY
            $lines += "$dd$mm$y"
            # MMDDYYYY
            $lines += "$mm$dd$y"
            # YYYYMMDD
            $lines += "$y$mm$dd"
            # DD-MM-YYYY
            $lines += "$dd-$mm-$y"
            # MM-DD-YYYY
            $lines += "$mm-$dd-$y"
            # YYYY-MM-DD
            $lines += "$y-$mm-$dd"
            # DD/MM/YYYY
            $lines += "$dd/$mm/$y"
            # MM/DD/YYYY
            $lines += "$mm/$dd/$y"
            # YYYY/MM/DD
            $lines += "$y/$mm/$dd"
            # DD.MM.YYYY
            $lines += "$dd.$mm.$y"
            # DDMMYY (short year)
            $yy = ($y % 100).ToString('D2')
            $lines += "$dd$mm$yy"
            $lines += "$mm$dd$yy"
            $lines += "$yy$mm$dd"
        }
    }
}

Write-Host "Birth dates done"

# === 4. Top passwords (expanded Rockyou-style) ===
$rockyouPasswords = @(
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
    "mollie","keeping","tamara","joseph","hardcore","keepout","scott","costello",
    "banana","javier","barcelona","jennifer","hottie","amanda","computer",
    "peanut","whatever","iceman","smokey","gateway","soccer","sparky","dolphin",
    "tigger","eagles","ranger","chelsea","biteme","zxcvbnm","harder","internet",
    "bigdog","andrew","1q2w3e4r","thx1138","55555","aaron","dave","network",
    "bond007","johnny","bigdaddy","1q2w3e","555555","bear","samantha","hockey",
    "summer1","7777777","jaguar","joe","city","la","newyork","philadelphia",
    "sanfrancisco","losangeles","chicago","houston","phoenix","philly","boston",
    "seattle","denver","miami","detroit","atlanta","portland","minneapolis",
    "texas","california","newjersey","pennsylvania","ohio","florida","illinois",
    "newmexico","arizona","wisconsin","georgia","northcarolina","tennessee",
    "missouri","maryland","virginia","washington","nevada","indiana","michigan",
    "colorado","utah","oklahoma","connecticut","iowa","massachusetts","arizona",
    "alabama","mississippi","kentucky","oregon","clark","stewie","broker",
    "keyboard","killer","pass","test","god","angel","baby","sweet","pretty",
    "beautiful","amazing","awesome","cool","nice","great","perfect","happy",
    "lucky","brave","strong","fast","power","force","light","dark","fire",
    "ice","water","earth","wind","storm","rain","snow","cloud","star","moon",
    "sun","sky","sea","ocean","river","lake","mountain","forest","garden",
    "home","house","castle","tower","bridge","road","street","avenue","lane",
    "garden","park","beach","island","desert","jungle","valley","hill","cave",
    "dream","wish","hope","faith","trust","truth","honor","glory","fame",
    "power","strength","courage","wisdom","knowledge","peace","love","light",
    "shadow","spirit","soul","heart","mind","body","blood","bone","stone",
    "steel","iron","gold","silver","copper","bronze","crystal","diamond","ruby",
    "emerald","pearl","ivory","jade","amber","coral","sapphire","topaz","opal",
    "black","white","red","blue","green","yellow","purple","orange","pink","gray",
    "brown","gold","silver","bronze","copper","ivory","ebony","pearl","ruby",
    "emerald","sapphire","diamond","crystal","glass","mirror","window","door",
    "key","lock","safe","vault","chest","box","bag","pouch","wallet","purse",
    "coin","cash","gold","silver","money","wealth","rich","poor","king","queen",
    "prince","princess","duke","count","lord","lady","knight","sir","dame",
    "wizard","witch","mage","sorcerer","warlock","enchantress","druid","shaman",
    "priest","monk","nun","bishop","pope","caliph","sultan","emperor","empire",
    "pharaoh","caesar","tsar","shogun","samurai","ninja","ronin","geisha",
    "viking","pirate","buccaneer","corsair","privateer","captain","admiral",
    "general","marshal","colonel","major","lieutenant","sergeant","captain",
    "soldier","marine","sailor","pilot","aviator","navigator","scout","spy",
    "agent","detective","inspector","sheriff","marshal","judge","lawyer","doctor",
    "nurse","scientist","engineer","inventor","artist","musician","singer","dancer",
    "writer","poet","philosopher","teacher","professor","student","scholar","mage"
)
foreach ($p in $rockyouPasswords) {
    $lines += $p
    $lines += $p.ToUpper()
    $lines += $p.Substring(0,1).ToUpper() + $p.Substring(1)
    $lines += "$p!"
    $lines += "$p!!"
    $lines += "$p!!!"
    $lines += "$p123"
    $lines += "$p1"
    $lines += "$p!"
    foreach ($y in 2009..2020) { $lines += "$p$y" }
    $lines += "my $p"
    $lines += "the $p"
    $lines += "$p my"
    $lines += "$p bitcoin"
    $lines += "bitcoin $p"
}

Write-Host "Rockyou passwords done"

# === 5. Top English words ===
$englishWords = @(
    "the","be","to","of","and","a","in","that","have","it",
    "for","not","on","with","he","as","you","do","at","this",
    "but","his","by","from","they","we","say","her","she","or",
    "an","will","my","one","all","would","there","their","what","so",
    "up","out","if","about","who","get","which","go","me","when",
    "make","can","like","time","no","just","him","know","take","people",
    "into","year","your","good","some","could","them","see","other","than",
    "then","now","look","only","come","its","over","think","also","back",
    "after","use","two","how","our","work","first","well","way","even",
    "new","want","because","any","these","give","day","most","us","great",
    "between","need","large","end","under","never","city","tree","cross",
    "carry","born","every","white","house","right","think","boy","old",
    "too","mean","before","through","story","off","member","against","move",
    "night","point","find","long","both","little","us","keep","head","hearth",
    "word","begin","life","hand","eye","picture","change","next","small",
    "form","real","home","school","large","program","idea","children","seem",
    " together","air","good","death","father","open","seem","together","line",
    "never","free","busy","dark","full","empty","clean","dirty","hot","cold",
    "warm","cool","fast","slow","hard","soft","high","low","deep","wide",
    "narrow","thick","thin","long","short","tall","big","small","huge","tiny",
    "early","late","young","old","new","fresh","sweet","sour","bitter","salty",
    "spicy","mild","strong","weak","loud","quiet","bright","dim","clear","fuzzy",
    "smooth","rough","sharp","dull","wet","dry","light","heavy","easy","difficult",
    "simple","complex","common","rare","normal","strange","funny","serious","calm",
    "angry","sad","glad","proud","afraid","sure","alone","together","near","far",
    "love","hate","hope","fear","trust","dream","wish","peace","war","fight",
    "dance","sing","play","work","rest","sleep","wake","run","walk","stop",
    "start","begin","finish","open","close","break","build","create","destroy",
    "heal","kill","save","lose","win","find","seek","hide","show","tell",
    "ask","answer","learn","teach","read","write","speak","hear","listen",
    "watch","feel","touch","taste","smell","breathe","live","die","grow","change",
    "turn","fall","rise","fly","swim","climb","jump","push","pull","throw",
    "catch","hold","drop","carry","give","take","send","receive","buy","sell",
    "pay","cost","worth","value","price","money","power","force","energy","speed",
    "sound","noise","music","color","shape","size","weight","number","letter","name",
    "place","space","world","earth","nature","water","fire","earth","air","spirit",
    "god","devil","angel","demon","ghost","spirit","soul","mind","heart","brain",
    "body","blood","bone","skin","hair","face","hand","foot","head","arm","leg",
    "king","queen","prince","princess","knight","warrior","hero","villain","master",
    "servant","friend","enemy","lover","stranger","neighbor","family","parent","child",
    "brother","sister","mother","father","son","daughter","husband","wife","baby","pet",
    "dog","cat","bird","fish","horse","cow","pig","sheep","chicken","rabbit",
    "mouse","snake","frog","bear","wolf","fox","deer","eagle","shark","whale",
    "lion","tiger","elephant","monkey","dragon","phoenix","unicorn","griffin","giant",
    "dwarf","elf","troll","ogre","goblin","fairy","witch","wizard","mage","ninja",
    "samurai","viking","pirate","knight","soldier","captain","chief","leader","boss",
    "teacher","student","doctor","nurse","police","fireman","driver","pilot","sailor",
    "farmer","hunter","fisher","miner","builder","artist","musician","singer","writer",
    "thinker","dreamer","believer","follower","leader","ruler","judge","lawyer","guard",
    "watcher","seeker","finder","maker","breaker","healer","killer","savior","loser",
    "winner","player","gamer","racer","fighter","runner","walker","flyer","swimmer",
    "climber","jumper","dancer","singer","speaker","listener","reader","writer","learner",
    "teacher","helper","giver","taker","sender","receiver","buyer","seller","payer",
    "owner","renter","lender","borrower","spender","saver","gainer","loser","winner"
)
foreach ($w in $englishWords) {
    $w = $w.Trim()
    if (-not $w) { continue }
    $lines += $w
    $lines += $w.ToUpper()
    $lines += $w.Substring(0,1).ToUpper() + $w.Substring(1)
    $lines += "$w!"
    $lines += "$w123"
    foreach ($y in 2009..2015) { $lines += "$w$y" }
}

Write-Host "English words done"

# === 6. Color + Animal + Adjective combos ===
$colors = "red","blue","green","yellow","purple","orange","black","white","silver","gold","pink","brown","gray","cyan","magenta","violet","indigo","teal","maroon","navy","crimson","scarlet","amber","ivory","pearl","ruby","emerald","sapphire","bronze","copper","obsidian","cobalt","azure","cerulean","lavender","coral","salmon","olive","lime","turquoise","chocolate","vanilla","strawberry","cherry","lemon","mint","honey","sand","stone","ash","smoke","cloud","snow","ice","flame","storm","thunder","lightning","shadow","dark","bright","neon","electric","cosmic","galactic","stellar","lunar","solar","aurora","midnight","sunrise","sunset","twilight","dawn","dusk"
$animals = "dragon","phoenix","unicorn","panther","tiger","lion","bear","wolf","eagle","falcon","hawk","owl","raven","crow","fox","deer","horse","horse","whale","shark","dolphin","serpent","cobra","viper","scorpion","spider","butterfly","beetle","wasp","bee","ant","worm","snake","lizard","turtle","frog","toad","fish","salmon","trout","tuna","swordfish","jellyfish","starfish","crab","lobster","octopus","squid","seal","walrus","penguin","pelican","swan","duck","goose","crane","heron","stork","flamingo","parrot","macaw","toucan","hawk","condor","vulture","buzzard","osprey","kite","falcon","kestrel","sparrow","finch","robin","bluejay","cardinal","wren","thrush","nightingale","lark","swallow","martin","swift","hummingbird","woodpecker","chickadee","titmouse","jays","magpie","crow","raven","razen","raptor","predator","beast","creature","monster","fiend","fiend","spirit","ghost","phantom","wraith","shade","specter","apparition","entity","being","force","power","energy","essence","soul","spirit","mind","heart","core","center","source","origin","beginning","end","alpha","omega","prime","first","last","final","ultimate","supreme","divine","sacred","holy","blessed","cursed","damned","eternal","infinite","absolute","perfect","flawless","immortal","indestructible","invincible","unstoppable","unbreakable","unyielding","unforgiving","merciless","relentless","endless","limitless","boundless","timeless","ageless","deathless","lifeless","soulless","mindless","heartless","hopeless","fearless","senseless","meaningless","purposeless","directionless","homeless","nameless","faceless","voiceless","soundless","speechless","wordless","silent","quiet","still","calm","peaceful","serene","tranquil","placid","gentle","mild","soft","tender","warm","kind","loving","caring","nurturing","protecting","defending","guarding","watching","keeping","holding","bearing","carrying","supporting","sustaining","maintaining","preserving","conserving","saving","rescuing","freeing","liberating","releasing","delivering","redeeming","forgiving","healing","mending","fixing","repairing","restoring","renewing","reviving","refreshing","rejuvenating","regenerating","recreating","rebuilding","reconstructing","reassembling","reorganizing","restructuring","redesigning","remodeling","reshaping","refining","perfecting","improving","enhancing","upgrading","advancing","progressing","developing","evolving","growing","expanding","extending","stretching","reaching","touching","feeling","sensing","perceiving","understanding","knowing","learning","discovering","finding","locating","identifying","recognizing","acknowledging","accepting","embracing","welcoming","greeting","meeting","encountering","facing","confronting","challenging","questioning","doubting","wondering","asking","seeking","searching","exploring","investigating","examining","studying","analyzing","observing","watching","monitoring","tracking","following","pursuing","chasing","hunting","hounding","tracking","trailing","shadowing","stalking","haunting","tormenting","plaguing","troubling","bothering","annoying","irritating","infuriating","enraging","enflaming","igniting","lighting","burning","blazing","flaming","glowing","shining","radiating","emitting","projecting","casting","throwing","hurling","launching","firing","shooting","blasting","exploding","detonating","erupting","bursting","breaking","shattering","smashing","crushing","destroying","demolishing","ruining","wrecking","devastating","ravaging","sacking","plundering","looting","pillaging","raiding","attacking","assaulting","invading","conquering","subduing","defeating","overcoming","surmounting","surpassing","exceeding","transcending","outshining","outperforming","outclassing","outmatching","outsmarting","outwitting","outthinking","outplanning","outmaneuvering","outflanking","outlining","outlining"
$adjectives = "great","big","small","large","tiny","huge","massive","enormous","gigantic","colossal","immense","vast","infinite","eternal","ancient","modern","classic","vintage","retro","future","past","present","golden","silver","bronze","crystal","diamond","emerald","ruby","sapphire","pearl","ivory","jade","obsidian","marble","granite","steel","iron","copper","titanium","platinum","titanium","cosmic","galactic","stellar","lunar","solar","nebula","quantum","atomic","molecular","nuclear","plasma","electric","magnetic","gravitational","thermal","kinetic","potential","dynamic","static","active","passive","positive","negative","neutral","balanced","chaotic","ordered","random","patterned","structured","organized","systematic","methodical","logical","rational","reasonable","sensible","practical","realistic","idealistic","optimistic","pessimistic","romantic","realistic","fantastic","magical","mystical","mysterious","secret","hidden","invisible","visible","apparent","obvious","clear","obscure","vague","ambiguous","uncertain","doubtful","questionable","probable","possible","impossible","certain","sure","confident","proud","humble","modest","arrogant","bold","brave","courageous","fearless","daring","adventurous","reckless","careless","thoughtful","mindful","aware","conscious","awake","alert","vigilant","watchful","observant","perceptive","insightful","intuitive","instinctive","natural","organic","pure","clean","fresh","raw","wild","untamed","free","liberated","unbound","unleashed","unfettered","unrestricted","unlimited","unbounded","endless","infinite","eternal","everlasting","permanent","temporary","fleeting","momentary","brief","short","long","extended","prolonged","delayed","early","late","prompt","timely","urgent","critical","essential","vital","crucial","important","significant","meaningful","valuable","precious","priceless","invaluable","worthless","useless","helpful","harmful","beneficial","damaging","destructive","creative","productive","constructive","generative","regenerative","transformative","revolutionary","evolutionary","progressive","conservative","traditional","innovative","original","unique","special","extraordinary","remarkable","exceptional","outstanding","excellent","superior","premium","quality","elite","select","chosen","elected","appointed","designated","named","titled","crowned","honored","respected","admired","loved","cherished","treasured","prized","valued","appreciated","recognized","acknowledged","celebrated","famous","renowned","notorious","infamous","legendary","mythical","historical","biblical","sacred","holy","divine","spiritual","religious","secular","worldly","earthly","heavenly","angelic","demonic","diabolical","infernal","hellish","nightmarish","terrifying","horrible","terrible","awful","dreadful","frightful","fearful","scary","creepy","eerie","spooky","haunting","unsettling","disturbing","alarming","shocking","startling","surprising","amazing","astonishing","astounding","incredible","unbelievable","fantastic","marvelous","wonderful","magnificent","splendid","glorious","majestic","regal","royal","imperial","noble","dignified","honorable","respectable","decent","proper","correct","right","true","accurate","precise","exact","perfect","flawless","impeccable","pristine","spotless","immaculate","untouched","unspoiled","virgin","fresh","new","novel","original","fresh"

foreach ($c in $colors) {
    foreach ($a in $animals) {
        $lines += "$c $a"
        $lines += "$c$a"
        $lines += "$c-$a"
        $lines += "$c_$a"
    }
}

Write-Host "Color+Animal combos done"

# === 7. First names + Last names (top combinations) ===
$firstNames = "james","john","robert","michael","william","david","richard","joseph","thomas","charles","christopher","daniel","matthew","anthony","mark","donald","steven","paul","andrew","joshua","Kenneth","kevin","brian","george","timothy","jason","jeffrey","ryan","jacob","gary","nicholas","eric","jonathan","stephen","larry","justin","scott","brandon","benjamin","samuel","reginald","patrick","frank","alexander","jack","dennis","jerry","tyler","aaron","jose","adam","nathan","henry","pete","zy","doyle","walter","mario","arthur","lawrence","lee","january","july","august","september","october","november","december","saturday","sunday","monday","tuesday","wednesday","thursday","friday"
$lastNames = "smith","johnson","williams","brown","jones","garcia","miller","davis","rodriguez","martinez","hernandez","lopez","gonzalez","wilson","anderson","thomas","taylor","moore","jackson","martin","lee","perez","thompson","white","harris","sanchez","clark","ramirez","lewis","robinson","walker","young","allen","king","wright","scott","torres","nguyen","hill","flores","green","adams","nelson","baker","hall","rivera","campbell","mitchell","carter","roberts","gomez","phillips","evans","turner","diaz","parker","cruz","edwards","collins","reyes","stewart","morris","morales","murphy","cook","rogers","gutierrez","ortiz","murray","ward","cox","howard","ward","peterson","gray","ramsey","watson","brooks","kelly","sanders","price","bennett","wood","barnes","ross","henderson","coleman","jenkins","perry","powell","long","patterson","hughes","flores","butler","simmons","foster","gonzales","bryant","alexander","russell","griffin","hayes","chavez","jimenez","castillo","wang","vasquez","mendoza","moreno","ford","hunt","benson","bishop","stone","hawkins","dunn","medina","fowler","willis","webb","simpson","stevens","tucker","porter","harrison","huff","hudson","spencer","gardner","stephens","payne","redding","holmes","walls","wade","sullivan","cummings","estes","jordan","patton","mccarthy","wells","curtis","romero","hogan","hart","elliott","cunningham","avila","blair","hicks","hunt","gilbert","garrett","romero","wilkins","munoz","moss","crawford","boyd","mason","morin","freeman","wells","webb","neal","may","stevens","berry","hopkins","errera","myers","jennings","barnett","ferguson","salazar","wade","wheeler","larson","liberman","frazier","burton","norton","harrington","salinas","zimmerman","elliott","goodman","maldonado","yates","becker","erickson","hobbs","mckinney","lucas","miles","crawford","llama","shelton","aguirre","tanner","powers","barker","gordon","shaw","holman","rice","black","fisher","horton","christensen","cline","baldwin","gillespie","holland","ramos","weaver","livingston","bates","austin","pierce","johnson","jensen","mcdonald","perry","castro","sutton","winters","greer","lloyd","fields","gallagher","mcdaniel","browning","barber","baxter","hale","hubbard","sawyer","knight","carr","mcdowell","marsh","mccormick","dean","bradley","poole","bass","franklin","logan","blake","cameron","mathis","singleton","richards","schmidt","cortez","christiansen","hensley","rojas","hardin","mccoy","newton","blanchard","jennings","barrett","nash","thornton","mcgee","morrow","dixon","page","cannon","gates","lowery","mcdonald","marsh","gaines","hinton","hopper","merritt","mccall","mccarthy","sweeney","wiggins","mathews","mcknight","dickerson","winters","gould","horne","bender","mcbride","mcleod","mcclain","mcclure","mckee","mcguire","mcmahon","mccullough","mccarty","mcintosh","mcfarland","mckay","mclaughlin","mcelroy","mcconnell","mcclellan","mcfadden","mcginnis","mckenna","mckinley","mclendon","mcneil","mcfarlane","mcgill","mckay","mclellan","mcfarlin","mckee","mcguire","mcmillan","mcmullen","mcmurray","mcmahon"

foreach ($fn in $firstNames) {
    foreach ($ln in $lastNames) {
        $lines += "$fn$ln"
        $lines += "$fn $ln"
        $lines += "$fn.$ln"
        $lines += "$fn_$ln"
        $lines += "$fn-$ln"
        $lines += "$fn$ln2009"
        $lines += "$fn$ln!"
    }
}

Write-Host "Names combos done"

# === 8. Bitcoin-specific dates ===
$bitcoinDates = @(
    # Genesis block
    "20090103","01032009","03012009","2009-01-03","01-03-2009","03-01-2009",
    "2009/01/03","01/03/2009","03/01/2009",
    "january 3 2009","jan 3 2009","3 january 2009","3 jan 2009",
    "3rd january 2009","january third 2009",
    # First transaction (Satoshi to Hal)
    "20090112","01122009","12012009","2009-01-12","01-12-2009","12-01-2009",
    "january 12 2009","jan 12 2009",
    # Bitcoin whitepaper published
    "20081031","10312008","31102008","2008-10-31","10-31-2008","31-10-2008",
    "october 31 2008","oct 31 2008",
    # MTGox founded
    "20100701","07012010","01072010","2010-07-01","07-01-2010",
    "july 1 2010","july first 2010",
    # First BTC price (NewBTC)
    "20100717","07172010","17072010","2010-07-17","07-17-2010",
    "july 17 2010","bitcoin pizza day",
    # Bitcoin pizza day
    "20100522","05222010","22052010","2010-05-22","05-22-2010",
    "may 22 2010","bitcoin pizza",
    # MtGox hack
    "201107","july 2011","201108","august 2011",
    # First halving
    "20121128","11282012","28112012","2012-11-28","11-28-2012",
    "november 28 2012","first halving",
    # Second halving
    "20160709","07092016","09072016","2016-07-09","07-09-2016",
    "july 9 2016","second halving",
    # Third halving
    "20200511","05112020","11052020","2020-05-11","05-11-2020",
    "may 11 2020","third halving",
    # Fourth halving
    "20240420","04202024","20042024","2024-04-20","04-20-2024",
    "april 20 2024","fourth halving",
    # Block heights
    "210000","336000","480000","600000","780000","840000"
)
foreach ($d in $bitcoinDates) {
    $lines += $d
    $lines += "$d!"
    $lines += "$d bitcoin"
    $lines += "bitcoin $d"
}

Write-Host "Bitcoin dates done"

# === 9. Common first names alone (top 500) ===
$commonFirstNames = @(
    "james","john","robert","michael","william","david","richard","joseph","thomas","charles",
    "chris","dan","matthew","anthony","mark","donald","steven","paul","andrew","joshua",
    "kenneth","kevin","brian","george","timothy","jason","jeffrey","ryan","jacob","gary",
    "nicholas","eric","jonathan","stephen","larry","justin","scott","brandon","benjamin","samuel",
    "gregory","frank","alexander","jack","dennis","jerry","tyler","aaron","jose","adam",
    "nathan","henry","pete","doyle","walter","mario","arthur","lawrence","lee","january",
    "july","august","september","october","november","december","saturday","sunday","monday","tuesday",
    "wednesday","thursday","friday",
    "jennifer","linda","barbara","patricia","jessica","sarah","karen","nancy","lisa","betty",
    "margaret","sandra","ashley","dorothy","kimberly","emily","donna","miller","melissa","deborah",
    "stephanie","rebecca","sharon","lauren","cynthia","kathleen","amy","angela","shirley","anna",
    "brenda","pamela","emma","nicole","helen","samantha","katherine","christine","marie","debra",
    "amanda","rachel","catherine","heather","diana","ruth","janet","olivia","julie","joyce",
    "virginia","kelly","lauren","christina","kathryn","joan","evelyn","judith","megan","amber",
    "alexis","oliver","sophia","isabella","mia","charlotte","amelia","harper","evelyn","abigail",
    "emilia","ella","avery","sofia","camila","aria","scarlett","victoria","madison","luna",
    "grace","chloe","penelope","layla","riley","zoey","nora","lily","eleanor","hannah",
    "hazel","violet","aurora","savannah","audrey","brooke","bella","karen","storm","maria",
    "esther","alyssa","josephine","megan","madelyn","khloe","serenity","aria","alice","madison",
    "kaylee","patricia","emerson","elena","quinn","nevaeh","harmony","angelina","ada","ivy",
    "lucy","piper","lydia","rose","allison","maya","genesis","harley","emery","anastasia",
    "eliana","julia","caroline","nova","sadie","isla","eliza","charlie","emilie","kennedy",
    "willow","sara","arianna","mary","athena","autumn","alana","cora","hailey","trinity"
)
foreach ($n in $commonFirstNames) {
    $lines += $n
    $lines += $n.ToUpper()
    $lines += $n.Substring(0,1).ToUpper() + $n.Substring(1)
    $lines += "$n!"
    $lines += "$n123"
    foreach ($y in 2009..2020) { $lines += "$n$y" }
    foreach ($nn in 1..99) { $lines += "$n$nn" }
}

Write-Host "Common first names done"

# === 10. Pet names ===
$petNames = @(
    "max","bella","charlie","lucky","cooper","beauty","sadie","tucker","annie","bailey",
    "lexi","bear","cecilia","jack","lola","molly","pepper","martha","toby","rover",
    "teddy","lucy","maggie","rocky","daisy","milo","chloe","winston","sasha","mocha",
    "simba","grace","shadow","gizmo","nala","titan","penelope","mocha","sunny","ginger",
    "scooby","spirit","nemo","patches","midnight","tinkerbell","slinky","muffin","princess",
    "king","duke","prince","royal","diamond","star","jewel","gem","treasure","goldie",
    "silver","copper","bronze","rusty","spot","freckles","dots","buttons","bubbles","fuzzy",
    "fluffy","fido","rex","spot","shep","shepherd","collie","lab","labrador","golden",
    "retriever","poodle","terrier","husky","pug","bulldog","boxer","rottweiler","doberman",
    "german shepherd","pomeranian","dachshund","beagle","corgi","samoyed","akita","maltese",
    "shih tzu","chihuahua","dalmatian","greyhound","whippet","border collie","australian shepherd",
    "french bulldog","english bulldog","great dane","bernese mountain dog","newfoundland","st bernard",
    "irish setter","golden retriever","yellow lab","chocolate lab","black lab","labrador retriever",
    "persian","siamese","maine coon","ragdoll","british shorthair","scottish fold","bengal",
    "sphynx","russian blue","abyssinian","somali","burmese","exotic shorthair","himalayan",
    "norwegian forest","siberian","turkish angora","turkish van","cornish rex","devon rex",
    "manx","ocicat","savannah","tonkinese","american shorthair","american bobtail","colorpoint",
    "munchkin","singaporese","burmese","burmesa","japanese bobtail","korat","laPerm","ragamuffin",
    "selkirk rex","snowshoe","somali","sphynx","tonkinese","turkish angola"
)
foreach ($p in $petNames) {
    $lines += $p
    $lines += $p.ToUpper()
    $lines += $p.Substring(0,1).ToUpper() + $p.Substring(1)
    $lines += "$p!"
    $lines += "$p123"
    foreach ($y in 2009..2020) { $lines += "$p$y" }
    $lines += "my $p"
    $lines += "$p my"
}

Write-Host "Pet names done"

# === 11. Abbreviations / Acronyms ===
$acronyms = @(
    "USA","UK","UN","EU","NATO","OTAN","CIA","FBI","NSA","CIA",
    "BBC","CNN","ABC","CBS","NBC","FOX","AP","UPI","Reuters","AFP",
    "IBM","Intel","AMD","NVIDIA","Qualcomm","Motorola","Samsung","LG","Sony","Panasonic",
    "NASA","ESA","JAXA","ISRO","CNSA","Roscosmos","DARPA","DoD","Pentagon","White House",
    "FIFA","NBA","NFL","MLB","NHL","UEFA","F1","WTA","ATP","PGA",
    "GDP","GDP","GNP","PIB","BIP","OKP","VAT","GST","TVA","IVA",
    "CEO","COO","CFO","CTO","CIO","CMO","CPO","CSO","CISO","CDO",
    "VP","SVP","EVP","MD","GM","PM","SM","DM","AM","BM",
    "AI","ML","DL","NN","NLP","CV","RL","GAN","RNN","LSTM",
    "TCP","IP","HTTP","HTTPS","FTP","SSH","SSL","TLS","DNS","API",
    "URL","URI","URN","URN","URN","URN","URN","URN","URN","URN",
    "RAM","ROM","CPU","GPU","TPU","FPGA","ASIC","SSD","HDD","NVMe",
    "USB","HDMI","VGA","DVI","DP","Thunderbolt","FireWire","SATA","SAS","SCSI",
    "WiFi","Bluetooth","Zigbee","Z-Wave","LoRa","NB-IoT","LTE","5G","4G","3G",
    "GPS","GLONASS","Galileo","BeiDou","QZSS","NavIC","IRNSS","DGPS","RTK","PPK",
    "PDF","PNG","JPEG","GIF","SVG","WebP","BMP","TIFF","ICO","PSD",
    "MP3","MP4","AVI","MKV","FLV","WMV","MOV","OGG","WAV","FLAC",
    "HTML","CSS","JS","TS","JSON","XML","YAML","TOML","CSV","SQL",
    "PHP","Python","Ruby","Java","C","C++","C#","Go","Rust","Swift",
    "Kotlin","Dart","Scala","Clojure","Haskell","Erlang","Elixir","F#","R","Julia",
    "Linux","Windows","macOS","iOS","Android","ChromeOS","FreeBSD","OpenBSD","NetBSD","DragonFly",
    "Ubuntu","Debian","Fedora","Arch","Gentoo","Slackware","Mint","Pop","Manjaro","EndeavourOS",
    "React","Angular","Vue","Svelte","Next","Nuxt","Gatsby","Remix","Ember","Backbone",
    "Node","Express","Django","Flask","Rails","Laravel","Spring","Hibernate","ASP","PHP",
    "Docker","Kubernetes","Helm","Terraform","Ansible","Chef","Puppet","Salt","Vagrant","Nomad",
    "AWS","Azure","GCP","Oracle","IBM Cloud","DigitalOcean","Linode","Vultr","Heroku","Vercel",
    "Redis","MongoDB","PostgreSQL","MySQL","MariaDB","SQLite","Cassandra","CockroachDB","Neo4j","Elasticsearch",
    "Kafka","RabbitMQ","NATS","ZeroMQ","gRPC","GraphQL","REST","SOAP","XML-RPC","JSON-RPC",
    "OAuth","JWT","SAML","OpenID","LDAP","Kerberos","RADIUS","TACACS","PAM","SELinux",
    "AES","RSA","ECC","DSA","ECDSA","EdDSA","Ed25519","X25519","ChaCha20","Salsa20",
    "SHA1","SHA256","SHA512","MD5","RIPEMD","BLAKE2","BLAKE3","Whirlpool","Tiger","Snefru",
    "HMAC","CMAC","GMAC","PMAC","KMAC","HKDF","PBKDF2","bcrypt","scrypt","Argon2",
    "WIF","P2PKH","P2SH","P2WPKH","P2TR","P2WSH","BIP32","BIP39","BIP44","BIP84",
    "HD","SECP256K1","S256","K1","256","K256","P256","P384","P521","X25519"
)
foreach ($a in $acronyms) {
    $lines += $a
    $lines += $a.ToLower()
    $lines += "$a!"
    $lines += "$a123"
}

Write-Host "Acronyms done"

# === 12. Brand slogans ===
$slogans = @(
    "just do it","think different","impossible is nothing",
    "the happiest place on earth","i'm lovin it",
    "finger lickin good","melts in your mouth not in your hand",
    "the quick red fox","have it your way",
    "because you're worth it","obey your thirst",
    "taste the rainbow","the best a man can get",
    "a diamond is forever","the ultimate driving machine",
    "beauty is only skin deep","turns you on to nature",
    "grace space and pace","built ford tough",
    "the silverado way","put a tiger in your tank",
    "drinking mbiras is like drinking molten glass",
    "id rather have a heineken","good to the last drop",
    "the pause that refreshes","obey your thirst",
    "when you care enough to send the very best",
    "the real thing","unleash the power",
    "dare to be great","the heart of it all",
    "life is a cocktail","because its true",
    "the new you","its the real thing",
    "the ultimate driving machine","beauty is only skin deep",
    "turns you on to nature","grace space and pace",
    "built ford tough","the silverado way",
    "put a tiger in your tank","id rather have a heineken",
    "good to the last drop","the pause that refreshes",
    "when you care enough to send the very best",
    "the real thing","unleash the power","dare to be great",
    "the heart of it all","life is a cocktail",
    "because its true","the new you","its the real thing",
    "made to move","engineered to perform",
    "performance all day","the power of choice",
    "driving pleasure","the art of progress",
    "vorfreude","joy of driving","engineering wonder",
    "the future is electric","accelerating the world",
    "sustainable energy","clean energy","green energy",
    "renewable energy","smart energy","efficient energy",
    "zero emissions","carbon neutral","carbon footprint",
    "climate change","global warming","environmental",
    "eco friendly","sustainable","recyclable","biodegradable",
    "compostable","organic","natural","holistic",
    "wellness","health","fitness","nutrition",
    "supplements","vitamins","minerals","proteins",
    "carbohydrates","fats","fiber","water",
    "hydration","electrolytes","antioxidants","probiotics",
    "prebiotics","synbiotics","postbiotics","metabolism",
    "calories","macros","micros","nutrition",
    "diet","exercise","workout","training",
    "cardio","strength","endurance","flexibility",
    "balance","coordination","agility","speed",
    "power","explosiveness","reaction time","reflexes",
    "stamina","outstamina","outperform","excel",
    "achieve","succeed","triumph","conquer",
    "overcome","surmount","transcend","evolve",
    "grow","develop","progress","advance",
    "improve","enhance","upgrade","optimize",
    "maximize","minimize","balance","harmonize",
    "synchronize","align","connect","unite",
    "merge","combine","integrate","unify",
    "consolidate","centralize","decentralize","distribute",
    "disperse","scatter","spread","expand",
    "extend","stretch","reach","touch",
    "feel","sense","perceive","understand",
    "comprehend","grasp","apprehend","realize",
    "recognize","acknowledge","accept","embrace",
    "welcome","greet","meet","encounter",
    "face","confront","challenge","question",
    "doubt","wonder","ask","seek",
    "search","explore","investigate","examine",
    "study","analyze","observe","watch",
    "monitor","track","follow","pursue",
    "chase","hunt","hound","trail",
    "shadow","stalk","haunt","torment",
    "plague","trouble","bother","annoy",
    "irritate","infuriate","enrage","enflame",
    "ignite","light","burn","blaze",
    "flame","glow","shine","radiate",
    "emit","project","cast","throw",
    "hurl","launch","fire","shoot",
    "blast","explode","detonate","erupt",
    "burst","break","shatter","smash",
    "crush","destroy","demolish","ruin",
    "wreck","devastate","ravage","sack",
    "plunder","loot","pillage","raid",
    "attack","assault","invade","conquer",
    "subdue","defeat","overcome","surmount",
    "surpass","exceed","transcend","outshine",
    "outperform","outclass","outmatch","outsmart",
    "outwit","outthink","outplan","outmaneuver",
    "outflank","outline","outweigh","outlast",
    "outlive","outgrow","outnumber","outproduce",
    "outearn","outspend","outsave","outgive",
    "outdo","outperform","excel","succeed",
    "triumph","conquer","achieve","accomplish",
    "complete","finish","end","conclude",
    "terminate","cease","stop","halt",
    "pause","rest","relax","unwind",
    "decompress","destress","calm","soothe",
    "comfort","console","reassure","encourage",
    "inspire","motivate","stimulate","energize",
    "invigorate","revitalize","rejuvenate","refresh",
    "renew","restore","revive","resurrect",
    "regenerate","recreate","rebuild","reconstruct",
    "reassemble","reorganize","restructure","redesign",
    "remodel","reshape","refine","perfect",
    "improve","enhance","upgrade","advance",
    "progress","develop","evolve","grow",
    "expand","extend","stretch","reach",
    "touch","feel","sense","perceive",
    "understand","comprehend","grasp","apprehend",
    "realize","recognize","acknowledge","accept",
    "embrace","welcome","greet","meet"
)
foreach ($s in $slogans) {
    $lines += $s
    $lines += $s.ToLower()
    $lines += $s.ToUpper()
    $lines += "$s!"
}

Write-Host "Slogans done"

# === 13. Single BIP39 words with all common suffixes ===
$bip39Words = Get-Content "Y:\btcsolver\bip39-words.txt" -ErrorAction SilentlyContinue | Where-Object { $_ -and $_.Trim() }
if ($bip39Words) {
    foreach ($w in $bip39Words) {
        $w = $w.Trim()
        if (-not $w) { continue }
        # Already in corpus, but add more variations
        $lines += "$w!"
        $lines += "$w!!"
        $lines += "$w!!!"
        $lines += "$w1"
        $lines += "$w12"
        $lines += "$w123"
        $lines += "$w1234"
        $lines += "$w12345"
        $lines += "$w123456"
        $lines += "$w bitcoin"
        $lines += "$w wallet"
        $lines += "$w key"
        $lines += "$w seed"
        $lines += "my $w"
        $lines += "the $w"
        foreach ($y in 2009..2015) { $lines += "$w$y" }
    }
}

Write-Host "BIP39 extended variations done"

# === 14. Common French phrases ===
$frenchPhrases = @(
    "liberte egalite fraternite","la vie est belle","amour vrai",
    "je t'aime","je t'aime moi non plus","c'est la vie",
    "ouf c'est la vie","tout va bien","rien ne va plus",
    "a bientot","au revoir","bonjour","bonsoir","bonne nuit",
    "merci beaucoup","s'il vous plait","excusez moi","pardon",
    "je suis francais","je suis franais","la france",
    "paris","marseille","lyon","toulouse","nice",
    "nantes","montpellier","strasbourg","bordeaux","lille",
    "rennes","rouen","le mans","dijon","grenoble",
    "clermont ferrand","amiens","metz","tours","poitiers",
    "la republiche francaise","vive la france","la marseillaise",
    "allons enfants de la patrie","le drapeau tricolore",
    "bleu blanc rouge","la tour eiffel","le louvre",
    "versailles","mont saint michel","chateau de chillon",
    "la croix de lorrain","le coq gaulois","la baguette",
    "le croissant","le fromage","le vin","le champagne",
    "le cognac","l'eau de vie","la bordeaux","le burgundy",
    "la chateauroux","le bordeaux","le saint emilion",
    "le pauillac","le margaux","le petrus","le chateau latour",
    "le chateau margaux","le chateau Lafite","le chateau Mouton",
    "le chateau Haut Brion","le chateau Cheval Blanc",
    "le chateau Angelus","le chateau Pavie","le chateau Ausone",
    "la vie en rose","la mer","le temps","l'amour",
    "la mort","la paix","la guerre","la liberté",
    "l'égalité","la fraternité","la justice","la vérité",
    "la sagesse","la connaissance","la science","la raison",
    "l'esprit","le corps","l'ame","le coeur","l'avenir",
    "le passe","le present","le temps","l'infini",
    "l'eternité","le ciel","la terre","la mer","la montagne",
    "la foret","le jardin","la ville","la campagne","le village",
    "maison","ma voiture","ma famille","mes enfants","mon chien",
    "mon chat","mon ami","mon amour","ma vie","mon coeur",
    "bitcoin france","bitcoin paris","btc france","crypto france",
    "monetie numerique","liberte financiere","argent numerique",
    "monnaie digitale","crypto monnaie","blockchain france",
    "clé privée","clé publique","portefeuille bitcoin",
    "phrase de recuperation","mnemonique","graine bitcoin"
)
foreach ($f in $frenchPhrases) {
    $lines += $f
    $lines += $f.ToLower()
    $lines += $f.ToUpper()
    $lines += "$f!"
}

Write-Host "French phrases done"

# === 15. Common German phrases ===
$germanPhrases = @(
    "einigkeit und recht und freiheit","die deutsche republik",
    "deutschland","berlin","munchen","hamburg","koln","frankfurt",
    "stuttgart","dusseldorf","dresden","leipzig","hannover","nurnberg",
    "braunschweig","wuppertal","bielefeld","bonn","munster","mannheim",
    "augsburg","wiesbaden","gelsenkirchen","munchen","karlsruhe","heidelberg",
    "ulm","regensburg","inf","oldenburg","lubeck","oberhausen","remagel",
    "heidelberg","mainz","krefeld","leverkusen","osnabruck","solingen",
    "halle","saarbrucken","herne","mülheim","paderborn","lum","reutlingen",
    "dachau","aachen","potsdam","tubingen","brem","kassel","cottbus",
    "freiburg","bingen","trier","göttingen","jena","zwickau","heilbronn",
    "bitcoin deutschland","btc deutschland","krypto deutschland",
    "digitales geld","kryptowährung","bitcoin wallet deutsch",
    "privater schlüssel","öffentlicher schlüssel","passwort",
    "geheimnis","geheim","sicherheit","freiheit","geld",
    "freiheit","gluck","liebe","leben","tod","frieden","krieg",
    "wahrheit","weisheit","wissen","wissenschaft","logik","sinn",
    "gebet","gott","himml","erde","see","berg","wald","garten",
    "stadt","dorf","haus","auto","familie","kinder","hund","katze",
    "freund","liebe","leben","herz","zukunft","vergangenheit","gegenwart",
    "zeit","unendlichkeit","ewigkeit","himmel","see","berg","wald",
    "garten","stadt","land","dorf","haus","auto","familie","kinder",
    "hund","katze","freund","liebe","leben","herz","zukunft",
    "mein bitcoin","mein wallet","mein geheimnis","mein schatz",
    "erste bitcoin","bitcoin 2009","genesis block deutsch",
    "halving deutsch","block belohnung","mining deutsch",
    "schwierigkeit","nonce","merkle","hash","sha256",
    "doppelte ausgabe","51 prozent angegriffen","gabel","soft fork",
    "hard fork","UTXO","script","op return","p2pkh","p2sh","p2wpkh","p2tr"
)
foreach ($g in $germanPhrases) {
    $lines += $g
    $lines += $g.ToLower()
    $lines += $g.ToUpper()
    $lines += "$g!"
}

Write-Host "German phrases done"

# === 16. Common Spanish phrases ===
$spanishPhrases = @(
    "patria libertad igualdad","la vida es bella","amor verdadero",
    "te quiero","te amo","la vida","la muerte","la paz","la guerra",
    "la libertad","la igualdad","la fraternidad","la justicia","la verdad",
    "la sabiduría","el conocimiento","la ciencia","la razón","el espíritu",
    "el cuerpo","el alma","el corazón","el futuro","el pasado","el presente",
    "el tiempo","el infinito","la eternidad","el cielo","la tierra","el mar",
    "la montaña","el bosque","el jardín","la ciudad","el campo","el pueblo",
    "casa","coche","familia","hijos","perro","gato","amigo","amor","vida",
    "corazón","bitcoin españa","btc españa","cripto españa",
    "dinero digital","libertad financiera","moneda digital",
    "criptomoneda","blockchain españa","clave privada","clave publica",
    "billetera bitcoin","frase de recuperacion","semilla bitcoin",
    "mi bitcoin","mi wallet","mi secreto","mi tesoro",
    "primer bitcoin","bitcoin 2009","bloque genesis",
    "primera bitcoin","bitcoin 2009","bloque genesis",
    "halving","recompensa de bloque","minería",
    "dificultad","nonce","merkle","hash","sha256",
    "doble gasto","ataque del 51 por ciento","bifurcación","soft fork",
    "hard fork","UTXO","script","op return","p2pkh","p2sh","p2wpkh","p2tr",
    "madrid","barcelona","valencia","sevilla","zaragoza","malaga",
    "murcia","las palmas","bilbao","almeria","cordoba","granada",
    "vigo","gijon","hospital","valladolid","san sebastian","alcala",
    "algemesi","bogota","caracas","lima","mexico city","buenos aires",
    "santiago","quito","la paz","montevideo","san jose","havana",
    "santo domingo","kingston","port au prince","nassau","bridgetown",
    "castries","rosario","cordoba argentine","mendoza","tucuman","salta",
    "mar del plata","cordoba","rosario","mendoza","tucuman","salta"
)
foreach ($s in $spanishPhrases) {
    $lines += $s
    $lines += $s.ToLower()
    $lines += $s.ToUpper()
    $lines += "$s!"
}

Write-Host "Spanish phrases done"

# === 17. Phone number patterns (US format) ===
# Common patterns: 555-01XX, area codes
foreach ($area in "212","213","310","312","415","416","503","504","505","617","619","650","702","703","704","706","707","713","714","718","720","727","732","734","737","740","747","754","757","760","763","765","769","770","772","773","774","775","779","781","785","786","801","802","803","804","805","806","808","810","812","813","814","815","816","817","818","828","830","831","832","843","845","847","848","850","856","857","858","859","860","862","863","864","865","870","872","878","901","903","904","906","907","908","909","910","912","913","914","915","916","917","918","919","920","925","928","929","930","931","934","936","937","938","940","941","947","949","951","952","954","956","959","970","971","972","973","975","978","979","980","984","985","989") {
    foreach ($prefix in "555","100","200","300","400","500","600","700","800","900") {
        foreach ($line in "0000","0100","0101","0110","0111","0112","0113","0114","0115","0116","0117","0118","0119","0120","0200","0211","0222","0300","0333","0400","0444","0500","0555","0600","0666","0700","0777","0800","0888","0900","0999","1000","1100","1111","1200","1212","1234","1234","1300","1337","1400","1500","1600","1700","1800","1900","2000","2100","2200","2300","2400","2500","2600","2700","2800","2900","3000","3100","3200","3300","3400","3500","3600","3700","3800","3900","4000","4100","4200","4300","4400","4500","4600","4700","4800","4900","5000","5100","5200","5300","5400","5500","5555","5600","5700","5800","5900","6000","6100","6200","6300","6400","6500","6600","6666","6700","6800","6900","7000","7100","7200","7300","7400","7500","7600","7700","7777","7800","7900","8000","8100","8200","8300","8400","8500","8600","8700","8800","8888","8900","9000","9100","9200","9300","9400","9500","9600","9700","9800","9900","9999") {
            $lines += "$area$prefix$line"
            $lines += "$area$prefix$line".Replace("-", "").Replace(" ", "")
        }
    }
}

Write-Host "Phone patterns done"

# === 18. Dice patterns (D&D style) ===
# Common dice result patterns
foreach ($d4a in 1..4) { foreach ($d4b in 1..4) { foreach ($d4c in 1..4) { foreach ($d4d in 1..4) { $lines += "$d4a$d4b$d4c$d4d" } } } }
foreach ($d6a in 1..6) { foreach ($d6b in 1..6) { foreach ($d6c in 1..6) { foreach ($d6d in 1..6) { $lines += "$d6a$d6b$d6c$d6d" } } } }
# 2d20 results
foreach ($d20a in 1..20) { foreach ($d20b in 1..20) { $lines += "$d20a$d20b" } }
# Common dice sums
foreach ($s in 2..24) { $lines += "$s" }
# D&D character stats (3d6 each, 6 stats)
foreach ($s1 in 3..18) { foreach ($s2 in 3..18) { foreach ($s3 in 3..18) { $lines += "$s1$s2$s3" } } }

Write-Host "Dice patterns done"

# === 19. Wallet software default passwords ===
$walletDefaults = @(
    "electrum","electrum123","electrum wallet","electrum default",
    "armory","armory123","armory wallet","armory default",
    "multibit","multibit123","multibit wallet","multibit default",
    "bitcoin-qt","bitcoin qt","bitcoin core","bitcoin core wallet",
    "bitcoin wallet","btc wallet","btc wallet default",
    "coinbase","coinbase123","coinbase wallet","coinbase default",
    "blockchain.info","blockchain wallet","blockchain wallet default",
    "exodus","exodus123","exodus wallet","exodus default",
    "trust wallet","trust wallet default","trust wallet password",
    "ledger","ledger live","ledger nano","ledger nano s","ledger nano x",
    "trezor","trezor one","trezor model t","trezor safe 3",
    "keepkey","keepkey default","keepkey password",
    "digital bitbox","bitbox01","bitbox02","bitbox default",
    "coldcard","coldcard default","coldcard password",
    "jade","blockstream jade","jade default",
    "blue wallet","blue wallet default","blue wallet password",
    "samourai","samourai wallet","samourai default",
    "wasabi","wasabi wallet","wasabi default",
    "sparrow","sparrow wallet","sparrow default",
    "blockstream green","green wallet","green default",
    "mycelium","mycelium wallet","mycelium default",
    "bread wallet","bread wallet default","bread wallet password",
    "unstoppable","unstoppable wallet","unstoppable default",
    "atomic","atomic swap","atomic default",
    "changelly","changelly wallet","changelly default",
    "simplex","simplex wallet","simplex default",
    "transak","transak wallet","transak default",
    "moonpay","moonpay wallet","moonpay default",
    "ramp","ramp wallet","ramp default",
    "coinmama","coinmama wallet","coinmama default",
    "bitspace","bitspace wallet","bitspace default",
    "bitpanda","bitpanda wallet","bitpanda default",
    "cointracking","cointracking wallet","cointracking default",
    "koinly","koinly wallet","koinly default",
    "cointracker","cointracker wallet","cointracker default",
    "accruint","accruint wallet","accruint default",
    "token tax","token tax wallet","token tax default",
    "cryptocom","crypto.com wallet","crypto.com default",
    "binance","binance wallet","binance default",
    "kraken","kraken wallet","kraken default",
    "ftx","ftx wallet","ftx default",
    "okx","okx wallet","okx default",
    "huobi","huobi wallet","huobi default",
    "kucoin","kucoin wallet","kucoin default",
    "bybit","bybit wallet","bybit default",
    "gate.io","gate.io wallet","gate.io default",
    "mexc","mexc wallet","mexc default",
    "bitget","bitget wallet","bitget default",
    "bitmart","bitmart wallet","bitmart default",
    "phemex","phemex wallet","phemex default",
    "deribit","deribit wallet","deribit default",
    "bitmex","bitmex wallet","bitmex default",
    "liquid","liquid wallet","liquid default",
    "bitso","bitso wallet","bitso default",
    "bter","bter wallet","bter default",
    "zb.com","zb.com wallet","zb.com default",
    "bitz","bitz wallet","bitz default",
    "hotbit","hotbit wallet","hotbit default",
    "crex24","crex24 wallet","crex24 default",
    "tradeogre","tradeogre wallet","tradeogre default",
    "b2bx","b2bx wallet","b2bx default",
    "cex.io","cex.io wallet","cex.io default",
    "exmo","exmo wallet","exmo default",
    "livecoin","livecoin wallet","livecoin default",
    "bl3p","bl3p wallet","bl3p default",
    "lykke","lykke wallet","lykke default",
    "bitbay","bitbay wallet","bitbay default",
    "crypto market","crypto market wallet","crypto market default",
    "btc exchange","crypto exchange","bitcoin exchange",
    "btc trading","crypto trading","bitcoin trading",
    "decentralized exchange","dex","centralized exchange","cex",
    "automated market maker","amm","liquidity pool","yield farming",
    "liquidity mining","staking","proof of stake","pos",
    "proof of work","pow","proof of authority","poa",
    "proof of history","poh","proof of space","pospace",
    "proof of capacity","poc","proof of elapsed time","poet",
    "proof of burn","pob","proof of importance","poi",
    "proof of activity","poa","delegated proof of stake","dpos",
    "leap proof of stake","lpos","bonded proof of stake","bpos",
    "proof of utility","pou","proof of reputation","por",
    "proof of contribution","poc","proof of existence","poe"
)
foreach ($w in $walletDefaults) {
    $lines += $w
    $lines += $w.ToLower()
    $lines += $w.ToUpper()
    $lines += "$w!"
    $lines += "$w123"
    $lines += "$w password"
    $lines += "$w default"
    $lines += "$w key"
    $lines += "$w seed"
    $lines += "$w wallet"
}

Write-Host "Wallet defaults done"

# === 20. Countries + Cities ===
$countries = @(
    "united states","america","usa","united kingdom","britain","uk","england","scotland","wales","ireland",
    "france","germany","italy","spain","portugal","greece","turkey","russia","china","japan",
    "korea","north korea","south korea","india","pakistan","bangladesh","sri lanka","nepal","bhutan",
    "myanmar","thailand","vietnam","cambodia","laos","malaysia","singapore","indonesia","philippines",
    "brunei","timor leste","australia","new zealand","fiji","samoa","tonga","vanuatu","solomon islands",
    "papua new guinea","cuba","jamaica","haiti","dominican republic","puerto rico","trinidad","barbados",
    "bahamas","brazil","argentina","chile","peru","colombia","venezuela","ecuador","bolivia","paraguay",
    "uruguay","mexico","guatemala","honduras","el salvador","nicaragua","costa rica","panama",
    "canada","greenland","iceland","norway","sweden","finland","denmark","switzerland","austria","czech",
    "slovakia","poland","hungary","romania","bulgaria","serbia","croatia","slovenia","bosnia","montenegro",
    "albania","macedonia","estonia","latvia","lithuania","belarus","ukraine","moldova","georgia","armenia",
    "azerbaijan","kazakhstan","uzbekistan","turkmenistan","kyrgyzstan","tajikistan","afghanistan","iran",
    "iraq","syria","lebanon","jordan","israel","palestine","saudi arabia","yemen","oman","uae",
    "qatar","bahrain","kuwait","egypt","libya","tunisia","algeria","morocco","mauritania","senegal",
    "mali","niger","chad","sudan","south sudan","ethiopia","eritrea","djibouti","somalia","kenya",
    "tanzania","uganda","rwanda","burundi","congo","drc","angola","mozambique","zimbabwe","zambia",
    "malawi","botswana","namibia","south africa","lesotho","swaziland","madagascar","mauritius","seychelles",
    "comoros","cabo verde","gambia","guinea","sierra leone","liberia","cote d'ivoire","ghana","togo","benin"
)
$cities = @(
    "new york","london","paris","tokyo","berlin","rome","madrid","moscow","beijing","mumbai",
    "shanghai","seoul","bangkok","dubai","hong kong","singapore","sydney","toronto","vancouver","mexico city",
    "buenos aires","sao paulo","rio de janeiro","cairo","lagos","nairobi","johannesburg","cape town","casablanca",
    "istanbul","athens","warsaw","prague","budapest","vienna","zurich","amsterdam","brussels","copenhagen",
    "stockholm","oslo","helsinki","reykjavik","lisbon","porto","milan","naples","venice","florence",
    "munich","hamburg","frankfurt","dusseldorf","cologne","stuttgart","dresden","leipzig","hanover","nuremberg",
    "marseille","lyon","toulouse","nice","nantes","strasbourg","bordeaux","lille","rennes","rouen",
    "barcelona","valencia","seville","zaragoza","malaga","bilbao","granada","cordoba","vigo","alicante",
    "mumbai","delhi","bangalore","hyderabad","chennai","kolkata","pune","ahmedabad","jaipur","lucknow",
    "shenzhen","guangzhou","chengdu","hangzhou","wuhan","nanjing","tianjin","chongqing","xian","suzhou",
    "osaka","kyoto","yokohama","nagoya","sapporo","fukuoka","kobe","sendai","hiroshima","nara",
    "busan","daejeon","incheon","gwangju","daegu","ulsan","suwon","seongnam","gangneung","jeju",
    "hanoi","ho chi minh city","da nang","hai phong","can tho","bien hoa","hue","nha trang","phu quoc","dalat"
)
foreach ($c in $countries) {
    $lines += $c
    $lines += $c.ToUpper()
    $lines += "$c!"
    $lines += "$c bitcoin"
    $lines += "bitcoin $c"
}
foreach ($ci in $cities) {
    $lines += $ci
    $lines += $ci.ToUpper()
    $lines += "$ci!"
    $lines += "$ci bitcoin"
    $lines += "bitcoin $ci"
}

Write-Host "Countries + Cities done"

# === Deduplicate and write ===
Write-Host "Deduplicating..."
$unique = $lines | Where-Object { $_ -and $_.Trim() } | Sort-Object -Unique
Write-Host "Total unique patterns (Wave 3): $($unique.Count)"
$unique | Out-File -FilePath $outputFile -Encoding UTF8
Write-Host "Written to $outputFile"
