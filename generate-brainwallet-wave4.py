#!/usr/bin/env python3
"""Wave 4: Music, movies, leet speak, 2-word pairs, MD5-as-key, foreign quotes."""
from pathlib import Path
from itertools import product
import hashlib

OUTPUT = Path(r"Y:\btcsolver\brainwallet-wave4-corpus.txt")
lines: set[str] = set()

def add(*items):
    for x in items:
        if x and str(x).strip():
            lines.add(str(x).strip())

print("1. Full / extended song lyrics & titles...")
songs = [
    # Classic rock / pop full lines
    "imagine theres no heaven","imagine all the people","living for today",
    "you may say im a dreamer","but im not the only one",
    "i hope someday youll join us","and the world will be as one",
    "let it be","whisper words of wisdom","let it be",
    "when i find myself in times of trouble","mother mary comes to me",
    "speaking words of wisdom","let it be",
    "is this the real life","is this just fantasy",
    "caught in a landslide","no escape from reality",
    "open your eyes","look up to the skies and see",
    "im just a poor boy","i need no sympathy",
    "because im easy come easy go","little high little low",
    "anyway the wind blows","doesnt really matter to me",
    "mama just killed a man","put a gun against his head",
    "pulled my trigger now hes dead","mama life had just begun",
    "but now ive gone and thrown it all away",
    "mama ooh","didnt mean to make you cry",
    "if im not back again this time tomorrow",
    "carry on carry on","as if nothing really matters",
    "too late my time has come","sends shivers down my spine",
    "bodys aching all the time","goodbye everybody ive got to go",
    "gotta leave you all behind and face the truth",
    "mama ooh i dont want to die","i sometimes wish id never been born at all",
    "i see a little silhouetto of a man","scaramouche scaramouche will you do the fandango",
    "thunderbolt and lightning","very very frightening me",
    "galileo galileo","galileo figaro","magnifico",
    "im just a poor boy nobody loves me",
    "hes just a poor boy from a poor family",
    "spare him his life from this monstrosity",
    "easy come easy go will you let me go",
    "bismillah no we will not let you go",
    "let him go","will not let you go","let me go",
    "no no no no no no no","oh mama mia mama mia",
    "mama mia let me go","beelzebub has a devil put aside for me",
    "for me for me for me",
    "so you think you can stone me and spit in my eye",
    "so you think you can love me and leave me to die",
    "oh baby cant do this to me baby",
    "just gotta get out just gotta get right outta here",
    "ooh yeah ooh yeah","nothing really matters",
    "anyone can see","nothing really matters",
    "nothing really matters to me",
    "anyway the wind blows",
    "theres a lady whos sure","all that glitters is gold",
    "and shes buying a stairway to heaven",
    "when she gets there she knows","if the stores are all closed",
    "with a word she can get what she came for",
    "ooh ooh and shes buying a stairway to heaven",
    "theres a sign on the wall","but she wants to be sure",
    "cause you know sometimes words have two meanings",
    "in a tree by the brook","theres a songbird who sings",
    "sometimes all of our thoughts are misgiven",
    "ooh it makes me wonder","ooh it makes me wonder",
    "theres a feeling i get","when i look to the west",
    "and my spirit is crying for leaving",
    "in my thoughts i have seen","rings of smoke through the trees",
    "and the voices of those who stand looking",
    "ooh it makes me wonder","ooh it really makes me wonder",
    "and its whispered that soon","if we all call the tune",
    "then the piper will lead us to reason",
    "and a new day will dawn","for those who stand long",
    "and the forests will echo with laughter",
    "if theres a bustle in your hedgerow","dont be alarmed now",
    "its just a spring clean for the may queen",
    "yes there are two paths you can go by","but in the long run",
    "theres still time to change the road youre on",
    "and it makes me wonder",
    "your head is humming and it wont go","in case you dont know",
    "the pipers calling you to join him",
    "dear lady can you hear the wind blow","and did you know",
    "your stairway lies on the whispering wind",
    "and as we wind on down the road","our shadows taller than our soul",
    "there walks a lady we all know","who shines white light and wants to show",
    "how everything still turns to gold","and if you listen very hard",
    "the tune will come to you at last","when all are one and one is all",
    "to be a rock and not to roll",
    "and shes buying a stairway to heaven",
    "on a dark desert highway","cool wind in my hair",
    "warm smell of colitas","rising up through the air",
    "up ahead in the distance","i saw a shimmering light",
    "my head grew heavy and my sight grew dim","i had to stop for the night",
    "there she stood in the doorway","i heard the mission bell",
    "and i was thinking to myself","this could be heaven or this could be hell",
    "then she lit up a candle","and she showed me the way",
    "there were voices down the corridor","i thought i heard them say",
    "welcome to the hotel california","such a lovely place such a lovely place",
    "plenty of room at the hotel california","any time of year any time of year",
    "you can find it here",
    "her mind is tiffany twisted","she got the mercedes bends",
    "she got a lot of pretty pretty boys","that she calls friends",
    "how they dance in the courtyard","sweet summer sweat",
    "some dance to remember","some dance to forget",
    "so i called up the captain","please bring me my wine",
    "he said we havent had that spirit here","since 1969",
    "and still those voices are calling from far away",
    "wake you up in the middle of the night","just to hear them say",
    "welcome to the hotel california","such a lovely place such a lovely place",
    "they livin it up at the hotel california","what a nice surprise what a nice surprise",
    "bring your alibis",
    "mirrors on the ceiling","the pink champagne on ice",
    "and she said we are all just prisoners here","of our own device",
    "and in the masters chambers","they gathered for the feast",
    "they stab it with their steely knives","but they just cant kill the beast",
    "last thing i remember","i was running for the door",
    "i had to find the passage back","to the place i was before",
    "relax said the night man","we are programmed to receive",
    "you can check out any time you like","but you can never leave",
    "hello darkness my old friend","ive come to talk with you again",
    "because a vision softly creeping","left its seeds while i was sleeping",
    "and the vision that was planted in my brain","still remains",
    "within the sound of silence",
    "in restless dreams i walked alone","narrow streets of cobblestone",
    "neath the halo of a street lamp","i turned my collar to the cold and damp",
    "when my eyes were stabbed by the flash of a neon light",
    "that split the night","and touched the sound of silence",
    "and in the naked light i saw","ten thousand people maybe more",
    "people talking without speaking","people hearing without listening",
    "people writing songs that voices never share","and no one dared",
    "disturb the sound of silence",
    "fools said i you do not know","silence like a cancer grows",
    "hear my words that i might teach you","take my arms that i might reach you",
    "but my words like silent raindrops fell","and echoed in the wells of silence",
    "and the people bowed and prayed","to the neon god they made",
    "and the sign flashed out its warning","in the words that it was forming",
    "and the sign said the words of the prophets","are written on the subway walls",
    "and tenement halls","and whispered in the sounds of silence",
    "yesterday","all my troubles seemed so far away",
    "now it looks as though theyre here to stay","oh i believe in yesterday",
    "suddenly","im not half the man i used to be",
    "theres a shadow hanging over me","oh yesterday came suddenly",
    "why she had to go i dont know","she wouldnt say",
    "i said something wrong","now i long for yesterday",
    "yesterday","love was such an easy game to play",
    "now i need a place to hide away","oh i believe in yesterday",
    "hey jude","dont make it bad","take a sad song and make it better",
    "remember to let her into your heart","then you can start to make it better",
    "hey jude","dont be afraid","you were made to go out and get her",
    "the minute you let her under your skin","then you begin to make it better",
    "and anytime you feel the pain","hey jude refrain",
    "dont carry the world upon your shoulders",
    "for well you know that its a fool","who plays it cool",
    "by making his world a little colder",
    "hey jude","dont let me down","you have found her now go and get her",
    "remember to let her into your heart","then you can start to make it better",
    "so let it out and let it in","hey jude begin",
    "youre waiting for someone to perform with",
    "and dont you know that its just you","hey jude youll do",
    "the movement you need is on your shoulder",
    "hey jude","dont make it bad","take a sad song and make it better",
    "remember to let her under your skin","then youll begin to make it better",
    "better better better better better better","na na na na na na na",
    "here comes the sun","here comes the sun","and i say its all right",
    "little darling","its been a long cold lonely winter",
    "little darling","it feels like years since its been here",
    "here comes the sun","here comes the sun","and i say its all right",
    "little darling","the smiles returning to the faces",
    "little darling","it seems like years since its been here",
    "here comes the sun","here comes the sun","and i say its all right",
    "sun sun sun here it comes",
    "little darling","i feel that ice is slowly melting",
    "little darling","it seems like years since its been clear",
    "here comes the sun","here comes the sun","and i say its all right",
    "here comes the sun","here comes the sun","its all right","its all right",
    "come together right now over me",
    "here come old flat top","he come grooving up slowly",
    "he got joo joo eyeball","he one holy roller",
    "he got hair down to his knee","got to be a joker he just do what he please",
    "he wear no shoeshine","he got toe jam football",
    "he got monkey finger","he shoot coca cola",
    "he say i know you you know me","one thing i can tell you is you got to be free",
    "come together right now over me",
    "he bag production","he got walrus gumboot",
    "he got ono sideboard","he one spinal cracker",
    "he got feet down below his knee","hold you in his armchair you can feel his disease",
    "come together right now over me",
    "he roller coaster","he got early warning",
    "he got muddy water","he one mojo filter",
    "he say one and one and one is three","got to be good looking cause hes so hard to see",
    "come together right now over me",
    "all you need is love","all you need is love",
    "all you need is love love","love is all you need",
    "theres nothing you can do that cant be done",
    "nothing you can sing that cant be sung",
    "nothing you can say but you can learn how to play the game",
    "its easy",
    "nothing you can make that cant be made",
    "no one you can save that cant be saved",
    "nothing you can do but you can learn how to be you in time",
    "its easy",
    "all you need is love","all you need is love",
    "all you need is love love","love is all you need",
    "all you need is love","all together now",
    "all you need is love","everybody",
    "all you need is love love","love is all you need",
    "love is all you need","love is all you need",
    "smells like teen spirit","hello hello hello how low",
    "with the lights out","its less dangerous",
    "here we are now","entertain us",
    "i feel stupid","and contagious",
    "here we are now","entertain us",
    "a mulatto","an albino","a mosquito","my libido","yeah",
    "load up on guns","bring your friends","its fun to lose and to pretend",
    "shes over bored and self assured","oh no i know a dirty word",
    "hello hello hello how low",
    "im worse at what i do best","and for this gift i feel blessed",
    "our little group has always been","and always will until the end",
    "and i forget just why i taste","oh yeah i guess it makes me smile",
    "i found it hard its hard to find","oh well whatever nevermind",
    "enter sandman","say your prayers little one",
    "dont forget my son","to include everyone",
    "i tuck you in","warm within","keep you free from sin",
    "till the sandman he comes",
    "sleep with one eye open","gripping your pillow tight",
    "exit light","enter night","take my hand","were off to never never land",
    "something something something","something something something",
    "hush little baby dont say a word","and never mind that noise you heard",
    "its just the beasts under your bed","in your closet in your head",
    "exit light","enter night","grain of sand",
    "exit light","enter night","take my hand","were off to never never land",
    "now i lay me down to sleep","pray the lord my soul to keep",
    "if i die before i wake","pray the lord my soul to take",
    "hush little baby dont say a word","and never mind that noise you heard",
    "its just the beasts under your bed","in your closet in your head",
    "exit light","enter night","take my hand","were off to never never land",
    "nothing else matters","so close no matter how far",
    "couldnt be much more from the heart","forever trusting who we are",
    "and nothing else matters",
    "never opened myself this way","life is ours we live it our way",
    "all these words i dont just say","and nothing else matters",
    "trust i seek and i find in you","every day for us something new",
    "open mind for a different view","and nothing else matters",
    "never cared for what they do","never cared for what they know",
    "but i know",
    "so close no matter how far","couldnt be much more from the heart",
    "forever trusting who we are","and nothing else matters",
    "never cared for what they say","never cared for games they play",
    "never cared for what they do","never cared for what they know",
    "and i know",
    "so close no matter how far","couldnt be much more from the heart",
    "forever trusting who we are","no nothing else matters",
    "back in black","i hit the sack","ive been too long im glad to be back",
    "yes im let loose","from the noose","thats kept me hanging about",
    "ive been looking at the sky","cause its gettin me high","forget the hearse cause i never die",
    "i got nine lives","cats eyes","abusin every one of them and running wild",
    "cause im back","yes im back well im back","yes im back",
    "well im back back","well im back in black","yes im back in black",
    "back in the back","of a cadillac","number one with a bullet im a power pack",
    "yes im in a bang","with a gang","theyve got to catch me if they want me to hang",
    "cause im back on the track","and im beatin the flack","nobodys gonna get me on another rap",
    "so look at me now","im just makin my play","dont try to push your luck just get out of my way",
    "cause im back","yes im back","well im back","yes im back",
    "well im back back","well im back in black","yes im back in black",
    "sweet child o mine","shes got a smile that it seems to me",
    "reminds me of childhood memories","where everything was as fresh as the bright blue sky",
    "now and then when i see her face","she takes me away to that special place",
    "and if i stared too long","id probably break down and cry",
    "oh sweet child o mine","oh oh oh sweet love of mine",
    "shes got eyes of the bluest skies","as if they thought of rain",
    "i hate to look into those eyes","and see an ounce of pain",
    "her hair reminds me of a warm safe place","where as a child id hide",
    "and pray for the thunder and the rain","to quietly pass me by",
    "oh sweet child o mine","oh oh oh sweet love of mine",
    "oh sweet child o mine","oh oh oh sweet love of mine",
    "oh sweet child o mine","oh oh oh sweet love of mine",
    "where do we go","where do we go now","where do we go",
    "where do we go","where do we go now","where do we go",
    "where do we go","sweet child","where do we go now",
    "wonderwall","today is gonna be the day","that theyre gonna throw it back to you",
    "by now you shouldve somehow","realized what you gotta do",
    "i dont believe that anybody","feels the way i do about you now",
    "backbeat the word is on the street","that the fire in your heart is out",
    "im sure youve heard it all before","but you never really had a doubt",
    "i dont believe that anybody","feels the way i do about you now",
    "and all the roads we have to walk are winding","and all the lights that lead us there are blinding",
    "there are many things that i would like to say to you","but i dont know how",
    "because maybe","youre gonna be the one that saves me",
    "and after all","youre my wonderwall",
    "today was gonna be the day","but theyll never throw it back to you",
    "by now you shouldve somehow","realized what youre not to do",
    "i dont believe that anybody","feels the way i do about you now",
    "and all the roads that lead you there were winding","and all the lights that light the way are blinding",
    "there are many things that i would like to say to you","but i dont know how",
    "i said maybe","youre gonna be the one that saves me",
    "and after all","youre my wonderwall",
    "i said maybe","youre gonna be the one that saves me",
    "and after all","youre my wonderwall",
    "i said maybe","youre gonna be the one that saves me",
    "youre gonna be the one that saves me","youre gonna be the one that saves me",
    "bohemian rhapsody","stairway to heaven","hotel california",
    "the sound of silence","hey jude","here comes the sun",
    "come together","all you need is love","smells like teen spirit",
    "enter sandman","nothing else matters","back in black",
    "sweet child o mine","wonderwall","imagine","let it be",
    "yesterday","free bird","we will rock you","we are the champions",
    "another brick in the wall","comfortably numb","money","like a rolling stone",
    "blowin in the wind","what a wonderful world","hallelujah",
    "born to run","dancing in the dark","thriller","billie jean","beat it",
    "rolling in the deep","smoke on the water","paradise city",
    "livin on a prayer","dont stop believin","every breath you take",
    "in the air tonight","money for nothing","paint it black",
    "gimme shelter","fortunate son","bad moon rising",
    "light my fire","break on through","born to be wild",
    "sweet home alabama","all along the watchtower","purple haze",
    "hey joe","the wind cries mary","little wing","voodoo child",
]
for s in songs:
    add(s, s.upper(), f"{s}!", f"{s}123")
