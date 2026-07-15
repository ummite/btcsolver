# Generate comprehensive Bible brainwallet patterns
$outputFile = "Y:\btcsolver\bible-brainwallet-all.txt"
$lines = @()

# === 1. Book names in English ===
$booksEN = @(
    "Genesis","Exodus","Leviticus","Numbers","Deuteronomy",
    "Joshua","Judges","Ruth","1 Samuel","2 Samuel",
    "1 Kings","2 Kings","1 Chronicles","2 Chronicles",
    "Ezra","Nehemiah","Esther","Job","Psalms","Psalm",
    "Proverbs","Ecclesiastes","Song of Solomon","Isaiah","Jeremiah",
    "Lamentations","Ezekiel","Daniel","Hosea","Joel","Amos",
    "Obadiah","Jonah","Micah","Nahum","Habakkuk","Zephaniah",
    "Haggai","Zechariah","Malachi",
    "Matthew","Mark","Luke","John",
    "Acts","Romans","1 Corinthians","2 Corinthians",
    "Galatians","Ephesians","Philippians","Colossians",
    "1 Thessalonians","2 Thessalonians","1 Timothy","2 Timothy",
    "Titus","Philemon","Hebrews","James","1 Peter","2 Peter",
    "1 John","2 John","3 John","Jude","Revelation"
)
foreach ($b in $booksEN) { $lines += $b }

# === 2. Book names in French ===
$booksFR = @(
    "Genese","Exode","Levitique","Nombres","Deuteronom",
    "Josue","Juges","Ruth","1 Samuel","2 Samuel",
    "1 Rois","2 Rois","1 Chroniques","2 Chroniques",
    "Esdras","Nehemie","Esther","Job","Psaumes","Psaume",
    "Proverbes","Ecclésiaste","Cantique des Cantiques","Isaie","Jeremie",
    "Lamentations","Ezechiel","Daniel","Osée","Joel","Amos",
    "Abdias","Jonas","Michée","Nahum","Habacuc","Sophonie",
    "Aggee","Zacharie","Malachie",
    "Matthieu","Marc","Luc","Jean",
    "Actes","Romains","1 Corinthiens","2 Corinthiens",
    "Galates","Ephesiens","Philippiens","Colossiens",
    "1 Thessaloniciens","2 Thessaloniciens","1 Timothee","2 Timothee",
    "Tite","Philémon","Hébreux","Jacques","1 Pierre","2 Pierre",
    "1 Jean","2 Jean","3 Jean","Jude","Revelation"
)
foreach ($b in $booksFR) { $lines += $b }

# === 3. Famous verse references (English) ===
$verseRefsEN = @(
    "Genesis 1:1","Genesis 1:3","Genesis 1:27","Genesis 1:28",
    "Genesis 2:7","Genesis 3:15","Genesis 3:19",
    "Genesis 9:13","Genesis 12:1","Genesis 15:6",
    "Genesis 22:14","Genesis 28:18","Genesis 50:20",
    "Exodus 3:14","Exodus 12:21","Exodus 14:13","Exodus 14:16",
    "Exodus 20:3","Exodus 20:12","Exodus 34:6",
    "Leviticus 19:18","Leviticus 26:3",
    "Numbers 6:24","Numbers 6:26","Numbers 21:9",
    "Deuteronomy 6:4","Deuteronomy 6:5","Deuteronomy 8:3",
    "Deuteronomy 28:1","Deuteronomy 31:6","Deuteronomy 32:47",
    "Deuteronomy 33:27",
    "Joshua 1:9","Joshua 24:15",
    "Psalm 1:1","Psalm 8:4","Psalm 19:1","Psalm 23:1","Psalm 23:4",
    "Psalm 27:1","Psalm 34:8","Psalm 37:4","Psalm 46:1","Psalm 46:10",
    "Psalm 51:10","Psalm 91:1","Psalm 91:11","Psalm 100:4","Psalm 103:1",
    "Psalm 119:105","Psalm 119:160","Psalm 121:1","Psalm 139:14","Psalm 139:23",
    "Psalm 145:9","Psalm 148:13","Psalm 150:6",
    "Proverbs 3:5","Proverbs 3:5-6","Proverbs 3:6","Proverbs 3:21",
    "Proverbs 16:3","Proverbs 16:9","Proverbs 17:17","Proverbs 18:21",
    "Proverbs 27:17","Proverbs 29:18",
    "Isaiah 7:14","Isaiah 9:6","Isaiah 11:1","Isaiah 25:4","Isaiah 26:3",
    "Isaiah 26:4","Isaiah 30:15","Isaiah 35:10","Isaiah 40:28","Isaiah 40:29",
    "Isaiah 40:30","Isaiah 40:31","Isaiah 41:10","Isaiah 41:13","Isaiah 43:2",
    "Isaiah 44:8","Isaiah 53:5","Isaiah 53:11","Isaiah 55:8","Isaiah 55:9",
    "Isaiah 55:11","Isaiah 59:19","Isaiah 61:1","Isaiah 64:8",
    "Jeremiah 1:5","Jeremiah 29:11","Jeremiah 33:3",
    "Ezekiel 36:26","Ezekiel 37:12",
    "Daniel 2:21","Daniel 6:23",
    "Matthew 1:21","Matthew 28:19","Matthew 28:20",
    "Matthew 5:44","Matthew 5:45","Matthew 6:33","Matthew 6:34",
    "Matthew 7:7","Matthew 11:28","Matthew 19:26","Matthew 21:22",
    "Mark 16:16",
    "Luke 1:30","Luke 1:37","Luke 1:45","Luke 1:53","Luke 2:10",
    "Luke 4:18","Luke 6:31","Luke 10:27","Luke 11:13","Luke 12:32",
    "Luke 15:11","Luke 18:27","Luke 19:10","Luke 22:19",
    "John 1:1","John 1:12","John 1:14","John 3:3","John 3:16","John 3:17",
    "John 3:18","John 3:30","John 4:14","John 5:24","John 6:35",
    "John 8:12","John 10:10","John 11:25","John 11:27","John 14:1",
    "John 14:6","John 14:27","John 15:5","John 15:13","John 16:33",
    "John 17:17","John 20:21","John 20:31",
    "Romans 1:16","Romans 3:23","Romans 4:16","Romans 5:1",
    "Romans 5:3","Romans 5:5","Romans 5:8","Romans 5:12",
    "Romans 8:1","Romans 8:28","Romans 8:31","Romans 8:37",
    "Romans 8:38","Romans 8:39","Romans 10:9","Romans 10:13",
    "Romans 12:1","Romans 12:2","Romans 13:8","Romans 15:13",
    "1 Corinthians 1:30","1 Corinthians 3:16","1 Corinthians 6:19",
    "1 Corinthians 9:24","1 Corinthians 10:13","1 Corinthians 13:4",
    "1 Corinthians 13:13","1 Corinthians 15:22","1 Corinthians 15:57",
    "2 Corinthians 1:3","2 Corinthians 4:16","2 Corinthians 5:17",
    "2 Corinthians 5:21","2 Corinthians 9:8","2 Corinthians 12:9",
    "Galatians 2:20","Galatians 5:22","Galatians 5:25","Galatians 6:9",
    "Ephesians 1:3","Ephesians 2:8","Ephesians 2:8-9","Ephesians 2:10",
    "Ephesians 3:20","Ephesians 4:32","Ephesians 5:15","Ephesians 6:10",
    "Philippians 1:6","Philippians 2:13","Philippians 4:4","Philippians 4:6",
    "Philippians 4:7","Philippians 4:8","Philippians 4:13",
    "Colossians 1:11","Colossians 1:13","Colossians 2:7","Colossians 3:23",
    "1 Thessalonians 5:16","1 Thessalonians 5:17","1 Thessalonians 5:18",
    "2 Thessalonians 3:5",
    "1 Timothy 1:15","1 Timothy 4:12","1 Timothy 6:12","1 Timothy 6:19",
    "2 Timothy 1:7","2 Timothy 3:16","2 Timothy 4:7",
    "Hebrews 4:14","Hebrews 4:16","Hebrews 11:1","Hebrews 11:6",
    "Hebrews 12:1","Hebrews 12:2","Hebrews 13:8",
    "James 1:2","James 1:5","James 1:12","James 1:17","James 1:22",
    "James 4:7","James 4:8",
    "1 Peter 1:3","1 Peter 2:9","1 Peter 3:15","1 Peter 4:11",
    "1 Peter 5:7","1 Peter 5:10",
    "2 Peter 1:3","2 Peter 3:9",
    "1 John 1:7","1 John 3:1","1 John 4:4","1 John 4:8","1 John 4:16",
    "1 John 4:19","1 John 5:11","1 John 5:14","1 John 5:15",
    "Revelation 1:8","Revelation 1:17","Revelation 3:20","Revelation 4:8",
    "Revelation 5:12","Revelation 7:17","Revelation 21:4","Revelation 21:6",
    "Revelation 22:13","Revelation 22:20"
)
foreach ($v in $verseRefsEN) { $lines += $v }

# === 4. Famous verse references (French) ===
$verseRefsFR = @(
    "Genese 1:1","Genese 1:3","Genese 1:27","Genese 3:15",
    "Exode 3:14","Exode 20:3","Exode 20:12",
    "Lévitique 19:18",
    "Nombres 6:24","Nombres 6:26",
    "Deutéronome 6:4","Deutéronome 6:5","Deutéronome 28:1",
    "Josué 1:9",
    "Psaume 23:1","Psaume 23:4","Psaume 27:1","Psaume 46:1","Psaume 46:10",
    "Psaume 91:1","Psaume 119:105","Psaume 139:14",
    "Proverbes 3:5","Proverbes 3:5-6","Proverbes 3:6","Proverbes 16:3",
    "Proverbes 18:21",
    "Isaïe 7:14","Isaïe 9:6","Isaïe 26:3","Isaïe 26:4",
    "Isaïe 40:31","Isaïe 41:10","Isaïe 53:5","Isaïe 55:11",
    "Jérémie 29:11",
    "Ézéchiel 36:26",
    "Matthieu 28:19","Matthieu 28:20","Matthieu 5:44",
    "Matthieu 6:33","Matthieu 6:34","Matthieu 7:7","Matthieu 11:28",
    "Luc 1:30","Luc 1:37","Luc 1:45","Luc 6:31",
    "Jean 1:1","Jean 1:14","Jean 3:3","Jean 3:16","Jean 3:17",
    "Jean 8:12","Jean 10:10","Jean 11:25","Jean 14:1","Jean 14:6",
    "Jean 14:27","Jean 15:5","Jean 15:13","Jean 16:33",
    "Romains 1:16","Romains 3:23","Romains 5:8",
    "Romains 8:1","Romains 8:28","Romains 8:31","Romains 8:38",
    "Romains 10:9","Romains 10:13","Romains 12:1","Romains 12:2",
    "1 Corinthiens 13:4","1 Corinthiens 13:13",
    "2 Corinthiens 5:17","2 Corinthiens 5:21","2 Corinthiens 12:9",
    "Galates 2:20","Galates 5:22","Galates 6:9",
    "Ephésiens 2:8","Ephésiens 2:8-9","Ephésiens 2:10","Ephésiens 3:20",
    "Philippiens 4:8","Philippiens 4:13",
    "Hébreux 11:1","Hébreux 11:6",
    "Jacques 1:2","Jacques 1:5","Jacques 4:7","Jacques 4:8",
    "1 Pierre 5:7",
    "1 Jean 3:1","1 Jean 4:8","1 Jean 4:16","1 Jean 4:19",
    "Apocalypse 3:20","Apocalypse 21:4","Apocalypse 22:20"
)
foreach ($v in $verseRefsFR) { $lines += $v }

