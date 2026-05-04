import { useState, useEffect, useMemo } from 'react'
import { STYLES } from '../styles.js'
import {
    FIGHTER_SUBCLASSES, BARBARIAN_SUBCLASSES, BARD_SUBCLASSES,
    CLERIC_SUBCLASSES, DRUID_SUBCLASSES, MONK_SUBCLASSES,
    ALL_MANEUVERS, STAT_KEYS, STAT_LABELS,
    FIGHTER_ASI_LEVELS,
    getFighterFeatures, getBarbarianFeatures, getBardFeatures,
    getClericFeatures, getDruidFeatures, getMonkFeatures,
} from '../constants.js'
import { searchSpells } from '../api/client.js'

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
  color: var(--goldl); letter-spacing: .1em; margin-bottom: .3rem;
}
.lu-subtitle { font-size: .78rem; color: var(--dim); font-style: italic; }
.lu-body { padding: 1.4rem 1.8rem; flex: 1; }
.lu-footer {
  padding: 1rem 1.8rem; border-top: 1px solid var(--bord);
  display: flex; justify-content: space-between; align-items: center; gap: 1rem;
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
  display: grid; grid-template-columns: repeat(3, 1fr); gap: .5rem; margin: .5rem 0 1rem;
}
.lu-stat {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .6rem; text-align: center; cursor: pointer; transition: all .15s;
}
.lu-stat:hover, .lu-stat.sel { border-color: var(--gold); background: rgba(200,150,42,.08); }
.lu-stat-label {
  font-family: 'Cinzel', serif; font-size: .6rem;
  letter-spacing: .1em; color: var(--dim); margin-bottom: .2rem;
}
.lu-stat-val { font-size: 1.4rem; color: var(--goldl); line-height: 1; }
.lu-stat-mod { font-size: .72rem; color: var(--dim); margin-top: .1rem; }
.lu-subclass-grid { display: grid; grid-template-columns: 1fr 1fr; gap: .6rem; margin: .5rem 0; }
.lu-subclass {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .9rem 1rem;
  cursor: pointer; transition: all .15s; text-align: left;
}
.lu-subclass:hover, .lu-subclass.sel { border-color: var(--gold); background: rgba(200,150,42,.08); }
.lu-subclass-name { font-family: 'Cinzel', serif; font-size: .82rem; color: var(--goldl); margin-bottom: .3rem; }
.lu-subclass-desc { font-size: .74rem; color: var(--dim); line-height: 1.55; }
.lu-maneuver-grid {
  display: grid; grid-template-columns: 1fr 1fr; gap: .4rem;
  max-height: 280px; overflow-y: auto; margin: .5rem 0;
  scrollbar-width: thin; scrollbar-color: var(--gold) var(--surf);
}
.lu-maneuver {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .5rem .7rem;
  cursor: pointer; transition: all .15s; font-size: .78rem; color: var(--dim);
}
.lu-maneuver:hover, .lu-maneuver.sel { border-color: var(--gold); color: var(--goldl); background: rgba(200,150,42,.08); }
.lu-maneuver.disabled { opacity: .35; cursor: not-allowed; pointer-events: none; }
.lu-hp-gained { display: flex; align-items: baseline; gap: .5rem; margin-bottom: 1rem; }
.lu-hp-num { font-family: 'Cinzel', serif; font-size: 2rem; color: var(--goldl); line-height: 1; }
.lu-hp-label { font-size: .8rem; color: var(--dim); }
.lu-info-row {
  display: flex; justify-content: space-between;
  font-size: .78rem; padding: .3rem 0;
  border-bottom: 1px solid var(--bord); color: var(--dim);
}
.lu-info-row:last-child { border-bottom: none; }
.lu-info-val { color: var(--goldl); font-family: 'Cinzel', serif; }
.lu-step-dots { display: flex; gap: .4rem; align-items: center; }
.lu-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--bord); transition: background .2s; }
.lu-dot.active { background: var(--gold); }
.lu-asi-mode { display: flex; gap: .5rem; margin-bottom: 1rem; }
.lu-mode-btn {
  flex: 1; background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .5rem; cursor: pointer;
  font-family: 'Cinzel', serif; font-size: .68rem;
  letter-spacing: .08em; color: var(--dim); transition: all .15s; text-align: center;
}
.lu-mode-btn:hover, .lu-mode-btn.sel { border-color: var(--gold); color: var(--goldl); background: rgba(200,150,42,.07); }
.ek-tabs { display: flex; gap: .4rem; margin-bottom: .8rem; }
.ek-tab {
  flex: 1; background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .35rem; cursor: pointer;
  font-family: 'Cinzel', serif; font-size: .62rem;
  letter-spacing: .08em; color: var(--dim); transition: all .15s; text-align: center;
}
.ek-tab.active { border-color: #b5a9f5; color: #b5a9f5; background: rgba(181,169,245,.08); }
.ek-search-bar {
  display: flex; align-items: center; gap: .5rem;
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .4rem .6rem; margin-bottom: .6rem;
}
.ek-search-input { background: none; border: none; outline: none; color: var(--text); font-size: .8rem; flex: 1; font-family: inherit; }
.ek-spell-list {
  display: flex; flex-direction: column; gap: .3rem;
  max-height: 240px; overflow-y: auto;
  scrollbar-width: thin; scrollbar-color: #b5a9f5 var(--surf);
}
.ek-spell-row {
  display: flex; align-items: center; gap: .6rem;
  padding: .4rem .6rem; border-radius: 2px;
  border: 1px solid transparent; cursor: pointer; transition: all .12s;
}
.ek-spell-row:hover { background: rgba(181,169,245,.06); border-color: rgba(181,169,245,.2); }
.ek-spell-row.selected { background: rgba(181,169,245,.1); border-color: #b5a9f5; }
.ek-spell-row.learned { opacity: .4; cursor: not-allowed; }
.ek-spell-name { font-family: 'Cinzel', serif; font-size: .72rem; color: var(--text); flex: 1; }
.ek-spell-check { font-size: .72rem; color: #b5a9f5; }
.ek-school-badge {
  font-size: .58rem; padding: .1rem .35rem; border-radius: 2px;
  background: rgba(181,169,245,.12); color: #b5a9f5;
  border: 1px solid rgba(181,169,245,.2); white-space: nowrap;
}
.ek-school-badge.recommended { background: rgba(126,200,227,.12); color: #7ec8e3; border-color: rgba(126,200,227,.2); }
.ek-selected-pills { display: flex; flex-wrap: wrap; gap: .3rem; margin-top: .6rem; }
.ek-pill {
  display: flex; align-items: center; gap: .3rem;
  background: rgba(181,169,245,.1); border: 1px solid rgba(181,169,245,.25);
  border-radius: 10px; padding: .2rem .5rem; font-size: .68rem; color: #b5a9f5;
}
.ek-pill-remove { cursor: pointer; opacity: .6; font-size: .8rem; line-height: 1; }
.ek-pill-remove:hover { opacity: 1; }
.ek-hint {
  font-size: .72rem; color: var(--dim); line-height: 1.6; margin-bottom: .8rem;
  background: rgba(181,169,245,.04); border: 1px solid rgba(181,169,245,.1);
  border-radius: 2px; padding: .6rem .8rem;
}
`

const SCHOOL_COLORS = {
  abjuration: '#7ec8e3', evocation: '#f5a96a', divination: '#f5e87e',
  conjuration: '#b5a9f5', enchantment: '#f5a9c8', illusion: '#a9f5d0',
  necromancy: '#b0f5a9', transmutation: '#f5cfa9',
}

const mod = v => Math.floor((v - 10) / 2)
const fmt = v => { const m = mod(v); return (m >= 0 ? '+' : '') + m }

function maneuversToGainAtLevel(level) {
    if (level === 3) return 3
    if (level === 7 || level === 10 || level === 15) return 2
    return 0
}
function ekNewCantrips(level) { return level === 3 ? 2 : level === 10 ? 1 : 0 }
function ekNewPrepared(level) {
    return ({ 3:3, 4:1, 7:1, 10:2, 13:2, 16:2, 19:1 })[level] || 0
}
const RECOMMENDED_SCHOOLS = ['abjuration', 'evocation']

export default function LevelUpModal({ player, levelUpResult, campaignId, onComplete, onClose }) {
    const {
        new_level, hp_gained, new_max_hp, new_proficiency_bonus,
        asi_available, subclass_choice_required, new_features,
        // Fighter
        second_wind_uses, weapon_mastery_count, extra_attacks, action_surge_uses, indomitable_max,
        // Barbarian
        rage_uses, rage_damage,
        // Bard
        bardic_die, bardic_inspiration_uses, bard_prepared_spells, bard_cantrips, bard_slot_summary,
        // Cleric
        channel_divinity_uses, cleric_cantrips, cleric_prepared_spells, cleric_slot_summary,
        // Druid
        wild_shape_uses, wild_shape_cr, druid_cantrips, druid_prepared_spells, druid_slot_summary,
        // Monk
        focus_points, martial_arts_die, unarmored_movement,
    } = levelUpResult

    const isFighter   = player.class === 'Fighter'
    const isBarbarian = player.class === 'Barbarian'
    const isBard      = player.class === 'Bard'
    const isCleric    = player.class === 'Cleric'
    const isDruid     = player.class === 'Druid'
    const isMonk      = player.class === 'Monk'

    const isBattleMaster     = player.subclass === 'Battle Master'
    const maneuversToGain    = isBattleMaster ? maneuversToGainAtLevel(new_level) : 0
    const canReplaceManeuver = isBattleMaster && new_level >= 7

    const [subclass, setSubclass] = useState(null)
    const isEKChosen    = subclass === 'Eldritch Knight'
    const isExistingEK  = player.subclass === 'Eldritch Knight'
    const isEK          = isEKChosen || isExistingEK
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
    const [ekSpellTab, setEkSpellTab] = useState('cantrip')
    const [spellSearch, setSpellSearch] = useState('')
    const [spellResults, setSpellResults] = useState([])
    const [searching, setSearching] = useState(false)
    const [selectedCantrips, setSelectedCantrips] = useState([])
    const [selectedPrepared, setSelectedPrepared] = useState([])

    const cantripSlots  = ekNewCantrips(new_level)
    const preparedSlots = ekNewPrepared(new_level)

    useEffect(() => {
        if (spellSearch.trim().length < 2) { setSpellResults([]); return }
        const timer = setTimeout(async () => {
            setSearching(true)
            try { const res = await searchSpells(campaignId, spellSearch, true); setSpellResults(res.spells || []) }
            catch { /* ignore */ } finally { setSearching(false) }
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
            return (cantripSlots === 0 || selectedCantrips.length === cantripSlots)
                && (preparedSlots === 0 || selectedPrepared.length === preparedSlots)
        }
        return true
    }

    const handleConfirm = async () => {
        const choices = {}
        if (subclass) choices.subclass = subclass
        if (asi_available && asi1) {
            choices.asi_stat1 = asi1
            choices.asi_stat2 = (asiMode === '+1+1' && asi2) ? asi2 : asi1
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
        if (selectedManeuvers.includes(m)) setSelectedManeuvers(s => s.filter(x => x !== m))
        else if (selectedManeuvers.length < maneuversToGain) setSelectedManeuvers(s => [...s, m])
    }
    const isAlreadyKnown = (spellId) => (player.known_spells || []).some(s => s.spell_id === spellId)
    const toggleCantrip = (spell) => {
        if (selectedCantrips.some(s => s.id === spell.id)) setSelectedCantrips(s => s.filter(x => x.id !== spell.id))
        else if (selectedCantrips.length < cantripSlots) setSelectedCantrips(s => [...s, spell])
    }
    const togglePrepared = (spell) => {
        if (selectedPrepared.some(s => s.id === spell.id)) setSelectedPrepared(s => s.filter(x => x.id !== spell.id))
        else if (selectedPrepared.length < preparedSlots) setSelectedPrepared(s => [...s, spell])
    }

    const activeList   = ekSpellTab === 'cantrip' ? selectedCantrips : selectedPrepared
    const activeToggle = ekSpellTab === 'cantrip' ? toggleCantrip    : togglePrepared
    const activeMax    = ekSpellTab === 'cantrip' ? cantripSlots      : preparedSlots
    const activeResults = ekSpellTab === 'cantrip'
        ? spellResults.filter(s => s.level === 0)
        : spellResults.filter(s => s.level > 0 && s.level <= 2)

    const subclassOptions = isMonk
        ? MONK_SUBCLASSES
        : isCleric
            ? CLERIC_SUBCLASSES
            : isDruid
                ? DRUID_SUBCLASSES
                : isBarbarian
                    ? BARBARIAN_SUBCLASSES
                    : isBard
                        ? BARD_SUBCLASSES
                        : FIGHTER_SUBCLASSES

    const subclassLabel = isMonk
        ? 'Choose your Monastic Tradition'
        : isCleric
            ? 'Choose your Divine Domain'
            : isDruid
                ? 'Choose your Druid Circle'
                : isBarbarian
                    ? 'Choose your Primal Path'
                    : isBard
                        ? 'Choose your Bard College'
                        : 'Choose your Fighter subclass'

    const moonCR = player.subclass === 'Circle of the Moon'
        ? Math.floor(new_level / 3) : null

    return (
        <>
            <style dangerouslySetInnerHTML={{ __html: MODAL_STYLES }} />
            <div className="lu-overlay">
                <div className="lu-modal">

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

                                    {isFighter && (<>
                                        <div className="lu-info-row"><span>Second Wind Uses</span><span className="lu-info-val">{second_wind_uses}</span></div>
                                        <div className="lu-info-row"><span>Attacks per Action</span><span className="lu-info-val">{extra_attacks}</span></div>
                                        <div className="lu-info-row"><span>Weapon Masteries</span><span className="lu-info-val">{weapon_mastery_count}</span></div>
                                        {action_surge_uses > 0 && <div className="lu-info-row"><span>Action Surge Uses</span><span className="lu-info-val">{action_surge_uses}</span></div>}
                                        {indomitable_max > 0 && <div className="lu-info-row"><span>Indomitable Uses</span><span className="lu-info-val">{indomitable_max}</span></div>}
                                        {isEK && <div className="lu-info-row"><span>Spell Slots</span><span className="lu-info-val" style={{ color: '#b5a9f5' }}>{({3:'2×L1',4:'3×L1',7:'4L1 2L2',10:'4L1 3L2',13:'4L1 3L2 2L3',16:'4L1 3L2 3L3',19:'4L1 3L2 3L3 1L4'})[new_level]||'Updated'}</span></div>}
                                    </>)}

                                    {isBarbarian && (<>
                                        <div className="lu-info-row"><span>Rage Uses / Long Rest</span><span className="lu-info-val">{rage_uses}</span></div>
                                        <div className="lu-info-row"><span>Rage Damage Bonus</span><span className="lu-info-val">+{rage_damage}</span></div>
                                        <div className="lu-info-row"><span>Weapon Mastery</span><span className="lu-info-val">{weapon_mastery_count} weapons</span></div>
                                        {new_level >= 5 && <div className="lu-info-row"><span>Attacks per Action</span><span className="lu-info-val">{extra_attacks}</span></div>}
                                        {new_level >= 5 && <div className="lu-info-row"><span>Fast Movement</span><span className="lu-info-val">+10 ft Speed</span></div>}
                                    </>)}

                                    {isBard && (<>
                                        <div className="lu-info-row"><span>Bardic Inspiration Die</span><span className="lu-info-val">d{bardic_die}</span></div>
                                        <div className="lu-info-row"><span>Inspiration Uses</span><span className="lu-info-val">{bardic_inspiration_uses} / Long Rest</span></div>
                                        <div className="lu-info-row"><span>Cantrips Known</span><span className="lu-info-val">{bard_cantrips}</span></div>
                                        <div className="lu-info-row"><span>Prepared Spells</span><span className="lu-info-val">{bard_prepared_spells}</span></div>
                                        <div className="lu-info-row"><span>Spell Slots</span><span className="lu-info-val" style={{ fontSize: '.72rem' }}>{bard_slot_summary}</span></div>
                                        {new_level === 5 && <div className="lu-info-row"><span>Font of Inspiration</span><span className="lu-info-val">Short Rest restore</span></div>}
                                        {player.subclass === 'College of Valor' && new_level >= 6 && <div className="lu-info-row"><span>Attacks per Action</span><span className="lu-info-val">{extra_attacks}</span></div>}
                                    </>)}

                                    {isCleric && (<>
                                        {channel_divinity_uses > 0 && <div className="lu-info-row"><span>Channel Divinity Uses</span><span className="lu-info-val">{channel_divinity_uses}</span></div>}
                                        <div className="lu-info-row"><span>Cantrips Known</span><span className="lu-info-val">{cleric_cantrips}</span></div>
                                        <div className="lu-info-row"><span>Prepared Spells</span><span className="lu-info-val">{cleric_prepared_spells}</span></div>
                                        <div className="lu-info-row"><span>Spell Slots</span><span className="lu-info-val" style={{ fontSize: '.72rem' }}>{cleric_slot_summary}</span></div>
                                    </>)}

                                    {isDruid && (<>
                                        {wild_shape_uses > 0 && <div className="lu-info-row"><span>Wild Shape Uses</span><span className="lu-info-val">{wild_shape_uses}</span></div>}
                                        {wild_shape_uses > 0 && <div className="lu-info-row"><span>Wild Shape Max CR</span><span className="lu-info-val">{moonCR !== null ? `${moonCR} (Moon)` : wild_shape_cr}</span></div>}
                                        <div className="lu-info-row"><span>Cantrips Known</span><span className="lu-info-val">{druid_cantrips}</span></div>
                                        <div className="lu-info-row"><span>Prepared Spells</span><span className="lu-info-val">{druid_prepared_spells}</span></div>
                                        <div className="lu-info-row"><span>Spell Slots</span><span className="lu-info-val" style={{ fontSize: '.72rem' }}>{druid_slot_summary}</span></div>
                                        {new_level === 5 && <div className="lu-info-row"><span>Wild Resurgence</span><span className="lu-info-val">Exchange slots ↔ uses</span></div>}
                                    </>)}

                                    {isMonk && (<>
                                        <div className="lu-info-row"><span>Martial Arts Die</span><span className="lu-info-val">d{martial_arts_die}</span></div>
                                        {focus_points > 0 && <div className="lu-info-row"><span>Focus Points</span><span className="lu-info-val">{focus_points}</span></div>}
                                        {unarmored_movement > 0 && <div className="lu-info-row"><span>Unarmored Movement</span><span className="lu-info-val">+{unarmored_movement} ft Speed</span></div>}
                                        {new_level >= 5 && <div className="lu-info-row"><span>Attacks per Action</span><span className="lu-info-val">{extra_attacks}</span></div>}
                                        {new_level === 2 && <div className="lu-info-row"><span>Focus Points Restore</span><span className="lu-info-val">Short Rest</span></div>}
                                    </>)}
                                </div>

                                {new_features.filter(f => f).length > 0 && (<>
                                    <div className="lu-step-label">New features</div>
                                    <div className="lu-feature-list">
                                        {new_features.filter(f => f).map(f => (
                                            <div key={f} className="lu-feature">
                                                <div className="lu-feature-name">{f}</div>
                                            </div>
                                        ))}
                                    </div>
                                </>)}
                            </>
                        )}

                        {/* ── Subclass ── */}
                        {currentStep === 'subclass' && (<>
                            <div className="lu-step-label">{subclassLabel}</div>
                            <div className="lu-subclass-grid">
                                {subclassOptions.map(sc => (
                                    <div key={sc.name}
                                        className={`lu-subclass${subclass === sc.name ? ' sel' : ''}`}
                                        onClick={() => setSubclass(sc.name)}>
                                        <div className="lu-subclass-name">{sc.name}</div>
                                        <div className="lu-subclass-desc">{sc.desc}</div>
                                    </div>
                                ))}
                            </div>
                            {isFighter && subclass === 'Eldritch Knight' && (
                                <div style={{ fontSize: '.72rem', color: '#b5a9f5', marginTop: '.6rem', lineHeight: 1.6 }}>
                                    You'll choose 2 cantrips and 3 prepared spells (abjuration and evocation recommended) in the next step.
                                    You gain spell slots that refresh on a Long Rest, and can bond up to 2 weapons via War Bond.
                                </div>
                            )}
                        </>)}

                        {/* ── ASI ── */}
                        {currentStep === 'asi' && (<>
                            <div className="lu-step-label">Ability Score Improvement</div>
                            <div className="lu-asi-mode">
                                <button className={`lu-mode-btn${asiMode === '+2' ? ' sel' : ''}`}
                                    onClick={() => { setAsiMode('+2'); setAsi2(null) }}>+2 to one stat</button>
                                <button className={`lu-mode-btn${asiMode === '+1+1' ? ' sel' : ''}`}
                                    onClick={() => setAsiMode('+1+1')}>+1 to two stats</button>
                            </div>
                            {asiMode === '+2' ? (<>
                                <div style={{ fontSize: '.76rem', color: 'var(--dim)', marginBottom: '.6rem' }}>Choose one stat to increase by 2</div>
                                <div className="lu-stat-grid">
                                    {STAT_KEYS.map(k => (
                                        <div key={k} className={`lu-stat${asi1 === k ? ' sel' : ''}`} onClick={() => setAsi1(k)}>
                                            <div className="lu-stat-label">{STAT_LABELS[k]}</div>
                                            <div className="lu-stat-val">{player[k]}</div>
                                            <div className="lu-stat-mod">{fmt(player[k])} → {fmt(player[k] + 2)}</div>
                                        </div>
                                    ))}
                                </div>
                            </>) : (<>
                                <div style={{ fontSize: '.76rem', color: 'var(--dim)', marginBottom: '.6rem' }}>
                                    Choose two different stats to increase by 1 each
                                    {asi1 && !asi2 && <span style={{ color: 'var(--gold)' }}> — now choose second stat</span>}
                                </div>
                                <div className="lu-stat-grid">
                                    {STAT_KEYS.map(k => {
                                        const isFirst = asi1 === k, isSecond = asi2 === k, isSel = isFirst || isSecond
                                        return (
                                            <div key={k} className={`lu-stat${isSel ? ' sel' : ''}`}
                                                onClick={() => {
                                                    if (isFirst) { setAsi1(asi2); setAsi2(null) }
                                                    else if (isSecond) { setAsi2(null) }
                                                    else if (!asi1) setAsi1(k)
                                                    else if (!asi2 && k !== asi1) setAsi2(k)
                                                }}>
                                                <div className="lu-stat-label">{STAT_LABELS[k]}</div>
                                                <div className="lu-stat-val">{player[k]}</div>
                                                <div className="lu-stat-mod">{isSel ? `${fmt(player[k])} → ${fmt(player[k]+1)}` : fmt(player[k])}</div>
                                            </div>
                                        )
                                    })}
                                </div>
                            </>)}
                        </>)}

                        {/* ── Maneuvers ── */}
                        {currentStep === 'maneuvers' && (<>
                            <div className="lu-step-label">
                                Choose {maneuversToGain} maneuver{maneuversToGain > 1 ? 's' : ''}
                                {selectedManeuvers.length > 0 && ` (${selectedManeuvers.length}/${maneuversToGain} selected)`}
                            </div>
                            {canReplaceManeuver && (
                                <div style={{ marginBottom: '.8rem' }}>
                                    <label style={{ display: 'flex', alignItems: 'center', gap: '.5rem', cursor: 'pointer', fontSize: '.78rem', color: 'var(--dim)' }}>
                                        <input type="checkbox" checked={replaceMode}
                                            onChange={e => { setReplaceMode(e.target.checked); setReplacedManeuver(null) }} />
                                        Also replace one known maneuver
                                    </label>
                                    {replaceMode && (<>
                                        <div style={{ fontSize: '.72rem', color: 'var(--dim)', margin: '.4rem 0 .3rem' }}>Select a known maneuver to replace:</div>
                                        <div style={{ display: 'flex', flexWrap: 'wrap', gap: '.3rem', marginBottom: '.5rem' }}>
                                            {(player.known_maneuvers || []).map(m => (
                                                <button key={m.maneuver_name}
                                                    className={`btn-sm${replacedManeuver === m.maneuver_name ? ' active' : ''}`}
                                                    style={replacedManeuver === m.maneuver_name ? { borderColor: 'var(--gold)', color: 'var(--gold)' } : {}}
                                                    onClick={() => setReplacedManeuver(m.maneuver_name)}>
                                                    {m.maneuver_name}
                                                </button>
                                            ))}
                                        </div>
                                    </>)}
                                </div>
                            )}
                            <div className="lu-maneuver-grid">
                                {MANEUVERS.map(m => {
                                    const alreadyKnown = (player.known_maneuvers || []).some(k => k.maneuver_name === m) && m !== replacedManeuver
                                    const isSel = selectedManeuvers.includes(m)
                                    const isDisabled = alreadyKnown || (!isSel && selectedManeuvers.length >= maneuversToGain)
                                    return (
                                        <div key={m} className={`lu-maneuver${isSel ? ' sel' : ''}${isDisabled ? ' disabled' : ''}`}
                                            onClick={() => !isDisabled && toggleManeuver(m)}>
                                            {m}
                                            {alreadyKnown && <span style={{ fontSize: '.65rem', opacity: .5, marginLeft: '.3rem' }}>known</span>}
                                        </div>
                                    )
                                })}
                            </div>
                        </>)}

                        {/* ── EK Spells ── */}
                        {currentStep === 'ek_spells' && (<>
                            <div className="lu-step-label">
                                {new_level === 3 ? 'Eldritch Knight — Choose Starting Spells' : 'Learn New Spells'}
                            </div>
                            <div className="ek-hint">
                                As an Eldritch Knight, you specialize in <span style={{ color: '#7ec8e3' }}>Abjuration</span> and{' '}
                                <span style={{ color: '#f5a96a' }}>Evocation</span> spells.
                                You may choose one spell from any school at levels 3, 8, 14, and 20.
                            </div>
                            <div className="ek-tabs">
                                {cantripSlots > 0 && <button className={`ek-tab${ekSpellTab === 'cantrip' ? ' active' : ''}`} onClick={() => setEkSpellTab('cantrip')}>Cantrips ({selectedCantrips.length}/{cantripSlots})</button>}
                                {preparedSlots > 0 && <button className={`ek-tab${ekSpellTab === 'prepared' ? ' active' : ''}`} onClick={() => setEkSpellTab('prepared')}>Spells ({selectedPrepared.length}/{preparedSlots})</button>}
                            </div>
                            <div className="ek-search-bar">
                                <span style={{ opacity: .5 }}>🔍</span>
                                <input className="ek-search-input" placeholder={ekSpellTab === 'cantrip' ? 'Search cantrips...' : 'Search spells...'}
                                    value={spellSearch} onChange={e => setSpellSearch(e.target.value)} autoFocus />
                                {searching && <span style={{ fontSize: '.7rem', color: 'var(--dim)' }}>...</span>}
                            </div>
                            <div className="ek-spell-list">
                                {spellSearch.trim().length < 2 && activeList.length === 0 && (
                                    <div style={{ padding: '.5rem', fontSize: '.72rem', color: 'var(--dim)', textAlign: 'center' }}>Type at least 2 characters to search</div>
                                )}
                                {activeResults.map(spell => {
                                    const known = isAlreadyKnown(spell.id)
                                    const isSel = activeList.some(s => s.id === spell.id)
                                    const atMax = activeList.length >= activeMax && !isSel
                                    return (
                                        <div key={spell.id}
                                            className={`ek-spell-row${isSel ? ' selected' : ''}${known || atMax ? ' learned' : ''}`}
                                            onClick={() => !known && !atMax && activeToggle({ id: spell.id, name: spell.name, school: spell.school })}>
                                            <span style={{ fontSize: '.9rem', color: SCHOOL_COLORS[spell.school] || '#b5a9f5' }}>{spell.level === 0 ? '⊕' : spell.level}</span>
                                            <span className="ek-spell-name">{spell.name}</span>
                                            <span className={`ek-school-badge${['abjuration','evocation'].includes(spell.school) ? ' recommended' : ''}`}>{spell.school}</span>
                                            {isSel && <span className="ek-spell-check">✓</span>}
                                            {known && <span style={{ fontSize: '.6rem', color: 'var(--dim)' }}>known</span>}
                                        </div>
                                    )
                                })}
                                {spellSearch.trim().length >= 2 && !searching && activeResults.length === 0 && (
                                    <div style={{ padding: '.5rem', fontSize: '.72rem', color: 'var(--dim)', textAlign: 'center' }}>No {ekSpellTab === 'cantrip' ? 'cantrips' : 'spells'} found</div>
                                )}
                            </div>
                            {activeList.length > 0 && (
                                <div className="ek-selected-pills">
                                    {activeList.map(s => (
                                        <div key={s.id} className="ek-pill">
                                            <span>{s.name}</span>
                                            <span className="ek-pill-remove" onClick={() => activeToggle(s)}>×</span>
                                        </div>
                                    ))}
                                </div>
                            )}
                            <div style={{ marginTop: '.8rem', fontSize: '.72rem', color: 'var(--dim)' }}>
                                {cantripSlots > 0 && <span style={{ color: selectedCantrips.length === cantripSlots ? '#7ef5a9' : 'var(--dim)' }}>Cantrips: {selectedCantrips.length}/{cantripSlots}</span>}
                                {cantripSlots > 0 && preparedSlots > 0 && ' · '}
                                {preparedSlots > 0 && <span style={{ color: selectedPrepared.length === preparedSlots ? '#7ef5a9' : 'var(--dim)' }}>Spells: {selectedPrepared.length}/{preparedSlots}</span>}
                            </div>
                        </>)}

                    </div>

                    <div className="lu-footer">
                        <div className="lu-step-dots">
                            {steps.map((_, i) => <div key={i} className={`lu-dot${i === stepIndex ? ' active' : ''}`} />)}
                        </div>
                        <div style={{ display: 'flex', gap: '.75rem' }}>
                            {stepIndex > 0 && <button className="btn-ghost" onClick={() => setStepIndex(i => i - 1)}>← Back</button>}
                            {!isLast
                                ? <button className="btn-gold" disabled={!canAdvance()} onClick={() => setStepIndex(i => i + 1)}>Continue →</button>
                                : <button className="btn-gold" disabled={!canAdvance()} onClick={handleConfirm}>Confirm ⚔</button>
                            }
                        </div>
                    </div>

                </div>
            </div>
        </>
    )
}