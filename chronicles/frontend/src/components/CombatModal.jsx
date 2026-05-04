import { useState, useEffect, useRef, useCallback } from 'react'
import { STYLES } from '../styles.js'
import * as api from '../api/client.js'

const SCHOOL_COLORS = {
    abjuration: '#7ec8e3',
    conjuration: '#b5a9f5',
    divination: '#f5e87e',
    enchantment: '#f5a9c8',
    evocation: '#f5a96a',
    illusion: '#a9f5d0',
    necromancy: '#b0f5a9',
    transmutation: '#f5cfa9',
}

const COMBAT_STYLES = `
${STYLES}

.combat-overlay {
  position: fixed; inset: 0; z-index: 100;
  background: rgba(0,0,0,.82);
  display: flex; align-items: center; justify-content: center;
  backdrop-filter: blur(3px);
}

.combat-modal {
  width: 95vw; max-width: 1100px;
  height: 92vh; max-height: 820px;
  background: #0d0e18;
  border: 1px solid #2a2d44;
  border-radius: 4px;
  display: flex; flex-direction: column;
  overflow: hidden;
  box-shadow: 0 0 80px rgba(0,0,0,.8), 0 0 30px rgba(200,150,42,.05);
}

.combat-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: .6rem 1rem;
  background: #0b0c15;
  border-bottom: 1px solid #1e2035;
  flex-shrink: 0;
}

.combat-title {
  font-family: 'Cinzel', serif; font-size: .75rem;
  letter-spacing: .2em; text-transform: uppercase;
  color: var(--gold);
}

.combat-round {
  font-family: 'Cinzel', serif; font-size: .68rem;
  color: var(--dim); letter-spacing: .12em;
}

/* ── Concentration banner ── */
.conc-banner {
  display: flex; align-items: center; justify-content: space-between;
  padding: .3rem 1rem;
  background: rgba(245,169,106,.06);
  border-bottom: 1px solid rgba(245,169,106,.2);
  font-family: 'Cinzel', serif; font-size: .6rem;
  color: #f5a96a; letter-spacing: .06em; flex-shrink: 0;
}
.conc-drop-btn {
  background: rgba(245,169,106,.12); border: 1px solid rgba(245,169,106,.3);
  color: #f5a96a; font-family: 'Cinzel', serif; font-size: .58rem;
  padding: .15rem .5rem; border-radius: 2px; cursor: pointer;
}

/* ── Spell slot pips (header) ── */
.slot-pips-bar {
  display: flex; gap: .6rem; align-items: center;
  padding: .3rem 1rem;
  background: #0c0d1a;
  border-bottom: 1px solid #161820;
  flex-shrink: 0; flex-wrap: wrap;
}
.slot-pip-group { display: flex; align-items: center; gap: .3rem; }
.slot-pip-label { font-family: 'Cinzel', serif; font-size: .55rem; color: #444; }
.slot-pip {
  width: 8px; height: 8px; border-radius: 50%;
  border: 1px solid #2a2d44;
  transition: background .2s, box-shadow .2s;
}
.slot-pip.full { box-shadow: 0 0 5px currentColor; }

.turn-order-bar {
  display: flex; gap: .5rem; align-items: center;
  padding: .55rem 1rem;
  background: #0c0d1a;
  border-bottom: 1px solid #1a1d2e;
  overflow-x: auto; flex-shrink: 0;
  scrollbar-width: thin; scrollbar-color: var(--gold) #0c0d1a;
}

.turn-chip {
  display: flex; align-items: center; gap: .3rem;
  padding: .25rem .6rem;
  border-radius: 2px; border: 1px solid #2a2d44;
  font-family: 'Cinzel', serif; font-size: .6rem;
  letter-spacing: .08em; white-space: nowrap;
  flex-shrink: 0; transition: all .2s;
  color: var(--dim); background: #13151f;
}

.turn-chip.active {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.08);
  box-shadow: 0 0 8px rgba(200,150,42,.2);
}

.turn-chip.enemy { border-color: #4a1a1a; color: #c06060; }
.turn-chip.enemy.active { border-color: var(--red); color: #ff8080; box-shadow: 0 0 8px rgba(200,60,60,.3); }
.turn-chip.ally { border-color: #1a3a2a; color: #60a080; }
.turn-chip.companion { border-color: #1a2a3a; color: #6080c0; }
.turn-chip.dead { opacity: .3; text-decoration: line-through; }
.turn-chip-icon { font-size: .7rem; }

.battlefield {
  display: flex; flex-direction: column;
  flex: 1; min-height: 0; padding: .75rem 1rem;
  justify-content: space-between;
}

.enemy-row {
  display: flex; gap: 1rem; justify-content: center;
  align-items: flex-start;
  flex: 1; min-height: 0;
  padding-top: .5rem;
}

.enemy-card {
  display: flex; flex-direction: column; align-items: center;
  gap: .3rem; cursor: pointer; transition: all .2s;
  position: relative; padding: .5rem;
  border-radius: 3px; border: 1px solid transparent;
  min-width: 80px;
}

.enemy-card:hover:not(.dead):not(.disabled) {
  border-color: rgba(200,150,42,.3);
  background: rgba(200,150,42,.04);
}

.enemy-card.targeted {
  border-color: var(--gold);
  background: rgba(200,150,42,.08);
  box-shadow: 0 0 16px rgba(200,150,42,.25);
}

.enemy-card.spell-targeting {
  border-color: rgba(181,169,245,.4);
  background: rgba(181,169,245,.05);
  animation: spell-pulse 1.2s ease-in-out infinite;
}

@keyframes spell-pulse {
  0%, 100% { border-color: rgba(181,169,245,.2); }
  50% { border-color: rgba(181,169,245,.7); box-shadow: 0 0 12px rgba(181,169,245,.25); }
}

.enemy-card.selecting:not(.dead) {
  animation: enemy-pulse 1.2s ease-in-out infinite;
}

@keyframes enemy-pulse {
  0%, 100% { border-color: rgba(200,150,42,.2); }
  50% { border-color: rgba(200,150,42,.6); box-shadow: 0 0 12px rgba(200,150,42,.2); }
}

.enemy-card.dead { opacity: .35; cursor: default; filter: grayscale(1); }
.enemy-card.shake { animation: shake .4s ease; }
@keyframes shake {
  0%, 100% { transform: translateX(0); }
  20% { transform: translateX(-6px); }
  40% { transform: translateX(6px); }
  60% { transform: translateX(-4px); }
  80% { transform: translateX(4px); }
}

.enemy-card.attack-out { animation: enemy-attack-down .35s ease; }
@keyframes enemy-attack-down {
  0%   { transform: translateY(0); }
  50%  { transform: translateY(18px); }
  100% { transform: translateY(0); }
}

.enemy-icon { width: 64px; height: 64px; position: relative; }
.enemy-icon svg { width: 100%; height: 100%; }

.bloodied-indicator {
  position: absolute; top: -4px; right: -4px;
  font-size: .75rem; filter: drop-shadow(0 0 4px rgba(255,0,0,.5));
}

.enemy-name {
  font-family: 'Cinzel', serif; font-size: .62rem;
  color: var(--dim); text-align: center;
  letter-spacing: .06em; max-width: 80px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}

.enemy-dead-mark { font-size: .7rem; color: var(--red); font-family: 'Cinzel', serif; }

.target-arrow {
  position: absolute; top: -22px; left: 50%;
  transform: translateX(-50%);
  font-size: 1rem; color: var(--gold);
  animation: bounce-arrow .6s ease-in-out infinite;
}
.target-arrow.spell { color: #b5a9f5; }
@keyframes bounce-arrow {
  0%, 100% { transform: translateX(-50%) translateY(0); }
  50% { transform: translateX(-50%) translateY(-4px); }
}

.player-row {
  display: flex; gap: .75rem; justify-content: center;
  align-items: flex-start; flex-wrap: wrap;
}

.combatant-card {
  background: #13151f; border: 1px solid #1e2035;
  border-radius: 3px; padding: .5rem .65rem;
  min-width: 100px; max-width: 130px;
  transition: all .2s;
}

.combatant-card.active-turn {
  border-color: var(--gold);
  background: rgba(200,150,42,.05);
  box-shadow: 0 0 12px rgba(200,150,42,.15);
}

.combatant-card.downed {
  border-color: var(--red);
  background: rgba(180,50,50,.05);
  opacity: .7;
}

.combatant-card.ally-card  { border-color: #1a3a2a; }
.combatant-card.comp-card  { border-color: #1a2a3a; }

.combatant-card.player-attacking { animation: player-attack-up .35s ease; }
@keyframes player-attack-up {
  0%   { transform: translateY(0); }
  50%  { transform: translateY(-18px); }
  100% { transform: translateY(0); }
}

.combatant-name {
  font-family: 'Cinzel', serif; font-size: .65rem;
  color: var(--goldl); letter-spacing: .06em;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  margin-bottom: .3rem;
}

.combatant-hp-bar {
  background: #0b0c15; border-radius: 1px; height: 4px; margin-bottom: .25rem;
}

.combatant-hp-fill {
  height: 100%; border-radius: 1px; transition: width .4s, background .4s;
}

.combatant-hp-text { font-size: .6rem; color: var(--dim); }
.combatant-class { font-size: .58rem; color: #3a3d55; margin-top: .15rem; font-family: 'Cinzel', serif; letter-spacing: .05em; }

.action-economy {
  display: flex; gap: .5rem; padding: .5rem 1rem;
  background: #0c0d1a; border-top: 1px solid #1a1d2e;
  flex-shrink: 0; align-items: center;
}

.economy-slot {
  display: flex; flex-direction: column; gap: .2rem; flex: 1;
}

.economy-label {
  font-family: 'Cinzel', serif; font-size: .55rem;
  letter-spacing: .12em; text-transform: uppercase; color: #3a3d55;
}

.economy-buttons { display: flex; gap: .35rem; flex-wrap: wrap; position: relative; }

.action-btn {
  background: #13151f; border: 1px solid #2a2d44;
  color: var(--dim); border-radius: 2px; cursor: pointer;
  font-family: 'Cinzel', serif; font-size: .62rem;
  letter-spacing: .06em; padding: .3rem .6rem;
  transition: all .15s; white-space: nowrap;
}

.action-btn:hover:not(:disabled) {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.06);
}

.action-btn:disabled { opacity: .3; cursor: not-allowed; }
.action-btn.used { border-color: #1e2035; color: #2a2d44; background: #0b0c15; cursor: not-allowed; }

.action-btn.selected {
  border-color: var(--gold); color: var(--goldl);
  background: rgba(200,150,42,.1);
}

.action-btn.spell-btn {
  border-color: #3a3050; color: #b5a9f5;
}
.action-btn.spell-btn:hover:not(:disabled) {
  border-color: #b5a9f5; color: #d0c8ff;
  background: rgba(181,169,245,.08);
}
.action-btn.spell-btn.selected {
  border-color: #b5a9f5; background: rgba(181,169,245,.12);
  color: #d0c8ff;
}

.action-btn.danger:hover:not(:disabled) {
  border-color: var(--red); color: #ff8080;
  background: rgba(180,50,50,.08);
}

.action-btn.end-turn {
  background: linear-gradient(135deg, #1a1f06, #2d3510);
  border-color: #4a5a20; color: #8a9a50;
  margin-left: auto;
}
.action-btn.end-turn:hover:not(:disabled) {
  border-color: #8a9a50; color: #b0c060;
  box-shadow: 0 0 12px rgba(100,130,30,.2);
}

/* ── Spell Picker Submenu ── */
.spell-picker {
  position: absolute; bottom: 100%; left: 0;
  background: #0a0b16; border: 1px solid #3a3050;
  border-radius: 4px; padding: .4rem;
  width: 320px; max-height: 340px;
  z-index: 20; display: flex; flex-direction: column;
  box-shadow: 0 -12px 40px rgba(0,0,0,.7);
}

.spell-picker-header {
  font-family: 'Cinzel', serif; font-size: .6rem; letter-spacing: .1em;
  color: #b5a9f5; padding: .2rem .3rem .4rem;
  border-bottom: 1px solid #2a2540;
  margin-bottom: .3rem; flex-shrink: 0;
}

.spell-picker-scroll {
  overflow-y: auto; flex: 1;
  scrollbar-width: thin; scrollbar-color: #3a3050 transparent;
}

.spell-group-label {
  font-family: 'Cinzel', serif; font-size: .52rem; letter-spacing: .1em;
  color: #4a4560; text-transform: uppercase;
  padding: .25rem .3rem .15rem;
}

.spell-row {
  display: flex; align-items: center; gap: .5rem;
  padding: .3rem .4rem; border-radius: 2px;
  cursor: pointer; transition: all .12s;
  border: 1px solid transparent;
}
.spell-row:hover { background: rgba(181,169,245,.07); border-color: rgba(181,169,245,.2); }
.spell-row.selected-spell { background: rgba(181,169,245,.12); border-color: #b5a9f5; }
.spell-row.no-slot { opacity: .35; cursor: not-allowed; }

.spell-row-glyph { font-size: .75rem; width: 16px; text-align: center; flex-shrink: 0; }
.spell-row-name { font-family: 'Cinzel', serif; font-size: .62rem; color: #d0c8e0; flex: 1; }
.spell-row-meta { font-size: .56rem; color: #5a5575; white-space: nowrap; }
.spell-row-conc { font-size: .55rem; color: #f5a96a; margin-left: .2rem; }

.spell-cast-bar {
  border-top: 1px solid #2a2540; padding: .4rem .3rem .2rem;
  display: flex; flex-direction: column; gap: .3rem; flex-shrink: 0;
}

.slot-level-row {
  display: flex; align-items: center; gap: .4rem; flex-wrap: wrap;
}

.slot-level-label { font-family: 'Cinzel', serif; font-size: .55rem; color: #5a5575; }

.slot-level-btn {
  font-family: 'Cinzel', serif; font-size: .58rem;
  padding: .15rem .45rem; border-radius: 2px;
  border: 1px solid #2a2540; background: transparent;
  color: #6a6585; cursor: pointer; transition: all .12s;
}
.slot-level-btn:hover:not(:disabled) { border-color: #b5a9f5; color: #d0c8ff; }
.slot-level-btn.chosen { border-color: #b5a9f5; color: #b5a9f5; background: rgba(181,169,245,.1); }
.slot-level-btn:disabled { opacity: .3; cursor: not-allowed; }

.cast-confirm-row { display: flex; gap: .4rem; align-items: center; }

.cast-now-btn {
  flex: 1; font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .08em; padding: .3rem .6rem; border-radius: 2px;
  border: 1px solid #b5a9f5;
  background: rgba(181,169,245,.1); color: #b5a9f5; cursor: pointer;
  transition: all .15s;
}
.cast-now-btn:hover:not(:disabled) { background: rgba(181,169,245,.2); }
.cast-now-btn:disabled { opacity: .35; cursor: not-allowed; }

.war-magic-note {
  font-size: .56rem; color: #f5a96a; font-style: italic;
}

/* ── Skills submenu ── */
.skills-submenu {
  position: absolute; bottom: 100%; left: 0;
  background: #0f1020; border: 1px solid #2a2d44;
  border-radius: 3px; padding: .4rem;
  min-width: 220px; z-index: 10;
  box-shadow: 0 -8px 24px rgba(0,0,0,.6);
}

.skill-btn {
  display: flex; justify-content: space-between; align-items: center;
  width: 100%; background: none; border: none;
  color: var(--dim); font-family: 'Cinzel', serif;
  font-size: .62rem; letter-spacing: .05em;
  padding: .3rem .5rem; cursor: pointer; border-radius: 2px;
  transition: all .15s; gap: .5rem;
}

.skill-btn:hover:not(:disabled) { background: rgba(200,150,42,.06); color: var(--goldl); }
.skill-btn:disabled { opacity: .3; cursor: not-allowed; }

.skill-uses {
  font-size: .58rem; color: var(--gold);
  border: 1px solid #2a2d44; border-radius: 2px;
  padding: .05rem .25rem; white-space: nowrap;
}
.skill-uses.empty { color: var(--red); border-color: #3a1a1a; }

.dice-section {
  background: #0b0c15; border-top: 1px solid #1a1d2e;
  padding: .6rem 1rem; flex-shrink: 0;
}

.dice-prompt {
  font-family: 'Cinzel', serif; font-size: .68rem;
  color: var(--goldl); letter-spacing: .08em; margin-bottom: .4rem;
}

.dice-prompt.spell-prompt { color: #b5a9f5; }

.dice-display { display: flex; gap: .5rem; align-items: center; flex-wrap: wrap; }

.die-face {
  background: #13151f; border: 1px solid #2a2d44;
  border-radius: 3px; width: 48px; height: 48px;
  display: flex; align-items: center; justify-content: center;
  font-family: 'Cinzel', serif; font-size: 1.2rem;
  color: var(--goldl); cursor: pointer;
  transition: all .15s; position: relative;
}

.die-face.spell-die { border-color: #3a3050; color: #b5a9f5; }
.die-face.spell-die:hover { border-color: #b5a9f5; box-shadow: 0 0 10px rgba(181,169,245,.2); }

.die-face:hover { border-color: var(--gold); box-shadow: 0 0 10px rgba(200,150,42,.2); }
.die-face.rolling { animation: die-roll .4s ease; }
.die-face.locked { border-color: var(--gold); background: rgba(200,150,42,.1); }

@keyframes die-roll {
  0%   { transform: rotate(0deg) scale(1); }
  25%  { transform: rotate(15deg) scale(1.1); }
  50%  { transform: rotate(-10deg) scale(.95); }
  75%  { transform: rotate(5deg) scale(1.05); }
  100% { transform: rotate(0deg) scale(1); }
}

.dice-label {
  font-size: .6rem; color: var(--dim); position: absolute;
  bottom: 2px; right: 4px; font-family: 'Cinzel', serif;
}

.dice-advantage-note { font-size: .65rem; color: var(--dim); font-style: italic; }

.dice-confirm-btn {
  background: linear-gradient(135deg, #1a1f06, #2d3510);
  border: 1px solid #4a5a20; color: #8a9a50;
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .08em; padding: .3rem .8rem;
  border-radius: 2px; cursor: pointer; transition: all .15s;
}
.dice-confirm-btn:hover { border-color: #8a9a50; color: #b0c060; }
.dice-confirm-btn:disabled { opacity: .4; cursor: not-allowed; }

.combat-log {
  background: #090a12; border-top: 1px solid #1a1d2e;
  padding: .5rem 1rem; max-height: 110px;
  overflow-y: auto; flex-shrink: 0;
  scrollbar-width: thin; scrollbar-color: #2a2d44 #090a12;
}

.log-entry {
  font-size: .68rem; color: var(--dim); line-height: 1.7;
  padding: .05rem 0; border-bottom: 1px solid #0f1020;
}

.log-entry:last-child { border-bottom: none; color: var(--text); }
.log-entry.hit   { color: #c07050; }
.log-entry.crit  { color: #e08040; font-weight: bold; }
.log-entry.miss  { color: #4a4d65; }
.log-entry.heal  { color: #50a070; }
.log-entry.death { color: var(--red); }
.log-entry.flee  { color: var(--gold); }
.log-entry.system { color: #5a5d80; font-style: italic; }
.log-entry.spell  { color: #b5a9f5; }

.initiative-screen {
  display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  flex: 1; gap: 1rem; padding: 2rem;
}

.initiative-title {
  font-family: 'Cinzel', serif; font-size: 1.4rem;
  color: var(--goldl); letter-spacing: .15em; text-align: center;
}

.initiative-sub {
  font-size: .8rem; color: var(--dim); text-align: center; line-height: 1.7;
}

.initiative-die {
  width: 96px; height: 96px;
  background: #13151f; border: 2px solid #2a2d44;
  border-radius: 6px; display: flex; align-items: center;
  justify-content: center; cursor: pointer;
  font-family: 'Cinzel', serif; font-size: 2.4rem;
  color: var(--goldl); transition: all .2s;
  box-shadow: 0 0 20px rgba(0,0,0,.5); position: relative;
}

.initiative-die:hover { border-color: var(--gold); box-shadow: 0 0 24px rgba(200,150,42,.2); }
.initiative-die.rolled { border-color: var(--gold); box-shadow: 0 0 30px rgba(200,150,42,.3); }
.initiative-die.rolling { animation: die-roll .5s ease; }
.initiative-bonus { font-size: .72rem; color: var(--gold); font-family: 'Cinzel', serif; }

.death-save-screen {
  display: flex; flex-direction: column;
  align-items: center; justify-content: center;
  flex: 1; gap: 1rem; padding: 2rem;
  background: rgba(120,0,0,.05);
}

.death-save-title {
  font-family: 'Cinzel', serif; font-size: 1.1rem;
  color: var(--red); letter-spacing: .15em;
}

.ds-pip {
  width: 14px; height: 14px; border-radius: 50%;
  border: 1px solid #2a2d44; background: #0b0c15; transition: all .3s;
}
.ds-pip.success { background: var(--grn); border-color: var(--grn); box-shadow: 0 0 6px rgba(50,180,80,.4); }
.ds-pip.failure { background: var(--red); border-color: var(--red); box-shadow: 0 0 6px rgba(180,50,50,.4); }
.ds-group-label { font-family: 'Cinzel', serif; font-size: .65rem; letter-spacing: .1em; margin-bottom: .4rem; }

.confirm-bar {
  display: flex; align-items: center; justify-content: space-between;
  padding: .4rem 1rem;
  background: rgba(200,150,42,.05);
  border-top: 1px solid rgba(200,150,42,.2);
  flex-shrink: 0;
}

.confirm-bar.spell-bar {
  background: rgba(181,169,245,.05);
  border-top-color: rgba(181,169,245,.2);
}

.confirm-text {
  font-family: 'Cinzel', serif; font-size: .68rem;
  color: var(--goldl); letter-spacing: .08em;
}
.confirm-text.spell-text { color: #b5a9f5; }

.confirm-ok {
  background: linear-gradient(135deg, #2a1f06, #4a3510);
  border: 1px solid var(--gold); color: var(--goldl);
  font-family: 'Cinzel', serif; font-size: .65rem;
  letter-spacing: .1em; padding: .3rem .9rem;
  border-radius: 2px; cursor: pointer; transition: all .15s;
}
.confirm-ok:hover { background: linear-gradient(135deg, #4a3510, #6a4e18); }

.confirm-ok.spell-ok {
  background: rgba(181,169,245,.1);
  border-color: #b5a9f5; color: #b5a9f5;
}
.confirm-ok.spell-ok:hover { background: rgba(181,169,245,.2); }

.confirm-cancel {
  background: none; border: 1px solid #2a2d44;
  color: var(--dim); font-family: 'Cinzel', serif;
  font-size: .62rem; padding: .3rem .7rem;
  border-radius: 2px; cursor: pointer; transition: all .15s;
  margin-right: .5rem;
}
.confirm-cancel:hover { border-color: var(--red); color: var(--red); }
`

