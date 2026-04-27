// ─── XP Thresholds ────────────────────────────────────────────────────────────

export const XP_THRESHOLDS = [
  0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000,
  85000, 100000, 120000, 140000, 165000, 195000, 225000, 265000, 305000, 355000
]

// ─── Stats ────────────────────────────────────────────────────────────────────

export const STAT_KEYS = ['str', 'dex', 'con', 'int', 'wis', 'cha']
export const STAT_LABELS = { str: 'STR', dex: 'DEX', con: 'CON', int: 'INT', wis: 'WIS', cha: 'CHA' }
export const STAT_LABELS_ARRAY = ['STR', 'DEX', 'CON', 'INT', 'WIS', 'CHA']

// ─── Classes ─────────────────────────────────────────────────────────────────

export const CLASSES = [
  'Barbarian', 'Bard', 'Cleric', 'Druid', 'Fighter',
  'Monk', 'Paladin', 'Ranger', 'Rogue', 'Sorcerer', 'Warlock', 'Wizard'
]

export const RACES = [
  'Human', 'Elf', 'Dwarf', 'Halfling', 'Half-Elf',
  'Half-Orc', 'Gnome', 'Tiefling', 'Dragonborn'
]

export const BACKGROUNDS = [
  'Acolyte', 'Charlatan', 'Criminal', 'Entertainer', 'Folk Hero',
  'Hermit', 'Noble', 'Outlander', 'Sage', 'Soldier', 'Urchin'
]

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
  1:  ['Fighting Style', 'Second Wind', 'Weapon Mastery'],
  2:  ['Action Surge', 'Tactical Mind'],
  3:  ['Fighter Subclass'],
  4:  ['Ability Score Improvement'],
  5:  ['Extra Attack', 'Tactical Shift'],
  6:  ['Ability Score Improvement'],
  7:  ['Subclass Feature'],
  8:  ['Ability Score Improvement'],
  9:  ['Indomitable', 'Tactical Master'],
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
    3:  ['Improved Critical', 'Remarkable Athlete'],
    7:  ['Additional Fighting Style'],
    10: ['Heroic Warrior'],
    15: ['Superior Critical'],
    18: ['Survivor'],
  },
  'Battle Master': {
    3:  ['Combat Superiority', 'Student of War'],
    7:  ['Know Your Enemy'],
    10: ['Improved Combat Superiority (d10)'],
    15: ['Relentless', 'Improved Combat Superiority (d12)'],
    18: ['Ultimate Combat Superiority'],
  },
  'Psi Warrior': {
    3:  ['Psionic Power'],
    7:  ['Telekinetic Adept'],
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
  if (level <= 4)  return 2
  if (level <= 8)  return 3
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