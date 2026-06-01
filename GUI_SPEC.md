# rpgman GUI spec

The desktop app over the rules engine. **Decisions (locked with the user):**
build-first but the same sheet doubles as the play view · **dense data cockpit**
(compact, dark, everything visible) · **click → side panel** breakdowns · first
milestone is **full build + play**.

## 0. Tech & architecture

- **Tauri 2** shell + **SvelteKit** (static adapter, SPA — `ssr=false`) + **Tailwind v4** + **Svelte 5 runes**.
- **The engine stays authoritative.** The frontend holds the editable
  `CharacterSheet` (a plain object, type-mirrored from Rust via **ts-rs**), and on
  every edit calls `compute` to get the derived `ComputedCharacter`. The UI never
  re-implements a rule.
- **Type sharing:** `ts-rs` v12 derives `.ts` types next to serde; a `#[test]`
  exports them to `src/lib/bindings/`, committed and CI-drift-checked. A
  hand-written `src/lib/ipc.ts` wraps each `invoke`.
- **The sheet is the document.** Simple mutations (set HP, record a choice, add a
  class) are plain edits to the sheet object in TS, then re-`compute`. Only
  rules-heavy mutations (rest recharge) are Rust commands so logic stays in Rust.

### Tauri commands (thin; engine does the work)
| command | in → out | purpose |
|---|---|---|
| `catalog()` | → `Catalog` | summaries of all classes/subclasses/species/backgrounds/feats for pickers |
| `compute(sheet)` | `Sheet` → `ComputedCharacter` | the derived sheet (called on every edit) |
| `explain(sheet, statId)` | → `StatBreakdown` | the contribution tree for one stat (side panel) |
| `rest(sheet, kind)` | `Sheet,"short"\|"long"` → `Sheet` | recharge resources/hit dice per their rules |
| `new_sheet(name)` | → `Sheet` | blank level-0 character |
| `list_characters()` | → `[{name,path}]` | saved files in the characters dir |
| `load_character(path)` | → `Sheet` | read a character JSON |
| `save_character(path, sheet)` | → `()` | write a character JSON |

`Catalog` is a new lightweight engine type: `{ classes:[{id,name,hit_die,caster}],
subclasses:[{id,name,class}], species:[{id,name}], backgrounds:[{id,name,abilities}],
feats:[{id,name,category}] }` — pickers need names without loading whole defs.

## 1. Layout — the cockpit

A persistent 3-zone shell, dark, dense:

```
┌───────────────────────────────────────────────┬───────────────────┐
│ TOPBAR  name · species · class chips · L· HP·AC·Init · Save  ⚙    │
├──────────┬────────────────────────────────────┤  INSPECTOR        │
│ LEFT     │  MAIN GRID                          │  (side panel)     │
│ RAIL     │                                     │                   │
│ • Sheet  │  abilities | saves | skills         │  default: nothing │
│ • Build  │  combat (AC/HP/init/speed)          │  click a stat →   │
│ • Spells │  resources & dice                   │  its breakdown     │
│ • Gear   │  features / conditions / effects    │  tree (sources,   │
│ • Notes  │                                     │  op, value, when) │
│          │                                     │  build mode →     │
│          │                                     │  choice resolver  │
└──────────┴────────────────────────────────────┴───────────────────┘
```

- **Topbar:** identity + the four numbers you glance at (Level, HP current/max,
  AC, Initiative) + Save state. HP is click-to-edit (damage/heal/temp).
- **Left rail:** section nav (Sheet / Build / Spells / Gear / Notes).
- **Main grid:** the dense sheet (below).
- **Inspector (right side panel):** the signature feature. Empty until you click
  a stat; then it shows that stat's full **breakdown tree** and stays open as you
  click around. In Build mode it also hosts the **choice resolver**.

Responsive: under ~1100px the inspector becomes an overlay drawer.

## 2. The dense sheet (main grid)

Compact cards, no wasted space. Every numeric value is a **`<Stat>`** —
click-target that opens its breakdown in the inspector and shows a subtle "ⓘ"
on hover.

