import { useState, useEffect } from 'react'
import './index.css'
import * as api from './api/client.js'
import TitleScreen from './components/TitleScreen.jsx'
import CharacterCreation from './components/CharacterCreation.jsx'
import Sidebar from './components/Sidebar.jsx'
import GameScreen from './components/GameScreen.jsx'
import DiceRollOverlay from './components/DiceRollOverlay.jsx'

const PHASE = {
  TITLE: 'title',
  CREATION: 'creation',
  LOADING: 'loading',
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
  const [pendingRoll, setPendingRoll] = useState(null)   // roll request from DM
  const [error, setError] = useState(null)

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
        result.campaign.id,
        result.session.id,
        `Begin the adventure. Open with a vivid, atmospheric scene that places me immediately in a specific moment.`,
        'exploration',
        null,
        result.player
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
      if (state.player) setPlayer(state.player)
      if (state.abilities) setAbilities(state.abilities)
      if (state.items) setItems(state.items)
      if (state.companions) setCompanions(state.companions)
      if (state.time) setCampaignTime(state.time)
    } catch (e) {
      console.error('Failed to refresh player state:', e)
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
      const result = await api.sendMessage({
        campaignId,
        sessionId,
        content,
        gameState: gs,
        rollResult
      })

      if (result.type === 'roll_request') {
        // DM wants a dice roll — show the overlay
        setPendingRoll(result.roll)
        setLoading(false)
        return
      }

      if (result.type === 'narrative') {
        setMessages(m => [...m, {
          role: 'dm',
          content: result.content,
          tools_used: result.tools_used || [],
          id: Date.now()
        }])

      // Setting new game state based on received narrative  
      if (result.new_state) {
        setGameState(result.new_state)
      }

        // Refresh player state after each DM response (HP, items, etc. may have changed)
        await refreshPlayerState(campaignId)
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

    // Add player message showing the roll
    const rollMsg = `[Rolled ${roll.skill}: ${roll.die} = ${result}${roll.dc ? ` vs DC ${roll.dc}` : ''}]`
    setMessages(m => [...m, { role: 'player', content: rollMsg, id: Date.now() }])

    // Send roll result back to backend to continue the tool loop
    await sendToBackend(
      campaign.id,
      session.id,
      `I rolled for ${roll.skill}.`,
      gameState,
      {
        die: roll.die,
        result,
        skill: roll.skill,
        dc: roll.dc
      },
      player
    )
  }

  // ── Session management ──────────────────────────────────────────────────────

  const handleEndSession = async () => {
    if (!campaign || !session) return
    try {
      await api.endSession(campaign.id, session.id)
    } catch (e) {
      console.error('Failed to end session:', e)
    }
  }

  const handleNewAdventure = async () => {
    await handleEndSession()
    setCampaign(null)
    setSession(null)
    setPlayer(null)
    setAbilities([])
    setItems([])
    setCompanions([])
    setCampaignTime(null)
    setMessages([])
    setPendingRoll(null)
    setGameState('exploration')
    setPhase(PHASE.TITLE)
  }

  // ── Render ──────────────────────────────────────────────────────────────────

  if (phase === PHASE.TITLE) {
    return <TitleScreen onStart={() => setPhase(PHASE.CREATION)} />
  }

  if (phase === PHASE.CREATION) {
    return (
      <>
        {error && (
          <div style={{
            position: 'fixed', top: '1rem', left: '50%', transform: 'translateX(-50%)',
            background: '#9b2535', color: '#fff', padding: '.75rem 1.5rem',
            borderRadius: '3px', fontFamily: 'Cinzel, serif', fontSize: '.8rem',
            zIndex: 200, letterSpacing: '.1em'
          }}>
            {error}
          </div>
        )}
        <CharacterCreation onComplete={handleCharacterComplete} />
      </>
    )
  }

  if (phase === PHASE.LOADING) {
    return (
      <div style={{
        minHeight: '100vh', display: 'flex', flexDirection: 'column',
        alignItems: 'center', justifyContent: 'center',
        background: '#0b0c12', color: '#e8c46a',
        fontFamily: 'Cinzel, serif', letterSpacing: '.15em', fontSize: '.9rem'
      }}>
        <div style={{ marginBottom: '1.5rem', fontSize: '2rem' }}>⚔</div>
        <div>Weaving your world…</div>
        <div style={{ marginTop: '.5rem', fontSize: '.72rem', color: '#6e7492' }}>
          Calling upon the ancient magics
        </div>
      </div>
    )
  }

  // GAME phase
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
        <DiceRollOverlay
          rollRequest={pendingRoll}
          onComplete={handleRollComplete}
        />
      )}
    </div>
  )
}