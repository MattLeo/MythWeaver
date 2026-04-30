import { useState } from 'react'
import { STYLES } from '../styles.js'
import * as api from '../api/client.js'

const INV_STYLES = `
${STYLES}

.inv-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0,0,0,.85);
  display: flex; align-items: center; justify-content: center;
  backdrop-filter: blur(4px);
}

.inv-modal {
  width: 95vw; max-width: 1100px;
  height: 90vh; max-height: 820px;
  background: #0d0e18;
  border: 1px solid #2a2d44;
  border-radius: 4px;
  display: flex; flex-direction: column;
  overflow: hidden;
  box-shadow: 0 0 80px rgba(0,0,0,.9);
}

/* ── Header ── */
.inv-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: .65rem 1.2rem;
  background: #0b0c15;
  border-bottom: 1px solid #1e2035;
  flex-shrink: 0;
}

.inv-title {
  font-family: 'Cinzel', serif; font-size: .8rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--gold);
}

.inv-close-btn {
  background: none; border: 1px solid #2a2d44;
  color: var(--dim); font-family: 'Cinzel', serif;
  font-size: .62rem; letter-spacing: .1em;
  padding: .3rem .8rem; border-radius: 2px;
  cursor: pointer; transition: all .15s;
}
.inv-close-btn:hover { border-color: var(--red); color: var(--red); }

/* ── Tabs ── */
.inv-tabs {
  display: flex; background: #0b0c15;
  border-bottom: 1px solid #1a1d2e; flex-shrink: 0;
}

.inv-tab {
  background: none; border: none;
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .12em; text-transform: uppercase;
  color: var(--dim); padding: .6rem 1.4rem;
  cursor: pointer; transition: all .15s;
  border-bottom: 2px solid transparent; margin-bottom: -1px;
}
.inv-tab:hover { color: var(--goldl); }
.inv-tab.active { color: var(--goldl); border-bottom-color: var(--gold); background: rgba(200,150,42,.04); }

/* ── Body ── */
.inv-body {
  display: flex; flex: 1; min-height: 0;
}

/* ── Inventory Grid ── */
.inv-grid-panel {
  flex: 1; min-width: 0; overflow-y: auto;
  padding: .75rem; display: grid;
  grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
  gap: .55rem; align-content: start;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #0b0c15;
}

.inv-item-card {
  background: #13151f; border: 1px solid #1e2035;
  border-radius: 3px; padding: .65rem;
  cursor: pointer; transition: all .15s;
  display: flex; flex-direction: column; gap: .25rem;
  position: relative;
}

.inv-item-card:hover { border-color: rgba(200,150,42,.35); background: rgba(200,150,42,.03); }
.inv-item-card.selected { border-color: var(--gold); background: rgba(200,150,42,.07); box-shadow: 0 0 10px rgba(200,150,42,.12); }
.inv-item-card.equipped { border-color: #1a3a2a; background: rgba(50,120,70,.06); }
.inv-item-card.equipped.selected { border-color: var(--grn); box-shadow: 0 0 10px rgba(50,180,80,.12); }

.inv-equipped-badge {
  position: absolute; top: .4rem; right: .4rem;
  font-size: .5rem; font-family: 'Cinzel', serif;
  letter-spacing: .08em; color: #50a070;
  background: rgba(50,120,70,.2); border: 1px solid #1a4a2a;
  border-radius: 2px; padding: .05rem .3rem;
}

.inv-item-type {
  font-family: 'Cinzel', serif; font-size: .52rem;
  letter-spacing: .12em; text-transform: uppercase; color: var(--text);
}

.inv-item-name {
  font-family: 'Cinzel', serif; font-size: .72rem;
  color: var(--goldl); letter-spacing: .04em; line-height: 1.3;
  padding-right: 1.5rem;
}

.inv-item-desc {
  font-size: .62rem; color: var(--dim); line-height: 1.6; flex: 1;
}

.inv-item-stats {
  font-size: .6rem; color: var(--text); font-family: 'Cinzel', serif;
}

.inv-rarity-dot {
  position: absolute; bottom: .5rem; right: .5rem;
  width: 5px; height: 5px; border-radius: 50%;
}
.inv-rarity-dot.common    { background: #3a3d55; }
.inv-rarity-dot.uncommon  { background: #2d6b3a; }
.inv-rarity-dot.rare      { background: #2a4a8a; }
.inv-rarity-dot.very_rare { background: #6a2a8a; }
.inv-rarity-dot.legendary { background: #8a5a10; }

.inv-empty {
  grid-column: 1 / -1; display: flex; align-items: center;
  justify-content: center; padding: 3rem;
  font-family: 'Cinzel', serif; font-size: .7rem;
  color: var(--text); letter-spacing: .1em;
}

/* ── Equipment Screen ── */
.equip-panel {
  flex: 1; min-width: 0; overflow-y: auto; padding: 1rem;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #0b0c15;
}

.equip-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  grid-template-rows: auto;
  gap: .6rem;
  max-width: 700px; margin: 0 auto;
}

.equip-slot {
  background: #0f1020; border: 1px solid #1e2035;
  border-radius: 3px; padding: .6rem .8rem;
  min-height: 72px; cursor: pointer;
  transition: all .15s; display: flex; flex-direction: column; gap: .2rem;
}

.equip-slot:hover { border-color: rgba(200,150,42,.3); }
.equip-slot.filled { border-color: #1a3a2a; background: rgba(50,120,70,.05); }
.equip-slot.filled:hover { border-color: rgba(80,180,100,.4); }
.equip-slot.selected { border-color: var(--gold); background: rgba(200,150,42,.06); }

.equip-slot-label {
  font-family: 'Cinzel', serif; font-size: .52rem;
  letter-spacing: .14em; text-transform: uppercase; color: var(--text);
}

.equip-slot-name {
  font-family: 'Cinzel', serif; font-size: .7rem;
  color: var(--goldl); letter-spacing: .04em; line-height: 1.3;
}

.equip-slot-detail {
  font-size: .6rem; color: var(--text); font-family: 'Cinzel', serif;
}

.equip-slot-empty {
  font-family: 'Cinzel', serif; font-size: .6rem;
  color: var(--text); letter-spacing: .06em; margin-top: .2rem;
}

/* ── Detail Panel ── */
.inv-detail {
  width: 240px; flex-shrink: 0;
  background: #0b0c15; border-left: 1px solid #1a1d2e;
  display: flex; flex-direction: column; overflow: hidden;
}

.inv-detail-header {
  padding: .8rem 1rem; border-bottom: 1px solid #1a1d2e; flex-shrink: 0;
}

.inv-detail-name {
  font-family: 'Cinzel', serif; font-size: .78rem;
  color: var(--goldl); letter-spacing: .05em; line-height: 1.4;
  margin-bottom: .2rem;
}

.inv-detail-type {
  font-family: 'Cinzel', serif; font-size: .55rem;
  letter-spacing: .14em; text-transform: uppercase; color: var(--text);
}

.inv-detail-body {
  flex: 1; overflow-y: auto; padding: .8rem 1rem;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #0b0c15;
}

.inv-detail-desc {
  font-size: .67rem; color: var(--dim); line-height: 1.8; margin-bottom: .7rem;
}

.inv-stat-row {
  display: flex; justify-content: space-between; align-items: center;
  padding: .18rem 0; border-bottom: 1px solid #13151f;
}

.inv-stat-label { font-family: 'Cinzel', serif; font-size: .57rem; color: var(--dim); letter-spacing: .08em; }
.inv-stat-value { font-family: 'Cinzel', serif; font-size: .62rem; color: var(--text); }

.inv-detail-footer {
  padding: .75rem 1rem; border-top: 1px solid #1a1d2e;
  flex-shrink: 0; display: flex; flex-direction: column; gap: .4rem;
}

.inv-action-btn {
  background: #13151f; border: 1px solid #2a2d44;
  color: var(--dim); font-family: 'Cinzel', serif;
  font-size: .62rem; letter-spacing: .1em;
  padding: .4rem; border-radius: 2px; cursor: pointer;
  transition: all .15s; text-align: center;
}
.inv-action-btn:hover { border-color: var(--gold); color: var(--goldl); background: rgba(200,150,42,.05); }
.inv-action-btn:disabled { opacity: .3; cursor: not-allowed; }

.inv-action-btn.equip-btn { border-color: var(--gold); color: var(--gold); }
.inv-action-btn.equip-btn:hover { border-color: var(--goldl); color: var(--goldl); background: rgba(50,180,80,.06); }

.inv-action-btn.unequip-btn { border-color: #2a2a1a; color: #a09050; }
.inv-action-btn.unequip-btn:hover { border-color: var(--amber); color: var(--amber); background: rgba(180,140,50,.06); }

.inv-action-btn.delete-btn { border-color: #2a1a1a; color: #a05050; }
.inv-action-btn.delete-btn:hover { border-color: var(--red); color: #ff8080; background: rgba(180,50,50,.06); }

.inv-detail-empty {
  display: flex; align-items: center; justify-content: center;
  flex: 1; padding: 1rem; font-family: 'Cinzel', serif;
  font-size: .65rem; color: var(--text); text-align: center;
  line-height: 1.8; letter-spacing: .06em;
}

/* ── Slot picker overlay ── */
.slot-picker {
  position: absolute; inset: 0;
  background: rgba(10,11,20,.92);
  display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  gap: .5rem; z-index: 5; padding: 1rem;
}

.slot-picker-title {
  font-family: 'Cinzel', serif; font-size: .7rem;
  color: var(--goldl); letter-spacing: .12em;
  margin-bottom: .3rem;
}

.slot-picker-btn {
  background: #13151f; border: 1px solid #2a2d44;
  color: var(--dim); font-family: 'Cinzel', serif;
  font-size: .62rem; letter-spacing: .08em;
  padding: .35rem .9rem; border-radius: 2px;
  cursor: pointer; transition: all .15s; width: 160px; text-align: center;
}
.slot-picker-btn:hover { border-color: var(--gold); color: var(--goldl); }

.slot-picker-cancel {
  margin-top: .3rem; background: none; border: none;
  color: #3a3d55; font-family: 'Cinzel', serif; font-size: .58rem;
  cursor: pointer; letter-spacing: .1em;
}
.slot-picker-cancel:hover { color: var(--red); }

/* ── Confirm delete ── */
.confirm-delete {
  display: flex; gap: .4rem;
}

.inv-toast {
  position: absolute; bottom: 4rem; left: 50%;
  transform: translateX(-50%);
  background: #0f1020; border: 1px solid var(--gold);
  border-radius: 3px; padding: .35rem .9rem;
  font-family: 'Cinzel', serif; font-size: .63rem;
  color: var(--goldl); letter-spacing: .08em;
  animation: toast-in .2s ease; pointer-events: none;
  white-space: nowrap; z-index: 10;
}
.inv-toast.error { border-color: var(--red); color: #ff8080; }
.inv-toast.success { border-color: var(--grn); color: #60c080; }

@keyframes toast-in {
  from { opacity: 0; transform: translateX(-50%) translateY(6px); }
  to   { opacity: 1; transform: translateX(-50%) translateY(0); }
}
`

