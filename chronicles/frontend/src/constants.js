// ─── XP Thresholds ────────────────────────────────────────────────────────────

export const XP_THRESHOLDS = [
    0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000,
    85000, 100000, 120000, 140000, 165000, 195000, 225000, 265000, 305000, 355000
]

// ─── Stats ────────────────────────────────────────────────────────────────────

export const STAT_KEYS = ['str', 'dex', 'con', 'int', 'wis', 'cha']
export const STAT_LABELS = { str: 'STR', dex: 'DEX', con: 'CON', int: 'INT', wis: 'WIS', cha: 'CHA' }
export const STAT_LABELS_ARRAY = ['STR', 'DEX', 'CON', 'INT', 'WIS', 'CHA']

// ─── Sex ─────────────────────────────────────────────────────────────────────

export const SEX_OPTIONS = ['Male', 'Female']

// ─── Classes ──────────────────────────────────────────────────────────────────

export const CLASSES = [
    'Barbarian', 'Bard', 'Cleric', 'Druid', 'Fighter',
    'Monk', 'Paladin', 'Ranger', 'Rogue', 'Sorcerer', 'Warlock', 'Wizard'
]

// ─── Species ──────────────────────────────────────────────────────────────────

export const SPECIES = [
    {
        name: 'Aasimar',
        desc: 'Mortals who carry a spark of the Upper Planes. Celestial features, healing powers, and radiant transformations.',
        subtype: null,
    },
    {
        name: 'Dragonborn',
        desc: 'Descended from dragons. Breath weapon, damage resistance, and draconic flight at level 5.',
        subtype: {
            label: 'Draconic Ancestry',
            options: [
                { name: 'Black', desc: 'Acid damage' },
                { name: 'Blue', desc: 'Lightning damage' },
                { name: 'Brass', desc: 'Fire damage' },
                { name: 'Bronze', desc: 'Lightning damage' },
                { name: 'Copper', desc: 'Acid damage' },
                { name: 'Gold', desc: 'Fire damage' },
                { name: 'Green', desc: 'Poison damage' },
                { name: 'Red', desc: 'Fire damage' },
                { name: 'Silver', desc: 'Cold damage' },
                { name: 'White', desc: 'Cold damage' },
            ]
        }
    },
    {
        name: 'Dwarf',
        desc: 'Hardy folk of the mountains. Darkvision, poison resistance, extra HP each level, and Stonecunning.',
        subtype: null,
    },
    {
        name: 'Elf',
        desc: 'Ancient and graceful. Darkvision, Fey Ancestry, Trance, and lineage-based innate magic.',
        subtype: {
            label: 'Elven Lineage',
            options: [
                { name: 'Drow', desc: 'Extended darkvision, Dancing Lights, Faerie Fire, Darkness' },
                { name: 'High Elf', desc: 'Prestidigitation cantrip, Detect Magic, Misty Step' },
                { name: 'Wood Elf', desc: 'Speed +5, Druidcraft cantrip, Longstrider, Pass without Trace' },
                { name: 'Astral Elf', desc: 'Radiant resistance, Starlight Step teleportation, Radiant Soul' },
            ]
        }
    },
    {
        name: 'Gnome',
        desc: 'Small and clever. Darkvision, Gnomish Cunning, and lineage-based magical abilities.',
        subtype: {
            label: 'Gnomish Lineage',
            options: [
                { name: 'Forest Gnome', desc: 'Minor Illusion, Speak with Animals (proficiency bonus times per Long Rest)' },
                { name: 'Rock Gnome', desc: 'Mending and Prestidigitation cantrips, create clockwork devices' },
            ]
        }
    },
    {
        name: 'Goliath',
        desc: 'Distant descendants of giants. Speed 35, supernatural giant ancestry boon, and Large Form at level 5.',
        subtype: {
            label: 'Giant Ancestry',
            options: [
                { name: 'Cloud Giant', desc: "Cloud's Jaunt — teleport up to 30 feet as a Bonus Action" },
                { name: 'Fire Giant', desc: "Fire's Burn — deal 1d10 extra Fire damage on a hit" },
                { name: 'Frost Giant', desc: "Frost's Chill — deal 1d6 Cold and reduce target Speed on a hit" },
                { name: 'Hill Giant', desc: "Hill's Tumble — knock Large or smaller targets Prone on a hit" },
                { name: 'Stone Giant', desc: "Stone's Endurance — reduce incoming damage by 1d12 + CON mod as a Reaction" },
                { name: 'Storm Giant', desc: "Storm's Thunder — deal 1d8 Thunder to a creature that damages you" },
            ]
        }
    },
    {
        name: 'Half-Elf',
        desc: 'Born of human and elven heritage. Darkvision, Fey Ancestry, proficiency in two skills of choice, and a trait inherited from their elven lineage.',
        subtype: {
            label: 'Elven Heritage',
            options: [
                { name: 'High Elf Heritage', desc: 'You know the Prestidigitation cantrip from your high elf blood.' },
                { name: 'Wood Elf Heritage', desc: 'Your Speed increases to 35 feet from your wood elf blood.' },
                { name: 'Drow Heritage', desc: 'Your Darkvision range increases to 120 feet from your drow blood.' },
            ]
        }
    },
    {
        name: 'Halfling',
        desc: 'Small and surprisingly lucky. Brave, nimble, naturally stealthy, and can reroll any 1 on a d20.',
        subtype: null,
    },
    {
        name: 'Human',
        desc: 'Resourceful and versatile. Gain Heroic Inspiration on Long Rests, a skill proficiency, and an Origin feat.',
        subtype: null,
    },
    {
        name: 'Orc',
        desc: 'Enduring and powerful. Darkvision 120 ft, Adrenaline Rush for bonus Dash + temp HP, Relentless Endurance.',
        subtype: null,
    },
    {
        name: 'Tiefling',
        desc: 'Touched by fiendish power. Darkvision, Thaumaturgy cantrip, fiendish legacy with innate spells and resistance.',
        subtype: {
            label: 'Fiendish Legacy',
            options: [
                { name: 'Abyssal', desc: 'Poison resistance, Poison Spray, Ray of Sickness, Hold Person' },
                { name: 'Chthonic', desc: 'Necrotic resistance, Chill Touch, False Life, Ray of Enfeeblement' },
                { name: 'Infernal', desc: 'Fire resistance, Fire Bolt, Hellish Rebuke, Darkness' },
            ]
        }
    },
]

