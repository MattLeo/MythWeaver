import { useState, useEffect, useRef } from 'react'
import { STYLES } from '../styles.js'
import * as api from '../api/client.js'

const COMBAT_STYLES = `
${STYLES}

.combat-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0,0,0,.82);
  display: flex; align-items: center; justify-content: center;
  backdrop-filter: blur(3px);
}

.combat-modal {
  width: 95vw; max-width: 1100px;
  height: 92vh; max-height: 820px;
  background: #0d0e18;
  border: 1px solid #2a2d44;
  border-radius: 4px;
  display: flex; flex-direction: column;
  overflow: hidden;
  box-shadow: 0 0 80px rgba(0,0,0,.8), 0 0 30px rgba(200,150,42,.05);
}

.combat-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: .6rem 1rem;
  background: #0b0c15;
  border-bottom: 1px solid #1e2035;
  flex-shrink: 0;
}

.combat-title {
  font-family: 'Cinzel', serif; font-size: .75rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--gold);
}

.combat-round {
  font-family: 'Cinzel', serif; font-size: .68rem;
  color: var(--dim); letter-spacing: .12em;
}

.turn-order-bar {
  display: flex; gap: .5rem; align-items: center;
  padding: .55rem 1rem;
  background: #0c0d1a;
  border-bottom: 1px solid #1a1d2e;
  overflow-x: auto; flex-shrink: 0;
  scrollbar-width: thin; scrollbar-color: var(--gold) #0c0d1a;
}

.turn-chip {
  display: flex; align-items: center; gap: .3rem;
  padding: .25rem .6rem;
  border-radius: 2px; border: 1px solid #2a2d44;
  font-family: 'Cinzel', serif; font-size: .6rem;
  letter-spacing: .08em; white-space: nowrap;
  flex-shrink: 0; transition: all .2s;
  color: var(--dim); background: #13151f;
}

.turn-chip.active {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.08);
  box-shadow: 0 0 8px rgba(200,150,42,.2);
}

.turn-chip.enemy { border-color: #4a1a1a; color: #c06060; }
.turn-chip.enemy.active { border-color: var(--red); color: #ff8080; box-shadow: 0 0 8px rgba(200,60,60,.3); }
.turn-chip.ally { border-color: #1a3a2a; color: #60a080; }
.turn-chip.companion { border-color: #1a2a3a; color: #6080c0; }
.turn-chip.dead { opacity: .3; text-decoration: line-through; }
.turn-chip-icon { font-size: .7rem; }

.battlefield {
  display: flex; flex-direction: column;
  flex: 1; min-height: 0; padding: .75rem 1rem;
  justify-content: space-between;
}

/* ── Enemy Row (top) ── */
.enemy-row {
  display: flex; gap: 1rem; justify-content: center;
  align-items: flex-start;
  flex: 1; min-height: 0;
  padding-top: .5rem;
}

.enemy-card {
  display: flex; flex-direction: column; align-items: center;
  gap: .3rem; cursor: pointer; transition: all .2s;
  position: relative; padding: .5rem;
  border-radius: 3px; border: 1px solid transparent;
  min-width: 80px;
}

.enemy-card:hover:not(.dead):not(.disabled) {
  border-color: rgba(200,150,42,.3);
  background: rgba(200,150,42,.04);
}

.enemy-card.targeted {
  border-color: var(--gold);
  background: rgba(200,150,42,.08);
  box-shadow: 0 0 16px rgba(200,150,42,.25);
}

.enemy-card.selecting:not(.dead) {
  animation: enemy-pulse 1.2s ease-in-out infinite;
}

@keyframes enemy-pulse {
  0%, 100% { border-color: rgba(200,150,42,.2); }
  50% { border-color: rgba(200,150,42,.6); box-shadow: 0 0 12px rgba(200,150,42,.2); }
}

.enemy-card.dead { opacity: .35; cursor: default; filter: grayscale(1); }

.enemy-card.shake { animation: shake .4s ease; }
@keyframes shake {
  0%, 100% { transform: translateX(0); }
  20% { transform: translateX(-6px); }
  40% { transform: translateX(6px); }
  60% { transform: translateX(-4px); }
  80% { transform: translateX(4px); }
}

/* Enemy attacks downward toward player row */
.enemy-card.attack-out {
  animation: enemy-attack-down .35s ease;
}
@keyframes enemy-attack-down {
  0%   { transform: translateY(0); }
  50%  { transform: translateY(18px); }
  100% { transform: translateY(0); }
}

.enemy-icon { width: 64px; height: 64px; position: relative; }
.enemy-icon svg { width: 100%; height: 100%; }

.bloodied-indicator {
  position: absolute; top: -4px; right: -4px;
  font-size: .75rem; filter: drop-shadow(0 0 4px rgba(255,0,0,.5));
}

.enemy-name {
  font-family: 'Cinzel', serif; font-size: .62rem;
  color: var(--dim); text-align: center;
  letter-spacing: .06em; max-width: 80px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}

.enemy-dead-mark { font-size: .7rem; color: var(--red); font-family: 'Cinzel', serif; }

.target-arrow {
  position: absolute; top: -22px; left: 50%;
  transform: translateX(-50%);
  font-size: 1rem; color: var(--gold);
  animation: bounce-arrow .6s ease-in-out infinite;
}
@keyframes bounce-arrow {
  0%, 100% { transform: translateX(-50%) translateY(0); }
  50% { transform: translateX(-50%) translateY(-4px); }
}

/* ── Player/Ally Row (bottom) ── */
.player-row {
  display: flex; gap: .75rem; justify-content: center;
  align-items: flex-start; flex-wrap: wrap;
}

.combatant-card {
  background: #13151f; border: 1px solid #1e2035;
  border-radius: 3px; padding: .5rem .65rem;
  min-width: 100px; max-width: 130px;
  transition: all .2s;
}

.combatant-card.active-turn {
  border-color: var(--gold);
  background: rgba(200,150,42,.05);
  box-shadow: 0 0 12px rgba(200,150,42,.15);
}

.combatant-card.downed {
  border-color: var(--red);
  background: rgba(180,50,50,.05);
  opacity: .7;
}

.combatant-card.ally-card  { border-color: #1a3a2a; }
.combatant-card.comp-card  { border-color: #1a2a3a; }

/* Player attacks upward toward enemy row */
.combatant-card.player-attacking {
  animation: player-attack-up .35s ease;
}
@keyframes player-attack-up {
  0%   { transform: translateY(0); }
  50%  { transform: translateY(-18px); }
  100% { transform: translateY(0); }
}

.combatant-name {
  font-family: 'Cinzel', serif; font-size: .65rem;
  color: var(--goldl); letter-spacing: .06em;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  margin-bottom: .3rem;
}

.combatant-hp-bar {
  background: #0b0c15; border-radius: 1px; height: 4px; margin-bottom: .25rem;
}

.combatant-hp-fill {
  height: 100%; border-radius: 1px; transition: width .4s, background .4s;
}

.combatant-hp-text { font-size: .6rem; color: var(--dim); }
.combatant-class { font-size: .58rem; color: #3a3d55; margin-top: .15rem; font-family: 'Cinzel', serif; letter-spacing: .05em; }

.action-economy {
  display: flex; gap: .5rem; padding: .5rem 1rem;
  background: #0c0d1a; border-top: 1px solid #1a1d2e;
  flex-shrink: 0; align-items: center;
}

.economy-slot {
  display: flex; flex-direction: column; gap: .2rem; flex: 1;
}

.economy-label {
  font-family: 'Cinzel', serif; font-size: .55rem;
  letter-spacing: .12em; text-transform: uppercase; color: #3a3d55;
}

.economy-buttons { display: flex; gap: .35rem; flex-wrap: wrap; position: relative; }

.action-btn {
  background: #13151f; border: 1px solid #2a2d44;
  color: var(--dim); border-radius: 2px; cursor: pointer;
  font-family: 'Cinzel', serif; font-size: .62rem;
  letter-spacing: .06em; padding: .3rem .6rem;
  transition: all .15s; white-space: nowrap;
}

.action-btn:hover:not(:disabled) {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.06);
}

.action-btn:disabled { opacity: .3; cursor: not-allowed; }

.action-btn.used {
  border-color: #1e2035; color: #2a2d44; background: #0b0c15;
  cursor: not-allowed;
}

.action-btn.selected {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.1);
}

.action-btn.danger:hover:not(:disabled) {
  border-color: var(--red); color: #ff8080;
  background: rgba(180,50,50,.08);
}

.action-btn.end-turn {
  background: linear-gradient(135deg, #1a1f06, #2d3510);
  border-color: #4a5a20; color: #8a9a50;
  margin-left: auto;
}
.action-btn.end-turn:hover:not(:disabled) {
  border-color: #8a9a50; color: #b0c060;
  box-shadow: 0 0 12px rgba(100,130,30,.2);
}

.skills-submenu {
  position: absolute; bottom: 100%; left: 0;
  background: #0f1020; border: 1px solid #2a2d44;
  border-radius: 3px; padding: .4rem;
  min-width: 220px; z-index: 10;
  box-shadow: 0 -8px 24px rgba(0,0,0,.6);
}

.skill-btn {
  display: flex; justify-content: space-between; align-items: center;
  width: 100%; background: none; border: none;
  color: var(--dim); font-family: 'Cinzel', serif;
  font-size: .62rem; letter-spacing: .05em;
  padding: .3rem .5rem; cursor: pointer; border-radius: 2px;
  transition: all .15s; gap: .5rem;
}

.skill-btn:hover:not(:disabled) { background: rgba(200,150,42,.06); color: var(--goldl); }
.skill-btn:disabled { opacity: .3; cursor: not-allowed; }

.skill-uses {
  font-size: .58rem; color: var(--gold);
  border: 1px solid #2a2d44; border-radius: 2px;
  padding: .05rem .25rem; white-space: nowrap;
}
.skill-uses.empty { color: var(--red); border-color: #3a1a1a; }

.dice-section {
  background: #0b0c15; border-top: 1px solid #1a1d2e;
  padding: .6rem 1rem; flex-shrink: 0;
}

.dice-prompt {
  font-family: 'Cinzel', serif; font-size: .68rem;
  color: var(--goldl); letter-spacing: .08em; margin-bottom: .4rem;
}

.dice-display { display: flex; gap: .5rem; align-items: center; flex-wrap: wrap; }

.die-face {
  background: #13151f; border: 1px solid #2a2d44;
  border-radius: 3px; width: 48px; height: 48px;
  display: flex; align-items: center; justify-content: center;
  font-family: 'Cinzel', serif; font-size: 1.2rem;
  color: var(--goldl); cursor: pointer;
  transition: all .15s; position: relative;
}

.die-face:hover { border-color: var(--gold); box-shadow: 0 0 10px rgba(200,150,42,.2); }
.die-face.rolling { animation: die-roll .4s ease; }
.die-face.locked { border-color: var(--gold); background: rgba(200,150,42,.1); }

@keyframes die-roll {
  0%   { transform: rotate(0deg) scale(1); }
  25%  { transform: rotate(15deg) scale(1.1); }
  50%  { transform: rotate(-10deg) scale(.95); }
  75%  { transform: rotate(5deg) scale(1.05); }
  100% { transform: rotate(0deg) scale(1); }
}

.dice-label {
  font-size: .6rem; color: var(--dim); position: absolute;
  bottom: 2px; right: 4px; font-family: 'Cinzel', serif;
}

.dice-advantage-note { font-size: .65rem; color: var(--dim); font-style: italic; }

.dice-confirm-btn {
  background: linear-gradient(135deg, #1a1f06, #2d3510);
  border: 1px solid #4a5a20; color: #8a9a50;
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .08em; padding: .3rem .8rem;
  border-radius: 2px; cursor: pointer; transition: all .15s;
}
.dice-confirm-btn:hover { border-color: #8a9a50; color: #b0c060; }
.dice-confirm-btn:disabled { opacity: .4; cursor: not-allowed; }

.combat-log {
  background: #090a12; border-top: 1px solid #1a1d2e;
  padding: .5rem 1rem; max-height: 110px;
  overflow-y: auto; flex-shrink: 0;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #090a12;
}

.log-entry {
  font-size: .68rem; color: var(--dim); line-height: 1.7;
  padding: .05rem 0; border-bottom: 1px solid #0f1020;
}

.log-entry:last-child { border-bottom: none; color: var(--text); }
.log-entry.hit   { color: #c07050; }
.log-entry.crit  { color: #e08040; font-weight: bold; }
.log-entry.miss  { color: #4a4d65; }
.log-entry.heal  { color: #50a070; }
.log-entry.death { color: var(--red); }
.log-entry.flee  { color: var(--gold); }
.log-entry.system { color: #5a5d80; font-style: italic; }

.initiative-screen {
  display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  flex: 1; gap: 1rem; padding: 2rem;
}

.initiative-title {
  font-family: 'Cinzel', serif; font-size: 1.4rem;
  color: var(--goldl); letter-spacing: .15em; text-align: center;
}

.initiative-sub {
  font-size: .8rem; color: var(--dim); text-align: center; line-height: 1.7;
}

.initiative-die {
  width: 96px; height: 96px;
  background: #13151f; border: 2px solid #2a2d44;
  border-radius: 6px; display: flex; align-items: center;
  justify-content: center; cursor: pointer;
  font-family: 'Cinzel', serif; font-size: 2.4rem;
  color: var(--goldl); transition: all .2s;
  box-shadow: 0 0 20px rgba(0,0,0,.5); position: relative;
}

.initiative-die:hover { border-color: var(--gold); box-shadow: 0 0 24px rgba(200,150,42,.2); }
.initiative-die.rolled { border-color: var(--gold); box-shadow: 0 0 30px rgba(200,150,42,.3); }
.initiative-die.rolling { animation: die-roll .5s ease; }
.initiative-bonus { font-size: .72rem; color: var(--gold); font-family: 'Cinzel', serif; }

.death-save-screen {
  display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  flex: 1; gap: 1rem; padding: 2rem;
  background: rgba(120,0,0,.05);
}

.death-save-title {
  font-family: 'Cinzel', serif; font-size: 1.1rem;
  color: var(--red); letter-spacing: .15em;
}

.ds-pip {
  width: 14px; height: 14px; border-radius: 50%;
  border: 1px solid #2a2d44; background: #0b0c15; transition: all .3s;
}
.ds-pip.success { background: var(--grn); border-color: var(--grn); box-shadow: 0 0 6px rgba(50,180,80,.4); }
.ds-pip.failure { background: var(--red); border-color: var(--red); box-shadow: 0 0 6px rgba(180,50,50,.4); }
.ds-group-label { font-family: 'Cinzel', serif; font-size: .65rem; letter-spacing: .1em; margin-bottom: .4rem; }

.confirm-bar {
  display: flex; align-items: center; justify-content: space-between;
  padding: .4rem 1rem;
  background: rgba(200,150,42,.05);
  border-top: 1px solid rgba(200,150,42,.2);
  flex-shrink: 0;
}

.confirm-text {
  font-family: 'Cinzel', serif; font-size: .68rem;
  color: var(--goldl); letter-spacing: .08em;
}

.confirm-ok {
  background: linear-gradient(135deg, #2a1f06, #4a3510);
  border: 1px solid var(--gold); color: var(--goldl);
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .1em; padding: .3rem .9rem;
  border-radius: 2px; cursor: pointer; transition: all .15s;
}
.confirm-ok:hover { background: linear-gradient(135deg, #4a3510, #6a4e18); }

.confirm-cancel {
  background: none; border: 1px solid #2a2d44;
  color: var(--dim); font-family: 'Cinzel', serif;
  font-size: .62rem; padding: .3rem .7rem;
  border-radius: 2px; cursor: pointer; transition: all .15s;
  margin-right: .5rem;
}
.confirm-cancel:hover { border-color: var(--red); color: var(--red); }
`