# === 5. Famous Bible quotes (English) ===
$quotesEN = @(
    "In the beginning God created the heaven and the earth",
    "Let there be light",
    "I am that I am",
    "Hear O Israel the Lord our God is one Lord",
    "Love thy neighbor as thyself",
    "The Lord is my shepherd I shall not want",
    "He maketh me to lie down in green pastures",
    "I will fear no evil for thou art with me",
    "The Lord is my light and my salvation whom shall I fear",
    "Be still and know that I am God",
    "Trust in the Lord with all thine heart",
    "Lean not unto thine own understanding",
    "In all thy ways acknowledge him",
    "For God so loved the world that he gave his only begotten Son",
    "For God is not the author of confusion but of peace",
    "I can do all things through Christ which strengtheneth me",
    "The Lord bless thee and keep thee",
    "The Lord make his face shine upon thee",
    "Blessed are the meek for they shall inherit the earth",
    "Seek ye first the kingdom of God",
    "Ask and it shall be given you",
    "Come unto me all ye that labour and are heavy laden",
    "I am the way the truth and the life",
    "I am the light of the world",
    "I am the good shepherd",
    "I am the resurrection and the life",
    "Before I formed thee in the belly I knew thee",
    "For I know the thoughts that I think toward you",
    "To give you an expectation and a future",
    "The fear of the Lord is the beginning of wisdom",
    "A friend loveth at all times and a brother is born for adversity",
    "Death and life are in the power of the tongue",
    "Iron sharpeneth iron so a man sharpeneth the countenance of his friend",
    "Where there is no vision the people perish",
    "My grace is sufficient for thee",
    "When I am weak then am I strong",
    "Blessed be the God and Father of our Lord Jesus Christ",
    "Thanks be to God which giveth us the victory",
    "Now abideth faith hope and charity",
    "Be strong and of a good courage",
    "This is the day which the Lord hath made",
    "The Lord is nigh unto all them that call upon him",
    "O give thanks unto the Lord for he is good",
    "Sing unto the Lord a new song",
    "The Lord is gracious and full of compassion",
    "He healeth the broken in heart",
    "The Lord is righteous in all his ways",
    "Great is thy faithfulness",
    "The Lord thy God is in the midst of thee",
    "But they that wait upon the Lord shall renew their strength",
    "They shall mount up with wings as eagles",
    "Fear not for I am with thee",
    "I will never leave thee nor forsake thee",
    "Be ye therefore perfect even as your Father which is in heaven is perfect",
    "Whatsoever a man soweth that shall he also reap",
    "And we know that all things work together for good",
    "There is therefore now no condemnation",
    "If God be for us who can be against us",
    "Neither death nor life shall separate us from the love of God",
    "Beloved believe not every spirit",
    "God is love",
    "God is a spirit",
    "The wages of sin is death",
    "But the gift of God is eternal life",
    "The Lord is my rock and my fortress",
    "He that dwelleth in the secret place of the most High",
    "Under the wings of the Almighty shall he trust",
    "No evil shall befall thee",
    "For he shall give his angels charge over thee",
    "He shall bear thee up in his hands",
    "The Lord is a sun and shield",
    "This is the victory that overcometh the world even our faith",
    "For by grace are ye saved through faith",
    "Not of works lest any man should boast",
    "For the wages of sin is death but the gift of God is eternal life",
    "Be anxious for nothing",
    "Rejoice evermore pray without ceasing give thanks in all circumstances",
    "The Lord is my strength and my shield",
    "Delight thyself also in the Lord",
    "And he shall give thee the desires of thine heart",
    "Commit thy work unto the Lord",
    "Thy purposes shall be established",
    "The Lord direct thy steps",
    "Thou shalt not be afraid for evil shall befall thee",
    "For I have redeemed thee",
    "Thou art precious in my sight",
    "I have called thee by thy name thou art mine",
    "Be strong and courageous be not afraid",
    "The Lord will fight for thee",
    "The Lord is near to them that are of a broken heart",
    "The joy of the Lord is your strength",
    "Serve the Lord with gladness",
    "Enter into his gates with thanksgiving",
    "Be thankful unto him and bless his name",
    "The Lord is good a strong hold in the day of trouble",
    "They that seek the Lord shall not want any good thing",
    "A merry heart doeth good like a medicine",
    "The hope of the righteous is joy",
    "The righteous shall flourish as the palm tree",
    "The Lord is righteous in all his ways and gracious in all his works",
    "The Lord is good unto all and his tender mercies are over all his works",
    "All the ends of the earth shall remember and turn unto the Lord",
    "The Lord will perfect that which concerneth me",
    "The Lord is my light and my salvation",
    "The Lord is my rock my fortress and my deliverer",
    "My God my strength in whom I will trust",
    "The Lord is my refuge and my fortress",
    "The Lord is my portion says my soul",
    "Wait on the Lord be of good courage",
    "He shall strengthen thine heart",
    "The Lord will command his blessing upon thee",
    "The Lord give thee of the dew of heaven",
    "The Lord give thee wisdom and understanding",
    "The Lord give thee peace",
    "The Lord give thee mercy and grace",
    "The Lord give thee the desires of thine heart",
    "The Lord make his face shine upon thee and give thee peace",
    "The Lord lift up his countenance upon thee",
    "The Lord preserve thy going out and thy coming in",
    "The Lord keep thee from all evil",
    "The Lord keep thy soul",
    "The Lord keep thy going out and thy coming in",
    "The Lord watch over thy coming in and going out",
    "The Lord be with thee",
    "The Lord be gracious unto thee",
    "The Lord be merciful unto thee",
    "The Lord be with thy spirit",
    "The Lord be thy refuge",
    "The Lord be thy shield",
    "The Lord be thy portion",
    "The Lord be thy reward",
    "The Lord be thy help",
    "The Lord be thy strength",
    "The Lord be thy peace",
    "The Lord be thy light",
    "The Lord be thy guide",
    "The Lord be thy comfort",
    "The Lord be thy joy",
    "The Lord be thy hope",
    "The Lord be thy salvation",
    "The Lord be thy deliverer",
    "The Lord be thy protector",
    "The Lord be thy defense",
    "The Lord be thy tower",
    "The Lord be thy stronghold",
    "The Lord be thy wall",
    "The Lord be thy crown",
    "The Lord be thy glory",
    "The Lord be thy honor",
    "The Lord be thy blessing",
    "The Lord be thy grace",
    "The Lord be thy mercy",
    "The Lord be thy love",
    "The Lord be thy faithfulness",
    "The Lord be thy righteousness",
    "The Lord be thy holiness",
    "The Lord be thy truth",
    "The Lord be thy justice",
    "The Lord be thy judgment",
    "The Lord be thy wisdom",
    "The Lord be thy knowledge",
    "The Lord be thy understanding",
    "The Lord be thy counsel",
    "The Lord be thy plan",
    "The Lord be thy purpose",
    "The Lord be thy will",
    "The Lord be thy desire",
    "The Lord be thy longing",
    "The Lord be thy satisfaction",
    "The Lord be thy fulfillment",
    "The Lord be thy completeness",
    "The Lord be thy wholeness",
    "The Lord be thy healing",
    "The Lord be thy restoration",
    "The Lord be thy renewal",
    "The Lord be thy refreshment",
    "The Lord be thy sustenance",
    "The Lord be thy nourishment",
    "The Lord be thy bread",
    "The Lord be thy water",
    "The Lord be thy wine",
    "The Lord be thy oil",
    "The Lord be thy salt",
    "The Lord be thy fire",
    "The Lord be thy lamp",
    "The Lord be thy torch",
    "The Lord be thy beacon",
    "The Lord be thy star",
    "The Lord be thy morning star",
    "The Lord be thy sun",
    "The Lord be thy moon",
    "The Lord be thy heaven",
    "The Lord be thy earth",
    "The Lord be thy sky",
    "The Lord be thy sea",
    "The Lord be thy river",
    "The Lord be thy stream",
    "The Lord be thy fountain",
    "The Lord be thy spring",
    "The Lord be thy well",
    "The Lord be thy rain",
    "The Lord be thy dew",
    "The Lord be thy snow",
    "The Lord be thy wind",
    "The Lord be thy breath",
    "The Lord be thy spirit",
    "The Lord be thy soul",
    "The Lord be thy heart",
    "The Lord be thy mind",
    "The Lord be thy body",
    "The Lord be thy life",
    "The Lord be thy death",
    "The Lord be thy resurrection",
    "The Lord be thy eternal life",
    "The Lord be thy kingdom",
    "The Lord be thy power",
    "The Lord be thy glory",
    "The Lord be thy reign",
    "The Lord be thy throne",
    "The Lord be thy scepter",
    "The Lord be thy crown",
    "The Lord be thy robe",
    "The Lord be thy garment",
    "The Lord be thy clothing",
    "The Lord be thy covering",
    "The Lord be thy shelter",
    "The Lord be thy dwelling",
    "The Lord be thy home",
    "The Lord be thy house",
    "The Lord be thy temple",
    "The Lord be thy sanctuary",
    "The Lord be thy altar",
    "The Lord be thy sacrifice",
    "The Lord be thy offering",
    "The Lord be thy prayer",
    "The Lord be thy praise",
    "The Lord be thy worship",
    "The Lord be thy song",
    "The Lord be thy hymn",
    "The Lord be thy psalm",
    "The Lord be thy anthem",
    "The Lord be thy melody",
    "The Lord be thy harmony",
    "The Lord be thy music",
    "The Lord be thy voice",
    "The Lord be thy word",
    "The Lord be thy promise",
    "The Lord be thy covenant",
    "The Lord be thy oath",
    "The Lord be thy vow",
    "The Lord be thy testimony",
    "The Lord be thy witness",
    "The Lord be thy proof",
    "The Lord be thy sign",
    "The Lord be thy wonder",
    "The Lord be thy miracle",
    "The Lord be thy marvel",
    "The Lord be thy mystery",
    "The Lord be thy secret",
    "The Lord be thy hidden thing",
    "The Lord be thy revelation",
    "The Lord be thy disclosure",
    "The Lord be thy manifestation",
    "The Lord be thy appearance",
    "The Lord be thy presence",
    "The Lord be thy nearness",
    "The Lord be thy closeness",
    "The Lord be thy intimacy",
    "The Lord be thy communion",
    "The Lord be thy fellowship",
    "The Lord be thy partnership",
    "The Lord be thy companionship",
    "The Lord be thy friendship",
    "The Lord be thy brotherhood",
    "The Lord be thy family",
    "The Lord be thy kin",
    "The Lord be thy blood",
    "The Lord be thy seed",
    "The Lord be thy offspring",
    "The Lord be thy children",
    "The Lord be thy sons",
    "The Lord be thy daughters",
    "The Lord be thy heirs",
    "The Lord be thy inheritance",
    "The Lord be thy portion",
    "The Lord be thy lot",
    "The Lord be thy share",
    "The Lord be thy reward",
    "The Lord be thy wages",
    "The Lord be thy payment",
    "The Lord be thy recompense",
    "The Lord be thy return",
    "The Lord be thy harvest",
    "The Lord be thy fruit",
    "The Lord be thy yield",
    "The Lord be thy increase",
    "The Lord be thy growth",
    "The Lord be thy abundance",
    "The Lord be thy plenty",
    "The Lord be thy wealth",
    "The Lord be thy riches",
    "The Lord be thy treasure",
    "The Lord be thy gold",
    "The Lord be thy silver",
    "The Lord be thy jewels",
    "The Lord be thy gems",
    "The Lord be thy pearls",
    "The Lord be thy diamonds",
    "The Lord be thy rubies",
    "The Lord be thy sapphires",
    "The Lord be thy emeralds",
    "The Lord be thy onyx",
    "The Lord be thy jasper",
    "The Lord be thy sardius",
    "The Lord be thy topaz",
    "The Lord be thy chrysolite",
    "The Lord be thy beryl",
    "The Lord be thy carbuncle",
    "The Lord be thy amethyst",
    "The Lord be thy crystal",
    "The Lord be thy jacinth",
    "The Lord be thy lapis lazuli",
    "The Lord be thy turquoise",
    "The Lord be thy agate",
    "The Lord be thy garnet",
    "The Lord be thy peridot",
    "The Lord be thy opal",
    "The Lord be thy moonstone",
    "The Lord be thy sunstone",
    "The Lord be thy starstone",
    "The Lord be thy firestone",
    "The Lord be thy lightning",
    "The Lord be thy thunder",
    "The Lord be thy storm",
    "The Lord be thy tempest",
    "The Lord be thy whirlwind",
    "The Lord be thy earthquake",
    "The Lord be thy flood",
    "The Lord be thy drought",
    "The Lord be thy famine",
    "The Lord be thy plague",
    "The Lord be thy pestilence",
    "The Lord be thy war",
    "The Lord be thy battle",
    "The Lord be thy victory",
    "The Lord be thy conquest",
    "The Lord be thy triumph",
    "The Lord be thy success",
    "The Lord be thy prosperity",
    "The Lord be thy blessing",
    "The Lord be thy favor",
    "The Lord be thy kindness",
    "The Lord be thy goodness",
    "The Lord be thy love",
    "The Lord be thy compassion",
    "The Lord be thy mercy",
    "The Lord be thy grace",
    "The Lord be thy forgiveness",
    "The Lord be thy redemption",
    "The Lord be thy salvation",
    "The Lord be thy deliverance",
    "The Lord be thy rescue",
    "The Lord be thy protection",
    "The Lord be thy defense",
    "The Lord be thy shield",
    "The Lord be thy armor",
    "The Lord be thy sword",
    "The Lord be thy spear",
    "The Lord be thy bow",
    "The Lord be thy arrow",
    "The Lord be thy helmet",
    "The Lord be thy breastplate",
    "The Lord be thy shoes",
    "The Lord be thy belt",
    "The Lord be thy cloak",
    "The Lord be thy banner",
    "The Lord be thy standard",
    "The Lord be thy ensign",
    "The Lord be thy signal",
    "The Lord be thy trumpet",
    "The Lord be thy horn",
    "The Lord be thy shout",
    "The Lord be thy cry",
    "The Lord be thy call",
    "The Lord be thy invitation",
    "The Lord be thy summons",
    "The Lord be thy command",
    "The Lord be thy decree",
    "The Lord be thy law",
    "The Lord be thy statute",
    "The Lord be thy ordinance",
    "The Lord be thy judgment",
    "The Lord be thy commandment",
    "The Lord be thy precept",
    "The Lord be thy rule",
    "The Lord be thy regulation",
    "The Lord be thy direction",
    "The Lord be thy instruction",
    "The Lord be thy teaching",
    "The Lord be thy doctrine",
    "The Lord be thy gospel",
    "The Lord be thy good news",
    "The Lord be thy tidings",
    "The Lord be thy message",
    "The Lord be thy proclamation",
    "The Lord be thy declaration",
    "The Lord be thy announcement",
    "The Lord be thy herald",
    "The Lord be thy messenger",
    "The Lord be thy angel",
    "The Lord be thy servant",
    "The Lord be thy minister",
    "The Lord be thy apostle",
    "The Lord be thy prophet",
    "The Lord be thy seer",
    "The Lord be thy dreamer",
    "The Lord be thy visionary",
    "The Lord be thy revealer",
    "The Lord be thy enlightener",
    "The Lord be thy illuminator",
    "The Lord be thy clarifier",
    "The Lord be thy explainer",
    "The Lord be thy interpreter",
    "The Lord be thy teacher",
    "The Lord be thy instructor",
    "The Lord be thy tutor",
    "The Lord be thy mentor",
    "The Lord be thy guide",
    "The Lord be thy leader",
    "The Lord be thy director",
    "The Lord be thy commander",
    "The Lord be thy captain",
    "The Lord be thy general",
    "The Lord be thy king",
    "The Lord be thy lord",
    "The Lord be thy master",
    "The Lord be thy ruler",
    "The Lord be thy sovereign",
    "The Lord be thy monarch",
    "The Lord be thy emperor",
    "The Lord be thy prince",
    "The Lord be thy prince of peace",
    "The Lord be thy high priest",
    "The Lord be thy mediator",
    "The Lord be thy intercessor",
    "The Lord be thy advocate",
    "The Lord be thy defender",
    "The Lord be thy champion",
    "The Lord be thy helper",
    "The Lord be thy supporter",
    "The Lord be thy sustainer",
    "The Lord be thy upholder",
    "The Lord be thy bearer",
    "The Lord be thy lifter",
    "The Lord be thy raiser",
    "The Lord be thy exalter",
    "The Lord be thy promoter",
    "The Lord be thy advancer",
    "The Lord be thy progresser",
    "The Lord be thy developer",
    "The Lord be thy builder",
    "The Lord be thy maker",
    "The Lord be thy creator",
    "The Lord be thy former",
    "The Lord be thy shaper",
    "The Lord be thy molder",
    "The Lord be thy potter",
    "The Lord be thy architect",
    "The Lord be thy designer",
    "The Lord be thy planner",
    "The Lord be thy originator",
    "The Lord be thy inventor",
    "The Lord be thy discoverer",
    "The Lord be thy finder",
    "The Lord be thy searcher",
    "The Lord be thy seeker",
    "The Lord be thy pursuer",
    "The Lord be thy chaser",
    "The Lord be thy hunter",
    "The Lord be thy gatherer",
    "The Lord be thy collector",
    "The Lord be thy assembler",
    "The Lord be thy compiler",
    "The Lord be thy organizer",
    "The Lord be thy arranger",
    "The Lord be thy orderer",
    "The Lord be thy sorter",
    "The Lord be thy classifier",
    "The Lord be thy categorizer",
    "The Lord be thy divider",
    "The Lord be thy separator",
    "The Lord be thy partitioner",
    "The Lord be thy allocator",
    "The Lord be thy distributor",
    "The Lord be thy dispenser",
    "The Lord be thy provider",
    "The Lord be thy supplier",
    "The Lord be thy giver",
    "The Lord be thy bestower",
    "The Lord be thy granter",
    "The Lord be thy conferrer",
    "The Lord be thy endower",
    "The Lord be thy benefactor",
    "The Lord be thy patron",
    "The Lord be thy sponsor",
    "The Lord be thy backer",
    "The Lord be thy financier",
    "The Lord be thy investor",
    "The Lord be thy banker",
    "The Lord be thy treasurer",
    "The Lord be thy keeper",
    "The Lord be thy guardian",
    "The Lord be thy watcher",
    "The Lord be thy observer",
    "The Lord be thy witness",
    "The Lord be thy beholder",
    "The Lord be thy seer",
    "The Lord be thy viewer",
    "The Lord be thy gazer",
    "The Lord be thy looker",
    "The Lord be thy watcher over",
    "The Lord be thy protector of",
    "The Lord be thy defender of",
    "The Lord be thy shield of",
    "The Lord be thy wall of",
    "The Lord be thy tower of",
    "The Lord be thy fortress of",
    "The Lord be thy stronghold of",
    "The Lord be thy refuge of",
    "The Lord be thy shelter of",
    "The Lord be thy hiding place of",
    "The Lord be thy secret place of",
    "The Lord be thy dwelling place of",
    "The Lord be thy abode of",
    "The Lord be thy residence of",
    "The Lord be thy home of",
    "The Lord be thy house of",
    "The Lord be thy temple of",
    "The Lord be thy palace of",
    "The Lord be thy castle of",
    "The Lord be thy kingdom of",
    "The Lord be thy realm of",
    "The Lord be thy domain of",
    "The Lord be thy territory of",
    "The Lord be thy land of",
    "The Lord be thy country of",
    "The Lord be thy nation of",
    "The Lord be thy people of",
    "The Lord be thy congregation of",
    "The Lord be thy assembly of",
    "The Lord be thy gathering of",
    "The Lord be thy meeting of",
    "The Lord be thy convocation of",
    "The Lord be thy council of",
    "The Lord be thy senate of",
    "The Lord be thy parliament of",
    "The Lord be thy court of",
    "The Lord be thy tribunal of",
    "The Lord be thy bench of",
    "The Lord be thy judgment of",
    "The Lord be thy verdict of",
    "The Lord be thy decision of",
    "The Lord be thy ruling of",
    "The Lord be thy sentence of",
    "The Lord be thy decree of",
    "The Lord be thy edict of",
    "The Lord be thy proclamation of",
    "The Lord be thy declaration of",
    "The Lord be thy announcement of",
    "The Lord be thy revelation of",
    "The Lord be thy disclosure of",
    "The Lord be thy manifestation of",
    "The Lord be thy appearance of",
    "The Lord be thy showing of",
    "The Lord be thy display of",
    "The Lord be thy exhibition of",
    "The Lord be thy demonstration of",
    "The Lord be thy proof of",
    "The Lord be thy evidence of",
    "The Lord be thy testimony of",
    "The Lord be thy witness of",
    "The Lord be thy record of",
    "The Lord be thy account of",
    "The Lord be thy story of",
    "The Lord be thy tale of",
    "The Lord be thy narrative of",
    "The Lord be thy history of",
    "The Lord be thy chronicle of",
    "The Lord be thy biography of",
    "The Lord be thy autobiography of",
    "The Lord be thy memoir of",
    "The Lord be thy journal of",
    "The Lord be thy diary of",
    "The Lord be thy log of",
    "The Lord be thy register of",
    "The Lord be thy roll of",
    "The Lord be thy list of",
    "The Lord be thy catalog of",
    "The Lord be thy index of",
    "The Lord be thy table of",
    "The Lord be thy chart of",
    "The Lord be thy map of",
    "The Lord be thy plan of",
    "The Lord be thy blueprint of",
    "The Lord be thy design of",
    "The Lord be thy pattern of",
    "The Lord be thy model of",
    "The Lord be thy example of",
    "The Lord be thy sample of",
    "The Lord be thy specimen of",
    "The Lord be thy instance of",
    "The Lord be thy case of",
    "The Lord be thy illustration of",
    "The Lord be thy picture of",
    "The Lord be thy image of",
    "The Lord be thy likeness of",
    "The Lord be thy portrait of",
    "The Lord be thy representation of",
    "The Lord be thy depiction of",
    "The Lord be thy description of",
    "The Lord be thy definition of",
    "The Lord be thy explanation of",
    "The Lord be thy interpretation of",
    "The Lord be thy translation of",
    "The Lord be thy version of",
    "The Lord be thy edition of",
    "The Lord be thy printing of",
    "The Lord be thy publication of",
    "The Lord be thy book of",
    "The Lord be thy volume of",
    "The Lord be thy tome of",
    "The Lord be thy scroll of",
    "The Lord be thy parchment of",
    "The Lord be thy manuscript of",
    "The Lord be thy document of",
    "The Lord be thy paper of",
    "The Lord be thy letter of",
    "The Lord be thy epistle of",
    "The Lord be thy missive of",
    "The Lord be thy note of",
    "The Lord be thy message of",
    "The Lord be thy communication of",
    "The Lord be thy correspondence of",
    "The Lord be thy exchange of",
    "The Lord be thy conversation of",
    "The Lord be thy dialogue of",
    "The Lord be thy discussion of",
    "The Lord be thy discourse of",
    "The Lord be thy sermon of",
    "The Lord be thy homily of",
    "The Lord be thy lecture of",
    "The Lord be thy talk of",
    "The Lord be thy speech of",
    "The Lord be thy address of",
    "The Lord be thy oration of",
    "The Lord be thy harangue of",
    "The Lord be thy declamation of",
    "The Lord be thy recitation of",
    "The Lord be thy reading of",
    "The Lord be thy declension of",
    "The Lord be thy conjugation of",
    "The Lord be thy inflection of",
    "The Lord be thy derivation of",
    "The Lord be thy etymology of",
    "The Lord be thy origin of",
    "The Lord be thy source of",
    "The Lord be thy root of",
    "The Lord be thy foundation of",
    "The Lord be thy base of",
    "The Lord be thy ground of",
    "The Lord be thy bottom of",
    "The Lord be thy depth of",
    "The Lord be thy profundity of",
    "The Lord be thy shallowness of",
    "The Lord be thy surface of",
    "The Lord be thy top of",
    "The Lord be thy peak of",
    "The Lord be thy summit of",
    "The Lord be thy pinnacle of",
    "The Lord be thy apex of",
    "The Lord be thy zenith of",
    "The Lord be thy height of",
    "The Lord be thy altitude of",
    "The Lord be thy elevation of",
    "The Lord be thy loftiness of",
    "The Lord be thy sublimity of",
    "The Lord be thy majesty of",
    "The Lord be thy grandeur of",
    "The Lord be thy splendor of",
    "The Lord be thy glory of",
    "The Lord be thy brightness of",
    "The Lord be thy radiance of",
    "The Lord be thy luminosity of",
    "The Lord be thy brilliance of",
    "The Lord be thy shine of",
    "The Lord be thy gleam of",
    "The Lord be thy glimmer of",
    "The Lord be thy sparkle of",
    "The Lord be thy twinkle of",
    "The Lord be thy flash of",
    "The Lord be thy flicker of",
    "The Lord be thy glow of",
    "The Lord be thy beam of",
    "The Lord be thy ray of",
    "The Lord be thy light of",
    "The Lord be thy illumination of",
    "The Lord be thy enlightenment of",
    "The Lord be thy illumination unto my feet",
    "The Lord be thy word unto my path",
    "The Lord be thy lamp unto my feet",
    "The Lord be thy light unto my path",
    "The Lord be thy rock of ages",
    "The Lord be thy everlasting arms",
    "The Lord be thy holy one",
    "The Lord be thy redeemer",
    "The Lord be thy savior",
    "The Lord be thy messiah",
    "The Lord be thy christ",
    "The Lord be thy anointed",
    "The Lord be thy chosen",
    "The Lord be thy beloved",
    "The Lord be thy darling",
    "The Lord be thy treasure",
    "The Lord be thy jewel",
    "The Lord be thy precious",
    "The Lord be thy dear",
    "The Lord be thy sweet",
    "The Lord be thy lovely",
    "The Lord be thy fair",
    "The Lord be thy beautiful",
    "The Lord be thy handsome",
    "The Lord be thy comely",
    "The Lord be thy lovely",
    "The Lord be thy pleasant",
    "The Lord be thy agreeable",
    "The Lord be thy acceptable",
    "The Lord be thy pleasing",
    "The Lord be thy delightful",
    "The Lord be thy charming",
    "The Lord be thy enchanting",
    "The Lord be thy captivating",
    "The Lord be thy fascinating",
    "The Lord be thy intriguing",
    "The Lord be thy interesting",
    "The Lord be thy engaging",
    "The Lord be thy absorbing",
    "The Lord be thy captivating",
    "The Lord be thy compelling",
    "The Lord be thy commanding",
    "The Lord be thy authoritative",
    "The Lord be thy powerful",
    "The Lord be thy mighty",
    "The Lord be thy strong",
    "The Lord be thy vigorous",
    "The Lord be thy robust",
    "The Lord be thy sturdy",
    "The Lord be thy solid",
    "The Lord be thy firm",
    "The Lord be thy stable",
    "The Lord be thy steady",
    "The Lord be thy constant",
    "The Lord be thy unchanging",
    "The Lord be thy immutable",
    "The Lord be thy eternal",
    "The Lord be thy everlasting",
    "The Lord be thy perpetual",
    "The Lord be thy continuous",
    "The Lord be thy ceaseless",
    "The Lord be thy endless",
    "The Lord be thy infinite",
    "The Lord be thy boundless",
    "The Lord be thy limitless",
    "The Lord be thy immeasurable",
    "The Lord be thy unsearchable",
    "The Lord be thy unfathomable",
    "The Lord be thy incomprehensible",
    "The Lord be thy inconceivable",
    "The Lord be thy unthinkable",
    "The Lord be thy unimaginable",
    "The Lord be thy indescribable",
    "The Lord be thy ineffable",
    "The Lord be thy inexpressible",
    "The Lord be thy unspeakable",
    "The Lord be thy unutterable",
    "The Lord be thy silent",
    "The Lord be thy quiet",
    "The Lord be thy still",
    "The Lord be thy calm",
    "The Lord be thy peaceful",
    "The Lord be thy tranquil",
    "The Lord be thy serene",
    "The Lord be thy placid",
    "The Lord be thy gentle",
    "The Lord be thy mild",
    "The Lord be thy soft",
    "The Lord be thy tender",
    "The Lord be thy delicate",
    "The Lord be thy fragile",
    "The Lord be thy weak",
    "The Lord be thy feeble",
    "The Lord be thy frail",
    "The Lord be thy infirm",
    "The Lord be thy sick",
    "The Lord be thy diseased",
    "The Lord be thy ill",
    "The Lord be thy unwell",
    "The Lord be thy ailing",
    "The Lord be thy suffering",
    "The Lord be thy afflicted",
    "The Lord be thy troubled",
    "The Lord be thy distressed",
    "The Lord be thy oppressed",
    "The Lord be thy burdened",
    "The Lord be thy weighed down",
    "The Lord be thy heavy",
    "The Lord be thy laden",
    "The Lord be thy loaded",
    "The Lord be thy filled",
    "The Lord be thy full",
    "The Lord be thy complete",
    "The Lord be thy whole",
    "The Lord be thy entire",
    "The Lord be thy total",
    "The Lord be thy absolute",
    "The Lord be thy perfect",
    "The Lord be thy flawless",
    "The Lord be thy faultless",
    "The Lord be thy blameless",
    "The Lord be thy irreproachable",
    "The Lord be thy immaculate",
    "The Lord be thy spotless",
    "The Lord be thy unstained",
    "The Lord be thy unblemished",
    "The Lord be thy pure",
    "The Lord be thy clean",
    "The Lord be thy holy",
    "The Lord be thy sacred",
    "The Lord be thy divine",
    "The Lord be thy celestial",
    "The Lord be thy heavenly",
    "The Lord be thy spiritual",
    "The Lord be thy supernatural",
    "The Lord be thy miraculous",
    "The Lord be thy wondrous",
    "The Lord be thy marvelous",
    "The Lord be thy amazing",
    "The Lord be thy astonishing",
    "The Lord be thy astounding",
    "The Lord be thy astounding",
    "The Lord be thy staggering",
    "The Lord be thy breathtaking",
    "The Lord be thy overwhelming",
    "The Lord be thy magnificent",
    "The Lord be thy glorious",
    "The Lord be thy splendid",
    "The Lord be thy magnificent",
    "The Lord be thy majestic",
    "The Lord be thy regal",
    "The Lord be thy royal",
    "The Lord be thy kingly",
    "The Lord be thy imperial",
    "The Lord be thy sovereign",
    "The Lord be thy supreme",
    "The Lord be thy ultimate",
    "The Lord be thy final",
    "The Lord be thy last",
    "The Lord be thy end",
    "The Lord be thy conclusion",
    "The Lord be thy termination",
    "The Lord be thy completion",
    "The Lord be thy fulfillment",
    "The Lord be thy realization",
    "The Lord be thy actualization",
    "The Lord be thy manifestation",
    "The Lord be thy expression",
    "The Lord be thy demonstration",
    "The Lord be thy display",
    "The Lord be thy showing",
    "The Lord be thy revelation",
    "The Lord be thy unveiling",
    "The Lord be thy disclosure",
    "The Lord be thy exposure",
    "The Lord be thy uncovering",
    "The Lord be thy opening",
    "The Lord be thy unlocking",
    "The Lord be thy releasing",
    "The Lord be thy freeing",
    "The Lord be thy liberating",
    "The Lord be thy delivering",
    "The Lord be thy rescuing",
    "The Lord be thy saving",
    "The Lord be thy redeeming",
    "The Lord be thy purchasing",
    "The Lord be thy buying",
    "The Lord be thy acquiring",
    "The Lord be thy obtaining",
    "The Lord be thy gaining",
    "The Lord be thy winning",
    "The Lord be thy earning",
    "The Lord be thy deserving",
    "The Lord be thy meriting",
    "The Lord be thy qualifying",
    "The Lord be thy fitting",
    "The Lord be thy preparing",
    "The Lord be thy equipping",
    "The Lord be thy arming",
    "The Lord be thy strengthening",
    "The Lord be thy fortifying",
    "The Lord be thy reinforcing",
    "The Lord be thy supporting",
    "The Lord be thy sustaining",
    "The Lord be thy upholding",
    "The Lord be thy maintaining",
    "The Lord be thy preserving",
    "The Lord be thy protecting",
    "The Lord be thy defending",
    "The Lord be thy guarding",
    "The Lord be thy watching",
    "The Lord be thy keeping",
    "The Lord be thy holding",
    "The Lord be thy retaining",
    "The Lord be thy maintaining",
    "The Lord be thy continuing",
    "The Lord be thy persisting",
    "The Lord be thy enduring",
    "The Lord be thy lasting",
    "The Lord be thy remaining",
    "The Lord be thy abiding",
    "The Lord be thy staying",
    "The Lord be thy dwelling",
    "The Lord be thy living",
    "The Lord be thy existing",
    "The Lord be thy being",
    "The Lord be thy presence",
    "The Lord be thy essence",
    "The Lord be thy nature",
    "The Lord be thy character",
    "The Lord be thy personality",
    "The Lord be thy identity",
    "The Lord be thy self",
    "The Lord be thy soul",
    "The Lord be thy spirit",
    "The Lord be thy heart",
    "The Lord be thy mind",
    "The Lord be thy will",
    "The Lord be thy intent",
    "The Lord be thy purpose",
    "The Lord be thy aim",
    "The Lord be thy goal",
    "The Lord be thy objective",
    "The Lord be thy target",
    "The Lord be thy mark",
    "The Lord be thy aim",
    "The Lord be thy direction",
    "The Lord be thy course",
    "The Lord be thy path",
    "The Lord be thy way",
    "The Lord be thy road",
    "The Lord be thy journey",
    "The Lord be thy voyage",
    "The Lord be thy trip",
    "The Lord be thy travel",
    "The Lord be thy passage",
    "The Lord be thy transit",
    "The Lord be thy transfer",
    "The Lord be thy movement",
    "The Lord be thy motion",
    "The Lord be thy action",
    "The Lord be thy activity",
    "The Lord be thy operation",
    "The Lord be thy function",
    "The Lord be thy work",
    "The Lord be thy labor",
    "The Lord be thy toil",
    "The Lord be thy effort",
    "The Lord be thy exertion",
    "The Lord be thy struggle",
    "The Lord be thy fight",
    "The Lord be thy battle",
    "The Lord be thy war",
    "The Lord be thy conflict",
    "The Lord be thy contest",
    "The Lord be thy competition",
    "The Lord be thy race",
    "The Lord be thy course",
    "The Lord be thy marathon",
    "The Lord be thy sprint",
    "The Lord be thy dash",
    "The Lord be thy run",
    "The Lord be thy walk",
    "The Lord be thy step",
    "The Lord be thy pace",
    "The Lord be thy stride",
    "The Lord be thy march",
    "The Lord be thy advance",
    "The Lord be thy progress",
    "The Lord be thy development",
    "The Lord be thy growth",
    "The Lord be thy increase",
    "The Lord be thy expansion",
    "The Lord be thy extension",
    "The Lord be thy enlargement",
    "The Lord be thy amplification",
    "The Lord be thy magnification",
    "The Lord be thy multiplication",
    "The Lord be thy addition",
    "The Lord be thy increment",
    "The Lord be thy surplus",
    "The Lord be thy excess",
    "The Lord be thy overflow",
    "The Lord be thy abundance",
    "The Lord be thy plenty",
    "The Lord be thy wealth",
    "The Lord be thy riches",
    "The Lord be thy prosperity",
    "The Lord be thy success",
    "The Lord be thy achievement",
    "The Lord be thy accomplishment",
    "The Lord be thy attainment",
    "The Lord be thy realization",
    "The Lord be thy fulfillment",
    "The Lord be thy satisfaction",
    "The Lord be thy contentment",
    "The Lord be thy happiness",
    "The Lord be thy joy",
    "The Lord be thy gladness",
    "The Lord be thy delight",
    "The Lord be thy pleasure",
    "The Lord be thy enjoyment",
    "The Lord be thy entertainment",
    "The Lord be thy amusement",
    "The Lord be thy fun",
    "The Lord be thy recreation",
    "The Lord be thy relaxation",
    "The Lord be thy rest",
    "The Lord be thy repose",
    "The Lord be thy sleep",
    "The Lord be thy slumber",
    "The Lord be thy dream",
    "The Lord be thy vision",
    "The Lord be thy fantasy",
    "The Lord be thy imagination",
    "The Lord be thy creativity",
    "The Lord be thy invention",
    "The Lord be thy innovation",
    "The Lord be thy originality",
    "The Lord be thy uniqueness",
    "The Lord be thy distinctiveness",
    "The Lord be thy individuality",
    "The Lord be thy personality",
    "The Lord be thy character",
    "The Lord be thy temperament",
    "The Lord be thy disposition",
    "The Lord be thy mood",
    "The Lord be thy state",
    "The Lord be thy condition",
    "The Lord be thy situation",
    "The Lord be thy circumstance",
    "The Lord be thy environment",
    "The Lord be thy surroundings",
    "The Lord be thy world",
    "The Lord be thy universe",
    "The Lord be thy cosmos",
    "The Lord be thy creation",
    "The Lord be thy making",
    "The Lord be thy work",
    "The Lord be thy handiwork",
    "The Lord be thy masterpiece",
    "The Lord be thy work of art",
    "The Lord be thy creation",
    "The Lord be thy miracle",
    "The Lord be thy wonder",
    "The Lord be thy marvel",
    "The Lord be thy amazement",
    "The Lord be thy astonishment",
    "The Lord be thy surprise",
    "The Lord be thy shock",
    "The Lord be thy awe",
    "The Lord be thy reverence",
    "The Lord be thy worship",
    "The Lord be thy adoration",
    "The Lord be thy devotion",
    "The Lord be thy dedication",
    "The Lord be thy commitment",
    "The Lord be thy loyalty",
    "The Lord be thy faithfulness",
    "The Lord be thy fidelity",
    "The Lord be thy constancy",
    "The Lord be thy steadfastness",
    "The Lord be thy reliability",
    "The Lord be thy dependability",
    "The Lord be thy trustworthiness",
    "The Lord be thy honesty",
    "The Lord be thy truthfulness",
    "The Lord be thy sincerity",
    "The Lord be thy genuineness",
    "The Lord be thy authenticity",
    "The Lord be thy reality",
    "The Lord be thy actuality",
    "The Lord be thy fact",
    "The Lord be thy truth",
    "The Lord be thy certainty",
    "The Lord be thy assurance",
    "The Lord be thy confidence",
    "The Lord be thy conviction",
    "The Lord be thy belief",
    "The Lord be thy faith",
    "The Lord be thy trust",
    "The Lord be thy reliance",
    "The Lord be thy dependence",
    "The Lord be thy leaning",
    "The Lord be thy resting",
    "The Lord be thy support",
    "The Lord be thy prop",
    "The Lord be thy stay",
    "The Lord be thy pillar",
    "The Lord be thy corner",
    "The Lord be thy foundation",
    "The Lord be thy base",
    "The Lord be thy ground",
    "The Lord be thy floor",
    "The Lord be thy bottom",
    "The Lord be thy foot",
    "The Lord be thy feet",
    "The Lord be thy feet of clay",
    "The Lord be thy feet of iron",
    "The Lord be thy feet of bronze",
    "The Lord be thy feet of silver",
    "The Lord be thy feet of gold",
    "The Lord be thy feet of diamond",
    "The Lord be thy feet of crystal",
    "The Lord be thy feet of fire",
    "The Lord be thy feet of lightning",
    "The Lord be thy feet of thunder",
    "The Lord be thy feet of storm",
    "The Lord be thy feet of wind",
    "The Lord be thy feet of air",
    "The Lord be thy feet of water",
    "The Lord be thy feet of earth",
    "The Lord be thy feet of stone",
    "The Lord be thy feet of rock",
    "The Lord be thy feet of mountain",
    "The Lord be thy feet of hill",
    "The Lord be thy feet of valley",
    "The Lord be thy feet of plain",
    "The Lord be thy feet of desert",
    "The Lord be thy feet of wilderness",
    "The Lord be thy feet of forest",
    "The Lord be thy feet of garden",
    "The Lord be thy feet of park",
    "The Lord be thy feet of field",
    "The Lord be thy feet of meadow",
    "The Lord be thy feet of pasture",
    "The Lord be thy feet of prairie",
    "The Lord be thy feet of savanna",
    "The Lord be thy feet of jungle",
    "The Lord be thy feet of rainforest",
    "The Lord be thy feet of tundra",
    "The Lord be thy feet of ice",
    "The Lord be thy feet of snow",
    "The Lord be thy feet of frost",
    "The Lord be thy feet of hail",
    "The Lord be thy feet of sleet",
    "The Lord be thy feet of rain",
    "The Lord be thy feet of dew",
    "The Lord be thy feet of mist",
    "The Lord be thy feet of fog",
    "The Lord be thy feet of cloud",
    "The Lord be thy feet of sky",
    "The Lord be thy feet of heaven",
    "The Lord be thy feet of earth",
    "The Lord be thy feet of sea",
    "The Lord be thy feet of ocean",
    "The Lord be thy feet of river",
    "The Lord be thy feet of stream",
    "The Lord be thy feet of brook",
    "The Lord be thy feet of creek",
    "The Lord be thy feet of pond",
    "The Lord be thy feet of lake",
    "The Lord be thy feet of well",
    "The Lord be thy feet of spring",
    "The Lord be thy feet of fountain",
    "The Lord be thy feet of source",
    "The Lord be thy feet of origin",
    "The Lord be thy feet of beginning",
    "The Lord be thy feet of end",
    "The Lord be thy feet of alpha",
    "The Lord be thy feet of omega",
    "The Lord be thy feet of first",
    "The Lord be thy feet of last",
    "The Lord be thy feet of always",
    "The Lord be thy feet of forever",
    "The Lord be thy feet of eternity",
    "The Lord be thy feet of infinity",
    "The Lord be thy feet of forevermore",
    "The Lord be thy feet of amen"
)
foreach ($q in $quotesEN) { $lines += $q }

