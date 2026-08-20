-- ─── Origin Feats ─────────────────────────────────────────────────────────────

INSERT INTO feats (id,name,category,prerequisite_level,prerequisite_feature,prerequisite_ability,prerequisite_ability_score,description,ability_score_options,ability_score_increase,repeatable,has_choice,choice_description,grants_spells) VALUES
 
('feat_alert','Alert','origin',0,NULL,NULL,0,
'Initiative Proficiency: when you roll Initiative, add your Proficiency Bonus to the roll. Initiative Swap: immediately after you roll Initiative, you can swap your Initiative with one willing ally in the same combat (neither of you can be Incapacitated).',
NULL,0,0,0,NULL,NULL),
 
('feat_crafter','Crafter','origin',0,NULL,NULL,0,
'Tool Proficiency: gain proficiency with three Artisan''s Tools of your choice. Discount: receive a 20% discount whenever you buy a nonmagical item. Fast Crafting: when you finish a Long Rest, craft one piece of gear from the Fast Crafting table (requires appropriate tools and proficiency). The item lasts until your next Long Rest.',
NULL,0,0,1,'Three Artisan''s Tools of your choice',NULL),
 
('feat_healer','Healer','origin',0,NULL,NULL,0,
'Battle Medic: if you have a Healer''s Kit, expend one use and tend to a creature within 5 feet (Utilize action). That creature expends one Hit Point Die, you roll it, and the creature regains HP equal to the roll plus your Proficiency Bonus. Healing Rerolls: whenever you roll a die to determine Hit Points restored with a spell or this feat, reroll on a 1 (must use new roll).',
NULL,0,0,0,NULL,NULL),
 
('feat_lucky','Lucky','origin',0,NULL,NULL,0,
'Luck Points: you have Luck Points equal to your Proficiency Bonus, regained on Long Rest. Advantage: spend 1 Luck Point to give yourself Advantage on a D20 Test before rolling. Disadvantage: spend 1 Luck Point to impose Disadvantage on an attack roll against you.',
NULL,0,0,0,NULL,NULL),
 
('feat_magic_initiate','Magic Initiate','origin',0,NULL,NULL,0,
'Two Cantrips: learn two cantrips from the Cleric, Druid, or Wizard spell list. Choose INT, WIS, or CHA as the spellcasting ability for this feat. Level 1 Spell: choose a level 1 spell from the same list — always prepared, cast once/LR without a slot (also castable with slots). Spell Change: when you gain a level, you can replace one chosen spell with another of the same level from the same list.',
NULL,0,1,1,'Spell list (Cleric/Druid/Wizard), two cantrips, one level 1 spell, spellcasting ability',NULL),
 
('feat_musician','Musician','origin',0,NULL,NULL,0,
'Instrument Training: gain proficiency with three Musical Instruments of your choice. Encouraging Song: when you finish a Short or Long Rest, play a song on a Musical Instrument you have proficiency with — give Heroic Inspiration to a number of allies who hear the song equal to your Proficiency Bonus.',
NULL,0,0,1,'Three Musical Instruments of your choice',NULL),
 
('feat_savage_attacker','Savage Attacker','origin',0,NULL,NULL,0,
'Once per turn when you hit a target with a weapon, roll the weapon''s damage dice twice and use either roll against the target.',
NULL,0,0,0,NULL,NULL),
 
('feat_skilled','Skilled','origin',0,NULL,NULL,0,
'You gain proficiency in any combination of three skills or tools of your choice.',
NULL,0,1,1,'Three skills or tools of your choice',NULL),
 
('feat_tavern_brawler','Tavern Brawler','origin',0,NULL,NULL,0,
'Enhanced Unarmed Strike: when you hit with an Unarmed Strike and deal damage, deal Bludgeoning damage equal to 1d4 + STR modifier instead of normal Unarmed Strike damage. Damage Rerolls: reroll a damage die for Unarmed Strikes on a 1 (must use new roll). Improvised Weaponry: proficiency with improvised weapons. Push: once per turn when you hit a creature with an Unarmed Strike as part of the Attack action, you can deal damage and also push it 5 feet away.',
NULL,0,0,0,NULL,NULL),
 
