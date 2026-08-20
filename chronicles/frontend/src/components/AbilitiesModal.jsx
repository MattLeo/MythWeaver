import { useState } from 'react'
import { STYLES } from '../styles.js'

const ABILITY_STYLES = `
${STYLES}

.ab-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0,0,0,.85);
  display: flex; align-items: center; justify-content: center;
  backdrop-filter: blur(4px);
}

.ab-modal {
  width: 95vw; max-width: 1000px;
  height: 88vh; max-height: 780px;
  background: #0d0e18;
  border: 1px solid #2a2d44;
  border-radius: 4px;
  display: flex; flex-direction: column;
  overflow: hidden;
  box-shadow: 0 0 80px rgba(0,0,0,.9);
}

/* ── Header ── */
.ab-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: .65rem 1.2rem;
  background: #0b0c15;
  border-bottom: 1px solid #1e2035;
  flex-shrink: 0;
}

.ab-title {
  font-family: 'Cinzel', serif; font-size: .8rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--gold);
}

.ab-close-btn {
  background: none; border: 1px solid #2a2d44;
  color: #9098b8; font-family: 'Cinzel', serif;
  font-size: .62rem; letter-spacing: .1em;
  padding: .3rem .8rem; border-radius: 2px;
  cursor: pointer; transition: all .15s;
}
.ab-close-btn:hover { border-color: var(--red); color: #c43545; }

/* ── Body ── */
.ab-body {
  display: flex; flex: 1; min-height: 0;
}

/* ── Ability List ── */
.ab-list {
  flex: 1; min-width: 0; overflow-y: auto; padding: .75rem;
  display: flex; flex-direction: column; gap: .45rem;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #0b0c15;
}

.ab-card {
  background: #13151f; border: 1px solid #1e2035;
  border-radius: 3px; padding: .7rem .9rem;
  cursor: pointer; transition: all .15s;
  display: flex; align-items: center; gap: .8rem;
}

.ab-card:hover { border-color: rgba(200,150,42,.35); background: rgba(200,150,42,.03); }
.ab-card.selected { border-color: var(--gold); background: rgba(200,150,42,.07); box-shadow: 0 0 10px rgba(200,150,42,.12); }
.ab-card.depleted { border-color: #2a1a1a; opacity: .65; }
.ab-card.depleted:hover { border-color: rgba(180,50,50,.35); }
.ab-card.depleted.selected { border-color: var(--red); }
.ab-card.infinite { border-color: #1a2a3a; }

/* ── Pips / Uses ── */
.ab-uses-block {
  display: flex; flex-direction: column; align-items: center;
  gap: .3rem; flex-shrink: 0; min-width: 52px;
}

.ab-pips {
  display: flex; gap: .25rem; flex-wrap: wrap;
  justify-content: center; max-width: 52px;
}

.ab-pip {
  width: 9px; height: 9px; border-radius: 50%;
  border: 1px solid #3a3d55; background: #0b0c15;
  transition: all .3s;
}
.ab-pip.filled { background: var(--gold); border-color: var(--gold); box-shadow: 0 0 4px rgba(200,150,42,.4); }
.ab-pip.depleted-pip { border-color: #3a1a1a; }

.ab-uses-label {
  font-family: 'Cinzel', serif; font-size: .55rem;
  letter-spacing: .08em; text-transform: uppercase;
  color: #5a5d80;
}

.ab-infinity {
  font-size: 1.1rem; color: #5a7a9a;
  line-height: 1;
}

/* ── Card text ── */
.ab-card-text { flex: 1; min-width: 0; }

.ab-card-name {
  font-family: 'Cinzel', serif; font-size: .75rem;
  color: var(--goldl); letter-spacing: .05em;
  margin-bottom: .2rem;
}

.ab-card-preview {
  font-size: .63rem; color: #9098b8; line-height: 1.5;
  overflow: hidden; text-overflow: ellipsis;
  display: -webkit-box; -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.ab-card-refresh {
  font-family: 'Cinzel', serif; font-size: .55rem;
  letter-spacing: .1em; text-transform: uppercase;
  margin-top: .25rem;
}

.ab-card-refresh.short  { color: #5a8a6a; }
.ab-card-refresh.long   { color: #5a6a8a; }
.ab-card-refresh.manual { color: #5a5d80; }
.ab-card-refresh.turn   { color: #8a7a4a; }

.ab-empty {
  display: flex; align-items: center; justify-content: center;
  flex: 1; padding: 3rem;
  font-family: 'Cinzel', serif; font-size: .7rem;
  color: #3a3d55; letter-spacing: .1em; text-align: center;
}

/* ── Detail Panel ── */
.ab-detail {
  width: 280px; flex-shrink: 0;
  background: #0b0c15; border-left: 1px solid #1a1d2e;
  display: flex; flex-direction: column; overflow: hidden;
}

.ab-detail-empty {
  display: flex; align-items: center; justify-content: center;
  flex: 1; padding: 1.5rem;
  font-family: 'Cinzel', serif; font-size: .65rem;
  color: #3a3d55; text-align: center;
  line-height: 1.9; letter-spacing: .06em;
}

.ab-detail-header {
  padding: .9rem 1.1rem; border-bottom: 1px solid #1a1d2e; flex-shrink: 0;
}

.ab-detail-name {
  font-family: 'Cinzel', serif; font-size: .85rem;
  color: var(--goldl); letter-spacing: .06em; line-height: 1.4;
  margin-bottom: .3rem;
}

.ab-detail-meta {
  display: flex; gap: .6rem; align-items: center; flex-wrap: wrap;
}

.ab-detail-uses {
  font-family: 'Cinzel', serif; font-size: .62rem;
  color: var(--gold); letter-spacing: .06em;
}

.ab-detail-refresh {
  font-family: 'Cinzel', serif; font-size: .58rem;
  letter-spacing: .1em; text-transform: uppercase; padding: .1rem .35rem;
  border-radius: 2px; border: 1px solid;
}

.ab-detail-refresh.short  { color: #60a878; border-color: #1a4a2a; background: rgba(42,120,80,.08); }
.ab-detail-refresh.long   { color: #6080b8; border-color: #1a2a4a; background: rgba(42,80,150,.08); }
.ab-detail-refresh.manual { color: #7a7d9a; border-color: #2a2d44; background: rgba(60,64,100,.08); }
.ab-detail-refresh.turn   { color: #b09040; border-color: #3a2a10; background: rgba(140,110,42,.08); }

.ab-detail-body {
  flex: 1; overflow-y: auto; padding: .9rem 1.1rem;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #0b0c15;
}

.ab-detail-desc {
  font-size: .7rem; color: #c0c4d8; line-height: 1.9;
}

/* ── Uses visual in detail ── */
.ab-detail-uses-row {
  display: flex; align-items: center; gap: .5rem;
  margin-top: 1rem; padding-top: .8rem;
  border-top: 1px solid #1a1d2e;
}

.ab-detail-pips {
  display: flex; gap: .3rem; flex-wrap: wrap;
}

.ab-detail-pip {
  width: 12px; height: 12px; border-radius: 50%;
  border: 1px solid #3a3d55; background: #0b0c15;
  transition: all .3s;
}
.ab-detail-pip.filled { background: var(--gold); border-color: var(--gold); box-shadow: 0 0 5px rgba(200,150,42,.4); }
.ab-detail-pip.empty  { border-color: #2a1a1a; }

.ab-detail-uses-text {
  font-family: 'Cinzel', serif; font-size: .68rem;
  color: #9098b8;
}
`