print(f"   songs: {len(lines)}")

print("2. Extended movie dialogues...")
movies = [
    "may the force be with you","ill be back","heres looking at you kid",
    "you talking to me","why so serious","i am your father",
    "to infinity and beyond","just keep swimming","hasta la vista baby",
    "you shall not pass","i see dead people","life is like a box of chocolates",
    "my precious","i am iron man","say hello to my little friend",
    "you cant handle the truth","go ahead make my day","et phone home",
    "im the king of the world","show me the money","with great power comes great responsibility",
    "i am the one who knocks","winter is coming","you know nothing jon snow",
    "a lannister always pays his debts","the cake is a lie",
    "all your base are belong to us","its dangerous to go alone take this",
    "this is the way","i am inevitable","there is no spoon",
    "follow the white rabbit","red pill blue pill","wake up neo",
    "the matrix has you","knock knock neo","i know kung fu",
    "houston we have a problem","bond james bond","shaken not stirred",
    "the name is bond james bond","license to kill",
    "one ring to rule them all","one ring to find them",
    "one ring to bring them all and in the darkness bind them",
    "you shall not pass","fly you fools","a wizard is never late",
    "not all those who wander are lost","even the smallest person can change the course of the future",
    "i would rather share one lifetime with you than face all the ages of this world alone",
    "my precious","gollum","smeagol","precious",
    "frankly my dear i dont give a damn","after all tomorrow is another day",
    "ill get you my pretty and your little dog too",
    "theres no place like home","pay no attention to that man behind the curtain",
    "toto ive a feeling were not in kansas anymore",
    "as god is my witness ill never be hungry again",
    "of all the gin joints in all the towns in all the world she walks into mine",
    "heres looking at you kid","well always have paris",
    "louis i think this is the beginning of a beautiful friendship",
    "round up the usual suspects","play it again sam",
    "youve got to ask yourself one question do i feel lucky well do ya punk",
    "go ahead make my day","i know what youre thinking",
    "did he fire six shots or only five","to tell you the truth in all this excitement ive kinda lost track myself",
    "but being as this is a 44 magnum the most powerful handgun in the world",
    "and would blow your head clean off","youve got to ask yourself one question",
    "do i feel lucky","well do ya punk",
    "i love the smell of napalm in the morning","the horror the horror",
    "terminate with extreme prejudice","charlie dont surf",
    "i am become death the destroyer of worlds",
    "now i am become death the destroyer of worlds",
    "you either die a hero or you live long enough to see yourself become the villain",
    "why so serious","its not about money its about sending a message",
    "introduce a little anarchy","upset the established order",
    "and everything becomes chaos","im an agent of chaos",
    "and you know the thing about chaos","its fair",
    "some men just want to watch the world burn",
    "im not a monster im just ahead of the curve",
    "you complete me","you had me at hello",
    "nobody puts baby in a corner","i feel the need the need for speed",
    "roads where were going we dont need roads",
    "great scott","where were going we dont need roads",
    "this is heavy","heavy duty","1.21 gigawatts",
    "flux capacitor","doc brown","marty mcfly",
    "life finds a way","clever girl","hold on to your butts",
    "welcome to jurassic park","objects in mirror are closer than they appear",
    "i am groot","we are groot","i am steve rogers",
    "avengers assemble","i can do this all day","i am iron man",
    "i love you 3000","on your left","wakanda forever",
    "i am inevitable","and i am iron man","snap",
    "whatever it takes","i am with you till the end of the line",
    "with great power comes great responsibility",
    "why do we fall sir","so that we can learn to pick ourselves up",
    "its not who i am underneath but what i do that defines me",
    "im batman","i am vengeance i am the night i am batman",
    "tell me do you bleed","you will","i am the night",
    "fear is a tool","i wear a mask","the batman",
    "i will find him","and i will kill him",
    "to infinity and beyond","youve got a friend in me",
    "reach for the sky","this town aint big enough for the two of us",
    "snake youre a solid snake","kept you waiting huh",
    "war has changed","metal gear","the phantom pain",
    "the cake is a lie","the cake is a lie the cake is a lie",
    "still alive","this was a triumph","im making a note here huge success",
    "its hard to overstate my satisfaction","aperture science",
    "we do what we must because we can","for the good of all of us",
    "except the ones who are dead","but theres no sense crying over every mistake",
    "you just keep on trying till you run out of cake",
    "and the science gets done","and you make a neat gun",
    "for the people who are still alive",
    "would you kindly","a man chooses a slave obeys",
    "is a man not entitled to the sweat of his brow",
    "no says the man in washington","it belongs to the poor",
    "no says the man in the vatican","it belongs to god",
    "no says the man in moscow","it belongs to everyone",
    "i rejected those answers","instead i chose something different",
    "i chose the impossible","i chose rapture",
    "a city where the artist would not fear the censor",
    "where the scientist would not be bound by petty morality",
    "where the great would not be constrained by the small",
    "and with the sweat of your brow rapture can become your city as well",
]
for m in movies:
    add(m, m.upper(), f"{m}!", f"{m}123")
