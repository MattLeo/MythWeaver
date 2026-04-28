const BASE = 'http://localhost:3000/api'

// ─── Campaign ─────────────────────────────────────────────────────────────────

export async function createCampaign(data) {
  const res = await fetch(`${BASE}/campaigns`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data)
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to create campaign (${res.status}): ${body}`)
  }
  return res.json()
}

export async function listCampaigns() {
  const res = await fetch(`${BASE}/campaigns`)
  if (!res.ok) throw new Error('Failed to list campaigns')
  return res.json()
}

export async function getCampaignState(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}`)
  if (!res.ok) throw new Error('Failed to get campaign state')
  return res.json()
}

export async function deleteCampaign(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}`, { method: 'DELETE' })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to delete campaign (${res.status}): ${body}`)
  }
  return res.json()
}

// ─── Player ───────────────────────────────────────────────────────────────────

export async function getPlayerState(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/player-state`)
  if (!res.ok) throw new Error('Failed to get player state')
  return res.json()
}

export async function levelUp(campaignId, choices) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/level-up`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(choices)
  })
  if (!res.ok) {
    const body = await res.text()
    console.error('Level up response body:', body)
    console.error('Level up status:', res.status)
    throw new Error(`Failed to level up (${res.status}): ${body}`)
  }
  return res.json()
}

// ─── Session ──────────────────────────────────────────────────────────────────

export async function startSession(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/session`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  })
  if (!res.ok) throw new Error('Failed to start session')
  return res.json()
}

export async function endSession(campaignId, sessionId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/sessions/${sessionId}/end`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  })
  if (!res.ok) throw new Error('Failed to end session')
  return res.json()
}

export async function getSessionMessages(campaignId, sessionId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/sessions/${sessionId}/messages`)
  if (!res.ok) throw new Error('Failed to get session messages')
  return res.json()
}

// ─── Messaging ────────────────────────────────────────────────────────────────

export async function sendMessage({ campaignId, sessionId, content, gameState, rollResult }) {
  const res = await fetch(`${BASE}/message`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      campaign_id: campaignId,
      session_id: sessionId,
      content,
      game_state: gameState,
      roll_result: rollResult,
    })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to send message (${res.status}): ${body}`)
  }
  return res.json()
}

// ─── Combat ───────────────────────────────────────────────────────────────────

export async function getCombatState(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat`)
  if (!res.ok) throw new Error('Failed to get combat state')
  return res.json()
}

export async function submitInitiative(campaignId, roll, advantageRolls = null) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat/initiative`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ roll, advantage_rolls: advantageRolls })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to submit initiative (${res.status}): ${body}`)
  }
  return res.json()
}

export async function setCombatTarget(campaignId, targetId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat/target`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ target_id: targetId })
  })
  if (!res.ok) throw new Error('Failed to set combat target')
  return res.json()
}

export async function resolveAttack(campaignId, targetId, roll, advantageRolls = null) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat/attack`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ target_id: targetId, roll, advantage_rolls: advantageRolls })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to resolve attack (${res.status}): ${body}`)
  }
  return res.json()
}

export async function resolveDamage(campaignId, rolls, isCrit = false) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat/damage`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ rolls, is_crit: isCrit })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to resolve damage (${res.status}): ${body}`)
  }
  return res.json()
}

export async function useCombatAbility(campaignId, abilityType, options = {}) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat/ability`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ability_type: abilityType,
      target_id: options.targetId ?? null,
      roll: options.roll ?? null,
      maneuver_name: options.maneuverName ?? null,
    })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to use combat ability (${res.status}): ${body}`)
  }
  return res.json()
}

export async function endCombatTurn(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat/end-turn`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to end combat turn (${res.status}): ${body}`)
  }
  return res.json()
}

export async function fleeCombat(campaignId, roll, skill) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat/flee`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ roll, skill })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to flee combat (${res.status}): ${body}`)
  }
  return res.json()
}

export async function endCombat(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat/end`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  })
  if (!res.ok) throw new Error('Failed to end combat')
  return res.json()
}