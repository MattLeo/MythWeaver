import { useState, useRef, useEffect } from 'react'
import { STYLES } from '../styles.js'

const GAME_STYLES = `
${STYLES}
.story { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.msgs {
  flex: 1; overflow-y: auto; padding: 1.5rem;
  display: flex; flex-direction: column; gap: 1.1rem;
}
.msg-dm {
  background: var(--surf); border: 1px solid var(--bord);
  border-left: 3px solid var(--gold);
  border-radius: 0 3px 3px 0; padding: 1.2rem 1.4rem;
  width: 100%; line-height: 1.9; font-size: .93rem;
}
.msg-dm p { margin-bottom: .55rem; }
.msg-dm p:last-child { margin-bottom: 0; }
.dm-lbl {
  font-family: 'Cinzel', serif; font-size: .62rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--gold); margin-bottom: .7rem;
  display: flex; align-items: center; gap: .4rem;
}
.msg-pl {
  background: var(--elev); border: 1px solid var(--bord);
  border-right: 3px solid var(--dim);
  border-radius: 3px 0 0 3px; padding: .8rem 1.2rem;
  max-width: 70%; align-self: flex-end;
  font-size: .88rem; color: var(--dim);
  line-height: 1.75; font-style: italic;
}
.pl-lbl {
  font-family: 'Cinzel', serif; font-size: .62rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--dim); margin-bottom: .4rem;
}
.empty {
  text-align: center; color: var(--dim); font-style: italic;
  margin-top: 3rem; font-family: 'Cinzel', serif;
  font-size: .8rem; letter-spacing: .12em;
}
.tool-indicator {
  font-family: 'Cinzel', serif; font-size: .65rem;
  color: var(--dim); letter-spacing: .15em;
  text-transform: uppercase; padding: .4rem 0;
  display: flex; align-items: center; gap: .4rem;
  opacity: .6;
}
.tool-dot {
  width: 5px; height: 5px; border-radius: 50%;
  background: var(--gold); animation: pulse 1s ease-in-out infinite;
}
.input-area {
  border-top: 1px solid var(--bord); padding: 1rem 1.4rem;
  background: var(--surf); display: flex; gap: .75rem; align-items: flex-end;
}
.input-area textarea {
  flex: 1; background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .7rem 1rem; color: var(--text);
  font-family: 'Lora', serif; font-size: .9rem; resize: none;
  outline: none; min-height: 50px; max-height: 140px;
  line-height: 1.65; transition: border .2s; width: 100%;
}
.input-area textarea:focus { border-color: var(--gold); }
.input-area textarea::placeholder { color: var(--dim); }
.send {
  background: linear-gradient(135deg, #8c6418, #c8962a);
  border: none; cursor: pointer; color: #0b0c12;
  font-family: 'Cinzel', serif; font-size: .75rem; font-weight: 700;
  letter-spacing: .12em; text-transform: uppercase;
  padding: .7rem 1.2rem; border-radius: 2px; white-space: nowrap;
  transition: all .2s; align-self: flex-end;
}
.send:hover:not(:disabled) { box-shadow: 0 4px 18px rgba(200,150,42,.4); transform: translateY(-1px); }
.send:disabled { opacity: .4; cursor: not-allowed; }
.state-bar {
  border-bottom: 1px solid var(--bord); padding: .5rem 1.4rem;
  background: var(--surf); display: flex; gap: .4rem; align-items: center;
}
.state-btn {
  background: transparent; border: 1px solid var(--bord);
  border-radius: 2px; color: var(--dim); cursor: pointer;
  font-family: 'Cinzel', serif; font-size: .6rem;
  letter-spacing: .1em; text-transform: uppercase;
  padding: .2rem .55rem; transition: all .15s;
}
.state-btn:hover, .state-btn.active {
  border-color: var(--gold); color: var(--gold);
  background: rgba(200,150,42,.07);
}
.state-label { font-size: .65rem; color: var(--dim); margin-right: .3rem; font-family: 'Cinzel', serif; letter-spacing: .1em; }
.mob-toggle {
  display: none; align-items: center; justify-content: center;
  position: fixed; bottom: 80px; right: 12px; z-index: 20;
  width: 40px; height: 40px; background: var(--surf);
  border: 1px solid var(--gold); border-radius: 50%;
  cursor: pointer; font-family: 'Cinzel', serif;
  font-size: .7rem; color: var(--gold);
}
@media(max-width:700px) { .mob-toggle { display: flex; } }
`

const GAME_STATES = ['exploration', 'combat', 'dialogue', 'rest', 'leveling', 'shopping']

export default function GameScreen({
  player, messages, loading, onSend,
  gameState, onGameStateChange,
  onToggleSidebar, sidebarOpen
}) {
  const [input, setInput] = useState('')
  const endRef = useRef(null)
  const taRef = useRef(null)

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages, loading])

  const handleSend = () => {
    if (!input.trim() || loading) return
    onSend(input.trim())
    setInput('')
  }

  const onKey = (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: GAME_STYLES }} />
      <div className="story">

        {/* Game state bar */}
        <div className="state-bar">
          <span className="state-label">Scene:</span>
          {GAME_STATES.map(s => (
            <button
              key={s}
              className={`state-btn${gameState === s ? ' active' : ''}`}
              onClick={() => onGameStateChange(s)}
            >
              {s}
            </button>
          ))}
        </div>

        {/* Messages */}
        <div className="msgs">
          {messages.length === 0 && !loading && (
            <div className="empty">✦ &nbsp; THE ADVENTURE BEGINS &nbsp; ✦</div>
          )}

          {messages.map(msg => (
            msg.role === 'dm'
              ? <div key={msg.id} className="msg-dm">
                <div className="dm-lbl">⚔ Dungeon Master</div>
                {msg.content.split('\n').filter(l => l.trim()).map((p, i) => (
                  <p key={i}>{p}</p>
                ))}
                {msg.tools_used && msg.tools_used.length > 0 && (
                  <div style={{ marginTop: '.5rem', fontSize: '.65rem', color: 'var(--dim)', fontFamily: 'Cinzel, serif', letterSpacing: '.1em' }}>
                    ⚙ {msg.tools_used.join(' · ')}
                  </div>
                )}
              </div>
              : <div key={msg.id} className="msg-pl">
                <div className="pl-lbl">✦ {player?.name || 'You'}</div>
                {msg.content}
              </div>
          ))}

          {loading && (
            <div className="msg-dm">
              <div className="dm-lbl">⚔ Dungeon Master</div>
              <div className="typing">
                <div className="dot" />
                <div className="dot" />
                <div className="dot" />
              </div>
              <div className="tool-indicator">
                <div className="tool-dot" />
                consulting the world…
              </div>
            </div>
          )}

          <div ref={endRef} />
        </div>

        {/* Input */}
        <div className="input-area">
          <textarea
            ref={taRef}
            placeholder="Describe your action, speak to an NPC, shape the world…"
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={onKey}
            rows={2}
          />
          <button className="send" disabled={loading || !input.trim()} onClick={handleSend}>
            {loading ? '…' : 'Act →'}
          </button>
        </div>
      </div>

      {/* Mobile sidebar toggle */}
      <button className="mob-toggle" onClick={onToggleSidebar}>
        {sidebarOpen ? '✕' : '☰'}
      </button>
    </>
  )
}