print(f"   movies: {len(lines)}")

print("3. Leet speak exhaustive...")
# Base words that people leet-ify
bases = [
    "password","bitcoin","wallet","secret","private","master","monkey",
    "dragon","money","freedom","satoshi","nakamoto","blockchain","crypto",
    "admin","root","login","welcome","hello","test","love","trust",
    "sunshine","shadow","hunter","killer","hacker","leet","elite",
    "pwned","owned","haxor","l33t","n00b","1337","h4x0r",
]
# Simple leet map
leet_map = {
    "a": ["a", "4", "@"],
    "e": ["e", "3"],
    "i": ["i", "1", "!"],
    "o": ["o", "0"],
    "s": ["s", "5", "$"],
    "t": ["t", "7"],
    "l": ["l", "1"],
    "b": ["b", "8"],
    "g": ["g", "9"],
}
def leet_variants(word, max_vars=50):
    """Generate limited leet variants of a word."""
    results = {word, word.upper(), word.capitalize()}
    # Common full substitutions
    simple = word
    for old, new in [("a","4"),("e","3"),("i","1"),("o","0"),("s","5"),("t","7"),("l","1"),("b","8")]:
        simple = simple.replace(old, new)
    results.add(simple)
    results.add(simple.upper())
    # Partial
    results.add(word.replace("a","@").replace("e","3").replace("o","0").replace("i","1").replace("s","$"))
    results.add(word.replace("a","4").replace("e","3").replace("i","1").replace("o","0"))
    results.add(word.replace("s","5").replace("a","4").replace("e","3"))
    results.add("p@" + word[1:] if word.startswith("p") else word)
    # With numbers
    for n in ["1","12","123","1234","!","!!","!!!","2009","2010","2011"]:
        results.add(word + n)
        results.add(simple + n)
    return list(results)[:max_vars]