('feat_tough','Tough','origin',0,NULL,NULL,0,
'Your Hit Point maximum increases by twice your character level when you gain this feat. Whenever you gain a character level thereafter, your HP maximum increases by an additional 2 Hit Points.',
NULL,0,0,0,NULL,NULL);

-- ─── General Feats ───────────────────────────────────────────────────────────

INSERT INTO feats (id,name,category,prerequisite_level,prerequisite_feature,prerequisite_ability,prerequisite_ability_score,description,ability_score_options,ability_score_increase,repeatable,has_choice,choice_description,grants_spells) VALUES
 
('feat_asi','Ability Score Improvement','general',4,NULL,NULL,0,
'Increase one ability score of your choice by 2, or increase two ability scores of your choice by 1 each. Cannot increase any score above 20.',
'["str","dex","con","int","wis","cha"]',2,1,1,'One score by 2 OR two scores by 1 each',NULL),
 
('feat_actor','Actor','general',4,NULL,'cha',13,
'Ability Score Increase: +1 CHA (max 20). Impersonation: while disguised as a real or fictional person, Advantage on CHA (Deception or Performance) checks to convince others you are that person. Mimicry: mimic sounds of other creatures including speech; WIS (Insight) DC 8 + CHA mod + Prof to detect.',
'["cha"]',1,0,0,NULL,NULL),
 
('feat_athlete','Athlete','general',4,NULL,'str/dex',13,
'Ability Score Increase: +1 STR or DEX (max 20). Climb Speed: gain Climb Speed equal to your Speed. Hop Up: when Prone, stand up with only 5 feet of movement. Jumping: make a running Long or High Jump after moving only 5 feet.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_charger','Charger','general',4,NULL,'str/dex',13,
'Ability Score Increase: +1 STR or DEX (max 20). Improved Dash: when you take the Dash action, Speed increases by 10 feet for that action. Charge Attack: if you move at least 10 feet straight toward a target before hitting it with a melee attack (Attack action), choose: gain 1d8 bonus damage OR push target up to 10 feet away (if no more than one size larger). Once per turn.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_chef','Chef','general',4,NULL,NULL,0,
'Ability Score Increase: +1 CON or WIS (max 20). Cook''s Utensils: gain proficiency if you lack it. Replenishing Meal: during a Short Rest (with ingredients and Cook''s Utensils), cook food for up to 4 + Prof Bonus creatures. At end of Short Rest, creatures who eat it and spend Hit Dice regain extra 1d8 HP. Bolstering Treats: with 1 hour of work or on Long Rest, create Prof Bonus treats (last 8 hours). A creature uses a Bonus Action to eat one, gaining Temp HP equal to your Prof Bonus.',
'["con","wis"]',1,0,0,NULL,NULL),
 
('feat_crossbow_expert','Crossbow Expert','general',4,NULL,'dex',13,
'Ability Score Increase: +1 DEX (max 20). Ignore Loading: ignore the Loading property of hand/heavy/light crossbows; can load without a free hand. Firing in Melee: no Disadvantage on crossbow attacks within 5 feet of an enemy. Dual Wielding: when you make the extra attack from the Light property with a crossbow, add your ability modifier to the damage if not already doing so.',
'["dex"]',1,0,0,NULL,NULL),
 
('feat_crusher','Crusher','general',4,NULL,NULL,0,
'Ability Score Increase: +1 STR or CON (max 20). Push: once per turn when you deal Bludgeoning damage, move the target 5 feet to an unoccupied space (must be no more than one size larger). Enhanced Critical: when you score a Critical Hit dealing Bludgeoning damage, attack rolls against the target have Advantage until start of your next turn.',
'["str","con"]',1,0,0,NULL,NULL),
 