// ─── Backgrounds ──────────────────────────────────────────────────────────────

export const BACKGROUNDS = [
    {
        name: 'Acolyte',
        desc: 'You devoted yourself to service in a temple, studying religion and learning to channel divine power.',
        feat: 'Magic Initiate (Cleric)',
        skills: ['Insight', 'Religion'],
        tool: "Calligrapher's Supplies",
        asi_stats: ['int', 'wis', 'cha'],
    },
    {
        name: 'Artisan',
        desc: 'You apprenticed in a workshop, learning to craft goods and sweet-talk demanding customers.',
        feat: 'Crafter',
        skills: ['Investigation', 'Persuasion'],
        tool: "Artisan's Tools (your choice)",
        asi_stats: ['str', 'dex', 'int'],
    },
    {
        name: 'Charlatan',
        desc: 'You learned to prey on unfortunates with comforting lies, sham potions, and forged documents.',
        feat: 'Skilled',
        skills: ['Deception', 'Sleight of Hand'],
        tool: 'Forgery Kit',
        asi_stats: ['dex', 'con', 'cha'],
    },
    {
        name: 'Criminal',
        desc: 'You eked out a living in dark alleyways, cutting purses and burgling shops.',
        feat: 'Alert',
        skills: ['Sleight of Hand', 'Stealth'],
        tool: "Thieves' Tools",
        asi_stats: ['dex', 'con', 'int'],
    },
    {
        name: 'Entertainer',
        desc: 'You followed roving fairs and carnivals, performing for crowds and thriving on applause.',
        feat: 'Musician',
        skills: ['Acrobatics', 'Performance'],
        tool: 'Musical Instrument (your choice)',
        asi_stats: ['str', 'dex', 'cha'],
    },
    {
        name: 'Farmer',
        desc: 'You grew up close to the land, tending animals and cultivating the earth.',
        feat: 'Tough',
        skills: ['Animal Handling', 'Nature'],
        tool: "Carpenter's Tools",
        asi_stats: ['str', 'con', 'wis'],
    },
    {
        name: 'Guard',
        desc: 'You spent countless hours at your post, watching for threats from without and within.',
        feat: 'Alert',
        skills: ['Athletics', 'Perception'],
        tool: 'Gaming Set (your choice)',
        asi_stats: ['str', 'int', 'wis'],
    },
    {
        name: 'Guide',
        desc: 'You came of age outdoors, exploring wildernesses and guiding nature priests.',
        feat: 'Magic Initiate (Druid)',
        skills: ['Stealth', 'Survival'],
        tool: "Cartographer's Tools",
        asi_stats: ['dex', 'con', 'wis'],
    },
    {
        name: 'Hermit',
        desc: 'You spent your early years secluded in a hut or monastery, pondering the mysteries of creation.',
        feat: 'Healer',
        skills: ['Medicine', 'Religion'],
        tool: 'Herbalism Kit',
        asi_stats: ['con', 'wis', 'cha'],
    },
    {
        name: 'Merchant',
        desc: 'You were apprenticed to a trader, learning commerce and traveling broadly to buy and sell goods.',
        feat: 'Lucky',
        skills: ['Animal Handling', 'Persuasion'],
        tool: "Navigator's Tools",
        asi_stats: ['con', 'int', 'cha'],
    },
    {
        name: 'Noble',
        desc: 'You were raised in a castle, surrounded by wealth, power, and privilege.',
        feat: 'Skilled',
        skills: ['History', 'Persuasion'],
        tool: 'Gaming Set (your choice)',
        asi_stats: ['str', 'int', 'cha'],
    },
    {
        name: 'Sage',
        desc: 'You traveled between manors and monasteries studying books and scrolls, learning the lore of the multiverse.',
        feat: 'Magic Initiate (Wizard)',
        skills: ['Arcana', 'History'],
        tool: "Calligrapher's Supplies",
        asi_stats: ['con', 'int', 'wis'],
    },
    {
        name: 'Sailor',
        desc: 'You lived as a seafarer, facing mighty storms and swapping stories in ports of call.',
        feat: 'Tavern Brawler',
        skills: ['Acrobatics', 'Perception'],
        tool: "Navigator's Tools",
        asi_stats: ['str', 'dex', 'wis'],
    },
    {
        name: 'Scribe',
        desc: 'You spent formative years in a scriptorium, learning to write clearly and produce finely crafted texts.',
        feat: 'Skilled',
        skills: ['Investigation', 'Perception'],
        tool: "Calligrapher's Supplies",
        asi_stats: ['dex', 'int', 'wis'],
    },
    {
        name: 'Soldier',
        desc: 'You trained for war as soon as you reached adulthood. Battle is in your blood.',
        feat: 'Savage Attacker',
        skills: ['Athletics', 'Intimidation'],
        tool: 'Gaming Set (your choice)',
        asi_stats: ['str', 'dex', 'con'],
    },
    {
        name: 'Wayfarer',
        desc: 'You grew up on the streets, surviving by odd jobs and occasional theft, never losing your pride.',
        feat: 'Lucky',
        skills: ['Insight', 'Stealth'],
        tool: "Thieves' Tools",
        asi_stats: ['dex', 'wis', 'cha'],
    },
]

// ─── Equipment packages ───────────────────────────────────────────────────────