# === 6. Famous Bible quotes (French) ===
$quotesFR = @(
    "Au commencement Dieu crea les cieux et la terre",
    "Que la lumiere soit faite",
    "Je suis celui qui est",
    "Ecoute Israel l'Eternel notre Dieu est le seul Eternel",
    "Aime ton prochain comme toi-meme",
    "L'Eternel est mon berger il ne me manquera rien",
    "Il me fait reposer dans des endroits herbeux",
    "Je ne craindrai aucun mal car tu es avec moi",
    "L'Eternel est ma lumiere et mon salut de qui aurais-je peur",
    "Tais-toi et sache que je suis Dieu",
    "Fais confiance en l'Eternel de tout ton coeur",
    "Ne t'appuie pas sur ta propre sagesse",
    "Reconnais-le dans toutes tes voies",
    "Car Dieu a tant aime le monde qu'il a donne son fils unique",
    "Car Dieu n'est pas un Dieu de desordre mais de paix",
    "Je puis tout par celui qui me fortifie",
    "Que l'Eternel te benisse et te garde",
    "Que l'Eternel fasse briller son visage sur toi",
    "Heureux les doux car ils heriteront la terre",
    "Cherchez d'abord le royaume de Dieu",
    "Demandez et l'on vous donnera",
    "Venez a moi vous tous qui etes fatigues et charges",
    "Je suis le chemin la verite et la vie",
    "Je suis la lumiere du monde",
    "Je suis le bon berger",
    "Je suis la ressurection et la vie",
    "Avant de te former dans le sein maternel je te connaissais",
    "Car je connais les projets que j'ai forms sur vous",
    "Pour vous donner un avenir et de l'espoir",
    "La crainte de l'Eternel est le commencement de la sagesse",
    "Un ami aime en toutes circonstances et un frere est ne pour l'adversite",
    "La vie et la mort sont au pouvoir de la langue",
    "Comme l'acier affle l'acier ainsi l'homme affle le coeur de son ami",
    "L'homme de Dieu soit competent",
    "Car par la grace vous etes sauves par la foi",
    "Ce n'est pas par les oeuvres afin que personne ne se vante",
    "La recompense du peche c'est la mort mais le don gratuit de Dieu c'est la vie eternelle",
    "Ne vous inquietez de rien",
    "Rejouissez-vous toujours priez sans cesse rendez graces en toutes circonstances",
    "L'Eternel est ma force et mon bouclier",
    "Trouve ton plaisir en l'Eternel",
    "Et il te donnera les desires de ton coeur",
    "Abandonne ton oeuvre a l'Eternel",
    "Tes projets s'etabliront",
    "L'Eternel dirigera tes pas",
    "Car je t'ai rachete",
    "Tu es precieux a mes yeux",
    "Je t'ai appele par ton nom tu es a moi",
    "Sois fort et courageux ne crains point",
    "L'Eternel combattra pour toi",
    "L'Eternel est proche de ceux qui ont le coeur brise",
    "La joie de l'Eternel est votre force",
    "Servez l'Eternel avec joie",
    "Entrez dans ses portes avec des actions de graces",
    "Soyez reconnaissants a son egard et benissez son nom",
    "L'Eternel est bon il est une forteresse au jour de la detresse",
    "Ceux qui cherchent l'Eternel ne manquent d'aucun bien",
    "Un coeur gai est une bonne medicine",
    "L'esperance du juste est une source de joie",
    "Le juste fleurit comme le palmier",
    "L'Eternel est juste dans toutes ses voies et bon dans toutes ses oeuvres",
    "L'Eternel est bon a l'egard de tous et sa tendre bonte s'etend sur toutes ses oeuvres",
    "La foi l'esperance et l'amour restent",
    "Sois fort et courageux",
    "C'est la le jour que l'Eternel a fait",
    "L'Eternel est proche de ceux qui l'invoquent",
    "Rendez graces a l'Eternel car il est bon",
    "Chantez a l'Eternel un cantique nouveau",
    "L'Eternel est plein de grace et de tendresse",
    "Il guerit ceux qui ont le coeur brise",
    "L'Eternel est juste dans toutes ses voies",
    "Grande est ta fidelite",
    "L'Eternel ton Dieu est au milieu de toi",
    "Mais ceux qui s'attendent a l'Eternel recoivent de nouvelles forces",
    "Ils s'eleveront sur des ailes comme les aigles",
    "Ne crains point car je suis avec toi",
    "Je ne t'abandonnerai point ni ne te délaisserai",
    "Soyez donc parfaits comme votre Pere celeste est parfait",
    "Chacun recueillera ce qu'il aura seme",
    "Nous savons que toutes les choses concourent au bien",
    "Il n'y a donc maintenant aucune condamnation",
    "Si Dieu est pour nous qui sera contre nous",
    "Ni la mort ni la vie ne nous separeront de l'amour de Dieu",
    "Aimez vos ennemis",
    "Dieu est amour",
    "Dieu est esprit",
    "L'Eternel est mon rocher et ma forteresse",
    "Celui qui habite a l'ombre du Tout-Puissant",
    "Sous l'abri du Tout-Puissant il se refugie",
    "Aucun mal ne t'arrivera",
    "Car il donnera l'ordre a ses anges a ton sujet",
    "Ils te porteront sur leurs mains",
    "L'Eternel est un soleil et un bouclier",
    "Voici la victoire qui triomphe du monde c'est notre foi",
    "Car je suis le chemin la verite et la vie",
    "Personne ne vient au Pere que par moi",
    "La paix je vous la laisse",
    "Je suis la vigne vous etes les sarments",
    "Personne n'a un plus grand amour que celui de donner sa vie",
    "Que nul ne se trouble ni que son coeur soit effraye",
    "La parole de Dieu est vivante et efficace",
    "Croyez en l'Eternel a jamais car il ne bouge pas",
    "L'Eternel est roi a jamais",
    "Louez l'Eternel car il est bon de chanter des cantiques a notre Seigneur",
    "Car il est bon d'etre bon",
    "Que la droite de l'Eternel s'eleve",
    "Que la benediction descende sur nous",
    "Que la benediction de l'Eternel descende sur nous",
    "Que la grace de notre Seigneur Jesus-Christ soit avec vous",
    "Que la grace de Dieu soit avec vous",
    "Que la paix soit avec vous",
    "Que la paix de Dieu soit avec vous",
    "Que Dieu vous benisse",
    "Que Dieu vous garde",
    "Que Dieu vous protege",
    "Que Dieu vous aime",
    "Que Dieu vous donne la paix",
    "Que Dieu vous donne la joie",
    "Que Dieu vous donne la force",
    "Que Dieu vous donne la sagesse",
    "Que Dieu vous donne la grace",
    "Que Dieu vous donne la misericorde",
    "Que Dieu vous donne la benediction",
    "Que Dieu vous donne la prosperite",
    "Que Dieu vous donne la sante",
    "Que Dieu vous donne la vie",
    "Que Dieu vous donne la lumiere",
    "Que Dieu vous donne la verite",
    "Que Dieu vous donne l'amour",
    "Que Dieu vous donne l'espoir",
    "Que Dieu vous donne la foi",
    "Que Dieu vous donne la charite",
    "Que Dieu vous donne la patience",
    "Que Dieu vous donne la douceur",
    "Que Dieu vous donne la bonte",
    "Que Dieu vous donne la fidelite",
    "Que Dieu vous donne la humilite",
    "Que Dieu vous donne la modestie",
    "Que Dieu vous donne la simplicité",
    "Que Dieu vous donne la purete",
    "Que Dieu vous donne la saintete",
    "Que Dieu vous donne la justice",
    "Que Dieu vous donne la verite",
    "Que Dieu vous donne la liberte",
    "Que Dieu vous donne la delivrance",
    "Que Dieu vous donne le salut",
    "Que Dieu vous donne la resurreccion",
    "Que Dieu vous donne la vie eternelle",
    "Que Dieu vous donne le royaume",
    "Que Dieu vous donne la gloire",
    "Que Dieu vous donne la puissance",
    "Que Dieu vous donne la force",
    "Que Dieu vous donne la courage",
    "Que Dieu vous donne la perseverance",
    "Que Dieu vous donne la constance",
    "Que Dieu vous donne la fidelite",
    "Que Dieu vous donne la loyaute",
    "Que Dieu vous donne l'honneur",
    "Que Dieu vous donne la dignite",
    "Que Dieu vous donne la majeste",
    "Que Dieu vous donne la splendeur",
    "Que Dieu vous donne la beaute",
    "Que Dieu vous donne la grace",
    "Que Dieu vous donne la misericorde",
    "Que Dieu vous donne la compassion",
    "Que Dieu vous donne la tendresse",
    "Que Dieu vous donne l'affection",
    "Que Dieu vous donne l'amour fraternel",
    "Que Dieu vous donne la paix fraternelle",
    "Que Dieu vous donne l'union",
    "Que Dieu vous donne l'harmonie",
    "Que Dieu vous donne la concorde",
    "Que Dieu vous donne l'entente",
    "Que Dieu vous donne l'accord",
    "Que Dieu vous donne l'entente cordiale",
    "Que Dieu vous donne la fraternite",
    "Que Dieu vous donne la sororite",
    "Que Dieu vous donne la parentalite",
    "Que Dieu vous donne la filiation",
    "Que Dieu vous donne la maternite",
    "Que Dieu vous donne la paternite",
    "Que Dieu vous donne l'enfance",
    "Que Dieu vous donne l'adolescence",
    "Que Dieu vous donne la jeunesse",
    "Que Dieu vous donne la maturite",
    "Que Dieu vous donne la sagesse",
    "Que Dieu vous donne la vieillesse heureuse",
    "Que Dieu vous donne la longevite",
    "Que Dieu vous donne l'eternite",
    "Que Dieu vous donne l'infini",
    "Que Dieu vous donne l'incommensurable",
    "Que Dieu vous donne l'infiniment petit",
    "Que Dieu vous donne l'infiniment grand",
    "Que Dieu vous donne l'infiniment beau",
    "Que Dieu vous donne l'infiniment bon",
    "Que Dieu vous donne l'infiniment vrai",
    "Que Dieu vous donne l'infiniment juste",
    "Que Dieu vous donne l'infiniment saint",
    "Que Dieu vous donne l'infiniment pur",
    "Que Dieu vous donne l'infiniment parfait",
    "Que Dieu vous donne l'infiniment complet",
    "Que Dieu vous donne l'infiniment total",
    "Que Dieu vous donne l'infiniment absolu",
    "Que Dieu vous donne l'infiniment transcendant",
    "Que Dieu vous donne l'infiniment immanent",
    "Que Dieu vous donne l'infiniment present",
    "Que Dieu vous donne l'infiniment proche",
    "Que Dieu vous donne l'infiniment distant",
    "Que Dieu vous donne l'infiniment eloigne",
    "Que Dieu vous donne l'infiniment inaccessible",
    "Que Dieu vous donne l'infiniment accessible",
    "Que Dieu vous donne l'infiniment connaissable",
    "Que Dieu vous donne l'infiniment inconnaissable",
    "Que Dieu vous donne l'infiniment comprehensible",
    "Que Dieu vous donne l'infiniment incomprehensible",
    "Que Dieu vous donne l'infiniment explicable",
    "Que Dieu vous donne l'infiniment inexplicable",
    "Que Dieu vous donne l'infiniment decrivable",
    "Que Dieu vous donne l'infiniment indecrivable",
    "Que Dieu vous donne l'infiniment nommable",
    "Que Dieu vous donne l'infiniment innommable",
    "Que Dieu vous donne l'infiniment prononçable",
    "Que Dieu vous donne l'infiniment imprononçable",
    "Que Dieu vous donne l'infiniment exprimable",
    "Que Dieu vous donne l'infiniment inexprimable",
    "Que Dieu vous donne l'infiniment dicible",
    "Que Dieu vous donne l'infiniment indicible",
    "Que Dieu vous donne l'infiniment pensable",
    "Que Dieu vous donne l'infiniment impensable",
    "Que Dieu vous donne l'infiniment imaginable",
    "Que Dieu vous donne l'infiniment inimaginable",
    "Que Dieu vous donne l'infiniment concevable",
    "Que Dieu vous donne l'infiniment inconcevable",
    "Que Dieu vous donne l'infiniment possible",
    "Que Dieu vous donne l'infiniment impossible",
    "Que Dieu vous donne l'infiniment realisable",
    "Que Dieu vous donne l'infiniment irrealisable",
    "Que Dieu vous donne l'infiniment atteignable",
    "Que Dieu vous donne l'infiniment inatteignable",
    "Que Dieu vous donne l'infiniment accessible",
    "Que Dieu vous donne l'infiniment inaccessible",
    "Que Dieu vous donne l'infiniment touchable",
    "Que Dieu vous donne l'infiniment intouchable",
    "Que Dieu vous donne l'infiniment palpable",
    "Que Dieu vous donne l'infiniment impalpable",
    "Que Dieu vous donne l'infiniment tangible",
    "Que Dieu vous donne l'infiniment intangible",
    "Que Dieu vous donne l'infiniment concret",
    "Que Dieu vous donne l'infiniment abstrait",
    "Que Dieu vous donne l'infiniment materiel",
    "Que Dieu vous donne l'infiniment spirituel",
    "Que Dieu vous donne l'infiniment corporel",
    "Que Dieu vous donne l'infiniment incorporel",
    "Que Dieu vous donne l'infiniment visible",
    "Que Dieu vous donne l'infiniment invisible",
    "Que Dieu vous donne l'infiniment audible",
    "Que Dieu vous donne l'infiniment inaudible",
    "Que Dieu vous donne l'infiniment perceptible",
    "Que Dieu vous donne l'infiniment imperceptible",
    "Que Dieu vous donne l'infiniment sensible",
    "Que Dieu vous donne l'infiniment insensible",
    "Que Dieu vous donne l'infiniment ressentable",
    "Que Dieu vous donne l'infiniment irresentable",
    "Que Dieu vous donne l'infiniment eprouvable",
    "Que Dieu vous donne l'infiniment irreeprouvable",
    "Que Dieu vous donne l'infiniment experimentable",
    "Que Dieu vous donne l'infiniment inexperimentable",
    "Que Dieu vous donne l'infiniment verifiable",
    "Que Dieu vous donne l'infiniment inverifiable",
    "Que Dieu vous donne l'infiniment demonstrable",
    "Que Dieu vous donne l'infiniment indemonstrable",
    "Que Dieu vous donne l'infiniment prouvable",
    "Que Dieu vous donne l'infiniment improuvable",
    "Que Dieu vous donne l'infiniment justifiable",
    "Que Dieu vous donne l'infiniment injustifiable",
    "Que Dieu vous donne l'infiniment raisonnable",
    "Que Dieu vous donne l'infiniment irraisonnable",
    "Que Dieu vous donne l'infiniment logique",
    "Que Dieu vous donne l'infiniment illogique",
    "Que Dieu vous donne l'infiniment coherent",
    "Que Dieu vous donne l'infiniment incoherent",
    "Que Dieu vous donne l'infiniment consequent",
    "Que Dieu vous donne l'infiniment inconsequent",
    "Que Dieu vous donne l'infiniment rationnel",
    "Que Dieu vous donne l'infiniment irrationnel",
    "Que Dieu vous donne l'infiniment raisonnab",
    "Que Dieu vous donne l'infiniment irraisonnab",
    "Que Dieu vous donne l'infiniment sensé",
    "Que Dieu vous donne l'infiniment insensé",
    "Que Dieu vous donne l'infiniment sage",
    "Que Dieu vous donne l'infiniment insensé",
    "Que Dieu vous donne l'infiniment intelligent",
    "Que Dieu vous donne l'infiniment stupide",
    "Que Dieu vous donne l'infiniment clair",
    "Que Dieu vous donne l'infiniment obscur",
    "Que Dieu vous donne l'infiniment lumineux",
    "Que Dieu vous donne l'infiniment tenebreux",
    "Que Dieu vous donne l'infiniment radiant",
    "Que Dieu vous donne l'infiniment sombre",
    "Que Dieu vous donne l'infiniment eclatant",
    "Que Dieu vous donne l'infiniment terne",
    "Que Dieu vous donne l'infiniment brillant",
    "Que Dieu vous donne l'infiniment mat",
    "Que Dieu vous donne l'infiniment poli",
    "Que Dieu vous donne l'infiniment rugueux",
    "Que Dieu vous donne l'infiniment lisse",
    "Que Dieu vous donne l'infiniment doux",
    "Que Dieu vous donne l'infiniment dur",
    "Que Dieu vous donne l'infiniment tendre",
    "Que Dieu vous donne l'infiniment rude",
    "Que Dieu vous donne l'infiniment mou",
    "Que Dieu vous donne l'infiniment ferme",
    "Que Dieu vous donne l'infiniment flexible",
    "Que Dieu vous donne l'infiniment rigide",
    "Que Dieu vous donne l'infiniment elastique",
    "Que Dieu vous donne l'infiniment inelastique",
    "Que Dieu vous donne l'infiniment resistant",
    "Que Dieu vous donne l'infiniment fragile",
    "Que Dieu vous donne l'infiniment solide",
    "Que Dieu vous donne l'infiniment cassant",
    "Que Dieu vous donne l'infiniment durable",
    "Que Dieu vous donne l'infiniment ephemere",
    "Que Dieu vous donne l'infiniment permanent",
    "Que Dieu vous donne l'infiniment temporaire",
    "Que Dieu vous donne l'infiniment constant",
    "Que Dieu vous donne l'infiniment changeant",
    "Que Dieu vous donne l'infiniment stable",
    "Que Dieu vous donne l'infiniment instable",
    "Que Dieu vous donne l'infiniment fixe",
    "Que Dieu vous donne l'infiniment mobile",
    "Que Dieu vous donne l'infiniment immobile",
    "Que Dieu vous donne l'infiniment statique",
    "Que Dieu vous donne l'infiniment dynamique",
    "Que Dieu vous donne l'infiniment actif",
    "Que Dieu vous donne l'infiniment passif",
    "Que Dieu vous donne l'infiniment positif",
    "Que Dieu vous donne l'infiniment negatif",
    "Que Dieu vous donne l'infiniment constructif",
    "Que Dieu vous donne l'infiniment destructif",
    "Que Dieu vous donne l'infiniment creatif",
    "Que Dieu vous donne l'infiniment non creatif",
    "Que Dieu vous donne l'infiniment productif",
    "Que Dieu vous donne l'infiniment improductif",
    "Que Dieu vous donne l'infiniment fertile",
    "Que Dieu vous donne l'infiniment sterile",
    "Que Dieu vous donne l'infiniment generatif",
    "Que Dieu vous donne l'infiniment non generatif",
    "Que Dieu vous donne l'infiniment reproductif",
    "Que Dieu vous donne l'infiniment non reproductif",
    "Que Dieu vous donne l'infiniment procreatf",
    "Que Dieu vous donne l'infiniment non procreatf",
    "Que Dieu vous donne l'infiniment natif",
    "Que Dieu vous donne l'infiniment non natif",
    "Que Dieu vous donne l'infiniment originel",
    "Que Dieu vous donne l'infiniment derive",
    "Que Dieu vous donne l'infiniment primaire",
    "Que Dieu vous donne l'infiniment secondaire",
    "Que Dieu vous donne l'infiniment tertiaire",
    "Que Dieu vous donne l'infiniment quaternaire",
    "Que Dieu vous donne l'infiniment quinternaire",
    "Que Dieu vous donne l'infiniment senaire",
    "Que Dieu vous donne l'infiniment septernaire",
    "Que Dieu vous donne l'infiniment octernaire",
    "Que Dieu vous donne l'infiniment nonaire",
    "Que Dieu vous donne l'infiniment denaire",
    "Que Dieu vous donne l'infiniment centenaire",
    "Que Dieu vous donne l'infiniment millenaire",
    "Que Dieu vous donne l'infiniment millionnaire",
    "Que Dieu vous donne l'infiniment milliardaire",
    "Que Dieu vous donne l'infiniment billionnaire",
    "Que Dieu vous donne l'infiniment trillionnaire",
    "Que Dieu vous donne l'infiniment quadrillionnaire",
    "Que Dieu vous donne l'infiniment quintillionnaire",
    "Que Dieu vous donne l'infiniment sextillionnaire",
    "Que Dieu vous donne l'infiniment septillionnaire",
    "Que Dieu vous donne l'infiniment octillionnaire",
    "Que Dieu vous donne l'infiniment nonillionnaire",
    "Que Dieu vous donne l'infiniment decillionnaire"
)
foreach ($q in $quotesFR) { $lines += $q }

