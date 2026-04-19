import { useState } from 'react'
import { STYLES } from '../styles.js'

const RACES = ["Human","Elf","Dwarf","Halfling","Half-Elf","Half-Orc","Gnome","Tiefling","Dragonborn"]
const CLASSES = ["Barbarian","Bard","Cleric","Druid","Fighter","Monk","Paladin","Ranger","Rogue","Sorcerer","Warlock","Wizard"]
const BACKGROUNDS = ["Acolyte","Charlatan","Criminal","Entertainer","Folk Hero","Hermit","Noble","Outlander","Sage","Soldier","Urchin"]
const STATS = ["STR","DEX","CON","INT","WIS","CHA"]
const STAT_KEYS = ["str","dex","con","int","wis","cha"]

const d = (n) => Math.floor(Math.random() * n) + 1
const mod = (v) => Math.floor((v - 10) / 2)
const fmt = (v) => { const m = mod(v); return (m >= 0 ? "+" : "") + m }
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
  max-width: 680px; width: 100%;
}
.card h2 {
  font-family: 'Cinzel', serif; color: var(--gold);
  font-size: 1.3rem; margin-bottom: 1.5rem;
  padding-bottom: .75rem; border-bottom: 1px solid var(--bord);
}
.steps { display: flex; gap: .45rem; justify-content: center; margin-bottom: 2rem; }
.step { width: 32px; height: 3px; border-radius: 2px; background: var(--bord); transition: background .3s; }
.step.on { background: var(--gold); }
.stat-g { display: grid; grid-template-columns: repeat(3,1fr); gap: .65rem; margin: .75rem 0; }
.stat-box {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .7rem; text-align: center;
}
.stat-box .sl { font-family: 'Cinzel', serif; font-size: .65rem; letter-spacing: .12em; color: var(--dim); margin-bottom: .2rem; }
.stat-box .sv { font-size: 1.6rem; color: var(--goldl); font-weight: bold; line-height: 1; }
.stat-box .sm { font-size: .75rem; color: var(--dim); margin-top: .15rem; }
.cnav { display: flex; justify-content: space-between; align-items: center; margin-top: 2rem; }
`

export default function CharacterCreation({ onComplete }) {
  const [step, setStep] = useState(0)
  const [char, setChar] = useState({
    name: '', race: '', class: '', background: '',
    stats: rollBlock(), backstory: '', starting_gold: 15
  })

  const upd = (k, v) => setChar(c => ({ ...c, [k]: v }))

  const canNext = () => {
    if (step === 0) return char.name.trim().length > 1
    if (step === 1) return !!(char.race && char.class)
    if (step === 2) return !!char.background
    return true
  }

  const handleComplete = () => {
    onComplete({
      player_name: char.name,
      player_race: char.race,
      player_class: char.class,
      player_background: char.background,
      player_stats: {
        str: char.stats[0], dex: char.stats[1], con: char.stats[2],
        int: char.stats[3], wis: char.stats[4], cha: char.stats[5]
      },
      player_backstory: char.backstory || null,
      starting_gold: char.starting_gold
    })
  }

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: CREATION_STYLES }} />
      <div className="creation">
        <div className="card">
          <div className="steps">
            {[0, 1, 2, 3].map(i => <div key={i} className={`step${i <= step ? ' on' : ''}`} />)}
          </div>

          {step === 0 && <>
            <h2>What is your name, adventurer?</h2>
            <input
              className="inp" autoFocus
              placeholder="Enter your character's name…"
              value={char.name}
              onChange={e => upd('name', e.target.value)}
              onKeyDown={e => { if (e.key === 'Enter' && canNext()) setStep(1) }}
            />
            <p style={{ marginTop: '1rem', fontSize: '.82rem', color: 'var(--dim)', fontStyle: 'italic' }}>
              This name will echo through the realm's history.
            </p>
          </>}

          {step === 1 && <>
            <h2>Choose Your Heritage & Path</h2>
            <p className="sec-title" style={{ marginBottom: '.5rem' }}>Race</p>
            <div className="pick-grid">
              {RACES.map(r => (
                <div key={r} className={`pick${char.race === r ? ' sel' : ''}`} onClick={() => upd('race', r)}>{r}</div>
              ))}
            </div>
            <p className="sec-title" style={{ margin: '1.1rem 0 .5rem' }}>Class</p>
            <div className="pick-grid">
              {CLASSES.map(cl => (
                <div key={cl} className={`pick${char.class === cl ? ' sel' : ''}`} onClick={() => upd('class', cl)}>{cl}</div>
              ))}
            </div>
          </>}

          {step === 2 && <>
            <h2>Choose Your Background</h2>
            <p style={{ fontSize: '.85rem', color: 'var(--dim)', fontStyle: 'italic', marginBottom: '1rem' }}>
              Your background grants proficiencies and shapes how the world sees you.
            </p>
            <div className="pick-grid">
              {BACKGROUNDS.map(b => (
                <div key={b} className={`pick${char.background === b ? ' sel' : ''}`} onClick={() => upd('background', b)}>{b}</div>
              ))}
            </div>
          </>}

          {step === 3 && <>
            <h2>Forge Your Legend</h2>
            <p style={{ fontSize: '.82rem', color: 'var(--dim)', fontStyle: 'italic', marginBottom: '.75rem' }}>
              Roll your ability scores (4d6, drop lowest), then optionally shape your origin.
            </p>
            <div className="stat-g">
              {STATS.map((s, i) => (
                <div key={s} className="stat-box">
                  <div className="sl">{s}</div>
                  <div className="sv">{char.stats[i]}</div>
                  <div className="sm">{fmt(char.stats[i])}</div>
                </div>
              ))}
            </div>
            <button className="btn-ghost" style={{ marginBottom: '1.1rem' }} onClick={() => upd('stats', rollBlock())}>
              ⚄ Reroll Stats
            </button>
            <textarea
              className="inp"
              style={{ resize: 'vertical', minHeight: '80px' }}
              placeholder="Optional: Describe your character's history, motivations, or what set them on the path of adventure…"
              value={char.backstory}
              onChange={e => upd('backstory', e.target.value)}
            />
          </>}

          <div className="cnav">
            {step > 0
              ? <button className="btn-ghost" onClick={() => setStep(s => s - 1)}>← Back</button>
              : <div />
            }
            {step < 3
              ? <button className="btn-gold" disabled={!canNext()} onClick={() => setStep(s => s + 1)}>Continue →</button>
              : <button className="btn-gold" onClick={handleComplete}>Begin Adventure ⚔</button>
            }
          </div>
        </div>
      </div>
    </>
  )
}