export const CLASS_EQUIPMENT = {
    Barbarian: [
        { label: 'A', desc: 'Greataxe, 4 Handaxes, Explorer\'s Pack, 15 GP' },
        { label: 'B', desc: '75 GP' },
    ],
    Bard: [
        { label: 'A', desc: 'Leather Armor, 2 Daggers, Musical Instrument, Entertainer\'s Pack, 19 GP' },
        { label: 'B', desc: '90 GP' },
    ],
    Cleric: [
        { label: 'A', desc: 'Chain Shirt, Shield, Mace, Holy Symbol, Priest\'s Pack, 7 GP' },
        { label: 'B', desc: '110 GP' },
    ],
    Druid: [
        { label: 'A', desc: 'Leather Armor, Shield, Sickle, Druidic Focus (Quarterstaff), Explorer\'s Pack, Herbalism Kit, 9 GP' },
        { label: 'B', desc: '50 GP' },
    ],
    Fighter: [
        { label: 'A', desc: 'Chain Mail, Greatsword, Flail, 8 Javelins, Dungeoneer\'s Pack, 4 GP' },
        { label: 'B', desc: 'Studded Leather, Scimitar, Shortsword, Longbow, 20 Arrows, Quiver, Dungeoneer\'s Pack, 11 GP' },
        { label: 'C', desc: '155 GP' },
    ],
    Monk: [
        { label: 'A', desc: 'Spear, 5 Daggers, Artisan\'s Tools or Musical Instrument, Explorer\'s Pack, 11 GP' },
        { label: 'B', desc: '50 GP' },
    ],
    Paladin: [
        { label: 'A', desc: 'Chain Mail, Shield, Longsword, 6 Javelins, Holy Symbol, Priest\'s Pack, 9 GP' },
        { label: 'B', desc: '150 GP' },
    ],
    Ranger: [
        { label: 'A', desc: 'Studded Leather, Scimitar, Shortsword, Longbow, 20 Arrows, Quiver, Druidic Focus, Explorer\'s Pack, 7 GP' },
        { label: 'B', desc: '150 GP' },
    ],
    Rogue: [
        { label: 'A', desc: 'Leather Armor, 2 Daggers, Shortsword, Shortbow, 20 Arrows, Quiver, Thieves\' Tools, Burglar\'s Pack, 8 GP' },
        { label: 'B', desc: '100 GP' },
    ],
    Sorcerer: [
        { label: 'A', desc: 'Spear, 2 Daggers, Arcane Focus (crystal), Dungeoneer\'s Pack, 28 GP' },
        { label: 'B', desc: '50 GP' },
    ],
    Warlock: [
        { label: 'A', desc: 'Leather Armor, Sickle, 2 Daggers, Arcane Focus (orb), Book of Occult Lore, Scholar\'s Pack, 15 GP' },
        { label: 'B', desc: '100 GP' },
    ],
    Wizard: [
        { label: 'A', desc: '2 Daggers, Arcane Focus (Quarterstaff), Robe, Spellbook, Scholar\'s Pack, 5 GP' },
        { label: 'B', desc: '55 GP' },
    ],
}

// ─── Fighter ──────────────────────────────────────────────────────────────────

export const FIGHTER_SUBCLASSES = [
    {
        name: 'Champion',
        desc: 'Focused on physical excellence. Expanded critical hit range, Remarkable Athlete, and exceptional resilience.'
    },
    {
        name: 'Battle Master',
        desc: 'Master of combat maneuvers. Superiority Dice fuel powerful tactical techniques in every fight.'
    },
    {
        name: 'Psi Warrior',
        desc: 'Augments martial might with psionic power. Telekinetic strikes, protective fields, and mental force.'
    },
    {
        name: 'Eldritch Knight',
        desc: 'Combines martial skill with arcane magic. Wizard spells and War Bond enhance combat capabilities.',
        coming_soon: true
    },
]

export const FIGHTER_BASE_FEATURES = {
    1: ['Fighting Style', 'Second Wind', 'Weapon Mastery'],
    2: ['Action Surge', 'Tactical Mind'],
    3: ['Fighter Subclass'],
    4: ['Ability Score Improvement'],
    5: ['Extra Attack', 'Tactical Shift'],
    6: ['Ability Score Improvement'],
    7: ['Subclass Feature'],
    8: ['Ability Score Improvement'],
    9: ['Indomitable', 'Tactical Master'],
    10: ['Subclass Feature'],
    11: ['Two Extra Attacks'],
    12: ['Ability Score Improvement'],
    13: ['Indomitable (two uses)', 'Studied Attacks'],
    14: ['Ability Score Improvement'],
    15: ['Subclass Feature'],
    16: ['Ability Score Improvement'],
    17: ['Action Surge (two uses)', 'Indomitable (three uses)'],
    18: ['Subclass Feature'],
    19: ['Epic Boon'],
    20: ['Three Extra Attacks'],
}

export const FIGHTER_SUBCLASS_FEATURES = {
    Champion: {
        3: ['Improved Critical', 'Remarkable Athlete'],
        7: ['Additional Fighting Style'],
        10: ['Heroic Warrior'],
        15: ['Superior Critical'],
        18: ['Survivor'],
    },
    'Battle Master': {
        3: ['Combat Superiority', 'Student of War'],
        7: ['Know Your Enemy'],
        10: ['Improved Combat Superiority (d10)'],
        15: ['Relentless', 'Improved Combat Superiority (d12)'],
        18: ['Ultimate Combat Superiority'],
    },
    'Psi Warrior': {
        3: ['Psionic Power'],
        7: ['Telekinetic Adept'],
        10: ['Guarded Mind'],
        15: ['Bulwark of Force'],
        18: ['Telekinetic Master'],
    },
}

export const FIGHTER_ASI_LEVELS = [4, 6, 8, 12, 14, 16]

export const ALL_MANEUVERS = [
    'Ambush',
    'Bait and Switch',
    "Commander's Strike",
    'Commanding Presence',
    'Disarming Attack',
    'Distracting Strike',
    'Evasive Footwork',
    'Feinting Attack',
    'Goading Attack',
    'Lunging Attack',
    'Maneuvering Attack',
    'Menacing Attack',
    'Parry',
    'Precision Attack',
    'Pushing Attack',
    'Rally',
    'Riposte',
    'Sweeping Attack',
    'Tactical Assessment',
    'Trip Attack',
]

export const MASTERY_PROPERTIES = [
    'cleave', 'graze', 'nick', 'push', 'sap', 'slow', 'topple', 'vex'
]

