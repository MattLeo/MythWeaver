import { useState, useEffect } from 'react'
import { STYLES } from '../styles.js'
import { getSpellsByClass } from '../api/client.js'
import {
  CLASSES, SPECIES, BACKGROUNDS, CLASS_EQUIPMENT, SEX_OPTIONS,
  STAT_KEYS, STAT_LABELS_ARRAY, statModifier, formatModifier, hitDieForClass,
  getBackgroundByName, getSpeciesByName,
} from '../constants.js'

const d = (n) => Math.floor(Math.random() * n) + 1

const rollBlock = () => Array.from({ length: 6 }, () => {
  const r = [d(6), d(6), d(6), d(6)].sort((a, b) => a - b)
  return r[1] + r[2] + r[3]
})

const STARTING_CANTRIP_COUNTS = {
  Wizard: 3, Cleric: 3, Sorcerer: 4, Bard: 2, Druid: 2, Warlock: 2,
}
const STARTING_SPELL_COUNTS = {
  Wizard: 6, Bard: 4, Sorcerer: 2, Warlock: 2,
}
const CANTRIP_CLASSES = Object.keys(STARTING_CANTRIP_COUNTS)
const KNOWN_SPELL_CLASSES = Object.keys(STARTING_SPELL_COUNTS)
const MI_LISTS = ['Cleric', 'Druid', 'Wizard']

const SCHOOL_COLORS_SPELL = {
  abjuration: '#7ec8e3', conjuration: '#b5a9f5', divination: '#f5e87e',
  enchantment: '#f5a9c8', evocation: '#f5a96a', illusion: '#a9f5d0',
  necromancy: '#b0f5a9', transmutation: '#f5cfa9',
}
const SCHOOL_GLYPHS_SPELL = {
  abjuration: '🛡', conjuration: '✦', divination: '👁',
  enchantment: '♡', evocation: '⚡', illusion: '◈',
  necromancy: '☽', transmutation: '⟳',
}
const DAMAGE_TYPE_COLORS_SPELL = {
  fire: '#f5764a', cold: '#7ec8e3', lightning: '#ffe066', acid: '#a8e86e',
  poison: '#8bcf6e', necrotic: '#b0f5a9', radiant: '#fff3a3', psychic: '#f5a9c8',
  force: '#c4a9f5', thunder: '#a9c4f5', piercing: '#d0c8b8', slashing: '#d0c8b8', bludgeoning: '#d0c8b8',
}
function hexToRgb(hex) {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return `${r},${g},${b}`
}
function formatCastingTime(ct) {
  if (!ct) return '—'
  return ct
    .replace('bonus_action', 'Bonus Action')
    .replace('reaction', 'Reaction')
    .replace('action', 'Action')
    .replace('1_minute', '1 Min')
    .replace('10_minutes', '10 Min')
    .replace('1_hour', '1 Hr')
    .replace(/_/g, ' ')
}

function formatRange(rangeType, rangeFeet) {
  if (rangeType === 'self') return 'Self'
  if (rangeType === 'touch') return 'Touch'
  if (rangeType === 'special') return 'Special'
  if (rangeFeet) return `${rangeFeet} ft`
  return rangeType || '—'
}

function formatDuration(dur) {
  if (!dur) return '—'
  return dur
    .replace('concentration_1_minute', '1 Min ◉')
    .replace('concentration_10_minutes', '10 Min ◉')
    .replace('concentration_1_hour', '1 Hr ◉')
    .replace('instantaneous', 'Instant')
    .replace('until_dispelled', '∞')
    .replace('until_dispelled_or_triggered', '∞/Trigger')
    .replace('1_minute', '1 Min')
    .replace('10_minutes', '10 Min')
    .replace('1_hour', '1 Hr')
    .replace('8_hours', '8 Hr')
    .replace('24_hours', '24 Hr')
    .replace('1_round', '1 Round')
    .replace(/_/g, ' ')
}

