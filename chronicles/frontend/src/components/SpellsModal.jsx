import { useState, useEffect, useCallback } from 'react'
import {
  getKnownSpells, getSpellSlots, getConcentration,
  getWarBonds, castSpell, learnSpell, forgetSpell,
  searchSpells, dropConcentration, createWarBond,
  breakWarBond, summonBondedWeapon,
} from '../api/client'

const SCHOOL_COLORS = {
  abjuration:    '#7ec8e3',
  conjuration:   '#b5a9f5',
  divination:    '#f5e87e',
  enchantment:   '#f5a9c8',
  evocation:     '#f5a96a',
  illusion:      '#a9f5d0',
  necromancy:    '#b0f5a9',
  transmutation: '#f5cfa9',
}

const SCHOOL_GLYPHS = {
  abjuration:    '🛡',
  conjuration:   '✦',
  divination:    '👁',
  enchantment:   '♡',
  evocation:     '⚡',
  illusion:      '◈',
  necromancy:    '☽',
  transmutation: '⟳',
}

const DAMAGE_TYPE_COLORS = {
  fire:        '#f5764a',
  cold:        '#7ec8e3',
  lightning:   '#ffe066',
  acid:        '#a8e86e',
  poison:      '#8bcf6e',
  necrotic:    '#b0f5a9',
  radiant:     '#fff3a3',
  psychic:     '#f5a9c8',
  force:       '#c4a9f5',
  thunder:     '#a9c4f5',
  piercing:    '#d0c8b8',
  slashing:    '#d0c8b8',
  bludgeoning: '#d0c8b8',
}

function SlotPips({ current, max, level }) {
  return (
    <div style={styles.slotRow}>
      <span style={styles.slotLabel}>Lvl {level}</span>
      <div style={styles.pipRow}>
        {Array.from({ length: max }, (_, i) => (
          <div
            key={i}
            style={{
              ...styles.pip,
              background: i < current
                ? `hsl(${30 + level * 20}, 80%, 60%)`
                : 'rgba(255,255,255,0.08)',
              boxShadow: i < current
                ? `0 0 6px hsl(${30 + level * 20}, 80%, 50%)`
                : 'none',
            }}
          />
        ))}
      </div>
      <span style={styles.slotCount}>{current}/{max}</span>
    </div>
  )
}

function SpellCard({ spell, isSelected, onClick }) {
  const school = spell.school || 'evocation'
  const color = SCHOOL_COLORS[school] || '#c4a9f5'
  const glyph = SCHOOL_GLYPHS[school] || '✦'
  const isCantrip = spell.level === 0

  return (
    <div
      onClick={onClick}
      style={{
        ...styles.spellCard,
        borderColor: isSelected ? color : 'rgba(255,255,255,0.06)',
        background: isSelected
          ? `linear-gradient(135deg, rgba(${hexToRgb(color)}, 0.12), rgba(${hexToRgb(color)}, 0.04))`
          : 'rgba(255,255,255,0.02)',
        boxShadow: isSelected ? `0 0 0 1px ${color}40, inset 0 0 20px ${color}08` : 'none',
      }}
    >
      <div style={styles.spellCardTop}>
        <span style={{ ...styles.schoolGlyph, color }}>{glyph}</span>
        <span style={styles.spellCardName}>{spell.name}</span>
        {isCantrip
          ? <span style={{ ...styles.levelBadge, background: 'rgba(255,255,255,0.1)', color: '#aaa' }}>⊕</span>
          : <span style={{ ...styles.levelBadge, background: `${color}22`, color }}>{spell.level}</span>
        }
      </div>
      {spell.concentration === 1 && (
        <span style={styles.concBadge}>◉ conc</span>
      )}
    </div>
  )
}

