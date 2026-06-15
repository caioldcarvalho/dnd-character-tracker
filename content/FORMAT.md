# rpgman content format

Rules are **data**. The engine interprets these JSON files; adding game content
never requires code. Each file under `content/<kind>/` holds one definition (or a
JSON array of them). Subdirs: `classes/ subclasses/ species/ backgrounds/ feats/`.

The engine loads everything at startup (`ContentDb::load_dir`). Every number a
character has is the reduction of **contributions** that target a **stat**, so
authoring is mostly "emit a contribution to stat X from this feature."

## StatId (a contribution's `target`)

`{ "kind": "<k>" }`, or `{ "kind": "<k>", "of": <param> }` when parameterized:

| kind | of | meaning |
|---|---|---|
| `character-level` | — | total level (sum of class levels) |
| `class-level` | class id string | levels in one class |
| `ability-score` | `"str"`…`"cha"` | an ability score |
| `ability-modifier` | ability | derived `floor((score-10)/2)` |
| `proficiency-bonus` | — | by level |
| `max-hit-points` | — | |
| `armor-class` | — | |
| `initiative` | — | |
| `speed` | `"walk"`/`"fly"`/`"swim"`/`"climb"`/`"burrow"` | |
| `saving-throw` | ability | |
| `skill-bonus` | skill id (kebab, e.g. `"sleight-of-hand"`) | |
| `passive-score` | skill | passive (base 10 + bonus) |
| `spell-save-dc` | casting source (class id) | |
| `spell-attack-bonus` | casting source | |
| `resource-max` | resource id | a pool's maximum |
| `spell-slot-max` | `{ "0": <n> }`? no — set by engine | (engine-derived) |
| `attacks-per-action` | — | Extra Attack |
| `weapon-attack-bonus` | `"melee"`/`"ranged"` | |
| `weapon-damage-bonus` | `"melee"`/`"ranged"` | |
| `weapon-masteries-known` | — | |
| `carrying-capacity` | — | |

## ValueExpr (a contribution's `value`)

Tagged by `"expr"`. The recursive bit: `stat` references another stat, so values
compose into a dependency graph.

```json
{ "expr": "literal", "value": 3 }
{ "expr": "stat", "id": { "kind": "proficiency-bonus" } }
{ "expr": "ability-mod", "ability": "con" }
{ "expr": "sum", "terms": [ ... ] }
{ "expr": "product", "terms": [ ... ] }
{ "expr": "min", "terms": [ ... ] }      // also "max"
{ "expr": "scaled", "per": <expr>, "by": <expr> }            // per × by
{ "expr": "floor-div", "num": <expr>, "den": <expr> }
{ "expr": "dice-average", "dice": { "count": 1, "sides": 10 } }
{ "expr": "level-table",
  "on": { "expr": "stat", "id": { "kind": "class-level", "of": "fighter" } },
  "table": [[3, 4], [7, 5], [15, 6]] }   // pairs [threshold, value]; greatest ≤ on wins
```

`level-table.on` defaults to character level if omitted. **Tables are arrays of
`[threshold, value]` pairs, never objects.**

## Contribution

```json
{
  "target": { "kind": "armor-class" },
  "op": "add",                       // base | add | multiply | floor | cap | override
  "value": { "expr": "literal", "value": 1 },
  "when": { "if": "wearing-armor" }, // optional; see Conditions
  "effect": "advantage",             // optional: advantage|disadvantage|auto-fail|auto-success
  "note": "Defense",                 // optional; shown in the breakdown
  "stack_group": "cover"             // optional; same group → only the largest applies
}
```

Op bands apply in order: **base** (max of competing bases wins — e.g. AC) →
**add** → **multiply** → **floor** → **cap** → **override**.

For a pure advantage/disadvantage effect, set `effect` and leave `value` at
literal 0 (or omit op-relevant fields).

### Conditions (`when.if`)
`always` · `effect-active` (+`id`) · `wearing-armor` · `unarmored` · `wielding-shield`.
Toggle effects (Rage, etc.) use `effect-active` and an `activation` of `toggle`.

## Feature (the universal bundle)

Used for class features, subclass features, species traits, and feats.

```json
{
  "id": "second-wind",
  "name": "Second Wind",
  "description": "...",
  "category": "origin",              // optional: origin | fighting-style | epic-boon | general
  "activation": "passive",           // passive | activated | reaction |
                                     //   { "toggle": { "default_on": false } }
  "contributions": [ ... ],
  "proficiencies": [ ... ],          // see below
  "resources": [ ... ],              // see ResourceDef
  "spell_grants": [ { "spell": "fireball", "always_prepared": true } ],
  "choices": [ ... ]                 // see ChoicePoint
}
```

### ProficiencyGrant
```json
{ "kind": "skill", "skill": "athletics", "expertise": false }
{ "kind": "saving-throw", "ability": "con" }
{ "kind": "tool", "tool": "thieves-tools" }      // also: weapon, armor, language
```

### ResourceDef (every spendable pool — dice or points)
```json
{
  "id": "superiority-dice",
  "name": "Superiority Dice",
  "kind": "dice",                    // dice | points
  "die": 8,                          // for dice pools
  "die_scaling": { "3": 8, "10": 10, "18": 12 },  // by char level; object IS ok here (u8→u8)
  "max": { "expr": "level-table", "on": {...}, "table": [[3,4],[7,5],[15,6]] },
  "recharge": "short-rest"           // short-rest | long-rest | dawn | special
}
```
The pool's `max` is a full ValueExpr — e.g. psi Energy Dice use
`scaled(2, proficiency-bonus)`. This is why no resource needs special code.