const CREATION_STYLES = `
${STYLES}
.creation {
  min-height: 100vh; display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  padding: 2rem; overflow-y: auto;
  background: radial-gradient(ellipse at 50% 0%, #0d1220 0%, #0b0c12 60%);
}
.card {
  background: var(--surf); border: 1px solid var(--bord);
  border-radius: 3px; padding: 2.25rem;
  max-width: 700px; width: 100%;
}
.card h2 {
  font-family: 'Cinzel', serif; color: var(--gold);
  font-size: 1.3rem; margin-bottom: .5rem;
  padding-bottom: .75rem; border-bottom: 1px solid var(--bord);
}
.card-sub {
  font-size: .8rem; color: var(--dim); font-style: italic;
  margin-bottom: 1.25rem; line-height: 1.6;
}
.steps { display: flex; gap: .45rem; justify-content: center; margin-bottom: 2rem; flex-wrap: wrap; }
.step { width: 28px; height: 3px; border-radius: 2px; background: var(--bord); transition: background .3s; }
.step.on { background: var(--gold); }
.stat-g { display: grid; grid-template-columns: repeat(3,1fr); gap: .65rem; margin: .75rem 0; }
.stat-box {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .7rem; text-align: center;
  cursor: pointer; transition: all .15s;
}
.stat-box:hover, .stat-box.sel {
  border-color: var(--gold); background: rgba(200,150,42,.07);
}
.stat-box .sl { font-family: 'Cinzel', serif; font-size: .65rem; letter-spacing: .12em; color: var(--dim); margin-bottom: .2rem; }
.stat-box .sv { font-size: 1.6rem; color: var(--goldl); font-weight: bold; line-height: 1; }
.stat-box .sm { font-size: .75rem; color: var(--dim); margin-top: .15rem; }
.stat-box .sa { font-size: .68rem; color: var(--gold); margin-top: .1rem; font-family: 'Cinzel', serif; }
.cnav { display: flex; justify-content: space-between; align-items: center; margin-top: 2rem; }
.pick-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: .6rem; }
.pick-grid-2 { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: .6rem; }
.pick {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .65rem .5rem;
  cursor: pointer; text-align: center; font-size: .85rem;
  color: var(--dim); transition: all .2s;
}
.pick:hover, .pick.sel {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.07);
}
.pick-card {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .85rem 1rem;
  cursor: pointer; transition: all .15s; text-align: left;
}
.pick-card:hover, .pick-card.sel {
  border-color: var(--gold); background: rgba(200,150,42,.07);
}
.pick-card-name {
  font-family: 'Cinzel', serif; font-size: .82rem;
  color: var(--goldl); margin-bottom: .25rem;
}
.pick-card-desc { font-size: .74rem; color: var(--dim); line-height: 1.55; }
.pick-card-meta { font-size: .68rem; color: var(--gold); margin-top: .35rem; font-family: 'Cinzel', serif; letter-spacing: .05em; }
.equip-card {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: 1rem 1.1rem;
  cursor: pointer; transition: all .15s;
}
.equip-card:hover, .equip-card.sel {
  border-color: var(--gold); background: rgba(200,150,42,.07);
}
.equip-label {
  font-family: 'Cinzel', serif; font-size: .9rem;
  color: var(--goldl); margin-bottom: .35rem;
}
.equip-desc { font-size: .78rem; color: var(--dim); line-height: 1.6; }
.asi-row {
  display: flex; justify-content: space-between; align-items: center;
  padding: .5rem .75rem; background: var(--elev);
  border: 1px solid var(--bord); border-radius: 2px; margin-bottom: .4rem;
  font-size: .82rem;
}
.asi-stat { font-family: 'Cinzel', serif; color: var(--goldl); font-size: .75rem; letter-spacing: .08em; }
.asi-btns { display: flex; gap: .4rem; align-items: center; }
.asi-btn {
  background: var(--surf); border: 1px solid var(--bord);
  color: var(--dim); width: 24px; height: 24px;
  border-radius: 2px; cursor: pointer; font-size: .85rem;
  display: flex; align-items: center; justify-content: center;
  transition: all .15s;
}
.asi-btn:hover:not(:disabled) { border-color: var(--gold); color: var(--gold); }
.asi-btn:disabled { opacity: .3; cursor: not-allowed; }
.asi-val { font-family: 'Cinzel', serif; font-size: .82rem; color: var(--goldl); min-width: 1.5rem; text-align: center; }
.info-box {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .75rem 1rem;
  font-size: .78rem; color: var(--dim); line-height: 1.65;
  margin-bottom: 1rem;
}
.info-box strong { color: var(--goldl); font-family: 'Cinzel', serif; font-size: .72rem; letter-spacing: .06em; }
.sex-grid { display: grid; grid-template-columns: 1fr 1fr; gap: .75rem; }
.pick-card.disabled {
  opacity: 0.35;
  cursor: not-allowed;
  pointer-events: none;
}
.pick-card.disabled { opacity: 0.35; cursor: not-allowed; pointer-events: none; }
`

// ─── Step definitions ─────────────────────────────────────────────────────────

// We build the step list dynamically based on selections
// Base steps: name, sex, species, [species_subtype?], class, background, background_asi, stats, equipment, backstory
const BASE_STEPS = ['name', 'sex', 'species', 'class', 'background', 'background_asi', 'stats', 'equipment', 'backstory']

const CLERIC_CANTRIPS = [
  { name: 'Guidance', school: 'Divination', note: 'Concentration' },
  { name: 'Light', school: 'Evocation', note: null },
  { name: 'Mending', school: 'Transmutation', note: null },
  { name: 'Resistance', school: 'Abjuration', note: 'Concentration' },
  { name: 'Sacred Flame', school: 'Evocation', note: 'DEX save' },
  { name: 'Spare the Dying', school: 'Necromancy', note: null },
  { name: 'Thaumaturgy', school: 'Transmutation', note: null },
  { name: 'Toll the Dead', school: 'Necromancy', note: 'WIS save' },
  { name: 'Word of Radiance', school: 'Evocation', note: 'CON save' },
]

const DRUID_CANTRIPS_CC = [
  { name: 'Druidcraft', school: 'Transmutation', note: null },
  { name: 'Elementalism', school: 'Transmutation', note: null },
  { name: 'Guidance', school: 'Divination', note: 'Concentration' },
  { name: 'Mending', school: 'Transmutation', note: null },
  { name: 'Message', school: 'Transmutation', note: null },
  { name: 'Poison Spray', school: 'Necromancy', note: null },
  { name: 'Produce Flame', school: 'Conjuration', note: null },
  { name: 'Resistance', school: 'Abjuration', note: 'Concentration' },
  { name: 'Shillelagh', school: 'Transmutation', note: 'WIS for attacks' },
  { name: 'Spare the Dying', school: 'Necromancy', note: null },
  { name: 'Starry Wisp', school: 'Evocation', note: null },
  { name: 'Thorn Whip', school: 'Transmutation', note: null },
  { name: 'Thunderclap', school: 'Evocation', note: null },
]

const SCHOOL_COLORS_CC = {
  Divination: '#f5e87e', Evocation: '#f5a96a', Transmutation: '#f5cfa9',
  Abjuration: '#7ec8e3', Necromancy: '#b0f5a9', Conjuration: '#b5a9f5',
}

function buildSteps(race, background, playerClass, divineOrder, primalOrder, backgroundFeatId) {
  const steps = ['name', 'sex', 'species']
  const sp = getSpeciesByName(race)
  if (sp?.subtype) steps.push('species_subtype')
  steps.push('class')

  // Spell selection for casting classes — happens right after class choice
  if (CANTRIP_CLASSES.includes(playerClass)) steps.push('starting_cantrips')
  if (KNOWN_SPELL_CLASSES.includes(playerClass)) steps.push('starting_spells')

  if (playerClass === 'Cleric') {
    steps.push('divine_order')
    if (divineOrder === 'Thaumaturge') steps.push('thaumaturge_cantrip')
  }
  if (playerClass === 'Druid') {
    steps.push('primal_order')
    if (primalOrder === 'Magician') steps.push('magician_cantrip')
  }

  steps.push('background', 'background_asi', 'background_feat')

  // Magic Initiate requires spell selection after feat is chosen
  if (backgroundFeatId === 'feat_magic_initiate') {
    steps.push('magic_initiate_list')
    steps.push('magic_initiate_spells')
  }

  steps.push('stats', 'equipment', 'backstory')
  return steps
}