('feat_defensive_duelist','Defensive Duelist','general',4,NULL,'dex',13,
'Ability Score Increase: +1 DEX (max 20). Parry: if you''re holding a Finesse weapon and another creature hits you with a melee attack, Reaction — add your Proficiency Bonus to your AC, potentially causing the attack to miss. Bonus lasts until start of your next turn.',
'["dex"]',1,0,0,NULL,NULL),
 
('feat_dual_wielder','Dual Wielder','general',4,NULL,'str/dex',13,
'Ability Score Increase: +1 STR or DEX (max 20). Enhanced Dual Wielding: when you take the Attack action with a Light weapon, make one extra attack as a Bonus Action with a different Melee weapon (must lack Two-Handed). No ability modifier to extra attack damage unless negative. Quick Draw: draw or stow two non-Two-Handed weapons when you would normally stow one.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_durable','Durable','general',4,NULL,NULL,0,
'Ability Score Increase: +1 CON (max 20). Defy Death: Advantage on Death Saving Throws. Speedy Recovery: Bonus Action — expend one Hit Point Die, roll it, regain that many HP.',
'["con"]',1,0,0,NULL,NULL),
 
('feat_elemental_adept','Elemental Adept','general',4,'spellcasting',NULL,0,
'Ability Score Increase: +1 INT, WIS, or CHA (max 20). Energy Mastery: choose one damage type (Acid, Cold, Fire, Lightning, or Thunder). Your spells ignore Resistance to that type. When you roll damage for a spell of that type, treat any 1 on a damage die as a 2.',
'["int","wis","cha"]',1,1,1,'Damage type: Acid, Cold, Fire, Lightning, or Thunder',NULL),
 
('feat_fey_touched','Fey Touched','general',4,NULL,NULL,0,
'Ability Score Increase: +1 INT, WIS, or CHA (max 20). Fey Magic: choose one level 1 spell from the Divination or Enchantment school. You always have that spell and Misty Step prepared. Cast each once/LR without a slot; also castable with spell slots of the appropriate level. Spellcasting ability = the increased stat.',
'["int","wis","cha"]',1,0,1,'One level 1 Divination or Enchantment spell; spellcasting ability',NULL),
 
('feat_grappler','Grappler','general',4,NULL,'str/dex',13,
'Ability Score Increase: +1 STR or DEX (max 20). Punch and Grab: when you hit a creature with an Unarmed Strike (Attack action), use both Damage and Grapple options on that strike. Once per turn. Attack Advantage: Advantage on attack rolls against a creature you have Grappled. Fast Wrestler: no extra movement cost to move a creature you have Grappled if it is your size or smaller.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_great_weapon_master','Great Weapon Master','general',4,NULL,'str',13,
'Ability Score Increase: +1 STR (max 20). Heavy Weapon Mastery: when you hit a creature with a Heavy weapon as part of the Attack action, deal extra damage equal to your Proficiency Bonus. Hew: immediately after you score a Critical Hit with a Melee weapon or reduce a creature to 0 HP with one, make one attack with the same weapon as a Bonus Action.',
'["str"]',1,0,0,NULL,NULL),
 
('feat_heavily_armored','Heavily Armored','general',4,'medium_armor_training',NULL,0,
'Ability Score Increase: +1 CON or STR (max 20). Armor Training: gain training with Heavy armor.',
'["con","str"]',1,0,0,NULL,NULL),
 
('feat_heavy_armor_master','Heavy Armor Master','general',4,'heavy_armor_training',NULL,0,
'Ability Score Increase: +1 CON or STR (max 20). Damage Reduction: when hit by an attack while wearing Heavy armor, reduce Bludgeoning, Piercing, and Slashing damage dealt by an amount equal to your Proficiency Bonus.',
'["con","str"]',1,0,0,NULL,NULL),
 
