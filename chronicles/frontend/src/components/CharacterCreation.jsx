import { useState } from 'react'
import { STYLES } from '../styles.js'
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
`

// ─── Step definitions ─────────────────────────────────────────────────────────

// We build the step list dynamically based on selections
// Base steps: name, sex, species, [species_subtype?], class, background, background_asi, stats, equipment, backstory
const BASE_STEPS = ['name', 'sex', 'species', 'class', 'background', 'background_asi', 'stats', 'equipment', 'backstory']

function buildSteps(race, background) {
  const steps = ['name', 'sex', 'species']
  const sp = getSpeciesByName(race)
  if (sp?.subtype) steps.push('species_subtype')
  steps.push('class', 'background', 'background_asi', 'stats', 'equipment', 'backstory')
  return steps
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
  })

  const steps = buildSteps(char.race, char.background)
  const [stepIndex, setStepIndex] = useState(0)
  const currentStep = steps[stepIndex]

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
      player_background_feat: bg?.feat || null,
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
                  onClick={() => upd('player_class', c)}
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
              {char.player_class} · {char.background}{bg ? ` · ${bg.feat}` : ''}<br />
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
              ? <button className="btn-ghost" onClick={() => setStepIndex(i => i - 1)}>← Back</button>
              : <div />
            }
            {!isLast
              ? <button className="btn-gold" disabled={!canAdvance()} onClick={() => setStepIndex(i => i + 1)}>
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