function CreationSpellCard({ spell, isSelected, isDisabled, onClick }) {
  const school = spell.school || 'evocation'
  const color = SCHOOL_COLORS_SPELL[school] || '#c4a9f5'
  const glyph = SCHOOL_GLYPHS_SPELL[school] || '✦'
  const isCantrip = spell.level === 0
  return (
    <div
      onClick={() => !isDisabled && onClick(spell)}
      style={{
        padding: '8px 10px', marginBottom: 4, borderRadius: 8, border: '1px solid',
        borderColor: isSelected ? color : 'rgba(255,255,255,0.06)',
        background: isSelected
          ? `linear-gradient(135deg, rgba(${hexToRgb(color)},0.12), rgba(${hexToRgb(color)},0.04))`
          : 'rgba(255,255,255,0.02)',
        boxShadow: isSelected ? `0 0 0 1px ${color}40, inset 0 0 20px ${color}08` : 'none',
        cursor: isDisabled ? 'not-allowed' : 'pointer',
        opacity: isDisabled ? 0.35 : 1,
        transition: 'all 0.15s',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span style={{ fontSize: 14, width: 18, textAlign: 'center', flexShrink: 0, color }}>{glyph}</span>
        <span style={{ flex: 1, fontSize: 13, color: '#d0c8b8', lineHeight: 1.2 }}>{spell.name}</span>
        {isCantrip
          ? <span style={{ fontSize: 11, fontWeight: 700, padding: '1px 6px', borderRadius: 8, background: 'rgba(255,255,255,0.1)', color: '#aaa', flexShrink: 0 }}>⊕</span>
          : <span style={{ fontSize: 11, fontWeight: 700, padding: '1px 6px', borderRadius: 8, background: `${color}22`, color, flexShrink: 0 }}>{spell.level}</span>
        }
      </div>
      {spell.concentration === 1 && (
        <span style={{ fontSize: 10, color: '#f5a96a', marginTop: 2, paddingLeft: 26 }}>◉ conc</span>
      )}
    </div>
  )
}

function CreationSpellDetail({ spell }) {
  if (!spell) return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', gap: 12 }}>
      <div style={{ fontSize: 48, color: '#2a2a3a' }}>✦</div>
      <p style={{ color: '#444', fontSize: 14 }}>Select a spell to view details</p>
    </div>
  )

  const school = spell.school || 'evocation'
  const color = SCHOOL_COLORS_SPELL[school] || '#c4a9f5'
  const glyph = SCHOOL_GLYPHS_SPELL[school] || '✦'
  const isCantrip = spell.level === 0
  const dmgColor = DAMAGE_TYPE_COLORS_SPELL[spell.damage_type] || '#d0c8b8'

  return (
    <div style={{ padding: '0 0 1rem 0', display: 'flex', flexDirection: 'column', gap: 14, height: '100%', overflowY: 'auto' }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, paddingBottom: 14, borderBottom: `1px solid ${color}40` }}>
        <span style={{ fontSize: 28, color, lineHeight: 1 }}>{glyph}</span>
        <div>
          <div style={{ fontSize: 22, fontWeight: 700, color: '#f0ead6', letterSpacing: '-0.01em' }}>{spell.name}</div>
          <div style={{ fontSize: 13, color: `${color}cc`, marginTop: 3 }}>
            {isCantrip ? 'Cantrip' : `Level ${spell.level}`} · {school.charAt(0).toUpperCase() + school.slice(1)}
          </div>
        </div>
      </div>

      {/* Stats chips */}
      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        {[
          { label: 'Cast', val: formatCastingTime(spell.casting_time) },
          { label: 'Range', val: formatRange(spell.range_type, spell.range_feet) },
          { label: 'Duration', val: formatDuration(spell.duration) },
        ].map(({ label, val }) => (
          <div key={label} style={{ display: 'flex', flexDirection: 'column', gap: 2, background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.07)', borderRadius: 8, padding: '6px 12px' }}>
            <span style={{ fontSize: 10, color: '#666', letterSpacing: '.06em', textTransform: 'uppercase' }}>{label}</span>
            <span style={{ fontSize: 13, color: '#d0c8b8' }}>{val}</span>
          </div>
        ))}
        {spell.concentration === 1 && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2, background: 'rgba(245,169,106,0.08)', border: '1px solid rgba(245,169,106,0.3)', borderRadius: 8, padding: '6px 12px' }}>
            <span style={{ fontSize: 10, color: '#f5a96a', textTransform: 'uppercase', letterSpacing: '.06em' }}>◉ Conc</span>
          </div>
        )}
        {spell.ritual === 1 && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2, background: 'rgba(126,200,227,0.08)', border: '1px solid rgba(126,200,227,0.3)', borderRadius: 8, padding: '6px 12px' }}>
            <span style={{ fontSize: 10, color: '#7ec8e3', textTransform: 'uppercase', letterSpacing: '.06em' }}>⊕ Ritual</span>
          </div>
        )}
      </div>

      {/* Components */}
      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        {spell.has_verbal === 1 && <span style={{ fontSize: 12, padding: '3px 10px', background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: 6, color: '#aaa' }}>V</span>}
        {spell.has_somatic === 1 && <span style={{ fontSize: 12, padding: '3px 10px', background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: 6, color: '#aaa' }}>S</span>}
        {spell.has_material === 1 && (
          <span style={{ fontSize: 12, padding: '3px 10px', background: 'rgba(255,255,255,0.04)', border: '1px solid rgba(255,255,255,0.08)', borderRadius: 6, color: '#aaa', maxWidth: 220, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            M ({spell.material_component || '—'})
          </span>
        )}
      </div>

      {/* Damage */}
      {spell.damage_die && (
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '8px 12px', background: 'rgba(255,255,255,0.03)', border: `1px solid ${dmgColor}33`, borderRadius: 8, fontSize: 15 }}>
          <span style={{ color: dmgColor, fontWeight: 700 }}>{spell.damage_die_count}{spell.damage_die}</span>
          <span style={{ color: dmgColor, opacity: 0.7, textTransform: 'capitalize' }}>{spell.damage_type}</span>
          {spell.save_type && (
            <span style={{ fontSize: 11, padding: '2px 8px', background: 'rgba(255,255,255,0.06)', borderRadius: 4, color: '#999', marginLeft: 'auto' }}>
              {spell.save_type.toUpperCase()} save
            </span>
          )}
          {spell.attack_type && (
            <span style={{ fontSize: 11, padding: '2px 8px', background: 'rgba(255,255,255,0.06)', borderRadius: 4, color: '#999', marginLeft: 'auto' }}>
              {spell.attack_type === 'ranged_spell' ? 'Ranged Spell Attack' : 'Melee Spell Attack'}
            </span>
          )}
        </div>
      )}

      {/* Description */}
      <div style={{ fontSize: 13, color: '#9098b8', lineHeight: 1.65 }}>{spell.description}</div>
    </div>
  )
}