// ─── Helpers ──────────────────────────────────────────────────────────────────

function EnemyIcon({ isAlly = false }) {
    return isAlly ? (
        <svg viewBox="0 0 64 64" fill="none">
            <circle cx="32" cy="20" r="12" stroke="#4a7a5a" strokeWidth="1.5" fill="#1a3a2a" />
            <path d="M14 56c0-9.94 8.06-18 18-18s18 8.06 18 18" stroke="#4a7a5a" strokeWidth="1.5" fill="none" />
            <path d="M26 34 L32 28 L38 34" stroke="#6a9a7a" strokeWidth="1.5" />
        </svg>
    ) : (
        <svg viewBox="0 0 64 64" fill="none">
            <path d="M32 8 L40 24 L56 26 L44 38 L47 54 L32 46 L17 54 L20 38 L8 26 L24 24 Z"
                stroke="#6a2a2a" strokeWidth="1.5" fill="#2a1010" />
            <circle cx="24" cy="26" r="3" fill="#c06060" opacity=".8" />
            <circle cx="40" cy="26" r="3" fill="#c06060" opacity=".8" />
            <path d="M24 38 Q32 44 40 38" stroke="#c06060" strokeWidth="1.5" fill="none" />
        </svg>
    )
}

function rollDie(sides) { return Math.floor(Math.random() * sides) + 1 }