// ─── Barbarian ────────────────────────────────────────────────────────────────
 
export const BARBARIAN_SUBCLASSES = [
    {
        name: 'Path of the Berserker',
        desc: 'Channel Rage into violent fury. Frenzy adds bonus d6 damage while raging, and Retaliation lets you strike back instantly when hit.',
    },
    {
        name: 'Path of the Wild Heart',
        desc: 'Walk in community with the animal world. Choose a beast aspect on each Rage — Bear for broad resistance, Eagle for mobility, Wolf to support allies.',
    },
    {
        name: 'Path of the World Tree',
        desc: 'Trace the roots and branches of Yggdrasil. Surge with temporary hit points, teleport enemies, and eventually travel vast distances in an instant.',
    },
    {
        name: 'Path of the Zealot',
        desc: 'Rage in ecstatic union with a god. Channel divine fury into your strikes and carry a personal healing pool that keeps you in the fight.',
    },
]
 
export const BARBARIAN_BASE_FEATURES = {
    1:  ['Rage', 'Unarmored Defense', 'Weapon Mastery'],
    2:  ['Danger Sense', 'Reckless Attack'],
    3:  ['Barbarian Subclass', 'Primal Knowledge'],
    4:  ['Ability Score Improvement'],
    5:  ['Extra Attack', 'Fast Movement'],
    6:  ['Subclass Feature'],
    7:  ['Feral Instinct', 'Instinctive Pounce'],
    8:  ['Ability Score Improvement'],
    9:  ['Brutal Strike'],
    10: ['Subclass Feature'],
    11: ['Relentless Rage'],
    12: ['Ability Score Improvement'],
    13: ['Improved Brutal Strike'],
    14: ['Subclass Feature'],
    15: ['Persistent Rage'],
    16: ['Ability Score Improvement'],
    17: ['Improved Brutal Strike (upgrade)'],
    18: ['Indomitable Might'],
    19: ['Epic Boon'],
    20: ['Primal Champion'],
}
 
export const BARBARIAN_SUBCLASS_FEATURES = {
    'Path of the Berserker': {
        3:  ['Frenzy'],
        6:  ['Mindless Rage'],
        10: ['Retaliation'],
        14: ['Intimidating Presence'],
    },
    'Path of the Wild Heart': {
        3:  ['Animal Speaker', 'Rage of the Wilds'],
        6:  ['Aspect of the Wilds'],
        10: ['Nature Speaker'],
        14: ['Power of the Wilds'],
    },
    'Path of the World Tree': {
        3:  ['Vitality of the Tree'],
        6:  ['Branches of the Tree'],
        10: ['Battering Roots'],
        14: ['Travel along the Tree'],
    },
    'Path of the Zealot': {
        3:  ['Divine Fury', 'Warrior of the Gods'],
        6:  ['Fanatical Focus'],
        10: ['Zealous Presence'],
        14: ['Rage of the Gods'],
    },
}
 
export const BARBARIAN_ASI_LEVELS = [4, 8, 12, 16, 19]
 
export function getBarbarianFeatures(player, newLevel) {
    const base = BARBARIAN_BASE_FEATURES[newLevel] || []
    const subFeatures = player.subclass
        ? (BARBARIAN_SUBCLASS_FEATURES[player.subclass]?.[newLevel] || [])
        : []
    return [...base, ...subFeatures]
}
 
export function barbarianRageUses(level) {
    if (level >= 17) return 6
    if (level >= 12) return 5
    if (level >= 6)  return 4
    if (level >= 3)  return 3
    return 2
}
 
export function barbarianRageDamage(level) {
    if (level >= 16) return 4
    if (level >= 9)  return 3
    return 2
}
 
export function barbarianWeaponMastery(level) {
    if (level >= 10) return 4
    if (level >= 4)  return 3
    return 2
}

// ─── Bard ─────────────────────────────────────────────────────────────────────
 
export const BARD_SUBCLASSES = [
    {
        name: 'College of Dance',
        desc: 'Move in harmony with the cosmos. Unarmed strikes powered by Bardic Inspiration, Unarmored Defense using DEX+CHA, and flowing battlefield movement.',
    },
    {
        name: 'College of Glamour',
        desc: 'Weave beguiling fey magic. Charm or frighten after casting, grant allies Temporary HP with Mantle of Inspiration, and command with otherworldly authority.',
    },
    {
        name: 'College of Lore',
        desc: 'Plumb the depths of magical knowledge. Gain extra skill proficiencies, cut enemy rolls with Cutting Words, and eventually steal spells from any list.',
    },
    {
        name: 'College of Valor',
        desc: 'Sing the deeds of ancient heroes. Martial weapon and armor training, Bardic Inspiration that boosts AC or damage, and Extra Attack at level 6.',
    },
]
 
export const BARD_BASE_FEATURES = {
    1:  ['Bardic Inspiration', 'Spellcasting'],
    2:  ['Expertise', 'Jack of All Trades'],
    3:  ['Bard Subclass'],
    4:  ['Ability Score Improvement'],
    5:  ['Font of Inspiration'],
    6:  ['Subclass Feature'],
    7:  ['Countercharm'],
    8:  ['Ability Score Improvement'],
    9:  ['Expertise (2 more skills)'],
    10: ['Magical Secrets'],
    11: [],
    12: ['Ability Score Improvement'],
    13: [],
    14: ['Subclass Feature'],
    15: [],
    16: ['Ability Score Improvement'],
    17: [],
    18: ['Superior Inspiration'],
    19: ['Epic Boon'],
    20: ['Words of Creation'],
}
 
export const BARD_SUBCLASS_FEATURES = {
    'College of Dance': {
        3:  ['Dazzling Footwork'],
        6:  ['Inspiring Movement', 'Tandem Footwork'],
        14: ['Leading Evasion'],
    },
    'College of Glamour': {
        3:  ['Beguiling Magic', 'Mantle of Inspiration'],
        6:  ['Mantle of Majesty'],
        14: ['Unbreakable Majesty'],
    },
    'College of Lore': {
        3:  ['Bonus Proficiencies', 'Cutting Words'],
        6:  ['Magical Discoveries'],
        14: ['Peerless Skill'],
    },
    'College of Valor': {
        3:  ['Combat Inspiration', 'Martial Training'],
        6:  ['Extra Attack'],
        14: ['Battle Magic'],
    },
}
 