function SpellDetail({ spell, slots, concentration, onCast, onForget, onLearn, mode, player }) {
  const [castLevel, setCastLevel] = useState(spell?.level || 1)
  const [casting, setCasting] = useState(false)

  useEffect(() => {
    if (spell) setCastLevel(Math.max(spell.level, 1))
  }, [spell?.spell_id])

  if (!spell) {
    return (
      <div style={styles.detailEmpty}>
        <div style={styles.detailEmptyGlyph}>✦</div>
        <p style={styles.detailEmptyText}>Select a spell to view details</p>
      </div>
    )
  }

  const school = spell.school || 'evocation'
  const color = SCHOOL_COLORS[school] || '#c4a9f5'
  const glyph = SCHOOL_GLYPHS[school] || '✦'
  const isCantrip = spell.level === 0

  // Available slots for casting
  const availableSlots = slots.filter(s => s.current_slots > 0 && s.slot_level >= spell.level)
  const canCast = isCantrip || availableSlots.length > 0

  // Check if concentrating on something else
  const concentratingOnOther = concentration && concentration.spell_id !== spell.spell_id
  const concentratingOnThis = concentration && concentration.spell_id === spell.spell_id

  // Upcast levels
  const maxSlotLevel = slots.length > 0 ? Math.max(...slots.map(s => s.slot_level)) : spell.level
  const castLevels = []
  for (let l = spell.level; l <= maxSlotLevel; l++) {
    const hasSlot = slots.some(s => s.slot_level === l && s.current_slots > 0)
    if (hasSlot || l === spell.level) castLevels.push(l)
  }

  const handleCast = async () => {
    setCasting(true)
    try {
      await onCast(spell, isCantrip ? null : castLevel)
    } finally {
      setCasting(false)
    }
  }

  const dmgColor = DAMAGE_TYPE_COLORS[spell.damage_type] || '#d0c8b8'

  return (
    <div style={styles.detailPanel}>
      {/* Header */}
      <div style={{ ...styles.detailHeader, borderColor: `${color}40` }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <span style={{ ...styles.detailGlyph, color }}>{glyph}</span>
          <div>
            <div style={styles.detailName}>{spell.name}</div>
            <div style={{ ...styles.detailMeta, color: `${color}cc` }}>
              {isCantrip ? 'Cantrip' : `Level ${spell.level}`}
              {' · '}
              {school.charAt(0).toUpperCase() + school.slice(1)}
            </div>
          </div>
        </div>
        {concentratingOnThis && (
          <div style={styles.concentratingBadge}>◉ Active</div>
        )}
      </div>

      {/* Stats row */}
      <div style={styles.detailStats}>
        <div style={styles.statChip}>
          <span style={styles.statLabel}>Cast</span>
          <span style={styles.statVal}>{formatCastingTime(spell.casting_time)}</span>
        </div>
        <div style={styles.statChip}>
          <span style={styles.statLabel}>Range</span>
          <span style={styles.statVal}>{formatRange(spell.range_type, spell.range_feet)}</span>
        </div>
        <div style={styles.statChip}>
          <span style={styles.statLabel}>Duration</span>
          <span style={styles.statVal}>{formatDuration(spell.duration)}</span>
        </div>
        {spell.concentration === 1 && (
          <div style={{ ...styles.statChip, borderColor: '#f5a96a44' }}>
            <span style={{ ...styles.statLabel, color: '#f5a96a' }}>◉ Conc</span>
          </div>
        )}
        {spell.ritual === 1 && (
          <div style={{ ...styles.statChip, borderColor: '#7ec8e344' }}>
            <span style={{ ...styles.statLabel, color: '#7ec8e3' }}>⊕ Ritual</span>
          </div>
        )}
      </div>

      {/* Components */}
      <div style={styles.componentsRow}>
        {spell.has_verbal === 1 && <span style={styles.component}>V</span>}
        {spell.has_somatic === 1 && <span style={styles.component}>S</span>}
        {spell.has_material === 1 && (
          <span style={{ ...styles.component, maxWidth: 200, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            M ({spell.material_component || '—'})
          </span>
        )}
      </div>

      {/* Damage info */}
      {spell.damage_die && (
        <div style={{ ...styles.damageRow, borderColor: `${dmgColor}33` }}>
          <span style={{ color: dmgColor, fontWeight: 700 }}>
            {getDamageString(spell, player?.level || 1)}
          </span>
          <span style={{ color: dmgColor, opacity: 0.7, textTransform: 'capitalize' }}>
            {' '}{spell.damage_type}
          </span>
          {spell.save_type && (
            <span style={styles.saveBadge}>{spell.save_type.toUpperCase()} save</span>
          )}
          {spell.attack_type && (
            <span style={styles.saveBadge}>{formatAttackType(spell.attack_type)}</span>
          )}
        </div>
      )}

      {/* Description */}
      <div style={styles.description}>{spell.description}</div>

      {/* Actions */}
      {mode === 'known' && (
        <div style={styles.actionArea}>
          {/* Slot level selector */}
          {!isCantrip && castLevels.length > 1 && (
            <div style={styles.slotSelector}>
              <span style={styles.slotSelectorLabel}>Cast at level:</span>
              <div style={styles.slotSelectorPills}>
                {castLevels.map(l => {
                  const hasSlot = slots.some(s => s.slot_level === l && s.current_slots > 0)
                  return (
                    <button
                      key={l}
                      onClick={() => setCastLevel(l)}
                      disabled={!hasSlot}
                      style={{
                        ...styles.slotPill,
                        background: castLevel === l ? `${color}33` : 'transparent',
                        borderColor: castLevel === l ? color : 'rgba(255,255,255,0.15)',
                        color: castLevel === l ? color : hasSlot ? '#ccc' : '#555',
                        cursor: hasSlot ? 'pointer' : 'not-allowed',
                      }}
                    >
                      {l}
                    </button>
                  )
                })}
              </div>
            </div>
          )}

          {/* Concentration warning */}
          {concentratingOnOther && spell.concentration === 1 && (
            <div style={styles.concWarning}>
              ⚠ Will drop concentration on <strong>{concentration.spell_name}</strong>
            </div>
          )}

          <div style={styles.actionRow}>
            <button
              onClick={handleCast}
              disabled={!canCast || casting}
              style={{
                ...styles.castBtn,
                background: canCast
                  ? `linear-gradient(135deg, ${color}33, ${color}18)`
                  : 'rgba(255,255,255,0.04)',
                borderColor: canCast ? color : 'rgba(255,255,255,0.1)',
                color: canCast ? color : '#555',
                cursor: canCast ? 'pointer' : 'not-allowed',
              }}
            >
              {casting ? '...' : isCantrip ? '✦ Cast Cantrip' : `✦ Cast (Slot ${castLevel})`}
            </button>
            <button
              onClick={() => onForget(spell)}
              style={styles.forgetBtn}
            >
              Forget
            </button>
          </div>
        </div>
      )}

      {mode === 'learn' && (
        <div style={styles.actionArea}>
          <button
            onClick={() => onLearn(spell)}
            style={{
              ...styles.castBtn,
              background: `linear-gradient(135deg, ${color}33, ${color}18)`,
              borderColor: color,
              color,
              cursor: 'pointer',
            }}
          >
            + Learn Spell
          </button>
        </div>
      )}
    </div>
  )
}

export default function SpellsModal({ campaignId, player, onClose, onCastInCombat }) {
  const [tab, setTab] = useState('known')        // 'known' | 'learn' | 'bonds'
  const [knownSpells, setKnownSpells] = useState([])
  const [slots, setSlots] = useState([])
  const [concentration, setConcentration] = useState(null)
  const [warBonds, setWarBonds] = useState([])
  const [selected, setSelected] = useState(null)
  const [filterLevel, setFilterLevel] = useState('all')
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState([])
  const [searching, setSearching] = useState(false)
  const [toast, setToast] = useState(null)
  const [loading, setLoading] = useState(true)

  // ── Load data ──────────────────────────────────────────────────────────────
  const loadAll = useCallback(async () => {
    try {
      const [knownRes, slotsRes, concRes, bondsRes] = await Promise.all([
        getKnownSpells(campaignId),
        getSpellSlots(campaignId),
        getConcentration(campaignId),
        getWarBonds(campaignId),
      ])
      setKnownSpells(knownRes.known_spells || [])
      setSlots(slotsRes.spell_slots || [])
      setConcentration(concRes.concentration || null)
      setWarBonds(bondsRes.war_bonds || [])
    } catch (e) {
      showToast('Failed to load spell data', 'error')
    } finally {
      setLoading(false)
    }
  }, [campaignId])

  useEffect(() => { loadAll() }, [loadAll])

  // ── Search ─────────────────────────────────────────────────────────────────
  useEffect(() => {
    if (tab !== 'learn' || searchQuery.trim().length < 2) {
      setSearchResults([])
      return
    }
    const timer = setTimeout(async () => {
      setSearching(true)
      try {
        const res = await searchSpells(campaignId, searchQuery, true)
        setSearchResults(res.spells || [])
      } catch (e) {
        showToast('Search failed', 'error')
      } finally {
        setSearching(false)
      }
    }, 300)
    return () => clearTimeout(timer)
  }, [searchQuery, tab, campaignId])

  // ── Helpers ────────────────────────────────────────────────────────────────
  const showToast = (msg, type = 'info') => {
    setToast({ msg, type })
    setTimeout(() => setToast(null), 3000)
  }

  // ── Actions ────────────────────────────────────────────────────────────────
  const handleCast = async (spell, slotLevel) => {
    try {
      const res = await castSpell(campaignId, spell.spell_id, {
        slotLevel,
        dropConcentration: false,
      })

      if (res.requires_confirmation) {
        const confirmed = window.confirm(res.message)
        if (confirmed) {
          const res2 = await castSpell(campaignId, spell.spell_id, {
            slotLevel,
            dropConcentration: true,
          })
          handleCastResult(res2, spell)
        }
        return
      }

      handleCastResult(res, spell)
    } catch (e) {
      showToast(e.message, 'error')
    }
  }

  const handleCastResult = (res, spell) => {
    showToast(res.message || `Cast ${spell.name}!`, 'success')
    setSlots(res.spell_slots || slots)
    setConcentration(res.concentration || null)
    if (onCastInCombat) {
      onCastInCombat({ spell, castResult: res })
    }
    loadAll()
  }

  const handleLearn = async (spell) => {
    try {
      const spellType = spell.level === 0 ? 'cantrip' : 'prepared'
      const res = await learnSpell(campaignId, spell.id || spell.spell_id, spellType)
      showToast(`Learned ${spell.name}!`, 'success')
      setKnownSpells(res.known_spells || knownSpells)
      setTab('known')
      setSelected(null)
      loadAll()
    } catch (e) {
      showToast(e.message, 'error')
    }
  }

  const handleForget = async (spell) => {
    if (!window.confirm(`Forget ${spell.name}? You can re-learn it during a long rest.`)) return
    try {
      const res = await forgetSpell(campaignId, spell.spell_id)
      showToast(`Forgot ${spell.name}`, 'info')
      setKnownSpells(res.known_spells || knownSpells)
      setSelected(null)
    } catch (e) {
      showToast(e.message, 'error')
    }
  }

  const handleDropConcentration = async () => {
    try {
      const res = await dropConcentration(campaignId)
      showToast(res.message, 'info')
      setConcentration(null)
    } catch (e) {
      showToast(e.message, 'error')
    }
  }

  const handleSummon = async (itemId) => {
    try {
      const res = await summonBondedWeapon(campaignId, itemId)
      showToast(res.message, 'success')
    } catch (e) {
      showToast(e.message, 'error')
    }
  }

  const handleBreakBond = async (itemId) => {
    if (!window.confirm('Break this War Bond?')) return
    try {
      const res = await breakWarBond(campaignId, itemId)
      showToast('War Bond broken', 'info')
      setWarBonds(res.war_bonds || [])
    } catch (e) {
      showToast(e.message, 'error')
    }
  }

  // ── Filter spells ──────────────────────────────────────────────────────────
  const cantrips = knownSpells.filter(s => s.level === 0)
  const prepared = knownSpells.filter(s => s.level > 0)
  const filteredPrepared = filterLevel === 'all'
    ? prepared
    : prepared.filter(s => s.level === parseInt(filterLevel))

  const displayList = tab === 'learn' ? searchResults : [...cantrips, ...filteredPrepared]

  // Spell slot totals
  const hasSlots = slots.length > 0
  const totalSlotsLeft = slots.reduce((a, s) => a + s.current_slots, 0)

  if (loading) {
    return (
      <div style={styles.overlay}>
        <div style={styles.modal}>
          <div style={{ color: '#888', textAlign: 'center', padding: 60 }}>Loading spells...</div>
        </div>
      </div>
    )
  }

  return (
    <div style={styles.overlay} onClick={e => e.target === e.currentTarget && onClose()}>
      <div style={styles.modal}>

        {/* ── Header ─────────────────────────────────────────────────────── */}
        <div style={styles.header}>
          <div style={styles.headerLeft}>
            <span style={styles.headerGlyph}>✦</span>
            <div>
              <div style={styles.headerTitle}>Arcane Arsenal</div>
              <div style={styles.headerSub}>Eldritch Knight · Spellbook</div>
            </div>
          </div>
          <button onClick={onClose} style={styles.closeBtn}>✕</button>
        </div>

        {/* ── Slot tracker ───────────────────────────────────────────────── */}
        {hasSlots && (
          <div style={styles.slotTracker}>
            <div style={styles.slotTrackerInner}>
              {slots.map(s => (
                <SlotPips key={s.slot_level} current={s.current_slots} max={s.max_slots} level={s.slot_level} />
              ))}
            </div>
            <div style={styles.slotTotal}>
              <span style={{ color: totalSlotsLeft > 0 ? '#f5cfa9' : '#666' }}>
                {totalSlotsLeft} slot{totalSlotsLeft !== 1 ? 's' : ''} remaining
              </span>
            </div>
          </div>
        )}

        {/* ── Concentration banner ────────────────────────────────────────── */}
        {concentration && (
          <div style={styles.concBanner}>
            <span>◉ Concentrating on <strong>{concentration.spell_name}</strong></span>
            <button onClick={handleDropConcentration} style={styles.concDropBtn}>Drop</button>
          </div>
        )}

        {/* ── Tabs ───────────────────────────────────────────────────────── */}
        <div style={styles.tabs}>
          {['known', 'learn', 'bonds'].map(t => (
            <button
              key={t}
              onClick={() => { setTab(t); setSelected(null) }}
              style={{
                ...styles.tab,
                borderBottom: tab === t ? '2px solid #f5cfa9' : '2px solid transparent',
                color: tab === t ? '#f5cfa9' : '#888',
              }}
            >
              {t === 'known' && `Known (${knownSpells.length})`}
              {t === 'learn' && 'Learn New'}
              {t === 'bonds' && `War Bonds (${warBonds.length}/2)`}
            </button>
          ))}
        </div>

        {/* ── Body ───────────────────────────────────────────────────────── */}
        <div style={styles.body}>

          {/* War Bonds tab */}
          {tab === 'bonds' ? (
            <div style={styles.bondsTab}>
              <p style={styles.bondsDesc}>
                Bond with up to 2 weapons. Bonded weapons can't be disarmed and can be summoned
                to your hand as a Bonus Action from anywhere.
              </p>
              {warBonds.length === 0 ? (
                <div style={styles.bondsEmpty}>
                  No bonded weapons. Open your Inventory and equip a weapon, then bond it here.
                </div>
              ) : (
                <div style={styles.bondsList}>
                  {warBonds.map(bond => (
                    <div key={bond.id} style={styles.bondCard}>
                      <div style={styles.bondName}>⚔ {bond.item_name}</div>
                      <div style={styles.bondActions}>
                        <button onClick={() => handleSummon(bond.item_id)} style={styles.summonBtn}>
                          ↑ Summon
                        </button>
                        <button onClick={() => handleBreakBond(bond.item_id)} style={styles.breakBondBtn}>
                          Break
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
              {warBonds.length < 2 && (
                <div style={styles.bondHint}>
                  To create a new War Bond, select a weapon from your inventory.
                </div>
              )}
            </div>
          ) : (
            /* ── Two-column spell layout ─────────────────────────────────── */
            <div style={styles.columns}>

              {/* Left: spell list */}
              <div style={styles.listCol}>
                {/* Filter / search */}
                {tab === 'learn' ? (
                  <div style={styles.searchBar}>
                    <span style={styles.searchIcon}>🔍</span>
                    <input
                      autoFocus
                      value={searchQuery}
                      onChange={e => setSearchQuery(e.target.value)}
                      placeholder="Search wizard spells..."
                      style={styles.searchInput}
                    />
                    {searching && <span style={{ color: '#888', fontSize: 12 }}>...</span>}
                  </div>
                ) : (
                  <div style={styles.filterRow}>
                    {['all', '1', '2', '3', '4'].map(l => (
                      <button
                        key={l}
                        onClick={() => setFilterLevel(l)}
                        style={{
                          ...styles.filterPill,
                          background: filterLevel === l ? 'rgba(245,207,169,0.15)' : 'transparent',
                          borderColor: filterLevel === l ? '#f5cfa9' : 'rgba(255,255,255,0.1)',
                          color: filterLevel === l ? '#f5cfa9' : '#888',
                        }}
                      >
                        {l === 'all' ? 'All' : `L${l}`}
                      </button>
                    ))}
                  </div>
                )}

                {/* Section: Cantrips */}
                {tab === 'known' && cantrips.length > 0 && (
                  <div style={styles.section}>
                    <div style={styles.sectionHeader}>Cantrips</div>
                    {cantrips.map(s => (
                      <SpellCard
                        key={s.spell_id}
                        spell={s}
                        isSelected={selected?.spell_id === s.spell_id}
                        onClick={() => setSelected(s)}
                      />
                    ))}
                  </div>
                )}

                {/* Section: Prepared spells */}
                {tab === 'known' && filteredPrepared.length > 0 && (
                  <div style={styles.section}>
                    <div style={styles.sectionHeader}>
                      Prepared Spells
                      <span style={styles.sectionCount}>{filteredPrepared.length}</span>
                    </div>
                    {filteredPrepared.map(s => (
                      <SpellCard
                        key={s.spell_id}
                        spell={s}
                        isSelected={selected?.spell_id === s.spell_id}
                        onClick={() => setSelected(s)}
                      />
                    ))}
                  </div>
                )}

                {/* Learn: search results */}
                {tab === 'learn' && (
                  <div style={styles.section}>
                    {searchQuery.trim().length < 2 && (
                      <div style={styles.searchHint}>
                        Type at least 2 characters to search.<br/>
                        As an Eldritch Knight, you may learn spells from the wizard list,
                        prioritizing Abjuration and Evocation.
                      </div>
                    )}
                    {searchResults.map(s => {
                      const alreadyKnown = knownSpells.some(k => k.spell_id === s.id)
                      return (
                        <div key={s.id} style={{ opacity: alreadyKnown ? 0.4 : 1, pointerEvents: alreadyKnown ? 'none' : 'auto' }}>
                          <SpellCard
                            spell={{ ...s, spell_id: s.id }}
                            isSelected={selected?.id === s.id || selected?.spell_id === s.id}
                            onClick={() => setSelected({ ...s, spell_id: s.id })}
                          />
                          {alreadyKnown && <div style={styles.alreadyKnown}>Already known</div>}
                        </div>
                      )
                    })}
                    {searchQuery.length >= 2 && !searching && searchResults.length === 0 && (
                      <div style={styles.searchHint}>No spells found for "{searchQuery}"</div>
                    )}
                  </div>
                )}

                {/* Empty state */}
                {tab === 'known' && knownSpells.length === 0 && (
                  <div style={styles.emptyState}>
                    <div style={{ fontSize: 32 }}>✦</div>
                    <div>No spells known yet.</div>
                    <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>
                      Use the Learn New tab to add spells.
                    </div>
                  </div>
                )}
              </div>

              {/* Right: detail panel */}
              <div style={styles.detailCol}>
                <SpellDetail
                  spell={selected}
                  slots={slots}
                  concentration={concentration}
                  onCast={handleCast}
                  onForget={handleForget}
                  onLearn={handleLearn}
                  mode={tab === 'learn' ? 'learn' : 'known'}
                  player={player}
                />
              </div>
            </div>
          )}
        </div>

        {/* ── Toast ──────────────────────────────────────────────────────── */}
        {toast && (
          <div style={{
            ...styles.toast,
            background: toast.type === 'error' ? '#5a1a1a' : toast.type === 'success' ? '#1a3a2a' : '#2a2a3a',
            borderColor: toast.type === 'error' ? '#f57e7e' : toast.type === 'success' ? '#7ef5a9' : '#a9a9f5',
          }}>
            {toast.msg}
          </div>
        )}
      </div>
    </div>
  )
}

// ─── Format helpers ───────────────────────────────────────────────────────────

function formatCastingTime(ct) {
  if (!ct) return '—'
  return ct
    .replace('bonus_action', 'Bonus Action')
    .replace('reaction', 'Reaction')
    .replace('action', 'Action')
    .replace('1_minute', '1 Min')
    .replace('10_minutes', '10 Min')
    .replace('12_hours', '12 Hr')
    .replace('1_hour', '1 Hr')
    .replace('24_hours', '24 Hr')
    .replace(/_/g, ' ')
}

function formatRange(rangeType, rangeFeet) {
  if (rangeType === 'self') return 'Self'
  if (rangeType === 'touch') return 'Touch'
  if (rangeType === 'special') return 'Special'
  if (rangeFeet) return `${rangeFeet} ft`
  return rangeType || '—'
}

function formatDuration(dur) {
  if (!dur) return '—'
  return dur
    .replace('concentration_1_minute', '1 Min ◉')
    .replace('concentration_10_minutes', '10 Min ◉')
    .replace('concentration_1_hour', '1 Hr ◉')
    .replace('concentration_2_hours', '2 Hr ◉')
    .replace('concentration_1_day', '1 Day ◉')
    .replace('concentration_6_rounds', '6 Rnd ◉')
    .replace('instantaneous', 'Instant')
    .replace('until_dispelled', '∞')
    .replace('until_dispelled_or_triggered', '∞/Trigger')
    .replace('1_minute', '1 Min')
    .replace('10_minutes', '10 Min')
    .replace('1_hour', '1 Hr')
    .replace('8_hours', '8 Hr')
    .replace('24_hours', '24 Hr')
    .replace('10_days', '10 Days')
    .replace('30_days', '30 Days')
    .replace('1_round', '1 Round')
    .replace(/_/g, ' ')
}

function formatAttackType(at) {
  if (!at) return ''
  if (at === 'ranged_spell') return 'Ranged Spell Attack'
  if (at === 'melee_spell') return 'Melee Spell Attack'
  return at
}

function getDamageString(spell, charLevel) {
  if (!spell.damage_die || !spell.damage_die_count) return null
  let count = spell.damage_die_count
  if (spell.level === 0) {
    if (charLevel >= 17 && spell.cantrip_dice_17) count = spell.cantrip_dice_17
    else if (charLevel >= 11 && spell.cantrip_dice_11) count = spell.cantrip_dice_11
    else if (charLevel >= 5 && spell.cantrip_dice_5) count = spell.cantrip_dice_5
  }
  return `${count}${spell.damage_die}`
}

function hexToRgb(hex) {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return `${r},${g},${b}`
}

// ─── Styles ───────────────────────────────────────────────────────────────────

const styles = {
  overlay: {
    position: 'fixed', inset: 0,
    background: 'rgba(0,0,0,0.75)',
    backdropFilter: 'blur(4px)',
    display: 'flex', alignItems: 'center', justifyContent: 'center',
    zIndex: 1000,
  },
  modal: {
    background: '#0e0e12',
    border: '1px solid rgba(255,255,255,0.08)',
    borderRadius: 12,
    width: '92vw', maxWidth: 960,
    height: '88vh',
    display: 'flex', flexDirection: 'column',
    overflow: 'hidden',
    boxShadow: '0 24px 80px rgba(0,0,0,0.8)',
  },
  header: {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    padding: '16px 20px',
    borderBottom: '1px solid rgba(255,255,255,0.06)',
    background: 'rgba(255,255,255,0.02)',
  },
  headerLeft: { display: 'flex', alignItems: 'center', gap: 12 },
  headerGlyph: { fontSize: 24, color: '#f5cfa9' },
  headerTitle: { fontSize: 18, fontWeight: 700, color: '#f0ead6', letterSpacing: '0.02em' },
  headerSub: { fontSize: 12, color: '#888', marginTop: 2 },
  closeBtn: {
    background: 'none', border: 'none', color: '#666',
    fontSize: 18, cursor: 'pointer', padding: '4px 8px',
    borderRadius: 4,
    transition: 'color 0.15s',
  },
  slotTracker: {
    padding: '10px 20px',
    borderBottom: '1px solid rgba(255,255,255,0.04)',
    background: 'rgba(0,0,0,0.2)',
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
  },
  slotTrackerInner: { display: 'flex', gap: 20, flexWrap: 'wrap' },
  slotRow: { display: 'flex', alignItems: 'center', gap: 8 },
  slotLabel: { fontSize: 11, color: '#666', width: 32, textAlign: 'right' },
  pipRow: { display: 'flex', gap: 4 },
  pip: {
    width: 10, height: 10, borderRadius: '50%',
    transition: 'background 0.2s, box-shadow 0.2s',
  },
  slotCount: { fontSize: 11, color: '#888', width: 26 },
  slotTotal: { fontSize: 12, color: '#888' },
  concBanner: {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    padding: '8px 20px',
    background: 'rgba(245,169,106,0.08)',
    borderBottom: '1px solid rgba(245,169,106,0.2)',
    fontSize: 13, color: '#f5a96a',
  },
  concDropBtn: {
    background: 'rgba(245,169,106,0.15)', border: '1px solid rgba(245,169,106,0.3)',
    color: '#f5a96a', fontSize: 11, padding: '3px 10px', borderRadius: 4,
    cursor: 'pointer',
  },
  tabs: {
    display: 'flex',
    borderBottom: '1px solid rgba(255,255,255,0.06)',
    padding: '0 20px',
  },
  tab: {
    background: 'none', border: 'none', borderBottom: '2px solid transparent',
    padding: '10px 16px', fontSize: 13, cursor: 'pointer',
    transition: 'color 0.15s, border-color 0.15s',
    marginBottom: -1,
  },
  body: { flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' },
  columns: { display: 'flex', flex: 1, overflow: 'hidden' },
  listCol: {
    width: 280, flexShrink: 0,
    borderRight: '1px solid rgba(255,255,255,0.06)',
    overflow: 'hidden auto',
    display: 'flex', flexDirection: 'column',
    padding: '12px 0',
  },
  detailCol: {
    flex: 1, overflow: 'hidden auto',
    padding: '0 4px',
  },
  filterRow: {
    display: 'flex', gap: 6, padding: '0 12px 10px',
    flexWrap: 'wrap',
  },
  filterPill: {
    padding: '4px 12px', borderRadius: 20,
    border: '1px solid', fontSize: 12,
    cursor: 'pointer', transition: 'all 0.15s',
  },
  searchBar: {
    display: 'flex', alignItems: 'center', gap: 8,
    margin: '0 12px 10px',
    background: 'rgba(255,255,255,0.04)',
    border: '1px solid rgba(255,255,255,0.08)',
    borderRadius: 8, padding: '6px 12px',
  },
  searchIcon: { fontSize: 14, opacity: 0.5 },
  searchInput: {
    background: 'none', border: 'none', outline: 'none',
    color: '#e0e0e0', fontSize: 13, flex: 1,
  },
  section: { padding: '0 12px' },
  sectionHeader: {
    fontSize: 10, fontWeight: 700, letterSpacing: '0.1em',
    color: '#555', textTransform: 'uppercase',
    padding: '8px 4px 6px',
    display: 'flex', alignItems: 'center', gap: 8,
  },
  sectionCount: {
    background: 'rgba(255,255,255,0.06)',
    color: '#888', fontSize: 10,
    padding: '1px 6px', borderRadius: 10,
  },
  spellCard: {
    padding: '8px 10px', marginBottom: 4, borderRadius: 8,
    border: '1px solid',
    cursor: 'pointer',
    transition: 'all 0.15s',
  },
  spellCardTop: { display: 'flex', alignItems: 'center', gap: 8 },
  schoolGlyph: { fontSize: 14, width: 18, textAlign: 'center', flexShrink: 0 },
  spellCardName: { flex: 1, fontSize: 13, color: '#d0c8b8', lineHeight: 1.2 },
  levelBadge: {
    fontSize: 11, fontWeight: 700,
    padding: '1px 6px', borderRadius: 8,
    flexShrink: 0,
  },
  concBadge: { fontSize: 10, color: '#f5a96a', marginTop: 2, paddingLeft: 26 },
  emptyState: {
    display: 'flex', flexDirection: 'column', alignItems: 'center',
    gap: 8, padding: '40px 20px',
    color: '#555', fontSize: 13, textAlign: 'center',
  },
  searchHint: {
    padding: '20px 12px', color: '#555', fontSize: 12,
    textAlign: 'center', lineHeight: 1.6,
  },
  alreadyKnown: { fontSize: 10, color: '#555', textAlign: 'center', marginBottom: 4, marginTop: -2 },

  // Detail panel
  detailPanel: {
    padding: '20px 24px',
    display: 'flex', flexDirection: 'column', gap: 14,
    height: '100%',
    overflowY: 'auto',
  },
  detailEmpty: {
    display: 'flex', flexDirection: 'column', alignItems: 'center',
    justifyContent: 'center', height: '100%', gap: 12,
  },
  detailEmptyGlyph: { fontSize: 48, color: '#2a2a3a' },
  detailEmptyText: { color: '#444', fontSize: 14 },
  detailHeader: {
    display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between',
    paddingBottom: 14, borderBottom: '1px solid',
  },
  detailGlyph: { fontSize: 28, lineHeight: 1 },
  detailName: { fontSize: 22, fontWeight: 700, color: '#f0ead6', letterSpacing: '-0.01em' },
  detailMeta: { fontSize: 13, marginTop: 3 },
  concentratingBadge: {
    background: 'rgba(245,169,106,0.15)', border: '1px solid rgba(245,169,106,0.3)',
    color: '#f5a96a', fontSize: 12, padding: '4px 10px', borderRadius: 6,
  },
  detailStats: { display: 'flex', gap: 8, flexWrap: 'wrap' },
  statChip: {
    display: 'flex', flexDirection: 'column', gap: 2,
    background: 'rgba(255,255,255,0.04)',
    border: '1px solid rgba(255,255,255,0.07)',
    borderRadius: 8, padding: '6px 12px',
  },
  statLabel: { fontSize: 10, color: '#666', letterSpacing: '0.06em', textTransform: 'uppercase' },
  statVal: { fontSize: 13, color: '#d0c8b8' },
  componentsRow: { display: 'flex', gap: 8, flexWrap: 'wrap' },
  component: {
    fontSize: 12, padding: '3px 10px',
    background: 'rgba(255,255,255,0.04)',
    border: '1px solid rgba(255,255,255,0.08)',
    borderRadius: 6, color: '#aaa',
  },
  damageRow: {
    display: 'flex', alignItems: 'center', gap: 10,
    padding: '8px 12px',
    background: 'rgba(255,255,255,0.03)',
    border: '1px solid',
    borderRadius: 8,
    fontSize: 15,
  },
  saveBadge: {
    fontSize: 11, padding: '2px 8px',
    background: 'rgba(255,255,255,0.06)',
    borderRadius: 4, color: '#999',
    marginLeft: 'auto',
  },
  description: {
    fontSize: 13, color: '#9098b8', lineHeight: 1.65,
    flex: 1,
  },
  actionArea: { display: 'flex', flexDirection: 'column', gap: 10, paddingTop: 4 },
  slotSelector: { display: 'flex', alignItems: 'center', gap: 10 },
  slotSelectorLabel: { fontSize: 12, color: '#888', flexShrink: 0 },
  slotSelectorPills: { display: 'flex', gap: 6 },
  slotPill: {
    width: 32, height: 32, borderRadius: 8,
    border: '1px solid', fontSize: 13, fontWeight: 700,
    transition: 'all 0.15s',
  },
  concWarning: {
    fontSize: 12, color: '#f5a96a',
    background: 'rgba(245,169,106,0.08)',
    border: '1px solid rgba(245,169,106,0.2)',
    borderRadius: 6, padding: '6px 12px',
  },
  actionRow: { display: 'flex', gap: 10 },
  castBtn: {
    flex: 1, padding: '12px 20px',
    border: '1px solid', borderRadius: 10,
    fontSize: 14, fontWeight: 700,
    letterSpacing: '0.03em',
    transition: 'all 0.15s',
  },
  forgetBtn: {
    padding: '12px 16px',
    background: 'rgba(255,80,80,0.08)',
    border: '1px solid rgba(255,80,80,0.2)',
    color: '#f57e7e', borderRadius: 10,
    fontSize: 13, cursor: 'pointer',
    transition: 'all 0.15s',
  },

  // War Bonds tab
  bondsTab: {
    padding: 24, display: 'flex', flexDirection: 'column', gap: 16,
    overflow: 'auto', flex: 1,
  },
  bondsDesc: { fontSize: 13, color: '#888', lineHeight: 1.6, margin: 0 },
  bondsEmpty: {
    padding: '20px 0', color: '#555', fontSize: 13, textAlign: 'center',
  },
  bondsList: { display: 'flex', flexDirection: 'column', gap: 10 },
  bondCard: {
    display: 'flex', alignItems: 'center', justifyContent: 'space-between',
    padding: '12px 16px',
    background: 'rgba(255,255,255,0.03)',
    border: '1px solid rgba(255,255,255,0.08)',
    borderRadius: 10,
  },
  bondName: { fontSize: 15, color: '#d0c8b8', fontWeight: 600 },
  bondActions: { display: 'flex', gap: 8 },
  summonBtn: {
    padding: '6px 14px',
    background: 'rgba(245,207,169,0.12)',
    border: '1px solid rgba(245,207,169,0.3)',
    color: '#f5cfa9', borderRadius: 6, fontSize: 13, cursor: 'pointer',
  },
  breakBondBtn: {
    padding: '6px 12px',
    background: 'rgba(255,80,80,0.08)',
    border: '1px solid rgba(255,80,80,0.2)',
    color: '#f57e7e', borderRadius: 6, fontSize: 12, cursor: 'pointer',
  },
  bondHint: { fontSize: 12, color: '#555', textAlign: 'center' },

  // Toast
  toast: {
    position: 'absolute', bottom: 16, left: '50%',
    transform: 'translateX(-50%)',
    border: '1px solid', borderRadius: 8,
    padding: '10px 20px', fontSize: 13, color: '#e0e0e0',
    pointerEvents: 'none',
    animation: 'fadeIn 0.2s ease',
    whiteSpace: 'nowrap',
  },
}