for b in bases:
    for v in leet_variants(b):
        add(v)
# Known classic leet passwords
classic_leet = [
    "p@ssw0rd","p@ssword","pa$$w0rd","p4ssw0rd","passw0rd",
    "h4ck3r","h4xor","h4x0r","h4cker","hack3r",
    "b1tc01n","b1tcoin","bitc0in","b1tc0in","81tc01n",
    "w4ll3t","w4llet","wall3t","w@ll3t",
    "s3cr3t","s3cret","secr3t","s3cr3t!",
    "pr1v4t3","pr1vate","priv4te","pr1v4te",
    "m4st3r","m4ster","mast3r","m@ster",
    "m0nk3y","m0nkey","monk3y","m0n3y",
    "dr4g0n","dr4gon","drag0n","dr@gon",
    "m0n3y","m0ney","mon3y","m0n3y!",
    "fr33d0m","fr33dom","freed0m","fr33d0m!",
    "s4t0sh1","s4toshi","sat0shi","s@toshi",
    "n4k4m0t0","n4kamoto","nakam0to",
    "bl0ckch41n","bl0ckchain","blockch41n",
    "crypt0","cryp70","cr4pto",
    "4dm1n","adm1n","@dmin","@dm1n",
    "r00t","r0ot","ro0t","r00t!",
    "l0g1n","log1n","l0gin","l0g1n!",
    "w3lc0m3","w3lcome","welc0me","w3lc0me!",
    "h3ll0","h3llo","hell0","h3ll0!",
    "t3st","t3st!","t3st123","t3st1ng",
    "l0v3","l0ve","lov3","l0v3!",
    "tru5t","tru5tn01","tru5tno1","tru5t!",
    "5un5h1n3","sunsh1ne","5unshine","5un5hine",
    "5h4d0w","sh4dow","5hadow","5h4d0w!",
    "hunt3r","hunt3r!","hunt3r123",
    "k1ll3r","k1ller","kill3r","k1ll3r!",
    "1337","l33t","31337","l33th4x0r",
    "n00b","n00b!","n00bie","pwn3d","0wn3d",
    "pr0","pr0h4x0r","pr0f3ss10n4l",
    "g0d","g0d!","g0d123","g0dmode",
    "1l0v3y0u","il0vey0u","1loveyou","1l0v3y0u!",
    "l3tm31n","l3tmein","letm31n","l3tm31n!",
    "qu3rty","qw3rty","querty","qw3rty!",
    "4bcd3f","4bcdef","abcd3f","4bcd3f!",
    "z3r0","z3ro","zer0","z3r0!",
    "0n3","0ne","on3","0n3!",
    "tw0","tw0!","tw0123",
    "thr33","thr3e","thre3","thr33!",
    "f0ur","f0ur!","f0ur123",
    "f1v3","f1ve","fiv3","f1v3!",
    "51x","51x!","51x123",
    "53v3n","s3ven","sev3n","53v3n!",
    "31ght","e1ght","eigh7","31ght!",
    "n1n3","n1ne","nin3","n1n3!",
    "t3n","t3n!","t3n123",
]
for c in classic_leet:
    add(c, c.upper(), f"{c}!", f"{c}123")
