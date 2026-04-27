import { useState } from 'react'
import { STYLES } from '../styles.js'
import {
  FIGHTER_SUBCLASSES, ALL_MANEUVERS, STAT_KEYS, STAT_LABELS,
  FIGHTER_ASI_LEVELS, getFighterFeatures
} from '../constants.js'

const MODAL_STYLES = `
${STYLES}
.lu-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0,0,0,.85);
  display: flex; align-items: center; justify-content: center;
  padding: 1.5rem;
}
.lu-modal {
  background: var(--surf); border: 1px solid var(--gold);
  border-radius: 3px; width: 100%; max-width: 560px;
  max-height: 90vh; overflow-y: auto;
  display: flex; flex-direction: column;
}
.lu-header {
  padding: 1.5rem 1.8rem 1rem;
  border-bottom: 1px solid var(--bord);
}
.lu-title {
  font-family: 'Cinzel', serif; font-size: 1.3rem;
  color: var(--goldl); letter-spacing: .1em;
  margin-bottom: .3rem;
}
.lu-subtitle {
  font-size: .78rem; color: var(--dim); font-style: italic;
}
.lu-body { padding: 1.4rem 1.8rem; flex: 1; }
.lu-footer {
  padding: 1rem 1.8rem;
  border-top: 1px solid var(--bord);
  display: flex; justify-content: space-between; align-items: center;
  gap: 1rem;
}
.lu-step-label {
  font-family: 'Cinzel', serif; font-size: .6rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--dim); margin-bottom: .8rem;
}
.lu-feature-list { display: flex; flex-direction: column; gap: .5rem; margin-bottom: 1rem; }
.lu-feature {
  background: var(--elev); border: 1px solid var(--bord);
  border-left: 3px solid var(--gold);
  border-radius: 0 2px 2px 0; padding: .6rem .9rem;
  font-size: .82rem; line-height: 1.6;
}
.lu-feature-name {
  font-family: 'Cinzel', serif; font-size: .72rem;
  color: var(--goldl); letter-spacing: .06em; margin-bottom: .2rem;
}
.lu-stat-grid {
  display: grid; grid-template-columns: repeat(3, 1fr); gap: .5rem;
  margin: .5rem 0 1rem;
}
.lu-stat {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .6rem; text-align: center;
  cursor: pointer; transition: all .15s;
}
.lu-stat:hover, .lu-stat.sel {
  border-color: var(--gold); background: rgba(200,150,42,.08);
}
.lu-stat-label {
  font-family: 'Cinzel', serif; font-size: .6rem;
  letter-spacing: .1em; color: var(--dim); margin-bottom: .2rem;
}
.lu-stat-val { font-size: 1.4rem; color: var(--goldl); line-height: 1; }
.lu-stat-mod { font-size: .72rem; color: var(--dim); margin-top: .1rem; }
.lu-subclass-grid {
  display: grid; grid-template-columns: 1fr 1fr; gap: .6rem; margin: .5rem 0;
}
.lu-subclass {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .9rem 1rem;
  cursor: pointer; transition: all .15s; text-align: left;
}
.lu-subclass:hover, .lu-subclass.sel {
  border-color: var(--gold); background: rgba(200,150,42,.08);
}
.lu-subclass-name {
  font-family: 'Cinzel', serif; font-size: .82rem;
  color: var(--goldl); margin-bottom: .3rem;
}
.lu-subclass-desc { font-size: .74rem; color: var(--dim); line-height: 1.55; }
.lu-maneuver-grid {
  display: grid; grid-template-columns: 1fr 1fr; gap: .4rem;
  max-height: 280px; overflow-y: auto;
  margin: .5rem 0;
  scrollbar-width: thin; scrollbar-color: var(--gold) var(--surf);
}
.lu-maneuver {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .5rem .7rem;
  cursor: pointer; transition: all .15s; font-size: .78rem;
  color: var(--dim);
}
.lu-maneuver:hover, .lu-maneuver.sel {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.08);
}
.lu-maneuver.disabled {
  opacity: .35; cursor: not-allowed; pointer-events: none;
}
.lu-hp-gained {
  display: flex; align-items: baseline; gap: .5rem;
  margin-bottom: 1rem;
}
.lu-hp-num {
  font-family: 'Cinzel', serif; font-size: 2rem;
  color: var(--goldl); line-height: 1;
}
.lu-hp-label { font-size: .8rem; color: var(--dim); }
.lu-info-row {
  display: flex; justify-content: space-between;
  font-size: .78rem; padding: .3rem 0;
  border-bottom: 1px solid var(--bord); color: var(--dim);
}
.lu-info-row:last-child { border-bottom: none; }
.lu-info-val { color: var(--goldl); font-family: 'Cinzel', serif; }
.lu-step-dots {
  display: flex; gap: .4rem; align-items: center;
}
.lu-dot {
  width: 6px; height: 6px; border-radius: 50%;
  background: var(--bord); transition: background .2s;
}
.lu-dot.active { background: var(--gold); }
.lu-asi-mode {
  display: flex; gap: .5rem; margin-bottom: 1rem;
}
.lu-mode-btn {
  flex: 1; background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .5rem; cursor: pointer;
  font-family: 'Cinzel', serif; font-size: .68rem;
  letter-spacing: .08em; color: var(--dim); transition: all .15s;
  text-align: center;
}
.lu-mode-btn:hover, .lu-mode-btn.sel {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.07);
}
`

