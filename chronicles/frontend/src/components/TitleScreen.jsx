import { useState, useEffect } from 'react'
import { STYLES } from '../styles.js'
import * as api from '../api/client.js'

const TITLE_STYLES = `
${STYLES}
.title {
  min-height: 100vh; display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  text-align: center; padding: 2rem;
  background: radial-gradient(ellipse at 50% 25%, #17062a 0%, #0b0c12 65%);
}
.title h1 {
  font-family: 'Cinzel', serif;
  font-size: clamp(2rem, 6vw, 4.8rem);
  color: var(--goldl); letter-spacing: .12em;
  text-shadow: 0 0 60px rgba(232,196,106,.35); line-height: 1.15;
}
.title .sub {
  color: var(--dim); max-width: 480px;
  margin: 1.2rem auto 2.5rem;
  font-style: italic; line-height: 1.85; font-size: .95rem;
}
.title .sig {
  margin-top: 1.5rem; font-size: .75rem; color: var(--dim);
  font-family: 'Cinzel', serif; letter-spacing: .15em;
}
.ornament { font-size: 2rem; margin-bottom: 1rem; opacity: .7; }
.section-label {
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--dim); margin-bottom: .6rem; text-align: left;
}
.campaign-list {
  width: 100%; max-width: 520px; margin: 0 auto 1.5rem;
  display: flex; flex-direction: column; gap: .6rem;
}
.campaign-card {
  background: var(--surf); border: 1px solid var(--bord);
  border-radius: 3px; padding: .9rem 1.2rem;
  cursor: pointer; text-align: left; width: 100%;
  transition: border-color .2s, background .2s;
  display: flex; justify-content: space-between; align-items: center;
}
.campaign-card:hover {
  border-color: var(--gold);
  background: rgba(200,150,42,.06);
}
.campaign-card-name {
  font-family: 'Cinzel', serif; font-size: .85rem;
  color: var(--goldl); letter-spacing: .08em; margin-bottom: .2rem;
}
.campaign-card-sub {
  font-size: .75rem; color: var(--dim); font-style: italic;
}
.campaign-card-badge {
  font-family: 'Cinzel', serif; font-size: .6rem;
  letter-spacing: .1em; text-transform: uppercase;
  color: var(--gold); border: 1px solid var(--gold);
  border-radius: 2px; padding: .15rem .45rem; opacity: .7;
  white-space: nowrap; margin-left: 1rem;
}
.divider {
  width: 100%; max-width: 520px; border: none;
  border-top: 1px solid var(--bord); margin: 1.2rem auto;
}
.btn-ghost {
  background: transparent; border: 1px solid var(--bord);
  color: var(--dim); cursor: pointer;
  font-family: 'Cinzel', serif; font-size: .72rem;
  font-weight: 700; letter-spacing: .12em; text-transform: uppercase;
  padding: .65rem 1.8rem; border-radius: 2px;
  transition: all .2s;
}
.btn-ghost:hover { border-color: var(--gold); color: var(--goldl); }
.loading-dim {
  color: var(--dim); font-size: .8rem; font-style: italic; margin-bottom: 1.5rem;
}
`

export default function TitleScreen({ onStart, onResume }) {
  const [campaigns, setCampaigns] = useState([])
  const [loadingCampaigns, setLoadingCampaigns] = useState(true)

  useEffect(() => {
    api.listCampaigns()
      .then(data => setCampaigns(data.campaigns || []))
      .catch(() => setCampaigns([]))
      .finally(() => setLoadingCampaigns(false))
  }, [])

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: TITLE_STYLES }} />
      <div className="title">
        <div className="ornament">⚔</div>
        <h1 className="cn">MythWeaver</h1>
        <p className="sub">
          An agentic AI Dungeon Master forged in the traditions of D&D 5th Edition.
          Your choices shape the world. Your story is your own.
        </p>

        {loadingCampaigns ? (
          <div className="loading-dim">Searching for saved adventures…</div>
        ) : campaigns.length > 0 ? (
          <>
            <div style={{ width: '100%', maxWidth: '520px' }}>
              <div className="section-label">Continue an Adventure</div>
            </div>
            <div className="campaign-list">
              {campaigns.map(c => (
                <button
                  key={c.campaign.id}
                  className="campaign-card"
                  onClick={() => onResume(c.campaign.id)}
                >
                  <div>
                    <div className="campaign-card-name">
                      {c.player
                        ? `${c.player.name} — ${c.player.race} ${c.player.class} Lv.${c.player.level}`
                        : c.campaign.name}
                    </div>
                    <div className="campaign-card-sub">
                      {c.player
                        ? `${c.player.current_hp}/${c.player.max_hp} HP · ${c.player.experience} XP · ${c.campaign.name}`
                        : 'No character data'}
                    </div>
                  </div>
                  {c.has_active_session && (
                    <div className="campaign-card-badge">Active</div>
                  )}
                </button>
              ))}
            </div>

            <hr className="divider" />

            <button className="btn-gold" onClick={onStart}>Begin New Legend</button>
          </>
        ) : (
          <button className="btn-gold" onClick={onStart}>Begin Your Legend</button>
        )}

        <p className="sig">Created by Matt Taylor · Powered by Anthropic · D&D 5e</p>
      </div>
    </>
  )
}