export const BARD_ASI_LEVELS = [4, 8, 12, 16, 19]
 
export function getBardFeatures(player, newLevel) {
    const base = (BARD_BASE_FEATURES[newLevel] || []).filter(f => f)
    const subFeatures = player.subclass
        ? (BARD_SUBCLASS_FEATURES[player.subclass]?.[newLevel] || [])
        : []
    return [...base, ...subFeatures]
}
 
export function bardInspirationDie(level) {
    if (level >= 15) return 12
    if (level >= 10) return 10
    if (level >= 5)  return 8
    return 6
}
 
export function bardPreparedSpells(level) {
    const table = [0,4,5,6,7,9,10,11,12,14,15,16,16,17,17,18,18,19,20,21,22]
    return table[level] || 22
}
 
export function bardCantrips(level) {
    if (level >= 10) return 4
    if (level >= 4)  return 3
    return 2
}
 
export function bardSpellSlots(level) {
    const table = {
        1:  [2,0,0,0,0,0,0,0,0],
        2:  [3,0,0,0,0,0,0,0,0],
        3:  [4,2,0,0,0,0,0,0,0],
        4:  [4,3,0,0,0,0,0,0,0],
        5:  [4,3,2,0,0,0,0,0,0],
        6:  [4,3,3,0,0,0,0,0,0],
        7:  [4,3,3,1,0,0,0,0,0],
        8:  [4,3,3,2,0,0,0,0,0],
        9:  [4,3,3,3,1,0,0,0,0],
        10: [4,3,3,3,2,0,0,0,0],
        11: [4,3,3,3,2,1,0,0,0],
        12: [4,3,3,3,2,1,0,0,0],
        13: [4,3,3,3,2,1,1,0,0],
        14: [4,3,3,3,2,1,1,0,0],
        15: [4,3,3,3,2,1,1,1,0],
        16: [4,3,3,3,2,1,1,1,0],
        17: [4,3,3,3,2,1,1,1,1],
        18: [4,3,3,3,3,1,1,1,1],
        19: [4,3,3,3,3,2,1,1,1],
        20: [4,3,3,3,3,2,2,1,1],
    }
    return table[level] || table[20]
}
 
// Compact display string e.g. "4/3/3/2"
export function bardSlotSummary(level) {
    return bardSpellSlots(level).filter(s => s > 0).join('/')
}

// ─── Cleric ───────────────────────────────────────────────────────────────────
 
export const CLERIC_SUBCLASSES = [
    {
        name: 'Life Domain',
        desc: 'Soothe the hurts of the world. Healing spells restore bonus HP, Preserve Life channels mass healing, and Supreme Healing always rolls maximum dice.',
    },
    {
        name: 'Light Domain',
        desc: 'Bring light to banish darkness. Radiance of the Dawn blinds enemies, Warding Flare deflects attacks, and Corona of Light weakens foes in your sunlight.',
    },
    {
        name: 'Trickery Domain',
        desc: 'Make mischief and challenge authority. Invoke Duplicity creates a perfect illusion of yourself, and Trickster\'s Transposition lets you swap places with it.',
    },
    {
        name: 'War Domain',
        desc: 'Inspire valor and smite foes. Guided Strike adds +10 to attack rolls, War Priest grants bonus weapon attacks, and Avatar of Battle grants damage resistance.',
    },
]
 
export const CLERIC_BASE_FEATURES = {
    1:  ['Spellcasting', 'Divine Order'],
    2:  ['Channel Divinity'],
    3:  ['Cleric Subclass'],
    4:  ['Ability Score Improvement'],
    5:  ['Sear Undead'],
    6:  ['Subclass Feature'],
    7:  ['Blessed Strikes'],
    8:  ['Ability Score Improvement'],
    9:  [],
    10: ['Divine Intervention'],
    11: [],
    12: ['Ability Score Improvement'],
    13: [],
    14: ['Improved Blessed Strikes'],
    15: [],
    16: ['Ability Score Improvement'],
    17: ['Subclass Feature'],
    18: [],
    19: ['Epic Boon'],
    20: ['Greater Divine Intervention'],
}
 
export const CLERIC_SUBCLASS_FEATURES = {
    'Life Domain': {
        3:  ['Disciple of Life', 'Life Domain Spells', 'Preserve Life'],
        6:  ['Blessed Healer'],
        17: ['Supreme Healing'],
    },
    'Light Domain': {
        3:  ['Light Domain Spells', 'Radiance of the Dawn', 'Warding Flare'],
        6:  ['Improved Warding Flare'],
        17: ['Corona of Light'],
    },
    'Trickery Domain': {
        3:  ['Blessing of the Trickster', 'Trickery Domain Spells', 'Invoke Duplicity'],
        6:  ["Trickster's Transposition"],
        17: ['Improved Duplicity'],
    },
    'War Domain': {
        3:  ['Guided Strike', 'War Domain Spells', 'War Priest'],
        6:  ["War God's Blessing"],
        17: ['Avatar of Battle'],
    },
}
 
export const CLERIC_ASI_LEVELS = [4, 8, 12, 16, 19]
 
export function getClericFeatures(player, newLevel) {
    const base = (CLERIC_BASE_FEATURES[newLevel] || []).filter(f => f)
    const subFeatures = player.subclass
        ? (CLERIC_SUBCLASS_FEATURES[player.subclass]?.[newLevel] || [])
        : []
    return [...base, ...subFeatures]
}
 
export function clericChannelDivinityUses(level) {
    if (level < 2)   return 0
    if (level < 6)   return 2
    if (level < 18)  return 3
    return 4
}
 
export function clericCantrips(level) {
    if (level >= 10) return 5
    if (level >= 4)  return 4
    return 3
}
 
export function clericPreparedSpells(level) {
    const table = [0,4,5,6,7,9,10,11,12,14,15,16,16,17,17,18,18,19,20,21,22]
    return table[level] || 22
}
 