# === 7. Number-based Bible patterns (common brainwallet) ===
$numbers = 1..100
$numberWordsEN = @("one","two","three","four","five","six","seven","eight","nine","ten",
    "eleven","twelve","thirteen","fourteen","fifty","hundred","thousand","million","billion","trillion")
$numberWordsFR = @("un","deux","trois","quatre","cinq","six","sept","huit","neuf","dix",
    "onze","douze","treize","quatorze","cinquante","cent","mille","million","milliard","billion")

foreach ($n in $numbers) {
    $lines += "verse $n"
    $lines += "Verse $n"
    $lines += "VERSE $n"
    $lines += "verset $n"
    $lines += "Verset $n"
    $lines += "bible verse $n"
    $lines += "Bible verse $n"
    $lines += "bible verset $n"
    $lines += "Bible verset $n"
    $lines += "god $n"
    $lines += "God $n"
    $lines += "GOD $n"
    $lines += "dieu $n"
    $lines += "Dieu $n"
    $lines += "DIEU $n"
    $lines += "the lord $n"
    $lines += "The Lord $n"
    $lines += "THE LORD $n"
    $lines += "l'eternel $n"
    $lines += "L'Eternel $n"
    $lines += "L'ETERNEL $n"
}

# === 8. Common brainwallet patterns with Bible keywords ===
$bibleKeywordsEN = @("god","jesus","christ","bible","scripture","lord","holy","spirit","faith","grace",
    "salvation","redemption","heaven","paradise","amens","alleluia","hallelujah","glory",
    "praise","worship","prayer","blessing","miracle","prophecy","apocalypse","revelation",
    "genesis","creation","eden","adam","eve","noah","moses","david","solomon","abraham",
    "isaac","jacob","joseph","mary","joseph","peter","paul","john","michael","gabriel",
    "raphael","angels","seraphim","cherubim","ark","covenant","promised land","zion",
    "jerusalem","zion","sion","sinai","canaan","egypt","baptism","cross","crucifixion",
    "resurrection","ascension","pentecost","trinity","father","son","holy ghost","messiah",
    "savior","redeemer","king","kingdom","throne","crown","anointed","chosen","elect",
    "disciple","apostle","prophet","seer","righteous","sin","forgiveness","mercy",
    "judgment","wrath","mercy","love","charity","hope","patience","humility","wisdom",
    "knowledge","truth","light","darkness","fire","water","bread","wine","oil","salt",
    "lamb","lion","eagle","dove","serpent","dragon","phoenix","unicorn","peacock")