print(f"   leet: {len(lines)}")

print("4. Two-word adjective+noun pairs (high frequency)...")
adjs = [
    "red","blue","green","black","white","golden","silver","dark","bright",
    "big","small","great","old","new","hot","cold","fast","slow","strong",
    "wild","free","true","pure","sweet","bitter","soft","hard","deep","high",
    "low","long","short","first","last","best","worst","secret","hidden",
    "magic","cosmic","epic","super","mega","ultra","hyper","cyber","digital",
    "virtual","crypto","private","public","open","closed","safe","secure",
    "lucky","happy","sad","angry","brave","calm","crazy","smart","wise",
]
nouns = [
    "dragon","tiger","wolf","eagle","bear","lion","fox","hawk","shark","whale",
    "fire","water","earth","wind","storm","thunder","lightning","rain","snow",
    "sun","moon","star","sky","ocean","river","mountain","forest","garden",
    "castle","tower","bridge","sword","shield","arrow","hammer","key","lock",
    "door","window","house","home","road","path","journey","quest","mission",
    "dream","hope","wish","love","heart","soul","mind","spirit","power",
    "force","energy","light","shadow","night","day","dawn","dusk","midnight",
    "king","queen","prince","knight","warrior","hero","master","lord","god",
    "angel","demon","ghost","phoenix","unicorn","wizard","mage","ninja",
    "wallet","bitcoin","crypto","money","gold","diamond","crystal","pearl",
    "code","hash","key","seed","password","secret","truth","freedom",
]
for a, n in product(adjs, nouns):
    add(f"{a} {n}", f"{a}{n}", f"{a}-{n}", f"{a}_{n}")
    add(f"{a} {n}!", f"{a}{n}123", f"my {a} {n}", f"the {a} {n}")