### ChoicePoint
```json
{
  "id": "fighter-asi-4",             // unique; a recorded choice with this key resolves it
  "prompt": "Ability Score Improvement (or a feat)",
  "choose": 2,                       // number of picks required
  "options": { "kind": "ability-score-increase" }
}
```
`options.kind`:
- `skills` / `expertise` — `{ "from": [<skill>...] }`
- `feat` — `{ "category": "fighting-style" }` (optional filter)
- `ability-score-increase` — picks are abilities (+1 each; same twice = +2) or a feat id
- `ability-scores` — `{ "from": [<ability>...], "points": 3, "max_each": 2 }` (backgrounds)
- `subclass`
- `spells` — `{ "from": [...], "max_level": 1 }`
- `weapon-mastery` — `{ "from": [...] }`
- `named` — `{ "from": [ { "id", "name", "contributions": [], "proficiencies": [] } ] }`

## ClassDef
```json
{
  "id": "fighter",
  "name": "Fighter",
  "hit_die": 10,
  "saving_throws": ["str", "con"],   // granted only if this is the FIRST class
  "subclass_level": 3,               // when a subclass is chosen (default 3)
  "spellcasting": {                  // omit for non-casters
    "ability": "int",
    "progression": "full",           // full | half | third | pact | none
    "preparation": "spellbook",      // prepared | known | spellbook
    "prepared_per_level": { "1": 4, "2": 5, ... }   // optional (object u8→i32 ok)
  },
  "levels": {
    "1": { "features": [ <Feature>... ], "choices": [ <ChoicePoint>... ] },
    "2": { ... }
  }
}
```
Spell slots are derived automatically from the combined caster level
(full + half/2 + third/3) — do **not** author slot tables.

## SubclassDef
```json
{ "id": "battle-master", "name": "Battle Master", "class": "fighter",
  "spellcasting": { ... },           // optional (e.g. Eldritch Knight)
  "levels": { "3": { "features": [...] }, "7": { ... } } }
```
Subclass feature levels in 2024 are typically **3, 6, 10, 14** (most classes) or
**3, 7, 10, 15, 18** (Fighter).

## SpeciesDef
```json
{ "id": "elf", "name": "Elf", "speed": 30, "size": "Medium",
  "traits": [ <Feature>... ] }
```
2024 species grant **no ability scores** (those come from the background).

## BackgroundDef
```json
{ "id": "soldier", "name": "Soldier", "description": "...",
  "abilities": ["str", "dex", "con"],   // the 3 choosable for +2/+1 or +1/+1/+1
  "skills": ["athletics", "intimidation"],
  "tools": ["gaming-set"],
  "origin_feat": "savage-attacker" }
```

## Abilities & skills (kebab ids)
Abilities: `str dex con int wis cha`.
Skills: `acrobatics animal-handling arcana athletics deception history insight
intimidation investigation medicine nature perception performance persuasion
religion sleight-of-hand stealth survival`.

## Spells

Spells live in `content/spells/`. Each file holds a JSON **array** of `SpellDef`
objects. Files are named by level band (e.g. `cantrips.json`, `level-1.json`) but
the file name has no semantic meaning — the engine reads all `*.json` files in
the directory.

### SpellDef schema

```json
{
  "id": "fire-bolt",            // kebab-case, globally unique
  "name": "Fire Bolt",          // display name
  "level": 0,                   // 0 = cantrip, 1–9 = spell level
  "school": "evocation",        // see SpellSchool below
  "classes": ["sorcerer", "wizard"],  // class ids that know/prepare this spell
  "casting_time": "1 action",   // free-form string
  "range": "120 feet",          // free-form string ("Self", "Touch", "120 feet", …)
  "components": {
    "verbal": true,
    "somatic": true,
    "material": "a pinch of sulfur"   // omit field entirely when not required
  },
  "duration": "Instantaneous",  // free-form string
  "concentration": false,       // defaults to false if omitted
  "ritual": false,              // defaults to false if omitted
  "description": "…"            // rules text; keep to 1–3 sentences for the seed set
}
```

### SpellSchool values (kebab-case in JSON)

`abjuration` · `conjuration` · `divination` · `enchantment` ·
`evocation` · `illusion` · `necromancy` · `transmutation`

### Class ids (for `classes` list)

Use the same kebab-case ids as in `content/classes/`: `barbarian`, `bard`,
`cleric`, `druid`, `fighter`, `monk`, `paladin`, `ranger`, `rogue`,
`sorcerer`, `warlock`, `wizard`.

### Optional combat fields

Three optional fields describe how a spell interacts in combat. Omit any that
do not apply.

```json
"attack": "ranged",          // or "melee" — the caster makes a spell attack roll
"save": {
  "ability": "dex",          // ability the TARGET saves with (str/dex/con/int/wis/cha)
  "effect": "half damage on a success"   // what happens when the target succeeds
},
"damage": {
  "dice": "3d6",             // dice expression; free-form string
  "damage_type": "fire"      // e.g. fire, cold, radiant, necrotic, healing, varies …
}
```

Rules: `attack` and `save` are mutually exclusive (a spell either attacks or
forces a save). `damage` can appear alone (e.g. auto-hit spells like Magic
Missile, or healing spells). Omit all three for utility/control spells.

### Notes for bulk authoring

- `components.material` should be omitted (not set to `null`) when the spell
  has no material component — serde will skip it on serialization too.
- `concentration` and `ritual` default to `false`; include them only when `true`.
- Spell ids must be unique across **all** files in `content/spells/`; the engine
  will overwrite silently on duplicate (last-write-wins from BTreeMap insertion).
