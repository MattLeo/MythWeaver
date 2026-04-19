import { STYLES } from '../styles.js'

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
`

export default function TitleScreen({ onStart }) {
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
        <button className="btn-gold" onClick={onStart}>Begin Your Legend</button>
        <p className="sig">Created by Matt Taylor · Powered by Ollama · D&D 5e</p>
      </div>
    </>
  )
}