print(f"   2words: {len(lines)}")

print("5. Three-word common phrases...")
dets = ["my","the","our","your","a","an"]
adjs3 = ["red","blue","black","white","golden","secret","private","big","great","first","last","new","old"]
nouns3 = ["dragon","wallet","bitcoin","key","password","secret","money","gold","house","dream","hope","life","love","heart","soul","power","sword","castle","mountain","ocean","star","moon","fire","shadow"]
for d, a, n in product(dets, adjs3, nouns3):
    add(f"{d} {a} {n}", f"{d}{a}{n}", f"{d}-{a}-{n}")
print(f"   3words: {len(lines)}")

print("6. MD5 hashes of common words (used as keys)...")
md5_bases = [
    "password","123456","12345678","qwerty","abc123","test","hello","admin",
    "bitcoin","satoshi","nakamoto","wallet","private","secret","master","love",
    "freedom","money","god","jesus","dragon","monkey","sunshine","shadow",
    "letmein","trustno1","iloveyou","welcome","login","guest","root","pass",
    "","a","1","0","bitcoin2009","genesis","hodl","moon","crypto",
]
for base in md5_bases:
    h = hashlib.md5(base.encode()).hexdigest()
    add(h)
    # Also SHA256 of simple words as hex strings used as passphrase
    s = hashlib.sha256(base.encode()).hexdigest()
    add(s)
    # Double SHA256
    d = hashlib.sha256(hashlib.sha256(base.encode()).digest()).hexdigest()
    add(d)