// ─── Constants ────────────────────────────────────────────────────────────────

const EQUIPMENT_SLOTS = [
  { id: 'helmet',    label: 'Helmet',     icon: '⛨' },
  { id: 'amulet',   label: 'Amulet',     icon: '◈' },
  { id: 'cloak',    label: 'Cloak',      icon: '◭' },
  { id: 'armor',    label: 'Armor',      icon: '⬡' },
  { id: 'ring_1',   label: 'Ring',       icon: '○' },
  { id: 'ring_2',   label: 'Ring',       icon: '○' },
  { id: 'main_hand',label: 'Main Hand',  icon: '⚔' },
  { id: 'shield',   label: 'Off Hand',   icon: '🛡' },
  { id: 'off_hand', label: 'Off Hand 2', icon: '⚔' },
  { id: 'boots',    label: 'Boots',      icon: '◬' },
]

const EQUIPPABLE_TYPES = ['weapon', 'armor', 'shield']

const SLOT_OPTIONS = {
  weapon: ['main_hand', 'off_hand'],
  armor:  ['armor'],
  shield: ['shield', 'off_hand'],
  wondrous: ['cloak', 'ring_1', 'ring_2', 'boots', 'helmet', 'amulet'],
}

function typeLabel(type) {
  const map = { weapon: 'Weapon', armor: 'Armor', shield: 'Shield',
    consumable: 'Consumable', wondrous: 'Wondrous', quest: 'Quest' }
  return map[type] || type
}

