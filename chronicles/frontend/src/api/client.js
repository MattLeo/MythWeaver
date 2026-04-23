const BASE = '/api'

export async function listCampaigns() {
  const res = await fetch(`${BASE}/campaigns`)
  if (!res.ok) throw new Error(`Failed to list campaigns`)
  return res.json()
}

export async function createCampaign(data) {
  const res = await fetch(`${BASE}/campaigns`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data)
  })
  if (!res.ok) throw new Error(`Failed to create campaign: ${res.statusText}`)
  return res.json()
}

export async function getCampaignState(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}`)
  if (!res.ok) throw new Error(`Failed to get campaign: ${res.statusText}`)
  return res.json()
}

export async function getPlayerState(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/player-state`)
  if (!res.ok) throw new Error(`Failed to get player state`)
  return res.json()
}

export async function getSessionMessages(campaignId, sessionId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/sessions/${sessionId}/messages`)
  if (!res.ok) throw new Error(`Failed to get session messages`)
  return res.json()
}

export async function startSession(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/session`, {
    method: 'POST'
  })
  if (!res.ok) throw new Error(`Failed to start session`)
  return res.json()
}

export async function endSession(campaignId, sessionId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/sessions/${sessionId}/end`, {
    method: 'POST'
  })
  if (!res.ok) throw new Error(`Failed to end session`)
  return res.json()
}

export async function sendMessage({ campaignId, sessionId, content, gameState, rollResult }) {
  const res = await fetch(`${BASE}/message`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      campaign_id: campaignId,
      session_id: sessionId,
      content,
      game_state: gameState || 'exploration',
      roll_result: rollResult || null
    })
  })
  if (!res.ok) throw new Error(`Failed to send message`)
  return res.json()
}