print(f"   hashes: {len(lines)}")

print("7. Famous foreign quotes (transliterated / original ASCII)...")
foreign_quotes = [
    # French
    "je pense donc je suis","liberte egalite fraternite","vivre libre ou mourir",
    "l etat cest moi","apres moi le deluge","cest la vie","plus ca change",
    "la vie est belle","tout est bien qui finit bien","qui vivra verra",
    "mieux vaut tard que jamais","l union fait la force","a chaque jour suffit sa peine",
    "le temps c est de l argent","savoir c est pouvoir","rien ne sert de courir",
    "il faut cultiver notre jardin","l enfer c est les autres",
    "on ne nait pas femme on le devient","je suis mon propre maitre",
    # German
    "ich denke also bin ich","arbeit macht frei","gott mit uns",
    "einigkeit und recht und freiheit","deutschland uber alles",
    "was mich nicht umbringt macht mich starker","gott ist tot",
    "der wille zur macht","also sprach zarathustra","ewige wiederkunft",
    "sei was du bist","erkenne dich selbst","zeit ist geld",
    "wissen ist macht","ubung macht den meister","ende gut alles gut",
    # Spanish
    "pienso luego existo","vencer o morir","la vida es sueno",
    "el que no arriesga no gana","mas vale tarde que nunca",
    "el tiempo es oro","saber es poder","la union hace la fuerza",
    "a cada dia su afan","quien ríe ultimo ríe mejor",
    "no hay mal que por bien no venga","todo lo que brilla no es oro",
    "dime con quien andas y te dire quien eres",
    # Latin
    "cogito ergo sum","veni vidi vici","alea iacta est","carpe diem",
    "memento mori","et tu brute","amor vincit omnia","in vino veritas",
    "sic transit gloria mundi","tempus fugit","ars longa vita brevis",
    "mens sana in corpore sano","fortuna audaces iuvat","divide et impera",
    "per aspera ad astra","ad astra per aspera","errare humanum est",
    "audentes fortuna iuvat","dulce et decorum est pro patria mori",
    "homo homini lupus","bellum omnium contra omnes","tabula rasa",
    "quod erat demonstrandum","eureka","gnothi seauton",
    # Italian
    "penso dunque sono","la vita e bella","chi va piano va sano e va lontano",
    "meglio tardi che mai","il tempo e denaro","sapere e potere",
    "l unione fa la forza","chi dorme non piglia pesci",
]
for q in foreign_quotes:
    add(q, q.upper(), f"{q}!", f"{q}123")