('feat_inspiring_leader','Inspiring Leader','general',4,NULL,'wis/cha',13,
'Ability Score Increase: +1 WIS or CHA (max 20). Bolstering Performance: when you finish a Short or Long Rest, give an inspiring performance (speech, song, or dance). Choose up to 6 allies within 30 feet who witness it — each gains Temp HP equal to your character level + the modifier of the stat increased.',
'["wis","cha"]',1,0,0,NULL,NULL),
 
('feat_keen_mind','Keen Mind','general',4,NULL,'int',13,
'Ability Score Increase: +1 INT (max 20). Lore Knowledge: choose one skill (Arcana, History, Investigation, Nature, or Religion) — gain proficiency if you lack it, or Expertise if you already have proficiency. Quick Study: take the Study action as a Bonus Action.',
'["int"]',1,0,1,'One of: Arcana, History, Investigation, Nature, Religion',NULL),
 
('feat_lightly_armored','Lightly Armored','general',4,NULL,NULL,0,
'Ability Score Increase: +1 STR or DEX (max 20). Armor Training: gain training with Light armor and Shields.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_mage_slayer','Mage Slayer','general',4,NULL,NULL,0,
'Ability Score Increase: +1 STR or DEX (max 20). Concentration Breaker: when you damage a creature that is concentrating, it has Disadvantage on its Concentration save. Guarded Mind: once per Short or Long Rest, if you fail an INT, WIS, or CHA saving throw, you can cause yourself to succeed instead.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_martial_weapon_training','Martial Weapon Training','general',4,NULL,NULL,0,
'Ability Score Increase: +1 STR or DEX (max 20). Weapon Proficiency: you gain proficiency with Martial weapons.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_medium_armor_master','Medium Armor Master','general',4,'medium_armor_training',NULL,0,
'Ability Score Increase: +1 STR or DEX (max 20). Dexterous Wearer: while wearing Medium armor, you can add 3 (rather than 2) to your AC if you have DEX 16 or higher.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_moderately_armored','Moderately Armored','general',4,'light_armor_training',NULL,0,
'Ability Score Increase: +1 STR or DEX (max 20). Armor Training: gain training with Medium armor.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_mounted_combatant','Mounted Combatant','general',4,NULL,NULL,0,
'Ability Score Increase: +1 STR, DEX, or WIS (max 20). Mounted Strike: while mounted, Advantage on attack rolls against unmounted creatures within 5 feet of your mount that are at least one size smaller than the mount. Leap Aside: if your mount makes a DEX save for half damage, it takes no damage on success and half on failure (you must be riding it, neither Incapacitated). Veer: while mounted, force an attack hitting your mount to hit you instead (you must not be Incapacitated).',
'["str","dex","wis"]',1,0,0,NULL,NULL),
 
('feat_observant','Observant','general',4,NULL,'int/wis',13,
'Ability Score Increase: +1 INT or WIS (max 20). Keen Observer: choose one of Insight, Investigation, or Perception — gain proficiency if you lack it, or Expertise if you already have proficiency. Quick Search: take the Search action as a Bonus Action.',
'["int","wis"]',1,0,1,'One of: Insight, Investigation, Perception',NULL),
 
('feat_piercer','Piercer','general',4,NULL,NULL,0,
'Ability Score Increase: +1 STR or DEX (max 20). Puncture: once per turn when you deal Piercing damage, reroll one damage die and must use the new roll. Enhanced Critical: when you score a Critical Hit dealing Piercing damage, roll one additional damage die for the extra Piercing damage.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_poisoner','Poisoner','general',4,NULL,NULL,0,
'Ability Score Increase: +1 DEX or INT (max 20). Potent Poison: when you deal Poison damage, it ignores Resistance to Poison damage. Brew Poison: gain proficiency with the Poisoner''s Kit. With 1 hour of work and 50 GP of materials, create Prof Bonus poison doses. Bonus Action to apply to a weapon. When the creature takes damage from the poisoned item, it makes a CON save (DC 8 + modified ability mod + Prof Bonus) or take 2d8 Poison damage and have the Poisoned condition until end of your next turn.',
'["dex","int"]',1,0,0,NULL,NULL),
 
