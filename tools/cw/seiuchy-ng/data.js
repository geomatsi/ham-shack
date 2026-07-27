/*
 * Seiuchy-NG -- all editable content lives here.
 *
 * Everything below is plain data: word lists and phrase templates.  Edit it with
 * any text editor, reload the page, and the trainer uses the new material.  The
 * only rules are: keep the JavaScript syntax intact (double quotes around every
 * string, comma between entries), and keep the text lowercase and free of
 * characters Morse cannot send (a-z, 0-9, space, and . , ? = + / are safe).
 *
 * Placeholders in templates are written {like_this} and are substituted at run
 * time.  Repeating an entry in a list makes it come up more often -- roughly
 * twice as often for a second copy, not exactly, because the trainer also
 * refuses to repeat anything it has just sent (see pick() in qso.js).
 *
 * This material is original to Seiuchy NG.  Names, towns, callsign prefixes,
 * radio models and on-air abbreviations are facts and common ham jargon; the
 * selection and the phrasing are this project's own.
 */
const SEIUCHY_DATA = {

  /* ===================================================================== *
   *  Word lists
   * ===================================================================== */

  // First names as they get sent on the air: short, no accents.
  names: [
    "aki", "al", "alan", "albert", "alex", "ali", "andre", "andrea", "andrei", "andy",
    "angel", "anton", "antonio", "arne", "arto", "art", "attila", "aziz", "ben", "bernd",
    "bert", "bill", "bo", "bob", "boris", "brian", "bruno", "carl", "carlos", "cesar",
    "chan", "charlie", "chen", "chris", "claude", "clive", "dan", "dave", "david", "denis",
    "diego", "dieter", "dima", "dirk", "dmitri", "don", "doug", "eddie", "edgar", "eduardo",
    "emil", "eric", "erik", "ernst", "esteban", "fabio", "fahd", "felix", "filip", "finn",
    "frank", "franz", "fred", "gabor", "gary", "geoff", "georg", "george", "gerd", "giorgi",
    "giovanni", "greg", "guido", "gunnar", "hans", "harald", "harry", "heinz", "helmut", "henk",
    "henri", "henry", "hiro", "hugo", "ian", "igor", "ivan", "jack", "jan", "janos",
    "jarmo", "javier", "jean", "jens", "jerry", "jim", "jiri", "joe", "john", "jorge",
    "jose", "juan", "juha", "jukka", "julio", "jun", "kai", "karel", "karl", "keith",
    "ken", "kevin", "kim", "klaus", "kurt", "lars", "laszlo", "lee", "leo", "leon",
    "lars", "luca", "luigi", "luis", "luke", "maciek", "manu", "marc", "marco", "marek",
    "mario", "mark", "martin", "masa", "mat", "matteo", "max", "michel", "mike", "miguel",
    "mikko", "milan", "mirko", "misha", "mohamed", "murat", "nick", "niels", "niko", "noah",
    "norm", "olaf", "ole", "oleg", "oliver", "omar", "orest", "oskar", "otto", "pablo",
    "paco", "paolo", "pascal", "patrick", "paul", "pavel", "pedro", "pekka", "per", "pete",
    "peter", "phil", "pierre", "piotr", "rafa", "ralf", "ramon", "raul", "ray", "reiner",
    "rene", "ricardo", "rich", "rob", "roberto", "rod", "roger", "roland", "rolf", "ron",
    "ruben", "rudi", "sami", "samuel", "sandro", "sasha", "scott", "sean", "seb", "serge",
    "sergey", "seppo", "shin", "simon", "stan", "steen", "stefan", "steve", "sven", "taka",
    "tarik", "tay", "ted", "terry", "thomas", "tim", "tino", "tom", "tomas", "toni",
    "tony", "toshi", "ulf", "uwe", "vadim", "valery", "vic", "victor", "vlad", "walt",
    "wei", "werner", "will", "willy", "wolf", "yuji", "yuri", "zoli",
    // women are still thin on the air but they are there
    "ana", "anne", "barb", "carla", "carol", "clara", "diana", "elena", "elsa", "emma",
    "eva", "gerda", "hanna", "helen", "ines", "irina", "jane", "julia", "kate", "laura",
    "lena", "lisa", "lucy", "maja", "maria", "marta", "nadia", "nina", "olga", "paula",
    "petra", "rita", "rosa", "sara", "sofia", "sonia", "tanya", "vera", "wendy", "yuki"
  ],

  // What people say when you ask what they are running.  Modern boxes, classics,
  // QRP kits and SDRs, spelled the way they are sent.
  rigs: [
    "ic 7300", "ic 7300", "ic 705", "ic 705", "ic 7610", "ic 7100", "ic 7000", "ic 706",
    "ic 703", "ic 718", "ic 746", "ic 756", "ic 761", "ic 765", "ic 775", "ic 7851",
    "ic 9700", "ic 7410", "ic 7200", "ic 728",
    "ft 991a", "ft 991a", "ft 891", "ft 891", "ft 710", "ftdx 10", "ftdx 10", "ftdx 101",
    "ftdx 3000", "ftdx 5000", "ft 817", "ft 818", "ft 857", "ft 897", "ft 450", "ft 950",
    "ft 1000mp", "ft 101", "ft 102", "ft 707", "ft 736", "ft 900", "ft 990",
    "ts 590sg", "ts 590sg", "ts 890", "ts 990", "ts 480", "ts 850", "ts 870", "ts 940",
    "ts 950", "ts 830s", "ts 820", "ts 520", "ts 530", "ts 130", "ts 440", "ts 450",
    "ts 2000", "ts 700",
    "k3", "k3s", "k4", "kx2", "kx2", "kx3", "kx3", "k2", "k1",
    "flex 6400", "flex 6600", "flex 6700", "hermes lite 2", "anan 7000", "sun sdr2",
    "xiegu g90", "xiegu g90", "xiegu x6100", "xiegu x5105", "xiegu g106",
    "qcx", "qcx mini", "qcx plus", "qmx", "mtr3b", "mtr4b", "tr35", "tr45l", "sw3b",
    "pixie", "rockmite", "tuna tin 2", "forty niner", "bitx40", "ubitx", "frog sounds",
    "hw 8", "hw 9", "hw 16", "hw 101", "sb 102", "drake tr4", "drake r4b", "collins kwm 2",
    "swan 350", "argonaut 509", "argonaut vi", "omni 6", "corsair 2", "triton 4", "eagle",
    "orion 2", "century 21", "uw3di", "ur5", "homebrew", "homebrew", "homebrew rig",
    "old military set", "boat anchor", "regen rx"
  ],

  // Power levels, used as padding around the rig.
  powers: [
    "5w", "5w", "5 watts", "10w", "10 watts", "20w", "25w", "40w", "50w", "60 watts",
    "80w", "100w", "100w", "100 watts", "100 watts", "abt 100w", "abt 5w", "200w",
    "400 watts", "500w", "kw", "full kw", "qrp", "qrp 5w", "qrpp", "1w", "half a watt",
    "2 watts", "legal limit"
  ],

  // Antennas, likewise.
  antennas: [
    "dipole", "dipole", "inv vee", "inv vee", "g5rv", "vertical", "vertical", "gp",
    "end fed", "efhw", "long wire", "random wire", "doublet", "zepp", "windom",
    "ocf dipole", "fan dipole", "loop", "delta loop", "mag loop", "quad", "yagi",
    "3 el yagi", "2 el yagi", "moxon", "hex beam", "cobweb", "buddipole", "hamstick",
    "w3dzz", "mobile whip", "wire in a tree", "attic dipole", "sloper", "beverage"
  ],

  // "{animal}" in a job is replaced by a random entry from the animals list.
  jobs: [
    "engineer", "teacher", "doctor", "nurse", "farmer", "pilot", "baker", "butcher",
    "cook", "chef", "mechanic", "electrician", "plumber", "carpenter", "welder", "roofer",
    "mason", "painter", "driver", "truck driver", "taxi driver", "train driver", "sailor",
    "fisherman", "miner", "lumberjack", "surveyor", "architect", "draftsman", "chemist",
    "physicist", "biologist", "geologist", "astronomer", "mathematician", "programmer",
    "sysadmin", "network engineer", "radio engineer", "radio officer", "air traffic ctrl",
    "technician", "watchmaker", "instrument maker", "machinist", "toolmaker", "optician",
    "dentist", "vet", "pharmacist", "paramedic", "fire fighter", "police officer",
    "soldier", "sailor in the navy", "coast guard", "customs officer", "postman",
    "librarian", "archivist", "historian", "translator", "journalist", "photographer",
    "printer", "bookbinder", "typesetter", "banker", "accountant", "clerk", "lawyer",
    "notary", "salesman", "shopkeeper", "grocer", "baker again hi", "brewer", "winemaker",
    "cheesemaker", "gardener", "forester", "beekeeper", "shepherd", "{animal} breeder",
    "{animal} trainer", "musician", "piano tuner", "organ builder", "luthier", "singer",
    "actor", "dancer", "sculptor", "potter", "weaver", "tailor", "shoemaker", "jeweler",
    "glass blower", "blacksmith", "boat builder", "model maker", "kite maker",
    "lighthouse keeper", "park ranger", "mountain guide", "ski instructor", "diving instr",
    "chess coach", "teacher of cw", "student", "apprentice", "retired hi hi"
  ],

  animals: [
    "dog", "cat", "horse", "pony", "goat", "sheep", "cow", "pig", "chicken", "duck",
    "goose", "rabbit", "parrot", "canary", "pigeon", "falcon", "owl", "bee", "koi",
    "tortoise", "husky", "spaniel", "collie", "alpaca", "donkey", "ferret"
  ],

  // CW clubs that hand out membership numbers.
  clubs: ["skcc", "skcc", "fists", "fists", "cwops", "agcw", "hsc", "naqcc", "qrp arci", "ten ten"],

  // Signal reports that get sent verbatim, besides 599/5nn and random ones.
  reports: ["579", "569", "559", "549", "539", "529", "588", "577", "566"],

  /* ===================================================================== *
   *  Phrase templates
   *
   *  One of these is picked at random and wrapped around the answer.  The
   *  placeholder ({name}, {rst}, ...) is what you are expected to copy; any
   *  other placeholder is just padding.
   * ===================================================================== */

  phrases: {

    name: [
      "name {name}", "name is {name}", "name hr is {name}", "my name is {name}",
      "op hr {name}", "op is {name}", "op {name}", "name hr {name}", "name name {name}",
      "hr {name}", "es my name is {name}", "the name is {name}", "op hr is {name}",
      "name {name} spelt {name}", "{name} is my name", "im {name}"
    ],

    rst: [
      "ur rst {rst}", "ur rst is {rst}", "rst {rst}", "rst is {rst}", "ur {rst}",
      "u r {rst}", "ur rprt {rst}", "ur rprt is {rst}", "rst rst {rst}",
      "ur sigs r {rst}", "sigs {rst}", "ur sigs {rst} hr", "copy u {rst}",
      "ur rst {rst} in qsb", "{rst} on my end", "rst ur {rst}"
    ],

    age: [
      "age {age}", "age hr {age}", "age is {age}", "my age is {age}", "im {age}",
      "im {age} yrs old", "im {age} years old", "age {age} {age}", "hr age {age}",
      "age hr is {age} {age}", "{age} yrs young hi", "es im {age} yrs"
    ],

    // "nr" (near) is deliberately part of some of these: learning to drop it is
    // half the exercise.
    qth: [
      "qth {qth}", "qth is {qth}", "qth hr {qth}", "qth hr is {qth}", "my qth is {qth}",
      "my qth {qth}", "qth qth {qth}", "im in {qth}", "hr in {qth}", "wkg u frm {qth}",
      "qth {qth} tonite", "qth is {qth} es nice wx", "qth nr {qth}", "qth is nr {qth}",
      "my qth is nr {qth}", "qth hr nr {qth}", "im nr {qth}", "just outside {qth}",
      "abt 20 km frm {qth}"
    ],

    rig: [
      "rig {rig}", "rig is {rig}", "rig hr is {rig}", "my rig is a {rig}", "tx is {rig}",
      "trx is {rig}", "rig hr {rig}", "using a {rig}", "running a {rig}",
      "rig is {rig} es pwr {pwr}", "rig hr {rig} running {pwr}",
      "rig {rig} into a {ant}", "rig is {rig} es ant is {ant}",
      "{rig} at {pwr} to a {ant}", "my rig is {rig} pwr {pwr} ant {ant}",
      "tx {rig} es {ant} up 10m", "rig hr is {rig} barefoot"
    ],

    // The callsign arrives already doubled in {call}.
    call: [
      "de {call} +", "de {call} k", "cq de {call} k", "cq de {call} pse k",
      "cq cq de {call} k", "de {call} kn", "{call} qrz", "de {call} ar k"
    ],

    club: [
      "{club} {nr}", "{club} nr {nr}", "hr {club} nr {nr}", "my {club} nr is {nr}",
      "im {club} {nr}", "im {club} nr {nr}", "{club} member nr {nr}",
      "r u {club}? my nr is {nr}", "u {club}? im {club} nr {nr}",
      "{club} nr hr is {nr}", "nr {nr} in {club}"
    ],

    // Jobs, phrased according to the age of the fictional operator.
    jobNow: [
      "im a {job}", "i work as a {job}", "my job is {job}", "im a {job} by trade",
      "work hr is {job}", "im a {job} hi"
    ],
    jobPast: [
      "im a retired {job}", "used to be a {job}", "im an ex {job}", "i was a {job}",
      "im a former {job}", "hr ex {job}", "retired {job} nw", "was a {job} b4 retiring"
    ],
    jobFuture: [
      "id like to be a {job}", "i want to be a {job}", "im studying to be a {job}",
      "training to be a {job}", "im in {job} school", "hoping to be a {job}"
    ],

    // Padding sent before / after the interesting bit.  {om} is how the other
    // operator addresses you (see "address" below).
    fluffBefore: [
      "tnx {om}", "tks {om}", "rr {om} =", "r r {om}", "ok {om}", "all ok {om} =",
      "fb {om} =", "fb cpi {om}", "fb on ur info {om}", "tu {om} =", "many tnx {om} =",
      "gm {om}", "ge {om}", "ga {om}", "gud to meet u {om} ="
    ],
    fluffAfter: [
      "hi hi", "= so hw?", "hw cpi?", "qsl?", "bk", "btu {om}", "= qru", "= hw?", "+",
      "ok?", "= wx hr fb", "= wx is cldy", "= wx rain hr", "= rig is fb tdy",
      "= band is gud", "= qsb on ur sigs", "= sri qrm", "= hpe u cpi", "es tnx {om}"
    ],

    // {name} is the name you entered in the settings.
    address: [
      "{name}", "{name}", "dr {name}", "om", "dr om", "", "", ""
    ],

    // Word separators sprinkled between sentences.
    separators: [" ", " = ", " ", " = ", " = ", " ", " = "]
  },

  /* ===================================================================== *
   *  Contest exchanges
   * ===================================================================== */

  contest: {
    // ARRL and RAC sections, as sent in Field Day and Sweepstakes.
    fieldDaySections: [
      "ct", "ema", "me", "nh", "ri", "vt", "wma", "eny", "nli", "nnj", "nny", "snj",
      "wny", "de", "epa", "md", "wpa", "al", "ga", "ky", "nc", "nfl", "sc", "sfl",
      "tn", "va", "pr", "vi", "wcf", "ar", "la", "ms", "nm", "ntx", "ok", "stx",
      "wtx", "eb", "lax", "org", "sb", "scv", "sdg", "sf", "sjv", "sv", "pac", "az",
      "ewa", "id", "mt", "nv", "or", "ut", "wwa", "wy", "ak", "mi", "oh", "wv", "il",
      "in", "wi", "co", "ia", "ks", "mn", "mo", "ne", "nd", "sd", "mar", "nl", "qc",
      "one", "ont", "gta", "onn", "ons", "mb", "sk", "ab", "bc", "nt"
    ],
    fieldDayCategories: ["a", "b", "c", "d", "e", "f"],

    // Maidenhead fields holding a useful amount of inhabited land, worked out
    // from coarse continent boxes.  Random QTHs are nudged towards these so you
    // do not spend the evening working the middle of the Pacific.
    populatedFields: [
      "ao", "ap", "aq", "bl", "bo", "bp", "bq", "cl", "cm", "cn", "co", "cp", "cq", "dj",
      "dk", "dl", "dm", "dn", "do", "dp", "dq", "ee", "ef", "eg", "eh", "ei", "ej", "ek",
      "el", "em", "en", "eo", "ep", "eq", "fd", "fe", "ff", "fg", "fh", "fi", "fj", "fk",
      "fl", "fm", "fn", "fo", "fp", "fq", "fr", "gd", "ge", "gf", "gg", "gh", "gi", "gj",
      "gk", "gn", "go", "gp", "gq", "gr", "hd", "he", "hf", "hg", "hh", "hi", "hj", "hk",
      "hm", "hp", "hq", "hr", "ij", "ik", "il", "im", "in", "io", "ip", "iq", "ir", "jf",
      "jg", "jh", "ji", "jj", "jk", "jl", "jm", "jn", "jo", "jp", "kf", "kg", "kh", "ki",
      "kj", "kk", "kl", "km", "kn", "ko", "kp", "lg", "lh", "li", "lj", "lk", "ll", "lm",
      "ln", "lo", "lp", "mj", "mk", "ml", "mm", "mn", "mo", "mp", "mq", "ni", "nj", "nk",
      "nl", "nm", "nn", "no", "np", "nq", "oe", "of", "og", "oh", "oi", "oj", "ok", "ol",
      "om", "on", "oo", "op", "oq", "pe", "pf", "pg", "ph", "pi", "pj", "pl", "pm", "pn",
      "po", "pp", "pq", "qe", "qf", "qg", "qh", "qi", "qm", "qn", "qo", "qp", "qq", "re",
      "rf", "ro", "rp", "rq"
    ]
  },

  /* ===================================================================== *
   *  Keying styles -- how the operator at the other end actually sends
   *
   *  Every value is a [base, spread] pair multiplying the textbook duration:
   *  the factor is base + random() * spread.  1 means perfect timing.
   *    dit/dah    length of the elements
   *    gap        space between elements inside a character
   *    letter     space between characters
   *    word       space between words
   *  "sloppyDots" occasionally adds a stray dot to digits and punctuation.
   *  "inRandom" marks the styles the "Random key" setting may choose from.
   *
   *  Rules of thumb used below: a keyer gets the elements exactly right and can
   *  only fumble the spaces; a straight key wobbles everything a little; a bug
   *  makes machine-gun dits against hand-made dahs, and the worse the fist, the
   *  wider that ratio gets.
   * ===================================================================== */

  keys: {
    computer:
      { label: "Computer", dit: [1, 0], dah: [1, 0], gap: [1, 0], letter: [1, 0], word: [1, 0], inRandom: true },
    paddle:
      { label: "Paddle, expert", dit: [1, 0], dah: [1, 0], gap: [1, 0], letter: [0.97, 0.18], word: [0.9, 0.25], inRandom: true },
    paddlemed:
      { label: "Paddle", dit: [1, 0], dah: [1, 0], gap: [1, 0], letter: [0.88, 0.45], word: [0.7, 0.7], inRandom: true },
    paddlebad:
      { label: "Paddle, novice", dit: [1, 0], dah: [1, 0], gap: [1, 0], letter: [0.85, 1.1], word: [0.6, 1.5], sloppyDots: true, inRandom: true },
    straight:
      { label: "Straight key, expert", dit: [0.97, 0.1], dah: [0.94, 0.16], gap: [0.97, 0.1], letter: [0.95, 0.18], word: [0.9, 0.2], inRandom: true },
    straightmed:
      { label: "Straight key", dit: [0.94, 0.28], dah: [0.88, 0.3], gap: [0.95, 0.3], letter: [0.88, 0.35], word: [0.85, 0.3], inRandom: true },
    straightbad:
      { label: "Straight key, novice", dit: [0.82, 0.55], dah: [0.78, 0.5], gap: [0.85, 0.55], letter: [0.8, 0.6], word: [0.7, 0.5], inRandom: true },
    straightvbad:
      { label: "Straight key, horror", dit: [0.75, 1.4], dah: [0.6, 0.9], gap: [0.8, 1.5], letter: [0.6, 1.1], word: [0.45, 0.9] },
    bug:
      { label: "Bug, expert", dit: [0.95, 0], dah: [0.9, 0.25], gap: [0.95, 0], letter: [0.9, 0.3], word: [0.9, 0.3], inRandom: true },
    bugmed:
      { label: "Bug", dit: [0.8, 0], dah: [0.85, 0.45], gap: [0.8, 0], letter: [0.85, 0.5], word: [0.85, 0.5], inRandom: true },
    bugbad:
      { label: "Bug, novice", dit: [0.6, 0], dah: [0.85, 0.75], gap: [0.6, 0], letter: [0.8, 0.8], word: [0.8, 0.8], sloppyDots: true, inRandom: true },
    bugvbad:
      { label: "Bug, nightmare", dit: [0.45, 0.1], dah: [0.7, 1.1], gap: [0.5, 0.1], letter: [0.55, 1.3], word: [0.5, 1.3], sloppyDots: true },
    cootie:
      { label: "Cootie", dit: [0.8, 0.4], dah: [0.85, 0.4], gap: [0.6, 0.2], letter: [0.85, 0.3], word: [0.8, 0.4], inRandom: true },
    swing:
      { label: "Swing", dit: [0.75, 0], dah: [1.25, 0], gap: [0.8, 0], letter: [1.1, 0], word: [1, 0], inRandom: true },
    elmer:
      { label: "Elmer, roomy spacing", dit: [1, 0], dah: [1, 0], gap: [1, 0], letter: [1.6, 0], word: [2.5, 0] },
    farnsworth:
      { label: "Farnsworth", dit: [1, 0], dah: [1, 0], gap: [1, 0], letter: [3, 0], word: [5, 0] },
    farnsworthsk:
      { label: "Farnsworth, straight key", dit: [0.94, 0.28], dah: [0.88, 0.3], gap: [0.95, 0.3], letter: [3, 0.5], word: [5, 0.5] }
  },

  /* ===================================================================== *
   *  Made-up place names
   *
   *  When "real place names only" is off, towns get built by gluing a prefix to
   *  a suffix (or by stringing syllables together), so you keep meeting places
   *  you have never heard of -- which is the point: you have to copy them
   *  letter by letter instead of guessing.
   *
   *  preWordChance / postWordChance are how often a loose word gets stuck on
   *  the front or the back.
   * ===================================================================== */

  placeStyles: {

    german: {
      prefixes: [
        "alten", "ober", "nieder", "hohen", "gross", "klein", "neu", "wald", "stein", "birken",
        "eichen", "rosen", "kirsch", "moor", "sand", "tann", "buchen", "kalten", "gruen",
        "schoen", "bruch", "ehren", "falken", "hasel", "kranich", "marien", "nord", "west",
        "rot", "weiss", "schwarz", "blau", "muehlen", "kirchen", "berg", "linden", "erlen",
        "fichten", "hirsch", "adler", "sonnen", "winter", "sommer"
      ],
      suffixes: [
        "dorf", "heim", "berg", "burg", "feld", "hausen", "bach", "tal", "stein", "brunn",
        "furt", "hofen", "kirchen", "ried", "see", "wald", "weiler", "ingen", "stadt",
        "roda", "leben", "scheid", "bruck", "au", "moos", "eck", "grund", "steig"
      ],
      preWords: ["bad ", "sankt ", "unter "],
      postWords: [" am see", " am rhein", " im tal", " an der elbe"],
      preWordChance: 0.08, postWordChance: 0.04
    },

    french: {
      prefixes: [
        "mont", "ville", "chateau", "beau", "bel", "roche", "val", "bois", "pont", "fond",
        "clair", "haute", "basse", "grand", "petit", "cour", "port", "mare", "plou", "ker",
        "chal", "bourg", "vaux", "puy", "aigue", "fleur", "champ", "sault", "mesnil", "mou",
        "sar", "cha", "lan", "gour", "cor", "neu", "ver", "bor", "carc", "malan"
      ],
      suffixes: [
        "ville", "court", "mont", "val", "gny", "sac", "lieu", "thier", "vaux", "chy",
        "lin", "dun", "ac", "an", "ay", "ec", "eux", "ieres", "ols", "ombe", "rieu",
        "sanne", "tot", "y", "gnac", "mard", "beliard", "reuil", "noise", "magny"
      ],
      preWords: ["saint ", "le ", "la "],
      postWords: [" sur mer", " le duc", " les bains", " sur loire"],
      preWordChance: 0.06, postWordChance: 0.04
    },

    english: {
      prefixes: [
        "north", "south", "east", "west", "upper", "lower", "great", "little", "new", "old",
        "black", "white", "red", "green", "king", "queen", "ash", "oak", "elm", "thorn",
        "stone", "mill", "bridge", "wood", "holm", "chester", "win", "brad", "ship", "hart",
        "fox", "raven", "barn", "field", "brook", "cold", "long", "mar", "pen", "wick"
      ],
      suffixes: [
        "ton", "ham", "ford", "field", "bury", "borough", "ley", "wick", "dale", "worth",
        "thorpe", "mouth", "bridge", "hill", "don", "stoke", "combe", "wold", "shaw", "moor",
        "by", "gate", "well", "church", "castle", "minster", "haven", "mere", "cliff"
      ],
      preWords: ["new ", "old ", "little "],
      postWords: [" on sea", " under wood", " magna"],
      preWordChance: 0.05, postWordChance: 0.03
    },

    celtic: {
      prefixes: [
        "bally", "kil", "dun", "glen", "inver", "ard", "drum", "carrick", "lough", "kin",
        "strath", "aber", "llan", "pen", "tre", "caer", "cwm", "rhos", "bal", "clon",
        "ennis", "tully", "knock", "rath", "ross", "auch", "craig", "kirk"
      ],
      suffixes: [
        "more", "beg", "na", "an", "ard", "ford", "ness", "town", "gowan", "dee", "wen",
        "fawr", "fach", "coed", "mawr", "aig", "loch", "mory", "shire", "vara", "gorm"
      ],
      preWordChance: 0, postWordChance: 0
    },

    nordic: {
      prefixes: [
        "stor", "lille", "ny", "gamle", "val", "berg", "sol", "havn", "fjord", "myr",
        "skog", "sand", "tors", "ulv", "orn", "bjorn", "hall", "sjo", "vester", "oster",
        "norr", "soder", "ang", "kvik", "gran", "bok", "elg", "hjort", "malm", "kvarn"
      ],
      suffixes: [
        "vik", "holm", "berg", "dal", "fjord", "sund", "by", "stad", "hamn", "nas",
        "fors", "lund", "torp", "borg", "koping", "hult", "sater", "strand", "vall", "asen"
      ],
      preWordChance: 0, postWordChance: 0
    },

    finnish: {
      prefixes: [
        "kivi", "joki", "mets", "salo", "koivu", "honka", "harju", "lehto", "niitty",
        "kallio", "ranta", "saari", "vuori", "aho", "kuusi", "pelto", "myllys", "hauki",
        "sini", "valkea", "musta", "iso", "pieni", "yla", "ala", "kaarna", "lampi"
      ],
      suffixes: [
        "jarvi", "koski", "maki", "lahti", "niemi", "saari", "vaara", "kyla", "joki",
        "harju", "ranta", "salmi", "virta", "luoto", "kangas", "suo", "nummi", "ala"
      ],
      preWordChance: 0, postWordChance: 0
    },

    dutch: {
      prefixes: [
        "oud", "nieuw", "groot", "klein", "noord", "zuid", "oost", "west", "hoog", "laag",
        "zand", "veen", "water", "molen", "kerk", "hei", "wilder", "beek", "rijs", "duin",
        "haar", "schoon", "vries", "boom", "grave"
      ],
      suffixes: [
        "dam", "dijk", "veen", "hoven", "kerk", "broek", "wijk", "dorp", "meer", "oord",
        "zand", "land", "sluis", "brug", "laar", "bergen", "hout", "beek", "recht", "poort"
      ],
      preWords: ["nieuw "],
      postWords: [" aan zee"],
      preWordChance: 0.06, postWordChance: 0.03
    },

    iberian: {
      prefixes: [
        "villa", "puerto", "san", "santa", "monte", "valle", "alta", "baja", "rio", "fuente",
        "torre", "campo", "casa", "pena", "vega", "laguna", "robl", "alcala", "guada",
        "cabo", "sierra", "aran", "castel", "novo", "porto", "vila", "ribeira", "praia"
      ],
      suffixes: [
        "mar", "real", "nueva", "vieja", "verde", "blanca", "mayor", "menor", "llana",
        "brava", "dulce", "hermosa", "dorada", "negra", "alta", "linda", "grande", "chico",
        "inho", "oso", "eiro", "velho", "novo"
      ],
      preWords: ["san ", "santa ", "vila "],
      postWords: [" del mar", " de arriba", " del rio", " de la sierra"],
      preWordChance: 0.05, postWordChance: 0.07
    },

    italian: {
      prefixes: [
        "castel", "monte", "borgo", "villa", "san", "santa", "rocca", "poggio", "campo",
        "torre", "val", "pieve", "colle", "casal", "porto", "fonte", "prato", "selva",
        "riva", "corte", "sasso", "ponte"
      ],
      suffixes: [
        "nova", "vecchia", "alta", "bassa", "maggiore", "marina", "ano", "ello", "etto",
        "ino", "one", "olo", "aggio", "ese", "ora", "usa", "ino di sopra"
      ],
      preWords: ["san ", "borgo "],
      postWords: [" sul mare", " di sotto", " in valle"],
      preWordChance: 0.05, postWordChance: 0.05
    },

    slaviceast: {
      prefixes: [
        "novo", "staro", "krasno", "verkh", "nizhne", "sredne", "zeleno", "sosno", "bereza",
        "kamen", "gorno", "ust", "ozer", "rechno", "sever", "yuzhno", "svetlo", "lesno",
        "bely", "cherno", "zlato", "solne", "mor", "tikhi", "bystro", "vysoko"
      ],
      suffixes: [
        "gorsk", "grad", "vka", "ovo", "ino", "sk", "borsk", "polye", "zero", "kamensk",
        "gorodok", "slavl", "yarsk", "tinsk", "mensk", "chansk", "vodsk", "lug", "nino"
      ],
      preWords: ["nizhny ", "stary ", "veliky "],
      preWordChance: 0.04, postWordChance: 0
    },

    slavicwest: {
      prefixes: [
        "nowy", "stary", "bialo", "zielona", "dabro", "krasno", "jaro", "lesno", "wodzi",
        "sando", "chelm", "gorzy", "kamien", "ostro", "piotr", "radzi", "sochac", "tarno",
        "wlodo", "zawier", "brze", "myslo", "koby", "gdy", "trze"
      ],
      suffixes: [
        "owice", "ice", "ow", "in", "no", "cin", "sk", "kow", "wola", "gora", "lesna",
        "wice", "czyn", "borz", "mierz", "szyn", "nsk", "czewo"
      ],
      preWords: ["nowy ", "stara "],
      preWordChance: 0.05, postWordChance: 0
    },

    slavicsouth: {
      prefixes: [
        "hrad", "bystr", "novo", "gorna", "dolna", "sveti", "veliko", "malo", "ravna",
        "plan", "kamen", "vodo", "bela", "cerni", "zlat", "breza", "smole", "trebi",
        "moravsk", "lipo", "kruse", "vrba"
      ],
      suffixes: [
        "ica", "ovo", "evo", "ovac", "grad", "brod", "polje", "gora", "vac", "nica",
        "iste", "ovce", "cany", "senik", "slav", "nava", "tice"
      ],
      preWordChance: 0, postWordChance: 0
    },

    baltic: {
      prefixes: [
        "aug", "dau", "jel", "kul", "lie", "mad", "ogre", "rez", "sig", "tuk", "val",
        "vent", "aly", "kau", "mar", "pan", "sia", "tel", "uk", "viln", "salas", "gulb",
        "kras", "aiz", "prei", "smilt", "birz"
      ],
      suffixes: [
        "pils", "kalns", "ciems", "mala", "upe", "lauks", "ai", "is", "as", "ute",
        "unai", "gala", "ava", "ene", "ini", "iai", "enai", "ele"
      ],
      preWordChance: 0, postWordChance: 0
    },

    hungarian: {
      prefixes: [
        "buda", "kis", "nagy", "also", "felso", "uj", "szent", "kek", "feher", "fekete",
        "bala", "csong", "dun", "tisza", "mezo", "hajdu", "kapos", "sopron", "veres",
        "sar", "erd", "hev", "kis kun"
      ],
      suffixes: [
        "var", "falva", "hely", "hida", "hat", "szeg", "ujvaros", "halom", "banya",
        "fured", "kut", "telek", "puszta", "keresztur", "haza", "vasar"
      ],
      preWordChance: 0, postWordChance: 0
    },

    greek: {
      prefixes: [
        "nea", "palaia", "ano", "kato", "agios", "agia", "megalo", "mikro", "kalo",
        "chryso", "aspro", "mavro", "petro", "thermo", "kastro", "elaio", "peuko", "para"
      ],
      suffixes: [
        "chori", "polis", "kastro", "vouni", "potamos", "limni", "valos", "akra", "thea",
        "nisos", "ampelos", "gialos", "rachi", "vrisi"
      ],
      preWordChance: 0, postWordChance: 0
    },

    turkish: {
      prefixes: [
        "yeni", "eski", "kara", "ak", "kizil", "gok", "buyuk", "kucuk", "tas", "deniz",
        "dag", "su", "gul", "cam", "elma", "ayva", "kadi", "sari", "bey", "demir"
      ],
      suffixes: [
        "kent", "hisar", "kale", "koy", "pinar", "dere", "ova", "bel", "yayla", "burun",
        "tepe", "li", "lar", "bag", "cesme", "yurt"
      ],
      preWordChance: 0, postWordChance: 0
    },

    arabic: {
      prefixes: [
        "bir", "ain", "sidi", "bab", "dar", "tell", "wadi", "qasr", "ras", "hammam",
        "souk", "borj", "medina", "beni", "oued", "kef", "jebel", "hay"
      ],
      suffixes: [
        "el bahr", "el kebir", "el jadid", "el qadim", "ia", "an", "oun", "iya", "ane",
        "oua", "at", "in", "es sghir"
      ],
      preWordChance: 0, postWordChance: 0
    },

    afrikaans: {
      prefixes: [
        "bloem", "klip", "vaal", "groot", "klein", "wit", "swart", "rooi", "berg", "rand",
        "port", "mos", "doorn", "kraal", "riet", "olifants", "sout", "steen", "veld"
      ],
      suffixes: [
        "fontein", "stad", "burg", "dal", "rivier", "baai", "kloof", "hoek", "vlei",
        "drift", "kop", "plaas", "sig", "berg", "poort"
      ],
      preWordChance: 0, postWordChance: 0
    },

    japanese: {
      prefixes: [
        "aki", "fuji", "hana", "hira", "ishi", "kami", "kane", "kita", "kuro", "mina",
        "naka", "nishi", "shimo", "shiro", "taka", "tori", "yama", "yoshi", "ao", "matsu",
        "sugi", "take", "hoshi", "tsuki", "umi", "kawa", "sato", "hama"
      ],
      suffixes: [
        "yama", "kawa", "gawa", "mura", "saki", "oka", "hama", "moto", "shima", "tani",
        "sawa", "bashi", "machi", "hara", "no", "da", "be", "zaki", "dai", "ura"
      ],
      preWordChance: 0, postWordChance: 0
    },

    chinese: {
      syllables: [
        "bei", "nan", "dong", "xi", "zhong", "shan", "hai", "jiang", "he", "cheng",
        "zhou", "an", "ping", "yang", "lin", "chang", "xin", "tai", "feng", "ming",
        "hua", "long", "jin", "qing", "shui", "tian", "yun", "bao", "de", "gu"
      ],
      minSyllables: 2, maxSyllables: 3
    },

    korean: {
      syllables: [
        "seo", "nam", "dae", "bu", "san", "gwang", "ju", "in", "cheon", "won",
        "jeon", "mok", "po", "chun", "gang", "yang", "hae", "dong", "cheol", "gyeong"
      ],
      minSyllables: 2, maxSyllables: 3
    },

    indian: {
      prefixes: [
        "nava", "puran", "rama", "krishna", "shiv", "sri", "deva", "chandra", "bhil",
        "jaya", "madhu", "nara", "raj", "sundar", "vira", "hari", "gopal", "amar", "indra"
      ],
      suffixes: [
        "pur", "nagar", "abad", "palli", "garh", "kot", "gram", "wadi", "halli", "kere",
        "bad", "pally", "puram", "khed", "gunta"
      ],
      preWordChance: 0, postWordChance: 0
    },

    malay: {
      prefixes: [
        "kuala", "kampung", "bukit", "sungai", "tanjung", "pulau", "batu", "air", "teluk",
        "gunung", "pasir", "seri", "kota", "jala", "muara", "padang", "cikar", "tegal"
      ],
      suffixes: [
        "baru", "lama", "besar", "kecil", "indah", "jaya", "sari", "mulia", "raya",
        "agung", "timur", "barat", "utara", "selatan", "manis"
      ],
      preWordChance: 0, postWordChance: 0
    },

    antipodean: {
      prefixes: [
        "wagga", "warra", "gunda", "yarra", "bendi", "kurra", "tamba", "mullum", "wonga",
        "birri", "coolang", "mooroo", "nunga", "illa", "karra", "minda", "para", "koro",
        "tara", "whanga", "waka", "roto", "manga", "puke"
      ],
      suffixes: [
        "bah", "mah", "gong", "ra", "na", "dah", "roo", "nya", "mba", "tta", "jong",
        "warra", "rangi", "tahi", "nui", "iti", "toa", "wai"
      ],
      preWordChance: 0, postWordChance: 0
    },

    // Used by any country that has no style of its own.
    generic: {
      syllables: [
        "ma", "co", "lu", "ca", "mi", "mo", "chi", "te", "bi", "de", "da", "me", "ri",
        "ra", "lo", "fi", "par", "mil", "la", "del", "fa", "con", "ti", "pan", "ta",
        "na", "sa", "so", "gri", "li", "col", "per", "mar", "pi", "ro", "nes", "pu", "po"
      ],
      minSyllables: 2, maxSyllables: 3
    }
  },

  /* ===================================================================== *
   *  Countries
   *
   *  weight    how often this one turns up, roughly following how much CW you
   *            actually hear from there
   *  prefixes  callsign prefixes; a digit and 1-3 letters get appended
   *  digits    which digits may follow the prefix (optional, default 0-9);
   *            set it where the real allocation is narrow, eg HB9, VK1-8
   *  style     which made-up-place-name recipe to use
   *  cities    real towns; listing one twice makes it twice as likely
   * ===================================================================== */

  countries: {

    /* --- Europe -------------------------------------------------------- */

    dl: { // Germany
      weight: 9, style: "german",
      prefixes: ["dl", "dl", "dj", "dk", "dh", "df", "do", "dm", "db", "dg", "dc", "dd"],
      cities: [
        "berlin", "berlin", "hamburg", "munich", "cologne", "frankfurt", "stuttgart",
        "duesseldorf", "dortmund", "essen", "leipzig", "bremen", "dresden", "hannover",
        "nuremberg", "duisburg", "bochum", "wuppertal", "bielefeld", "bonn", "muenster",
        "mannheim", "karlsruhe", "wiesbaden", "augsburg", "kiel", "erfurt", "rostock",
        "mainz", "kassel", "freiburg", "jena", "ulm", "trier", "passau", "flensburg"
      ]
    },

    i: { // Italy
      weight: 5, style: "italian",
      prefixes: ["i", "ik", "iz", "iw", "iu", "in", "iv", "is"],
      cities: [
        "rome", "rome", "milan", "milan", "naples", "turin", "palermo", "genoa", "bologna",
        "florence", "bari", "catania", "venice", "verona", "messina", "padua", "trieste",
        "brescia", "parma", "modena", "perugia", "livorno", "cagliari", "pisa", "ancona",
        "lecce", "rimini", "siena", "bergamo", "trento"
      ]
    },

    f: { // France
      weight: 5, style: "french",
      prefixes: ["f", "f", "f", "f", "f", "tm"],
      cities: [
        "paris", "paris", "marseille", "lyon", "toulouse", "nice", "nantes", "montpellier",
        "strasbourg", "bordeaux", "lille", "rennes", "reims", "le havre", "toulon",
        "grenoble", "dijon", "angers", "nimes", "brest", "le mans", "amiens", "tours",
        "limoges", "metz", "besancon", "perpignan", "orleans", "caen", "avignon", "pau",
        "la rochelle", "annecy", "chamonix", "biarritz"
      ]
    },

    g: { // England
      weight: 5, style: "english",
      prefixes: ["g", "g", "m", "m", "2e"],
      cities: [
        "london", "london", "birmingham", "manchester", "leeds", "liverpool", "sheffield",
        "bristol", "leicester", "coventry", "nottingham", "newcastle", "southampton",
        "portsmouth", "plymouth", "derby", "norwich", "york", "oxford", "cambridge",
        "brighton", "exeter", "reading", "luton", "ipswich", "hull", "preston", "blackpool",
        "carlisle", "truro"
      ]
    },

    gm: { // Scotland
      weight: 2, style: "celtic",
      prefixes: ["gm", "mm", "2m"],
      cities: [
        "glasgow", "edinburgh", "aberdeen", "dundee", "inverness", "perth", "stirling",
        "ayr", "oban", "fort william", "elgin", "dumfries", "kirkwall", "lerwick",
        "paisley", "falkirk", "thurso"
      ]
    },

    gw: { // Wales
      weight: 1, style: "celtic",
      prefixes: ["gw", "mw", "2w"],
      cities: [
        "cardiff", "swansea", "newport", "wrexham", "bangor", "aberystwyth", "llandudno",
        "carmarthen", "merthyr tydfil", "caernarfon", "holyhead", "brecon", "conwy"
      ]
    },

    gi: { // Northern Ireland
      weight: 1, style: "celtic",
      prefixes: ["gi", "mi", "2i"],
      cities: [
        "belfast", "londonderry", "lisburn", "newry", "armagh", "bangor", "coleraine",
        "omagh", "enniskillen", "ballymena", "portrush"
      ]
    },

    ei: { // Ireland
      weight: 2, style: "celtic",
      prefixes: ["ei", "ei", "ej"],
      cities: [
        "dublin", "dublin", "cork", "limerick", "galway", "waterford", "dundalk",
        "drogheda", "kilkenny", "sligo", "tralee", "wexford", "ennis", "athlone",
        "killarney", "westport"
      ]
    },

    ea: { // Spain
      weight: 4, style: "iberian", digits: "1234567",
      prefixes: ["ea", "ea", "ea", "eb", "ec", "ed"],
      cities: [
        "madrid", "madrid", "barcelona", "barcelona", "valencia", "seville", "zaragoza",
        "malaga", "murcia", "bilbao", "alicante", "cordoba", "valladolid", "vigo", "gijon",
        "granada", "oviedo", "pamplona", "santander", "salamanca", "san sebastian", "cadiz",
        "toledo", "burgos", "leon", "lleida", "tarragona", "avila", "cuenca"
      ]
    },

    ea8: { // Canary Islands
      weight: 1, style: "iberian", digits: "8",
      prefixes: ["ea", "eb", "ec"],
      cities: [
        "las palmas", "santa cruz", "arrecife", "puerto del rosario", "telde", "arucas",
        "la laguna", "adeje", "los llanos", "maspalomas"
      ]
    },

    ct: { // Portugal
      weight: 2, style: "iberian",
      prefixes: ["ct", "ct", "cr", "cs"],
      cities: [
        "lisbon", "lisbon", "porto", "braga", "coimbra", "aveiro", "faro", "setubal",
        "evora", "guarda", "viseu", "leiria", "viana do castelo", "portimao", "sintra"
      ]
    },

    cu: { // Azores
      weight: 1, style: "iberian",
      prefixes: ["cu"],
      cities: [
        "ponta delgada", "angra do heroismo", "horta", "praia da vitoria", "ribeira grande",
        "lajes", "velas"
      ]
    },

    pa: { // Netherlands
      weight: 3, style: "dutch",
      prefixes: ["pa", "pa", "pd", "pe", "ph", "pc", "pb"],
      cities: [
        "amsterdam", "amsterdam", "rotterdam", "the hague", "utrecht", "eindhoven",
        "tilburg", "groningen", "almere", "breda", "nijmegen", "apeldoorn", "haarlem",
        "enschede", "arnhem", "zwolle", "leiden", "maastricht", "delft", "alkmaar", "texel"
      ]
    },

    on: { // Belgium
      weight: 2, style: "dutch",
      prefixes: ["on", "on", "oo", "ot"],
      cities: [
        "brussels", "brussels", "antwerp", "ghent", "charleroi", "liege", "bruges",
        "namur", "leuven", "mons", "mechelen", "ostend", "hasselt", "kortrijk",
        "tournai", "spa"
      ]
    },

    lx: { // Luxembourg
      weight: 1, style: "french",
      prefixes: ["lx"],
      cities: [
        "luxembourg", "esch sur alzette", "differdange", "dudelange", "ettelbruck",
        "wiltz", "echternach", "vianden"
      ]
    },

    hb: { // Switzerland
      weight: 3, style: "german", digits: "39",
      prefixes: ["hb"],
      cities: [
        "bern", "bern", "zurich", "basel", "geneva", "geneva", "lausanne", "lucerne",
        "st gallen", "winterthur", "lugano", "biel", "thun", "fribourg", "neuchatel",
        "sion", "chur", "davos", "zermatt", "interlaken", "montreux", "aarau", "olten",
        "locarno", "la chaux de fonds", "yverdon"
      ]
    },

    oe: { // Austria
      weight: 2, style: "german", digits: "123456789",
      prefixes: ["oe"],
      cities: [
        "vienna", "vienna", "graz", "linz", "salzburg", "innsbruck", "klagenfurt",
        "villach", "wels", "st polten", "dornbirn", "bregenz", "steyr", "kufstein",
        "zell am see"
      ]
    },

    ok: { // Czechia
      weight: 3, style: "slavicsouth",
      prefixes: ["ok", "ok", "ol"],
      cities: [
        "prague", "prague", "brno", "ostrava", "plzen", "liberec", "olomouc",
        "ceske budejovice", "hradec kralove", "usti nad labem", "pardubice", "zlin",
        "jihlava", "karlovy vary", "tabor", "kolin"
      ]
    },

    om: { // Slovakia
      weight: 2, style: "slavicsouth",
      prefixes: ["om"],
      cities: [
        "bratislava", "bratislava", "kosice", "presov", "zilina", "nitra",
        "banska bystrica", "trnava", "trencin", "martin", "poprad", "michalovce"
      ]
    },

    sp: { // Poland
      weight: 4, style: "slavicwest",
      prefixes: ["sp", "sp", "sq", "so", "sn", "hf", "3z"],
      cities: [
        "warsaw", "warsaw", "krakow", "lodz", "wroclaw", "poznan", "gdansk", "szczecin",
        "bydgoszcz", "lublin", "katowice", "bialystok", "gdynia", "czestochowa", "radom",
        "torun", "kielce", "rzeszow", "olsztyn", "zakopane", "opole", "legnica"
      ]
    },

    oh: { // Finland
      weight: 3, style: "finnish",
      prefixes: ["oh", "oh", "og", "oi"],
      cities: [
        "helsinki", "helsinki", "espoo", "tampere", "vantaa", "oulu", "turku", "jyvaskyla",
        "lahti", "kuopio", "pori", "joensuu", "vaasa", "rovaniemi", "kotka", "hameenlinna",
        "seinajoki", "mikkeli", "kajaani", "kemi"
      ]
    },

    sm: { // Sweden
      weight: 3, style: "nordic",
      prefixes: ["sm", "sm", "sa", "sb", "sc"],
      cities: [
        "stockholm", "stockholm", "gothenburg", "malmo", "uppsala", "vasteras", "orebro",
        "linkoping", "helsingborg", "jonkoping", "norrkoping", "lund", "umea", "gavle",
        "sundsvall", "lulea", "kiruna", "visby", "kalmar", "karlstad", "ostersund"
      ]
    },

    la: { // Norway
      weight: 2, style: "nordic",
      prefixes: ["la", "la", "lb", "lc", "ln"],
      cities: [
        "oslo", "oslo", "bergen", "trondheim", "stavanger", "kristiansand", "tromso",
        "drammen", "bodo", "alesund", "sandnes", "molde", "narvik", "hammerfest",
        "lillehammer", "kirkenes", "svolvaer"
      ]
    },

    oz: { // Denmark
      weight: 2, style: "nordic",
      prefixes: ["oz", "oz", "5q"],
      cities: [
        "copenhagen", "copenhagen", "aarhus", "odense", "aalborg", "esbjerg", "randers",
        "kolding", "horsens", "vejle", "roskilde", "helsingor", "skagen", "ribe"
      ]
    },

    tf: { // Iceland
      weight: 1, style: "nordic",
      prefixes: ["tf"],
      cities: [
        "reykjavik", "reykjavik", "kopavogur", "hafnarfjordur", "akureyri", "keflavik",
        "selfoss", "akranes", "isafjordur", "egilsstadir", "vestmannaeyjar"
      ]
    },

    es: { // Estonia
      weight: 1, style: "finnish",
      prefixes: ["es"],
      cities: [
        "tallinn", "tallinn", "tartu", "narva", "parnu", "kohtla jarve", "viljandi",
        "rakvere", "kuressaare", "haapsalu", "voru"
      ]
    },

    yl: { // Latvia
      weight: 1, style: "baltic",
      prefixes: ["yl"],
      cities: [
        "riga", "riga", "daugavpils", "liepaja", "jelgava", "jurmala", "ventspils",
        "rezekne", "valmiera", "ogre", "tukums", "sigulda", "cesis"
      ]
    },

    ly: { // Lithuania
      weight: 2, style: "baltic",
      prefixes: ["ly"],
      cities: [
        "vilnius", "vilnius", "kaunas", "klaipeda", "siauliai", "panevezys", "alytus",
        "marijampole", "mazeikiai", "jonava", "utena", "telsiai", "palanga"
      ]
    },

    r: { // European Russia
      weight: 6, style: "slaviceast",
      prefixes: ["r", "r", "ra", "rk", "rn", "rv", "rw", "rz", "ua", "ub", "uc", "ud"],
      cities: [
        "moscow", "moscow", "st petersburg", "st petersburg", "nizhny novgorod", "kazan",
        "samara", "rostov on don", "voronezh", "volgograd", "saratov", "krasnodar", "tula",
        "yaroslavl", "ryazan", "lipetsk", "kursk", "bryansk", "tver", "vologda",
        "arkhangelsk", "murmansk", "pskov", "kaliningrad", "sochi", "ufa", "perm",
        "izhevsk", "kirov", "penza", "smolensk"
      ]
    },

    r9: { // Asiatic Russia
      weight: 2, style: "slaviceast", digits: "890",
      prefixes: ["r", "ra", "rk", "rw", "rz", "ua", "ub"],
      cities: [
        "novosibirsk", "omsk", "krasnoyarsk", "irkutsk", "vladivostok", "khabarovsk",
        "chelyabinsk", "yekaterinburg", "tyumen", "tomsk", "barnaul", "kemerovo",
        "ulan ude", "yakutsk", "magadan", "chita", "surgut", "norilsk", "blagoveshchensk",
        "petropavlovsk", "nakhodka"
      ]
    },

    ur: { // Ukraine
      weight: 4, style: "slaviceast",
      prefixes: ["ur", "ur", "us", "ut", "uu", "uy", "ux", "um"],
      cities: [
        "kyiv", "kyiv", "kharkiv", "odesa", "dnipro", "lviv", "zaporizhzhia",
        "kryvyi rih", "mykolaiv", "vinnytsia", "poltava", "chernihiv", "cherkasy", "sumy",
        "zhytomyr", "rivne", "ternopil", "ivano frankivsk", "lutsk", "uzhhorod",
        "chernivtsi", "kherson"
      ]
    },

    ew: { // Belarus
      weight: 1, style: "slaviceast",
      prefixes: ["ew", "eu", "ev"],
      cities: [
        "minsk", "minsk", "gomel", "mogilev", "vitebsk", "grodno", "brest", "bobruisk",
        "baranovichi", "pinsk", "orsha", "mozyr"
      ]
    },

    er: { // Moldova
      weight: 1, style: "slavicsouth",
      prefixes: ["er"],
      cities: [
        "chisinau", "chisinau", "balti", "tiraspol", "bender", "cahul", "ungheni",
        "soroca", "orhei"
      ]
    },

    yo: { // Romania
      weight: 3, style: "slavicsouth",
      prefixes: ["yo", "yo", "yp", "yq", "yr"],
      cities: [
        "bucharest", "bucharest", "cluj napoca", "timisoara", "iasi", "constanta",
        "craiova", "brasov", "galati", "ploiesti", "oradea", "braila", "arad", "pitesti",
        "sibiu", "bacau", "targu mures", "baia mare", "suceava", "sinaia"
      ]
    },

    lz: { // Bulgaria
      weight: 2, style: "slavicsouth",
      prefixes: ["lz"],
      cities: [
        "sofia", "sofia", "plovdiv", "varna", "burgas", "ruse", "stara zagora", "pleven",
        "sliven", "dobrich", "shumen", "pernik", "haskovo", "veliko tarnovo", "bansko"
      ]
    },

    ha: { // Hungary
      weight: 2, style: "hungarian",
      prefixes: ["ha", "ha", "hg"],
      cities: [
        "budapest", "budapest", "debrecen", "szeged", "miskolc", "pecs", "gyor",
        "nyiregyhaza", "kecskemet", "szekesfehervar", "szombathely", "sopron", "eger",
        "veszprem", "kaposvar", "siofok"
      ]
    },

    yt: { // Serbia
      weight: 2, style: "slavicsouth",
      prefixes: ["yt", "yu"],
      cities: [
        "belgrade", "belgrade", "novi sad", "nis", "kragujevac", "subotica", "zrenjanin",
        "pancevo", "cacak", "kraljevo", "leskovac", "valjevo"
      ]
    },

    "9a": { // Croatia
      weight: 2, style: "slavicsouth",
      prefixes: ["9a"],
      cities: [
        "zagreb", "zagreb", "split", "rijeka", "osijek", "zadar", "pula",
        "slavonski brod", "karlovac", "varazdin", "sibenik", "dubrovnik", "sisak"
      ]
    },

    s5: { // Slovenia
      weight: 2, style: "slavicsouth",
      prefixes: ["s5"],
      cities: [
        "ljubljana", "ljubljana", "maribor", "celje", "kranj", "velenje", "koper",
        "novo mesto", "ptuj", "bled", "nova gorica"
      ]
    },

    e7: { // Bosnia and Herzegovina
      weight: 1, style: "slavicsouth",
      prefixes: ["e7"],
      cities: [
        "sarajevo", "sarajevo", "banja luka", "tuzla", "zenica", "mostar", "bihac",
        "brcko", "prijedor", "doboj"
      ]
    },

    z3: { // North Macedonia
      weight: 1, style: "slavicsouth",
      prefixes: ["z3"],
      cities: [
        "skopje", "skopje", "bitola", "kumanovo", "prilep", "tetovo", "ohrid", "veles",
        "stip", "gostivar"
      ]
    },

    "4o": { // Montenegro
      weight: 1, style: "slavicsouth",
      prefixes: ["4o"],
      cities: [
        "podgorica", "niksic", "herceg novi", "budva", "bar", "cetinje", "kotor",
        "tivat", "pljevlja"
      ]
    },

    za: { // Albania
      weight: 1, style: "generic",
      prefixes: ["za"],
      cities: [
        "tirana", "tirana", "durres", "vlore", "shkoder", "elbasan", "korce", "fier",
        "berat", "gjirokaster", "saranda"
      ]
    },

    sv: { // Greece
      weight: 2, style: "greek", digits: "123456789",
      prefixes: ["sv", "sv", "sw", "sy", "sz"],
      cities: [
        "athens", "athens", "thessaloniki", "patras", "heraklion", "larissa", "volos",
        "ioannina", "kavala", "chania", "rhodes", "corfu", "kalamata", "serres", "drama",
        "mytilene", "sparta"
      ]
    },

    "9h": { // Malta
      weight: 1, style: "italian",
      prefixes: ["9h"],
      cities: [
        "valletta", "sliema", "birkirkara", "mosta", "qormi", "rabat", "victoria",
        "marsaxlokk", "mellieha"
      ]
    },

    ta: { // Turkey
      weight: 2, style: "turkish",
      prefixes: ["ta", "ta", "tb", "tc"],
      cities: [
        "istanbul", "istanbul", "ankara", "izmir", "bursa", "antalya", "adana", "konya",
        "gaziantep", "mersin", "kayseri", "eskisehir", "samsun", "trabzon", "erzurum",
        "denizli", "malatya", "van", "bodrum", "izmit"
      ]
    },

    "5b": { // Cyprus
      weight: 1, style: "greek",
      prefixes: ["5b", "c4", "h2"],
      cities: [
        "nicosia", "limassol", "larnaca", "paphos", "famagusta", "kyrenia", "ayia napa",
        "polis"
      ]
    },

    "4l": { // Georgia
      weight: 1, style: "generic",
      prefixes: ["4l"],
      cities: [
        "tbilisi", "tbilisi", "batumi", "kutaisi", "rustavi", "poti", "zugdidi", "gori",
        "telavi"
      ]
    },

    un: { // Kazakhstan
      weight: 1, style: "slaviceast",
      prefixes: ["un", "uo", "up", "uq"],
      cities: [
        "almaty", "almaty", "astana", "shymkent", "karaganda", "aktobe", "taraz",
        "pavlodar", "semey", "atyrau", "kostanay", "oskemen", "aktau"
      ]
    },

    /* --- Africa and the Middle East ------------------------------------ */

    "4x": { // Israel
      weight: 1, style: "generic",
      prefixes: ["4x", "4z"],
      cities: [
        "jerusalem", "jerusalem", "tel aviv", "haifa", "beer sheva", "netanya", "ashdod",
        "rishon lezion", "eilat", "tiberias", "nazareth"
      ]
    },

    a6: { // United Arab Emirates
      weight: 1, style: "arabic",
      prefixes: ["a6"],
      cities: [
        "dubai", "dubai", "abu dhabi", "sharjah", "al ain", "ajman", "fujairah",
        "ras al khaimah"
      ]
    },

    cn: { // Morocco
      weight: 1, style: "arabic",
      prefixes: ["cn", "5c"],
      cities: [
        "casablanca", "casablanca", "rabat", "marrakech", "fes", "tangier", "agadir",
        "meknes", "oujda", "tetouan", "essaouira", "kenitra"
      ]
    },

    "7x": { // Algeria
      weight: 1, style: "arabic",
      prefixes: ["7x"],
      cities: [
        "algiers", "algiers", "oran", "constantine", "annaba", "blida", "batna", "setif",
        "tlemcen", "bejaia", "ghardaia"
      ]
    },

    "3v": { // Tunisia
      weight: 1, style: "arabic",
      prefixes: ["3v", "ts"],
      cities: [
        "tunis", "tunis", "sfax", "sousse", "kairouan", "bizerte", "gabes", "monastir",
        "nabeul", "tozeur"
      ]
    },

    su: { // Egypt
      weight: 1, style: "arabic",
      prefixes: ["su"],
      cities: [
        "cairo", "cairo", "alexandria", "giza", "port said", "suez", "luxor", "aswan",
        "tanta", "ismailia", "hurghada"
      ]
    },

    zs: { // South Africa
      weight: 2, style: "afrikaans",
      prefixes: ["zs", "zs", "zr", "zu"],
      cities: [
        "cape town", "cape town", "johannesburg", "durban", "pretoria", "port elizabeth",
        "bloemfontein", "east london", "nelspruit", "kimberley", "polokwane", "george",
        "stellenbosch", "knysna", "upington"
      ]
    },

    /* --- The Americas -------------------------------------------------- */

    k: { // United States
      weight: 9, style: "english",
      prefixes: [
        "k", "k", "w", "w", "n", "n", "aa", "ab", "ac", "ad", "ae", "ka", "kb", "kc",
        "kd", "ke", "kf", "kg", "ki", "kk", "na", "nc", "nd", "ne", "ni", "wa", "wb",
        "wd", "we", "wg"
      ],
      cities: [
        "new york", "new york", "chicago", "chicago", "los angeles", "houston", "phoenix",
        "philadelphia", "san antonio", "san diego", "dallas", "austin", "san jose",
        "jacksonville", "columbus", "charlotte", "indianapolis", "seattle", "denver",
        "boston", "nashville", "portland", "las vegas", "detroit", "memphis", "baltimore",
        "milwaukee", "albuquerque", "tucson", "fresno", "sacramento", "kansas city",
        "atlanta", "omaha", "raleigh", "miami", "cleveland", "tulsa", "wichita",
        "new orleans", "honolulu", "anchorage", "boise", "spokane", "dayton", "bangor"
      ]
    },

    ve: { // Canada
      weight: 3, style: "english",
      prefixes: ["ve", "ve", "va", "vo", "vy"],
      cities: [
        "toronto", "toronto", "montreal", "vancouver", "calgary", "edmonton", "ottawa",
        "winnipeg", "quebec city", "hamilton", "kitchener", "halifax", "victoria",
        "saskatoon", "regina", "st johns", "sudbury", "thunder bay", "kelowna",
        "whitehorse", "yellowknife", "moncton", "fredericton", "charlottetown"
      ]
    },

    kp4: { // Puerto Rico
      weight: 1, style: "iberian", digits: "4",
      prefixes: ["kp", "np", "wp"],
      cities: [
        "san juan", "san juan", "ponce", "mayaguez", "caguas", "arecibo", "bayamon",
        "aguadilla"
      ]
    },

    xe: { // Mexico
      weight: 1, style: "iberian",
      prefixes: ["xe", "xf"],
      cities: [
        "mexico city", "mexico city", "guadalajara", "monterrey", "puebla", "tijuana",
        "leon", "merida", "queretaro", "cancun", "veracruz", "oaxaca", "chihuahua",
        "morelia"
      ]
    },

    py: { // Brazil
      weight: 3, style: "iberian",
      prefixes: ["py", "py", "pu", "pp", "pt", "pr", "ps", "pv", "pw"],
      cities: [
        "sao paulo", "sao paulo", "rio de janeiro", "rio de janeiro", "brasilia",
        "salvador", "fortaleza", "belo horizonte", "manaus", "curitiba", "recife",
        "porto alegre", "belem", "goiania", "campinas", "sao luis", "natal",
        "florianopolis", "santos", "joao pessoa"
      ]
    },

    lu: { // Argentina
      weight: 2, style: "iberian", digits: "1234567",
      prefixes: ["lu", "lu", "lw"],
      cities: [
        "buenos aires", "buenos aires", "cordoba", "rosario", "mendoza", "la plata",
        "tucuman", "mar del plata", "salta", "santa fe", "san juan", "resistencia",
        "neuquen", "bahia blanca", "ushuaia"
      ]
    },

    ce: { // Chile
      weight: 1, style: "iberian", digits: "1234567",
      prefixes: ["ce", "ca", "xq"],
      cities: [
        "santiago", "santiago", "valparaiso", "concepcion", "antofagasta", "vina del mar",
        "temuco", "iquique", "la serena", "puerto montt", "arica", "punta arenas",
        "valdivia"
      ]
    },

    cx: { // Uruguay
      weight: 1, style: "iberian",
      prefixes: ["cx"],
      cities: [
        "montevideo", "montevideo", "salto", "paysandu", "maldonado", "rivera",
        "punta del este", "colonia", "tacuarembo"
      ]
    },

    hk: { // Colombia
      weight: 1, style: "iberian",
      prefixes: ["hk", "hj"],
      cities: [
        "bogota", "bogota", "medellin", "cali", "barranquilla", "cartagena", "cucuta",
        "bucaramanga", "pereira", "manizales", "santa marta"
      ]
    },

    yv: { // Venezuela
      weight: 1, style: "iberian",
      prefixes: ["yv", "yy"],
      cities: [
        "caracas", "caracas", "maracaibo", "valencia", "barquisimeto", "maracay",
        "ciudad guayana", "merida", "san cristobal"
      ]
    },

    oa: { // Peru
      weight: 1, style: "iberian",
      prefixes: ["oa", "ob", "oc"],
      cities: [
        "lima", "lima", "arequipa", "trujillo", "chiclayo", "cusco", "piura", "iquitos",
        "huancayo", "tacna"
      ]
    },

    co: { // Cuba
      weight: 1, style: "iberian",
      prefixes: ["co", "cm"],
      cities: [
        "havana", "havana", "santiago de cuba", "camaguey", "holguin", "santa clara",
        "cienfuegos", "matanzas", "varadero"
      ]
    },

    /* --- Asia and the Pacific ------------------------------------------ */

    ja: { // Japan
      weight: 6, style: "japanese",
      prefixes: [
        "ja", "ja", "je", "jf", "jg", "jh", "ji", "jj", "jk", "jl", "jm", "jn", "jo",
        "jp", "jq", "jr", "js", "7k", "7l", "7m", "7n"
      ],
      cities: [
        "tokyo", "tokyo", "yokohama", "osaka", "osaka", "nagoya", "sapporo", "fukuoka",
        "kobe", "kyoto", "kawasaki", "saitama", "hiroshima", "sendai", "chiba",
        "kitakyushu", "sakai", "niigata", "hamamatsu", "kumamoto", "okayama", "shizuoka",
        "kanazawa", "nagasaki", "matsuyama", "nara", "naha", "aomori", "akita"
      ]
    },

    by: { // China
      weight: 2, style: "chinese",
      prefixes: ["by", "bg", "bh", "bd", "ba", "bi"],
      cities: [
        "beijing", "beijing", "shanghai", "shanghai", "guangzhou", "shenzhen", "chengdu",
        "wuhan", "xian", "hangzhou", "chongqing", "tianjin", "nanjing", "shenyang",
        "qingdao", "dalian", "jinan", "harbin", "kunming", "xiamen", "suzhou", "changsha"
      ]
    },

    bv: { // Taiwan
      weight: 1, style: "chinese",
      prefixes: ["bv", "bm", "bu", "bx"],
      cities: [
        "taipei", "taipei", "kaohsiung", "taichung", "tainan", "hsinchu", "keelung",
        "chiayi", "hualien", "taitung"
      ]
    },

    hl: { // South Korea
      weight: 1, style: "korean",
      prefixes: ["hl", "ds", "dt"],
      cities: [
        "seoul", "seoul", "busan", "incheon", "daegu", "daejeon", "gwangju", "ulsan",
        "suwon", "jeonju", "changwon", "jeju", "chuncheon"
      ]
    },

    vu: { // India
      weight: 2, style: "indian",
      prefixes: ["vu", "at"],
      cities: [
        "mumbai", "mumbai", "delhi", "delhi", "bangalore", "hyderabad", "chennai",
        "kolkata", "pune", "ahmedabad", "jaipur", "lucknow", "kanpur", "nagpur", "indore",
        "bhopal", "patna", "kochi", "coimbatore", "visakhapatnam", "mysore", "goa"
      ]
    },

    "9m": { // Malaysia
      weight: 1, style: "malay",
      prefixes: ["9m", "9w"],
      cities: [
        "kuala lumpur", "kuala lumpur", "penang", "ipoh", "johor bahru", "kuching",
        "kota kinabalu", "melaka", "shah alam", "seremban", "alor setar"
      ]
    },

    yb: { // Indonesia
      weight: 1, style: "malay",
      prefixes: ["yb", "yc", "yd", "yf", "yg"],
      cities: [
        "jakarta", "jakarta", "surabaya", "bandung", "medan", "semarang", "palembang",
        "makassar", "denpasar", "yogyakarta", "malang", "padang", "manado", "balikpapan"
      ]
    },

    hs: { // Thailand
      weight: 1, style: "generic",
      prefixes: ["hs", "e2"],
      cities: [
        "bangkok", "bangkok", "chiang mai", "phuket", "pattaya", "khon kaen", "hat yai",
        "udon thani", "nakhon ratchasima", "ayutthaya"
      ]
    },

    du: { // Philippines
      weight: 1, style: "malay",
      prefixes: ["du", "dv", "dw", "dy", "dz", "4f"],
      cities: [
        "manila", "manila", "quezon city", "cebu", "davao", "makati", "iloilo", "baguio",
        "bacolod", "cagayan de oro", "zamboanga"
      ]
    },

    vk: { // Australia
      weight: 3, style: "antipodean", digits: "12345678",
      prefixes: ["vk", "vk", "vi"],
      cities: [
        "sydney", "sydney", "melbourne", "melbourne", "brisbane", "perth", "adelaide",
        "canberra", "hobart", "darwin", "newcastle", "wollongong", "geelong",
        "townsville", "cairns", "toowoomba", "ballarat", "bendigo", "launceston",
        "alice springs", "broome"
      ]
    },

    zl: { // New Zealand
      weight: 2, style: "antipodean", digits: "1234",
      prefixes: ["zl", "zl", "zm"],
      cities: [
        "auckland", "auckland", "wellington", "christchurch", "hamilton", "tauranga",
        "dunedin", "palmerston north", "napier", "nelson", "rotorua", "invercargill",
        "whangarei", "queenstown", "gisborne"
      ]
    }
  }
};