function itemStats(item) {
  const s = []
  if (item.damage_die)  s.push({ label: 'Damage', value: `${item.damage_die} ${item.damage_type || ''}` })
  if (item.base_ac)     s.push({ label: 'AC', value: String(item.base_ac) })
  if (item.armor_type)  s.push({ label: 'Type', value: item.armor_type })
  if (item.weapon_type) s.push({ label: 'Weapon', value: item.weapon_type })
  if (item.slot)        s.push({ label: 'Slot', value: item.slot.replace('_', ' ') })
  return s
}

function slotLabel(slot) {
  return slot?.replace('_', ' ') || ''
}

// ─── Main Component ───────────────────────────────────────────────────────────

export default function InventoryModal({ campaignId, player, items, onClose, onUpdate }) {
  const [tab, setTab] = useState('inventory')
  const [selected, setSelected] = useState(null)
  const [showSlotPicker, setShowSlotPicker] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState(false)
  const [loading, setLoading] = useState(false)
  const [toast, setToast] = useState(null)

  const showToast = (text, type = 'success') => {
    setToast({ text, type })
    setTimeout(() => setToast(null), 2000)
  }

  const allItems = items || []
  const inventory = allItems.filter(i => !i.is_equipped)
  const equipped  = allItems.filter(i => i.is_equipped)

  // ── Equip ──────────────────────────────────────────────────────────────────
  const handleEquip = async (slot) => {
    if (!selected || loading) return
    setShowSlotPicker(false)
    setLoading(true)
    try {
      await api.equipItem(campaignId, selected.id, slot)
      showToast(`${selected.name} equipped to ${slotLabel(slot)}`)
      setSelected(null)
      if (onUpdate) await onUpdate()
    } catch (e) {
      showToast('Failed to equip item', 'error')
    }
    setLoading(false)
  }

  const startEquip = () => {
    const slots = SLOT_OPTIONS[selected.item_type]
    if (!slots) return
    if (slots.length === 1) {
      handleEquip(slots[0])
    } else {
      setShowSlotPicker(true)
    }
  }

  // ── Unequip ────────────────────────────────────────────────────────────────
  const handleUnequip = async () => {
    if (!selected || loading) return
    setLoading(true)
    try {
      await api.unequipItem(campaignId, selected.id)
      showToast(`${selected.name} unequipped`)
      setSelected(null)
      if (onUpdate) await onUpdate()
    } catch (e) {
      showToast('Failed to unequip', 'error')
    }
    setLoading(false)
  }

  // ── Delete ─────────────────────────────────────────────────────────────────
  const handleDelete = async () => {
    if (!selected || loading) return
    setLoading(true)
    setConfirmDelete(false)
    try {
      await api.deleteItem(campaignId, selected.id)
      showToast(`${selected.name} discarded`, 'success')
      setSelected(null)
      if (onUpdate) await onUpdate()
    } catch (e) {
      showToast('Failed to discard', 'error')
    }
    setLoading(false)
  }

  // ── Equipment slot detail ──────────────────────────────────────────────────
  const handleSlotClick = (slotId) => {
    const item = equipped.find(i => i.slot === slotId)
    if (item) setSelected(item)
  }

  const canEquip = selected && !selected.is_equipped && SLOT_OPTIONS[selected.item_type]

  // ─────────────────────────────────────────────────────────────────────────
  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: INV_STYLES }} />
      <div className="inv-overlay" onClick={e => e.target === e.currentTarget && onClose()}>
        <div className="inv-modal" style={{ position: 'relative' }}>

          {/* Header */}
          <div className="inv-header">
            <div className="inv-title">⚔ Inventory</div>
            <button className="inv-close-btn" onClick={onClose}>Close</button>
          </div>

          {/* Tabs */}
          <div className="inv-tabs">
            <button
              className={`inv-tab${tab === 'inventory' ? ' active' : ''}`}
              onClick={() => { setTab('inventory'); setSelected(null); setConfirmDelete(false); setShowSlotPicker(false) }}
            >
              Inventory ({inventory.length})
            </button>
            <button
              className={`inv-tab${tab === 'equipment' ? ' active' : ''}`}
              onClick={() => { setTab('equipment'); setSelected(null); setConfirmDelete(false); setShowSlotPicker(false) }}
            >
              Equipment ({equipped.length})
            </button>
          </div>

          {/* Body */}
          <div className="inv-body">

            {/* ── Inventory Tab ── */}
            {tab === 'inventory' && (
              <div className="inv-grid-panel">
                {inventory.length === 0 && (
                  <div className="inv-empty">Your inventory is empty</div>
                )}
                {inventory.map(item => (
                  <div
                    key={item.id}
                    className={`inv-item-card${selected?.id === item.id ? ' selected' : ''}`}
                    onClick={() => { setSelected(item); setConfirmDelete(false); setShowSlotPicker(false) }}
                  >
                    <div className="inv-item-type">{typeLabel(item.item_type)}</div>
                    <div className="inv-item-name">{item.name}</div>
                    <div className="inv-item-desc">{item.description}</div>
                    {item.damage_die && (
                      <div className="inv-item-stats">{item.damage_die} {item.damage_type}</div>
                    )}
                    {item.base_ac && (
                      <div className="inv-item-stats">AC {item.base_ac}</div>
                    )}
                    {item.quantity > 1 && (
                      <div className="inv-item-stats">×{item.quantity}</div>
                    )}
                    <div className={`inv-rarity-dot ${item.rarity}`} />
                  </div>
                ))}
              </div>
            )}

            {/* ── Equipment Tab ── */}
            {tab === 'equipment' && (
              <div className="equip-panel">
                <div className="equip-grid">
                  {EQUIPMENT_SLOTS.map(slot => {
                    const item = equipped.find(i => i.slot === slot.id)
                    return (
                      <div
                        key={slot.id}
                        className={[
                          'equip-slot',
                          item ? 'filled' : '',
                          selected?.id === item?.id ? 'selected' : ''
                        ].filter(Boolean).join(' ')}
                        onClick={() => handleSlotClick(slot.id)}
                      >
                        <div className="equip-slot-label">{slot.icon} {slot.label}</div>
                        {item ? (
                          <>
                            <div className="equip-slot-name">{item.name}</div>
                            {item.damage_die && (
                              <div className="equip-slot-detail">{item.damage_die} {item.damage_type}</div>
                            )}
                            {item.base_ac && (
                              <div className="equip-slot-detail">AC {item.base_ac}</div>
                            )}
                          </>
                        ) : (
                          <div className="equip-slot-empty">— empty —</div>
                        )}
                      </div>
                    )
                  })}
                </div>
              </div>
            )}

            {/* ── Detail Panel ── */}
            <div className="inv-detail">
              {!selected && (
                <div className="inv-detail-empty">
                  {tab === 'inventory'
                    ? 'Select an item\nto see details'
                    : 'Click an equipment\nslot to inspect it'}
                </div>
              )}

              {selected && (
                <>
                  <div className="inv-detail-header">
                    <div className="inv-detail-name">{selected.name}</div>
                    <div className="inv-detail-type">
                      {typeLabel(selected.item_type)} · {selected.rarity}
                    </div>
                  </div>

                  <div className="inv-detail-body">
                    <div className="inv-detail-desc">{selected.description}</div>
                    {itemStats(selected).map(s => (
                      <div key={s.label} className="inv-stat-row">
                        <span className="inv-stat-label">{s.label}</span>
                        <span className="inv-stat-value">{s.value}</span>
                      </div>
                    ))}
                    {selected.notes && (
                      <div className="inv-detail-desc" style={{ marginTop: '.5rem', color: '#4a4d65' }}>
                        {selected.notes}
                      </div>
                    )}
                  </div>

                  <div className="inv-detail-footer">
                    {/* Equip button */}
                    {canEquip && !selected.is_equipped && (
                      <button
                        className="inv-action-btn equip-btn"
                        disabled={loading}
                        onClick={startEquip}
                      >
                        Equip
                      </button>
                    )}

                    {/* Unequip button */}
                    {selected.is_equipped && (
                      <button
                        className="inv-action-btn unequip-btn"
                        disabled={loading}
                        onClick={handleUnequip}
                      >
                        Unequip
                      </button>
                    )}

                    {/* Delete / Discard */}
                    {!confirmDelete ? (
                      <button
                        className="inv-action-btn delete-btn"
                        disabled={loading}
                        onClick={() => setConfirmDelete(true)}
                      >
                        Discard
                      </button>
                    ) : (
                      <div className="confirm-delete">
                        <button
                          className="inv-action-btn delete-btn"
                          style={{ flex: 1 }}
                          onClick={handleDelete}
                        >
                          Confirm
                        </button>
                        <button
                          className="inv-action-btn"
                          style={{ flex: 1 }}
                          onClick={() => setConfirmDelete(false)}
                        >
                          Cancel
                        </button>
                      </div>
                    )}
                  </div>
                </>
              )}
            </div>
          </div>

          {/* Slot Picker Overlay */}
          {showSlotPicker && selected && (
            <div className="slot-picker">
              <div className="slot-picker-title">Choose a slot</div>
              {(SLOT_OPTIONS[selected.item_type] || []).map(slot => (
                <button
                  key={slot}
                  className="slot-picker-btn"
                  onClick={() => handleEquip(slot)}
                >
                  {slotLabel(slot)}
                </button>
              ))}
              <button className="slot-picker-cancel" onClick={() => setShowSlotPicker(false)}>
                Cancel
              </button>
            </div>
          )}

          {/* Toast */}
          {toast && (
            <div className={`inv-toast ${toast.type}`}>{toast.text}</div>
          )}

        </div>
      </div>
    </>
  )
}