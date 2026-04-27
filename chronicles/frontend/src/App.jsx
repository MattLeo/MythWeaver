import { useState } from 'react'
import './index.css'
import * as api from './api/client.js'
import TitleScreen from './components/TitleScreen.jsx'
import CharacterCreation from './components/CharacterCreation.jsx'
import Sidebar from './components/Sidebar.jsx'
import GameScreen from './components/GameScreen.jsx'
import DiceRollOverlay from './components/DiceRollOverlay.jsx'
import LevelUpModal from './components/LevelUpModal.jsx'

const PHASE = {
  TITLE: 'title',
  CREATION: 'creation',
  LOADING: 'loading',
  RESUMING: 'resuming',
  GAME: 'game',
}

export default function App() {
  const [phase, setPhase] = useState(PHASE.TITLE)
  const [campaign, setCampaign] = useState(null)
  const [session, setSession] = useState(null)
  const [player, setPlayer] = useState(null)
  const [abilities, setAbilities] = useState([])
  const [items, setItems] = useState([])
  const [companions, setCompanions] = useState([])
  const [campaignTime, setCampaignTime] = useState(null)
  const [messages, setMessages] = useState([])
  const [loading, setLoading] = useState(false)
  const [gameState, setGameState] = useState('exploration')
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const [pendingRoll, setPendingRoll] = useState(null)
  const [error, setError] = useState(null)
  const [levelUpAvailable, setLevelUpAvailable] = useState(false)
  const [levelUpResult, setLevelUpResult] = useState(null)
  const [showLevelUp, setShowLevelUp] = useState(false)
  const [knownManeuvers, setKnownManeuvers] = useState([])

  // ── Resume campaign ─────────────────────────────────────────────────────────

  const handleResume = async (campaignId) => {
    setPhase(PHASE.RESUMING)
    setError(null)
    try {
      const campState = await api.getCampaignState(campaignId)
      if (!campState.campaign || !campState.player) throw new Error('Campaign not found')
      setCampaign(campState.campaign)
      setPlayer(campState.player)
      setCampaignTime(campState.time || null)

      let sess = campState.session
      if (!sess) {
        const newSess = await api.startSession(campaignId)
        sess = newSess.session
      }
      setSession(sess)

      const playerState = await api.getPlayerState(campaignId)
      if (playerState.abilities) setAbilities(playerState.abilities)
      if (playerState.items) setItems(playerState.items)
      if (playerState.companions) setCompanions(playerState.companions)
      if (playerState.known_maneuvers) setKnownManeuvers(playerState.known_maneuvers)

      if (sess?.id) {
        const msgData = await api.getSessionMessages(campaignId, sess.id)
        if (msgData.messages && msgData.messages.length > 0) {
          const restored = msgData.messages
            .filter(m => m.role === 'user' || m.role === 'assistant')
            .map((m, i) => ({ role: m.role === 'user' ? 'player' : 'dm', content: m.content, tools_used: [], id: i }))
          setMessages(restored)
        }
      }

      // Check if level up was pending
      checkLevelUp(campState.player)
      setPhase(PHASE.GAME)
    } catch (e) {
      setError('Could not resume that campaign.')
      setPhase(PHASE.TITLE)
    }
  }

  // ── Level up detection ──────────────────────────────────────────────────────

  const checkLevelUp = (p) => {
    if (!p) return
    const thresholds = [0,300,900,2700,6500,14000,23000,34000,48000,64000,85000,100000,120000,140000,165000,195000,225000,265000,305000,355000]
    const nextThreshold = thresholds[p.level] ?? Infinity
    if (p.experience >= nextThreshold && p.level < 20) {
      setLevelUpAvailable(true)
    } else {
      setLevelUpAvailable(false)
    }
  }

  // ── Campaign creation ───────────────────────────────────────────────────────

  const handleCharacterComplete = async (charData) => {
    setPhase(PHASE.LOADING)
    setError(null)
    try {
      const result = await api.createCampaign(charData)
      setCampaign(result.campaign)
      setPlayer(result.player)
      setSession(result.session)
      setPhase(PHASE.GAME)
      await refreshPlayerState(result.campaign.id)
      setLoading(true)
      await sendToBackend(
        result.campaign.id, result.session.id,
        'Begin the adventure. Open with a vivid, atmospheric scene that places me immediately in a specific moment.',
        'exploration', null, result.player
      )
    } catch (e) {
      setError(e.message)
      setPhase(PHASE.CREATION)
    }
  }

  // ── Player state refresh ────────────────────────────────────────────────────

  const refreshPlayerState = async (campaignId) => {
    try {
      const state = await api.getPlayerState(campaignId || campaign?.id)
      if (state.player) {
        setPlayer(state.player)
        checkLevelUp(state.player)
      }
      if (state.abilities) setAbilities(state.abilities)
      if (state.items) setItems(state.items)
      if (state.companions) setCompanions(state.companions)
      if (state.time) setCampaignTime(state.time)
      if (state.known_maneuvers) setKnownManeuvers(state.known_maneuvers)
    } catch (e) {
      console.error('Failed to refresh player state:', e)
    }
  }

  // ── Level up flow ───────────────────────────────────────────────────────────

  const handleLevelUpClick = async () => {
    // Fetch what the level up will grant before opening modal
    // We call the backend to get the level up result preview
    // by temporarily calling level_up — but we don't want to commit yet
    // Instead, calculate it on the frontend based on player data
    const nextLevel = (player?.level || 1) + 1
    const isFighter = player?.class === 'Fighter'

    // Build a preview result to pass to the modal
    const preview = buildLevelUpPreview(player, nextLevel)
    setLevelUpResult(preview)
    setShowLevelUp(true)
  }

  const buildLevelUpPreview = (p, newLevel) => {
    const conMod = Math.floor((p.con - 10) / 2)
    const hitDie = p.class === 'Fighter' ? 10 : p.class === 'Barbarian' ? 12 : 8
    const hpGained = Math.floor(hitDie / 2) + 1 + conMod
    const newMaxHp = p.max_hp + hpGained
    const newProf = newLevel <= 4 ? 2 : newLevel <= 8 ? 3 : newLevel <= 12 ? 4 : newLevel <= 16 ? 5 : 6

    const isFighter = p.class === 'Fighter'

    const secondWindUses = isFighter ? (newLevel >= 10 ? 4 : newLevel >= 4 ? 3 : 2) : 2
    const weaponMasteryCount = isFighter ? (newLevel >= 16 ? 6 : newLevel >= 4 ? 4 : 3) : 0
    const extraAttacks = isFighter ? (newLevel >= 20 ? 4 : newLevel >= 11 ? 3 : newLevel >= 5 ? 2 : 1) : 1
    const actionSurgeUses = isFighter ? (newLevel >= 17 ? 2 : newLevel >= 2 ? 1 : 0) : 0
    const indomitableMax = isFighter ? (newLevel >= 17 ? 3 : newLevel >= 13 ? 2 : newLevel >= 9 ? 1 : 0) : 0

    const asiLevels = isFighter ? [4,6,8,12,14,16] : [4,8,12,16,19]
    const asiAvailable = asiLevels.includes(newLevel)
    const subclassChoiceRequired = newLevel === 3 && !p.subclass

    const features = getFighterFeatures(p, newLevel)

    return {
      new_level: newLevel,
      hp_gained: hpGained,
      new_max_hp: newMaxHp,
      new_proficiency_bonus: newProf,
      asi_available: asiAvailable,
      subclass_choice_required: subclassChoiceRequired,
      new_features: features,
      second_wind_uses: secondWindUses,
      weapon_mastery_count: weaponMasteryCount,
      extra_attacks: extraAttacks,
      action_surge_uses: actionSurgeUses,
      indomitable_max: indomitableMax,
    }
  }

  const getFighterFeatures = (p, level) => {
    const base = {
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

    const subclassFeatures = {
      'Champion': { 3: ['Improved Critical', 'Remarkable Athlete'], 7: ['Additional Fighting Style'], 10: ['Heroic Warrior'], 15: ['Superior Critical'], 18: ['Survivor'] },
      'Battle Master': { 3: ['Combat Superiority', 'Student of War'], 7: ['Know Your Enemy'], 10: ['Improved Combat Superiority (d10)'], 15: ['Relentless', 'Improved Combat Superiority (d12)'], 18: ['Ultimate Combat Superiority'] },
      'Psi Warrior': { 3: ['Psionic Power'], 7: ['Telekinetic Adept'], 10: ['Guarded Mind'], 15: ['Bulwark of Force'], 18: ['Telekinetic Master'] },
    }

    const features = [...(base[level] || [])]
    if (p.subclass && subclassFeatures[p.subclass]?.[level]) {
      features.push(...subclassFeatures[p.subclass][level])
    }
    return features
  }

  const handleLevelUpComplete = async (choices) => {
    setShowLevelUp(false)
    setLevelUpAvailable(false)
    try {
      const result = await api.levelUp(campaign.id, choices)
      if (result.player) setPlayer(result.player)
      if (result.abilities) setAbilities(result.abilities)
      if (result.items) setItems(result.items)
      if (result.companions) setCompanions(result.companions)
      if (result.known_maneuvers) setKnownManeuvers(result.known_maneuvers)
      if (result.time) setCampaignTime(result.time)

      // Check if another level up is available
      checkLevelUp(result.player)

      // Add a DM message acknowledging the level up
      const newLevel = result.player?.level
      setMessages(m => [...m, {
        role: 'dm',
        content: `You have reached level ${newLevel}. Your capabilities grow — the path ahead demands ever greater strength.`,
        tools_used: [],
        id: Date.now()
      }])
    } catch (e) {
      console.error('Level up failed:', e)
    }
  }

  // ── Messaging ───────────────────────────────────────────────────────────────

  const handleSend = async (content) => {
    if (loading || !campaign || !session) return
    setMessages(m => [...m, { role: 'player', content, id: Date.now() }])
    setLoading(true)
    await sendToBackend(campaign.id, session.id, content, gameState, null, player)
  }

  const sendToBackend = async (campaignId, sessionId, content, gs, rollResult, currentPlayer) => {
    try {
      const result = await api.sendMessage({ campaignId, sessionId, content, gameState: gs, rollResult })

      if (result.type === 'roll_request') {
        if (result.opening_narrative && result.opening_narrative.trim()) {
          setMessages(m => [...m, { role: 'dm', content: result.opening_narrative, tools_used: [], id: Date.now() }])
        }
        setPendingRoll(result.roll)
        setLoading(false)
        return
      }

      if (result.type === 'narrative') {
        if (result.content && result.content.trim()) {
          setMessages(m => [...m, { role: 'dm', content: result.content, tools_used: result.tools_used || [], id: Date.now() }])
        }

        if (result.combat_turns && result.combat_turns.length > 0) {
          for (let i = 0; i < result.combat_turns.length; i++) {
            await new Promise(resolve => setTimeout(resolve, 2500))
            setMessages(m => [...m, { role: 'dm', content: result.combat_turns[i], tools_used: [], id: Date.now() + i + 1 }])
          }
        }

        if (result.player_downed) {
          setGameState('combat')
          await new Promise(resolve => setTimeout(resolve, 2500))
          setMessages(m => [...m, { role: 'dm', content: 'You are unconscious and dying. You must make death saving throws — roll a d20.', tools_used: [], id: Date.now() }])
          setPendingRoll({ tool_call_id: 'death_save', die: 'd20', skill: 'Death Save', dc: 10, reason: 'Roll a death saving throw. 10 or higher is a success. Three successes stabilize you. Three failures mean death.' })
          setLoading(false)
          return
        }

        if (result.new_state && result.new_state !== '') setGameState(result.new_state)
        await refreshPlayerState(campaignId)

        // Check for level up after XP award
        if (result.level_up_available) {
          setLevelUpAvailable(true)
        }
      }
    } catch (e) {
      setMessages(m => [...m, { role: 'dm', content: 'The ancient magics waver… (Connection error — is the backend running?)', tools_used: [], id: Date.now() }])
    }
    setLoading(false)
  }

  // ── Dice roll completion ────────────────────────────────────────────────────

  const handleRollComplete = async (result) => {
    if (!pendingRoll) return
    const roll = pendingRoll
    setPendingRoll(null)
    setLoading(true)
    const rollMsg = `[Rolled ${roll.skill}: ${roll.die} = ${result}${roll.dc ? ` vs DC ${roll.dc}` : ''}]`
    setMessages(m => [...m, { role: 'player', content: rollMsg, id: Date.now() }])
    await sendToBackend(campaign.id, session.id, `I rolled for ${roll.skill}.`, gameState, { die: roll.die, result, skill: roll.skill, dc: roll.dc }, player)
  }

  // ── Session management ──────────────────────────────────────────────────────

  const handleEndSession = async () => {
    if (!campaign || !session) return
    try { await api.endSession(campaign.id, session.id) }
    catch (e) { console.error('Failed to end session:', e) }
  }

  const handleNewAdventure = async () => {
    await handleEndSession()
    setCampaign(null); setSession(null); setPlayer(null)
    setAbilities([]); setItems([]); setCompanions([])
    setCampaignTime(null); setMessages([]); setPendingRoll(null)
    setGameState('exploration'); setLevelUpAvailable(false)
    setLevelUpResult(null); setShowLevelUp(false)
    setPhase(PHASE.TITLE)
  }

  // ── Render ──────────────────────────────────────────────────────────────────

  if (phase === PHASE.TITLE) return <TitleScreen onStart={() => setPhase(PHASE.CREATION)} onResume={handleResume} />

  if (phase === PHASE.CREATION) return (
    <>
      {error && (
        <div style={{ position: 'fixed', top: '1rem', left: '50%', transform: 'translateX(-50%)', background: '#9b2535', color: '#fff', padding: '.75rem 1.5rem', borderRadius: '3px', fontFamily: 'Cinzel, serif', fontSize: '.8rem', zIndex: 200, letterSpacing: '.1em' }}>
          {error}
        </div>
      )}
      <CharacterCreation onComplete={handleCharacterComplete} />
    </>
  )

  if (phase === PHASE.LOADING || phase === PHASE.RESUMING) return (
    <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', background: '#0b0c12', color: '#e8c46a', fontFamily: 'Cinzel, serif', letterSpacing: '.15em', fontSize: '.9rem' }}>
      <div style={{ marginBottom: '1.5rem', fontSize: '2rem' }}>⚔</div>
      <div>{phase === PHASE.RESUMING ? 'Resuming your adventure…' : 'Weaving your world…'}</div>
      <div style={{ marginTop: '.5rem', fontSize: '.72rem', color: '#6e7492' }}>
        {phase === PHASE.RESUMING ? 'Restoring your campaign' : 'Calling upon the ancient magics'}
      </div>
    </div>
  )

  return (
    <div style={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
      <Sidebar
        player={player}
        abilities={abilities}
        items={items}
        companions={companions}
        campaignTime={campaignTime}
        isOpen={sidebarOpen}
        onNewAdventure={handleNewAdventure}
        levelUpAvailable={levelUpAvailable}
        onLevelUp={handleLevelUpClick}
      />

      <GameScreen
        player={player}
        messages={messages}
        loading={loading}
        onSend={handleSend}
        gameState={gameState}
        onGameStateChange={setGameState}
        onToggleSidebar={() => setSidebarOpen(o => !o)}
        sidebarOpen={sidebarOpen}
      />

      {pendingRoll && (
        <DiceRollOverlay rollRequest={pendingRoll} onComplete={handleRollComplete} />
      )}

      {showLevelUp && levelUpResult && (
        <LevelUpModal
          player={{ ...player, known_maneuvers: knownManeuvers }}
          levelUpResult={levelUpResult}
          onComplete={handleLevelUpComplete}
          onClose={() => setShowLevelUp(false)}
        />
      )}
    </div>
  )
}