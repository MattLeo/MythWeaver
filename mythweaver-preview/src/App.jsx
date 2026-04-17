import { useState, useRef, useEffect } from "react";

// ─── constants ──────────────────────────────────────────────────────────────
const RACES = ["Human","Elf","Dwarf","Halfling","Half-Elf","Half-Orc","Gnome","Tiefling","Dragonborn"];
const CLASSES = ["Barbarian","Bard","Cleric","Druid","Fighter","Monk","Paladin","Ranger","Rogue","Sorcerer","Warlock","Wizard"];
const BACKGROUNDS = ["Acolyte","Charlatan","Criminal","Entertainer","Folk Hero","Hermit","Noble","Outlander","Sage","Soldier","Urchin"];
const CLASS_HD = {Barbarian:12,Fighter:10,Paladin:10,Ranger:10,Cleric:8,Druid:8,Monk:8,Rogue:8,Bard:8,Warlock:8,Sorcerer:6,Wizard:6};
const STATS = ["STR","DEX","CON","INT","WIS","CHA"];
const DICE = [4,6,8,10,12,20];

// ─── helpers ────────────────────────────────────────────────────────────────
const d = (n) => Math.floor(Math.random()*n)+1;
const mod = (v) => Math.floor((v-10)/2);
const fmt = (v) => { const m=mod(v); return (m>=0?"+":"")+m; };
const rollBlock = () => Array.from({length:6},()=>{const r=[d(6),d(6),d(6),d(6)].sort((a,b)=>a-b);return r[1]+r[2]+r[3];});

// ─── DM system prompt ────────────────────────────────────────────────────────
function buildPrompt(c) {
  return `You are the Dungeon Master for a collaborative D&D 5th Edition tabletop adventure.

PLAYER CHARACTER:
- Name: ${c.name} | Race: ${c.race} | Class: ${c.class} Lv.${c.level} | Background: ${c.background}
- STR ${c.stats[0]}, DEX ${c.stats[1]}, CON ${c.stats[2]}, INT ${c.stats[3]}, WIS ${c.stats[4]}, CHA ${c.stats[5]}
- Current HP: ${c.hp}/${c.maxHp}${c.backstory?`\n- Backstory: ${c.backstory}`:""}

YOUR ROLE:

STORYTELLING
- Write vivid, literary prose: 2-4 paragraphs per turn. Use all five senses.
- Always end with a clear decision point, question, or situation requiring the player's next action.
- Vary tone — tense in combat, eerie in exploration, warm in social scenes.

WORLD-BUILDING (crucial)
- The world begins as a blank canvas built collaboratively with the player.
- When the player proposes lore ("I've heard a dragon cult controls this city", "My family once ruled here", "There's a legend about..."), EMBRACE it and canonize it permanently. Let their history, rumors, and assumptions shape the world.
- Introduce factions, mysteries, NPCs, and secrets gradually through play.

D&D 5e RULES
- Skill checks: when outcome is uncertain, call for one — e.g. "Roll a DC 14 Dexterity (Stealth) check" or "Make a Perception check (DC 12)".
- Saving throws: e.g. "Make a DC 15 Constitution saving throw".
- Combat: track initiative narratively, describe enemy condition (lightly wounded, bloodied, staggered, near death) — never exact numbers.
- Apply class features meaningfully: Sneak Attack for Rogues, Rage for Barbarians, spell slots for casters, etc.
- Background proficiencies matter: Criminals have advantage on Stealth/Deception, Soldiers on Intimidation, etc.
- When HP drops to 0: begin death saving throw sequence.

HP TRACKING (IMPORTANT)
- Whenever the player takes or heals damage, always include this exact format somewhere in your response: [HP: X/Y] where X = new current HP, Y = max HP. This lets the tracker update automatically.
- Keep HP math consistent with what's been established.

DICE ROLLS
- When the player reports their roll (e.g. "I rolled a 17"), narrate the outcome dramatically based on the DC you set. High rolls = cinematic success. Low rolls = complications, not necessarily failure.

STYLE
- Prose should feel like a great fantasy novel: specific, sensory, surprising.
- Create memorable NPCs with distinct voices, goals, and secrets.
- Plant seeds for future revelations. Reward curiosity and bold action.
- Never be a passive narrator — you are an active co-author.`;
}

