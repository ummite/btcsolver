// pop_culture_2009_keys.rs — Générateur de phrases brainwallet populaires (2009-era)
// Catégories: citations celebres, proverbes, culture pop, phrases courantes, versets bibliques
// Transformations: SHA256, double-SHA256

use sha2::{Sha256, Digest};
use std::collections::BTreeSet;
use std::io::Write;

fn sha256(data: &[u8]) -> [u8; 32] {
    let h = Sha256::digest(data);
    h.into()
}

fn dsha256(data: &[u8]) -> [u8; 32] {
    let h1 = Sha256::digest(data);
    let h2 = Sha256::digest(&h1);
    h2.into()
}

fn add(known: &mut BTreeSet<[u8; 32]>, phrases: &[&str]) {
    for p in phrases {
        let pb = p.as_bytes();
        let h1 = sha256(pb);
        let h2 = dsha256(pb);
        // Also try lowercase
        let pl = p.to_lowercase();
        let h3 = sha256(pl.as_bytes());
        let h4 = dsha256(pl.as_bytes());
        for h in [h1, h2, h3, h4] {
            known.insert(h);
        }
    }
}

fn add_with_years(known: &mut BTreeSet<[u8; 32]>, base: &str, years: &[u16]) {
    for &y in years {
        let p = format!("{}{}", base, y);
        let pb = p.as_bytes();
        known.insert(sha256(pb));
        known.insert(dsha256(pb));
    }
}