export function clericSlotSummary(level) {
    // Same full-caster table as Bard
    const slots = {
        1:[2,0,0,0,0,0,0,0,0], 2:[3,0,0,0,0,0,0,0,0], 3:[4,2,0,0,0,0,0,0,0],
        4:[4,3,0,0,0,0,0,0,0], 5:[4,3,2,0,0,0,0,0,0], 6:[4,3,3,0,0,0,0,0,0],
        7:[4,3,3,1,0,0,0,0,0], 8:[4,3,3,2,0,0,0,0,0], 9:[4,3,3,3,1,0,0,0,0],
        10:[4,3,3,3,2,0,0,0,0],11:[4,3,3,3,2,1,0,0,0],12:[4,3,3,3,2,1,0,0,0],
        13:[4,3,3,3,2,1,1,0,0],14:[4,3,3,3,2,1,1,0,0],15:[4,3,3,3,2,1,1,1,0],
        16:[4,3,3,3,2,1,1,1,0],17:[4,3,3,3,2,1,1,1,1],18:[4,3,3,3,3,1,1,1,1],
        19:[4,3,3,3,3,2,1,1,1],20:[4,3,3,3,3,2,2,1,1],
    }
    return (slots[level] || slots[20]).filter(s => s > 0).join('/')
}

// ─── Druid ────────────────────────────────────────────────────────────────────
 
export const DRUID_CANTRIPS_LIST = [
    { name: 'Druidcraft',    school: 'Transmutation', note: null },
    { name: 'Elementalism',  school: 'Transmutation', note: null },
    { name: 'Guidance',      school: 'Divination',    note: 'Concentration' },
    { name: 'Mending',       school: 'Transmutation', note: null },
    { name: 'Message',       school: 'Transmutation', note: null },
    { name: 'Poison Spray',  school: 'Necromancy',    note: null },
    { name: 'Produce Flame', school: 'Conjuration',   note: null },
    { name: 'Resistance',    school: 'Abjuration',    note: 'Concentration' },
    { name: 'Shillelagh',    school: 'Transmutation', note: 'WIS for attacks' },
    { name: 'Spare the Dying', school: 'Necromancy',  note: null },
    { name: 'Starry Wisp',   school: 'Evocation',     note: null },
    { name: 'Thorn Whip',    school: 'Transmutation', note: null },
    { name: 'Thunderclap',   school: 'Evocation',     note: null },
]
 
export const DRUID_SUBCLASSES = [
    {
        name: 'Circle of the Land',
        desc: 'Ancient mystics who safeguard natural lore. Choose a land type each Long Rest for bonus prepared spells, and recover spell slots on Short Rests.',
    },
    {
        name: 'Circle of the Moon',
        desc: 'Channel lunar magic to assume more powerful Beast forms. Wild Shape CR scales with your level, and your attacks deal Radiant damage.',
    },
    {
        name: 'Circle of the Sea',
        desc: 'Embody tides and storms. Wrath of the Sea buffets foes with cold waves, and at higher levels you soar and resist elemental damage.',
    },
    {
        name: 'Circle of the Stars',
        desc: 'Harness secrets hidden in constellations. Starry Form grants Archer, Chalice, or Dragon constellation powers, and free Guiding Bolt casts.',
    },
]
 
export const DRUID_BASE_FEATURES = {
    1:  ['Spellcasting', 'Druidic', 'Primal Order'],
    2:  ['Wild Shape', 'Wild Companion'],
    3:  ['Druid Subclass'],
    4:  ['Ability Score Improvement', 'Wild Shape (CR 1/2, 6 forms)'],
    5:  ['Wild Resurgence'],
    6:  ['Subclass Feature', 'Wild Shape (3 uses)'],
    7:  ['Elemental Fury'],
    8:  ['Ability Score Improvement', 'Wild Shape (CR 1, 8 forms, Fly Speed)'],
    9:  [],
    10: ['Subclass Feature'],
    11: [],
    12: ['Ability Score Improvement'],
    13: [],
    14: ['Subclass Feature'],
    15: ['Improved Elemental Fury'],
    16: ['Ability Score Improvement'],
    17: ['Wild Shape (4 uses)'],
    18: ['Beast Spells'],
    19: ['Epic Boon'],
    20: ['Archdruid'],
}
 
export const DRUID_SUBCLASS_FEATURES = {
    'Circle of the Land': {
        3:  ['Circle of the Land Spells', "Land's Aid"],
        6:  ['Natural Recovery'],
        10: ["Nature's Ward"],
        14: ["Nature's Sanctuary"],
    },
    'Circle of the Moon': {
        3:  ['Circle Forms', 'Circle of the Moon Spells'],
        6:  ['Improved Circle Forms'],
        10: ['Moonlight Step'],
        14: ['Lunar Form'],
    },
    'Circle of the Sea': {
        3:  ['Circle of the Sea Spells', 'Wrath of the Sea'],
        6:  ['Aquatic Affinity'],
        10: ['Stormborn'],
        14: ['Oceanic Gift'],
    },
    'Circle of the Stars': {
        3:  ['Star Map', 'Starry Form'],
        6:  ['Cosmic Omen'],
        10: ['Twinkling Constellations'],
        14: ['Full of Stars'],
    },
}
 
export const DRUID_ASI_LEVELS = [4, 8, 12, 16, 19]
 
export function getDruidFeatures(player, newLevel) {
    const base = (DRUID_BASE_FEATURES[newLevel] || []).filter(f => f)
    const subFeatures = player.subclass
        ? (DRUID_SUBCLASS_FEATURES[player.subclass]?.[newLevel] || [])
        : []
    return [...base, ...subFeatures]
}
 
export function druidWildShapeUses(level) {
    if (level < 2)   return 0
    if (level < 6)   return 2
    if (level < 17)  return 3
    return 4
}
 
export function druidWildShapeCR(level) {
    if (level >= 8) return '1'
    if (level >= 4) return '1/2'
    return '1/4'
}
 
export function druidCantrips(level) {
    if (level >= 10) return 4
    if (level >= 4)  return 3
    return 2
}
 