print(f"   foreign: {len(lines)}")

print("8. Sports teams & players...")
sports = [
    "yankees","red sox","dodgers","giants","cubs","mets","cardinals","braves",
    "lakers","celtics","bulls","warriors","heat","spurs","knicks","nets",
    "cowboys","patriots","packers","steelers","49ers","eagles","ravens","chiefs",
    "manchester united","manchester city","liverpool","arsenal","chelsea","tottenham",
    "barcelona","real madrid","bayern munich","juventus","psg","milan","inter",
    "ferrari","mclaren","mercedes f1","red bull racing","williams","lotus",
    "michael jordan","lebron james","kobe bryant","tiger woods","messi","ronaldo",
    "pele","maradona","federer","nadal","serena williams","usain bolt",
    "muhammad ali","mike tyson","conor mcgregor","floyd mayweather",
    "super bowl","world cup","world series","nba finals","champions league",
    "olympics","stanley cup","tour de france","wimbledon","masters",
]
for s in sports:
    add(s, s.upper(), f"{s}!", f"{s}123", f"{s}2009", f"{s}2010")
print(f"   sports: {len(lines)}")

print("9. Common email-style / username patterns...")
users = ["admin","user","test","guest","root","bitcoin","satoshi","wallet","crypto","hodl"]
domains = ["gmail.com","yahoo.com","hotmail.com","aol.com","mail.com","bitcoin.org","bitcointalk.org"]
for u, d in product(users, domains):
    add(f"{u}@{d}", f"{u}{d}", f"{u}.{d}")
for u in users:
    for n in range(0, 100):
        add(f"{u}{n}", f"{u}_{n}", f"{u}.{n}")
print(f"   users: {len(lines)}")

print("10. Calendar months + years...")
months = [
    "january","february","march","april","may","june","july","august",
    "september","october","november","december",
    "jan","feb","mar","apr","jun","jul","aug","sep","oct","nov","dec",
]
for m in months:
    for y in range(2008, 2026):
        add(f"{m}{y}", f"{m} {y}", f"{m}-{y}")
    add(m, m.upper(), f"{m}!", f"{m}123")
print(f"   months: {len(lines)}")

# Write
print(f"\nWriting {len(lines)} unique patterns...")
sorted_lines = sorted(lines)
OUTPUT.write_text("\n".join(sorted_lines) + "\n", encoding="utf-8")
print(f"Done. Total Wave 4: {len(sorted_lines)}")