// ─── CSS ────────────────────────────────────────────────────────────────────
const STYLES = `
@import url('https://fonts.googleapis.com/css2?family=Cinzel:wght@400;600;700&family=Lora:ital,wght@0,400;0,500;1,400&display=swap');
*{box-sizing:border-box;margin:0;padding:0;}
:root{
  --bg:#0b0c12;--surf:#11121a;--elev:#171825;--bord:#23243a;
  --gold:#c8962a;--goldl:#e8c46a;--text:#ece2ca;--dim:#6e7492;
  --red:#9b2535;--grn:#2a7a50;
}
html,body,#root{height:100%;overflow:hidden;background:var(--bg);color:var(--text);font-family:'Lora',Georgia,serif;}
.cn{font-family:'Cinzel',serif;}
/* scrollbars */
*::-webkit-scrollbar{width:4px;height:4px;}
*::-webkit-scrollbar-track{background:transparent;}
*::-webkit-scrollbar-thumb{background:var(--bord);border-radius:2px;}

/* ── Title ── */
.title{min-height:100vh;display:flex;flex-direction:column;align-items:center;justify-content:center;
  text-align:center;padding:2rem;
  background:radial-gradient(ellipse at 50% 25%,#17062a 0%,#0b0c12 65%);}
.title h1{font-family:'Cinzel',serif;font-size:clamp(2rem,6vw,4.8rem);color:var(--goldl);
  letter-spacing:.12em;text-shadow:0 0 60px rgba(232,196,106,.35);line-height:1.15;}
.title .sub{color:var(--dim);max-width:480px;margin:1.2rem auto 2.5rem;font-style:italic;line-height:1.85;font-size:.95rem;}
.title .sig{margin-top:1.5rem;font-size:.75rem;color:var(--dim);font-family:'Cinzel',serif;letter-spacing:.15em;}
.ornament{font-size:2rem;margin-bottom:1rem;opacity:.7;}

/* ── Buttons ── */
.btn-gold{background:linear-gradient(135deg,#8c6418,#c8962a);color:#0b0c12;border:none;cursor:pointer;
  font-family:'Cinzel',serif;font-size:.9rem;font-weight:700;letter-spacing:.15em;text-transform:uppercase;
  padding:.9rem 2.5rem;border-radius:2px;transition:all .2s;box-shadow:0 4px 24px rgba(200,150,42,.28);}
.btn-gold:hover:not(:disabled){transform:translateY(-2px);box-shadow:0 8px 32px rgba(200,150,42,.45);}
.btn-gold:disabled{opacity:.45;cursor:not-allowed;}
.btn-ghost{background:transparent;color:var(--gold);border:1px solid var(--gold);cursor:pointer;
  font-family:'Cinzel',serif;font-size:.8rem;letter-spacing:.1em;text-transform:uppercase;
  padding:.65rem 1.75rem;border-radius:2px;transition:all .2s;}
.btn-ghost:hover{background:rgba(200,150,42,.1);}
.btn-sm{background:var(--elev);border:1px solid var(--bord);border-radius:2px;color:var(--dim);
  font-family:'Cinzel',serif;font-size:.68rem;cursor:pointer;padding:.3rem .55rem;transition:all .15s;letter-spacing:.05em;}
.btn-sm:hover{border-color:var(--gold);color:var(--gold);}

/* ── Creation ── */
.creation{min-height:100vh;display:flex;flex-direction:column;align-items:center;justify-content:center;
  padding:2rem;background:radial-gradient(ellipse at 50% 0%,#0d1220 0%,#0b0c12 60%);}
.card{background:var(--surf);border:1px solid var(--bord);border-radius:3px;padding:2.25rem;
  max-width:680px;width:100%;}
.card h2{font-family:'Cinzel',serif;color:var(--gold);font-size:1.3rem;margin-bottom:1.5rem;
  padding-bottom:.75rem;border-bottom:1px solid var(--bord);}
.steps{display:flex;gap:.45rem;justify-content:center;margin-bottom:2rem;}
.step{width:32px;height:3px;border-radius:2px;background:var(--bord);transition:background .3s;}
.step.on{background:var(--gold);}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(120px,1fr));gap:.6rem;}
.pick{background:var(--elev);border:1px solid var(--bord);border-radius:2px;padding:.65rem .5rem;
  cursor:pointer;text-align:center;font-size:.85rem;color:var(--dim);transition:all .2s;}
.pick:hover,.pick.sel{border-color:var(--gold);color:var(--goldl);background:rgba(200,150,42,.07);}
.inp{width:100%;background:var(--elev);border:1px solid var(--bord);border-radius:2px;
  padding:.8rem 1rem;color:var(--text);font-family:'Lora',serif;font-size:1rem;outline:none;transition:border .2s;}
.inp:focus{border-color:var(--gold);}
.inp::placeholder{color:var(--dim);}
.stat-g{display:grid;grid-template-columns:repeat(3,1fr);gap:.65rem;margin:.75rem 0;}
.stat-box{background:var(--elev);border:1px solid var(--bord);border-radius:2px;padding:.7rem;text-align:center;}
.stat-box .sl{font-family:'Cinzel',serif;font-size:.65rem;letter-spacing:.12em;color:var(--dim);margin-bottom:.2rem;}
.stat-box .sv{font-size:1.6rem;color:var(--goldl);font-weight:bold;line-height:1;}
.stat-box .sm{font-size:.75rem;color:var(--dim);margin-top:.15rem;}
.cnav{display:flex;justify-content:space-between;align-items:center;margin-top:2rem;}

/* ── Game ── */
.game{display:flex;height:100vh;overflow:hidden;}
.sidebar{width:250px;min-width:250px;background:var(--surf);border-right:1px solid var(--bord);
  overflow-y:auto;padding:1.1rem;display:flex;flex-direction:column;gap:.85rem;}
.sec{border:1px solid var(--bord);border-radius:2px;padding:.8rem;}
.sec-title{font-family:'Cinzel',serif;font-size:.65rem;letter-spacing:.16em;text-transform:uppercase;
  color:var(--dim);margin-bottom:.5rem;}
.cn-name{font-family:'Cinzel',serif;font-size:1rem;color:var(--goldl);}
.cn-sub{font-size:.75rem;color:var(--dim);margin-top:.1rem;}
.hp-bar{background:var(--bord);border-radius:1px;height:5px;margin:.4rem 0 .2rem;}
.hp-fill{height:100%;border-radius:1px;transition:width .6s,background .6s;}
.hp-txt{font-size:.8rem;}
.stat-r{display:flex;justify-content:space-between;align-items:baseline;font-size:.78rem;padding:.15rem 0;}
.sr-l{font-family:'Cinzel',serif;font-size:.65rem;letter-spacing:.08em;color:var(--dim);}
.sr-v{color:var(--text);}
.sr-m{color:var(--gold);min-width:2rem;text-align:right;}
.inv-i{font-size:.76rem;color:var(--dim);padding:.18rem 0;border-bottom:1px solid var(--bord);}
.inv-i:last-child{border-bottom:none;}
.gp{font-size:.82rem;color:var(--gold);margin-top:.4rem;}
.dice-row{display:flex;flex-wrap:wrap;gap:.4rem;margin:.4rem 0;}
.roll-res{font-family:'Cinzel',serif;font-size:.8rem;color:var(--goldl);margin-top:.3rem;}
.roll-hint{font-size:.68rem;color:var(--dim);margin-top:.35rem;line-height:1.5;}
.hp-adj{display:flex;align-items:center;gap:.4rem;margin-top:.4rem;}
.hp-adj button{background:var(--elev);border:1px solid var(--bord);color:var(--text);width:26px;height:26px;
  border-radius:2px;cursor:pointer;font-size:.9rem;display:flex;align-items:center;justify-content:center;}
.hp-adj button:hover{border-color:var(--gold);color:var(--gold);}
.hp-adj input{background:var(--elev);border:1px solid var(--bord);color:var(--text);width:42px;
  text-align:center;border-radius:2px;padding:.2rem;font-size:.78rem;font-family:'Cinzel',serif;}
.hp-adj input:focus{outline:none;border-color:var(--gold);}

/* ── Story ── */
.story{flex:1;display:flex;flex-direction:column;overflow:hidden;}
.msgs{flex:1;overflow-y:auto;padding:1.5rem;display:flex;flex-direction:column;gap:1.1rem;}
.msg-dm{background:var(--surf);border:1px solid var(--bord);border-left:3px solid var(--gold);
  border-radius:0 3px 3px 0;padding:1.2rem 1.4rem;max-width:95%;line-height:1.9;font-size:.93rem;}
.msg-dm p{margin-bottom:.55rem;}
.msg-dm p:last-child{margin-bottom:0;}
.dm-lbl{font-family:'Cinzel',serif;font-size:.62rem;letter-spacing:.2em;text-transform:uppercase;
  color:var(--gold);margin-bottom:.7rem;display:flex;align-items:center;gap:.4rem;}
.msg-pl{background:var(--elev);border:1px solid var(--bord);border-right:3px solid var(--dim);
  border-radius:3px 0 0 3px;padding:.8rem 1.2rem;max-width:70%;align-self:flex-end;
  font-size:.88rem;color:var(--dim);line-height:1.75;font-style:italic;}
.pl-lbl{font-family:'Cinzel',serif;font-size:.62rem;letter-spacing:.2em;text-transform:uppercase;
  color:var(--dim);margin-bottom:.4rem;}
.typing{display:flex;gap:.35rem;align-items:center;padding:.15rem 0;}
.dot{width:7px;height:7px;border-radius:50%;background:var(--gold);animation:pulse 1.2s ease-in-out infinite;}
.dot:nth-child(2){animation-delay:.2s;}
.dot:nth-child(3){animation-delay:.4s;}
@keyframes pulse{0%,80%,100%{opacity:.25;transform:scale(.8)}40%{opacity:1;transform:scale(1)}}
.empty{text-align:center;color:var(--dim);font-style:italic;margin-top:3rem;
  font-family:'Cinzel',serif;font-size:.8rem;letter-spacing:.12em;}
.input-area{border-top:1px solid var(--bord);padding:1rem 1.4rem;background:var(--surf);display:flex;gap:.75rem;align-items:flex-end;}
.input-area textarea{flex:1;background:var(--elev);border:1px solid var(--bord);border-radius:2px;
  padding:.7rem 1rem;color:var(--text);font-family:'Lora',serif;font-size:.9rem;resize:none;outline:none;
  min-height:50px;max-height:140px;line-height:1.65;transition:border .2s;}
.input-area textarea:focus{border-color:var(--gold);}
.input-area textarea::placeholder{color:var(--dim);}
.send{background:linear-gradient(135deg,#8c6418,#c8962a);border:none;cursor:pointer;color:#0b0c12;
  font-family:'Cinzel',serif;font-size:.75rem;font-weight:700;letter-spacing:.12em;text-transform:uppercase;
  padding:.7rem 1.2rem;border-radius:2px;white-space:nowrap;transition:all .2s;align-self:flex-end;}
.send:hover:not(:disabled){box-shadow:0 4px 18px rgba(200,150,42,.4);transform:translateY(-1px);}
.send:disabled{opacity:.4;cursor:not-allowed;}
.mob-toggle{display:none;}
@media(max-width:700px){
  .sidebar{position:fixed;left:0;top:0;bottom:0;z-index:10;transform:translateX(-100%);transition:transform .25s;}
  .sidebar.open{transform:translateX(0);}
  .mob-toggle{display:flex;align-items:center;justify-content:center;position:fixed;bottom:80px;right:12px;
    z-index:20;width:40px;height:40px;background:var(--surf);border:1px solid var(--gold);border-radius:50%;
    cursor:pointer;font-family:'Cinzel',serif;font-size:.7rem;color:var(--gold);}
}
`;

