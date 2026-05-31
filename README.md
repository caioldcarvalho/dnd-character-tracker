# rpgman

A desktop D&D 2024 character manager built on one idea: **every stat is a
traceable list of contributions**, not an opaque formula. When your AC or HP is
"wrong", you can always see *what* contributed to it — and from where.

```
Armor Class = 17
     +16  Item: Chain Mail
      +1  Feat: Defense (while wearing armor)
```

The app is **just math**: the rules live in editable JSON data files, and a Rust
engine interprets them. Adding a class is adding a file, not writing code.

## Architecture

```
ContentDb (rules as data, content/)  ─┐
                                      ├─►  engine (pure Rust)  ─►  ComputedCharacter
CharacterSheet (the saved file)      ─┘                          (+ breakdown trees)
```

- **`crates/engine`** — pure Rust, no I/O. Every computable value is a `StatId`
  whose value is the reduction of its `Contribution`s; a contribution's value can
  reference another stat, so stats form a dependency DAG. Op-precedence bands
  (base/add/multiply/floor/cap/override) encode the D&D 2024 rules (e.g. AC takes
  the *max* of competing bases, then adds shield). Memoized evaluator with cycle
  detection; advantage/disadvantage tracked as a separate non-numeric channel.
- **`content/`** — the rulebook as data: classes, subclasses, species,
  backgrounds, feats. See [`content/FORMAT.md`](content/FORMAT.md) for the schema.
- **`src-tauri` + `src`** — Tauri 2 shell + SvelteKit view layer *(not built yet)*.

Targets **D&D 2024 (One D&D)**, based on the CC-licensed SRD 5.2.

## Status

- ✅ Engine complete — 33 tests passing, 0 warnings.
- ✅ All 12 classes + 12 subclasses authored as data; 3 species, 4 backgrounds, feats.
- ⬜ Desktop GUI (Tauri + Svelte) — next.

## Develop

```sh
cargo test  -p rpgman-engine                      # run the test suite
cargo run   -p rpgman-engine --example demo       # print sample characters with provenance
cargo run   -p rpgman-engine --example validate   # check every content file parses
```

## Why

Online character sheet managers throw a formula at each stat and you lose track
of what's accounted for. Modeling each value as an array of named contributions
gives full control and a complete audit trail — and makes the whole game
expressible as data.