('feat_polearm_master','Polearm Master','general',4,NULL,'str/dex',13,
'Ability Score Increase: +1 DEX or STR (max 20). Pole Strike: immediately after attacking with a Quarterstaff, Spear, or Heavy+Reach weapon (Attack action), Bonus Action to make a melee attack with the opposite end (1d4 Bludgeoning). Reactive Strike: while holding such a weapon, Reaction to make one melee attack against a creature that enters your reach.',
'["dex","str"]',1,0,0,NULL,NULL),
 
('feat_resilient','Resilient','general',4,NULL,NULL,0,
'Ability Score Increase: choose one ability in which you lack saving throw proficiency; increase it by 1 (max 20). Saving Throw Proficiency: gain saving throw proficiency with the chosen ability.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score in which you lack saving throw proficiency',NULL),
 
('feat_ritual_caster','Ritual Caster','general',4,NULL,'int/wis/cha',13,
'Ability Score Increase: +1 INT, WIS, or CHA (max 20). Ritual Spells: choose level 1 spells with the Ritual tag equal to your Proficiency Bonus — always prepared, castable with spell slots. Spellcasting ability = the increased stat. Gain additional Ritual spells as Prof Bonus increases. Quick Ritual: once per Long Rest, cast a prepared Ritual spell at normal casting time without expending a slot.',
'["int","wis","cha"]',1,0,1,'Spellcasting ability and Ritual spells equal to Proficiency Bonus',NULL),
 
('feat_sentinel','Sentinel','general',4,NULL,'str/dex',13,
'Ability Score Increase: +1 STR or DEX (max 20). Guardian: immediately after a creature within 5 feet takes the Disengage action or hits a target other than you with an attack, make an Opportunity Attack against it. Halt: when you hit a creature with an Opportunity Attack, its Speed becomes 0 for the rest of the current turn.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_shadow_touched','Shadow Touched','general',4,NULL,NULL,0,
'Ability Score Increase: +1 INT, WIS, or CHA (max 20). Shadow Magic: choose one level 1 spell from the Illusion or Necromancy school — you always have that spell and Invisibility prepared. Cast each once/LR without a slot; also castable with spell slots. Spellcasting ability = the increased stat.',
'["int","wis","cha"]',1,0,1,'One level 1 Illusion or Necromancy spell; spellcasting ability',NULL),
 
('feat_sharpshooter','Sharpshooter','general',4,NULL,'dex',13,
'Ability Score Increase: +1 DEX (max 20). Bypass Cover: ranged weapon attacks ignore Half Cover and Three-Quarters Cover. Firing in Melee: no Disadvantage on ranged weapon attacks within 5 feet of an enemy. Long Shots: attacking at long range doesn''t impose Disadvantage on ranged weapon attack rolls.',
'["dex"]',1,0,0,NULL,NULL),
 
('feat_shield_master','Shield Master','general',4,'shield_training',NULL,0,
'Ability Score Increase: +1 STR (max 20). Shield Bash: if you hit a creature with a Melee weapon as part of the Attack action, bash with your Shield (if equipped) — target makes STR save (DC 8 + STR mod + Prof) or you push it 5 feet or give it Prone (your choice). Once per turn. Interpose Shield: Reaction when you make a DEX save for half damage and are holding a Shield — take no damage on success.',
'["str"]',1,0,0,NULL,NULL),
 
('feat_skill_expert','Skill Expert','general',4,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 20). Skill Proficiency: gain proficiency in one skill of your choice. Expertise: choose one skill in which you have proficiency but lack Expertise — gain Expertise with it.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score, one skill proficiency, one skill for Expertise',NULL),
 