const mod = v => Math.floor((v - 10) / 2)
const fmt = v => { const m = mod(v); return (m >= 0 ? '+' : '') + m }

function maneuversToGainAtLevel(level) {
  // Returns how many NEW maneuvers to pick at this level
  if (level === 3) return 3
  if (level === 7 || level === 10 || level === 15) return 2
  return 0
}

export default function LevelUpModal({ player, levelUpResult, onComplete, onClose }) {
  const {
    new_level, hp_gained, new_max_hp, new_proficiency_bonus,
    asi_available, subclass_choice_required, new_features,
    second_wind_uses, weapon_mastery_count, extra_attacks,
    action_surge_uses, indomitable_max,
  } = levelUpResult

  const isFighter = player.class === 'Fighter'
  const isBattleMaster = player.subclass === 'Battle Master'
  const maneuversToGain = isBattleMaster ? maneuversToGainAtLevel(new_level) : 0
  const canReplaceManeuver = isBattleMaster && new_level >= 7

  // Build steps
  const steps = ['summary']
  if (subclass_choice_required) steps.push('subclass')
  if (asi_available) steps.push('asi')
  if (maneuversToGain > 0) steps.push('maneuvers')

  const [stepIndex, setStepIndex] = useState(0)
  const [subclass, setSubclass] = useState(null)
  const [asiMode, setAsiMode] = useState('+2') // '+2' or '+1+1'
  const [asi1, setAsi1] = useState(null)
  const [asi2, setAsi2] = useState(null)
  const [selectedManeuvers, setSelectedManeuvers] = useState([])
  const [replacedManeuver, setReplacedManeuver] = useState(null)
  const [replaceMode, setReplaceMode] = useState(false)

  const currentStep = steps[stepIndex]
  const isLast = stepIndex === steps.length - 1

  const canAdvance = () => {
    if (currentStep === 'summary') return true
    if (currentStep === 'subclass') return subclass !== null && subclass !== 'Eldritch Knight'
    if (currentStep === 'asi') {
      if (asiMode === '+2') return asi1 !== null
      return asi1 !== null && asi2 !== null && asi1 !== asi2
    }
    if (currentStep === 'maneuvers') return selectedManeuvers.length === maneuversToGain
    return true
  }

  const handleConfirm = () => {
    const choices = {}
    if (subclass) choices.subclass = subclass
    if (asi_available && asi1) {
      choices.asi_stat1 = asi1
      if (asiMode === '+1+1' && asi2) choices.asi_stat2 = asi2
      else choices.asi_stat2 = asi1 // same stat = +2
    }
    if (selectedManeuvers.length > 0) choices.new_maneuvers = selectedManeuvers
    if (replacedManeuver) choices.replaced_maneuver = replacedManeuver
    onComplete(choices)
  }

  const toggleManeuver = (m) => {
    if (selectedManeuvers.includes(m)) {
      setSelectedManeuvers(s => s.filter(x => x !== m))
    } else if (selectedManeuvers.length < maneuversToGain) {
      setSelectedManeuvers(s => [...s, m])
    }
  }

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: MODAL_STYLES }} />
      <div className="lu-overlay">
        <div className="lu-modal">

          <div className="lu-header">
            <div className="lu-title">Level {new_level}!</div>
            <div className="lu-subtitle">
              {player.name} — {player.race} {player.class}
              {player.subclass ? ` · ${player.subclass}` : ''}
            </div>
          </div>

          <div className="lu-body">

            {/* ── Summary ── */}
            {currentStep === 'summary' && (
              <>
                <div className="lu-step-label">What's new</div>

                <div className="lu-hp-gained">
                  <div className="lu-hp-num">+{hp_gained}</div>
                  <div className="lu-hp-label">hit points — new maximum: {new_max_hp}</div>
                </div>

                <div style={{ marginBottom: '1rem' }}>
                  <div className="lu-info-row">
                    <span>Proficiency Bonus</span>
                    <span className="lu-info-val">+{new_proficiency_bonus}</span>
                  </div>
                  {isFighter && (
                    <>
                      <div className="lu-info-row">
                        <span>Second Wind Uses</span>
                        <span className="lu-info-val">{second_wind_uses}</span>
                      </div>
                      <div className="lu-info-row">
                        <span>Attacks per Action</span>
                        <span className="lu-info-val">{extra_attacks}</span>
                      </div>
                      <div className="lu-info-row">
                        <span>Weapon Masteries</span>
                        <span className="lu-info-val">{weapon_mastery_count}</span>
                      </div>
                      {action_surge_uses > 0 && (
                        <div className="lu-info-row">
                          <span>Action Surge Uses</span>
                          <span className="lu-info-val">{action_surge_uses}</span>
                        </div>
                      )}
                      {indomitable_max > 0 && (
                        <div className="lu-info-row">
                          <span>Indomitable Uses</span>
                          <span className="lu-info-val">{indomitable_max}</span>
                        </div>
                      )}
                    </>
                  )}
                </div>

                {new_features.length > 0 && (
                  <>
                    <div className="lu-step-label">New features</div>
                    <div className="lu-feature-list">
                      {new_features.map(f => (
                        <div key={f} className="lu-feature">
                          <div className="lu-feature-name">{f}</div>
                        </div>
                      ))}
                    </div>
                  </>
                )}
              </>
            )}

            {/* ── Subclass ── */}
            {currentStep === 'subclass' && (
              <>
                <div className="lu-step-label">Choose your Fighter subclass</div>
                <div className="lu-subclass-grid">
                  {FIGHTER_SUBCLASSES.map(sc => (
                    <div
                      key={sc.name}
                      className={`lu-subclass${subclass === sc.name ? ' sel' : ''}${sc.name === 'Eldritch Knight' ? ' disabled' : ''}`}
                      onClick={() => sc.name !== 'Eldritch Knight' && setSubclass(sc.name)}
                      style={sc.name === 'Eldritch Knight' ? { opacity: .4, cursor: 'not-allowed' } : {}}
                    >
                      <div className="lu-subclass-name">{sc.name}</div>
                      <div className="lu-subclass-desc">{sc.desc}</div>
                    </div>
                  ))}
                </div>
              </>
            )}

            {/* ── ASI ── */}
            {currentStep === 'asi' && (
              <>
                <div className="lu-step-label">Ability Score Improvement</div>
                <div className="lu-asi-mode">
                  <button
                    className={`lu-mode-btn${asiMode === '+2' ? ' sel' : ''}`}
                    onClick={() => { setAsiMode('+2'); setAsi2(null) }}
                  >
                    +2 to one stat
                  </button>
                  <button
                    className={`lu-mode-btn${asiMode === '+1+1' ? ' sel' : ''}`}
                    onClick={() => setAsiMode('+1+1')}
                  >
                    +1 to two stats
                  </button>
                </div>

                {asiMode === '+2' ? (
                  <>
                    <div style={{ fontSize: '.76rem', color: 'var(--dim)', marginBottom: '.6rem' }}>
                      Choose one stat to increase by 2
                    </div>
                    <div className="lu-stat-grid">
                      {STAT_KEYS.map(k => (
                        <div
                          key={k}
                          className={`lu-stat${asi1 === k ? ' sel' : ''}`}
                          onClick={() => setAsi1(k)}
                        >
                          <div className="lu-stat-label">{STAT_LABELS[k]}</div>
                          <div className="lu-stat-val">{player[k]}</div>
                          <div className="lu-stat-mod">{fmt(player[k])} → {fmt(player[k] + 2)}</div>
                        </div>
                      ))}
                    </div>
                  </>
                ) : (
                  <>
                    <div style={{ fontSize: '.76rem', color: 'var(--dim)', marginBottom: '.6rem' }}>
                      Choose two different stats to increase by 1 each
                      {asi1 && !asi2 && <span style={{ color: 'var(--gold)' }}> — now choose second stat</span>}
                    </div>
                    <div className="lu-stat-grid">
                      {STAT_KEYS.map(k => {
                        const isFirst = asi1 === k
                        const isSecond = asi2 === k
                        const isSel = isFirst || isSecond
                        return (
                          <div
                            key={k}
                            className={`lu-stat${isSel ? ' sel' : ''}`}
                            onClick={() => {
                              if (isFirst) { setAsi1(asi2); setAsi2(null) }
                              else if (isSecond) { setAsi2(null) }
                              else if (!asi1) { setAsi1(k) }
                              else if (!asi2 && k !== asi1) { setAsi2(k) }
                            }}
                          >
                            <div className="lu-stat-label">{STAT_LABELS[k]}</div>
                            <div className="lu-stat-val">{player[k]}</div>
                            <div className="lu-stat-mod">
                              {isSel ? `${fmt(player[k])} → ${fmt(player[k] + 1)}` : fmt(player[k])}
                            </div>
                          </div>
                        )
                      })}
                    </div>
                  </>
                )}
              </>
            )}

            {/* ── Maneuvers ── */}
            {currentStep === 'maneuvers' && (
              <>
                <div className="lu-step-label">
                  Choose {maneuversToGain} maneuver{maneuversToGain > 1 ? 's' : ''}
                  {selectedManeuvers.length > 0 && ` (${selectedManeuvers.length}/${maneuversToGain} selected)`}
                </div>

                {canReplaceManeuver && (
                  <div style={{ marginBottom: '.8rem' }}>
                    <label style={{ display: 'flex', alignItems: 'center', gap: '.5rem', cursor: 'pointer', fontSize: '.78rem', color: 'var(--dim)' }}>
                      <input
                        type="checkbox"
                        checked={replaceMode}
                        onChange={e => { setReplaceMode(e.target.checked); setReplacedManeuver(null) }}
                      />
                      Also replace one known maneuver
                    </label>
                    {replaceMode && (
                      <>
                        <div style={{ fontSize: '.72rem', color: 'var(--dim)', margin: '.4rem 0 .3rem' }}>
                          Select a known maneuver to replace:
                        </div>
                        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '.3rem', marginBottom: '.5rem' }}>
                          {(player.known_maneuvers || []).map(m => (
                            <button
                              key={m.maneuver_name}
                              className={`btn-sm${replacedManeuver === m.maneuver_name ? ' active' : ''}`}
                              style={replacedManeuver === m.maneuver_name ? { borderColor: 'var(--gold)', color: 'var(--gold)' } : {}}
                              onClick={() => setReplacedManeuver(m.maneuver_name)}
                            >
                              {m.maneuver_name}
                            </button>
                          ))}
                        </div>
                      </>
                    )}
                  </div>
                )}

                <div className="lu-maneuver-grid">
                  {MANEUVERS.map(m => {
                    const alreadyKnown = (player.known_maneuvers || []).some(k => k.maneuver_name === m)
                      && m !== replacedManeuver
                    const isSel = selectedManeuvers.includes(m)
                    const isDisabled = alreadyKnown || (!isSel && selectedManeuvers.length >= maneuversToGain)
                    return (
                      <div
                        key={m}
                        className={`lu-maneuver${isSel ? ' sel' : ''}${isDisabled ? ' disabled' : ''}`}
                        onClick={() => !isDisabled && toggleManeuver(m)}
                      >
                        {m}
                        {alreadyKnown && <span style={{ fontSize: '.65rem', opacity: .5, marginLeft: '.3rem' }}>known</span>}
                      </div>
                    )
                  })}
                </div>
              </>
            )}

          </div>

          <div className="lu-footer">
            <div className="lu-step-dots">
              {steps.map((_, i) => (
                <div key={i} className={`lu-dot${i === stepIndex ? ' active' : ''}`} />
              ))}
            </div>

            <div style={{ display: 'flex', gap: '.75rem' }}>
              {stepIndex > 0 && (
                <button className="btn-ghost" onClick={() => setStepIndex(i => i - 1)}>
                  ← Back
                </button>
              )}
              {!isLast ? (
                <button
                  className="btn-gold"
                  disabled={!canAdvance()}
                  onClick={() => setStepIndex(i => i + 1)}
                >
                  Continue →
                </button>
              ) : (
                <button
                  className="btn-gold"
                  disabled={!canAdvance()}
                  onClick={handleConfirm}
                >
                  Confirm ⚔
                </button>
              )}
            </div>
          </div>

        </div>
      </div>
    </>
  )
}