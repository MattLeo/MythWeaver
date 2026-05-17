import { useState, useEffect, useRef, useCallback } from 'react'
import * as api from '../api/client.js'

const NOTES_STYLES = `
.notes-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0,0,0,.65); backdrop-filter: blur(3px);
  display: flex; align-items: center; justify-content: center;
}
.notes-modal {
  background: var(--surf); border: 1px solid var(--bord);
  border-radius: 3px; width: min(680px, 95vw); height: min(600px, 90vh);
  display: flex; flex-direction: column; overflow: hidden;
}
.notes-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 1rem 1.25rem; border-bottom: 1px solid var(--bord);
  flex-shrink: 0;
}
.notes-title {
  font-family: 'Cinzel', serif; font-size: .9rem;
  color: var(--goldl); letter-spacing: .1em; text-transform: uppercase;
}
.notes-status {
  font-size: .7rem; color: var(--dim); font-style: italic;
  transition: opacity .3s;
}
.notes-close-btn {
  background: none; border: 1px solid var(--bord); border-radius: 2px;
  color: var(--dim); cursor: pointer; font-size: .75rem;
  padding: .3rem .8rem; font-family: 'Cinzel', serif;
  letter-spacing: .08em;
}
.notes-close-btn:hover { border-color: var(--goldl); color: var(--goldl); }
.notes-textarea {
  flex: 1; width: 100%; resize: none;
  background: var(--bg); color: var(--fg);
  border: none; outline: none;
  font-size: .85rem; line-height: 1.7;
  padding: 1.25rem; font-family: inherit;
  box-sizing: border-box;
}
.notes-textarea::placeholder { color: var(--dim); }
.notes-footer {
  padding: .6rem 1.25rem; border-top: 1px solid var(--bord);
  font-size: .68rem; color: var(--dim); flex-shrink: 0;
}
`

export default function NotesModal({ campaignId, onClose }) {
  const [notes, setNotes]   = useState('')
  const [status, setStatus] = useState('') // '', 'saving...', 'saved'
  const [loaded, setLoaded] = useState(false)
  const debounceRef = useRef(null)

  // Load on mount
  useEffect(() => {
    api.getNotes(campaignId)
      .then(data => { setNotes(data.notes || ''); setLoaded(true) })
      .catch(() => setLoaded(true))
  }, [campaignId])

  // Debounced auto-save
  const save = useCallback((value) => {
    if (debounceRef.current) clearTimeout(debounceRef.current)
    setStatus('saving…')
    debounceRef.current = setTimeout(async () => {
      try {
        await api.saveNotes(campaignId, value)
        setStatus('saved')
        setTimeout(() => setStatus(''), 2000)
      } catch {
        setStatus('error saving')
      }
    }, 800)
  }, [campaignId])

  const handleChange = (e) => {
    setNotes(e.target.value)
    save(e.target.value)
  }

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: NOTES_STYLES }} />
      <div className="notes-overlay" onClick={e => e.target === e.currentTarget && onClose()}>
        <div className="notes-modal">

          <div className="notes-header">
            <div className="notes-title">✦ Campaign Notes</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
              <span className="notes-status">{status}</span>
              <button className="notes-close-btn" onClick={onClose}>Close</button>
            </div>
          </div>

          <textarea
            className="notes-textarea"
            placeholder="Jot down quest leads, NPC names, secrets uncovered, things to remember…"
            value={notes}
            onChange={handleChange}
            disabled={!loaded}
            autoFocus={loaded}
          />

          <div className="notes-footer">
            Notes are saved automatically as you type.
          </div>

        </div>
      </div>
    </>
  )
}