('feat_skulker','Skulker','general',4,NULL,'dex',13,
'Ability Score Increase: +1 DEX (max 20). Blindsight: you have Blindsight with a range of 10 feet. Fog of War: Advantage on DEX (Stealth) checks made as part of the Hide action during combat. Sniper: if you make an attack roll while hidden and the roll misses, making the attack roll doesn''t reveal your location.',
'["dex"]',1,0,0,NULL,NULL),
 
('feat_slasher','Slasher','general',4,NULL,NULL,0,
'Ability Score Increase: +1 STR or DEX (max 20). Hamstring: once per turn when you deal Slashing damage, reduce the target''s Speed by 10 feet until the start of your next turn. Enhanced Critical: when you score a Critical Hit dealing Slashing damage, the target has Disadvantage on attack rolls until the start of your next turn.',
'["str","dex"]',1,0,0,NULL,NULL),
 
('feat_speedy','Speedy','general',4,NULL,'dex/con',13,
'Ability Score Increase: +1 DEX or CON (max 20). Speed Increase: your Speed increases by 10 feet. Dash over Difficult Terrain: when you take the Dash action, Difficult Terrain doesn''t cost you extra movement for the rest of that turn. Agile Movement: Opportunity Attacks have Disadvantage against you.',
'["dex","con"]',1,0,0,NULL,NULL),
 
('feat_spell_sniper','Spell Sniper','general',4,'spellcasting',NULL,0,
'Ability Score Increase: +1 INT, WIS, or CHA (max 20). Bypass Cover: spell attack rolls ignore Half Cover and Three-Quarters Cover. Casting in Melee: no Disadvantage on spell attack rolls within 5 feet of an enemy. Increased Range: when you cast a spell with a range of at least 10 feet that requires an attack roll, increase its range by 60 feet.',
'["int","wis","cha"]',1,0,0,NULL,NULL),
 
('feat_telekinetic','Telekinetic','general',4,NULL,NULL,0,
'Ability Score Increase: +1 INT, WIS, or CHA (max 20). Minor Telekinesis: learn Mage Hand — cast without Verbal or Somatic components, the hand can be Invisible, and its range and maximum distance from you increase by 30 feet. Spellcasting ability = the increased stat. Telekinetic Shove: Bonus Action — telekinetically shove one creature you can see within 30 feet; STR save (DC 8 + increased stat mod + Prof) or be moved 5 feet toward or away from you (your choice).',
'["int","wis","cha"]',1,0,1,'Spellcasting ability (INT/WIS/CHA)',NULL),
 
('feat_telepathic','Telepathic','general',4,NULL,NULL,0,
'Ability Score Increase: +1 INT, WIS, or CHA (max 20). Telepathic Utterance: speak telepathically to any creature you can see within 60 feet in a language you both know (one-way only). Detect Thoughts: always prepared; cast once/LR without a slot or components. Also castable with spell slots. Spellcasting ability = the increased stat.',
'["int","wis","cha"]',1,0,1,'Spellcasting ability (INT/WIS/CHA)',
'["spell_detect_thoughts"]'),
 
('feat_war_caster','War Caster','general',4,'spellcasting',NULL,0,
'Ability Score Increase: +1 INT, WIS, or CHA (max 20). Concentration: Advantage on CON saves to maintain Concentration. Reactive Spell: when a creature leaves your reach, Reaction to cast a spell at it (action casting time, targets only that creature) instead of an Opportunity Attack. Somatic Components: perform Somatic components even with weapons or a Shield in one or both hands.',
'["int","wis","cha"]',1,0,0,NULL,NULL),
 
('feat_weapon_master','Weapon Master','general',4,NULL,NULL,0,
'Ability Score Increase: +1 STR or DEX (max 20). Mastery Property: use the mastery property of one Simple or Martial weapon of your choice (must have proficiency). Change weapon choice whenever you finish a Long Rest.',
'["str","dex"]',1,0,1,'One Simple or Martial weapon for Mastery',NULL);

