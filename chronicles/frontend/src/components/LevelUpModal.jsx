import { useState, useEffect, useMemo } from 'react'
import { STYLES } from '../styles.js'
import {
    FIGHTER_SUBCLASSES, BARBARIAN_SUBCLASSES, ALL_MANEUVERS, STAT_KEYS, STAT_LABELS,
    FIGHTER_ASI_LEVELS, getFighterFeatures, getBarbarianFeatures,
} from '../constants.js'
import { searchSpells, learnSpell, seedEkSlots } from '../api/client.js'

const MANEUVERS = ALL_MANEUVERS

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
  border-radius: 2px; padding: .6rem .4rem;
  text-align: center; cursor: pointer;
  transition: border-color .15s, background .15s;
}
.lu-stat.sel { border-color: var(--gold); background: rgba(212,175,55,.08); }
.lu-stat-key { font-family: 'Cinzel', serif; font-size: .65rem; color: var(--dim); letter-spacing: .1em; }
.lu-stat-val { font-size: 1.1rem; color: var(--goldl); font-weight: 600; margin: .2rem 0; }
.lu-stat-mod { font-size: .7rem; color: var(--dim); }
.lu-hp-gained {
  display: flex; align-items: baseline; gap: .6rem;
  margin-bottom: 1rem;
}
.lu-hp-num { font-family: 'Cinzel', serif; font-size: 2rem; color: #7ef5a9; }
.lu-hp-label { font-size: .8rem; color: var(--dim); }
.lu-info-row {
  display: flex; justify-content: space-between;
  padding: .3rem 0; border-bottom: 1px solid var(--bord);
  font-size: .82rem;
}
.lu-info-val { color: var(--goldl); font-weight: 600; }
.lu-subclass-grid { display: flex; flex-direction: column; gap: .6rem; margin-bottom: .8rem; }
.lu-subclass {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .8rem 1rem; cursor: pointer;
  transition: border-color .15s;
}
.lu-subclass.sel { border-color: var(--gold); }
.lu-subclass-name {
  font-family: 'Cinzel', serif; font-size: .78rem;
  color: var(--goldl); letter-spacing: .06em; margin-bottom: .3rem;
}
.lu-subclass-desc { font-size: .76rem; color: var(--dim); line-height: 1.5; }
.lu-asi-mode { display: flex; gap: .5rem; margin-bottom: 1rem; }
.lu-mode-btn {
  flex: 1; padding: .5rem; font-size: .78rem;
  background: var(--elev); border: 1px solid var(--bord);
  color: var(--fg); border-radius: 2px; cursor: pointer;
}
.lu-mode-btn.sel { border-color: var(--gold); color: var(--goldl); }
.lu-maneuver-grid {
  display: flex; flex-wrap: wrap; gap: .4rem; margin-top: .5rem;
}
.lu-maneuver {
  padding: .35rem .7rem; font-size: .75rem;
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; cursor: pointer;
}
.lu-maneuver.sel { border-color: var(--gold); color: var(--goldl); }
.lu-maneuver.disabled { opacity: .35; cursor: default; }
.lu-step-dots { display: flex; gap: .4rem; align-items: center; }
.lu-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--bord); }
.lu-dot.active { background: var(--gold); }
.ek-hint {
  font-size: .75rem; color: var(--dim); line-height: 1.6;
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .6rem .8rem; margin-bottom: .8rem;
}
.ek-tabs { display: flex; gap: .5rem; margin-bottom: .8rem; }
.ek-tab {
  padding: .4rem .9rem; font-size: .76rem;
  background: var(--elev); border: 1px solid var(--bord);
  color: var(--dim); border-radius: 2px; cursor: pointer;
}
.ek-tab.active { border-color: var(--gold); color: var(--goldl); }
.ek-search { width: 100%; margin-bottom: .6rem; }
.ek-spell-list { display: flex; flex-direction: column; gap: .3rem; max-height: 220px; overflow-y: auto; }
.ek-spell {
  display: flex; justify-content: space-between; align-items: center;
  padding: .4rem .7rem; font-size: .76rem;
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; cursor: pointer;
}
.ek-spell.sel { border-color: var(--gold); }
.ek-spell.known { opacity: .4; cursor: default; }
.ek-spell-school { font-size: .65rem; color: var(--dim); }
.ek-spell-school.recommended { color: #7ec8e3; }
.ek-selected-list { display: flex; flex-wrap: wrap; gap: .4rem; margin-top: .6rem; }
.ek-selected-tag {
  padding: .25rem .6rem; font-size: .72rem;
  background: rgba(212,175,55,.12); border: 1px solid var(--gold);
  border-radius: 2px; color: var(--goldl);
}
`

function fmtMod(score) {
    const m = Math.floor((score - 10) / 2)
    return (m >= 0 ? '+' : '') + m
}

function maneuversToGainAtLevel(level) {
    if (level === 3) return 3
    if (level === 7 || level === 10 || level === 15) return 2
    return 0
}

function ekNewCantrips(level) {
    if (level === 3) return 2
    if (level === 10) return 1
    return 0
}

function ekNewPrepared(level) {
    const table = { 3: 3, 4: 1, 7: 1, 10: 2, 13: 2, 16: 2, 19: 1 }
    return table[level] || 0
}

const RECOMMENDED_SCHOOLS = ['abjuration', 'evocation']

export default function LevelUpModal({ player, levelUpResult, campaignId, onComplete, onClose }) {
    const {
        new_level, hp_gained, new_max_hp, new_proficiency_bonus,
        asi_available, subclass_choice_required, new_features,
        second_wind_uses, weapon_mastery_count, extra_attacks,
        action_surge_uses, indomitable_max,
        rage_uses, rage_damage,
    } = levelUpResult

    const isFighter   = player.class === 'Fighter'
    const isBarbarian = player.class === 'Barbarian'
    const isBattleMaster = player.subclass === 'Battle Master'
    const maneuversToGain   = isBattleMaster ? maneuversToGainAtLevel(new_level) : 0
    const canReplaceManeuver = isBattleMaster && new_level >= 7

    // Subclass state (used for both Fighter and Barbarian)
    const [subclass, setSubclass] = useState(null)
    const isEKChosen   = subclass === 'Eldritch Knight'
    const isExistingEK = player.subclass === 'Eldritch Knight'
    const isEK         = isEKChosen || isExistingEK
    const needsEKSpells = isEK && (ekNewCantrips(new_level) > 0 || ekNewPrepared(new_level) > 0)

    const steps = useMemo(() => {
        const s = ['summary']
        if (subclass_choice_required) s.push('subclass')
        if (asi_available) s.push('asi')
        if (maneuversToGain > 0) s.push('maneuvers')
        if (needsEKSpells) s.push('ek_spells')
        return s
    }, [subclass_choice_required, asi_available, maneuversToGain, needsEKSpells])

    const [stepIndex, setStepIndex] = useState(0)
    const [asiMode, setAsiMode] = useState('+2')
    const [asi1, setAsi1] = useState(null)
    const [asi2, setAsi2] = useState(null)
    const [selectedManeuvers, setSelectedManeuvers] = useState([])
    const [replacedManeuver, setReplacedManeuver] = useState(null)
    const [replaceMode, setReplaceMode] = useState(false)

    // EK spell state
    const [ekSpellTab, setEkSpellTab] = useState('cantrip')
    const [spellSearch, setSpellSearch] = useState('')
    const [spellResults, setSpellResults] = useState([])
    const [searching, setSearching] = useState(false)
    const [selectedCantrips, setSelectedCantrips] = useState([])
    const [selectedPrepared, setSelectedPrepared] = useState([])

    const cantripSlots = ekNewCantrips(new_level)
    const preparedSlots = ekNewPrepared(new_level)

    useEffect(() => {
        if (spellSearch.trim().length < 2) { setSpellResults([]); return }
        const timer = setTimeout(async () => {
            setSearching(true)
            try {
                const res = await searchSpells(campaignId, spellSearch, true)
                setSpellResults(res.spells || [])
            } catch (e) { /* ignore */ }
            finally { setSearching(false) }
        }, 300)
        return () => clearTimeout(timer)
    }, [spellSearch, campaignId])

    useEffect(() => { setSpellSearch(''); setSpellResults([]) }, [ekSpellTab])

    const currentStep = steps[stepIndex]
    const isLast = stepIndex === steps.length - 1

    const canAdvance = () => {
        if (currentStep === 'summary') return true
        if (currentStep === 'subclass') return subclass !== null
        if (currentStep === 'asi') {
            if (asiMode === '+2') return asi1 !== null
            return asi1 !== null && asi2 !== null && asi1 !== asi2
        }
        if (currentStep === 'maneuvers') return selectedManeuvers.length === maneuversToGain
        if (currentStep === 'ek_spells') {
            const cantripOk = cantripSlots === 0 || selectedCantrips.length === cantripSlots
            const preparedOk = preparedSlots === 0 || selectedPrepared.length === preparedSlots
            return cantripOk && preparedOk
        }
        return true
    }

    const handleConfirm = () => {
        const choices = {}
        if (subclass) choices.subclass = subclass
        if (asi_available && asi1) {
            choices.asi_stat1 = asi1
            if (asiMode === '+1+1' && asi2) choices.asi_stat2 = asi2
            else choices.asi_stat2 = asi1
        }
        if (selectedManeuvers.length > 0) choices.new_maneuvers = selectedManeuvers
        if (replacedManeuver) choices.replaced_maneuver = replacedManeuver
        if (isEK && campaignId) {
            choices.ek_cantrips = selectedCantrips.map(s => s.id)
            choices.ek_prepared = selectedPrepared.map(s => s.id)
        }
        onComplete(choices)
    }

    const toggleManeuver = (m) => {
        if (selectedManeuvers.includes(m)) {
            setSelectedManeuvers(s => s.filter(x => x !== m))
        } else if (selectedManeuvers.length < maneuversToGain) {
            setSelectedManeuvers(s => [...s, m])
        }
    }

    const isAlreadyKnown = (spellId) =>
        (player.known_spells || []).some(s => s.spell_id === spellId)

    const toggleCantrip = (spell) => {
        if (selectedCantrips.some(s => s.id === spell.id)) {
            setSelectedCantrips(s => s.filter(x => x.id !== spell.id))
        } else if (selectedCantrips.length < cantripSlots) {
            setSelectedCantrips(s => [...s, spell])
        }
    }

    const togglePrepared = (spell) => {
        if (selectedPrepared.some(s => s.id === spell.id)) {
            setSelectedPrepared(s => s.filter(x => x.id !== spell.id))
        } else if (selectedPrepared.length < preparedSlots) {
            setSelectedPrepared(s => [...s, spell])
        }
    }

    const activeList    = ekSpellTab === 'cantrip' ? selectedCantrips : selectedPrepared
    const activeToggle  = ekSpellTab === 'cantrip' ? toggleCantrip    : togglePrepared
    const activeMax     = ekSpellTab === 'cantrip' ? cantripSlots      : preparedSlots
    const activeResults = ekSpellTab === 'cantrip'
        ? spellResults.filter(s => s.level === 0)
        : spellResults.filter(s => s.level > 0 && s.level <= 2)

    const isRecommended = (school) => RECOMMENDED_SCHOOLS.includes(school)

    // EK slot labels for summary display
    const ek = {
        3: '2× Level 1', 4: '2× Level 1', 5: '3× Level 1', 6: '3× Level 1',
        7: '4× L1, 2× L2', 8: '4× L1, 2× L2', 9: '4× L1, 2× L2',
        10: '4× L1, 3× L2', 11: '4× L1, 3× L2', 12: '4× L1, 3× L2',
        13: '4/3/2', 14: '4/3/2', 15: '4/3/2',
        16: '4/3/3', 17: '4/3/3', 18: '4/3/3',
        19: '4/3/3/1', 20: '4/3/3/1',
    }

    return (
        <>
            <style dangerouslySetInnerHTML={{ __html: MODAL_STYLES }} />
            <div className="lu-overlay">
                <div className="lu-modal">

                    {/* ── Header ── */}
                    <div className="lu-header">
                        <div className="lu-title">Level {new_level}!</div>
                        <div className="lu-subtitle">
                            {player.name} — {player.race} {player.class}
                            {(player.subclass || subclass) ? ` · ${player.subclass || subclass}` : ''}
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

                                    {/* Fighter stats */}
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
                                                <span>Weapon Mastery</span>
                                                <span className="lu-info-val">{weapon_mastery_count} weapons</span>
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
                                            {isEK && (
                                                <div className="lu-info-row">
                                                    <span>EK Spell Slots</span>
                                                    <span className="lu-info-val" style={{ fontSize: '.74rem' }}>
                                                        {ek[new_level] || 'Updated'}
                                                    </span>
                                                </div>
                                            )}
                                        </>
                                    )}

                                    {/* Barbarian stats */}
                                    {isBarbarian && (
                                        <>
                                            <div className="lu-info-row">
                                                <span>Rage Uses / Long Rest</span>
                                                <span className="lu-info-val">{rage_uses}</span>
                                            </div>
                                            <div className="lu-info-row">
                                                <span>Rage Damage Bonus</span>
                                                <span className="lu-info-val">+{rage_damage}</span>
                                            </div>
                                            <div className="lu-info-row">
                                                <span>Weapon Mastery</span>
                                                <span className="lu-info-val">{weapon_mastery_count} weapons</span>
                                            </div>
                                            {new_level >= 5 && (
                                                <div className="lu-info-row">
                                                    <span>Attacks per Action</span>
                                                    <span className="lu-info-val">{extra_attacks}</span>
                                                </div>
                                            )}
                                            {new_level >= 5 && (
                                                <div className="lu-info-row">
                                                    <span>Fast Movement</span>
                                                    <span className="lu-info-val">+10 ft Speed</span>
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
                                <div className="lu-step-label">
                                    Choose your {isBarbarian ? 'Primal Path' : 'Fighter subclass'}
                                </div>
                                <div className="lu-subclass-grid">
                                    {(isBarbarian ? BARBARIAN_SUBCLASSES : FIGHTER_SUBCLASSES).map(sc => (
                                        <div
                                            key={sc.name}
                                            className={`lu-subclass${subclass === sc.name ? ' sel' : ''}`}
                                            onClick={() => setSubclass(sc.name)}
                                        >
                                            <div className="lu-subclass-name">
                                                {sc.name}
                                            </div>
                                            <div className="lu-subclass-desc">{sc.desc}</div>
                                        </div>
                                    ))}
                                </div>
                                {!isBarbarian && subclass === 'Eldritch Knight' && (
                                    <div style={{ fontSize: '.72rem', color: '#b5a9f5', marginTop: '.6rem', lineHeight: 1.6 }}>
                                        You'll choose 2 cantrips and 3 prepared spells (abjuration and evocation recommended) in the next step.
                                        You gain spell slots that refresh on a Long Rest, and can bond up to 2 weapons via War Bond.
                                    </div>
                                )}
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
                                                    <div className="lu-stat-key">{STAT_LABELS[k]}</div>
                                                    <div className="lu-stat-val">{player[k]}</div>
                                                    <div className="lu-stat-mod">{fmtMod(player[k])} → {fmtMod(Math.min(20, player[k] + 2))}</div>
                                                </div>
                                            ))}
                                        </div>
                                    </>
                                ) : (
                                    <>
                                        <div style={{ fontSize: '.76rem', color: 'var(--dim)', marginBottom: '.6rem' }}>
                                            Choose two different stats to increase by 1 each
                                        </div>
                                        <div className="lu-stat-grid">
                                            {STAT_KEYS.map(k => {
                                                const isSel = asi1 === k || asi2 === k
                                                return (
                                                    <div
                                                        key={k}
                                                        className={`lu-stat${isSel ? ' sel' : ''}`}
                                                        onClick={() => {
                                                            if (asi1 === k) { setAsi1(asi2); setAsi2(null) }
                                                            else if (asi2 === k) { setAsi2(null) }
                                                            else if (!asi1) setAsi1(k)
                                                            else if (!asi2) setAsi2(k)
                                                        }}
                                                    >
                                                        <div className="lu-stat-key">{STAT_LABELS[k]}</div>
                                                        <div className="lu-stat-val">{player[k]}</div>
                                                        <div className="lu-stat-mod">{fmtMod(player[k])} → {fmtMod(Math.min(20, player[k] + (isSel ? 1 : 0)))}</div>
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
                                    Choose {maneuversToGain} maneuver{maneuversToGain > 1 ? 's' : ''} ({selectedManeuvers.length}/{maneuversToGain} selected)
                                </div>

                                {canReplaceManeuver && (
                                    <div style={{ marginBottom: '.8rem' }}>
                                        <button
                                            className={`lu-mode-btn${replaceMode ? ' sel' : ''}`}
                                            style={{ width: '100%' }}
                                            onClick={() => { setReplaceMode(r => !r); setReplacedManeuver(null) }}
                                        >
                                            {replaceMode ? '✓ Replacing a maneuver' : 'Replace a known maneuver?'}
                                        </button>
                                        {replaceMode && (
                                            <>
                                                <div style={{ fontSize: '.72rem', color: 'var(--dim)', margin: '.5rem 0 .3rem' }}>
                                                    Select one known maneuver to replace:
                                                </div>
                                                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '.4rem' }}>
                                                    {(player.known_maneuvers || []).map(m => (
                                                        <button
                                                            key={m.maneuver_name}
                                                            className={`lu-maneuver${replacedManeuver === m.maneuver_name ? ' sel' : ''}`}
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

                        {/* ── EK Spells ── */}
                        {currentStep === 'ek_spells' && (
                            <>
                                <div className="lu-step-label">
                                    {new_level === 3 ? 'Eldritch Knight — Choose Starting Spells' : 'Learn New Spells'}
                                </div>
                                <div className="ek-hint">
                                    As an Eldritch Knight, you specialize in <span style={{ color: '#7ec8e3' }}>Abjuration</span> and{' '}
                                    <span style={{ color: '#f5a96a' }}>Evocation</span> spells.
                                    You may choose one spell from any school at levels 3, 8, 14, and 20.
                                </div>

                                {cantripSlots > 0 && preparedSlots > 0 && (
                                    <div className="ek-tabs">
                                        <button
                                            className={`ek-tab${ekSpellTab === 'cantrip' ? ' active' : ''}`}
                                            onClick={() => setEkSpellTab('cantrip')}
                                        >
                                            Cantrips ({selectedCantrips.length}/{cantripSlots})
                                        </button>
                                        <button
                                            className={`ek-tab${ekSpellTab === 'prepared' ? ' active' : ''}`}
                                            onClick={() => setEkSpellTab('prepared')}
                                        >
                                            Spells ({selectedPrepared.length}/{preparedSlots})
                                        </button>
                                    </div>
                                )}

                                <input
                                    className="inp ek-search"
                                    placeholder={ekSpellTab === 'cantrip' ? 'Search cantrips…' : 'Search spells…'}
                                    value={spellSearch}
                                    onChange={e => setSpellSearch(e.target.value)}
                                />

                                {searching && <div style={{ fontSize: '.75rem', color: 'var(--dim)' }}>Searching…</div>}

                                {activeResults.length > 0 && (
                                    <div className="ek-spell-list">
                                        {activeResults.map(spell => {
                                            const known = isAlreadyKnown(spell.id)
                                            const sel = activeList.some(s => s.id === spell.id)
                                            const full = !sel && activeList.length >= activeMax
                                            return (
                                                <div
                                                    key={spell.id}
                                                    className={`ek-spell${sel ? ' sel' : ''}${known ? ' known' : ''}`}
                                                    onClick={() => !known && !full && activeToggle(spell)}
                                                >
                                                    <span>{spell.name}</span>
                                                    <span className={`ek-spell-school${isRecommended(spell.school) ? ' recommended' : ''}`}>
                                                        {spell.school}
                                                    </span>
                                                </div>
                                            )
                                        })}
                                    </div>
                                )}

                                {(selectedCantrips.length > 0 || selectedPrepared.length > 0) && (
                                    <>
                                        <div style={{ fontSize: '.7rem', color: 'var(--dim)', marginTop: '.8rem', marginBottom: '.3rem' }}>
                                            Selected:
                                        </div>
                                        <div className="ek-selected-list">
                                            {[...selectedCantrips, ...selectedPrepared].map(s => (
                                                <div key={s.id} className="ek-selected-tag">{s.name}</div>
                                            ))}
                                        </div>
                                    </>
                                )}

                                <div style={{ fontSize: '.7rem', color: 'var(--dim)', marginTop: '.6rem' }}>
                                    {cantripSlots > 0 && (
                                        <span style={{ color: selectedCantrips.length === cantripSlots ? '#7ef5a9' : 'var(--dim)' }}>
                                            Cantrips: {selectedCantrips.length}/{cantripSlots}
                                        </span>
                                    )}
                                    {cantripSlots > 0 && preparedSlots > 0 && ' · '}
                                    {preparedSlots > 0 && (
                                        <span style={{ color: selectedPrepared.length === preparedSlots ? '#7ef5a9' : 'var(--dim)' }}>
                                            Spells: {selectedPrepared.length}/{preparedSlots}
                                        </span>
                                    )}
                                </div>
                            </>
                        )}

                    </div>

                    {/* ── Footer ── */}
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