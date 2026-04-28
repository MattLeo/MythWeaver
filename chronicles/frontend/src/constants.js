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