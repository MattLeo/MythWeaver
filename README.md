# MythWeaver

MythWeaver is an AI-powered Dungeon Master that runs a fully persistent D&D 5th Edition campaign entirely through conversation. You play. The AI narrates, reacts, and manages the world.

---

## What It Is

Most AI roleplaying tools are glorified chatbots — they forget what happened, invent contradictions, and have no mechanical backbone. MythWeaver is different. It combines a large language model acting as a Dungeon Master with a real game engine running underneath. The AI handles storytelling, worldbuilding, and character. The backend handles every number.

You create a character, step into a world that has never existed before, and shape it entirely through your choices. The world remembers everything.

---

## How It Works

When you send a message, it goes to the AI Dungeon Master, which has access to a suite of tools it can call against a persistent database. When you find an item, the AI doesn't just say you found it — it creates the item in the database and adds it to your inventory. When you spend gold, the gold is deducted. When you level up, your stats change. Everything that happens in the story is mechanically real.

**Combat** is fully backend-resolved. When a fight breaks out, the AI calls `start_combat` with the enemy stats it invents for the scene, initiative is rolled, and turns proceed in order. When you attack, you roll a physical d20 on screen. The backend checks it against the enemy's armor class, requests a damage roll if it hits, applies the result, then automatically resolves every enemy and ally turn — each one narrated by the AI. You never have to track hit points or do math. The AI never cheats the dice.

**The world is persistent.** Every location you visit, every NPC you meet, every piece of lore you establish gets saved to the database. The AI queries this before creating anything new, so characters and places stay consistent across sessions. Past sessions are summarized and fed back into the AI's context so it remembers the shape of your story even across multiple play sessions.

**Sessions are resumable.** When you return to the title screen, all your saved campaigns appear with your character's current state. Select one and your full chat history, inventory, abilities, and world state are restored exactly as you left them.

---

## What the AI Does

The AI's job is purely narrative and creative. It decides what the world looks like, how NPCs speak and behave, what threats emerge, and how the world reacts to your choices. It is instructed never to ask clarifying questions, never to offer you a list of options, and never to break character. It reacts to whatever you do and moves the story forward.

It has access to tools for creating and querying locations, NPCs, world facts, items, companions, and quests. It is required to use these tools rather than just narrating — if it describes you receiving a sword, that sword must exist in the database first.

---

## The Experience

You type what your character does. The AI responds with vivid literary prose — two to four paragraphs, written like a novel, never like a game menu. When the outcome is uncertain, a dice overlay appears and you roll. The result feeds back into the narrative. Combat plays out turn by turn with a short delay between each exchange, giving the fight a sense of rhythm and weight.

The world has no predetermined story. It is built entirely from your choices and the AI's improvisations in response to them. Every campaign is unique.

---

*Built by Matt Taylor · Powered by Anthropic Claude · D&D 5th Edition*