// ─── Helpers ──────────────────────────────────────────────────────────────────

function refreshLabel(type) {
    const map = {
        short_rest: 'Short Rest',
        long_rest: 'Long Rest',
        per_turn: 'Per Turn',
        manual: 'Manual',
    }
    return map[type] || type
}

function refreshClass(type) {
    const map = {
        short_rest: 'short',
        long_rest: 'long',
        per_turn: 'turn',
        manual: 'manual',
    }
    return map[type] || 'manual'
}

function isInfinite(ab) {
    return ab.refresh_type === 'per_turn'
}

// ─── Main Component ───────────────────────────────────────────────────────────

export default function AbilitiesModal({ player, abilities, onClose }) {
    const [selected, setSelected] = useState(null)

    const allAbilities = abilities || []

    return (
        <>
            <style dangerouslySetInnerHTML={{ __html: ABILITY_STYLES }} />
            <div className="ab-overlay" onClick={e => e.target === e.currentTarget && onClose()}>
                <div className="ab-modal">

                    <div className="ab-header">
                        <div className="ab-title">✦ Abilities</div>
                        <button className="ab-close-btn" onClick={onClose}>Close</button>
                    </div>

                    <div className="ab-body">
                        <div className="ab-list">
                            {allAbilities.length === 0 && (
                                <div className="ab-empty">No abilities found</div>
                            )}
                            {allAbilities.map(ab => {
                                const infinite = isInfinite(ab)
                                const depleted = !infinite && ab.current_uses === 0
                                return (
                                    <div
                                        key={ab.id}
                                        className={[
                                            'ab-card',
                                            selected?.id === ab.id ? 'selected' : '',
                                            depleted ? 'depleted' : '',
                                            infinite ? 'infinite' : '',
                                        ].filter(Boolean).join(' ')}
                                        onClick={() => setSelected(ab)}
                                    >
                                        <div className="ab-uses-block">
                                            {infinite ? (
                                                <>
                                                    <div className="ab-infinity">∞</div>
                                                    <div className="ab-uses-label">At Will</div>
                                                </>
                                            ) : (
                                                <>
                                                    <div className="ab-pips">
                                                        {Array.from({ length: ab.max_uses }).map((_, i) => (
                                                            <div key={i} className={`ab-pip${i < ab.current_uses ? ' filled' : ' depleted-pip'}`} />
                                                        ))}
                                                    </div>
                                                    <div className="ab-uses-label">{ab.current_uses}/{ab.max_uses}</div>
                                                </>
                                            )}
                                        </div>
                                        <div className="ab-card-text">
                                            <div className="ab-card-name">{ab.name}</div>
                                            {ab.description && (
                                                <div className="ab-card-preview">{ab.description}</div>
                                            )}
                                            <div className={`ab-card-refresh ${refreshClass(ab.refresh_type)}`}>
                                                {refreshLabel(ab.refresh_type)}
                                            </div>
                                        </div>
                                    </div>
                                )
                            })}
                        </div>

                        <div className="ab-detail">
                            {!selected ? (
                                <div className="ab-detail-empty">
                                    Select an ability<br />to read its description
                                </div>
                            ) : (
                                <>
                                    <div className="ab-detail-header">
                                        <div className="ab-detail-name">{selected.name}</div>
                                        <div className="ab-detail-meta">
                                            {!isInfinite(selected) && (
                                                <div className="ab-detail-uses">
                                                    {selected.current_uses} / {selected.max_uses} uses
                                                </div>
                                            )}
                                            <div className={`ab-detail-refresh ${refreshClass(selected.refresh_type)}`}>
                                                {refreshLabel(selected.refresh_type)}
                                            </div>
                                        </div>
                                    </div>
                                    <div className="ab-detail-body">
                                        <div className="ab-detail-desc">
                                            {selected.description || 'No description available.'}
                                        </div>
                                        {!isInfinite(selected) && (
                                            <div className="ab-detail-uses-row">
                                                <div className="ab-detail-pips">
                                                    {Array.from({ length: selected.max_uses }).map((_, i) => (
                                                        <div key={i} className={`ab-detail-pip${i < selected.current_uses ? ' filled' : ' empty'}`} />
                                                    ))}
                                                </div>
                                                <div className="ab-detail-uses-text">
                                                    {selected.current_uses === 0
                                                        ? `Expended — restores on ${refreshLabel(selected.refresh_type).toLowerCase()}`
                                                        : `${selected.current_uses} remaining`}
                                                </div>
                                            </div>
                                        )}
                                    </div>
                                </>
                            )}
                        </div>
                    </div>

                </div>
            </div>
        </>
    )
}