-- ─── Fighting Style Feats ─────────────────────────────────────────────────────
 
INSERT INTO feats (id,name,category,prerequisite_level,prerequisite_feature,prerequisite_ability,prerequisite_ability_score,description,ability_score_options,ability_score_increase,repeatable,has_choice,choice_description,grants_spells) VALUES
 
('feat_fs_archery','Archery','fighting_style',0,'fighting_style',NULL,0,
'You gain a +2 bonus to attack rolls you make with Ranged weapons.',
NULL,0,0,0,NULL,NULL),
 
('feat_fs_blind_fighting','Blind Fighting','fighting_style',0,'fighting_style',NULL,0,
'You have Blindsight with a range of 10 feet.',
NULL,0,0,0,NULL,NULL),
 
('feat_fs_defense','Defense','fighting_style',0,'fighting_style',NULL,0,
'While you''re wearing Light, Medium, or Heavy armor, you gain a +1 bonus to Armor Class.',
NULL,0,0,0,NULL,NULL),
 
('feat_fs_dueling','Dueling','fighting_style',0,'fighting_style',NULL,0,
'When you''re holding a Melee weapon in one hand and no other weapons, you gain a +2 bonus to damage rolls with that weapon.',
NULL,0,0,0,NULL,NULL),
 
('feat_fs_great_weapon_fighting','Great Weapon Fighting','fighting_style',0,'fighting_style',NULL,0,
'When you roll damage for an attack with a Melee weapon held in two hands, treat any 1 or 2 on a damage die as a 3. The weapon must have the Two-Handed or Versatile property.',
NULL,0,0,0,NULL,NULL),
 
('feat_fs_interception','Interception','fighting_style',0,'fighting_style',NULL,0,
'When a creature you can see hits another creature within 5 feet of you with an attack roll, Reaction — reduce the damage dealt to the target by 1d10 plus your Proficiency Bonus. You must be holding a Shield or a Simple or Martial weapon.',
NULL,0,0,0,NULL,NULL),
 
('feat_fs_protection','Protection','fighting_style',0,'fighting_style',NULL,0,
'When a creature you can see attacks a target other than you that is within 5 feet of you, Reaction — if you''re holding a Shield, impose Disadvantage on the triggering attack roll and all other attack rolls against the target until the start of your next turn (while within 5 feet).',
NULL,0,0,0,NULL,NULL),
 
('feat_fs_thrown_weapon_fighting','Thrown Weapon Fighting','fighting_style',0,'fighting_style',NULL,0,
'When you hit with a ranged attack roll using a weapon that has the Thrown property, you gain a +2 bonus to the damage roll.',
NULL,0,0,0,NULL,NULL),
 
('feat_fs_two_weapon_fighting','Two-Weapon Fighting','fighting_style',0,'fighting_style',NULL,0,
'When you make an extra attack as a result of using a weapon that has the Light property, you can add your ability modifier to the damage of that attack if you aren''t already adding it to the damage.',
NULL,0,0,0,NULL,NULL),
 
('feat_fs_unarmed_fighting','Unarmed Fighting','fighting_style',0,'fighting_style',NULL,0,
'When you hit with an Unarmed Strike and deal damage, deal Bludgeoning damage equal to 1d6 + STR modifier instead of normal Unarmed Strike damage. If not holding any weapons or a Shield, the d6 becomes a d8. At the start of each of your turns, deal 1d4 Bludgeoning damage to one creature Grappled by you.',
NULL,0,0,0,NULL,NULL);
 
 
-- ─── Epic Boon Feats ──────────────────────────────────────────────────────────
 
INSERT INTO feats (id,name,category,prerequisite_level,prerequisite_feature,prerequisite_ability,prerequisite_ability_score,description,ability_score_options,ability_score_increase,repeatable,has_choice,choice_description,grants_spells) VALUES
 