export default function CharacterCreation({ onComplete }) {
  const [char, setChar] = useState({
    name: '',
    sex: '',
    race: '',
    species_subtype: '',
    player_class: '',
    background: '',
    background_asi: { str: 0, dex: 0, con: 0, int: 0, wis: 0, cha: 0 },
    stats: rollBlock(),
    equipment_choice: '',
    backstory: '',
    divine_order: '',         // Cleric: "Protector" | "Thaumaturge"
    thaumaturge_cantrip: '',  // Cleric Thaumaturge: chosen extra cantrip
    primal_order: '',         // Druid: "Warden" | "Magician"
    magician_cantrip: '',     // Druid Magician: chosen extra cantrip
    background_feat_id: '',
    background_feat_choices: {},
    starting_cantrips: [],
    starting_spells: [],
    magic_initiate_list: '',
    magic_initiate_cantrips: [],
    magic_initiate_spell: '',
  })

  const steps = buildSteps(
    char.race, char.background, char.player_class,
    char.divine_order, char.primal_order, char.background_feat_id
  )
  const [stepIndex, setStepIndex] = useState(0)
  const currentStep = steps[stepIndex]
  const [originFeats, setOriginFeats] = useState([])
  const [classSpells, setClassSpells] = useState([])
  const [miSpells, setMiSpells] = useState([])
  const [spellsLoading, setSpellsLoading] = useState(false)
  const [selectedSpell, setSelectedSpell] = useState(null)

  useEffect(() => {
    fetch('/api/feats?category=origin')
      .then(r => r.json())
      .then(d => setOriginFeats(d.feats || []))
      .catch(() => { })
  }, [])

  useEffect(() => {
    if (!['starting_cantrips', 'starting_spells'].includes(currentStep)) return
    if (!char.player_class || classSpells.length > 0) return
    setSpellsLoading(true)
    getSpellsByClass(char.player_class)
      .then(d => setClassSpells(d.spells || []))
      .catch(() => { })
      .finally(() => setSpellsLoading(false))
  }, [currentStep, char.player_class])

  useEffect(() => {
    if (currentStep !== 'magic_initiate_spells' || !char.magic_initiate_list) return
    if (miSpells.length > 0) return   // ← add this
    setSpellsLoading(true)
    getSpellsByClass(char.magic_initiate_list)
      .then(d => setMiSpells(d.spells || []))
      .catch(() => { })
      .finally(() => setSpellsLoading(false))
  }, [currentStep, char.magic_initiate_list])

  const upd = (k, v) => setChar(c => ({ ...c, [k]: v }))

  const bg = getBackgroundByName(char.background)
  const sp = getSpeciesByName(char.race)

  // ── ASI helpers ─────────────────────────────────────────────────────────────

  const asiTotal = () =>
    Object.values(char.background_asi).reduce((a, b) => a + b, 0)

  const changeAsi = (stat, delta) => {
    const current = char.background_asi[stat] || 0
    const total = asiTotal()
    if (delta > 0 && total >= 3) return
    if (delta > 0 && current >= 2) return  // no single stat > +2
    if (delta < 0 && current <= 0) return
    upd('background_asi', { ...char.background_asi, [stat]: current + delta })
  }

  const asiValid = () => {
    if (!bg) return false
    const total = asiTotal()
    if (total !== 3) return false
    const vals = bg.asi_stats.map(s => char.background_asi[s] || 0)
    // Valid: one stat at 2 and one at 1 (total=3), or all three at 1 (total=3)
    return true
  }

  // ── Equipment packages ───────────────────────────────────────────────────────

  const equipOptions = CLASS_EQUIPMENT[char.player_class] || []

  // ── Validation ───────────────────────────────────────────────────────────────

  const canAdvance = () => {
    switch (currentStep) {
      case 'name': return char.name.trim().length > 1
      case 'sex': return !!char.sex
      case 'species': return !!char.race
      case 'species_subtype': return !!char.species_subtype
      case 'class': return !!char.player_class
      case 'background': return !!char.background
      case 'background_asi': return asiValid()
      case 'stats': return true
      case 'equipment': return !!char.equipment_choice
      case 'backstory': return true
      case 'divine_order': return !!char.divine_order
      case 'thaumaturge_cantrip': return !!char.thaumaturge_cantrip
      case 'primal_order': return !!char.primal_order
      case 'magician_cantrip': return !!char.magician_cantrip
      case 'background_feat': return !!char.background_feat_id
      case 'starting_cantrips':
        return char.starting_cantrips.length === (STARTING_CANTRIP_COUNTS[char.player_class] || 0)
      case 'starting_spells':
        return char.starting_spells.length === (STARTING_SPELL_COUNTS[char.player_class] || 0)
      case 'magic_initiate_list':
        return !!char.magic_initiate_list
      case 'magic_initiate_spells':
        return char.magic_initiate_cantrips.length === 2 && !!char.magic_initiate_spell
      default: return true
    }
  }

  // ── Submit ────────────────────────────────────────────────────────────────────

  const handleComplete = () => {
    onComplete({
      player_name: char.name.trim(),
      player_sex: char.sex.toLowerCase(),
      player_race: char.race,
      player_species_subtype: char.species_subtype || null,
      player_class: char.player_class,
      player_background: char.background,
      background_feat_id: char.background_feat_id || null,
      background_feat_choices: Object.keys(char.background_feat_choices).length > 0
        ? JSON.stringify(char.background_feat_choices)
        : null,
      player_background_skill_1: bg?.skills[0] || 'Athletics',
      player_background_skill_2: bg?.skills[1] || 'Perception',
      player_background_tool: bg?.tool || 'None',
      player_background_asi: char.background_asi,
      player_stats: {
        str: char.stats[0],
        dex: char.stats[1],
        con: char.stats[2],
        int: char.stats[3],
        wis: char.stats[4],
        cha: char.stats[5],
      },
      player_backstory: char.backstory || null,
      equipment_choice: char.equipment_choice,
      divine_order: char.divine_order || null,
      thaumaturge_cantrip: char.thaumaturge_cantrip || null,
      primal_order: char.primal_order || null,
      magician_cantrip: char.magician_cantrip || null,
      starting_cantrips: char.starting_cantrips,
      starting_spells: char.starting_spells,
      magic_initiate_cantrips: char.magic_initiate_cantrips,
      magic_initiate_spell: char.magic_initiate_spell || null,
    })
  }

  // ── Render step ───────────────────────────────────────────────────────────────

  const renderStep = () => {
    switch (currentStep) {

      case 'name':
        return (
          <>
            <h2>Name Your Character</h2>
            <p className="card-sub">What name will echo through the ages?</p>
            <input
              className="inp"
              placeholder="Character name…"
              value={char.name}
              onChange={e => upd('name', e.target.value)}
              onKeyDown={e => e.key === 'Enter' && canAdvance() && setStepIndex(i => i + 1)}
              autoFocus
            />
          </>
        )

      case 'sex':
        return (
          <>
            <h2>Choose Your Sex</h2>
            <p className="card-sub">This determines the pronouns used throughout your adventure.</p>
            <div className="sex-grid">
              {SEX_OPTIONS.map(s => (
                <div
                  key={s}
                  className={`pick-card${char.sex === s ? ' sel' : ''}`}
                  onClick={() => upd('sex', s)}
                  style={{ textAlign: 'center', padding: '1.5rem 1rem' }}
                >
                  <div className="pick-card-name" style={{ fontSize: '1rem' }}>{s}</div>
                  <div className="pick-card-desc">
                    {s === 'Male' ? 'He / Him / His' : 'She / Her / Her'}
                  </div>
                </div>
              ))}
            </div>
          </>
        )

      case 'species':
        return (
          <>
            <h2>Choose Your Species</h2>
            <p className="card-sub">Your species shapes your innate abilities and traits.</p>
            <div style={{
              maxHeight: '420px', overflowY: 'auto',
              scrollbarWidth: 'thin', scrollbarColor: 'var(--gold) var(--surf)',
              paddingRight: '.25rem',
            }}>
              <div className="pick-grid-2">
                {SPECIES.map(s => (
                  <div
                    key={s.name}
                    className={`pick-card${char.race === s.name ? ' sel' : ''}`}
                    onClick={() => {
                      upd('race', s.name)
                      upd('species_subtype', '')
                    }}
                  >
                    <div className="pick-card-name">{s.name}</div>
                    <div className="pick-card-desc">{s.desc}</div>
                    {s.subtype && (
                      <div className="pick-card-meta">Choose {s.subtype.label} →</div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          </>
        )

      case 'species_subtype': {
        if (!sp?.subtype) return null
        return (
          <>
            <h2>Choose Your {sp.subtype.label}</h2>
            <p className="card-sub">
              {char.race === 'Dragonborn' && 'Your ancestry determines your breath weapon damage type and resistance.'}
              {char.race === 'Elf' && 'Your lineage grants innate spells and unique abilities.'}
              {char.race === 'Gnome' && 'Your lineage shapes your magical nature.'}
              {char.race === 'Goliath' && 'Your giant ancestry grants a supernatural boon.'}
              {char.race === 'Tiefling' && 'Your legacy determines your innate spells and damage resistance.'}
            </p>
            <div className="pick-grid-2">
              {sp.subtype.options.map(o => (
                <div
                  key={o.name}
                  className={`pick-card${char.species_subtype === o.name ? ' sel' : ''}`}
                  onClick={() => upd('species_subtype', o.name)}
                >
                  <div className="pick-card-name">{o.name}</div>
                  <div className="pick-card-desc">{o.desc}</div>
                </div>
              ))}
            </div>
          </>
        )
      }

      case 'class':
        return (
          <>
            <h2>Choose Your Class</h2>
            <p className="card-sub">Your class defines your role, abilities, and fighting style.</p>
            <div className="pick-grid">
              {CLASSES.map(c => (
                <div
                  key={c}
                  className={`pick${char.player_class === c ? ' sel' : ''}`}
                  onClick={() => {
                    upd('player_class', c)
                    upd('starting_cantrips', [])
                    upd('starting_spells', [])
                    setClassSpells([])
                  }}
                >
                  {c}
                </div>
              ))}
            </div>
          </>
        )

      case 'background':
        return (
          <>
            <h2>Choose Your Background</h2>
            <p className="card-sub">
              Your background grants two skill proficiencies, a tool proficiency, an Origin feat,
              and ability score increases applied in the next step.
            </p>
            <div style={{
              maxHeight: '420px', overflowY: 'auto',
              scrollbarWidth: 'thin', scrollbarColor: 'var(--gold) var(--surf)',
              paddingRight: '.25rem',
            }}>
              <div className="pick-grid-2">
                {BACKGROUNDS.map(b => (
                  <div
                    key={b.name}
                    className={`pick-card${char.background === b.name ? ' sel' : ''}`}
                    onClick={() => {
                      upd('background', b.name)
                      upd('background_asi', { str: 0, dex: 0, con: 0, int: 0, wis: 0, cha: 0 })
                    }}
                  >
                    <div className="pick-card-name">{b.name}</div>
                    <div className="pick-card-desc">{b.desc}</div>
                    <div className="pick-card-meta">
                      {b.skills.join(' · ')} · {b.feat}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </>
        )

      case 'background_asi': {
        if (!bg) return null
        const remaining = 3 - asiTotal()
        const statValues = bg.asi_stats.map(s => ({
          key: s,
          label: s.toUpperCase(),
          bonus: char.background_asi[s] || 0,
          base: char.stats[STAT_KEYS.indexOf(s)] || 10,
        }))

        return (
          <>
            <h2>Distribute Ability Score Increases</h2>
            <p className="card-sub">
              The <strong>{bg.name}</strong> background grants +3 to distribute among{' '}
              {bg.asi_stats.map(s => s.toUpperCase()).join(', ')}. You may apply +2 to one and +1 to
              another, or +1 to all three. No single score can exceed 20.
            </p>

            <div className="info-box" style={{ marginBottom: '1rem' }}>
              <strong>Points remaining: {remaining}</strong>
              {remaining === 0 && <span style={{ color: 'var(--grn)', marginLeft: '.5rem' }}>✓ All distributed</span>}
            </div>

            {statValues.map(({ key, label, bonus, base }) => (
              <div key={key} className="asi-row">
                <div>
                  <span className="asi-stat">{label}</span>
                  <span style={{ fontSize: '.75rem', color: 'var(--dim)', marginLeft: '.5rem' }}>
                    {base} → {base + bonus}
                    <span style={{ color: 'var(--gold)', marginLeft: '.3rem' }}>
                      ({formatModifier(base + bonus)})
                    </span>
                  </span>
                </div>
                <div className="asi-btns">
                  <button
                    className="asi-btn"
                    disabled={bonus <= 0}
                    onClick={() => changeAsi(key, -1)}
                  >−</button>
                  <span className="asi-val">+{bonus}</span>
                  <button
                    className="asi-btn"
                    disabled={remaining <= 0 || bonus >= 2}
                    onClick={() => changeAsi(key, 1)}
                  >+</button>
                </div>
              </div>
            ))}

            <div className="info-box" style={{ marginTop: '1rem', fontSize: '.72rem' }}>
              <strong>Background grants:</strong> {bg.skills.join(', ')} proficiency · {bg.tool} · {bg.feat} feat
            </div>
          </>
        )
      }

      case 'background_feat':
        return (
          <>
            <h2>Background Feat</h2>
            <p className="card-sub">Your background grants you one Origin feat.</p>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '.5rem', maxHeight: '340px', overflowY: 'auto' }}>
              {originFeats.map(feat => (
                <div key={feat.id}
                  className={`equip-card${char.background_feat_id === feat.id ? ' sel' : ''}`}
                  onClick={() => upd('background_feat_id', feat.id)}>
                  <div className="equip-label">{feat.name}</div>
                  <div className="equip-desc">{feat.description}</div>
                </div>
              ))}
            </div>
          </>
        )

      case 'stats': {
        const finalStats = STAT_KEYS.map((k, i) => {
          const base = char.stats[i]
          const bonus = char.background_asi[k] || 0
          return { key: k, base, bonus, final: Math.min(20, base + bonus) }
        })

        return (
          <>
            <h2>Forge Your Abilities</h2>
            <p className="card-sub">
              Roll 4d6 and drop the lowest for each score. Background bonuses are already applied below.
            </p>

            <div className="stat-g">
              {finalStats.map((s, i) => (
                <div key={s.key} className="stat-box" style={{ cursor: 'default' }}>
                  <div className="sl">{STAT_LABELS_ARRAY[i]}</div>
                  <div className="sv">{s.final}</div>
                  <div className="sm">{formatModifier(s.final)}</div>
                  {s.bonus > 0 && (
                    <div className="sa">+{s.bonus} bg</div>
                  )}
                </div>
              ))}
            </div>

            <div style={{ display: 'flex', gap: '.75rem', marginTop: '.75rem', alignItems: 'center' }}>
              <button className="btn-ghost" onClick={() => upd('stats', rollBlock())}>
                ⚄ Reroll
              </button>
              <div className="info-box" style={{ margin: 0, flex: 1, fontSize: '.72rem' }}>
                Hit Die: d{hitDieForClass(char.player_class)} · Starting HP:{' '}
                {hitDieForClass(char.player_class) + Math.floor((Math.min(20, char.stats[2] + (char.background_asi.con || 0)) - 10) / 2)} (d{hitDieForClass(char.player_class)} + CON mod)
              </div>
            </div>

            <textarea
              className="inp"
              style={{ resize: 'vertical', minHeight: '70px', marginTop: '1rem' }}
              placeholder="Optional: describe your character's history, motivations, or what set them on the path of adventure…"
              value={char.backstory}
              onChange={e => upd('backstory', e.target.value)}
            />
          </>
        )
      }

      case 'equipment': {
        const options = CLASS_EQUIPMENT[char.player_class] || []
        return (
          <>
            <h2>Starting Equipment</h2>
            <p className="card-sub">
              Choose your starting gear. Option B (and C for Fighter) is gold only — spend it however you like once your adventure begins.
            </p>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '.6rem' }}>
              {options.map(opt => (
                <div
                  key={opt.label}
                  className={`equip-card${char.equipment_choice === opt.label ? ' sel' : ''}`}
                  onClick={() => upd('equipment_choice', opt.label)}
                >
                  <div className="equip-label">Option {opt.label}</div>
                  <div className="equip-desc">{opt.desc}</div>
                </div>
              ))}
            </div>
          </>
        )
      }

      case 'backstory':
        return (
          <>
            <h2>Your Story</h2>
            <p className="card-sub">
              Optionally add any additional backstory, personality details, or notes about your character.
              This will be shared with the Dungeon Master.
            </p>
            <textarea
              className="inp"
              style={{ resize: 'vertical', minHeight: '120px' }}
              placeholder="Describe your character's history, motivations, personality, fears, or goals…"
              value={char.backstory}
              onChange={e => upd('backstory', e.target.value)}
              autoFocus
            />

            {/* Summary */}
            <div className="info-box" style={{ marginTop: '1.25rem', fontSize: '.76rem', lineHeight: 1.9 }}>
              <strong>Summary</strong><br />
              {char.name} · {char.sex} {char.race}{char.species_subtype ? ` (${char.species_subtype})` : ''}<br />
              {char.player_class} · {char.background}{bg ? ` · ${bg.feat}` : ''}
              {char.divine_order ? ` · ${char.divine_order}` : ''}
              {char.thaumaturge_cantrip ? ` (${char.thaumaturge_cantrip})` : ''}
              {char.primal_order ? ` · ${char.primal_order}` : ''}
              {char.magician_cantrip ? ` (${char.magician_cantrip})` : ''}<br />
              STR {Math.min(20, char.stats[0] + (char.background_asi.str || 0))} ·{' '}
              DEX {Math.min(20, char.stats[1] + (char.background_asi.dex || 0))} ·{' '}
              CON {Math.min(20, char.stats[2] + (char.background_asi.con || 0))} ·{' '}
              INT {Math.min(20, char.stats[3] + (char.background_asi.int || 0))} ·{' '}
              WIS {Math.min(20, char.stats[4] + (char.background_asi.wis || 0))} ·{' '}
              CHA {Math.min(20, char.stats[5] + (char.background_asi.cha || 0))}<br />
              Equipment: Option {char.equipment_choice}
            </div>
          </>
        )

      case 'divine_order':
        return (
          <>
            <h2>Divine Order</h2>
            <p className="card-sub">
              You have dedicated yourself to one of these sacred roles. This choice shapes your
              combat capabilities and connection to divine knowledge.
            </p>
            <div className="pick-grid-2">
              <div
                className={`pick-card${char.divine_order === 'Protector' ? ' sel' : ''}`}
                onClick={() => upd('divine_order', 'Protector')}
              >
                <div className="pick-card-name">Protector</div>
                <div className="pick-card-desc">
                  Trained for battle. You gain proficiency with Martial weapons and training
                  with Heavy armor — letting you wade into the front line as a warrior of your deity.
                </div>
              </div>
              <div
                className={`pick-card${char.divine_order === 'Thaumaturge' ? ' sel' : ''}`}
                onClick={() => upd('divine_order', 'Thaumaturge')}
              >
                <div className="pick-card-name">Thaumaturge</div>
                <div className="pick-card-desc">
                  Attuned to divine knowledge. You know one extra cantrip from the Cleric spell list,
                  and add your Wisdom modifier (min +1) to Intelligence (Arcana or Religion) checks.
                </div>
              </div>
            </div>
          </>
        )

      case 'thaumaturge_cantrip':
        return (
          <>
            <h2>Extra Cantrip</h2>
            <p className="card-sub">
              As a Thaumaturge, you know one extra cantrip from the Cleric spell list.
              Choose which one you begin your adventure with.
            </p>
            <div className="pick-grid-2">
              {CLERIC_CANTRIPS.map(c => (
                <div
                  key={c.name}
                  className={`pick-card${char.thaumaturge_cantrip === c.name ? ' sel' : ''}`}
                  onClick={() => upd('thaumaturge_cantrip', c.name)}
                >
                  <div className="pick-card-name">{c.name}</div>
                  <div className="pick-card-desc" style={{ display: 'flex', flexDirection: 'column', gap: '.2rem' }}>
                    <span style={{ color: SCHOOL_COLORS_CC[c.school] || 'var(--dim)', fontSize: '.72rem' }}>
                      {c.school}
                    </span>
                    {c.note && (
                      <span style={{ fontSize: '.7rem', color: 'var(--dim)' }}>{c.note}</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </>
        )

      case 'primal_order':
        return (
          <>
            <h2>Primal Order</h2>
            <p className="card-sub">
              You have dedicated yourself to one of these sacred roles. This choice shapes
              your combat abilities and your connection to mystical knowledge.
            </p>
            <div className="pick-grid-2">
              <div
                className={`pick-card${char.primal_order === 'Warden' ? ' sel' : ''}`}
                onClick={() => {
                  upd('primal_order', 'Warden')
                  upd('magician_cantrip', '') // clear if switching
                }}
              >
                <div className="pick-card-name">Warden</div>
                <div className="pick-card-desc">
                  Trained for battle. You gain proficiency with Martial weapons and training
                  with Medium armor, letting you protect the natural world up close.
                </div>
              </div>
              <div
                className={`pick-card${char.primal_order === 'Magician' ? ' sel' : ''}`}
                onClick={() => upd('primal_order', 'Magician')}
              >
                <div className="pick-card-name">Magician</div>
                <div className="pick-card-desc">
                  Attuned to mystical forces. You know one extra cantrip from the Druid
                  spell list, and add your Wisdom modifier (min +1) to Intelligence
                  (Arcana or Nature) checks.
                </div>
              </div>
            </div>
          </>
        )

      case 'magician_cantrip':
        return (
          <>
            <h2>Extra Cantrip</h2>
            <p className="card-sub">
              As a Magician, you know one extra cantrip from the Druid spell list.
              Choose which one you begin your adventure with.
            </p>
            <div className="pick-grid-2">
              {DRUID_CANTRIPS_CC.map(c => (
                <div
                  key={c.name}
                  className={`pick-card${char.magician_cantrip === c.name ? ' sel' : ''}`}
                  onClick={() => upd('magician_cantrip', c.name)}
                >
                  <div className="pick-card-name">{c.name}</div>
                  <div className="pick-card-desc" style={{ display: 'flex', flexDirection: 'column', gap: '.2rem' }}>
                    <span style={{ color: SCHOOL_COLORS_CC[c.school] || 'var(--dim)', fontSize: '.72rem' }}>
                      {c.school}
                    </span>
                    {c.note && (
                      <span style={{ fontSize: '.7rem', color: 'var(--dim)' }}>{c.note}</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </>
        )

      case 'starting_cantrips': {
        const needed = STARTING_CANTRIP_COUNTS[char.player_class] || 0
        const cantrips = classSpells.filter(s => s.level === 0)
        const toggle = (spell) => {
          const already = char.starting_cantrips.includes(spell.id)
          if (already) upd('starting_cantrips', char.starting_cantrips.filter(id => id !== spell.id))
          else if (char.starting_cantrips.length < needed) upd('starting_cantrips', [...char.starting_cantrips, spell.id])
        }
        return (
          <>
            <h2>Starting Cantrips</h2>
            <p className="card-sub">
              Choose {needed} cantrip{needed !== 1 ? 's' : ''} from the {char.player_class} spell list.
              &nbsp;({char.starting_cantrips.length}/{needed} chosen)
            </p>
            {spellsLoading ? <p className="card-sub">Loading spells…</p> : (
              <div style={{ display: 'flex', gap: '1rem', height: 340 }}>
                <div style={{ width: 220, flexShrink: 0, overflowY: 'auto', scrollbarWidth: 'thin', scrollbarColor: 'var(--gold) var(--surf)' }}>
                  {cantrips.map(s => (
                    <CreationSpellCard
                      key={s.id} spell={s}
                      isSelected={char.starting_cantrips.includes(s.id)}
                      isDisabled={!char.starting_cantrips.includes(s.id) && char.starting_cantrips.length >= needed}
                      onClick={spell => { toggle(spell); setSelectedSpell(spell) }}
                    />
                  ))}
                </div>
                <div style={{ flex: 1, borderLeft: '1px solid rgba(255,255,255,0.06)', paddingLeft: '1rem', overflowY: 'auto' }}>
                  <CreationSpellDetail spell={selectedSpell} />
                </div>
              </div>
            )}
          </>
        )
      }

      case 'starting_spells': {
        const needed = STARTING_SPELL_COUNTS[char.player_class] || 0
        const spells = classSpells.filter(s => s.level === 1)
        const toggle = (spell) => {
          const already = char.starting_spells.includes(spell.id)
          if (already) upd('starting_spells', char.starting_spells.filter(id => id !== spell.id))
          else if (char.starting_spells.length < needed) upd('starting_spells', [...char.starting_spells, spell.id])
        }
        return (
          <>
            <h2>Starting {char.player_class === 'Wizard' ? 'Spellbook' : 'Spells'}</h2>
            <p className="card-sub">
              Choose {needed} level 1 spell{needed !== 1 ? 's' : ''} for your {char.player_class === 'Wizard' ? 'spellbook' : 'known spells'}.
              &nbsp;({char.starting_spells.length}/{needed} chosen)
            </p>
            {spellsLoading ? <p className="card-sub">Loading spells…</p> : (
              <div style={{ display: 'flex', gap: '1rem', height: 340 }}>
                <div style={{ width: 220, flexShrink: 0, overflowY: 'auto', scrollbarWidth: 'thin', scrollbarColor: 'var(--gold) var(--surf)' }}>
                  {spells.map(s => (
                    <CreationSpellCard
                      key={s.id} spell={s}
                      isSelected={char.starting_spells.includes(s.id)}
                      isDisabled={!char.starting_spells.includes(s.id) && char.starting_spells.length >= needed}
                      onClick={spell => { toggle(spell); setSelectedSpell(spell) }}
                    />
                  ))}
                </div>
                <div style={{ flex: 1, borderLeft: '1px solid rgba(255,255,255,0.06)', paddingLeft: '1rem', overflowY: 'auto' }}>
                  <CreationSpellDetail spell={selectedSpell} />
                </div>
              </div>
            )}
          </>
        )
      }

      case 'magic_initiate_list':
        return (
          <>
            <h2>Magic Initiate</h2>
            <p className="card-sub">
              Choose a spell list to learn from. You'll pick two cantrips and one level 1 spell from it.
            </p>
            <div className="pick-grid-2">
              {MI_LISTS.map(list => (
                <div
                  key={list}
                  className={`pick-card${char.magic_initiate_list === list ? ' sel' : ''}`}
                  onClick={() => {
                    upd('magic_initiate_list', list)
                    upd('magic_initiate_cantrips', [])
                    upd('magic_initiate_spell', '')
                    setMiSpells([])
                  }}
                >
                  <div className="pick-card-name">{list} List</div>
                  <div className="pick-card-desc">
                    {list === 'Cleric' && 'Divine magic — healing, radiant damage, protection'}
                    {list === 'Druid' && 'Nature magic — elementals, animals, plants'}
                    {list === 'Wizard' && 'Arcane magic — the broadest selection of spells'}
                  </div>
                </div>
              ))}
            </div>
          </>
        )

      case 'magic_initiate_spells': {
        const miCantrips = miSpells.filter(s => s.level === 0)
        const miLevel1 = miSpells.filter(s => s.level === 1)
        const toggleMiCantrip = (spell) => {
          const already = char.magic_initiate_cantrips.includes(spell.id)
          if (already) upd('magic_initiate_cantrips', char.magic_initiate_cantrips.filter(id => id !== spell.id))
          else if (char.magic_initiate_cantrips.length < 2) upd('magic_initiate_cantrips', [...char.magic_initiate_cantrips, spell.id])
        }
        return (
          <>
            <h2>Magic Initiate Spells</h2>
            <p className="card-sub">
              Choose 2 cantrips and 1 level 1 spell from the {char.magic_initiate_list} list.
            </p>
            {spellsLoading ? <p className="card-sub">Loading spells…</p> : (
              <div style={{ display: 'flex', gap: '1rem', height: 380 }}>
                <div style={{ width: 220, flexShrink: 0, overflowY: 'auto', scrollbarWidth: 'thin', scrollbarColor: 'var(--gold) var(--surf)' }}>
                  <div style={{ fontSize: '.72rem', color: 'var(--goldl)', fontFamily: "'Cinzel',serif", letterSpacing: '.05em', padding: '4px 0 6px' }}>
                    Cantrips ({char.magic_initiate_cantrips.length}/2)
                  </div>
                  {miCantrips.map(s => (
                    <CreationSpellCard
                      key={s.id} spell={s}
                      isSelected={char.magic_initiate_cantrips.includes(s.id)}
                      isDisabled={!char.magic_initiate_cantrips.includes(s.id) && char.magic_initiate_cantrips.length >= 2}
                      onClick={spell => { toggleMiCantrip(spell); setSelectedSpell(spell) }}
                    />
                  ))}
                  <div style={{ fontSize: '.72rem', color: 'var(--goldl)', fontFamily: "'Cinzel',serif", letterSpacing: '.05em', padding: '10px 0 6px', borderTop: '1px solid rgba(255,255,255,0.06)', marginTop: 4 }}>
                    Level 1 Spell {char.magic_initiate_spell ? '✓' : '(choose 1)'}
                  </div>
                  {miLevel1.map(s => (
                    <CreationSpellCard
                      key={s.id} spell={s}
                      isSelected={char.magic_initiate_spell === s.id}
                      isDisabled={false}
                      onClick={spell => { upd('magic_initiate_spell', spell.id); setSelectedSpell(spell) }}
                    />
                  ))}
                </div>
                <div style={{ flex: 1, borderLeft: '1px solid rgba(255,255,255,0.06)', paddingLeft: '1rem', overflowY: 'auto' }}>
                  <CreationSpellDetail spell={selectedSpell} />
                </div>
              </div>
            )}
          </>
        )
      }

      default:
        return null
    }
  }

  const isLast = stepIndex === steps.length - 1

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: CREATION_STYLES }} />
      <div className="creation">
        <div className="card">

          {/* Step dots */}
          <div className="steps">
            {steps.map((_, i) => (
              <div key={i} className={`step${i <= stepIndex ? ' on' : ''}`} />
            ))}
          </div>

          {renderStep()}

          <div className="cnav">
            {stepIndex > 0
              ? <button className="btn-ghost" onClick={() => { setStepIndex(i => i - 1); setSelectedSpell(null) }}>← Back</button>
              : <div />
            }
            {!isLast
              ? <button className="btn-gold" disabled={!canAdvance()} onClick={() => { setStepIndex(i => i + 1); setSelectedSpell(null) }}>
                Continue →
              </button>
              : <button className="btn-gold" disabled={!canAdvance()} onClick={handleComplete}>
                Begin Adventure ⚔
              </button>
            }
          </div>

        </div>
      </div>
    </>
  )
}