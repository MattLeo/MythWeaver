import { useState } from 'react'
import './index.css'
import * as api from './api/client.js'
import TitleScreen from './components/TitleScreen.jsx'
import CharacterCreation from './components/CharacterCreation.jsx'
import Sidebar from './components/Sidebar.jsx'
import GameScreen from './components/GameScreen.jsx'
import DiceRollOverlay from './components/DiceRollOverlay.jsx'
import LevelUpModal from './components/LevelUpModal.jsx'
import CombatModal from './components/CombatModal.jsx'
import ShopModal from './components/ShopModal.jsx'
import InventoryModal from './components/InventoryModal.jsx'
import AbilitiesModal from './components/AbilitiesModal.jsx'
import {
  isLevelUpAvailable,
  getFighterFeatures,
  hitDieForClass,
  proficiencyForLevel,
  FIGHTER_ASI_LEVELS,
} from './constants.js'

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
  const [showCombat, setShowCombat] = useState(false)
  const [combatInitiativeBonus, setCombatInitiativeBonus] = useState(0)
  const [showShop, setShowShop] = useState(false)
  const [showInventory, setShowInventory] = useState(false)
  const [showAbilities, setShowAbilities] = useState(false)


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

      setLevelUpAvailable(isLevelUpAvailable(campState.player))
      setPhase(PHASE.GAME)
    } catch (e) {
      setError('Could not resume that campaign.')
      setPhase(PHASE.TITLE)
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
        setLevelUpAvailable(isLevelUpAvailable(state.player))
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

  const handleLevelUpClick = () => {
    const preview = buildLevelUpPreview(player)
    setLevelUpResult(preview)
    setShowLevelUp(true)
  }

  const buildLevelUpPreview = (p) => {
    const newLevel = p.level + 1
    const conMod = Math.floor((p.con - 10) / 2)
    const hitDie = hitDieForClass(p.class)
    const hpGained = Math.floor(hitDie / 2) + 1 + conMod
    const newMaxHp = p.max_hp + hpGained
    const newProf = proficiencyForLevel(newLevel)
    const isFighter = p.class === 'Fighter'

    const secondWindUses = isFighter ? (newLevel >= 10 ? 4 : newLevel >= 4 ? 3 : 2) : 2
    const weaponMasteryCount = isFighter ? (newLevel >= 16 ? 6 : newLevel >= 4 ? 4 : 3) : 0
    const extraAttacks = isFighter ? (newLevel >= 20 ? 4 : newLevel >= 11 ? 3 : newLevel >= 5 ? 2 : 1) : 1
    const actionSurgeUses = isFighter ? (newLevel >= 17 ? 2 : newLevel >= 2 ? 1 : 0) : 0
    const indomitableMax = isFighter ? (newLevel >= 17 ? 3 : newLevel >= 13 ? 2 : newLevel >= 9 ? 1 : 0) : 0
    const asiAvailable = isFighter ? FIGHTER_ASI_LEVELS.includes(newLevel) : [4, 8, 12, 16, 19].includes(newLevel)
    const subclassChoiceRequired = newLevel === 3 && !p.subclass

    return {
      new_level: newLevel,
      hp_gained: hpGained,
      new_max_hp: newMaxHp,
      new_proficiency_bonus: newProf,
      asi_available: asiAvailable,
      subclass_choice_required: subclassChoiceRequired,
      new_features: getFighterFeatures(p, newLevel),
      second_wind_uses: secondWindUses,
      weapon_mastery_count: weaponMasteryCount,
      extra_attacks: extraAttacks,
      action_surge_uses: actionSurgeUses,
      indomitable_max: indomitableMax,
    }
  }

  const handleLevelUpComplete = async (choices) => {
    setShowLevelUp(false)
    setLevelUpAvailable(false)
    try {
      const result = await api.levelUp(campaign.id, choices)
      if (result.player) {
        setPlayer(result.player)
        setLevelUpAvailable(isLevelUpAvailable(result.player))
      }
      if (result.abilities) setAbilities(result.abilities)
      if (result.items) setItems(result.items)
      if (result.companions) setCompanions(result.companions)
      if (result.known_maneuvers) setKnownManeuvers(result.known_maneuvers)
      if (result.time) setCampaignTime(result.time)

      await refreshPlayerState(campaign.id)

      setMessages(m => [...m, {
        role: 'dm',
        content: `You have reached level ${result.player?.level}. Your capabilities grow — the path ahead demands ever greater strength.`,
        tools_used: [],
        id: Date.now()
      }])
    } catch (e) {
      console.error('Level up failed:', e)
      console.error('Error details:', e.message, e.stack)
    }
  }

  // ── Messaging ───────────────────────────────────────────────────────────────

  const handleSend = async (content) => {
    if (loading || !campaign || !session) return
    setMessages(m => [...m, { role: 'player', content, id: Date.now() }])
    setLoading(true)
    await sendToBackend(campaign.id, session.id, content, gameState, null, player)
  }

  const sendToBackend = async (campaignId, sessionId, content, gs, rollResult) => {
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

        if (result.needs_initiative) {
          const dexMod = Math.floor((player.dex - 10) / 2)
          setCombatInitiativeBonus(dexMod)
          setShowCombat(true)
          setGameState('combat')
        }

        if (result.needs_shop) {
          setShowShop(true)
          setGameState('shopping')
        }

        if (result.new_state && result.new_state !== '') setGameState(result.new_state)
        await refreshPlayerState(campaignId)

        if (result.level_up_available) setLevelUpAvailable(true)
      }
    } catch (e) {
      setMessages(m => [...m, {
        role: 'dm',
        content: 'The ancient magics waver… (Connection error — is the backend running?)',
        tools_used: [],
        id: Date.now()
      }])
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
    await sendToBackend(campaign.id, session.id, `I rolled for ${roll.skill}.`, gameState, { die: roll.die, result, skill: roll.skill, dc: roll.dc })
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

  // ── Combat Handler ──────────────────────────────────────────────────────────

  const handleCombatEnd = async (outcome, combatLog) => {
    setShowCombat(false)
    setGameState('exploration')

    const logSummary = combatLog.map(e => e.text).join('\n')
    if (outcome === 'victory' || outcome === 'fled') {
      setLoading(true)
      await sendToBackend(
        campaign.id, session.id,
        `[COMBAT RESOLVED — ${outcome.toUpperCase()}]\n\nCombat log:\n${logSummary}\n\nNarrate this combat cinematically in 2-3 paragraphs using the weapon names and actions from the log. Do not invent details not in the log. Do not address the player directly or give instructions.\n\nAfter narrating, call award_experience with an appropriate XP amount for the difficulty of this encounter. Then set state to exploration with [STATE:exploration].`,
        'exploration', null
      )
    }
    await refreshPlayerState(campaign.id)
  }

  // ── Shop Handler ───────────────────────────────────────────────────────────

  const handleShopClose = async (purchased, sold) => {
    setShowShop(false)
    setGameState('exploration')
    const parts = []
    if (purchased) parts.push(`purchased: ${purchased}`)
    if (sold) parts.push(`sold: ${sold}`)
    const summary = parts.length > 0 ? parts.join(' / ') : 'browsed but bought nothing'
    setLoading(true)
    await sendToBackend(
      campaign.id, session.id,
      `[SHOP CLOSED — ${summary}]\n\nBriefly narrate the player leaving the shop in one sentence. Then continue the story. [STATE:exploration]`,
      'exploration', null
    )
    await refreshPlayerState(campaign.id)
  }

  // ── Render ──────────────────────────────────────────────────────────────────


  if (phase === PHASE.TITLE) return (
    <TitleScreen onStart={() => setPhase(PHASE.CREATION)} onResume={handleResume} />
  )

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
        onInventory={() => setShowInventory(true)}
        onAbilities={() => setShowAbilities(true)}
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

      {showCombat && (
        <CombatModal
          campaignId={campaign.id}
          player={player}
          abilities={abilities}
          initiativeBonus={combatInitiativeBonus}
          hasAdvantage={false}
          onCombatEnd={handleCombatEnd}
          onPlayerUpdate={() => refreshPlayerState(campaign.id)}
        />
      )}

      {showShop && (
        <ShopModal
          campaignId={campaign.id}
          player={player}
          items={items}
          onShopClose={handleShopClose}
          onPlayerUpdate={() => refreshPlayerState(campaign.id)}
        />
      )}

      {showInventory && (
        <InventoryModal
          campaignId={campaign.id}
          player={player}
          items={items}
          onClose={() => setShowInventory(false)}
          onUpdate={() => refreshPlayerState(campaign.id)}
        />
      )}

      {showAbilities && (
        <AbilitiesModal
          player={player}
          abilities={abilities}
          onClose={() => setShowAbilities(false)}
        />
      )}

    </div>
  )
}