function hpColor(pct) {
    if (pct > 55) return 'var(--grn)'
    if (pct > 25) return 'var(--amber)'
    return 'var(--red)'
}

function parseDamageDie(die) {
    const match = die?.match(/^(\d+)?d(\d+)$/)
    if (!match) return { count: 1, sides: 6 }
    return { count: parseInt(match[1] || '1'), sides: parseInt(match[2]) }
}

function schoolGlyph(school) {
    const glyphs = {
        abjuration: '🛡', conjuration: '✦', divination: '👁',
        enchantment: '♡', evocation: '⚡', illusion: '◈',
        necromancy: '☽', transmutation: '⟳',
    }
    return glyphs[school] || '✦'
}

function cantripDiceAtLevel(spell, charLevel) {
    if (charLevel >= 17 && spell.cantrip_dice_17) return spell.cantrip_dice_17
    if (charLevel >= 11 && spell.cantrip_dice_11) return spell.cantrip_dice_11
    if (charLevel >= 5 && spell.cantrip_dice_5) return spell.cantrip_dice_5
    return spell.damage_die_count || 1
}

function gfbSecondaryDamage(playerLevel, spellcastingMod) {
    // Primary extra fire dice (also used for secondary die count):
    //   L1-4:  0 dice, secondary = just modifier (min 0)
    //   L5-10: 1d8 primary, secondary = 1d8 + mod
    //   L11-16: 2d8 primary, secondary = 2d8 + mod
    //   L17+:  3d8 primary, secondary = 3d8 + mod
    const diceCount = playerLevel >= 17 ? 3
        : playerLevel >= 11 ? 2
            : playerLevel >= 5 ? 1
                : 0
    return { diceCount, mod: Math.max(0, spellcastingMod) }
}

function gfbPrimaryBonusDice(playerLevel) {
    if (playerLevel >= 17) return 3
    if (playerLevel >= 11) return 2
    if (playerLevel >= 5) return 1
    return 0
}

function upcastDice(spell, castLevel) {
    const base = spell.damage_die_count || 0
    const extra = (castLevel - spell.level) * (spell.slot_scale_dice || 0)
    return base + extra
}

// ─── Slot Pip Bar ─────────────────────────────────────────────────────────────

function SlotPipsBar({ slots }) {
    if (!slots || slots.length === 0) return null
    return (
        <div className="slot-pips-bar">
            {slots.map(s => (
                <div key={s.slot_level} className="slot-pip-group">
                    <span className="slot-pip-label">L{s.slot_level}</span>
                    {Array.from({ length: s.max_slots }, (_, i) => {
                        const full = i < s.current_slots
                        const hue = 30 + s.slot_level * 20
                        return (
                            <div
                                key={i}
                                className={`slot-pip${full ? ' full' : ''}`}
                                style={{
                                    background: full ? `hsl(${hue},70%,55%)` : 'transparent',
                                    borderColor: full ? `hsl(${hue},70%,40%)` : '#2a2d44',
                                    color: full ? `hsl(${hue},70%,55%)` : 'transparent',
                                }}
                            />
                        )
                    })}
                </div>
            ))}
        </div>
    )
}

// ─── Spell Picker ─────────────────────────────────────────────────────────────

function SpellPicker({ spells, slots, concentration, charLevel, onCast, onClose, warMagicMode }) {
    const [selected, setSelected] = useState(null)
    const [castLevel, setCastLevel] = useState(1)

    const cantrips = spells.filter(s => s.level === 0)
    const prepared = spells.filter(s => s.level > 0)

    // In War Magic mode only cantrips are allowed
    const showCantrips = cantrips
    const showPrepared = warMagicMode ? [] : prepared

    useEffect(() => {
        if (selected && selected.level > 0) {
            setCastLevel(Math.max(selected.level, 1))
        }
    }, [selected?.spell_id])

    const hasSlot = (level) => slots.some(s => s.slot_level === level && s.current_slots > 0)

    const availableSlots = selected && selected.level > 0
        ? slots.filter(s => s.slot_level >= selected.level && s.current_slots > 0)
        : []

    const canCast = selected && (
        selected.level === 0 || availableSlots.some(s => s.slot_level === castLevel)
    )

    const concWarning = selected?.concentration === 1 && concentration

    return (
        <div className="spell-picker" onClick={e => e.stopPropagation()}>
            <div className="spell-picker-header">
                {warMagicMode ? '⚡ War Magic — Choose Cantrip' : '✦ Cast a Spell'}
            </div>

            <div className="spell-picker-scroll">
                {showCantrips.length > 0 && (
                    <>
                        <div className="spell-group-label">Cantrips</div>
                        {showCantrips.map(s => (
                            <div
                                key={s.spell_id}
                                className={`spell-row${selected?.spell_id === s.spell_id ? ' selected-spell' : ''}`}
                                onClick={() => setSelected(s)}
                            >
                                <span className="spell-row-glyph" style={{ color: SCHOOL_COLORS[s.school] || '#b5a9f5' }}>
                                    {schoolGlyph(s.school)}
                                </span>
                                <span className="spell-row-name">{s.name}</span>
                                <span className="spell-row-meta">
                                    {s.damage_die_count ? `${cantripDiceAtLevel(s, charLevel)}${s.damage_die}` : s.casting_time?.replace('action', 'Actn')}
                                </span>
                            </div>
                        ))}
                    </>
                )}

                {showPrepared.length > 0 && (
                    <>
                        <div className="spell-group-label">Prepared Spells</div>
                        {showPrepared.map(s => {
                            const hasAnySlot = slots.some(sl => sl.slot_level >= s.level && sl.current_slots > 0)
                            return (
                                <div
                                    key={s.spell_id}
                                    className={`spell-row${selected?.spell_id === s.spell_id ? ' selected-spell' : ''}${!hasAnySlot ? ' no-slot' : ''}`}
                                    onClick={() => hasAnySlot && setSelected(s)}
                                >
                                    <span className="spell-row-glyph" style={{ color: SCHOOL_COLORS[s.school] || '#b5a9f5' }}>
                                        {schoolGlyph(s.school)}
                                    </span>
                                    <span className="spell-row-name">{s.name}</span>
                                    <span className="spell-row-meta">L{s.level}</span>
                                    {s.concentration === 1 && <span className="spell-row-conc">◉</span>}
                                </div>
                            )
                        })}
                    </>
                )}

                {showCantrips.length === 0 && showPrepared.length === 0 && (
                    <div style={{ padding: '.5rem', fontSize: '.62rem', color: '#4a4565', textAlign: 'center' }}>
                        {warMagicMode ? 'No cantrips known' : 'No spells available'}
                    </div>
                )}
            </div>

            {selected && (
                <div className="spell-cast-bar">
                    {/* Slot level selector for leveled spells */}
                    {selected.level > 0 && availableSlots.length > 0 && (
                        <div className="slot-level-row">
                            <span className="slot-level-label">Slot:</span>
                            {availableSlots.map(s => (
                                <button
                                    key={s.slot_level}
                                    className={`slot-level-btn${castLevel === s.slot_level ? ' chosen' : ''}`}
                                    onClick={() => setCastLevel(s.slot_level)}
                                >
                                    {s.slot_level} ({s.current_slots} left)
                                </button>
                            ))}
                        </div>
                    )}

                    {concWarning && (
                        <div style={{ fontSize: '.56rem', color: '#f5a96a' }}>
                            ⚠ Will drop {concentration.spell_name}
                        </div>
                    )}

                    <div className="cast-confirm-row">
                        <button
                            className="cast-now-btn"
                            disabled={!canCast}
                            onClick={() => onCast(selected, selected.level === 0 ? null : castLevel)}
                        >
                            ✦ {selected.level === 0 ? 'Cast Cantrip' : `Cast (Slot ${castLevel})`}
                        </button>
                        <button className="confirm-cancel" onClick={onClose}>✕</button>
                    </div>

                    {warMagicMode && (
                        <div className="war-magic-note">War Magic: replaces one attack</div>
                    )}
                </div>
            )}
        </div>
    )
}

// ─── Main Component ───────────────────────────────────────────────────────────