function EnemyIcon({ isAlly = false }) {
    return isAlly ? (
        <svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="32" cy="20" r="12" stroke="#4a7a5a" strokeWidth="1.5" fill="#1a3a2a" />
            <path d="M14 56c0-9.94 8.06-18 18-18s18 8.06 18 18" stroke="#4a7a5a" strokeWidth="1.5" fill="none" />
            <path d="M26 34 L32 28 L38 34" stroke="#6a9a7a" strokeWidth="1.5" />
        </svg>
    ) : (
        <svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M32 8 L40 24 L56 26 L44 38 L47 54 L32 46 L17 54 L20 38 L8 26 L24 24 Z"
                stroke="#6a2a2a" strokeWidth="1.5" fill="#2a1010" />
            <circle cx="24" cy="26" r="3" fill="#c06060" opacity=".8" />
            <circle cx="40" cy="26" r="3" fill="#c06060" opacity=".8" />
            <path d="M24 38 Q32 44 40 38" stroke="#c06060" strokeWidth="1.5" fill="none" />
        </svg>
    )
}

function rollDie(sides) { return Math.floor(Math.random() * sides) + 1 }
function hpColor(pct) {
    if (pct > 55) return 'var(--grn)'
    if (pct > 25) return 'var(--amber)'
    return 'var(--red)'
}
function parseDamageDie(die) {
    const match = die.match(/^(\d+)?d(\d+)$/)
    if (!match) return { count: 1, sides: 6 }
    return { count: parseInt(match[1] || '1'), sides: parseInt(match[2]) }
}

