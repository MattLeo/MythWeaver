import { useState, useEffect } from 'react'

const DICE_FACES = { d4: 4, d6: 6, d8: 8, d10: 10, d12: 12, d20: 20 }

const OVERLAY_STYLES = `
.roll-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(11, 12, 18, 0.92);
  display: flex; align-items: center; justify-content: center;
  animation: fadeIn .2s ease;
}
@keyframes fadeIn { from { opacity: 0 } to { opacity: 1 } }

.roll-card {
  background: var(--surf); border: 1px solid var(--gold);
  border-radius: 4px; padding: 2.5rem 3rem;
  text-align: center; max-width: 420px; width: 90%;
  box-shadow: 0 0 60px rgba(200,150,42,.2);
}
.roll-title {
  font-family: 'Cinzel', serif; font-size: .7rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--gold); margin-bottom: .5rem;
}
.roll-skill {
  font-family: 'Cinzel', serif; font-size: 1.4rem;
  color: var(--goldl); margin-bottom: .25rem;
}
.roll-dc {
  font-size: .82rem; color: var(--dim);
  font-style: italic; margin-bottom: 1.5rem;
}
.roll-reason {
  font-size: .88rem; color: var(--text);
  line-height: 1.7; margin-bottom: 1.75rem;
  font-style: italic;
}
.dice-display {
  width: 100px; height: 100px; margin: 0 auto 1.5rem;
  display: flex; align-items: center; justify-content: center;
  border: 2px solid var(--gold); border-radius: 4px;
  background: var(--elev); position: relative;
  transition: all .1s;
}
.dice-display.rolling {
  animation: shake .5s ease-in-out;
  border-color: var(--goldl);
  box-shadow: 0 0 24px rgba(232,196,106,.35);
}
@keyframes shake {
  0%,100% { transform: rotate(0deg) scale(1); }
  15% { transform: rotate(-8deg) scale(1.05); }
  30% { transform: rotate(8deg) scale(1.08); }
  45% { transform: rotate(-6deg) scale(1.05); }
  60% { transform: rotate(6deg) scale(1.03); }
  75% { transform: rotate(-3deg) scale(1.01); }
}
.dice-number {
  font-family: 'Cinzel', serif;
  font-size: 2.8rem; font-weight: 700;
  color: var(--goldl); line-height: 1;
}
.dice-type {
  position: absolute; bottom: 4px; right: 7px;
  font-family: 'Cinzel', serif; font-size: .6rem;
  color: var(--dim); letter-spacing: .1em;
}
.roll-result-line {
  font-family: 'Cinzel', serif; font-size: .9rem;
  margin-bottom: 1.5rem; min-height: 1.4rem;
}
.roll-result-line.success { color: var(--grn); }
.roll-result-line.failure { color: var(--red); }
.roll-result-line.pending { color: var(--dim); }
`

export default function DiceRollOverlay({ rollRequest, onComplete }) {
  const [phase, setPhase] = useState('prompt')   // prompt | rolling | result
  const [result, setResult] = useState(null)
  const [displayNum, setDisplayNum] = useState('?')

  const sides = DICE_FACES[rollRequest.die] || 20

  const handleRoll = () => {
    setPhase('rolling')
    const finalResult = Math.floor(Math.random() * sides) + 1

    // Rapid number cycling
    let cycles = 0
    const interval = setInterval(() => {
      setDisplayNum(Math.floor(Math.random() * sides) + 1)
      cycles++
      if (cycles > 14) {
        clearInterval(interval)
        setDisplayNum(finalResult)
        setResult(finalResult)
        setPhase('result')
      }
    }, 60)
  }

  const handleContinue = () => {
    onComplete(result)
  }

  const isSuccess = result !== null && rollRequest.dc !== null && result >= rollRequest.dc
  const isCritical = result === 20
  const isFumble = result === 1

  const resultLabel = () => {
    if (result === null) return ''
    if (isCritical) return '✦ Critical Success!'
    if (isFumble) return '✗ Critical Failure!'
    if (result >= rollRequest.dc) return '✓ Success'
    return '✗ Failure'
  }

  const resultClass = () => {
    if (result === null) return 'pending'
    if (isCritical || result >= rollRequest.dc) return 'success'
    return 'failure'
  }

  return (
    <>
      <style dangerouslySetInnerHTML={{ __html: OVERLAY_STYLES }} />
      <div className="roll-overlay">
        <div className="roll-card">
          <div className="roll-title">Dice Roll Required</div>
          <div className="roll-skill">{rollRequest.skill}</div>
          {rollRequest.dc && (
            <div className="roll-dc">Difficulty Class {rollRequest.dc}</div>
          )}
          <div className="roll-reason">{rollRequest.reason}</div>

          <div className={`dice-display${phase === 'rolling' ? ' rolling' : ''}`}>
            <div className="dice-number">
              {phase === 'prompt' ? rollRequest.die.toUpperCase().replace('D', '') : displayNum}
            </div>
            <div className="dice-type">{rollRequest.die}</div>
          </div>

          <div className={`roll-result-line ${resultClass()}`}>
            {resultLabel()}
          </div>

          {phase === 'prompt' && (
            <button className="btn-gold" onClick={handleRoll}>
              Roll {rollRequest.die.toUpperCase()}
            </button>
          )}

          {phase === 'result' && (
            <button className="btn-gold" onClick={handleContinue}>
              Continue →
            </button>
          )}
        </div>
      </div>
    </>
  )
}