('feat_boon_combat_prowess','Boon of Combat Prowess','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). Peerless Aim: once per turn, when you miss with an attack roll, you can hit instead.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score of your choice',NULL),
 
('feat_boon_dimensional_travel','Boon of Dimensional Travel','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). Blink Steps: immediately after you take the Attack action or the Magic action, teleport up to 30 feet to an unoccupied space you can see.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score of your choice',NULL),
 
('feat_boon_energy_resistance','Boon of Energy Resistance','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). Energy Resistances: gain Resistance to two damage types of your choice (Acid, Cold, Fire, Lightning, Necrotic, Poison, Psychic, Radiant, or Thunder). Change choices on Long Rest. Energy Redirection: Reaction when you take damage of a chosen type — direct equal damage toward a creature you can see within 60 feet (DEX save DC 8 + CON mod + Prof).',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score; two damage types for Resistance',NULL),
 
('feat_boon_fate','Boon of Fate','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). Improve Fate: when you or another creature within 60 feet succeeds on or fails a D20 Test, roll 2d4 and apply the total as a bonus or penalty to the d20 roll. Once per Initiative roll or Short or Long Rest.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score of your choice',NULL),
 
('feat_boon_fortitude','Boon of Fortitude','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). Fortified Health: Hit Point maximum increases by 40. Whenever you regain HP, also regain additional HP equal to your CON modifier (once per turn).',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score of your choice',NULL),
 
('feat_boon_irresistible_offense','Boon of Irresistible Offense','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 STR or DEX (max 30). Overcome Defenses: Bludgeoning, Piercing, and Slashing damage you deal always ignores Resistance. Overwhelming Strike: when you roll a 20 on the d20 for an attack roll, deal extra damage equal to the increased ability score (same type as the attack).',
'["str","dex"]',1,0,1,'STR or DEX',NULL),
 
('feat_boon_recovery','Boon of Recovery','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). Last Stand: once per Long Rest, when you would be reduced to 0 HP, drop to 1 HP and regain HP equal to half your HP maximum. Recover Vitality: pool of ten d10s. Bonus Action to expend dice, roll them, regain that many HP. Regain all dice on Long Rest.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score of your choice',NULL),
 
('feat_boon_skill','Boon of Skill','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). All-Around Adept: gain proficiency in all skills. Expertise: choose one skill in which you lack Expertise — gain Expertise in it.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score; one skill for Expertise',NULL),
 
('feat_boon_speed','Boon of Speed','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). Escape Artist: Bonus Action to take the Disengage action, which also ends the Grappled condition on you. Quickness: Speed increases by 30 feet.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score of your choice',NULL),
 
('feat_boon_spell_recall','Boon of Spell Recall','epic_boon',19,'spellcasting',NULL,0,
'Ability Score Increase: +1 INT, WIS, or CHA (max 30). Free Casting: whenever you cast a spell with a level 1–4 spell slot, roll 1d4. If the number rolled equals the slot''s level, the slot isn''t expended.',
'["int","wis","cha"]',1,0,1,'INT, WIS, or CHA',NULL),
 
('feat_boon_night_spirit','Boon of the Night Spirit','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). Merge with Shadows: while in Dim Light or Darkness, Bonus Action to give yourself the Invisible condition. Ends immediately after you take an action, Bonus Action, or Reaction. Shadowy Form: while in Dim Light or Darkness, Resistance to all damage except Psychic and Radiant.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score of your choice',NULL),
 
('feat_boon_truesight','Boon of Truesight','epic_boon',19,NULL,NULL,0,
'Ability Score Increase: +1 to one ability score of your choice (max 30). Truesight: you have Truesight with a range of 60 feet.',
'["str","dex","con","int","wis","cha"]',1,0,1,'One ability score of your choice',NULL);

