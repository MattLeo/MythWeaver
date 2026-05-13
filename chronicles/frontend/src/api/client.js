const BASE = 'http://localhost:3001/api'

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

export async function processInitialTurns(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/combat/process-start`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to process initial turns (${res.status}): ${body}`)
  }
  return res.json()
}

// ─── Shop ─────────────────────────────────────────────────────────────────────

export async function getShopState(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/shop`)
  if (!res.ok) throw new Error('Failed to get shop state')
  return res.json()
}

export async function buyItem(campaignId, shopItemId, quantity = 1) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/shop/buy`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ shop_item_id: shopItemId, quantity })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to buy item (${res.status}): ${body}`)
  }
  return res.json()
}

export async function sellItem(campaignId, playerItemId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/shop/sell`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ player_item_id: playerItemId })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to sell item (${res.status}): ${body}`)
  }
  return res.json()
}

export async function closeShop(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/shop/close`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  })
  if (!res.ok) throw new Error('Failed to close shop')
  return res.json()
}

// ─── Inventory ────────────────────────────────────────────────────────────────

export async function equipItem(campaignId, itemId, slot) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/inventory/equip`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ item_id: itemId, slot })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to equip item (${res.status}): ${body}`)
  }
  return res.json()
}

export async function unequipItem(campaignId, itemId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/inventory/unequip`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ item_id: itemId })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to unequip item (${res.status}): ${body}`)
  }
  return res.json()
}

export async function deleteItem(campaignId, itemId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/inventory/delete`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ item_id: itemId })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to delete item (${res.status}): ${body}`)
  }
  return res.json()
}

// ─── Spells ───────────────────────────────────────────────────────────────────

export async function getKnownSpells(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/spells`)
  if (!res.ok) throw new Error('Failed to get known spells')
  return res.json()
}

export async function getCastableSpells(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/spells/castable`)
  if (!res.ok) throw new Error('Failed to get castable spells')
  return res.json()
}

export async function getSpellSlots(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/spells/slots`)
  if (!res.ok) throw new Error('Failed to get spell slots')
  return res.json()
}

export async function seedEkSlots(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/spells/slots/seed`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to seed EK spell slots (${res.status}): ${body}`)
  }
  return res.json()
}

export async function learnSpell(campaignId, spellId, spellType = 'prepared') {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/spells/learn`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ spell_id: spellId, spell_type: spellType })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to learn spell (${res.status}): ${body}`)
  }
  return res.json()
}

export async function forgetSpell(campaignId, spellId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/spells/forget`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ spell_id: spellId })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to forget spell (${res.status}): ${body}`)
  }
  return res.json()
}

/**
 * Cast a spell.
 * @param {string} campaignId
 * @param {string} spellId
 * @param {object} options
 * @param {number|null} options.slotLevel - null for cantrips
 * @param {string|null} options.targetId
 * @param {boolean} options.dropConcentration - true to confirm dropping current concentration
 */
export async function castSpell(campaignId, spellId, options = {}) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/spells/cast`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      spell_id: spellId,
      slot_level: options.slotLevel ?? null,
      target_id: options.targetId ?? null,
      drop_concentration: options.dropConcentration ?? false,
    })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to cast spell (${res.status}): ${body}`)
  }
  return res.json()
}

export async function searchSpells(campaignId, query, wizardOnly = false) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/spells/search`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query, wizard_only: wizardOnly })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to search spells (${res.status}): ${body}`)
  }
  return res.json()
}

export async function applyBonusDamage(campaignId, damage) {
    const res = await fetch(`${BASE}/campaigns/${campaignId}/bonus-damage`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ damage }),
    })
    return res.json()
}

// ─── Concentration ────────────────────────────────────────────────────────────

export async function getConcentration(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/concentration`)
  if (!res.ok) throw new Error('Failed to get concentration')
  return res.json()
}

export async function dropConcentration(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/concentration/drop`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
  })
  if (!res.ok) throw new Error('Failed to drop concentration')
  return res.json()
}

// ─── War Bond ─────────────────────────────────────────────────────────────────

export async function getWarBonds(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/war-bonds`)
  if (!res.ok) throw new Error('Failed to get war bonds')
  return res.json()
}

export async function createWarBond(campaignId, itemId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/war-bonds/create`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ item_id: itemId })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to create war bond (${res.status}): ${body}`)
  }
  return res.json()
}

export async function breakWarBond(campaignId, itemId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/war-bonds/break`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ item_id: itemId })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to break war bond (${res.status}): ${body}`)
  }
  return res.json()
}

export async function summonBondedWeapon(campaignId, itemId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/war-bonds/summon`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ item_id: itemId })
  })
  if (!res.ok) {
    const body = await res.text()
    throw new Error(`Failed to summon bonded weapon (${res.status}): ${body}`)
  }
  return res.json()
}

// ─── Feats ───────────────────────────────────────────────────────────────────

export async function listFeats(category = null) {
    const url = category ? `/api/feats?category=${category}` : '/api/feats'
    const res = await fetch(url.startsWith('http') ? url : `${BASE.replace('/api', '')}${url}`)
    if (!res.ok) throw new Error('Failed to list feats')
    return res.json()
}
 
export async function getAvailableFeats(campaignId, category = null) {
    const url = category
        ? `${BASE}/campaigns/${campaignId}/feats?category=${category}`
        : `${BASE}/campaigns/${campaignId}/feats`
    const res = await fetch(url)
    if (!res.ok) throw new Error('Failed to get available feats')
    return res.json()
}
 
export async function getPlayerFeats(campaignId) {
    const res = await fetch(`${BASE}/campaigns/${campaignId}/player-feats`)
    if (!res.ok) throw new Error('Failed to get player feats')
    return res.json()
}

// ─── Notes ───────────────────────────────────────────────────────────────────

export async function getNotes(campaignId) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/notes`)
  if (!res.ok) throw new Error('Failed to get notes')
  return res.json()
}

export async function saveNotes(campaignId, notes) {
  const res = await fetch(`${BASE}/campaigns/${campaignId}/notes`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ notes })
  })
  if (!res.ok) throw new Error('Failed to save notes')
  return res.json()
}