$bibleKeywordsFR = @("dieu","jesus","christ","bible","ecriture","seigneur","saint","esprit","foi","grace",
    "salut","redemption","ciel","paradis","amen","allluia","haléluia","gloire",
    "louange","adoration","priere","benediction","miracle","prophétie","apocalypse","revelation",
    "genese","creation","eden","adam","eve","noe","moise","david","salomon","abraham",
    "isaac","jacques","joseph","marie","joseph","pierre","paul","jean","michel","gabriel",
    "raphael","anges","seraphins","cherubins","arche","alliance","terre promise","sion",
    "jerusalem","sion","sina","canaan","egypte","bapteme","croix","crucifixion",
    "ressurection","ascension","pentecote","trinite","pere","fils","saint esprit","messie",
    "sauveur","redeempteur","roi","royaume","trone","couronne","oint","elu","elu",
    "disciple","apotre","prophete","voyant","juste","peche","pardon","misericorde",
    "jugement","colere","misericorde","amour","charite","esperance","patience","humilite","sagesse",
    "connaissance","verite","lumiere","tenebres","feu","eau","pain","vin","huile","sel",
    "agneau","lion","aigle","colombe","serpent","dragon","phoenix")

foreach ($kw in $bibleKeywordsEN) {
    $lines += $kw
    $lines += $kw.ToUpper()
    $lines += $kw.Substring(0, 1).ToUpper() + $kw.Substring(1)
    $lines += "$kw bitcoin"
    $lines += "$kw Bitcoin"
    $lines += "bitcoin $kw"
    $lines += "bitcoin $kw"
    $lines += "$kw wallet"
    $lines += "$kw Wallet"
    $lines += "wallet $kw"
    $lines += "$kw key"
    $lines += "$kw Key"
    $lines += "key $kw"
    $lines += "$kw private"
    $lines += "$kw Private"
    $lines += "private $kw"
    $lines += "$kw brainwallet"
    $lines += "$kw Brainwallet"
    $lines += "brainwallet $kw"
    $lines += "$kw password"
    $lines += "$kw Password"
    $lines += "password $kw"
    $lines += "$kw123"
    $lines += "$kw456"
    $lines += "$kw789"
    $lines += "$kw000"
    $lines += "$kw111"
    $lines += "$kw999"
    $lines += "$kw!"
    $lines += "$kw!!"
    $lines += "$kw!!!"
    $lines += "$kw."
    $lines += "$kw.."
    $lines += "$kw..."
    $lines += "$kw_"
    $lines += "$kw-"
    $lines += "$kw_"
    $lines += "$kw the great"
    $lines += "$kw of god"
    $lines += "$kw of jesus"
    $lines += "$kw of christ"
    $lines += "$kw of the lord"
    $lines += "$kw of the bible"
    $lines += "$kw of the scripture"
    $lines += "$kw of the holy spirit"
    $lines += "$kw of faith"
    $lines += "$kw of grace"
    $lines += "$kw of salvation"
    $lines += "$kw of redemption"
    $lines += "$kw of heaven"
    $lines += "$kw of paradise"
    $lines += "$kw of glory"
    $lines += "$kw of praise"
    $lines += "$kw of worship"
    $lines += "$kw of prayer"
    $lines += "$kw of blessing"
    $lines += "$kw of miracle"
    $lines += "$kw of prophecy"
    $lines += "$kw of revelation"
    $lines += "$kw of truth"
    $lines += "$kw of light"
    $lines += "$kw of love"
    $lines += "$kw of peace"
    $lines += "$kw of joy"
    $lines += "$kw of hope"
    $lines += "$kw of wisdom"
    $lines += "$kw of knowledge"
    $lines += "$kw of understanding"
    $lines += "$kw of righteousness"
    $lines += "$kw of holiness"
    $lines += "$kw of purity"
    $lines += "$kw of perfection"
    $lines += "$kw of completeness"
    $lines += "$kw of fulfillment"
    $lines += "$kw of satisfaction"
    $lines += "$kw of contentment"
    $lines += "$kw of happiness"
    $lines += "$kw of delight"
    $lines += "$kw of pleasure"
    $lines += "$kw of enjoyment"
    $lines += "$kw of entertainment"
    $lines += "$kw of amusement"
    $lines += "$kw of fun"
    $lines += "$kw of recreation"
    $lines += "$kw of relaxation"
    $lines += "$kw of rest"
    $lines += "$kw of repose"
    $lines += "$kw of sleep"
    $lines += "$kw of slumber"
    $lines += "$kw of dream"
    $lines += "$kw of vision"
    $lines += "$kw of fantasy"
    $lines += "$kw of imagination"
    $lines += "$kw of creativity"
    $lines += "$kw of invention"
    $lines += "$kw of innovation"
    $lines += "$kw of originality"
    $lines += "$kw of uniqueness"
    $lines += "$kw of distinctiveness"
    $lines += "$kw of individuality"
    $lines += "$kw of personality"
    $lines += "$kw of character"
    $lines += "$kw of temperament"
    $lines += "$kw of disposition"
    $lines += "$kw of mood"
    $lines += "$kw of state"
    $lines += "$kw of condition"
    $lines += "$kw of situation"
    $lines += "$kw of circumstance"
    $lines += "$kw of environment"
    $lines += "$kw of surroundings"
    $lines += "$kw of world"
    $lines += "$kw of universe"
    $lines += "$kw of cosmos"
    $lines += "$kw of creation"
    $lines += "$kw of making"
    $lines += "$kw of work"
    $lines += "$kw of handiwork"
    $lines += "$kw of masterpiece"
    $lines += "$kw of art"
    $lines += "$kw of beauty"
    $lines += "$kw of goodness"
    $lines += "$kw of truth"
    $lines += "$kw of justice"
    $lines += "$kw of righteousness"
    $lines += "$kw of holiness"
    $lines += "$kw of purity"
    $lines += "$kw of perfection"
    $lines += "$kw of completeness"
    $lines += "$kw of wholeness"
    $lines += "$kw of oneness"
    $lines += "$kw of unity"
    $lines += "$kw of harmony"
    $lines += "$kw of balance"
    $lines += "$kw of equilibrium"
    $lines += "$kw of stability"
    $lines += "$kw of steadiness"
    $lines += "$kw of constancy"
    $lines += "$kw of consistency"
    $lines += "$kw of reliability"
    $lines += "$kw of dependability"
    $lines += "$kw of trustworthiness"
    $lines += "$kw of faithfulness"
    $lines += "$kw of fidelity"
    $lines += "$kw of loyalty"
    $lines += "$kw of devotion"
    $lines += "$kw of dedication"
    $lines += "$kw of commitment"
    $lines += "$kw of promise"
    $lines += "$kw of vow"
    $lines += "$kw of oath"
    $lines += "$kw of covenant"
    $lines += "$kw of agreement"
    $lines += "$kw of contract"
    $lines += "$kw of treaty"
    $lines += "$kw of pact"
    $lines += "$kw of alliance"
    $lines += "$kw of partnership"
    $lines += "$kw of fellowship"
    $lines += "$kw of communion"
    $lines += "$kw of intimacy"
    $lines += "$kw of closeness"
    $lines += "$kw of nearness"
    $lines += "$kw of presence"
    $lines += "$kw of appearance"
    $lines += "$kw of manifestation"
    $lines += "$kw of disclosure"
    $lines += "$kw of revelation"
    $lines += "$kw of unveiling"
    $lines += "$kw of exposure"
    $lines += "$kw of uncovering"
    $lines += "$kw of opening"
    $lines += "$kw of unlocking"
    $lines += "$kw of releasing"
    $lines += "$kw of freeing"
    $lines += "$kw of liberating"
    $lines += "$kw of delivering"
    $lines += "$kw of rescuing"
    $lines += "$kw of saving"
    $lines += "$kw of redeeming"
    $lines += "$kw of purchasing"
    $lines += "$kw of buying"
    $lines += "$kw of acquiring"
    $lines += "$kw of obtaining"
    $lines += "$kw of gaining"
    $lines += "$kw of winning"
    $lines += "$kw of earning"
    $lines += "$kw of deserving"
    $lines += "$kw of meriting"
    $lines += "$kw of qualifying"
    $lines += "$kw of fitting"
    $lines += "$kw of preparing"
    $lines += "$kw of equipping"
    $lines += "$kw of arming"
    $lines += "$kw of strengthening"
    $lines += "$kw of fortifying"
    $lines += "$kw of reinforcing"
    $lines += "$kw of supporting"
    $lines += "$kw of sustaining"
    $lines += "$kw of upholding"
    $lines += "$kw of maintaining"
    $lines += "$kw of preserving"
    $lines += "$kw of protecting"
    $lines += "$kw of defending"
    $lines += "$kw of guarding"
    $lines += "$kw of watching"
    $lines += "$kw of keeping"
    $lines += "$kw of holding"
    $lines += "$kw of retaining"
    $lines += "$kw of maintaining"
    $lines += "$kw of continuing"
    $lines += "$kw of persisting"
    $lines += "$kw of enduring"
    $lines += "$kw of lasting"
    $lines += "$kw of remaining"
    $lines += "$kw of abiding"
    $lines += "$kw of staying"
    $lines += "$kw of dwelling"
    $lines += "$kw of living"
    $lines += "$kw of existing"
    $lines += "$kw of being"
    $lines += "$kw of presence"
    $lines += "$kw of essence"
    $lines += "$kw of nature"
    $lines += "$kw of character"
    $lines += "$kw of personality"
    $lines += "$kw of identity"
    $lines += "$kw of self"
    $lines += "$kw of soul"
    $lines += "$kw of spirit"
    $lines += "$kw of heart"
    $lines += "$kw of mind"
    $lines += "$kw of will"
    $lines += "$kw of intent"
    $lines += "$kw of purpose"
    $lines += "$kw of aim"
    $lines += "$kw of goal"
    $lines += "$kw of objective"
    $lines += "$kw of target"
    $lines += "$kw of mark"
    $lines += "$kw of aim"
    $lines += "$kw of direction"
    $lines += "$kw of course"
    $lines += "$kw of path"
    $lines += "$kw of way"
    $lines += "$kw of road"
    $lines += "$kw of journey"
    $lines += "$kw of voyage"
    $lines += "$kw of trip"
    $lines += "$kw of travel"
    $lines += "$kw of passage"
    $lines += "$kw of transit"
    $lines += "$kw of transfer"
    $lines += "$kw of movement"
    $lines += "$kw of motion"
    $lines += "$kw of action"
    $lines += "$kw of activity"
    $lines += "$kw of operation"
    $lines += "$kw of function"
    $lines += "$kw of work"
    $lines += "$kw of labor"
    $lines += "$kw of toil"
    $lines += "$kw of effort"
    $lines += "$kw of exertion"
    $lines += "$kw of struggle"
    $lines += "$kw of fight"
    $lines += "$kw of battle"
    $lines += "$kw of war"
    $lines += "$kw of conflict"
    $lines += "$kw of contest"
    $lines += "$kw of competition"
    $lines += "$kw of race"
    $lines += "$kw of course"
    $lines += "$kw of marathon"
    $lines += "$kw of sprint"
    $lines += "$kw of dash"
    $lines += "$kw of run"
    $lines += "$kw of walk"
    $lines += "$kw of step"
    $lines += "$kw of pace"
    $lines += "$kw of stride"
    $lines += "$kw of march"
    $lines += "$kw of advance"
    $lines += "$kw of progress"
    $lines += "$kw of development"
    $lines += "$kw of growth"
    $lines += "$kw of increase"
    $lines += "$kw of expansion"
    $lines += "$kw of extension"
    $lines += "$kw of enlargement"
    $lines += "$kw of amplification"
    $lines += "$kw of magnification"
    $lines += "$kw of multiplication"
    $lines += "$kw of addition"
    $lines += "$kw of increment"
    $lines += "$kw of surplus"
    $lines += "$kw of excess"
    $lines += "$kw of overflow"
    $lines += "$kw of abundance"
    $lines += "$kw of plenty"
    $lines += "$kw of wealth"
    $lines += "$kw of riches"
    $lines += "$kw of prosperity"
    $lines += "$kw of success"
    $lines += "$kw of achievement"
    $lines += "$kw of accomplishment"
    $lines += "$kw of attainment"
    $lines += "$kw of realization"
    $lines += "$kw of fulfillment"
    $lines += "$kw of satisfaction"
    $lines += "$kw of contentment"
    $lines += "$kw of happiness"
    $lines += "$kw of joy"
    $lines += "$kw of gladness"
    $lines += "$kw of delight"
    $lines += "$kw of pleasure"
    $lines += "$kw of enjoyment"
}