- **Abilities** (6): score, modifier, save proficiency dot. Click score → breakdown.
- **Saves** (6): bonus + adv/disadv glyph (▲▼ from `D20Test.adv`), prof dot.
- **Skills** (18): bonus, prof/expertise dot, governing ability, passive where relevant.
- **Combat:** AC, HP (current/max/temp), Initiative, Speed(s), Proficiency Bonus,
  Attacks/Action, Carrying Capacity.
- **Weapons:** per-weapon attack bonus, damage dice + bonus, mastery tag.
- **Resources & dice:** each pool as pips/counter (current/max, die size,
  recharge color) — superiority, energy, ki/focus, rage, channel divinity, lay on
  hands, bardic, sorcery, etc. Click ± to spend/restore.
- **Spellcasting:** per source — save DC, attack, prepared count; spell-slot pips per level.
- **Conditions / effects / exhaustion:** condition chips (toggle), active-effect
  toggles (Rage…), exhaustion 0–6 stepper, concentration indicator, inspiration,
  death-saves tracker (when at 0 HP).
- **Features:** grouped by source (species/class/subclass/background/feat), collapsible.
- **Errors:** if `ComputedCharacter.errors` is non-empty (e.g. a content cycle),
  a visible banner — authoring bugs never hide.

## 3. The Inspector (breakdown side panel)

The product's thesis made visible. For a clicked stat:

```
ARMOR CLASS = 17
  base   16   Item: Chain Mail
  add    +1   Feat: Defense        (while wearing armor)
  ─────────────────────────────────
  = 17
```

- Renders `StatBreakdown.lines`: value, source label, note, band; **dimmed** rows
  for non-applied lines (e.g. a Base that lost the max, a non-stacking dupe) — you
  see what was considered *and rejected*, not just the winner.
- d20 tests also show the advantage channel (which sources grant adv/disadv, and
  the resolved Flat/Advantage/Disadvantage) and any auto-fail/success.
- Header has the stat name + final total; pin/close controls.

## 4. Build mode

Build-first: you make a character, then play it. Driven entirely by the engine's
`pending_choices` — the UI never hardcodes a class's choices.

1. **Identity:** name, then pick **species → background → class(es)** from
   `catalog()`. Multiclass = add another class entry; level via +/−.
2. **Choice resolver** (inspector, Build mode): the engine returns
   `pending_choices` (subclass, ASI, skills, expertise, fighting style, metamagic,
   weapon mastery, named options…). Each renders the right control for its
   `ChoiceOptions` kind; selecting writes a `RecordedChoice` to the sheet and
   re-computes. A "N choices pending" badge guides you to a complete character.
3. **Live preview:** the cockpit updates instantly as choices resolve — you watch
   HP/AC/saves change with full provenance. This *is* the "understand my
   character" tool.

## 5. Play mode (runtime mutations)

Same sheet, now mutating runtime state on the sheet object:
- **HP:** damage / heal / temp HP (topbar editor). Damage prompts a concentration
  CON save when concentrating.
- **Resources:** spend/restore pips; **Short Rest / Long Rest** buttons call
  `rest()` (Rust) to recharge per each pool's rule and roll-free restore hit dice/slots.
- **Spell slots:** click to expend/restore; concentration set/drop.
- **Conditions / effects:** toggle chips; exhaustion stepper; inspiration; death saves.
- **Autosave** to the character's JSON file (debounced) + explicit Save.

## 6. Persistence

- Characters live as JSON in a `characters/` dir under the app data dir (Tauri
  path API); `list/load/save_character` commands. Import/Export = file dialog.
- The saved file is exactly the `CharacterSheet` — portable, diffable,
  hand-editable, backup-friendly (the user's stated requirement).

## 7. Build order (milestones)

- **M0 — Shell runs:** Tauri 2 + SvelteKit + Tailwind boot to an empty cockpit; ts-rs export wired.
- **M1 — Read-only cockpit:** `compute` a bundled sample → render the full dense
  sheet; click any stat → inspector breakdown. *Proves the core value.*
- **M2 — Play:** HP, resources, rests, conditions, effects, slots; autosave/load; character list.
- **M3 — Build:** identity pickers + choice resolver from `pending_choices`; create from scratch.
- **M4 — Polish:** keyboard nav, multiclass UX, errors banner, import/export, theming pass.

Each milestone is a runnable app; we stop and look after each.
