import { useState } from 'react'
import { STYLES } from '../styles.js'

const STAT_LABELS = ['STR', 'DEX', 'CON', 'INT', 'WIS', 'CHA']
const STAT_KEYS = ['str', 'dex', 'con', 'int', 'wis', 'cha']
const DICE = [4, 6, 8, 10, 12, 20]

const mod = (v) => Math.floor((v - 10) / 2)
const fmt = (v) => { const m = mod(v); return (m >= 0 ? '+' : '') + m }
const d = (n) => Math.floor(Math.random() * n) + 1

const SIDEBAR_STYLES = `
${STYLES}
.sidebar {
  width: 260px; min-width: 260px;
  background: var(--surf); border-right: 1px solid var(--bord);
  overflow-y: auto; padding: 1.1rem;
  display: flex; flex-direction: column; gap: .85rem;
}
.cn-name { font-family: 'Cinzel', serif; font-size: 1rem; color: var(--goldl); }
.cn-sub { font-size: .75rem; color: var(--dim); margin-top: .1rem; }
.hp-bar { background: var(--bord); border-radius: 1px; height: 5px; margin: .4rem 0 .2rem; }
.hp-fill { height: 100%; border-radius: 1px; transition: width .6s, background .6s; }
.hp-txt { font-size: .8rem; }
.hp-adj { display: flex; align-items: center; gap: .4rem; margin-top: .4rem; }
.hp-adj button {
  background: var(--elev); border: 1px solid var(--bord);
  color: var(--text); width: 26px; height: 26px;
  border-radius: 2px; cursor: pointer; font-size: .9rem;
  display: flex; align-items: center; justify-content: center;
}
.hp-adj button:hover { border-color: var(--gold); color: var(--gold); }
.hp-adj input {
  background: var(--elev); border: 1px solid var(--bord);
  color: var(--text); width: 42px; text-align: center;
  border-radius: 2px; padding: .2rem; font-size: .78rem;
  font-family: 'Cinzel', serif;
}
.hp-adj input:focus { outline: none; border-color: var(--gold); }
.inv-i {
  font-size: .76rem; color: var(--dim); padding: .18rem 0;
  border-bottom: 1px solid var(--bord);
}
.inv-i:last-child { border-bottom: none; }
.inv-i.equipped { color: var(--goldl); }
.gp { font-size: .82rem; color: var(--gold); margin-top: .4rem; }
.dice-row { display: flex; flex-wrap: wrap; gap: .4rem; margin: .4rem 0; }
.roll-res { font-family: 'Cinzel', serif; font-size: .8rem; color: var(--goldl); margin-top: .3rem; }
.roll-hint { font-size: .68rem; color: var(--dim); margin-top: .35rem; line-height: 1.5; }
.ability-row {
  display: flex; justify-content: space-between; align-items: center;
  font-size: .75rem; padding: .18rem 0; border-bottom: 1px solid var(--bord);
}
.ability-row:last-child { border-bottom: none; }
.ability-name { color: var(--dim); flex: 1; }
.ability-uses { font-family: 'Cinzel', serif; color: var(--gold); font-size: .7rem; }
.ability-uses.empty { color: var(--red); }
.time-display {
  font-family: 'Cinzel', serif; font-size: .72rem;
  color: var(--goldl); text-transform: uppercase; letter-spacing: .1em;
}
.death-saves { display: flex; gap: 1rem; margin-top: .4rem; }
.ds-group { display: flex; flex-direction: column; gap: .2rem; }
.ds-label { font-size: .6rem; color: var(--dim); font-family: 'Cinzel', serif; letter-spacing: .1em; }
.ds-dots { display: flex; gap: .25rem; }
.ds-dot {
  width: 10px; height: 10px; border-radius: 50%;
  border: 1px solid var(--bord); background: var(--elev);
}
.ds-dot.success-filled { background: var(--grn); border-color: var(--grn); }
.ds-dot.failure-filled { background: var(--red); border-color: var(--red); }
.xp-bar { background: var(--bord); border-radius: 1px; height: 3px; margin: .3rem 0 .15rem; }
.xp-fill { height: 100%; border-radius: 1px; background: var(--gold); transition: width .8s; }
.xp-txt { font-size: .68rem; color: var(--dim); }

@media(max-width:700px) {
  .sidebar {
    position: fixed; left: 0; top: 0; bottom: 0; z-index: 10;
    transform: translateX(-100%); transition: transform .25s;
  }
  .sidebar.open { transform: translateX(0); }
}
`