fn main() {
    let mut known = BTreeSet::new();
    let years = [1980u16, 1981, 1982, 1983, 1984, 1985, 1986, 1987, 1988, 1989,
                 1990, 1991, 1992, 1993, 1994, 1995, 1996, 1997, 1998, 1999,
                 2000, 2001, 2002, 2003, 2004, 2005, 2006, 2007, 2008, 2009];

    println!("=== Pop Culture 2009 Brainwallet Generator ===");
    println!("Generating keys from famous quotes, proverbs, pop culture phrases...");

    // 1. Movie quotes (English — most common brainwallet language)
    add(&mut known, &[
        "To be or not to be", "May the force be with you", "I'll be back",
        "Here's looking at you kid", "Elementary my dear Watson",
        "E.T. phone home", "Just keep swimming", "After all tomorrow and tomorrow and tomorrow",
        "You talking to me", "I am Groot", "Hasta la vista baby",
        "I'll have what she's having", "Say hello to my little friend",
        "You can't handle the truth", "There's no place like home",
        "I see dead people", "Why so serious", "Why god why",
        "To infinity and beyond", "Life is like a box of chocolates",
        "Frankly my dear I don't give a damn", "I'm gonna make him an offer he can't refuse",
        "You shall not pass", "I am your father", "Roads go on forever",
        "It's alive it's alive", "I am iron man", "I am the king of the world",
        "My precious", "One ring to rule them all", "A martian named smith",
        "Keep your friends close but your enemies closer", "I drink your milkshake",
        "Wake up Neo", "There is no spoon", "Follow the white rabbit",
        "Knock knock Neo", "Free your mind", "The answer is out there",
        "I know kung fu", "With great power comes great responsibility",
        "I am become death", "Carpe diem", "Seize the day",
        "Houston we have a problem", "Say hello to the space age",
        "Beam me up Scotty", "Space the final frontier",
        "Great expectations", "The greatest story ever told",
        "Bon voyage Charlie", "Rosebud", "I'm walking here",
        "Louis I've made a huge mistake", "Get busy living or get busy dying",
        "Show me the money", "Talk to the hand", "Fake news",
        "It's not just a job it's a lifestyle", "Just do it",
        "Think different", "I'm lovin it", "The best laid plans",
        "May the odds be ever in your favor", "I volunteer as tribute",
        "After all this time always", "Fear is the mind killer",
        "In the beginning was the word", "All that glitters is not gold",
        "The quick brown fox jumps over the lazy dog",
        "A penny saved is a penny earned", "Actions speak louder than words",
        "Better late than never", "Beauty is in the eye of the beholder",
        "Birds of a feather flock together", "Cleanliness is next to godliness",
        "Curiosity killed the cat", "Don't count your chickens",
        "Don't look a gift horse in the mouth", "Easy come easy go",
        "Every cloud has a silver lining", "Fortune favors the bold",
        "Give the devil his due", "Good things come to those who wait",
        "Half a loaf is better than no bread", "Haste makes waste",
        "Hope for the best prepare for the worst", "Ignorance is bliss",
        "If the shoe fits wear it", "It takes two to tango",
        "Kill two birds with one stone", "Laughter is the best medicine",
        "Let the good times roll", "Look before you leap",
        "Make hay while the sun shines", "Necessity is the mother of invention",
        "No pain no gain", "Once bitten twice shy",
        "Patience is a virtue", "Practice makes perfect",
        "Rome was not built in a day", "Slow and steady wins the race",
        "The early bird catches the worm", "The pen is mightier than the sword",
        "There but for the grace of god go I", "Two heads are better than one",
        "Unity is strength", "Veni vidi vici", "War is peace",
        "When in Rome do as the Romans do", "You reap what you sow",
    ]);

    // 2. Biblical verses (very popular brainwallet choice)
    add(&mut known, &[
        "In the beginning god created the heaven and the earth",
        "For god so loved the world", "I am the way the truth and the life",
        "The lord is my shepherd I shall not want",
        "Trust in the lord with all thine heart",
        "Be still and know that I am god",
        "The lord is my light and my salvation",
        "Fear not for I am with thee",
        "Jesus wept", "God is love",
        "Love is patient love is kind",
        "Blessed are the poor in spirit",
        "Give ye therefore to the godly",
        "The lord bless thee and keep thee",
        "Thou shalt not steal", "Thou shalt not kill",
        "I can do all things through christ",
        "He that believeth and is baptized shall be saved",
        "Repent for the kingdom of heaven is at hand",
        "Let there be light", "God created man in his own image",
        "And god saw that it was good",
        "The apple of his eye", "A blessing and a curse",
        "The road to hell is paved with good intentions",
        "He who laughs last laughs best",
        "God helps those who help themselves",
        "The spirit is willing but the flesh is weak",
        "To thine own self be true",
        "All is well that ends well",
        "The grass is always greener on the other side",
        "When the going gets tough the tough get going",
        "What goes around comes around",
        "What you seek is seeking you",
        "The journey of a thousand miles begins with one step",
        "Know thyself", "Know thy enemy",
        "Know the truth and the truth shall set you free",
        "Faith hope and love", "Faith without works is dead",
        "Pray without ceasing", "Give thanks in all circumstances",
        "Rejoice evermore", "Serve the lord",
        "Seek ye first the kingdom of god",
        "Ask and it shall be given", "Knock and it shall be opened",
        "The lord is gracious and full of compassion",
        "A new commandment I give love one another",
        "Greater love hath no man than this",
        "The meek shall inherit the earth",
        "Blessed are the peacemakers",
        "Blessed are they that mourn for they shall be comforted",
        "Blessed are the merciful", "Blessed are the pure in heart",
        "Blessed are they which are persecuted",
        "Ye are the salt of the earth",
        "Ye are the light of the world",
        "No man can serve two masters",
        "Render unto caesar what is caesars",
        "Suffer the little children to come unto me",
        "Whatsoever ye do do it heartily",
        "Be strong and of a good courage",
        "This too shall pass",
    ]);

    // 3. Song lyrics (2000s-2009 hits)
    add(&mut known, &[
        "I got 99 problems but a bitch aint one",
        "We are the world we are the children",
        "Imagine all the living in peace you imagine",
        "No woman no cry", "I will survive",
        "I want to dance with somebody",
        "We will rock you we will rock you",
        "Bohemian rhapsody", "Is this the real life",
        "Stairway to heaven", "Free bird",
        "Sweet child o mine", "Hotel california",
        "Yesterday all my troubles seemed so far away",
        "Let it be let it be let it be",
        "Hey Jude dont be afraid you were made to go out and win",
        "The long and winding road",
        "Come together", "Revolution",
        "All you need is love",
        "Hey there delilah", "Take me home country roads",
        "I still cant believe shes gone",
        "Every rose has its thorn",
        "Baby one more time", "Oops I did it again",
        "Toxic", "Halo", "Single ladies",
        "Umbrella", "Low", "Bleeding love",
        "Poker face", "Just dance", "Bad romance",
        "Viva la vida", "Clocks", "Fix you",
        "Yellow cold and lonely",
        "Boulevard of broken dreams",
        "Sugar sweet harmony",
        "My heart will go on",
        "I will always love you",
        "Nothing compares 2 u",
        "I didnt mean to turn around",
        "Total eclipse of the heart",
        "Like a virgin", "Like a prayer",
        "Material girl", "Vogue",
        "Thriller", "Billie Jean",
        "Smooth criminal", "Beat it",
        "Heal the world", "Man in the mirror",
        "Black or white", "Earth song",
        "Tears in heaven", "Wonderwall",
        "Smells like teen spirit",
        "Loser", "Under the bridge",
        "Californication", "Scar tissue",
        "Seven nation army", "More than a feeling",
        "Dont stop believin", "Living on a prayer",
        "Walk this way", "Pour some sugar on me",
        "I wanna dance with somebody",
        "You are the best thing",
        "All star", "Bad guy",
        "Old town road", "Shape of you",
        "Despacito", "Gangnam style",
        "Baby shark", "Wrecking ball",
        "Rolling in the deep", "Someone like you",
        "Firework", "Diamonds", "Roar",
        "Shake it off", "Blank space",
        "Bad blood", "Look what you made me do",
        "This is me", "Let it go",
        "Do you want to build a snowman",
        "A whole new world", "Be our guest",
        "Beauty and the beast", "Under the sea",
        "Kiss the girl", "Part of your world",
        "Colors of the wind", "Hakuna matata",
        "Circle of life", "Can you feel the love tonight",
        "I just cant wait to be king",
        "Hakuna matata whats that word ah nuthing worried",
    ]);

    // 4. Famous quotes & sayings
    add(&mut known, &[
        "The only thing we have to fear is fear itself",
        "That's one small step for man one giant leap for mankind",
        "I have a dream", "Freedom is never voluntarily given",
        "Ask not what your country can do for you",
        "The time has come to fan into flames that dying spark",
        "We shall fight on the beaches",
        "Blood sweat and tears",
        "Make love not war", "Peace love and understanding",
        "Power to the people",
        "The revolution will not be televised",
        "All power flows from the land",
        "Knowledge is power", "Power corrupts absolute power corrupts absolutely",
        "Give me liberty or give me death",
        "Life liberty and the pursuit of happiness",
        "E pluribus unum", "In god we trust",
        "The brave do not live forever they live forever",
        "I think therefore I am",
        "Man is born free and everywhere he is in chains",
        "The unexamined life is not worth living",
        "I know that I know nothing",
        "To each his own", "Carpe diem quam minimum credula postero",
        "Memento mori", "Memento vivere",
        "Per aspera ad astra",
        "The only constant is change",
        "That which does not kill us makes us stronger",
        "God is dead", "Man is a rope stretched between beast and overman",
        "He who has a why to live can bear almost any how",
        "The mind is everything what you think you become",
        "In the middle of difficulty lies opportunity",
        "The best time to plant a tree was 20 years ago the second best time is now",
        "Success is not final failure is not fatal it is the courage to continue that counts",
        "It does not matter how slowly you go as long as you do not stop",
        "The harder you work the harder it will seem",
        "Push yourself because no one else is going to do it for you",
        "Great things never come from comfort zones",
        "Dream it wish it do it",
        "Stay hungry stay foolish",
        "The future belongs to those who believe in the beauty of their dreams",
        "Everything you can imagine is real",
        "Not all those who wander are lost",
        "In three words I can sum up everything I have learned about life it goes on",
        "The purpose of our lives is to be happy",
        "Get busy living or get busy dying",
        "You only live once", "YOLO",
        "Live laugh love",
        "Good vibes only", "Namaste",
        "Om mani padme hum",
        "Sat nam", "Radical acceptance",
        "Live in the moment", "Be here now",
        "Be the change you wish to see in the world",
        "Do or do not there is no try",
        "May the force be with you always",
        "A Jedi uses the force for knowledge and defense never for attack",
        "I am a jedi like my father before me",
        "Do or do not there is no try",
        "Fear leads to anger anger leads to hate hate leads to suffering",
        "I am the one who knocks",
        "A guy walks into a bar",
        "Life is what happens when you're busy making other plans",
        "The world is a book and those who do not travel read only one page",
        "Happiness depends upon ourselves",
        "Turn every trauma into a triumph",
        "Be yourself everyone else is already taken",
        "Two things are infinite the universe and human stupidity",
        "Any sufficiently advanced technology is indistinguishable from magic",
        "The universe is under no obligation to make sense to you",
        "Reality is merely an illusion albeit a very persistent one",
    ]);

    // 5. Pop culture 2009 specifically
    add(&mut known, &[
        "Ice ice baby", "Single ladies put a ring on it",
        "Poker face", "Just dance", "Bad romance",
        "Viva la vida", "Rehab", "Umbrella",
        "Low", "Bleeding love", "I kissed a girl",
        "Poker face", "Love story", "You belong with me",
        "Chasing cars", "Use someone",
        "The Climb", "Breakaway",
        "Since U Been Gone", "Crazy in Love",
        "Irreplaceable", "Confessions part 2",
        "Beautiful", "Against all odds",
        "Tik tok", "Dynamite", "Firework",
        "Call me maybe", "We no speak Americano",
        "Party in the USA", "California dreaming",
        "I gots plenty of nuttin",
        "Right round", "Boom boom pow",
        "Piranha", "Kiss kiss",
        "Down", "Live like we're dying",
        "Runaway love", "Human",
        "The Climb", "Horse with no name",
        "American idiot", "Boulevard of broken dreams",
        "Freak on a leash", "Given up",
        "Numb", "In the end",
        "Crawling", "Papercut",
        "One step closer", "Photograph",
        "Breaking the habit", "Somewhere I belong",
        "What I've done", "Claire de lune",
        "New divide", "Leave out all the rest",
        "Secrets", "What if I",
        "Mad world", "Somebody that I used to know",
        "Get lucky", "Get low",
        "Titanium", "Wake me up",
        "Levels", "Animals", "Clarity",
        "Turn down for what", "Harlem shake",
        "Thrift shop", "Blurred lines",
        "Dark horse", "Problem",
        "Black or white", "She blazed a trail",
        "Shake it off", "Bad blood",
        "Wildest dreams", "Style",
        "Blank space", "Out of the woods",
        "Delicate", "Look what you made me do",
        "This is me trying", "You need to calm down",
        "Lover", "Cardigan", "Willow",
        "Anti hero", "Lavender haze",
        "Karma", "Midnight rain",
        "Snow on the beach", "Vigilante shame",
        "Bejeweled", "Master of hate",
        "Question everything", "Great things never come from comfort zones",
        "Lavender haze im in it",
        "Im a vampire", "I am a ghost",
        "I am a god", "I am a star",
        "I am the night", "I am the storm",
        "I am the fire", "I am the light",
        "I am the darkness", "I am the shadow",
        "I am the wind", "I am the rain",
        "I am the earth", "I am the sky",
        "I am the ocean", "I am the sea",
        "I am the mountain", "I am the river",
        "I am the forest", "I am the tree",
        "I am the flower", "I am the sun",
        "I am the moon", "I am the star",
        "I am the galaxy", "I am the universe",
        "I am everything", "I am nothing",
        "I am alive", "I am free",
        "I am happy", "I am sad",
        "I am strong", "I am weak",
        "I am brave", "I am afraid",
        "I am loved", "I am alone",
        "I am here", "I am there",
        "I am now", "I am then",
        "I am one", "I am many",
        "I am all", "I am none",
    ]);

    // 6. Password patterns (very common brainwallet mistakes)
    let passwords = [
        "password", "123456", "12345678", "123456789", "1234567890",
        "qwerty", "abc123", "monkey", "master", "dragon",
        "letmein", "login", "princess", "football", "shadow",
        "sunshine", "trustno1", "iloveyou", "batman", "access",
        "hello", "charlie", "donald", "password1", "password123",
        "qwerty123", "admin", "admin123", "root", "toor",
        "pass", "test", "guest", "master123", "changeme",
        "welcome", "love", "sex", "money", "god",
        "jesus", "christ", "devil", "demon", "angel",
        "heaven", "hell", "paradise", "freedom", "justice",
        "peace", "war", "death", "life", "power",
        "king", "queen", "prince", "princess", "kingdom",
        "empire", "nation", "world", "earth", "planet",
        "star", "moon", "sun", "galaxy", "cosmos",
        "infinity", "eternity", "forever", "always", "never",
        "hacker", "cracker", "penguin", "matrix", "cypher",
        "oracle", "phoenix", "eagle", "wolf", "tiger",
        "lion", "bear", "dragon", "snake", "spider",
        "black", "white", "red", "blue", "green",
        "gold", "silver", "diamond", "ruby", "emerald",
        "thunder", "lightning", "storm", "fire", "ice",
        "water", "wind", "earth", "nature", "spirit",
        "mystic", "magic", "wizard", "warrior", "ninja",
        "samurai", "knight", "viking", "pirate", "soldier",
        "general", "captain", "major", "colonel", "admiral",
        "president", "governor", "senator", "minister", "chief",
        "doctor", "professor", "teacher", "student", "scholar",
        "artist", "musician", "writer", "poet", "philosopher",
        "scientist", "engineer", "builder", "creator", "inventor",
        "explorer", "adventurer", "traveler", "wanderer", "nomad",
        "survivor", "champion", "winner", "hero", "legend",
        "myth", "fable", "tale", "story", "legend",
        "dream", "vision", "reality", "truth", "lie",
        "hope", "faith", "belief", "trust", "doubt",
        "fear", "courage", "bravery", "strength", "weakness",
        "wisdom", "knowledge", "intelligence", "genius", "brilliant",
        "beautiful", "gorgeous", "perfect", "flawless", "pure",
        "dark", "light", "bright", "shining", "glowing",
        "hidden", "secret", "mystery", "enigma", "riddle",
    ];
    for p in &passwords {
        add(&mut known, &[*p]);
        // Common patterns with numbers
        add_with_years(&mut known, p, &years);
        // With common suffixes
        for s in ["1", "12", "123", "!", "!", "!!", "!!!", "01", "99"] {
            add(&mut known, &[&format!("{}{}", p, s)]);
        }
    }

    // 7. Common phrases & idioms
    add(&mut known, &[
        "at the end of the day", "bottom line", "case in point",
        "cut to the chase", "get to the point", "hit the nail on the head",
        "in a nutshell", "long story short", "making a mountain out of a molehill",
        "once in a blue moon", "piece of cake", "pull yourself together",
        "the bottom line is", "the final word", "the last straw",
        "to make a long story short", "under the weather", "wrap your head around",
        "you had me at hello", "a drop in the bucket", "a matter of time",
        "a pain in the neck", "a piece of the pie", "a taste of your own medicine",
        "actions speak louder than words", "add insult to injury",
        "all roads lead to rome", "all that glitters is not gold",
        "all's fair in love and war", "all's well that ends well",
        "an apple a day keeps the doctor away", "as the crow flies",
        "back to square one", "beat around the bush",
        "better safe than sorry", "bitter sweet",
        "blood is thicker than water", "break a leg",
        "by the skin of your teeth", "can't see the forest for the trees",
        "count your blessings", "curiosity killed the cat",
        "don't put all your eggs in one basket", "don't throw the baby out with the bathwater",
        "every dog has its day", "every cloud has a silver lining",
        "face the music", "familiarity breeds contempt",
        "fight fire with fire", "find the silver lining",
        "for all intents and purposes", "good things come to those who wait",
        "give someone the benefit of the doubt", "grass is always greener",
        "great minds think alike", "haste makes waste",
        "if at first you don't succeed try try again", "ignorance is bliss",
        "it takes two to tango", "kill two birds with one stone",
        "laughing stock", "let sleeping dogs lie",
        "let the cat out of the bag", "look before you leap",
        "make hay while the sun shines", "make it or break it",
        "no news is good news", "no pain no gain",
        "once bitten twice shy", "on the same page",
        "out of the picture", "out of sight out of mind",
        "patience is a virtue", "perfect is the enemy of good",
        "penny wise and pound foolish", "pick your battles",
        "practice makes perfect", "prevention is better than cure",
        "put the cart before the horse", "quality over quantity",
        "read between the lines", "reap what you sow",
        "right place right time right person", "rules of the road",
        "seeing is believing", "set the record straight",
        "slow and steady wins the race", "smile and the world smiles with you",
        "spare the rod spoil the child", "still waters run deep",
        "strike while the iron is hot", "take it with a grain of salt",
        "the best of both worlds", "the bigger they are the harder they fall",
        "the early bird catches the worm", "the pen is mightier than the sword",
        "there's no place like home", "there's no such thing as a free lunch",
        "the road to hell is paved with good intentions",
        "the truth will set you free", "the writing is on the wall",
        "think outside the box", "time flies when you're having fun",
        "to each their own", "tomorrow is another day",
        "two heads are better than one", "turn a blind eye",
        "variety is the spice of life", "when pigs fly",
        "where there's a will there's a way", "words fail me",
        "you can't have it both ways", "you can't judge a book by its cover",
        "you reap what you sow", "your guess is as good as mine",
    ]);

    // 8. Simple sentences (low entropy brainwallets)
    add(&mut known, &[
        "I love you", "I love bitcoin", "bitcoin is the future",
        "bitcoin is money", "digital gold", "peer to peer electronic cash",
        "satoshi nakamoto", "nakamoto", "satoshi",
        "genesis block", "the times 03 jan 2009 chancellor on brink of second bailout for banks",
        "trust no one verify everything",
        "not your keys not your coins",
        "hodl", "to the moon", "diamond hands",
        "buy low sell high", "invest in yourself",
        "money is power", "time is money",
        "money makes the world go round",
        "cash is king", "gold is money",
        "silver is the new gold",
        "crypto is the future", "blockchain is the future",
        "decentralization", "censorship resistance",
        "financial freedom", "be your own bank",
        "own your money", "control your wealth",
        "privacy is a human right", "freedom is a human right",
        "liberty equality fraternity",
        "life liberty property",
        "we hold these truths to be self evident",
        "all men are created equal",
        "government of the people by the people for the people",
        "in we trust", "e pluribus unum",
        "anno domini", "before christ", "after christ",
        "the year is 2009", "bitcoin was born in 2009",
        "2009 bitcoin genesis", "block zero",
        "block one", "the first block",
        "halving", "21 million", "twenty one million",
        "limited supply", "scarce resource",
        "deflationary", "sound money", "hard money",
        "store of value", "digital store of value",
        "open source", "free software", "free and open source software",
        "code is law", "trust code not people",
        "math is trust", "proof of work",
        "hashing is the answer", "sha256",
        "double spend", "byzantine generals problem",
        "consensus mechanism", "distributed ledger",
        "immutable record", "transparent ledger",
        "pseudonymous", "anonymous transactions",
        "untraceable", "uncensorable",
        "permissionless", "trustless",
        "non custodial", "self custody",
        "cold storage", "hot wallet",
        "paper wallet", "hardware wallet",
        "seed phrase", "recovery phrase",
        "mnemonic code", "bip39",
        "derivation path", "master key",
        "public key", "private key",
        "wallet address", "bitcoin address",
        "transaction fee", "mining reward",
        "block reward", "subsidy",
        "merkle tree", "merkle root",
        "nonce", "difficulty",
        "hashrate", "network hashrate",
        "lightning network", "layer two",
        "segwit", "taproot",
        "ordinals", "inscriptions",
        "runes", "bitcoin os",
    ]);

    // 9. Famous numbers & mathematical expressions as phrases
    add(&mut known, &[
        "three point one four one five nine",
        "two point seven one eight two eight",
        "one point six one eight zero three",
        "one point four one four two one",
        "one point seven three two zero five",
        "six point two eight three one eight",
        "zero point five seven seven two one",
        "twenty one million", "21000000",
        "twenty one", "forty two",
        "the answer to life the universe and everything",
        "forty two is the answer",
        "42", "42 is the answer",
        "one two three four five six seven eight nine ten",
        "zero one two three four five six seven eight nine",
        "ten thousand", "one million", "one billion", "one trillion",
        "infinite loop", "recursive function",
        "fibonacci sequence", "golden ratio",
        "prime number", "mersenne prime",
        "perfect number", "triangular number",
        "factorial", "combinatorics",
        "probability theory", "statistics",
        "normal distribution", "bell curve",
        "standard deviation", "variance",
        "mean median mode",
        "arithmetic geometric", "algebra calculus",
        "differential equation", "integral calculus",
        "linear algebra", "matrix multiplication",
        "eigenvalue", "eigenvector",
        "fourier transform", "laplace transform",
        "taylor series", "maclaurin series",
        "euler formula", "euler identity",
        "pythagorean theorem", "fermats last theorem",
        "riemann hypothesis", "goldbach conjecture",
        "twin prime conjecture", "collatz conjecture",
        "p equals np", "halting problem",
        "godel incompleteness", "church turing thesis",
        "lambda calculus", "boolean algebra",
        "set theory", "graph theory",
        "game theory", "information theory",
        "entropy", "shannon entropy",
        "compression", "encryption",
        "rsa", "elliptic curve",
        "secp256k1", "nist p256",
        "aes256", "blake2", "keccak",
        "whirlpool", "tiger", "snefru",
        "md5", "sha1", "sha256", "sha512",
        "hmac", "pbkdf2", "bcrypt", "scrypt",
        "argon2", "chacha20", "salsa20",
        "poly1305", "ed25519", "curve25519",
    ]);

    // 10. Keyboard patterns & walks
    let kb_patterns = [
        "qwertyuiop", "asdfghjkl", "zxcvbnm",
        "qwertyuiopasdfghjklzxcvbnm",
        "1234567890", "0987654321",
        "qazwsx", "edcrfv", "tgbyhn", "yhnujm", "ikm",
        "qwerty", "asdf", "zxcv",
        "qweasd", "asdzxc", "zxcasd",
        "poiuytrewq", "lkjhgfdsa", "mnbvcxz",
        "mnbvcxzlkjhgfdsapoiuytrewq",
        "1qaz2wsx3edc4rfv5tgb6yhn7ujm8ik9ol0p",
        "zaq1xsw2cde3vfr4tgb5yhn6ujm7ik8ol9p0",
        "abcdefghijklmnopqrstuvwxyz",
        "zyxwvutsrqponmlkjihgfedcba",
        "etaoinshrdlu", "etaoinsdhrlue",
        "the quick brown fox jumps over the lazy dog",
        "pack my box with five dozen liquor jugs",
        "how vexingly quick daft zebras jump",
        "the five boxing wizards jump quickly",
        "sphinx of black quartz judge my vow",
    ];
    for p in &kb_patterns {
        add(&mut known, &[*p]);
    }

    // 11. Color + noun combinations
    let colors = ["red", "blue", "green", "yellow", "black", "white", "orange", "purple", "pink", "brown", "gray", "grey", "gold", "silver", "copper", "bronze", "ivory", "azure", "crimson", "scarlet", "violet", "indigo", "magenta", "cyan", "turquoise"];
    let nouns = ["dragon", "phoenix", "eagle", "wolf", "tiger", "lion", "bear", "snake", "spider", "horse", "fox", "hawk", "falcon", "panther", "shark", "whale", "dolphin", "raven", "crow", "owl", "eagle", "bear", "wolf", "fox", "cat", "dog", "bird", "fish", "star", "moon", "sun", "fire", "ice", "water", "earth", "wind", "storm", "thunder", "lightning", "mountain", "river", "ocean", "forest", "desert", "island", "castle", "tower", "bridge", "road", "path", "gate", "key", "lock", "sword", "shield", "crown", "ring", "gem", "crystal", "diamond", "ruby", "emerald", "sapphire", "pearl"];
    for c in &colors {
        for n in &nouns {
            add(&mut known, &[&format!("{} {}", c, n), &format!("{}{}", c, n)]);
        }
    }

    // 12. Animal + adjective combinations
    let animals = ["dragon", "phoenix", "eagle", "wolf", "tiger", "lion", "bear", "snake", "spider", "horse", "fox", "hawk", "falcon", "panther", "shark", "whale", "dolphin", "raven", "crow", "owl"];
    let adjectives = ["dark", "light", "black", "white", "red", "blue", "green", "golden", "silver", "fire", "ice", "storm", "thunder", "shadow", "blood", "steel", "iron", "stone", "crystal", "mystic", "ancient", "eternal", "infinite", "cosmic", "sacred", "holy", "divine", "supreme", "ultimate", "legendary", "mythic", "epic", "mighty", "powerful", "fierce", "brave", "noble", "royal", "imperial", "grand"];
    for a in &animals {
        for adj in &adjectives {
            add(&mut known, &[&format!("{} {}", adj, a), &format!("{}{}", adj, a)]);
        }
    }

    // 13. Famous URLs & web references
    add(&mut known, &[
        "google.com", "facebook.com", "twitter.com", "youtube.com",
        "amazon.com", "wikipedia.org", "reddit.com", "yahoo.com",
        "baidu.com", "yandex.ru", "bing.com", "ask.com",
        "bitcointalk.org", "bitcoin.org",
        "nakamotoinstitute.org",
        "satoshi.nakamoto", "satoshi nakamoto",
        "bitcoin whitepaper", "bitcoin pdf",
        "peer to peer electronic cash system",
        "bitcoin genesis", "bitcoin block 0",
        "blockchain.info", "blockchain.com",
        "blockexplorer.com", "blockchain explorer",
        "bitaddress.org", "bitaddress",
        "electrum wallet", "multibit", "armory wallet",
        "coinbase", "bitstamp", "bitfinex",
        "mt gox", "mtgox",
        "silk road", "silkroad",
        "tor", "darknet", "deepweb",
        "i2p", "freenet", "gnunet",
        "zerocoin", "zerocash", "zcoin",
        "namecoin", "peercoin", "litecoin",
        "dogecoin", "dash", "monero",
        "ethereum", "ripple", "stellar",
        "neo", "cardano", "tron",
        "polkadot", "cosmos", "avalanche",
        "solana", "near", "algorand",
        "filecoin", "ipfs", "sia",
        "storage", "bandwidth", "compute",
        "decentralized", "distributed", "peer to peer",
        "p2p", "dht", "kademlia",
        "gossip protocol", "consensus",
        "proof of stake", "proof of authority",
        "proof of history", "proof of space",
        "proof of capacity", "proof of burn",
        "proof of work", "mining",
        "hashing", "mining pool",
        "solo mining", "cloud mining",
        "asic", "gpu mining", "cpu mining",
        "fpga", "mining rig",
        "mining farm", "mining operation",
    ]);

    // 14. Number words (English)
    let num_words = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
        "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen",
        "sixteen", "seventeen", "eighteen", "nineteen", "twenty",
        "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        "hundred", "thousand", "million", "billion", "trillion",
        "first", "second", "third", "fourth", "fifth",
        "half", "quarter", "double", "triple",
    ];
    for n in &num_words {
        add(&mut known, &[*n]);
        for n2 in &num_words {
            add(&mut known, &[&format!("{} {}", n, n2)]);
        }
    }

    // 15. Famous movie quotes (French — secondary language)
    add(&mut known, &[
        "Je suis Spartacus", "Après la bataille",
        "La vie est belle", "Amore mio",
        "Je t'aime moi non plus",
        "C'est la vie", "Joie de vivre",
        "L'amour est mort", "Je ne regrette rien",
        "La Marseillaise", "Aux armes citoyens",
        "Liberté égalité fraternité",
        "Vive la France", "Vive la liberté",
        "Aller plus loin", "Fais ce que dois",
        "L'avenir c'est maintenant",
        "Nous sommes tous des survivants",
        "Le monde appartient à ceux qui se lèvent tôt",
        "Petit à petit l'oiseau fait son nid",
        "Qui vivra verra", "Tout est bien qui finit bien",
        "L'amour de sa vie", "Le plus beau du monde",
        "La force tranquille", "Le pouvoir suprême",
        "La connaissance est le pouvoir",
        "La vérité vous rendra libre",
        "L'ignorance est un fléau",
        "Le savoir c'est la lumière",
        "La sagesse vient avec l'âge",
        "Le temps c'est de l'argent",
        "L'argent ne fait pas le bonheur",
        "L'habit ne fait pas le moine",
        "Chaque birdie son jour",
        "Petit poisson deviendra grand",
        "Qui sème le vent récolte la tempête",
        "Tant qu'il y a de l'huile il y a de la lumière",
        "L'union fait la force",
        "La patience est une vertu",
        "Le courage est la première des vertus",
        "La liberté ou la mort",
        "Veni vidi vici",
        "Carpe diem", "Memento mori",
        "Per aspera ad astra",
        "Cogito ergo sum",
        "Je pense donc je suis",
        "L'homme est né libre et partout il est dans les fers",
        "La nature horreur de la vacuité",
        "Dieu est mort",
        "L'homme est condamné à être libre",
        "L'enfer c'est les autres",
        "L'imagination est plus importante que le savoir",
        "La créativité c'est l'intelligence qui s'amuse",
        "La science sans conscience n'est que ruine de l'âme",
        "Le but du jeu c'est de changer le jeu",
        "Le seul moyen de faire du bon travail est d'aimer ce que l'on fait",
        "Soyez fous restez fous",
        "Rêvez grand", "Osez rêver",
        "Croyez en vous", "Faites confiance au processus",
        "Tout est possible", "Rien n'est impossible",
        "Le meilleur moyen de prédire l'avenir est de le créer",
        "Chaque fin est un nouveau commencement",
        "La vie est un voyage pas une destination",
        "Le bonheur ne est pas une station d'arrivée c'est un mode de vie",
    ]);

    // 16. Date patterns (birth dates, significant dates)
    let months = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
    let days = [1u16, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
    let date_years = [1970u16, 1971, 1972, 1973, 1974, 1975, 1976, 1977, 1978, 1979,
                      1980, 1981, 1982, 1983, 1984, 1985, 1986, 1987, 1988, 1989,
                      1990, 1991, 1992, 1993, 1994, 1995, 1996, 1997, 1998, 1999, 2000];
    // Numeric date formats: MMDDYYYY, DDMMYYYY, YYYYMMDD
    for &m in &[1u16, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] {
        for &d in &days {
            for &y in &date_years {
                let p1 = format!("{:02}{:02}{}", m, d, y); // MMDDYYYY
                let p2 = format!("{:02}{:02}{}", d, m, y); // DDMMYYYY
                let p3 = format!("{}{:02}{:02}", y, m, d); // YYYYMMDD
                let p4 = format!("{}-{:02}-{:02}", y, m, d); // YYYY-MM-DD
                let p5 = format!("{:02}/{:02}/{}", m, d, y); // MM/DD/YY
                for p in [p1, p2, p3, p4, p5] {
                    known.insert(sha256(p.as_bytes()));
                    known.insert(dsha256(p.as_bytes()));
                }
            }
        }
    }
    println!("  Date patterns: {} keys", known.len());

    // 17. First name + last name combinations (common names)
    let first_names = ["john", "james", "robert", "michael", "william", "david", "richard", "joseph", "thomas", "charles",
                       "christopher", "daniel", "matthew", "anthony", "mark", "donald", "steven", "paul", "andrew", "joshua",
                       "jennifer", "linda", "mary", "patricia", "elizabeth", "barbara", "susan", "jessica", "sarah", "karen",
                       "alice", "bob", "mike", "sue", "tom", "jake", "sam", "alex", "max", "leo",
                       "satoshi", "nakamoto", "hal", "finney", "nicolas", "adam", "ian", "gavin", "martti", "amir"];
    let last_names = ["smith", "johnson", "williams", "brown", "jones", "miller", "davis", "garcia", "rodriguez", "wilson",
                      "martinez", "anderson", "taylor", "thomas", "hernandez", "moore", "martin", "jackson", "thompson", "white",
                      "harris", "sanchez", "clark", "ramirez", "lewis", "lee", "walker", "hall", "allen", "young",
                      "bitcoin", "crypto", "blockchain", "money", "gold", "silver", "diamond", "star", "moon", "sun",
                      "nakamoto", "finney", "becker", "katon", "greene", "clarke", "jones", "smith", "brown", "williams"];
    for fn_ in &first_names {
        for ln in &last_names {
            add(&mut known, &[&format!("{} {}", fn_, ln), &format!("{}{}", fn_, ln)]);
            add_with_years(&mut known, &format!("{}{}", fn_, ln), &years);
        }
    }
    println!("  Name combinations: {} keys", known.len());

    // 18. Sports teams + years (very popular brainwallet theme)
    let teams = [
        "lakers", "celtics", "warriors", "bulls", "chicago bulls", "los angeles lakers",
        "golden state warriors", "boston celtics", "miami heat", "houston rockets",
        "yankees", "reds", "cubs", "sox", "mets", "giants", "astros", "dodgers",
        "packers", "cowboys", "patriots", "chiefs", "steelers", "49ers", "eagles", "bears",
        "manchester united", "barcelona", "real madrid", "arsenal", "chelsea", "liverpool",
        "juventus", "milan", "inter milan", "bayern munich", "psg", "marseille",
        "red bulls", "new york red bulls", "la galaxy", "chivas",
    ];
    for t in &teams {
        add(&mut known, &[*t]);
        add_with_years(&mut known, t, &years);
        for n in 1u16..=99 {
            add(&mut known, &[&format!("{} {}", t, n), &format!("{}{}", t, n)]);
        }
    }
    println!("  Sports teams: {} keys", known.len());

    // 19. Verb + noun brainwallet patterns
    let verbs = ["i want to", "i need to", "i love to", "i hate to", "i will", "i can", "i must", "i should", "help me", "save me", "find me", "free me", "kill me", "love me", "hate me", "trust me", "follow me", "watch me", "hear me", "see me"];
    let verb_nouns = ["fly", "swim", "run", "jump", "dance", "sing", "fight", "win", "lose", "die", "live", "breathe", "think", "dream", "sleep", "wake", "eat", "drink", "play", "work", "build", "create", "destroy", "conquer", "rule", "serve", "pray", "believe", "hope", "wonder", "search", "seek", "hunt", "chase", "escape", "survive", "endure", "suffer", "rejoice", "celebrate"];
    for v in &verbs {
        for vn in &verb_nouns {
            add(&mut known, &[&format!("{} {}", v, vn)]);
        }
    }
    println!("  Verb+noun patterns: {} keys", known.len());

    // 20. Common username patterns
    let prefixes = ["x", "the", "real", "its", "im", "dr", "mr", "ms", "prof", "captain"];
    let user_bases = ["player", "gamer", "ninja", "warrior", "killer", "shadow", "ghost", "stealth", "coyote", "viper", "cobra", "hawk", "eagle", "wolf", "bear", "lion", "tiger", "panther", "dragon", "phoenix", "angel", "demon", "devil", "god", "master", "lord", "king", "queen", "prince", "dude"];
    for p in &prefixes {
        for b in &user_bases {
            let u = format!("{}{}", p, b);
            add(&mut known, &[&u]);
            add_with_years(&mut known, &u, &years);
            for n in 1u16..=99 {
                add(&mut known, &[&format!("{}{}", u, n)]);
            }
        }
    }
    println!("  Username patterns: {} keys", known.len());

    // 21. Card game patterns (poker hands, card combinations)
    let suits = ["hearts", "diamonds", "clubs", "spades", "heart", "diamond", "club", "spade"];
    let ranks = ["ace", "king", "queen", "jack", "ten", "nine", "eight", "seven", "six", "five", "four", "three", "two"];
    for s in &suits {
        for r in &ranks {
            add(&mut known, &[&format!("{} of {}", r, s), &format!("{}{}", r, s)]);
        }
    }
    add(&mut known, &[
        "royal flush", "straight flush", "four of a kind", "full house",
        "flush", "straight", "three of a kind", "two pair", "one pair", "high card",
        "poker face", "blackjack", "21", "natural 21",
        "all in", "raise the bet", "call your bluff",
        "ace of spades", "ace of hearts", "ace of diamonds", "ace of clubs",
        "king of hearts", "king of spades",
        "joker", "wild card", "deck of cards",
        "shuffle", "deal", "fold", "check", "bet", "raise", "call",
    ]);
    println!("  Card patterns: {} keys", known.len());

    // 22. Dice patterns
    for d1 in 1..=6 {
        for d2 in 1..=6 {
            for d3 in 1..=6 {
                let p = format!("{}{}{}", d1, d2, d3);
                known.insert(sha256(p.as_bytes()));
                known.insert(dsha256(p.as_bytes()));
                // Also as words
                let words = format!("{} {} {}", d1, d2, d3);
                known.insert(sha256(words.as_bytes()));
                known.insert(dsha256(words.as_bytes()));
            }
        }
    }
    println!("  Dice patterns: {} keys", known.len());

    // 23. Coin flip patterns (H/T sequences)
    for i in 0u32..65536 { // 16-bit = up to 16 flips
        let mut s = String::with_capacity(16);
        let mut n = i as u16;
        for bit in 0..16 {
            s.push(if n & 1 == 1 { 'H' } else { 'T' });
            n >>= 1;
            if n == 0 && bit < 15 { break; }
        }
        known.insert(sha256(s.as_bytes()));
        known.insert(dsha256(s.as_bytes()));
    }
    println!("  Coin flip patterns: {} keys", known.len());

    // 24. Famous people + years
    let famous = [
        "albert einstein", "isaac newton", "charles darwin", "nikola tesla",
        "leonardo da vinci", "william shakespeare", "aristotle", "plato",
        "socrates", "confucius", "buddha", "lao tzu",
        "winston churchill", "abraham lincoln", "martin luther king",
        "mahatma gandhi", "nelson mandela", "john f kennedy",
        "elvis presley", "john lennon", "bob dylan", "mickey mouse",
        "winnie the pooh", "bambi", "mario", "sonic", "link", "zelda",
        "gandalf", "frodo", "aragorn", "legolas", "gimli",
        "dumbledore", "harry potter", "hermione", "ron weasley",
        "spiderman", "superman", "batman", "iron man", "captain america",
        "wonder woman", "the hulk", "thor", "black widow", "hawkeye",
        "jack sparrow", "titanic", "rosa", "leonardo dicaprio",
    ];
    for f in &famous {
        add(&mut known, &[*f]);
        add_with_years(&mut known, f, &years);
    }
    println!("  Famous people: {} keys", known.len());

    // 25. Phone number patterns (common formats)
    for area in [212u16, 213, 310, 311, 312, 313, 404, 407, 408, 415, 503, 504, 505, 617, 619, 702, 703, 704, 707, 800, 801, 808, 818, 900, 901, 909, 914, 916, 917, 919] {
        for exch in 555u16..=559 {
            for line in 0u16..=9999 {
                let p = format!("{}{}{:04}", area, exch, line);
                known.insert(sha256(p.as_bytes()));
                known.insert(dsha256(p.as_bytes()));
            }
        }
    }
    println!("  Phone patterns: {} keys", known.len());

    // Write output
    let outfile = "data/pop-culture-2009-keys.txt";
    let mut f = std::fs::File::create(outfile).expect("Cannot create output file");
    let mut count = 0;
    for key in &known {
        let hex: String = key.iter().map(|b| format!("{:02x}", b)).collect();
        writeln!(f, "{}", hex).expect("Write failed");
        count += 1;
    }
    println!("Generated {} unique keys -> {}", count, outfile);
    let size = std::fs::metadata(outfile).unwrap().len();
    println!("File size: {:.1} MB", size as f64 / 1_000_000.0);
}
