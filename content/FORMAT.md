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