export default function CombatModal({
    campaignId, player, abilities,
    initiativeBonus = 0, hasAdvantage = false,
    onCombatEnd, onPlayerUpdate,
}) {
    const [phase, setPhase] = useState('initiative')
    const [combatState, setCombatState] = useState(null)

    const [diceValues, setDiceValues] = useState([])
    const [diceRolling, setDiceRolling] = useState([])
    const [lockedDie, setLockedDie] = useState(null)
    const [pendingDiceConfig, setPendingDiceConfig] = useState(null)

    const [initiativeRoll, setInitiativeRoll] = useState(null)
    const [initiativeRolling, setInitiativeRolling] = useState(false)
    const [initiativeAdvRolls, setInitiativeAdvRolls] = useState([null, null])

    const [actionUsed, setActionUsed] = useState(false)
    const [bonusActionUsed, setBonusActionUsed] = useState(false)
    const [attacksRemaining, setAttacksRemaining] = useState(1)
    const [actionSurgeAvailable, setActionSurgeAvailable] = useState(false)

    const [selectedAction, setSelectedAction] = useState(null)
    const [selectedTarget, setSelectedTarget] = useState(null)
    const [showSkillsMenu, setShowSkillsMenu] = useState(false)
    const [isCrit, setIsCrit] = useState(false)

    const [deathSuccesses, setDeathSuccesses] = useState(0)
    const [deathFailures, setDeathFailures] = useState(0)

    const [shakingEnemy, setShakingEnemy] = useState(null)
    const [playerAttacking, setPlayerAttacking] = useState(false)
    const [attackingEnemyId, setAttackingEnemyId] = useState(null)

    const [log, setLog] = useState([])
    const logRef = useRef(null)
    const logData = useRef([])

    useEffect(() => {
        api.getCombatState(campaignId).then(s => { if (s) setCombatState(s) }).catch(() => { })
    }, [campaignId])

    useEffect(() => {
        if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight
    }, [log])

    const addLog = (text, type = '') => {
        const entry = { text, type, id: Date.now() + Math.random() }
        logData.current = [...logData.current, entry]
        setLog(logData.current)
    }

    const refreshCombat = async () => {
        try {
            const s = await api.getCombatState(campaignId)
            if (s) setCombatState(s)
        } catch (e) { }
    }

    // ── Derived ────────────────────────────────────────────────────────────────
    const turnOrder = combatState?.turn_order || []
    const currentActor = combatState?.current_actor
    const enemies = (combatState?.enemies || []).filter(e => e.participant_type === 'enemy')
    // Allies from combat_enemies table go in the bottom row with the player
    const npcAllies = (combatState?.enemies || []).filter(e => e.participant_type === 'ally')
    const round = combatState?.round_number || 1
    const isPlayerTurn = currentActor?.participant_type === 'player'
    const canAct = phase === 'player_turn' && isPlayerTurn

    // ── Initiative ─────────────────────────────────────────────────────────────
    const rollInitiative = () => {
        setInitiativeRolling(true)
        if (hasAdvantage) {
            const r1 = rollDie(20), r2 = rollDie(20)
            setInitiativeAdvRolls([r1, r2])
        } else {
            const r = rollDie(20)
            setTimeout(() => { setInitiativeRoll(r); setInitiativeRolling(false) }, 500)
            return
        }
        setTimeout(() => setInitiativeRolling(false), 500)
    }

    const chooseInitiativeAdvantage = (idx) => setInitiativeRoll(initiativeAdvRolls[idx])

    const confirmInitiative = async () => {
        if (initiativeRoll === null) return
        try {
            await api.submitInitiative(campaignId, initiativeRoll,
                hasAdvantage ? initiativeAdvRolls : null)
            addLog(
                `Initiative: ${player.name} rolls ${initiativeRoll} + ${initiativeBonus} = ${initiativeRoll + initiativeBonus}`,
                'system'
            )

            const freshState = await api.getCombatState(campaignId)
            setCombatState(freshState)
            const actor = freshState?.current_actor

            if (actor?.participant_type === 'player') {
                setActionUsed(false)
                setBonusActionUsed(false)
                setAttacksRemaining(freshState?.action_economy?.attacks_remaining || 1)
                setActionSurgeAvailable(freshState?.action_economy?.action_surge_available || false)
                addLog(`--- Round 1 — ${player.name}'s turn ---`, 'system')
                setPhase('player_turn')
            } else {
                addLog('--- Round 1 begins ---', 'system')
                setPhase('enemy_turns')
                try {
                    const result = await api.processInitialTurns(campaignId)
                    const turnResults = result.turn_results || []
                    for (let i = 0; i < turnResults.length; i++) {
                        await new Promise(r => setTimeout(r, 1600))
                        const t = turnResults[i]
                        addLog(t.text, t.hit ? 'hit' : 'miss')
                        if (t.player_downed) {
                            await refreshCombat()
                            if (onPlayerUpdate) await onPlayerUpdate()
                            setDeathSuccesses(0); setDeathFailures(0)
                            setPhase('death_saves'); return
                        }
                        if (t.combat_ended) { endCombatVictory(); return }
                    }
                } catch (e) { console.error('Failed to process initial turns:', e) }

                await refreshCombat()
                if (onPlayerUpdate) await onPlayerUpdate()
                addLog(`--- ${player.name}'s turn ---`, 'system')
                setActionUsed(false); setBonusActionUsed(false)
                const freshState = await api.getCombatState(campaignId)
                setCombatState(freshState)
                const livingEnemies = (freshState?.enemies || [])
                    .filter(e => e.participant_type === 'enemy' && e.is_alive)
                if (livingEnemies.length === 0) {
                    endCombatVictory(); return
                }
                setPhase('player_turn')
            }
        } catch (e) { console.error('Initiative failed:', e) }
    }

    // ── Attack flow ────────────────────────────────────────────────────────────
    const startAttack = () => {
        setSelectedAction('attack')
        setSelectedTarget(null)
        setShowSkillsMenu(false)
    }

    const selectTarget = (enemyId) => {
        if (selectedAction !== 'attack') return
        setSelectedTarget(enemyId)
    }

    const confirmAttack = async () => {
        if (!selectedTarget) return
        startDiceRoll({
            count: hasAdvantage ? 2 : 1,
            sides: 20,
            label: 'Attack Roll',
            isAdvantage: hasAdvantage,
            onConfirm: async (rolls, chosen) => {
                const roll = chosen ?? rolls[0]

                // Animate player attacking upward
                setPlayerAttacking(true)
                setTimeout(() => setPlayerAttacking(false), 400)

                try {
                    const result = await api.resolveAttack(campaignId, selectedTarget, roll)

                    if (result.hit) {
                        addLog(result.is_crit
                            ? `${player.name} attacks ${result.target_name} with ${result.weapon_name} — Critical Hit! (rolled ${roll})`
                            : `${player.name} attacks ${result.target_name} with ${result.weapon_name} — Hit! (rolled ${roll + result.attack_bonus} vs AC ${result.enemy_ac})`,
                            result.is_crit ? 'crit' : 'hit'
                        )
                        setIsCrit(result.is_crit)
                        const { count, sides } = parseDamageDie(result.damage_die)
                        startDiceRoll({
                            count: result.is_crit ? count * 2 : count,
                            sides,
                            label: result.is_crit ? 'Damage (CRIT — roll twice!)' : `Damage Roll (${result.damage_die})`,
                            isAdvantage: false,
                            onConfirm: async (dmgRolls) => {
                                await confirmDamage(dmgRolls, result.is_crit, result.target_name, result.weapon_name)
                            }
                        })
                    } else {
                        addLog(`${player.name} attacks ${result.target_name} with ${result.weapon_name} — Miss (rolled ${roll + result.attack_bonus} vs AC ${result.enemy_ac})`, 'miss')
                        // Shake the enemy on miss too (near-miss feel)
                        setShakingEnemy(selectedTarget)
                        setTimeout(() => setShakingEnemy(null), 500)
                        finishAttack()
                    }
                } catch (e) { console.error('Attack failed:', e); cancelAction() }
            }
        })
    }

    const confirmDamage = async (rolls, crit, targetName, weaponName) => {
        try {
            const result = await api.resolveDamage(campaignId, rolls, crit)

            addLog(
                `${player.name} deals ${result.damage_dealt} damage to ${targetName} with ${weaponName}${result.enemy_dead ? ' — Enemy falls!' : ''}`,
                result.enemy_dead ? 'crit' : 'hit'
            )

            // Shake the hit enemy
            setShakingEnemy(selectedTarget)
            setTimeout(() => setShakingEnemy(null), 500)

            await refreshCombat()

            if (result.all_enemies_defeated) {
                addLog('All enemies defeated!', 'system')
                endCombatVictory(); return
            }

            if (result.can_attack_again) {
                setAttacksRemaining(r => r - 1)
                setSelectedTarget(null)
                setSelectedAction('attack')
                setPhase('player_turn'); return
            }

            finishAttack()
        } catch (e) { console.error('Damage failed:', e); cancelAction() }
    }

    const finishAttack = () => {
        setActionUsed(true)
        setAttacksRemaining(0)
        setSelectedAction(null)
        setSelectedTarget(null)
        setPhase('player_turn')
    }

    // ── Dice ───────────────────────────────────────────────────────────────────
    const startDiceRoll = (config) => {
        setDiceValues(Array.from({ length: config.count }, () => null))
        setDiceRolling(Array(config.count).fill(false))
        setLockedDie(null)
        setPendingDiceConfig(config)
        setPhase('rolling')
    }

    const rollSingleDie = (index) => {
        if (!pendingDiceConfig) return
        setDiceRolling(r => r.map((v, i) => i === index ? true : v))
        setTimeout(() => {
            const val = rollDie(pendingDiceConfig.sides)
            setDiceValues(v => v.map((x, i) => i === index ? val : x))
            setDiceRolling(r => r.map((v, i) => i === index ? false : v))
        }, 300)
    }

    const rollAllDice = () => {
        if (!pendingDiceConfig) return
        setDiceRolling(Array(pendingDiceConfig.count).fill(true))
        setTimeout(() => {
            const vals = Array.from({ length: pendingDiceConfig.count }, () => rollDie(pendingDiceConfig.sides))
            setDiceValues(vals)
            setDiceRolling(Array(pendingDiceConfig.count).fill(false))
        }, 350)
    }

    const confirmDiceRoll = () => {
        if (!pendingDiceConfig) return
        const allRolled = diceValues.every(v => v !== null)
        if (!allRolled) return
        if (pendingDiceConfig.isAdvantage && diceValues.length === 2) {
            if (lockedDie === null) return
            const chosen = diceValues[lockedDie]
            const config = pendingDiceConfig
            setPendingDiceConfig(null)
            setPhase('player_turn')
            config.onConfirm(diceValues, chosen)
        } else {
            const config = pendingDiceConfig
            setPendingDiceConfig(null)
            setPhase('player_turn')
            config.onConfirm(diceValues, null)
        }
    }

    // ── Skills ─────────────────────────────────────────────────────────────────
    const combatAbilities = (abilities || []).filter(ab => {
        const n = ab.name.toLowerCase()
        return n.includes('second wind') || n.includes('action surge') ||
            n.includes('indomitable') || n.includes('rage') || n.includes('cunning')
    })

    const useSkill = async (ability) => {
        setShowSkillsMenu(false)
        const name = ability.name.toLowerCase()
        if (name.includes('second wind')) {
            try {
                const result = await api.useCombatAbility(campaignId, 'second_wind')
                addLog(`${player.name} uses Second Wind and recovers ${result.healing} HP.`, 'heal')
                setBonusActionUsed(true)
                await refreshCombat()
                if (onPlayerUpdate) await onPlayerUpdate()
            } catch (e) { console.error(e) }
        } else if (name.includes('action surge')) {
            try {
                await api.useCombatAbility(campaignId, 'action_surge')
                addLog(`${player.name} activates Action Surge!`, 'system')
                setActionUsed(false)
                setActionSurgeAvailable(false)
            } catch (e) { console.error(e) }
        }
    }

    // ── Flee ───────────────────────────────────────────────────────────────────
    const startFlee = () => { setSelectedAction('flee'); setShowSkillsMenu(false) }

    const pickFleeSkill = (skill) => {
        setSelectedAction(null)
        startDiceRoll({
            count: 1, sides: 20,
            label: `${skill} Check (DC 15 to flee)`,
            isAdvantage: false,
            onConfirm: async (rolls) => {
                try {
                    const result = await api.fleeCombat(campaignId, rolls[0], skill)
                    addLog(result.text, result.success ? 'flee' : 'miss')
                    if (result.success) {
                        setPhase('fled')
                        setTimeout(() => onCombatEnd('fled', log), 1500)
                    } else {
                        setActionUsed(true)
                        setSelectedAction(null)
                        setPhase('fled')
                        setTimeout(() => onCombatEnd('fled', logData.current), 1500)
                        if (onPlayerUpdate) await onPlayerUpdate()
                    }
                } catch (e) { console.error(e) }
            }
        })
    }

    // ── End turn ───────────────────────────────────────────────────────────────
    const endTurn = async () => {
        setSelectedAction(null); setSelectedTarget(null); setShowSkillsMenu(false)
        setPhase('enemy_turns')
        addLog('--- Player turn ends ---', 'system')
        try {
            const result = await api.endCombatTurn(campaignId)
            const turnResults = result.turn_results || []
            for (let i = 0; i < turnResults.length; i++) {
                await new Promise(r => setTimeout(r, 1600))
                const t = turnResults[i]
                addLog(t.text, t.hit ? (t.damage ? 'hit' : 'system') : 'miss')
                if (t.player_downed) {
                    await refreshCombat()
                    if (onPlayerUpdate) await onPlayerUpdate()
                    setDeathSuccesses(0); setDeathFailures(0)
                    setPhase('death_saves'); return
                }
                if (t.combat_ended) { endCombatVictory(); return }
            }

            // ── Check if all enemies are dead even if combat_ended wasn't set ──
            const freshState = await api.getCombatState(campaignId)
            setCombatState(freshState)
            const livingEnemies = (freshState?.enemies || [])
                .filter(e => e.participant_type === 'enemy' && e.is_alive)
            if (livingEnemies.length === 0) {
                endCombatVictory(); return
            }

            if (onPlayerUpdate) await onPlayerUpdate()
            const actor = freshState?.current_actor
            if (actor?.participant_type === 'player') {
                setActionUsed(false); setBonusActionUsed(false)
                setAttacksRemaining(freshState?.action_economy?.attacks_remaining || 1)
                setActionSurgeAvailable(freshState?.action_economy?.action_surge_available || false)
                addLog(`--- Round ${freshState?.round_number} — ${player.name}'s turn ---`, 'system')
                setPhase('player_turn')
            } else {
                setPhase('player_turn')
            }
        } catch (e) { console.error('End turn failed:', e); setPhase('player_turn') }
    }

    // ── Death saves ─────────────────────────────────────────────────────────────
    const rollDeathSave = () => {
        startDiceRoll({
            count: 1, sides: 20,
            label: 'Death Saving Throw (10+ = Success)',
            isAdvantage: false,
            onConfirm: async (rolls) => {
                const roll = rolls[0]
                const nat20 = roll === 20
                const nat1 = roll === 1

                if (nat20) {
                    addLog(`${player.name} rolls a Natural 20 — stabilizes with 1 HP!`, 'heal')
                    setDeathSuccesses(3)
                    if (onPlayerUpdate) await onPlayerUpdate()
                    // Resume combat — check whose turn it is
                    await refreshCombat()
                    const freshState = await api.getCombatState(campaignId)
                    setCombatState(freshState)
                    setActionUsed(false); setBonusActionUsed(false)
                    setAttacksRemaining(freshState?.action_economy?.attacks_remaining || 1)
                    addLog(`${player.name} is back on their feet!`, 'heal')
                    setPhase('player_turn')
                    return
                }

                if (nat1) {
                    const newFail = deathFailures + 2
                    setDeathFailures(newFail)
                    addLog(`${player.name} rolls a 1 — two failures!`, 'death')
                    if (newFail >= 3) {
                        addLog(`${player.name} has died.`, 'death')
                        onCombatEnd('death', logData.current); return
                    }
                    setPhase('death_saves'); return
                }

                if (roll >= 10) {
                    const newSucc = deathSuccesses + 1
                    setDeathSuccesses(newSucc)
                    addLog(`${player.name} succeeds on death save (${newSucc}/3).`, 'heal')
                    if (newSucc >= 3) {
                        addLog(`${player.name} stabilizes!`, 'heal')
                        if (onPlayerUpdate) await onPlayerUpdate()
                        // Stabilized — player continues but it's now the enemy's turn, advance
                        setDeathSuccesses(0); setDeathFailures(0)
                        setPhase('enemy_turns')
                        try {
                            const result = await api.endCombatTurn(campaignId)
                            const turnResults = result.turn_results || []
                            for (let i = 0; i < turnResults.length; i++) {
                                await new Promise(r => setTimeout(r, 1600))
                                const t = turnResults[i]
                                addLog(t.text, t.hit ? 'hit' : 'miss')
                                if (t.player_downed) {
                                    setDeathSuccesses(0); setDeathFailures(0)
                                    setPhase('death_saves'); return
                                }
                                if (t.combat_ended) { endCombatVictory(); return }
                            }
                        } catch (e) { console.error(e) }
                        await refreshCombat()
                        if (onPlayerUpdate) await onPlayerUpdate()
                        setActionUsed(false); setBonusActionUsed(false)
                        addLog(`--- ${player.name}'s turn ---`, 'system')
                        setPhase('player_turn')
                        return
                    }
                    setPhase('death_saves'); return
                }

                // Failure
                const newFail = deathFailures + 1
                setDeathFailures(newFail)
                addLog(`${player.name} fails death save (${newFail}/3).`, 'death')
                if (newFail >= 3) {
                    addLog(`${player.name} has died.`, 'death')
                    onCombatEnd('death', log); return
                }
                setPhase('death_saves')
            }
        })
    }

    // ── Victory ────────────────────────────────────────────────────────────────
    const endCombatVictory = async () => {
        try { await api.endCombat(campaignId) } catch (e) { }
        addLog('Victory! All enemies have been defeated.', 'system')
        setTimeout(() => onCombatEnd('victory', logData.current), 800)
    }
    const cancelAction = () => {
        setSelectedAction(null); setSelectedTarget(null)
        setShowSkillsMenu(false); setPendingDiceConfig(null)
        setPhase('player_turn')
    }

    // ─────────────────────────────────────────────────────────────────────────
    return (
        <>
            <style dangerouslySetInnerHTML={{ __html: COMBAT_STYLES }} />
            <div className="combat-overlay">
                <div className="combat-modal">

                    {/* Header */}
                    <div className="combat-header">
                        <div className="combat-title">⚔ Combat</div>
                        <div className="combat-round">
                            {phase === 'initiative' ? 'Roll for Initiative' :
                                phase === 'enemy_turns' ? 'Enemy Turn…' :
                                    phase === 'death_saves' ? '☠ Death Saving Throws' :
                                        `Round ${round}`}
                        </div>
                    </div>

                    {/* Turn Order Bar */}
                    {phase !== 'initiative' && turnOrder.length > 0 && (
                        <div className="turn-order-bar">
                            {turnOrder.map((p, i) => (
                                <div key={p.id + i} className={[
                                    'turn-chip', p.participant_type,
                                    i === 0 ? 'active' : '',
                                    !p.is_alive ? 'dead' : ''
                                ].filter(Boolean).join(' ')}>
                                    <span className="turn-chip-icon">
                                        {p.participant_type === 'player' ? '⚔' :
                                            p.participant_type === 'enemy' ? '☠' : '🛡'}
                                    </span>
                                    {p.name}
                                </div>
                            ))}
                        </div>
                    )}

                    {/* Initiative Screen */}
                    {phase === 'initiative' && (
                        <div className="initiative-screen">
                            <div className="initiative-title">Roll for Initiative</div>
                            <div className="initiative-sub">
                                Roll a d20{initiativeBonus !== 0 ? ` + ${initiativeBonus} (DEX modifier)` : ''}.
                                {hasAdvantage && ' You have advantage — roll twice and pick the higher result.'}
                            </div>
                            {!hasAdvantage ? (
                                <>
                                    <div
                                        className={`initiative-die${initiativeRolling ? ' rolling' : ''}${initiativeRoll !== null ? ' rolled' : ''}`}
                                        onClick={rollInitiative}
                                    >
                                        {initiativeRoll !== null ? initiativeRoll : '?'}
                                        <span className="dice-label">d20</span>
                                    </div>
                                    {initiativeRoll !== null && (
                                        <div className="initiative-bonus">
                                            {initiativeRoll} + {initiativeBonus} = {initiativeRoll + initiativeBonus}
                                        </div>
                                    )}
                                </>
                            ) : (
                                <>
                                    <div style={{ display: 'flex', gap: '1rem' }}>
                                        {[0, 1].map(i => (
                                            <div key={i}
                                                className={`initiative-die${initiativeRolling ? ' rolling' : ''}${initiativeAdvRolls[i] !== null ? ' rolled' : ''}${initiativeRoll === initiativeAdvRolls[i] && initiativeAdvRolls[i] !== null ? ' locked' : ''}`}
                                                onClick={() => initiativeAdvRolls[i] !== null ? chooseInitiativeAdvantage(i) : rollInitiative()}
                                            >
                                                {initiativeAdvRolls[i] ?? '?'}
                                                <span className="dice-label">d20</span>
                                            </div>
                                        ))}
                                    </div>
                                    {initiativeAdvRolls[0] !== null && (
                                        <div className="initiative-sub" style={{ marginTop: 0 }}>Click the die you want to use</div>
                                    )}
                                    {initiativeRoll !== null && (
                                        <div className="initiative-bonus">
                                            Using {initiativeRoll} + {initiativeBonus} = {initiativeRoll + initiativeBonus}
                                        </div>
                                    )}
                                </>
                            )}
                            <button className="btn-gold" disabled={initiativeRoll === null} onClick={confirmInitiative}>
                                Enter Combat
                            </button>
                        </div>
                    )}

                    {/* Death Saves Screen */}
                    {phase === 'death_saves' && (
                        <div className="death-save-screen">
                            <div className="death-save-title">☠ Death Saving Throws</div>
                            <div style={{ display: 'flex', gap: '1.5rem' }}>
                                <div>
                                    <div className="ds-group-label" style={{ color: 'var(--grn)' }}>Successes</div>
                                    <div style={{ display: 'flex', gap: '.4rem' }}>
                                        {[0, 1, 2].map(i => <div key={i} className={`ds-pip${i < deathSuccesses ? ' success' : ''}`} />)}
                                    </div>
                                </div>
                                <div>
                                    <div className="ds-group-label" style={{ color: 'var(--red)' }}>Failures</div>
                                    <div style={{ display: 'flex', gap: '.4rem' }}>
                                        {[0, 1, 2].map(i => <div key={i} className={`ds-pip${i < deathFailures ? ' failure' : ''}`} />)}
                                    </div>
                                </div>
                            </div>
                            <button className="btn-gold" onClick={rollDeathSave}>Roll Death Save</button>
                        </div>
                    )}

                    {/* Battlefield */}
                    {phase !== 'initiative' && phase !== 'death_saves' && (
                        <div className="battlefield">

                            {/* ── Enemy Row (top) — enemies only ── */}
                            <div className="enemy-row">
                                {enemies.map(enemy => (
                                    <div
                                        key={enemy.id}
                                        className={[
                                            'enemy-card',
                                            !enemy.is_alive ? 'dead' : '',
                                            selectedTarget === enemy.id ? 'targeted' : '',
                                            selectedAction === 'attack' && enemy.is_alive && !selectedTarget ? 'selecting' : '',
                                            shakingEnemy === enemy.id ? 'shake' : '',
                                            attackingEnemyId === enemy.id ? 'attack-out' : '',
                                        ].filter(Boolean).join(' ')}
                                        onClick={() => enemy.is_alive && selectedAction === 'attack' && selectTarget(enemy.id)}
                                    >
                                        {selectedTarget === enemy.id && <div className="target-arrow">▼</div>}
                                        <div className="enemy-icon">
                                            <EnemyIcon isAlly={false} />
                                            {enemy.is_bloodied && enemy.is_alive && (
                                                <div className="bloodied-indicator">🩸</div>
                                            )}
                                        </div>
                                        {enemy.is_alive
                                            ? <div className="enemy-name">{enemy.name}</div>
                                            : <div className="enemy-dead-mark">✝ {enemy.name}</div>
                                        }
                                    </div>
                                ))}
                            </div>

                            {/* ── Player/Companion/Ally Row (bottom) ── */}
                            <div className="player-row">

                                {/* Player */}
                                <div className={[
                                    'combatant-card',
                                    isPlayerTurn && phase === 'player_turn' ? 'active-turn' : '',
                                    player.current_hp === 0 ? 'downed' : '',
                                    playerAttacking ? 'player-attacking' : '',
                                ].filter(Boolean).join(' ')}>
                                    <div className="combatant-name">{player.name}</div>
                                    <div className="combatant-hp-bar">
                                        <div className="combatant-hp-fill" style={{
                                            width: `${Math.max(0, (player.current_hp / player.max_hp) * 100)}%`,
                                            background: hpColor(player.current_hp / player.max_hp * 100)
                                        }} />
                                    </div>
                                    <div className="combatant-hp-text">{player.current_hp}/{player.max_hp} HP</div>
                                    <div className="combatant-class">{player.class}</div>
                                </div>

                                {/* NPC Allies (from combat_enemies with participant_type='ally') */}
                                {npcAllies.map(ally => (
                                    <div key={ally.id} className={[
                                        'combatant-card ally-card',
                                        !ally.is_alive ? 'downed' : '',
                                        currentActor?.id === ally.id ? 'active-turn' : '',
                                    ].filter(Boolean).join(' ')}>
                                        <div className="combatant-name">{ally.name}</div>
                                        <div className="combatant-hp-bar">
                                            <div className="combatant-hp-fill" style={{
                                                width: `${Math.max(0, (ally.current_hp / ally.max_hp) * 100)}%`,
                                                background: hpColor(ally.current_hp / ally.max_hp * 100)
                                            }} />
                                        </div>
                                        <div className="combatant-hp-text">{ally.current_hp}/{ally.max_hp} HP</div>
                                        <div className="combatant-class">Ally</div>
                                    </div>
                                ))}

                            </div>
                        </div>
                    )}

                    {/* Dice Rolling */}
                    {phase === 'rolling' && pendingDiceConfig && (
                        <div className="dice-section">
                            <div className="dice-prompt">{pendingDiceConfig.label}</div>
                            <div className="dice-display">
                                {diceValues.map((val, i) => (
                                    <div
                                        key={i}
                                        className={[
                                            'die-face',
                                            diceRolling[i] ? 'rolling' : '',
                                            pendingDiceConfig.isAdvantage && lockedDie === i ? 'locked' : ''
                                        ].filter(Boolean).join(' ')}
                                        onClick={() => {
                                            if (pendingDiceConfig.isAdvantage && val !== null) setLockedDie(i)
                                            else rollSingleDie(i)
                                        }}
                                    >
                                        {val ?? '?'}
                                        <span className="dice-label">d{pendingDiceConfig.sides}</span>
                                    </div>
                                ))}
                                {diceValues.some(v => v === null) && (
                                    <button className="dice-confirm-btn" onClick={rollAllDice}>Roll All</button>
                                )}
                                {diceValues.every(v => v !== null) && (
                                    <>
                                        {pendingDiceConfig.isAdvantage && (
                                            <span className="dice-advantage-note">
                                                {lockedDie !== null ? `Using ${diceValues[lockedDie]}` : 'Click the die to use'}
                                            </span>
                                        )}
                                        <button
                                            className="dice-confirm-btn"
                                            disabled={pendingDiceConfig.isAdvantage && lockedDie === null}
                                            onClick={confirmDiceRoll}
                                        >
                                            Confirm
                                        </button>
                                    </>
                                )}
                                <button className="action-btn danger" style={{ marginLeft: 'auto' }} onClick={cancelAction}>
                                    Cancel
                                </button>
                            </div>
                        </div>
                    )}

                    {/* Target Confirm Bar */}
                    {phase === 'player_turn' && selectedTarget && selectedAction === 'attack' && (
                        <div className="confirm-bar">
                            <div className="confirm-text">
                                ⚔ Attack {enemies.find(e => e.id === selectedTarget)?.name}
                            </div>
                            <div style={{ display: 'flex', gap: '.4rem' }}>
                                <button className="confirm-cancel" onClick={cancelAction}>Cancel</button>
                                <button className="confirm-ok" onClick={confirmAttack}>Attack</button>
                            </div>
                        </div>
                    )}

                    {/* Flee Skill Picker */}
                    {phase === 'player_turn' && selectedAction === 'flee' && (
                        <div className="confirm-bar">
                            <div className="confirm-text">Choose skill to flee:</div>
                            <div style={{ display: 'flex', gap: '.4rem' }}>
                                <button className="confirm-ok" onClick={() => pickFleeSkill('Athletics')}>Athletics</button>
                                <button className="confirm-ok" onClick={() => pickFleeSkill('Acrobatics')}>Acrobatics</button>
                                <button className="confirm-cancel" onClick={cancelAction}>Cancel</button>
                            </div>
                        </div>
                    )}

                    {/* Action Menu */}
                    {phase === 'player_turn' && !selectedTarget && selectedAction !== 'flee' && (
                        <div className="action-economy">

                            <div className="economy-slot">
                                <div className="economy-label">Action</div>
                                <div className="economy-buttons">
                                    <button
                                        className={`action-btn${actionUsed ? ' used' : ''}${selectedAction === 'attack' ? ' selected' : ''}`}
                                        disabled={!canAct || actionUsed}
                                        onClick={startAttack}
                                    >⚔ Attack</button>

                                    <div style={{ position: 'relative' }}>
                                        <button
                                            className={`action-btn${showSkillsMenu ? ' selected' : ''}`}
                                            disabled={!canAct}
                                            onClick={() => setShowSkillsMenu(s => !s)}
                                        >✦ Skills</button>
                                        {showSkillsMenu && (
                                            <div className="skills-submenu">
                                                {combatAbilities.length === 0 && (
                                                    <div style={{ padding: '.3rem .5rem', fontSize: '.65rem', color: 'var(--dim)' }}>
                                                        No combat abilities
                                                    </div>
                                                )}
                                                {combatAbilities.map(ab => (
                                                    <button key={ab.id} className="skill-btn"
                                                        disabled={ab.current_uses === 0}
                                                        onClick={() => useSkill(ab)}
                                                    >
                                                        <span>{ab.name}</span>
                                                        <span className={`skill-uses${ab.current_uses === 0 ? ' empty' : ''}`}>
                                                            {ab.refresh_type === 'per_turn' ? '∞' : `${ab.current_uses}/${ab.max_uses}`}
                                                        </span>
                                                    </button>
                                                ))}
                                            </div>
                                        )}
                                    </div>

                                    <button className="action-btn" disabled title="Coming soon">✧ Spells</button>

                                    <button className="action-btn" disabled={!canAct || actionUsed}
                                        onClick={() => { setActionUsed(true); addLog(`${player.name} passes their action.`, 'system') }}
                                    >Pass</button>

                                    <button className="action-btn danger" disabled={!canAct || actionUsed}
                                        onClick={startFlee}
                                    >↪ Flee</button>
                                </div>
                            </div>

                            <div className="economy-slot">
                                <div className="economy-label">Bonus Action</div>
                                <div className="economy-buttons">
                                    <button className={`action-btn${bonusActionUsed ? ' used' : ''}`}
                                        disabled={!canAct || bonusActionUsed}
                                        onClick={() => { setBonusActionUsed(true); addLog(`${player.name} passes bonus action.`, 'system') }}
                                    >Pass</button>
                                </div>
                            </div>

                            <button className="action-btn end-turn"
                                disabled={!canAct || phase === 'enemy_turns'}
                                onClick={endTurn}
                            >End Turn →</button>

                        </div>
                    )}

                    {/* Enemy Turns In Progress */}
                    {phase === 'enemy_turns' && (
                        <div className="action-economy" style={{ justifyContent: 'center' }}>
                            <div style={{ fontFamily: 'Cinzel, serif', fontSize: '.72rem', color: 'var(--dim)', letterSpacing: '.1em' }}>
                                Enemy turn resolving…
                            </div>
                        </div>
                    )}

                    {/* Combat Log */}
                    <div className="combat-log" ref={logRef}>
                        {log.length === 0 && <div className="log-entry system">Combat begins…</div>}
                        {log.map(entry => (
                            <div key={entry.id} className={`log-entry ${entry.type}`}>{entry.text}</div>
                        ))}
                    </div>

                </div>
            </div>
        </>
    )
}