export default function CombatModal({
    campaignId, player, abilities,
    initiativeBonus = 0, hasAdvantage = false,
    onCombatEnd, onPlayerUpdate,
}) {
    const [phase, setPhase] = useState('initiative')
    const [combatState, setCombatState] = useState(null)

    const [diceValues, setDiceValues] = useState([])
    const [diceRolling, setDiceRolling] = useState([])
    const [lockedDie, setLockedDie] = useState(null)
    const [pendingDiceConfig, setPendingDiceConfig] = useState(null)

    const [initiativeRoll, setInitiativeRoll] = useState(null)
    const [initiativeRolling, setInitiativeRolling] = useState(false)
    const [initiativeAdvRolls, setInitiativeAdvRolls] = useState([null, null])

    const [actionUsed, setActionUsed] = useState(false)
    const [bonusActionUsed, setBonusActionUsed] = useState(false)
    const [attacksRemaining, setAttacksRemaining] = useState(1)
    const [actionSurgeAvailable, setActionSurgeAvailable] = useState(false)

    const [selectedAction, setSelectedAction] = useState(null)
    const [selectedTarget, setSelectedTarget] = useState(null)
    const [showSkillsMenu, setShowSkillsMenu] = useState(false)
    const [showSpellPicker, setShowSpellPicker] = useState(false)
    const [warMagicMode, setWarMagicMode] = useState(false)
    const [pendingSpell, setPendingSpell] = useState(null) // { spell, castLevel }
    const [isCrit, setIsCrit] = useState(false)

    // Spell state
    const [knownSpells, setKnownSpells] = useState([])
    const [spellSlots, setSpellSlots] = useState([])
    const [concentration, setConcentration] = useState(null)
    const [warBonds, setWarBonds] = useState([])

    const [deathSuccesses, setDeathSuccesses] = useState(0)
    const [deathFailures, setDeathFailures] = useState(0)

    const [shakingEnemy, setShakingEnemy] = useState(null)
    const [playerAttacking, setPlayerAttacking] = useState(false)
    const [attackingEnemyId, setAttackingEnemyId] = useState(null)

    const [log, setLog] = useState([])
    const logRef = useRef(null)
    const logData = useRef([])

    // ── EK detection ──────────────────────────────────────────────────────────
    const isEK = player?.subclass === 'Eldritch Knight'
    const hasWarMagic = isEK && (player?.level || 0) >= 7
    const hasImprovedWarMagic = isEK && (player?.level || 0) >= 18

    // ── Load ──────────────────────────────────────────────────────────────────
    useEffect(() => {
        api.getCombatState(campaignId).then(s => { if (s) setCombatState(s) }).catch(() => { })
    }, [campaignId])

    const loadSpellData = useCallback(async () => {
        if (!isEK) return
        try {
            const [knownRes, slotsRes, concRes, bondsRes] = await Promise.all([
                api.getKnownSpells(campaignId),
                api.getSpellSlots(campaignId),
                api.getConcentration(campaignId),
                api.getWarBonds(campaignId),
            ])
            setKnownSpells(knownRes.known_spells || [])
            setSpellSlots(slotsRes.spell_slots || [])
            setConcentration(concRes.concentration || null)
            setWarBonds(bondsRes.war_bonds || [])
        } catch (e) { /* non-fatal */ }
    }, [campaignId, isEK])

    useEffect(() => { loadSpellData() }, [loadSpellData])

    useEffect(() => {
        if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight
    }, [log])

    const addLog = (text, type = '') => {
        const entry = { text, type, id: Date.now() + Math.random() }
        logData.current = [...logData.current, entry]
        setLog(logData.current)
    }

    const refreshCombat = async () => {
        try {
            const s = await api.getCombatState(campaignId)
            if (s) setCombatState(s)
        } catch (e) { }
    }

    // ── Derived ────────────────────────────────────────────────────────────────
    const turnOrder = combatState?.turn_order || []
    const currentActor = combatState?.current_actor
    const enemies = (combatState?.enemies || []).filter(e => e.participant_type === 'enemy')
    const npcAllies = (combatState?.enemies || []).filter(e => e.participant_type === 'ally')
    const round = combatState?.round_number || 1
    const isPlayerTurn = currentActor?.participant_type === 'player'
    const canAct = phase === 'player_turn' && isPlayerTurn

    const totalSlotsLeft = spellSlots.reduce((a, s) => a + s.current_slots, 0)

    // Spells available: only action-cast ones in combat
    const combatSpells = knownSpells.filter(s =>
        s.casting_time === 'action' || s.casting_time === 'bonus_action'
    )
    const actionSpells = combatSpells.filter(s => s.casting_time === 'action')
    const bonusSpells = combatSpells.filter(s => s.casting_time === 'bonus_action')

    // ── Initiative ─────────────────────────────────────────────────────────────
    const rollInitiative = () => {
        setInitiativeRolling(true)
        if (hasAdvantage) {
            const r1 = rollDie(20), r2 = rollDie(20)
            setInitiativeAdvRolls([r1, r2])
            setTimeout(() => setInitiativeRolling(false), 500)
        } else {
            const r = rollDie(20)
            setTimeout(() => { setInitiativeRoll(r); setInitiativeRolling(false) }, 500)
        }
    }

    const chooseInitiativeAdvantage = (idx) => setInitiativeRoll(initiativeAdvRolls[idx])

    const confirmInitiative = async () => {
        if (initiativeRoll === null) return
        try {
            await api.submitInitiative(campaignId, initiativeRoll, hasAdvantage ? initiativeAdvRolls : null)
            addLog(`Initiative: ${player.name} rolls ${initiativeRoll} + ${initiativeBonus} = ${initiativeRoll + initiativeBonus}`, 'system')

            const freshState = await api.getCombatState(campaignId)
            setCombatState(freshState)
            const actor = freshState?.current_actor

            if (actor?.participant_type === 'player') {
                setActionUsed(false); setBonusActionUsed(false)
                setAttacksRemaining(freshState?.action_economy?.attacks_remaining || 1)
                setActionSurgeAvailable(freshState?.action_economy?.action_surge_available || false)
                addLog(`--- Round 1 — ${player.name}'s turn ---`, 'system')
                setPhase('player_turn')
            } else {
                addLog('--- Round 1 begins ---', 'system')
                setPhase('enemy_turns')
                try {
                    const result = await api.processInitialTurns(campaignId)
                    const turnResults = result.turn_results || []
                    for (let i = 0; i < turnResults.length; i++) {
                        await new Promise(r => setTimeout(r, 1600))
                        const t = turnResults[i]
                        addLog(t.text, t.hit ? 'hit' : 'miss')
                        if (t.player_downed) {
                            await refreshCombat()
                            if (onPlayerUpdate) await onPlayerUpdate()
                            setDeathSuccesses(0); setDeathFailures(0)
                            setPhase('death_saves'); return
                        }
                        if (t.combat_ended) { endCombatVictory(); return }
                    }
                } catch (e) { console.error('Failed to process initial turns:', e) }

                await refreshCombat()
                if (onPlayerUpdate) await onPlayerUpdate()
                const freshState2 = await api.getCombatState(campaignId)
                setCombatState(freshState2)
                const livingEnemies = (freshState2?.enemies || []).filter(e => e.participant_type === 'enemy' && e.is_alive)
                if (livingEnemies.length === 0) { endCombatVictory(); return }
                setActionUsed(false); setBonusActionUsed(false)
                addLog(`--- ${player.name}'s turn ---`, 'system')
                setPhase('player_turn')
            }
        } catch (e) { console.error('Initiative failed:', e) }
    }

    // ── Weapon attack flow ─────────────────────────────────────────────────────
    const startAttack = () => {
        setSelectedAction('attack')
        setSelectedTarget(null)
        setShowSkillsMenu(false)
        setShowSpellPicker(false)
    }

    const selectTarget = (enemyId) => {
        if (selectedAction !== 'attack' && selectedAction !== 'spell_target') return
        setSelectedTarget(enemyId)
    }

    const confirmAttack = async () => {
        if (!selectedTarget) return
        startDiceRoll({
            count: hasAdvantage ? 2 : 1,
            sides: 20,
            label: 'Attack Roll',
            isAdvantage: hasAdvantage,
            onConfirm: async (rolls, chosen) => {
                const roll = chosen ?? rolls[0]
                setPlayerAttacking(true)
                setTimeout(() => setPlayerAttacking(false), 400)
                try {
                    const result = await api.resolveAttack(campaignId, selectedTarget, roll)
                    if (result.hit) {
                        addLog(result.is_crit
                            ? `${player.name} attacks ${result.target_name} — Critical Hit! (${roll})`
                            : `${player.name} attacks ${result.target_name} — Hit! (${roll + result.attack_bonus} vs AC ${result.enemy_ac})`,
                            result.is_crit ? 'crit' : 'hit')
                        setIsCrit(result.is_crit)
                        const { count, sides } = parseDamageDie(result.damage_die)
                        startDiceRoll({
                            count: result.is_crit ? count * 2 : count,
                            sides,
                            label: result.is_crit ? 'Damage (CRIT — roll twice!)' : `Damage (${result.damage_die})`,
                            isAdvantage: false,
                            onConfirm: async (dmgRolls) => {
                                await confirmDamage(dmgRolls, result.is_crit, result.target_name, result.weapon_name)
                            }
                        })
                    } else {
                        addLog(`${player.name} attacks ${result.target_name} — Miss (${roll + result.attack_bonus} vs AC ${result.enemy_ac})`, 'miss')
                        setShakingEnemy(selectedTarget)
                        setTimeout(() => setShakingEnemy(null), 500)
                        finishAttack()
                    }
                } catch (e) { console.error('Attack failed:', e); cancelAction() }
            }
        })
    }

    const confirmDamage = async (rolls, crit, targetName, weaponName) => {
        try {
            const result = await api.resolveDamage(campaignId, rolls, crit)
            addLog(`${player.name} deals ${result.damage_dealt} damage to ${targetName} with ${weaponName}${result.enemy_dead ? ' — falls!' : ''}`,
                result.enemy_dead ? 'crit' : 'hit')
            setShakingEnemy(selectedTarget)
            setTimeout(() => setShakingEnemy(null), 500)
            await refreshCombat()
            if (result.all_enemies_defeated) { endCombatVictory(); return }
            if (result.can_attack_again) {
                setAttacksRemaining(r => r - 1)
                setSelectedTarget(null)
                setSelectedAction('attack')
                setPhase('player_turn'); return
            }
            finishAttack()
        } catch (e) { console.error('Damage failed:', e); cancelAction() }
    }

    const finishAttack = () => {
        setActionUsed(true)
        setAttacksRemaining(0)
        setSelectedAction(null)
        setSelectedTarget(null)
        setPhase('player_turn')
    }

    // ── Spell casting in combat ────────────────────────────────────────────────
    const openSpellPicker = (warMagic = false) => {
        setWarMagicMode(warMagic)
        setShowSpellPicker(true)
        setShowSkillsMenu(false)
        setSelectedAction(warMagic ? 'war_magic' : 'spell')
    }

    const handleSpellSelected = async (spell, castLevel) => {
        setShowSpellPicker(false)
        setPendingSpell({ spell, castLevel })

        // Check concentration conflict
        if (spell.concentration === 1 && concentration) {
            const ok = window.confirm(`Casting ${spell.name} will drop concentration on ${concentration.spell_name}. Continue?`)
            if (!ok) { cancelAction(); return }
        }

        // Expend slot via API
        try {
            const castRes = await api.castSpell(campaignId, spell.spell_id, {
                slotLevel: castLevel,
                dropConcentration: spell.concentration === 1 && !!concentration,
            })
            if (castRes.requires_confirmation) {
                const ok = window.confirm(castRes.message)
                if (!ok) { cancelAction(); return }
                await api.castSpell(campaignId, spell.spell_id, { slotLevel: castLevel, dropConcentration: true })
            }
            // Update local slot state
            setSpellSlots(castRes.spell_slots || spellSlots)
            if (spell.concentration === 1) setConcentration({ spell_id: spell.spell_id, spell_name: spell.name })
            else if (castRes.concentration) setConcentration(castRes.concentration)
        } catch (e) {
            addLog(`Failed to cast ${spell.name}: ${e.message}`, 'miss')
            cancelAction(); return
        }

        // Now resolve the spell effect
        if (!spell.damage_die || !spell.damage_die_count) {
            // Utility/concentration spell — no damage to roll
            addLog(`${player.name} casts ${spell.name}${castLevel ? ` (level ${castLevel} slot)` : ''}`, 'spell')
            finishSpellAction(spell)
            return
        }

        const diceCount = spell.level === 0
            ? cantripDiceAtLevel(spell, player.level || 1)
            : upcastDice(spell, castLevel || spell.level)

        const { sides } = parseDamageDie(spell.damage_die)

        // Attack roll needed?
        if (spell.attack_type === 'ranged_spell' || spell.attack_type === 'melee_spell') {
            // Need to pick a target first
            setSelectedAction('spell_target')
            setPhase('player_turn')
            // Store spell info for after target selection
            setPendingSpell({ spell, castLevel, diceCount, sides })
            addLog(`${player.name} casts ${spell.name} — choose a target`, 'spell')
            return
        }

        // Save spell — roll damage directly, apply to target (or all for AoE)
        if (spell.save_type) {
            if (spell.target_type === 'single') {
                setSelectedAction('spell_target_save')
                setPhase('player_turn')
                setPendingSpell({ spell, castLevel, diceCount, sides })
                addLog(`${player.name} casts ${spell.name} — choose a target`, 'spell')
                return
            }
            // AoE save: roll damage, model handles the save narration
            // AoE save: log damage, let model narrate saves
            startDiceRoll({
                count: diceCount, sides,
                label: `${spell.name} — ${diceCount}d${sides} ${spell.damage_type} (${spell.save_type?.toUpperCase()} save, half on success)`,
                isAdvantage: false,
                isSpell: true,
                onConfirm: async (rolls) => {
                    const livingEnemies = enemies.filter(e => e.is_alive)
                    let allDefeated = false
                    for (const enemy of livingEnemies) {
                        try {
                            await api.setCombatTarget(campaignId, enemy.id)
                            const result = await api.resolveDamage(campaignId, rolls, false)
                            addLog(`${spell.name} hits ${enemy.name} for ${result.damage_dealt} ${spell.damage_type} (${spell.save_type?.toUpperCase()} save for half)${result.enemy_dead ? ' — falls!' : ''}`, 'spell')
                            setShakingEnemy(enemy.id)
                            setTimeout(() => setShakingEnemy(null), 500)
                            if (result.all_enemies_defeated) { allDefeated = true; break }
                        } catch (e) { console.error(e) }
                    }
                    await refreshCombat()
                    if (allDefeated) { endCombatVictory(); return }
                    finishSpellAction(spell)
                }
            })
            return
        }

        // No attack, no save (self buff etc.) — already handled above
        addLog(`${player.name} casts ${spell.name}`, 'spell')
        finishSpellAction(spell)
    }

    const confirmSpellAttack = async () => {
        if (!selectedTarget || !pendingSpell) return
        const { spell, castLevel, diceCount, sides } = pendingSpell

        const isGFB = spell.spell_id === 'spell_green_flame_blade'
        // INT modifier for EK (the only class that gets GFB via class features)
        const intMod = Math.floor((player.int - 10) / 2)

        startDiceRoll({
            count: 1, sides: 20,
            label: `${spell.name} — Spell Attack Roll`,
            isAdvantage: false,
            isSpell: true,
            onConfirm: async (rolls) => {
                const roll = rolls[0]
                setPlayerAttacking(true)
                setTimeout(() => setPlayerAttacking(false), 400)
                try {
                    const result = await api.resolveAttack(campaignId, selectedTarget, roll)
                    if (result.hit) {
                        addLog(
                            `${player.name}'s ${spell.name} hits ${result.target_name}! `
                            + `(${roll + result.attack_bonus} vs AC ${result.enemy_ac})`,
                            'hit'
                        )
                        setIsCrit(result.is_crit)

                        // ── Primary damage: weapon + bonus fire dice ──────────────
                        // For GFB: weapon damage die comes from resolve_attack result,
                        // plus extra fire dice from the cantrip scaling.
                        const bonusDice = isGFB ? gfbPrimaryBonusDice(player.level || 1) : 0
                        const totalDiceCount = result.is_crit
                            ? (diceCount + bonusDice) * 2
                            : diceCount + bonusDice

                        const damageLabel = isGFB && bonusDice > 0
                            ? `${spell.name} — Weapon + ${bonusDice}d8 Fire Damage${result.is_crit ? ' (CRIT)' : ''}`
                            : `${spell.name} Damage${result.is_crit ? ' (CRIT — roll twice!)' : ''} (${diceCount}d${sides})`

                        startDiceRoll({
                            count: totalDiceCount,
                            sides,
                            label: damageLabel,
                            isAdvantage: false,
                            isSpell: true,
                            onConfirm: async (dmgRolls) => {
                                // Apply primary damage to primary target
                                const result2 = await api.resolveDamage(campaignId, dmgRolls, result.is_crit)
                                setShakingEnemy(selectedTarget)
                                setTimeout(() => setShakingEnemy(null), 500)
                                addLog(
                                    `${spell.name} deals ${result2.damage_dealt} damage`
                                    + (isGFB && bonusDice > 0 ? ' (weapon + fire)' : '')
                                    + ` to ${result.target_name}`
                                    + (result2.enemy_dead ? ' — falls!' : ''),
                                    result2.enemy_dead ? 'crit' : 'hit'
                                )
                                await refreshCombat()
                                if (result2.all_enemies_defeated) { endCombatVictory(); return }

                                // ── GFB secondary target ──────────────────────────────
                                if (isGFB) {
                                    const { diceCount: secDice, mod: secMod } =
                                        gfbSecondaryDamage(player.level || 1, intMod)

                                    const livingOthers = enemies.filter(
                                        e => e.is_alive && e.id !== selectedTarget
                                    )

                                    if (livingOthers.length === 0) {
                                        // No valid secondary target — fire fizzles
                                        addLog(
                                            'Green fire finds no second target within 5 feet.',
                                            'spell'
                                        )
                                        finishSpellAction(spell)
                                        return
                                    }

                                    // If there's exactly one other living enemy it auto-targets;
                                    // if multiple, the player should pick — we use the first
                                    // for simplicity (combat UI doesn't currently support
                                    // mid-flow target changes). This can be upgraded later.
                                    const secondaryTarget = livingOthers[0]

                                    if (secDice === 0) {
                                        // L1-4: secondary takes modifier fire damage directly
                                        if (secMod > 0) {
                                            await api.setCombatTarget(campaignId, secondaryTarget.id)
                                            const secResult = await api.resolveDamage(
                                                campaignId,
                                                [secMod], // pass as a single "roll" of fixed value
                                                false
                                            )
                                            addLog(
                                                `Green fire leaps to ${secondaryTarget.name} `
                                                + `for ${secResult.damage_dealt} fire damage`
                                                + (secResult.enemy_dead ? ' — falls!' : ''),
                                                secResult.enemy_dead ? 'crit' : 'hit'
                                            )
                                            setShakingEnemy(secondaryTarget.id)
                                            setTimeout(() => setShakingEnemy(null), 500)
                                            await refreshCombat()
                                            if (secResult.all_enemies_defeated) {
                                                endCombatVictory(); return
                                            }
                                        } else {
                                            addLog(
                                                'Green fire leaps to a nearby creature but deals no damage (INT mod 0).',
                                                'spell'
                                            )
                                        }
                                        finishSpellAction(spell)
                                    } else {
                                        // L5+: roll secondary dice, then apply + mod
                                        startDiceRoll({
                                            count: secDice,
                                            sides: 8,
                                            label: `Green Flame — ${secDice}d8 + ${secMod} fire on ${secondaryTarget.name}`,
                                            isAdvantage: false,
                                            isSpell: true,
                                            onConfirm: async (secRolls) => {
                                                const secTotal = secRolls.reduce((a, b) => a + b, 0) + secMod
                                                await api.setCombatTarget(campaignId, secondaryTarget.id)
                                                const secResult = await api.resolveDamage(
                                                    campaignId,
                                                    [secTotal],
                                                    false
                                                )
                                                setShakingEnemy(secondaryTarget.id)
                                                setTimeout(() => setShakingEnemy(null), 500)
                                                addLog(
                                                    `Green fire leaps to ${secondaryTarget.name} `
                                                    + `for ${secResult.damage_dealt} fire damage`
                                                    + (secResult.enemy_dead ? ' — falls!' : ''),
                                                    secResult.enemy_dead ? 'crit' : 'hit'
                                                )
                                                await refreshCombat()
                                                if (secResult.all_enemies_defeated) {
                                                    endCombatVictory(); return
                                                }
                                                finishSpellAction(spell)
                                            }
                                        })
                                    }
                                } else {
                                    // Not GFB — normal spell finish
                                    finishSpellAction(spell)
                                }
                            }
                        })
                    } else {
                        addLog(
                            `${player.name}'s ${spell.name} misses ${result.target_name} `
                            + `(${roll + result.attack_bonus} vs AC ${result.enemy_ac})`,
                            'miss'
                        )
                        setShakingEnemy(selectedTarget)
                        setTimeout(() => setShakingEnemy(null), 500)
                        finishSpellAction(spell)
                    }
                } catch (e) { console.error('Spell attack failed:', e); cancelAction() }
            }
        })
    }


    const confirmSpellSave = async () => {
        if (!selectedTarget || !pendingSpell) return
        const { spell, castLevel, diceCount, sides } = pendingSpell
        startDiceRoll({
            count: diceCount, sides,
            label: `${spell.name} Damage — ${diceCount}d${sides} ${spell.damage_type} (${spell.save_type?.toUpperCase()} save)`,
            isAdvantage: false,
            isSpell: true,
            onConfirm: async (rolls) => {
                try {
                    await api.setCombatTarget(campaignId, selectedTarget)
                    const result = await api.resolveDamage(campaignId, rolls, false)
                    setShakingEnemy(selectedTarget)
                    setTimeout(() => setShakingEnemy(null), 500)
                    addLog(`${spell.name}: ${result.damage_dealt} ${spell.damage_type} damage — target makes ${spell.save_type?.toUpperCase()} save (half on success)`, 'spell')
                    await refreshCombat()
                    if (result.all_enemies_defeated) { endCombatVictory(); return }
                    finishSpellAction(spell)
                } catch (e) { console.error(e); cancelAction() }
            }
        })
    }
    const finishSpellAction = (spell) => {
        const isWarMagic = warMagicMode
        setWarMagicMode(false)
        setSelectedAction(null)
        setSelectedTarget(null)
        setPendingSpell(null)

        if (isWarMagic) {
            // War Magic counts as one attack
            if (attacksRemaining <= 1) {
                setActionUsed(true)
                setAttacksRemaining(0)
            } else {
                setAttacksRemaining(r => r - 1)
            }
        } else {
            setActionUsed(true)
            setAttacksRemaining(0)
        }
        setPhase('player_turn')
        loadSpellData()
    }

    // ── Bonus action spell ─────────────────────────────────────────────────────
    const openBonusSpellPicker = () => {
        setWarMagicMode(false)
        setShowSpellPicker(true)
        setShowSkillsMenu(false)
        setSelectedAction('bonus_spell')
    }

    // ── War Bond Summon ────────────────────────────────────────────────────────
    const summonBondedWeapon = async (itemId, itemName) => {
        try {
            const res = await api.summonBondedWeapon(campaignId, itemId)
            addLog(res.message || `${itemName} flies to your hand!`, 'system')
            setBonusActionUsed(true)
        } catch (e) {
            addLog(`Failed to summon ${itemName}: ${e.message}`, 'miss')
        }
    }

    // ── Dice ───────────────────────────────────────────────────────────────────
    const startDiceRoll = (config) => {
        setDiceValues(Array.from({ length: config.count }, () => null))
        setDiceRolling(Array(config.count).fill(false))
        setLockedDie(null)
        setPendingDiceConfig(config)
        setPhase('rolling')
    }

    const rollSingleDie = (index) => {
        if (!pendingDiceConfig) return
        setDiceRolling(r => r.map((v, i) => i === index ? true : v))
        setTimeout(() => {
            const val = rollDie(pendingDiceConfig.sides)
            setDiceValues(v => v.map((x, i) => i === index ? val : x))
            setDiceRolling(r => r.map((v, i) => i === index ? false : v))
        }, 300)
    }

    const rollAllDice = () => {
        if (!pendingDiceConfig) return
        setDiceRolling(Array(pendingDiceConfig.count).fill(true))
        setTimeout(() => {
            const vals = Array.from({ length: pendingDiceConfig.count }, () => rollDie(pendingDiceConfig.sides))
            setDiceValues(vals)
            setDiceRolling(Array(pendingDiceConfig.count).fill(false))
        }, 350)
    }

    const confirmDiceRoll = () => {
        if (!pendingDiceConfig) return
        const allRolled = diceValues.every(v => v !== null)
        if (!allRolled) return
        if (pendingDiceConfig.isAdvantage && diceValues.length === 2) {
            if (lockedDie === null) return
            const chosen = diceValues[lockedDie]
            const config = pendingDiceConfig
            setPendingDiceConfig(null)
            setPhase('player_turn')
            config.onConfirm(diceValues, chosen)
        } else {
            const config = pendingDiceConfig
            setPendingDiceConfig(null)
            setPhase('player_turn')
            config.onConfirm(diceValues, null)
        }
    }

    // ── Skills ─────────────────────────────────────────────────────────────────
    const combatAbilities = (abilities || []).filter(ab => {
        const n = ab.name.toLowerCase()
        return n.includes('second wind') || n.includes('action surge') ||
            n.includes('indomitable') || n.includes('rage') || n.includes('cunning')
    })

    const useSkill = async (ability) => {
        setShowSkillsMenu(false)
        const name = ability.name.toLowerCase()
        if (name.includes('second wind')) {
            try {
                const result = await api.useCombatAbility(campaignId, 'second_wind')
                addLog(`${player.name} uses Second Wind and recovers ${result.healing} HP.`, 'heal')
                setBonusActionUsed(true)
                await refreshCombat()
                if (onPlayerUpdate) await onPlayerUpdate()
            } catch (e) { console.error(e) }
        } else if (name.includes('action surge')) {
            try {
                await api.useCombatAbility(campaignId, 'action_surge')
                addLog(`${player.name} activates Action Surge!`, 'system')
                setActionUsed(false)
                setActionSurgeAvailable(false)
            } catch (e) { console.error(e) }
        }
    }

    // ── Flee ───────────────────────────────────────────────────────────────────
    const startFlee = () => { setSelectedAction('flee'); setShowSkillsMenu(false); setShowSpellPicker(false) }

    const pickFleeSkill = (skill) => {
        setSelectedAction(null)
        startDiceRoll({
            count: 1, sides: 20,
            label: `${skill} Check (DC 15 to flee)`,
            isAdvantage: false,
            onConfirm: async (rolls) => {
                try {
                    const result = await api.fleeCombat(campaignId, rolls[0], skill)
                    addLog(result.text, result.success ? 'flee' : 'miss')
                    setPhase('fled')
                    setTimeout(() => onCombatEnd('fled', logData.current), 1500)
                    if (onPlayerUpdate) await onPlayerUpdate()
                } catch (e) { console.error(e) }
            }
        })
    }

    // ── End turn ───────────────────────────────────────────────────────────────
    const endTurn = async () => {
        setSelectedAction(null); setSelectedTarget(null)
        setShowSkillsMenu(false); setShowSpellPicker(false)
        setPhase('enemy_turns')
        addLog('--- Player turn ends ---', 'system')
        try {
            const result = await api.endCombatTurn(campaignId)
            const turnResults = result.turn_results || []
            for (let i = 0; i < turnResults.length; i++) {
                await new Promise(r => setTimeout(r, 1600))
                const t = turnResults[i]
                addLog(t.text, t.hit ? (t.damage ? 'hit' : 'system') : 'miss')
                if (t.player_downed) {
                    await refreshCombat()
                    if (onPlayerUpdate) await onPlayerUpdate()
                    setDeathSuccesses(0); setDeathFailures(0)
                    setPhase('death_saves'); return
                }
                if (t.combat_ended) { endCombatVictory(); return }
            }
            const freshState = await api.getCombatState(campaignId)
            setCombatState(freshState)
            const livingEnemies = (freshState?.enemies || []).filter(e => e.participant_type === 'enemy' && e.is_alive)
            if (livingEnemies.length === 0) { endCombatVictory(); return }
            if (onPlayerUpdate) await onPlayerUpdate()
            const actor = freshState?.current_actor
            if (actor?.participant_type === 'player') {
                setActionUsed(false); setBonusActionUsed(false)
                setAttacksRemaining(freshState?.action_economy?.attacks_remaining || 1)
                setActionSurgeAvailable(freshState?.action_economy?.action_surge_available || false)
                addLog(`--- Round ${freshState?.round_number} — ${player.name}'s turn ---`, 'system')
                setPhase('player_turn')
                await loadSpellData()
            } else {
                setPhase('player_turn')
            }
        } catch (e) { console.error('End turn failed:', e); setPhase('player_turn') }
    }

    // ── Death saves ─────────────────────────────────────────────────────────────
    const rollDeathSave = () => {
        startDiceRoll({
            count: 1, sides: 20,
            label: 'Death Saving Throw (10+ = Success)',
            isAdvantage: false,
            onConfirm: async (rolls) => {
                const roll = rolls[0]
                if (roll === 20) {
                    addLog(`${player.name} rolls a Natural 20 — stabilizes with 1 HP!`, 'heal')
                    setDeathSuccesses(3)
                    if (onPlayerUpdate) await onPlayerUpdate()
                    await refreshCombat()
                    const freshState = await api.getCombatState(campaignId)
                    setCombatState(freshState)
                    setActionUsed(false); setBonusActionUsed(false)
                    setAttacksRemaining(freshState?.action_economy?.attacks_remaining || 1)
                    addLog(`${player.name} is back on their feet!`, 'heal')
                    setPhase('player_turn'); return
                }
                if (roll === 1) {
                    const nf = deathFailures + 2; setDeathFailures(nf)
                    addLog(`${player.name} rolls a 1 — two failures!`, 'death')
                    if (nf >= 3) { addLog(`${player.name} has died.`, 'death'); onCombatEnd('death', logData.current); return }
                    setPhase('death_saves'); return
                }
                if (roll >= 10) {
                    const ns = deathSuccesses + 1; setDeathSuccesses(ns)
                    addLog(`${player.name} succeeds on death save (${ns}/3).`, 'heal')
                    if (ns >= 3) {
                        addLog(`${player.name} stabilizes!`, 'heal')
                        if (onPlayerUpdate) await onPlayerUpdate()
                        setDeathSuccesses(0); setDeathFailures(0)
                        setPhase('enemy_turns')
                        try {
                            const result = await api.endCombatTurn(campaignId)
                            const turnResults = result.turn_results || []
                            for (let i = 0; i < turnResults.length; i++) {
                                await new Promise(r => setTimeout(r, 1600))
                                const t = turnResults[i]
                                addLog(t.text, t.hit ? 'hit' : 'miss')
                                if (t.player_downed) { setDeathSuccesses(0); setDeathFailures(0); setPhase('death_saves'); return }
                                if (t.combat_ended) { endCombatVictory(); return }
                            }
                        } catch (e) { console.error(e) }
                        await refreshCombat()
                        if (onPlayerUpdate) await onPlayerUpdate()
                        setActionUsed(false); setBonusActionUsed(false)
                        addLog(`--- ${player.name}'s turn ---`, 'system')
                        setPhase('player_turn'); return
                    }
                    setPhase('death_saves'); return
                }
                const nf = deathFailures + 1; setDeathFailures(nf)
                addLog(`${player.name} fails death save (${nf}/3).`, 'death')
                if (nf >= 3) { addLog(`${player.name} has died.`, 'death'); onCombatEnd('death', log); return }
                setPhase('death_saves')
            }
        })
    }

    // ── Victory ────────────────────────────────────────────────────────────────
    const endCombatVictory = async () => {
        try { await api.endCombat(campaignId) } catch (e) { }
        addLog('Victory! All enemies have been defeated.', 'system')
        setTimeout(() => onCombatEnd('victory', logData.current), 800)
    }

    const cancelAction = () => {
        setSelectedAction(null); setSelectedTarget(null)
        setShowSkillsMenu(false); setShowSpellPicker(false)
        setPendingDiceConfig(null); setPendingSpell(null)
        setWarMagicMode(false)
        setPhase('player_turn')
    }

    const isSpellTargeting = selectedAction === 'spell_target' || selectedAction === 'spell_target_save'
    const isSaveSpellTargeting = selectedAction === 'spell_target_save'

    // ─────────────────────────────────────────────────────────────────────────
    return (
        <>
            <style dangerouslySetInnerHTML={{ __html: COMBAT_STYLES }} />
            <div className="combat-overlay" onClick={() => { setShowSkillsMenu(false); setShowSpellPicker(false) }}>
                <div className="combat-modal" onClick={e => e.stopPropagation()}>

                    {/* Header */}
                    <div className="combat-header">
                        <div className="combat-title">⚔ Combat</div>
                        <div className="combat-round">
                            {phase === 'initiative' ? 'Roll for Initiative' :
                                phase === 'enemy_turns' ? 'Enemy Turn…' :
                                    phase === 'death_saves' ? '☠ Death Saving Throws' :
                                        `Round ${round}`}
                        </div>
                    </div>

                    {/* EK: Concentration banner */}
                    {isEK && concentration && (
                        <div className="conc-banner">
                            <span>◉ Concentrating on <strong>{concentration.spell_name}</strong></span>
                            <button className="conc-drop-btn" onClick={async () => {
                                await api.dropConcentration(campaignId)
                                setConcentration(null)
                                addLog(`${player.name} drops concentration.`, 'system')
                            }}>Drop</button>
                        </div>
                    )}

                    {/* EK: Slot pips bar */}
                    {isEK && spellSlots.length > 0 && (
                        <SlotPipsBar slots={spellSlots} />
                    )}

                    {/* Turn Order Bar */}
                    {phase !== 'initiative' && turnOrder.length > 0 && (
                        <div className="turn-order-bar">
                            {turnOrder.map((p, i) => (
                                <div key={p.id + i} className={[
                                    'turn-chip', p.participant_type,
                                    i === 0 ? 'active' : '',
                                    !p.is_alive ? 'dead' : ''
                                ].filter(Boolean).join(' ')}>
                                    <span className="turn-chip-icon">
                                        {p.participant_type === 'player' ? '⚔' :
                                            p.participant_type === 'enemy' ? '☠' : '🛡'}
                                    </span>
                                    {p.name}
                                </div>
                            ))}
                        </div>
                    )}

                    {/* Initiative Screen */}
                    {phase === 'initiative' && (
                        <div className="initiative-screen">
                            <div className="initiative-title">Roll for Initiative</div>
                            <div className="initiative-sub">
                                Roll a d20{initiativeBonus !== 0 ? ` + ${initiativeBonus} (DEX modifier)` : ''}.
                                {hasAdvantage && ' You have advantage — roll twice and pick the higher result.'}
                            </div>
                            {!hasAdvantage ? (
                                <>
                                    <div
                                        className={`initiative-die${initiativeRolling ? ' rolling' : ''}${initiativeRoll !== null ? ' rolled' : ''}`}
                                        onClick={rollInitiative}
                                    >
                                        {initiativeRoll !== null ? initiativeRoll : '?'}
                                        <span className="dice-label">d20</span>
                                    </div>
                                    {initiativeRoll !== null && (
                                        <div className="initiative-bonus">
                                            {initiativeRoll} + {initiativeBonus} = {initiativeRoll + initiativeBonus}
                                        </div>
                                    )}
                                </>
                            ) : (
                                <>
                                    <div style={{ display: 'flex', gap: '1rem' }}>
                                        {[0, 1].map(i => (
                                            <div key={i}
                                                className={`initiative-die${initiativeRolling ? ' rolling' : ''}${initiativeAdvRolls[i] !== null ? ' rolled' : ''}${initiativeRoll === initiativeAdvRolls[i] && initiativeAdvRolls[i] !== null ? ' locked' : ''}`}
                                                onClick={() => initiativeAdvRolls[i] !== null ? chooseInitiativeAdvantage(i) : rollInitiative()}
                                            >
                                                {initiativeAdvRolls[i] ?? '?'}
                                                <span className="dice-label">d20</span>
                                            </div>
                                        ))}
                                    </div>
                                    {initiativeAdvRolls[0] !== null && (
                                        <div className="initiative-sub" style={{ marginTop: 0 }}>Click the die you want to use</div>
                                    )}
                                    {initiativeRoll !== null && (
                                        <div className="initiative-bonus">
                                            Using {initiativeRoll} + {initiativeBonus} = {initiativeRoll + initiativeBonus}
                                        </div>
                                    )}
                                </>
                            )}
                            <button className="btn-gold" disabled={initiativeRoll === null} onClick={confirmInitiative}>
                                Enter Combat
                            </button>
                        </div>
                    )}

                    {/* Death Saves Screen */}
                    {phase === 'death_saves' && (
                        <div className="death-save-screen">
                            <div className="death-save-title">☠ Death Saving Throws</div>
                            <div style={{ display: 'flex', gap: '1.5rem' }}>
                                <div>
                                    <div className="ds-group-label" style={{ color: 'var(--grn)' }}>Successes</div>
                                    <div style={{ display: 'flex', gap: '.4rem' }}>
                                        {[0, 1, 2].map(i => <div key={i} className={`ds-pip${i < deathSuccesses ? ' success' : ''}`} />)}
                                    </div>
                                </div>
                                <div>
                                    <div className="ds-group-label" style={{ color: 'var(--red)' }}>Failures</div>
                                    <div style={{ display: 'flex', gap: '.4rem' }}>
                                        {[0, 1, 2].map(i => <div key={i} className={`ds-pip${i < deathFailures ? ' failure' : ''}`} />)}
                                    </div>
                                </div>
                            </div>
                            <button className="btn-gold" onClick={rollDeathSave}>Roll Death Save</button>
                        </div>
                    )}

                    {/* Battlefield */}
                    {phase !== 'initiative' && phase !== 'death_saves' && (
                        <div className="battlefield">

                            {/* Enemy Row */}
                            <div className="enemy-row">
                                {enemies.map(enemy => (
                                    <div
                                        key={enemy.id}
                                        className={[
                                            'enemy-card',
                                            !enemy.is_alive ? 'dead' : '',
                                            selectedTarget === enemy.id ? 'targeted' : '',
                                            selectedAction === 'attack' && enemy.is_alive && !selectedTarget ? 'selecting' : '',
                                            isSpellTargeting && enemy.is_alive && !selectedTarget ? 'spell-targeting' : '',
                                            shakingEnemy === enemy.id ? 'shake' : '',
                                            attackingEnemyId === enemy.id ? 'attack-out' : '',
                                        ].filter(Boolean).join(' ')}
                                        onClick={() => {
                                            if (!enemy.is_alive) return
                                            if (selectedAction === 'attack') selectTarget(enemy.id)
                                            else if (isSpellTargeting) setSelectedTarget(enemy.id)
                                        }}
                                    >
                                        {selectedTarget === enemy.id && (
                                            <div className={`target-arrow${isSpellTargeting ? ' spell' : ''}`}>▼</div>
                                        )}
                                        <div className="enemy-icon">
                                            <EnemyIcon isAlly={false} />
                                            {enemy.is_bloodied && enemy.is_alive && (
                                                <div className="bloodied-indicator">🩸</div>
                                            )}
                                        </div>
                                        {enemy.is_alive
                                            ? <div className="enemy-name">{enemy.name}</div>
                                            : <div className="enemy-dead-mark">✝ {enemy.name}</div>
                                        }
                                    </div>
                                ))}
                            </div>

                            {/* Player/Ally Row */}
                            <div className="player-row">
                                <div className={[
                                    'combatant-card',
                                    isPlayerTurn && phase === 'player_turn' ? 'active-turn' : '',
                                    player.current_hp === 0 ? 'downed' : '',
                                    playerAttacking ? 'player-attacking' : '',
                                ].filter(Boolean).join(' ')}>
                                    <div className="combatant-name">{player.name}</div>
                                    <div className="combatant-hp-bar">
                                        <div className="combatant-hp-fill" style={{
                                            width: `${Math.max(0, (player.current_hp / player.max_hp) * 100)}%`,
                                            background: hpColor(player.current_hp / player.max_hp * 100)
                                        }} />
                                    </div>
                                    <div className="combatant-hp-text">{player.current_hp}/{player.max_hp} HP</div>
                                    <div className="combatant-class">{player.class}{isEK ? ' · EK' : ''}</div>
                                    {isEK && totalSlotsLeft > 0 && (
                                        <div style={{ fontSize: '.55rem', color: '#b5a9f5', marginTop: '.15rem' }}>
                                            ✦ {totalSlotsLeft} slot{totalSlotsLeft !== 1 ? 's' : ''}
                                        </div>
                                    )}
                                </div>

                                {npcAllies.map(ally => (
                                    <div key={ally.id} className={[
                                        'combatant-card ally-card',
                                        !ally.is_alive ? 'downed' : '',
                                        currentActor?.id === ally.id ? 'active-turn' : '',
                                    ].filter(Boolean).join(' ')}>
                                        <div className="combatant-name">{ally.name}</div>
                                        <div className="combatant-hp-bar">
                                            <div className="combatant-hp-fill" style={{
                                                width: `${Math.max(0, (ally.current_hp / ally.max_hp) * 100)}%`,
                                                background: hpColor(ally.current_hp / ally.max_hp * 100)
                                            }} />
                                        </div>
                                        <div className="combatant-hp-text">{ally.current_hp}/{ally.max_hp} HP</div>
                                        <div className="combatant-class">Ally</div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* Dice Rolling */}
                    {phase === 'rolling' && pendingDiceConfig && (
                        <div className="dice-section">
                            <div className={`dice-prompt${pendingDiceConfig.isSpell ? ' spell-prompt' : ''}`}>
                                {pendingDiceConfig.label}
                            </div>
                            <div className="dice-display">
                                {diceValues.map((val, i) => (
                                    <div
                                        key={i}
                                        className={[
                                            'die-face',
                                            pendingDiceConfig.isSpell ? 'spell-die' : '',
                                            diceRolling[i] ? 'rolling' : '',
                                            pendingDiceConfig.isAdvantage && lockedDie === i ? 'locked' : ''
                                        ].filter(Boolean).join(' ')}
                                        onClick={() => {
                                            if (pendingDiceConfig.isAdvantage && val !== null) setLockedDie(i)
                                            else rollSingleDie(i)
                                        }}
                                    >
                                        {val ?? '?'}
                                        <span className="dice-label">d{pendingDiceConfig.sides}</span>
                                    </div>
                                ))}
                                {diceValues.some(v => v === null) && (
                                    <button className="dice-confirm-btn" onClick={rollAllDice}>Roll All</button>
                                )}
                                {diceValues.every(v => v !== null) && (
                                    <>
                                        {pendingDiceConfig.isAdvantage && (
                                            <span className="dice-advantage-note">
                                                {lockedDie !== null ? `Using ${diceValues[lockedDie]}` : 'Click die to use'}
                                            </span>
                                        )}
                                        <button
                                            className="dice-confirm-btn"
                                            disabled={pendingDiceConfig.isAdvantage && lockedDie === null}
                                            onClick={confirmDiceRoll}
                                        >Confirm</button>
                                    </>
                                )}
                                <button className="action-btn danger" style={{ marginLeft: 'auto' }} onClick={cancelAction}>
                                    Cancel
                                </button>
                            </div>
                        </div>
                    )}

                    {/* Attack confirm bar */}
                    {phase === 'player_turn' && selectedTarget && selectedAction === 'attack' && (
                        <div className="confirm-bar">
                            <div className="confirm-text">
                                ⚔ Attack {enemies.find(e => e.id === selectedTarget)?.name}
                            </div>
                            <div style={{ display: 'flex', gap: '.4rem' }}>
                                <button className="confirm-cancel" onClick={cancelAction}>Cancel</button>
                                <button className="confirm-ok" onClick={confirmAttack}>Attack</button>
                            </div>
                        </div>
                    )}

                    {/* Spell attack confirm bar */}
                    {phase === 'player_turn' && selectedTarget && selectedAction === 'spell_target' && pendingSpell && (
                        <div className="confirm-bar spell-bar">
                            <div className="confirm-text spell-text">
                                ✦ {pendingSpell.spell.name} → {enemies.find(e => e.id === selectedTarget)?.name}
                            </div>
                            <div style={{ display: 'flex', gap: '.4rem' }}>
                                <button className="confirm-cancel" onClick={cancelAction}>Cancel</button>
                                <button className="confirm-ok spell-ok" onClick={confirmSpellAttack}>
                                    Roll Attack
                                </button>
                            </div>
                        </div>
                    )}

                    {/* Spell save confirm bar */}
                    {phase === 'player_turn' && selectedTarget && selectedAction === 'spell_target_save' && pendingSpell && (
                        <div className="confirm-bar spell-bar">
                            <div className="confirm-text spell-text">
                                ✦ {pendingSpell.spell.name} → {enemies.find(e => e.id === selectedTarget)?.name}
                                <span style={{ color: '#f5a96a', fontSize: '.6rem', marginLeft: '.5rem' }}>
                                    ({pendingSpell.spell.save_type?.toUpperCase()} save)
                                </span>
                            </div>
                            <div style={{ display: 'flex', gap: '.4rem' }}>
                                <button className="confirm-cancel" onClick={cancelAction}>Cancel</button>
                                <button className="confirm-ok spell-ok" onClick={confirmSpellSave}>
                                    Roll Damage
                                </button>
                            </div>
                        </div>
                    )}

                    {/* Flee skill picker */}
                    {phase === 'player_turn' && selectedAction === 'flee' && (
                        <div className="confirm-bar">
                            <div className="confirm-text">Choose skill to flee:</div>
                            <div style={{ display: 'flex', gap: '.4rem' }}>
                                <button className="confirm-ok" onClick={() => pickFleeSkill('Athletics')}>Athletics</button>
                                <button className="confirm-ok" onClick={() => pickFleeSkill('Acrobatics')}>Acrobatics</button>
                                <button className="confirm-cancel" onClick={cancelAction}>Cancel</button>
                            </div>
                        </div>
                    )}

                    {/* Action Menu */}
                    {phase === 'player_turn' && !selectedTarget && selectedAction !== 'flee' &&
                        selectedAction !== 'spell_target' && selectedAction !== 'spell_target_save' && (
                            <div className="action-economy">
                                {/* ACTION */}
                                <div className="economy-slot">
                                    <div className="economy-label">Action</div>
                                    <div className="economy-buttons">
                                        <button
                                            className={`action-btn${actionUsed ? ' used' : ''}${selectedAction === 'attack' ? ' selected' : ''}`}
                                            disabled={!canAct || actionUsed}
                                            onClick={startAttack}
                                        >⚔ Attack</button>

                                        {/* War Magic: replace attack with cantrip (level 7+ EK) */}
                                        {isEK && hasWarMagic && !actionUsed && attacksRemaining > 0 && (
                                            <div style={{ position: 'relative' }}>
                                                <button
                                                    className={`action-btn spell-btn${warMagicMode && showSpellPicker ? ' selected' : ''}`}
                                                    disabled={!canAct}
                                                    onClick={() => openSpellPicker(true)}
                                                >⚡ War Magic</button>
                                                {showSpellPicker && warMagicMode && (
                                                    <SpellPicker
                                                        spells={knownSpells.filter(s => s.level === 0 && s.casting_time === 'action')}
                                                        slots={spellSlots}
                                                        concentration={concentration}
                                                        charLevel={player.level || 1}
                                                        onCast={handleSpellSelected}
                                                        onClose={() => { setShowSpellPicker(false); setSelectedAction(null) }}
                                                        warMagicMode={true}
                                                    />
                                                )}
                                            </div>
                                        )}

                                        {/* Full spell action (non-War-Magic) */}
                                        {isEK && (
                                            <div style={{ position: 'relative' }}>
                                                <button
                                                    className={`action-btn spell-btn${selectedAction === 'spell' && showSpellPicker ? ' selected' : ''}${actionUsed ? ' used' : ''}`}
                                                    disabled={!canAct || actionUsed || (knownSpells.length === 0 && spellSlots.length === 0)}
                                                    onClick={() => openSpellPicker(false)}
                                                >✦ Spell</button>
                                                {showSpellPicker && !warMagicMode && selectedAction === 'spell' && (
                                                    <SpellPicker
                                                        spells={actionSpells}
                                                        slots={spellSlots}
                                                        concentration={concentration}
                                                        charLevel={player.level || 1}
                                                        onCast={handleSpellSelected}
                                                        onClose={() => { setShowSpellPicker(false); setSelectedAction(null) }}
                                                        warMagicMode={false}
                                                    />
                                                )}
                                            </div>
                                        )}

                                        {/* Non-EK placeholder */}
                                        {!isEK && (
                                            <button className="action-btn" disabled title="Eldritch Knight only">✦ Spell</button>
                                        )}

                                        <div style={{ position: 'relative' }}>
                                            <button
                                                className={`action-btn${showSkillsMenu ? ' selected' : ''}`}
                                                disabled={!canAct}
                                                onClick={() => { setShowSkillsMenu(s => !s); setShowSpellPicker(false) }}
                                            >✦ Skills</button>
                                            {showSkillsMenu && (
                                                <div className="skills-submenu">
                                                    {combatAbilities.length === 0 && (
                                                        <div style={{ padding: '.3rem .5rem', fontSize: '.65rem', color: 'var(--dim)' }}>No abilities</div>
                                                    )}
                                                    {combatAbilities.map(ab => (
                                                        <button key={ab.id} className="skill-btn"
                                                            disabled={ab.current_uses === 0}
                                                            onClick={() => useSkill(ab)}
                                                        >
                                                            <span>{ab.name}</span>
                                                            <span className={`skill-uses${ab.current_uses === 0 ? ' empty' : ''}`}>
                                                                {ab.refresh_type === 'per_turn' ? '∞' : `${ab.current_uses}/${ab.max_uses}`}
                                                            </span>
                                                        </button>
                                                    ))}
                                                </div>
                                            )}
                                        </div>

                                        <button className="action-btn" disabled={!canAct || actionUsed}
                                            onClick={() => { setActionUsed(true); addLog(`${player.name} passes action.`, 'system') }}
                                        >Pass</button>

                                        <button className="action-btn danger" disabled={!canAct || actionUsed}
                                            onClick={startFlee}
                                        >↪ Flee</button>
                                    </div>
                                </div>

                                {/* BONUS ACTION */}
                                <div className="economy-slot">
                                    <div className="economy-label">Bonus Action</div>
                                    <div className="economy-buttons">
                                        {/* War Bonds summon */}
                                        {isEK && warBonds.length > 0 && warBonds.map(bond => (
                                            <button
                                                key={bond.id}
                                                className={`action-btn spell-btn${bonusActionUsed ? ' used' : ''}`}
                                                disabled={!canAct || bonusActionUsed}
                                                onClick={() => summonBondedWeapon(bond.item_id, bond.item_name)}
                                                title={`Summon ${bond.item_name} (War Bond)`}
                                            >
                                                ↑ {bond.item_name.length > 8 ? bond.item_name.slice(0, 8) + '…' : bond.item_name}
                                            </button>
                                        ))}

                                        {/* Bonus action spells */}
                                        {isEK && bonusSpells.length > 0 && (
                                            <div style={{ position: 'relative' }}>
                                                <button
                                                    className={`action-btn spell-btn${bonusActionUsed ? ' used' : ''}${selectedAction === 'bonus_spell' && showSpellPicker ? ' selected' : ''}`}
                                                    disabled={!canAct || bonusActionUsed}
                                                    onClick={openBonusSpellPicker}
                                                >✦ B.Spell</button>
                                                {showSpellPicker && selectedAction === 'bonus_spell' && (
                                                    <SpellPicker
                                                        spells={bonusSpells}
                                                        slots={spellSlots}
                                                        concentration={concentration}
                                                        charLevel={player.level || 1}
                                                        onCast={async (spell, castLevel) => {
                                                            await handleSpellSelected(spell, castLevel)
                                                            setBonusActionUsed(true)
                                                        }}
                                                        onClose={() => { setShowSpellPicker(false); setSelectedAction(null) }}
                                                        warMagicMode={false}
                                                    />
                                                )}
                                            </div>
                                        )}

                                        <button className={`action-btn${bonusActionUsed ? ' used' : ''}`}
                                            disabled={!canAct || bonusActionUsed}
                                            onClick={() => { setBonusActionUsed(true); addLog(`${player.name} passes bonus action.`, 'system') }}
                                        >Pass</button>
                                    </div>
                                </div>

                                <button className="action-btn end-turn"
                                    disabled={!canAct || phase === 'enemy_turns'}
                                    onClick={endTurn}
                                >End Turn →</button>
                            </div>
                        )}

                    {/* Enemy Turns */}
                    {phase === 'enemy_turns' && (
                        <div className="action-economy" style={{ justifyContent: 'center' }}>
                            <div style={{ fontFamily: 'Cinzel, serif', fontSize: '.72rem', color: 'var(--dim)', letterSpacing: '.1em' }}>
                                Enemy turn resolving…
                            </div>
                        </div>
                    )}

                    {/* Combat Log */}
                    <div className="combat-log" ref={logRef}>
                        {log.length === 0 && <div className="log-entry system">Combat begins…</div>}
                        {log.map(entry => (
                            <div key={entry.id} className={`log-entry ${entry.type}`}>{entry.text}</div>
                        ))}
                    </div>

                </div>
            </div>
        </>
    )
}