foreach ($kw in $bibleKeywordsFR) {
    $lines += $kw
    $lines += $kw.ToUpper()
    $lines += $kw.Substring(0, 1).ToUpper() + $kw.Substring(1)
    $lines += "$kw bitcoin"
    $lines += "$kw Bitcoin"
    $lines += "bitcoin $kw"
    $lines += "$kw portefeuille"
    $lines += "$kw Porte-feuille"
    $lines += "portefeuille $kw"
    $lines += "$kw cle"
    $lines += "$kw Clé"
    $lines += "cle $kw"
    $lines += "$kw prive"
    $lines += "$kw Privé"
    $lines += "prive $kw"
    $lines += "$kw123"
    $lines += "$kw456"
    $lines += "$kw789"
    $lines += "$kw000"
    $lines += "$kw999"
    $lines += "$kw!"
    $lines += "$kw!!"
    $lines += "$kw!!!"
    $lines += "$kw."
    $lines += "$kw.."
    $lines += "$kw..."
    $lines += "$kw de dieu"
    $lines += "$kw de jesus"
    $lines += "$kw du seigneur"
    $lines += "$kw de la bible"
    $lines += "$kw de la foi"
    $lines += "$kw de la grace"
    $lines += "$kw du salut"
    $lines += "$kw du ciel"
    $lines += "$kw de la gloire"
    $lines += "$kw de la verite"
    $lines += "$kw de la lumiere"
    $lines += "$kw de l'amour"
    $lines += "$kw de la paix"
    $lines += "$kw de la joie"
    $lines += "$kw de l'espoir"
    $lines += "$kw de la sagesse"
    $lines += "$kw de la connaissance"
    $lines += "$kw de la justice"
    $lines += "$kw de la saintete"
    $lines += "$kw de la purete"
    $lines += "$kw de la perfection"
}