export default function Sidebar({ player, abilities, items, companions, campaignTime, isOpen, onNewAdventure }) {
  const [hpDelta, setHpDelta] = useState('1')
  const [lastRoll, setLastRoll] = useState(null)

  if (!player) return null

  const hpPct = player.max_hp > 0 ? Math.max(0, (player.current_hp / player.max_hp) * 100) : 0
  const hpCol = hpPct > 55 ? 'var(--grn)' : hpPct > 25 ? 'var(--amber)' : 'var(--red)'

  const equipped = items?.filter(i => i.is_equipped) || []
  const inventory = items?.filter(i => !i.is_equipped) || []

  // XP progress
  const xpThresholds = [0,300,900,2700,6500,14000,23000,34000,48000,64000,85000,100000,120000,140000,165000,195000,225000,265000,305000,355000]
  const currentThreshold = xpThresholds[player.level - 1] || 0
  const nextThreshold = xpThresholds[player.level] || xpThresholds[xpThresholds.length - 1]
  const xpPct = ((player.experience - currentThreshold) / (nextThreshold - currentThreshold)) * 100

  const rollDie = (n) => setLastRoll(`d${n}: ${d(n)}`)

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: SIDEBAR_STYLES }} />
      <div className={`sidebar${isOpen ? ' open' : ''}`}>

        {/* Identity */}
        <div>
          <div className="cn-name">{player.name}</div>
          <div className="cn-sub">Level {player.level} {player.race} {player.class}</div>
          <div className="cn-sub">{player.background}</div>
        </div>

        {/* Time */}
        {campaignTime && (
          <div className="time-display">
            ☽ {campaignTime.time_of_day} · Day {campaignTime.current_day} · {campaignTime.season}
          </div>
        )}

        {/* HP */}
        <div className="sec">
          <div className="sec-title">Hit Points</div>
          <div className="hp-bar">
            <div className="hp-fill" style={{ width: `${hpPct}%`, background: hpCol }} />
          </div>
          <div className="hp-txt" style={{ color: hpCol }}>
            {player.current_hp} / {player.max_hp}
            {player.temp_hp > 0 && <span style={{ color: 'var(--goldl)' }}> (+{player.temp_hp} temp)</span>}
          </div>

          {/* Death saves — shown when player is at 0 HP */}
          {player.current_hp === 0 && !player.is_dead && (
            <div className="death-saves">
              <div className="ds-group">
                <div className="ds-label">Successes</div>
                <div className="ds-dots">
                  {[0,1,2].map(i => (
                    <div key={i} className={`ds-dot${i < player.death_save_successes ? ' success-filled' : ''}`} />
                  ))}
                </div>
              </div>
              <div className="ds-group">
                <div className="ds-label">Failures</div>
                <div className="ds-dots">
                  {[0,1,2].map(i => (
                    <div key={i} className={`ds-dot${i < player.death_save_failures ? ' failure-filled' : ''}`} />
                  ))}
                </div>
              </div>
            </div>
          )}
          {player.is_dead && (
            <div style={{ color: 'var(--red)', fontSize: '.75rem', marginTop: '.4rem', fontFamily: 'Cinzel, serif' }}>
              ✝ Fallen
            </div>
          )}
        </div>

        {/* XP */}
        <div className="sec">
          <div className="sec-title">Experience</div>
          <div className="xp-bar">
            <div className="xp-fill" style={{ width: `${Math.min(100, xpPct)}%` }} />
          </div>
          <div className="xp-txt">{player.experience.toLocaleString()} XP · Next: {nextThreshold.toLocaleString()}</div>
        </div>

        {/* Stats */}
        <div className="sec">
          <div className="sec-title">Abilities · AC {player.armor_class} · Prof +{player.proficiency_bonus}</div>
          {STAT_KEYS.map((key, i) => (
            <div key={key} className="stat-r">
              <span className="sr-l">{STAT_LABELS[i]}</span>
              <span className="sr-v">{player[key]}</span>
              <span className="sr-m">{fmt(player[key])}</span>
            </div>
          ))}
        </div>

        {/* Abilities */}
        {abilities && abilities.length > 0 && (
          <div className="sec">
            <div className="sec-title">Class Abilities</div>
            {abilities.map(ab => (
              <div key={ab.id} className="ability-row">
                <span className="ability-name">{ab.name}</span>
                <span className={`ability-uses${ab.current_uses === 0 ? ' empty' : ''}`}>
                  {ab.refresh_type === 'per_turn' ? '∞' : `${ab.current_uses}/${ab.max_uses}`}
                </span>
              </div>
            ))}
          </div>
        )}

        {/* Equipped */}
        {equipped.length > 0 && (
          <div className="sec">
            <div className="sec-title">Equipped</div>
            {equipped.map(item => (
              <div key={item.id} className="inv-i equipped">
                [{item.slot?.replace('_', ' ')}] {item.name}
              </div>
            ))}
          </div>
        )}

        {/* Inventory */}
        <div className="sec">
          <div className="sec-title">Inventory</div>
          {inventory.length === 0
            ? <div className="inv-i">Empty</div>
            : inventory.map(item => (
              <div key={item.id} className="inv-i">
                {item.quantity > 1 ? `${item.name} ×${item.quantity}` : item.name}
              </div>
            ))
          }
          <div className="gp">⊙ {player.gold} gp</div>
        </div>

        {/* Active companions */}
        {companions && companions.length > 0 && (
          <div className="sec">
            <div className="sec-title">Companions</div>
            {companions.map(c => (
              <div key={c.id} className="ability-row">
                <span className="ability-name">{c.name}</span>
                <span className="ability-uses" style={{ color: c.current_hp > 0 ? 'var(--grn)' : 'var(--red)' }}>
                  {c.current_hp}/{c.max_hp}
                </span>
              </div>
            ))}
          </div>
        )}

        {/* Dice roller */}
        <div className="sec">
          <div className="sec-title">Dice Roller</div>
          <div className="dice-row">
            {DICE.map(n => (
              <button key={n} className="btn-sm" onClick={() => rollDie(n)}>d{n}</button>
            ))}
          </div>
          {lastRoll && <div className="roll-res">🎲 {lastRoll}</div>}
          <p className="roll-hint">Roll here for reference — the DM will request rolls automatically.</p>
        </div>

        {/* New adventure */}
        <div style={{ marginTop: 'auto', paddingTop: '.75rem' }}>
          <button className="btn-ghost" style={{ width: '100%', fontSize: '.72rem' }} onClick={onNewAdventure}>
            ✦ New Adventure
          </button>
        </div>
      </div>
    </>
  )
}