export default function App() {
  const [phase, setPhase] = useState("title");
  const [step, setStep] = useState(0);
  const [char, setChar] = useState({
    name:"",race:"",class:"",background:"",
    stats:rollBlock(),hp:10,maxHp:10,level:1,
    inventory:["Adventurer's Pack","Torch (5)","Rations (3 days)"],
    gold:15,backstory:""
  });
  const [messages, setMessages] = useState([]);
  const [hist, setHist] = useState([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [lastRoll, setLastRoll] = useState(null);
  const [hpDelta, setHpDelta] = useState("1");
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const endRef = useRef(null);
  const taRef = useRef(null);

  useEffect(()=>{ endRef.current?.scrollIntoView({behavior:"smooth"}); },[messages,loading]);

  const upd = (k,v) => setChar(c=>({...c,[k]:v}));

  const canNext = () => {
    if(step===0) return char.name.trim().length>1;
    if(step===1) return !!(char.race&&char.class);
    if(step===2) return !!char.background;
    return true;
  };

  const startGame = async () => {
    const hp = (CLASS_HD[char.class]||8)+mod(char.stats[2]);
    const fc = {...char,hp,maxHp:hp};
    setChar(fc);
    setPhase("game");
    setLoading(true);
    const openMsg = `Begin the adventure. Character: ${fc.name}, ${fc.race} ${fc.class}, Background: ${fc.background}.${fc.backstory?` Backstory: ${fc.backstory}`:""} Open with a vivid, atmospheric scene that places me immediately in a specific moment — tension or wonder, not a generic tavern.`;
    try {
      const r = await fetch("/api/v1/messages",{
        method:"POST",headers:{"Content-Type":"application/json"},
        body:JSON.stringify({model:"claude-sonnet-4-6",max_tokens:1000,
          system:buildPrompt(fc),messages:[{role:"user",content:openMsg}]})
      });
      const d = await r.json();
      const txt = d.content?.[0]?.text||"The adventure begins...";
      setMessages([{role:"dm",content:txt,id:Date.now()}]);
      setHist([{role:"user",content:openMsg},{role:"assistant",content:txt}]);
      const m = txt.match(/\[HP:\s*(\d+)\/(\d+)\]/);
      if(m) setChar(c=>({...c,hp:parseInt(m[1]),maxHp:parseInt(m[2])}));
    } catch(e){
      setMessages([{role:"dm",content:"The ancient magics waver... (Connection error — please try again.)",id:Date.now()}]);
    }
    setLoading(false);
  };

  const send = async () => {
    if(!input.trim()||loading) return;
    const txt = input.trim()+(lastRoll?` [Rolled: ${lastRoll}]`:"");
    setInput(""); setLastRoll(null);
    setMessages(m=>[...m,{role:"player",content:txt,id:Date.now()}]);
    setLoading(true);
    const newHist=[...hist,{role:"user",content:txt}];
    try {
      const r = await fetch("/api/v1/messages",{
        method:"POST",headers:{"Content-Type":"application/json"},
        body:JSON.stringify({model:"claude-sonnet-4-6",max_tokens:1000,
          system:buildPrompt(char),messages:newHist})
      });
      const data = await r.json();
      const dmTxt = data.content?.[0]?.text||"The DM considers your action...";
      setMessages(m=>[...m,{role:"dm",content:dmTxt,id:Date.now()}]);
      setHist([...newHist,{role:"assistant",content:dmTxt}]);
      const m = dmTxt.match(/\[HP:\s*(\d+)\/(\d+)\]/);
      if(m) setChar(c=>({...c,hp:parseInt(m[1]),maxHp:parseInt(m[2])}));
    } catch(e){
      setMessages(m=>[...m,{role:"dm",content:"The vision clouds... (Connection error.)",id:Date.now()}]);
    }
    setLoading(false);
  };

  const onKey = (e) => { if(e.key==="Enter"&&!e.shiftKey){e.preventDefault();send();} };

  const rollDie = (n) => { setLastRoll(`d${n}: ${d(n)}`); };

  const adjustHp = (dir) => {
    const delta = parseInt(hpDelta)||1;
    setChar(c=>({...c,hp:Math.max(0,Math.min(c.maxHp,c.hp+(dir*delta)))}));
  };

  const hpPct = char.maxHp>0?Math.max(0,(char.hp/char.maxHp)*100):0;
  const hpCol = hpPct>55?'var(--grn)':hpPct>25?'#b07820':'var(--red)';

  // ── TITLE ────────────────────────────────────────────────────────────────
  if(phase==="title") return (
    <>
      <style dangerouslySetInnerHTML={{__html:STYLES}}/>
      <div className="title">
        <div className="ornament">⚔</div>
        <h1 className="cn">Chronicles<br/>of the Realm</h1>
        <p className="sub">An AI-driven tabletop adventure forged in the traditions of D&D 5th Edition. Your choices shape the world. Your story is your own.</p>
        <button className="btn-gold" onClick={()=>setPhase("creation")}>Begin Your Legend</button>
        <p className="sig">Powered by Claude · D&D 5e</p>
      </div>
    </>
  );

  // ── CHARACTER CREATION ────────────────────────────────────────────────────
  if(phase==="creation") return (
    <>
      <style dangerouslySetInnerHTML={{__html:STYLES}}/>
      <div className="creation">
        <div className="card">
          <div className="steps">
            {[0,1,2,3].map(i=><div key={i} className={`step${i<=step?" on":""}`}/>)}
          </div>

          {step===0 && <>
            <h2>What is your name, adventurer?</h2>
            <input className="inp" autoFocus placeholder="Enter your character's name…" value={char.name}
              onChange={e=>upd("name",e.target.value)} onKeyDown={e=>{if(e.key==="Enter"&&canNext())setStep(1);}}/>
            <p style={{marginTop:"1rem",fontSize:".82rem",color:"var(--dim)",fontStyle:"italic"}}>
              This name will echo through the realm's history.
            </p>
          </>}

          {step===1 && <>
            <h2>Choose Your Heritage & Path</h2>
            <p className="sec-title" style={{marginBottom:".5rem"}}>Race</p>
            <div className="grid">
              {RACES.map(r=><div key={r} className={`pick${char.race===r?" sel":""}`} onClick={()=>upd("race",r)}>{r}</div>)}
            </div>
            <p className="sec-title" style={{margin:"1.1rem 0 .5rem"}}>Class</p>
            <div className="grid">
              {CLASSES.map(cl=><div key={cl} className={`pick${char.class===cl?" sel":""}`} onClick={()=>upd("class",cl)}>{cl}</div>)}
            </div>
          </>}

          {step===2 && <>
            <h2>Choose Your Background</h2>
            <p style={{fontSize:".85rem",color:"var(--dim)",fontStyle:"italic",marginBottom:"1rem"}}>
              Your background grants proficiencies and shapes how the world sees you.
            </p>
            <div className="grid">
              {BACKGROUNDS.map(b=><div key={b} className={`pick${char.background===b?" sel":""}`} onClick={()=>upd("background",b)}>{b}</div>)}
            </div>
          </>}

          {step===3 && <>
            <h2>Forge Your Legend</h2>
            <p style={{fontSize:".82rem",color:"var(--dim)",fontStyle:"italic",marginBottom:".75rem"}}>
              Roll your ability scores (4d6, drop lowest), then optionally shape your origin.
            </p>
            <div className="stat-g">
              {STATS.map((s,i)=>(
                <div key={s} className="stat-box">
                  <div className="sl">{s}</div>
                  <div className="sv">{char.stats[i]}</div>
                  <div className="sm">{fmt(char.stats[i])}</div>
                </div>
              ))}
            </div>
            <button className="btn-ghost" style={{marginBottom:"1.1rem"}} onClick={()=>upd("stats",rollBlock())}>
              ⚄ Reroll Stats
            </button>
            <textarea className="inp" style={{resize:"vertical",minHeight:"80px"}}
              placeholder="Optional: Describe your character's history, motivations, or the events that set them on the path of adventure. The DM will weave your past into the world…"
              value={char.backstory} onChange={e=>upd("backstory",e.target.value)}/>
          </>}

          <div className="cnav">
            {step>0
              ? <button className="btn-ghost" onClick={()=>setStep(s=>s-1)}>← Back</button>
              : <div/>
            }
            {step<3
              ? <button className="btn-gold" disabled={!canNext()} onClick={()=>setStep(s=>s+1)}>Continue →</button>
              : <button className="btn-gold" onClick={startGame}>Begin Adventure ⚔</button>
            }
          </div>
        </div>
      </div>
    </>
  );

  // ── GAME ─────────────────────────────────────────────────────────────────
  return (
    <>
      <style dangerouslySetInnerHTML={{__html:STYLES}}/>
      <div className="game">

        {/* Sidebar */}
        <div className={`sidebar${sidebarOpen?" open":""}`}>
          {/* Identity */}
          <div>
            <div className="cn-name">{char.name}</div>
            <div className="cn-sub">Level {char.level} {char.race} {char.class}</div>
            <div className="cn-sub">{char.background}</div>
          </div>

          {/* HP */}
          <div className="sec">
            <div className="sec-title">Hit Points</div>
            <div className="hp-bar">
              <div className="hp-fill" style={{width:`${hpPct}%`,background:hpCol}}/>
            </div>
            <div className="hp-txt" style={{color:hpCol}}>{char.hp} / {char.maxHp}</div>
            <div className="hp-adj">
              <button onClick={()=>adjustHp(-1)}>−</button>
              <input value={hpDelta} onChange={e=>setHpDelta(e.target.value.replace(/\D/g,""))} style={{color:"var(--text)"}}/>
              <button onClick={()=>adjustHp(1)}>+</button>
              <span style={{fontSize:".68rem",color:"var(--dim)"}}>adjust</span>
            </div>
          </div>

          {/* Stats */}
          <div className="sec">
            <div className="sec-title">Abilities</div>
            {STATS.map((s,i)=>(
              <div key={s} className="stat-r">
                <span className="sr-l">{s}</span>
                <span className="sr-v">{char.stats[i]}</span>
                <span className="sr-m">{fmt(char.stats[i])}</span>
              </div>
            ))}
          </div>

          {/* Inventory */}
          <div className="sec">
            <div className="sec-title">Inventory</div>
            {char.inventory.map((it,i)=><div key={i} className="inv-i">{it}</div>)}
            <div className="gp">⊙ {char.gold} gp</div>
          </div>

          {/* Dice */}
          <div className="sec">
            <div className="sec-title">Dice Roller</div>
            <div className="dice-row">
              {DICE.map(n=><button key={n} className="btn-sm" onClick={()=>rollDie(n)}>d{n}</button>)}
            </div>
            {lastRoll && <div className="roll-res">🎲 {lastRoll}</div>}
            <p className="roll-hint">Roll first, then send your action — the result is appended automatically.</p>
          </div>

          {/* Reset */}
          <div style={{marginTop:"auto",paddingTop:".75rem"}}>
            <button className="btn-ghost" style={{width:"100%",fontSize:".72rem"}}
              onClick={()=>{setPhase("title");setMessages([]);setHist([]);setSidebarOpen(false);}}>
              ✦ New Adventure
            </button>
          </div>
        </div>

        {/* Story area */}
        <div className="story">
          <div className="msgs">
            {messages.length===0&&!loading&&<div className="empty">✦ &nbsp; THE ADVENTURE BEGINS &nbsp; ✦</div>}

            {messages.map(msg=>(
              msg.role==="dm"
                ? <div key={msg.id} className="msg-dm">
                    <div className="dm-lbl">⚔ Dungeon Master</div>
                    {msg.content.split("\n").filter(l=>l.trim()).map((p,i)=><p key={i}>{p}</p>)}
                  </div>
                : <div key={msg.id} className="msg-pl">
                    <div className="pl-lbl">✦ {char.name}</div>
                    {msg.content}
                  </div>
            ))}

            {loading&&(
              <div className="msg-dm">
                <div className="dm-lbl">⚔ Dungeon Master</div>
                <div className="typing"><div className="dot"/><div className="dot"/><div className="dot"/></div>
              </div>
            )}
            <div ref={endRef}/>
          </div>

          {/* Input */}
          <div className="input-area">
            <div style={{flex:1}}>
              <textarea ref={taRef} style={{width:"100%"}}
                placeholder={lastRoll?`${lastRoll} — What do you do?`:"Describe your action, speak to an NPC, shape the world…"}
                value={input} onChange={e=>setInput(e.target.value)} onKeyDown={onKey} rows={2}/>
            </div>
            <button className="send" disabled={loading||!input.trim()} onClick={send}>
              {loading?"…":"Act →"}
            </button>
          </div>
        </div>

        {/* Mobile sidebar toggle */}
        <button className="mob-toggle" onClick={()=>setSidebarOpen(o=>!o)}>
          {sidebarOpen?"✕":"☰"}
        </button>
      </div>
    </>
  );
}