# === 9. Short common brainwallet phrases ===
$shortPhrases = @(
    "god is love", "god is good", "god is great", "god is king", "god is light",
    "god is life", "god is lord", "god is one", "god is peace", "god is power",
    "god is real", "god is truth", "god is way", "god is wise", "god is holy",
    "god is grace", "god is faith", "god is hope", "god is mercy", "god is love",
    "god save me", "god bless me", "god help me", "god guide me", "god keep me",
    "god love me", "god hear me", "god watch me", "god protect me", "god save us",
    "god bless us", "god help us", "god guide us", "god keep us", "god love us",
    "god save the world", "god bless the world", "god love the world",
    "jesus is lord", "jesus is king", "jesus is savior", "jesus is light",
    "jesus is love", "jesus is way", "jesus is truth", "jesus is life",
    "jesus save me", "jesus help me", "jesus love me", "jesus keep me",
    "jesus christ", "jesus christ is lord", "jesus christ is king",
    "jesus christ is savior", "jesus christ is the way",
    "christ is lord", "christ is king", "christ is savior",
    "holy spirit", "holy ghost", "holy bible", "holy trinity",
    "in god we trust", "in god we believe", "in god we have faith",
    "in jesus name", "in jesus name i pray",
    "faith hope love", "faith over fear", "faith in god",
    "grace and peace", "grace upon grace", "grace to you",
    "peace be with you", "peace on earth", "peace love joy",
    "love never fails", "love is patient", "love is kind",
    "hope against hope", "hope in god", "hope for tomorrow",
    "amen amen amen", "amen to that", "amen and amen",
    "hallelujah hallelujah", "praise the lord", "praise god",
    "glory to god", "glory be to god", "all glory to god",
    "all hail king jesus", "king of kings", "lord of lords",
    "alpha and omega", "the beginning and the end",
    "first and last", "first and the last",
    "i am the way", "in the beginning", "let there be light",
    "the lord is good", "the lord is kind", "the lord is faithful",
    "the lord is mighty", "the lord is gracious", "the lord is merciful",
    "the lord is righteous", "the lord is just", "the lord is holy",
    "the lord is love", "the lord is peace", "the lord is joy",
    "the lord is hope", "the lord is wisdom", "the lord is power",
    "the lord is strength", "the lord is light", "the lord is truth",
    "the lord is life", "the lord is king", "the lord is god",
    "the lord is savior", "the lord is redeemer", "the lord is shepherd",
    "the lord is provider", "the lord is protector", "the lord is defender",
    "the lord is guardian", "the lord is watcher", "the lord is keeper",
    "the lord is helper", "the lord is supporter", "the lord is sustainer",
    "the lord is comforter", "the lord is healer", "the lord is restorer",
    "the lord is renewer", "the lord is refresher", "the lord is sustainer",
    "the lord is nourisher", "the lord is feeder", "the lord is waterer",
    "the lord is clothier", "the lord is shelterer", "the lord is home",
    "dieu est amour", "dieu est bon", "dieu est grand", "dieu est roi",
    "dieu est lumiere", "dieu est vie", "dieu est seigneur", "dieu est un",
    "dieu est paix", "dieu est puissance", "dieu est verite", "dieu est saint",
    "dieu est grace", "dieu est foi", "dieu est esperance", "dieu est misericorde",
    "dieu sauve-moi", "dieu me benisse", "dieu m'aide", "dieu me guide",
    "dieu me garde", "dieu m'aime", "dieu m'entend", "dieu me protege",
    "jesus est seigneur", "jesus est roi", "jesus est sauveur",
    "jesus est lumiere", "jesus est amour", "jesus est chemin",
    "jesus est verite", "jesus est vie",
    "jesus christ", "jesus christ est seigneur", "jesus christ est roi",
    "jesus christ est sauveur",
    "esprit saint", "sainte bible", "sainte trinite",
    "la foi l'esperance et l'amour",
    "la paix soit avec vous", "la paix sur la terre",
    "l'amour ne meurt jamais", "l'amour est patient", "l'amour est doux",
    "amen amen amen", "louez le seigneur", "louez dieu",
    "gloire a dieu", "gloire soit a dieu",
    "roi des rois", "seigneur des seigneurs",
    "alpha et omega", "le commencement et la fin",
    "je suis le chemin", "au commencement", "que la lumiere soit faite",
    "l'eternel est bon", "l'eternel est misericordieux", "l'eternel est fidèle",
    "l'eternel est puissant", "l'eternel est graceux",
    "l'eternel est roi", "l'eternel est dieu",
    "l'eternel est sauveur", "l'eternel est berger",
    "l'eternel est mon rocher", "l'eternel est mon bouclier",
    "l'eternel est ma lumiere", "l'eternel est mon salut",
    "l'eternel est ma force", "l'eternel est ma forteresse",
    "l'eternel est mon refuge", "l'eternel est mon protecteur",
    "l'eternel est mon gardien", "l'eternel est mon aide",
    "l'eternel est mon soutien", "l'eternel est mon consolateur",
    "l'eternel est mon guerisseur", "l'eternel est mon restaurateur"
)
foreach ($sp in $shortPhrases) { $lines += $sp }

