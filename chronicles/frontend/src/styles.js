export const STYLES = `
/* ── Typography ── */
.cn { font-family: 'Cinzel', serif; }

/* ── Buttons ── */
.btn-gold {
  background: linear-gradient(135deg, #8c6418, #c8962a);
  color: #0b0c12; border: none; cursor: pointer;
  font-family: 'Cinzel', serif; font-size: .9rem; font-weight: 700;
  letter-spacing: .15em; text-transform: uppercase;
  padding: .9rem 2.5rem; border-radius: 2px;
  transition: all .2s; box-shadow: 0 4px 24px rgba(200,150,42,.28);
}
.btn-gold:hover:not(:disabled) { transform: translateY(-2px); box-shadow: 0 8px 32px rgba(200,150,42,.45); }
.btn-gold:disabled { opacity: .45; cursor: not-allowed; }

.btn-ghost {
  background: transparent; color: var(--gold);
  border: 1px solid var(--gold); cursor: pointer;
  font-family: 'Cinzel', serif; font-size: .8rem;
  letter-spacing: .1em; text-transform: uppercase;
  padding: .65rem 1.75rem; border-radius: 2px; transition: all .2s;
}
.btn-ghost:hover { background: rgba(200,150,42,.1); }

.btn-sm {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; color: var(--dim);
  font-family: 'Cinzel', serif; font-size: .68rem; cursor: pointer;
  padding: .3rem .55rem; transition: all .15s; letter-spacing: .05em;
}
.btn-sm:hover { border-color: var(--gold); color: var(--gold); }

/* ── Form elements ── */
.inp {
  width: 100%; background: var(--elev);
  border: 1px solid var(--bord); border-radius: 2px;
  padding: .8rem 1rem; color: var(--text);
  font-family: 'Lora', serif; font-size: 1rem;
  outline: none; transition: border .2s;
}
.inp:focus { border-color: var(--gold); }
.inp::placeholder { color: var(--dim); }

/* ── Cards / Sections ── */
.sec { border: 1px solid var(--bord); border-radius: 2px; padding: .8rem; }
.sec-title {
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .16em; text-transform: uppercase;
  color: var(--dim); margin-bottom: .5rem;
}

/* ── Grid ── */
.pick-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(120px, 1fr)); gap: .6rem; }
.pick {
  background: var(--elev); border: 1px solid var(--bord);
  border-radius: 2px; padding: .65rem .5rem;
  cursor: pointer; text-align: center; font-size: .85rem;
  color: var(--dim); transition: all .2s;
}
.pick:hover, .pick.sel {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.07);
}

/* ── Typing indicator ── */
.typing { display: flex; gap: .35rem; align-items: center; padding: .15rem 0; }
.dot {
  width: 7px; height: 7px; border-radius: 50%;
  background: var(--gold); animation: pulse 1.2s ease-in-out infinite;
}
.dot:nth-child(2) { animation-delay: .2s; }
.dot:nth-child(3) { animation-delay: .4s; }
@keyframes pulse {
  0%,80%,100% { opacity: .25; transform: scale(.8); }
  40% { opacity: 1; transform: scale(1); }
}

/* ── Stat row ── */
.stat-r { display: flex; justify-content: space-between; align-items: baseline; font-size: .78rem; padding: .15rem 0; }
.sr-l { font-family: 'Cinzel', serif; font-size: .65rem; letter-spacing: .08em; color: var(--dim); }
.sr-v { color: var(--text); }
.sr-m { color: var(--gold); min-width: 2rem; text-align: right; }
`