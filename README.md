# rpgman — a D&D 2024 character tracker

A desktop app for building and playing **Dungeons & Dragons 2024** characters,
built on one idea: **every number is traceable.** Click any stat — Armor Class,
HP, an attack bonus — and a side panel shows you *exactly* what makes it up:

```
Armor Class = 19
     16   Chain Mail
     +2   Shield
     +1   Defense (fighting style, while wearing armor)
```

No more wondering "wait, why is my AC 19?" — the app shows its work.

> ⚠️ **Early version.** It does a lot already (build any of the 12 classes, play
> with live HP/resources/rests), but it's a work in progress — see the
> [Roadmap](ROADMAP.md). Expect rough edges and please
> [report bugs](../../issues).

---

## Download & install (Windows)

1. Go to the **[Releases page](../../releases/latest)**.
2. Under **Assets**, download the installer ending in **`.msi`**
   (e.g. `rpgman_0.1.0_x64_en-US.msi`).
3. Double-click it to install.

**Windows will probably show a blue "Windows protected your PC" warning.** This
is normal — the app isn't code-signed (that costs money), so Windows doesn't
recognize the publisher. The app is safe; the warning just means "we didn't pay
Microsoft to vouch for this." To install anyway:

> Click **More info** → **Run anyway**.

That's it. Launch **rpgman** from your Start menu.

*(macOS / Linux builds may also be attached to releases. On macOS you may need to
right-click → Open the first time for the same unsigned-app reason.)*

---

## How to use it

When the app opens you'll see your **character library** (empty at first).

### Make a character
1. Click **+ New**. You land in **Build** mode (the ⚒ icon in the left rail).
2. **Identity** — give it a name, pick a **Species** and **Background**.
3. **Ability Scores** — set your six scores (there's a "standard array"
   quick-fill button). You'll see the final score update live as your
   species/background bonuses apply.
4. **Classes** — add a class (e.g. Fighter) and set its level. At level 3 a
   **Subclass** picker appears.
5. **Choices** — as you add levels, the app tells you what's left to decide
   (subclass, skills, fighting style, ability boosts, maneuvers…). The number
   badge on the **Build** tab counts them down. Resolve them all and your
   character is complete.

Your character **auto-saves**. Use **← Library** in the top-left to switch
characters, or **Save** in the top-right.

### Play with it
Switch to the **Sheet** tab (▦) for the full cockpit:

- **Hit Points** — take damage, heal, set temporary HP, quick ±1/±5 buttons.
- **Short Rest / Long Rest** — recharge your abilities, spell slots, and HP
  the way the rules say.
- **Resources & spell slots** — click the dots to spend or restore them
  (superiority dice, ki, rage, channel divinity, spell slots, …).
- **Status** — toggle conditions, track exhaustion, concentration, inspiration,
  and death saves.

### See *why* a number is what it is
This is the whole point. **Click any stat** — an ability, a save, AC, HP, a
weapon's attack or damage — and the **Inspector** on the right breaks it down
line by line, including bonuses that were *considered but didn't apply* (shown
struck through). It even shows the rules: weapon attack = ability modifier +
proficiency + magic + feats; damage leaves out proficiency, just like the books.

Other tabs: **Gear** (✦ add weapons, equip armor & shields — your AC updates
live), **Spells** (DC, attack bonus, slot tracking), **Notes** (✎ free-form
journal that saves with the character).

---

## What it covers today

- **All 12 D&D 2024 classes**, each with a subclass (Battle Master, Evoker, Life
  Domain, Thief, Oath of Devotion, Hunter, Open Hand, Draconic, Fiend, Lore,
  Circle of the Moon, Psi Warrior).
- **Species** (Human, Elf, Dwarf), **backgrounds**, **origin & general feats**.
- Multiclassing, the full level-up choice flow, and live play (HP, resources,
  rests, conditions, spell slots).
- **Honest gap:** there's no spell *database* yet — you track slots and save DCs,
  but can't search/pick spells from a list. That's high on the [roadmap](ROADMAP.md).

Targets the **CC-licensed System Reference Document 5.2** (D&D 2024). No
copyrighted text from the books is included.

---

## For developers

rpgman is **"just math": the rules live in editable JSON data files, and a Rust
engine interprets them.** Adding a class is adding a file, not writing code.

```
content/ (rules as JSON)   ─┐
                            ├─►  Rust engine (pure math) ─► Tauri commands ─► Svelte UI
CharacterSheet (the file)  ─┘     "every stat = a list of
                                   traceable contributions"
```

- **`crates/engine`** — pure Rust, no I/O. Every value is a `StatId` whose result
  is the reduction of its `Contribution`s; a contribution can reference another
  stat, so stats form a dependency graph. See
  [`content/FORMAT.md`](content/FORMAT.md) for the data schema.
- **`src-tauri`** — thin [Tauri 2](https://tauri.app) shell exposing the engine.
- **`src`** — [SvelteKit](https://kit.svelte.dev) + Tailwind UI. Types are shared
  Rust→TS via [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs).

### Run it from source

Prerequisites: [Rust](https://rustup.rs), [Node 20+](https://nodejs.org), and the
[Tauri system dependencies](https://tauri.app/start/prerequisites/) for your OS.

```sh
npm install
npm run tauri dev        # launches the app with hot reload
```

### Tests

```sh
cargo test -p rpgman-engine                    # the rules engine (38 tests)
cargo test -p rpgman                           # the app commands
npm run check                                  # frontend typecheck
cargo run -p rpgman-engine --example demo      # print sample characters with full provenance
cargo run -p rpgman-engine --example validate  # check every content file parses
```

### Add game content

No Rust needed — drop a JSON file in `content/{classes,subclasses,species,backgrounds,feats}/`
following [`content/FORMAT.md`](content/FORMAT.md). The engine picks it up, and
`cargo run -p rpgman-engine --example validate` checks it.

---

## License

Code: MIT. Game content is derived from the **SRD 5.2**, released by Wizards of
the Coast under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
rpgman is an unofficial fan project, not affiliated with or endorsed by Wizards
of the Coast.