export function druidPreparedSpells(level) {
    const table = [0,4,5,6,7,9,10,11,12,14,15,16,16,17,17,18,18,19,20,21,22]
    return table[level] || 22
}
 
export function druidSlotSummary(level) {
    const slots = {
        1:[2,0,0,0,0,0,0,0,0], 2:[3,0,0,0,0,0,0,0,0], 3:[4,2,0,0,0,0,0,0,0],
        4:[4,3,0,0,0,0,0,0,0], 5:[4,3,2,0,0,0,0,0,0], 6:[4,3,3,0,0,0,0,0,0],
        7:[4,3,3,1,0,0,0,0,0], 8:[4,3,3,2,0,0,0,0,0], 9:[4,3,3,3,1,0,0,0,0],
        10:[4,3,3,3,2,0,0,0,0],11:[4,3,3,3,2,1,0,0,0],12:[4,3,3,3,2,1,0,0,0],
        13:[4,3,3,3,2,1,1,0,0],14:[4,3,3,3,2,1,1,0,0],15:[4,3,3,3,2,1,1,1,0],
        16:[4,3,3,3,2,1,1,1,0],17:[4,3,3,3,2,1,1,1,1],18:[4,3,3,3,3,1,1,1,1],
        19:[4,3,3,3,3,2,1,1,1],20:[4,3,3,3,3,2,2,1,1],
    }
    return (slots[level] || slots[20]).filter(s => s > 0).join('/')
}

// ─── Monk ─────────────────────────────────────────────────────────────────────
 
export const MONK_SUBCLASSES = [
    {
        name: 'Warrior of Mercy',
        desc: 'Manipulate forces of life and death. Hand of Harm deals necrotic damage, Hand of Healing restores HP, and Hand of Ultimate Mercy revives the dead.',
    },
    {
        name: 'Warrior of Shadow',
        desc: 'Harness shadow power for stealth and subterfuge. Cast Darkness, teleport between shadows, and eventually shroud yourself in invisibility.',
    },
    {
        name: 'Warrior of the Elements',
        desc: 'Wield strikes and bursts of elemental power. Imbue attacks with acid, cold, fire, lightning, or thunder, and burst with elemental energy.',
    },
    {
        name: 'Warrior of the Open Hand',
        desc: 'Master unarmed combat techniques. Addle, push, or topple foes with Flurry of Blows, and set up lethal vibrations with Quivering Palm.',
    },
]
 
export const MONK_BASE_FEATURES = {
    1:  ['Martial Arts', 'Unarmored Defense'],
    2:  ["Monk's Focus", 'Unarmored Movement', 'Uncanny Metabolism'],
    3:  ['Deflect Attacks', 'Monk Subclass'],
    4:  ['Ability Score Improvement', 'Slow Fall'],
    5:  ['Extra Attack', 'Stunning Strike'],
    6:  ['Empowered Strikes', 'Subclass Feature'],
    7:  ['Evasion'],
    8:  ['Ability Score Improvement'],
    9:  ['Acrobatic Movement'],
    10: ['Heightened Focus', 'Self-Restoration'],
    11: ['Subclass Feature'],
    12: ['Ability Score Improvement'],
    13: ['Deflect Energy'],
    14: ['Disciplined Survivor'],
    15: ['Perfect Focus'],
    16: ['Ability Score Improvement'],
    17: ['Subclass Feature'],
    18: ['Superior Defense'],
    19: ['Epic Boon'],
    20: ['Body and Mind'],
}
 
export const MONK_SUBCLASS_FEATURES = {
    'Warrior of Mercy': {
        3:  ['Hand of Harm', 'Hand of Healing', 'Implements of Mercy'],
        6:  ["Physician's Touch"],
        11: ['Flurry of Healing and Harm'],
        17: ['Hand of Ultimate Mercy'],
    },
    'Warrior of Shadow': {
        3:  ['Shadow Arts'],
        6:  ['Shadow Step'],
        11: ['Improved Shadow Step'],
        17: ['Cloak of Shadows'],
    },
    'Warrior of the Elements': {
        3:  ['Elemental Attunement', 'Manipulate Elements'],
        6:  ['Elemental Burst'],
        11: ['Stride of the Elements'],
        17: ['Elemental Epitome'],
    },
    'Warrior of the Open Hand': {
        3:  ['Open Hand Technique'],
        6:  ['Wholeness of Body'],
        11: ['Fleet Step'],
        17: ['Quivering Palm'],
    },
}
 
export const MONK_ASI_LEVELS = [4, 8, 12, 16, 19]
 
export function getMonkFeatures(player, newLevel) {
    const base = MONK_BASE_FEATURES[newLevel] || []
    const subFeatures = player.subclass
        ? (MONK_SUBCLASS_FEATURES[player.subclass]?.[newLevel] || [])
        : []
    return [...base, ...subFeatures]
}
 
// Focus Points = Monk level (0 at level 1, gained at level 2)
export function monkFocusPoints(level) {
    return level >= 2 ? level : 0
}
 
// Martial Arts die by level
export function monkMartialArtsDie(level) {
    if (level >= 17) return 12
    if (level >= 11) return 10
    if (level >= 5)  return 8
    return 6
}
 
// Unarmored Movement bonus in feet
export function monkUnarmoredMovement(level) {
    if (level < 2)   return 0
    if (level < 6)   return 10
    if (level < 10)  return 15
    if (level < 14)  return 20
    if (level < 18)  return 25
    return 30
}

// ─── Paladin ──────────────────────────────────────────────────────────────────
 
export const PALADIN_SUBCLASSES = [
    {
        name: 'Oath of Devotion',
        desc: 'Uphold the ideals of justice and order. Sacred Weapon imbues your blade with holy power, and Holy Nimbus floods your aura with divine radiance.',
    },
    {
        name: 'Oath of Glory',
        desc: 'Strive for the heights of heroism. Inspire allies with Temporary HP after smiting, grant Speed bonuses with your aura, and deflect attacks with Glorious Defense.',
    },
    {
        name: 'Oath of the Ancients',
        desc: 'Preserve life and light in the world. Restrain foes with spectral vines, grant your allies Resistance to energy damage, and shrug off death itself.',
    },
    {
        name: 'Oath of Vengeance',
        desc: 'Punish evildoers at any cost. Mark a foe with Vow of Enmity for Advantage on attacks, cut off retreats with Relentless Avenger, and sprout wings at level 20.',
    },
]
 