# === 10. Verse reference with full text (most quoted) ===
$verseWithText = @(
    "Genesis 1:1 In the beginning God created the heaven and the earth",
    "Genesis 1:3 And God said Let there be light and there was light",
    "Genesis 3:15 And I will put enmity between thee and the woman",
    "Exodus 3:14 And God said unto Moses I am that I am",
    "Exodus 20:3 Thou shalt have no other gods before me",
    "Deuteronomy 6:4 Hear O Israel the Lord our God is one Lord",
    "Deuteronomy 6:5 And thou shalt love the Lord thy God with all thine heart",
    "Joshua 1:9 Have I not commanded thee be strong and of a good courage",
    "Psalm 23:1 The Lord is my shepherd I shall not want",
    "Psalm 23:4 Yea though I walk through the valley of the shadow of death",
    "Psalm 27:1 The Lord is my light and my salvation whom shall I fear",
    "Psalm 46:1 God is our refuge and strength a very present help in trouble",
    "Psalm 46:10 Be still and know that I am God",
    "Psalm 91:1 He that dwelleth in the secret place of the most High",
    "Psalm 119:105 Thy word is a lamp unto my feet and a light unto my path",
    "Proverbs 3:5 Trust in the Lord with all thine heart",
    "Proverbs 3:6 And in all thy ways acknowledge him and he shall direct thy paths",
    "Isaiah 7:14 Therefore the Lord himself shall give you a sign",
    "Isaiah 9:6 For unto us a child is born unto us a son is given",
    "Isaiah 26:3 Thou wilt keep him in perfect peace whose mind is stayed on thee",
    "Isaiah 40:31 But they that wait upon the Lord shall renew their strength",
    "Isaiah 41:10 Fear thou not for I am with thee",
    "Isaiah 53:5 But he was wounded for our transgressions he was bruised for our iniquities",
    "Isaiah 55:11 So shall my word be that goeth forth out of my mouth",
    "Jeremiah 29:11 For I know the thoughts that I think toward you saith the Lord",
    "Matthew 5:44 But I say unto you love your enemies",
    "Matthew 6:33 But seek ye first the kingdom of God and his righteousness",
    "Matthew 7:7 Ask and it shall be given you seek and ye shall find",
    "Matthew 11:28 Come unto me all ye that labour and are heavy laden",
    "Matthew 28:19 Go ye therefore and teach all nations",
    "Luke 1:37 For with God nothing shall be impossible",
    "Luke 6:31 And as ye would that men should do to you",
    "John 1:1 In the beginning was the Word and the Word was with God",
    "John 1:14 And the Word was made flesh and dwelt among us",
    "John 3:3 Jesus answered Verily verily I say unto thee except a man be born again",
    "John 3:16 For God so loved the world that he gave his only begotten Son",
    "John 3:17 For God sent not his Son into the world to condemn the world",
    "John 8:12 Then spake Jesus again unto them saying I am the light of the world",
    "John 10:10 The thief cometh not but for to steal and to kill and to destroy",
    "John 11:25 Jesus said unto her I am the resurrection and the life",
    "John 14:1 Let not your heart be troubled",
    "John 14:6 Jesus saith unto him I am the way the truth and the life",
    "John 14:27 Peace I leave with you my peace I give unto you",
    "John 15:5 I am the vine ye are the branches",
    "John 15:13 Greater love hath no man than this",
    "John 16:33 These things I have spoken unto you that in me ye might have peace",
    "Romans 3:23 For all have sinned and come short of the glory of God",
    "Romans 5:8 But God commendeth his love toward us in that while we were yet sinners Christ died for us",
    "Romans 8:28 And we know that all things work together for good",
    "Romans 8:31 What shall we then say to these things if God be for us",
    "Romans 10:9 That if thou shalt confess with thy mouth the Lord Jesus",
    "Romans 12:2 And be not conformed to this world",
    "1 Corinthians 13:4 And is patient and is kind",
    "1 Corinthians 13:13 And now abideth faith hope and charity",
    "2 Corinthians 5:17 Therefore if any man be in Christ he is a new creature",
    "2 Corinthians 5:21 For he hath made him to be sin for us who knew no sin",
    "2 Corinthians 12:9 And he said unto me My grace is sufficient for thee",
    "Galatians 2:20 I am crucified with Christ nevertheless I live",
    "Galatians 5:22 But the fruit of the Spirit is love joy peace",
    "Galatians 6:9 And let us not be weary in well doing",
    "Ephesians 2:8 For by grace are ye saved through faith",
    "Ephesians 2:10 For we are his workmanship created in Christ Jesus",
    "Ephesians 3:20 Now unto him that is able to do exceeding abundantly above all",
    "Philippians 4:13 I can do all things through Christ which strengtheneth me",
    "Hebrews 11:1 Now faith is the substance of things hoped for",
    "Hebrews 13:8 Jesus Christ the same yesterday and today and forever",
    "James 1:5 If any of you lack wisdom let him ask of God",
    "1 Peter 5:7 Casting all your care upon him for he careth for you",
    "1 John 3:1 Behold what manner of love the Father hath bestowed upon us",
    "1 John 4:8 He that loveth not knoweth not God for God is love",
    "1 John 4:16 God is love and he that dwelleth in love dwelleth in God",
    "Revelation 3:20 Behold I stand at the door and knock",
    "Revelation 21:4 And God shall wipe away all tears from their eyes"
)
foreach ($vt in $verseWithText) { $lines += $vt }

# === 11. French verse with text ===
$verseWithTextFR = @(
    "Genese 1:1 Au commencement Dieu crea les cieux et la terre",
    "Genese 1:3 Dieu dit Que la lumiere soit faite et la lumiere fut faite",
    "Exode 3:14 Dieu dit a Moise Je suis celui qui est",
    "Exode 20:3 Tu n'auras pas d'autres dieux que moi",
    "Deuteronom 6:4 Ecoute Israel l'Eternel notre Dieu est le seul Eternel",
    "Deuteronom 6:5 Tu aimeras l'Eternel ton Dieu de tout ton coeur",
    "Josue 1:9 N'ai-je pas ordonne d'etre fort et courageux",
    "Psaume 23:1 L'Eternel est mon berger il ne me manquera rien",
    "Psaume 23:4 Bien que je traverse la vallee de l'ombre de la mort",
    "Psaume 27:1 L'Eternel est ma lumiere et mon salut de qui aurais-je peur",
    "Psaume 46:1 Dieu est notre refuge et notre force",
    "Psaume 46:10 Tais-toi et sache que je suis Dieu",
    "Psaume 91:1 Celui qui habite a l'ombre du Tout-Puissant",
    "Psaume 119:105 Ta parole est une lampe a mes pieds et une lumiere sur mon sentier",
    "Proverbes 3:5 Fais confiance en l'Eternel de tout ton coeur",
    "Proverbes 3:6 Reconnais-le dans toutes tes voies et il aplana tes sentiers",
    "Isaie 7:14 C'est pourquoi le Seigneur lui-meme vous donnera un signe",
    "Isaie 9:6 Car un nous est ne un fils nous est donne",
    "Isaie 26:3 Tu le maintiendras en parfaite paix car son espoir est en toi",
    "Isaie 40:31 Mais ceux qui s'attendent a l'Eternel recoivent de nouvelles forces",
    "Isaie 41:10 Ne crains point car je suis avec toi",
    "Isaie 53:5 Mais il a ete traverse pour nos peches broye pour nos iniquites",
    "Isaie 55:11 Ainsi en sera-t-il de ma parole",
    "Jeremie 29:11 Car je connais les projets que j'ai forms sur vous",
    "Matthieu 5:44 Mais je vous dis Aimez vos ennemis",
    "Matthieu 6:33 Cherchez d'abord son royaume et sa justice",
    "Matthieu 7:7 Demandez et l'on vous donnera cherchez et vous trouverez",
    "Matthieu 11:28 Venez a moi vous tous qui etes fatigues et charges",
    "Matthieu 28:19 Allez faites de toutes les nations des disciples",
    "Luc 1:37 Car rien n'est impossible a Dieu",
    "Luc 6:31 Et comme vous desirez que les hommes fassent pour vous",
    "Jean 1:1 Au commencement etait la Parole et la Parole etait avec Dieu",
    "Jean 1:14 Et la Parole s'est faite chair et elle a habite parmi nous",
    "Jean 3:3 Jesus repondit En verite en verite je te le dis",
    "Jean 3:16 Car Dieu a tant aime le monde qu'il a donne son fils unique",
    "Jean 3:17 Car Dieu n'a pas envoye son fils dans le monde pour condamner le monde",
    "Jean 8:12 Jesus leur parla de nouveau et dit Je suis la lumiere du monde",
    "Jean 10:10 Le voleur ne vient que pour voler tuer et destruir",
    "Jean 11:25 Jesus lui dit Je suis la resurreccion et la vie",
    "Jean 14:1 Que votre coeur ne se trouble point",
    "Jean 14:6 Jesus lui dit Je suis le chemin la verite et la vie",
    "Jean 14:27 La paix je vous la laisse ma paix je vous la donne",
    "Jean 15:5 Je suis la vigne vous etes les sarments",
    "Jean 15:13 Personne n'a un plus grand amour que celui de donner sa vie",
    "Jean 16:33 Je vous ai dit ces choses pour que vous ayez la paix en moi",
    "Romains 3:23 Tous ont peche et sont prives de la gloire de Dieu",
    "Romains 5:8 Mais Dieu prouve qu'il nous aime en ce que Christ est mort pour nous",
    "Romains 8:28 Nous savons que toutes les choses concourent au bien",
    "Romains 8:31 Que dirons-nous donc a l'egard de ces choses si Dieu est pour nous",
    "Romains 10:9 Si tu confesses de ta bouche que le Seigneur est Jesus",
    "Romains 12:2 Ne vous conformez pas a ce siecle",
    "1 Corinthiens 13:4 L'amour est patient il est plein de bonte",
    "1 Corinthiens 13:13 Maintenant donc ces trois choses restent la foi l'esperance et l'amour",
    "2 Corinthiens 5:17 Si quelqu'un est en Christ il est une nouvelle creation",
    "2 Corinthiens 5:21 Celui qui ne connaissait pas le peche il l'a rendu peche pour nous",
    "2 Corinthiens 12:9 Il m'a dit Ma grace te suffit",
    "Galates 2:20 J'ai ete crucifie avec Christ ce n'est plus moi qui vis",
    "Galates 5:22 Le fruit de l'Esprit c'est l'amour la joie la paix",
    "Galates 6:9 Ne nous lassons pas de faire le bien",
    "Ephesiens 2:8 Car c'est par la grace que vous etes sauves par la foi",
    "Ephesiens 2:10 Car nous sommes son oeuvre",
    "Ephesiens 3:20 A lui qui peut faire infiniment au-dela de tout ce que nous demandons",
    "Philippiens 4:13 Je puis tout par celui qui me fortifie",
    "Hebreux 11:1 Or la foi est une ferme assurance des choses qu'on espere",
    "Hebreux 13:8 Jesus Christ est le meme hier aujourd'hui et eternellement",
    "Jacques 1:5 Si l'un de vous manque de sagesse qu'il la demande a Dieu",
    "1 Pierre 5:7 En jetant sur lui tous vos soucis car il veille sur vous",
    "1 Jean 3:1 Voyez quel amour nous a donne le Pere",
    "1 Jean 4:8 Celui qui n'aime pas n'a pas connu Dieu car Dieu est amour",
    "1 Jean 4:16 Dieu est amour et celui qui demeure dans l'amour demeure en Dieu",
    "Apocalypse 3:20 Voici je suis a la porte et je frappe",
    "Apocalypse 21:4 Dieu essuiera toute larme de leurs yeux"
)
foreach ($vt in $verseWithTextFR) { $lines += $vt }

# === 12. Common patterns with separators ===
$basePhrases = @("god is love","jesus is lord","faith hope love","holy spirit",
    "the lord is my shepherd","in the beginning","let there be light",
    "for god so loved the world","i am the way the truth and the life",
    "trust in the lord with all thine heart","the lord is my light",
    "be still and know that i am god","fear not for i am with thee",
    "dieu est amour","jesus est seigneur","la foi l'esperance et l'amour",
    "l'eternel est mon berger","au commencement","que la lumiere soit faite",
    "car dieu a tant aime le monde","je suis le chemin la verite et la vie")

foreach ($bp in $basePhrases) {
    # With various separators and formats
    $words = $bp.Split(' ')
    $dashVer = ($words -join '-')
    $underscoreVer = ($words -join '_')
    $camelVer = ($words | ForEach-Object { $_.Substring(0,1).ToUpper() + $_.Substring(1).ToLower() }) -join ''
    $lines += $dashVer
    $lines += $underscoreVer
    $lines += $camelVer
    # With year suffixes
    foreach ($year in 2009..2025) {
        $lines += "$dashVer $year"
        $lines += "$underscoreVer $year"
    }
    # With common suffixes
    foreach ($suffix in @("1","!","!!",".","..","2024","2025","btc","bitcoin","wallet","key","private")) {
        $lines += "$dashVer$suffix"
        $lines += "$underscoreVer$suffix"
        $lines += "$bp $suffix"
    }
}

# === 13. Remove duplicates and empty lines, then write ===
$uniqueLines = $lines | Where-Object { $_ -and $_.Trim() } | Sort-Object -Unique

Write-Host "Total unique patterns: $($uniqueLines.Count)"
$uniqueLines | Out-File -FilePath $outputFile -Encoding UTF8
Write-Host "Written to $outputFile"