export const PALADIN_BASE_FEATURES = {
    1:  ['Lay On Hands', 'Spellcasting', 'Weapon Mastery'],
    2:  ['Fighting Style', "Paladin's Smite"],
    3:  ['Channel Divinity', 'Paladin Subclass'],
    4:  ['Ability Score Improvement'],
    5:  ['Extra Attack', 'Faithful Steed'],
    6:  ['Aura of Protection'],
    7:  ['Subclass Feature'],
    8:  ['Ability Score Improvement'],
    9:  ['Abjure Foes'],
    10: ['Aura of Courage'],
    11: ['Radiant Strikes'],
    12: ['Ability Score Improvement'],
    13: [],
    14: ['Restoring Touch'],
    15: ['Subclass Feature'],
    16: ['Ability Score Improvement'],
    17: [],
    18: ['Aura Expansion'],
    19: ['Epic Boon'],
    20: ['Subclass Feature'],
}
 
export const PALADIN_SUBCLASS_FEATURES = {
    'Oath of Devotion': {
        3:  ['Oath of Devotion Spells', 'Sacred Weapon'],
        7:  ['Aura of Devotion'],
        15: ['Smite of Protection'],
        20: ['Holy Nimbus'],
    },
    'Oath of Glory': {
        3:  ['Oath of Glory Spells', 'Inspiring Smite', 'Peerless Athlete'],
        7:  ['Aura of Alacrity'],
        15: ['Glorious Defense'],
        20: ['Living Legend'],
    },
    'Oath of the Ancients': {
        3:  ['Oath of the Ancients Spells', "Nature's Wrath"],
        7:  ['Aura of Warding'],
        15: ['Undying Sentinel'],
        20: ['Elder Champion'],
    },
    'Oath of Vengeance': {
        3:  ['Oath of Vengeance Spells', 'Vow of Enmity'],
        7:  ['Relentless Avenger'],
        15: ['Soul of Vengeance'],
        20: ['Avenging Angel'],
    },
}
 
export const PALADIN_ASI_LEVELS = [4, 8, 12, 16, 19]
 
export function getPaladinFeatures(player, newLevel) {
    const base = (PALADIN_BASE_FEATURES[newLevel] || []).filter(f => f)
    const subFeatures = player.subclass
        ? (PALADIN_SUBCLASS_FEATURES[player.subclass]?.[newLevel] || [])
        : []
    return [...base, ...subFeatures]
}
 
// Lay on Hands pool = 5 × Paladin level
export function paladinLayOnHandsPool(level) {
    return level * 5
}
 
// Channel Divinity: 0 before L3, 2 from L3-10, 3 from L11+
export function paladinChannelDivinityUses(level) {
    if (level < 3)   return 0
    if (level < 11)  return 2
    return 3
}
 
// Prepared spells column from the PHB table
export function paladinPreparedSpells(level) {
    const table = [0,2,3,4,5,6,6,7,7,9,9,10,10,11,11,12,12,14,14,15,15]
    return table[level] || 15
}
 
// Half-caster slot table: max level 5 slots
export function paladinSlotSummary(level) {
    const slots = {
        1: [2,0,0,0,0], 2: [2,0,0,0,0], 3: [3,0,0,0,0], 4: [3,0,0,0,0],
        5: [4,2,0,0,0], 6: [4,2,0,0,0], 7: [4,3,0,0,0], 8: [4,3,0,0,0],
        9: [4,3,2,0,0], 10:[4,3,2,0,0], 11:[4,3,3,0,0], 12:[4,3,3,0,0],
        13:[4,3,3,1,0], 14:[4,3,3,1,0], 15:[4,3,3,2,0], 16:[4,3,3,2,0],
        17:[4,3,3,3,1], 18:[4,3,3,3,1], 19:[4,3,3,3,2], 20:[4,3,3,3,2],
    }
    return (slots[level] || slots[20]).filter(s => s > 0).join('/')
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

export function xpToNextLevel(level) {
    return XP_THRESHOLDS[level] ?? XP_THRESHOLDS[XP_THRESHOLDS.length - 1]
}

export function xpProgress(experience, level) {
    const current = XP_THRESHOLDS[level - 1] || 0
    const next = XP_THRESHOLDS[level] ?? current
    if (next <= current) return 100
    return ((experience - current) / (next - current)) * 100
}

export function isLevelUpAvailable(player) {
    return player.experience >= XP_THRESHOLDS[player.level] && player.level < 20
}

export function statModifier(score) {
    return Math.floor((score - 10) / 2)
}

export function formatModifier(score) {
    const m = statModifier(score)
    return (m >= 0 ? '+' : '') + m
}

export function proficiencyForLevel(level) {
    if (level <= 4) return 2
    if (level <= 8) return 3
    if (level <= 12) return 4
    if (level <= 16) return 5
    return 6
}

export function hitDieForClass(cls) {
    switch (cls) {
        case 'Barbarian': return 12
        case 'Fighter': case 'Paladin': case 'Ranger': return 10
        case 'Cleric': case 'Druid': case 'Monk': case 'Rogue': case 'Bard': case 'Warlock': return 8
        case 'Sorcerer': case 'Wizard': return 6
        default: return 8
    }
}

export function getFighterFeatures(player, newLevel) {
    const base = FIGHTER_BASE_FEATURES[newLevel] || []
    const subFeatures = player.subclass
        ? (FIGHTER_SUBCLASS_FEATURES[player.subclass]?.[newLevel] || [])
        : []
    return [...base, ...subFeatures]
}

export function getBackgroundByName(name) {
    return BACKGROUNDS.find(b => b.name === name) || null
}

export function getSpeciesByName(name) {
    return SPECIES.find(s => s.name === name) || null
}