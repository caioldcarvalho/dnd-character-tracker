# Roadmap

What's done, what's planned. Rough priority order within each section. This is a
living document — open an [issue](../../issues) to suggest or vote on things.

## ✅ Done

- Contribution-graph rules engine (every stat is a traceable list of inputs).
- All 12 D&D 2024 classes + one subclass each, as data files.
- Species (Human/Elf/Dwarf), backgrounds, origin & general feats.
- Build flow: identity → abilities → multiclass → data-driven choice resolver.
- Play mode: HP, temp HP, resources, spell slots, short/long rests, conditions,
  exhaustion, concentration, inspiration, death saves.
- The Inspector: click any stat (incl. AC, HP, weapon attack/damage) to see its
  full breakdown, including dropped/non-applied contributions.
- Gear editor (weapons + armor + shield feeding AC), Notes, Spells (slots/DC).
- Self-contained desktop builds (content baked into the binary).

## 🔜 Next up

### Content depth
- [ ] **More subclasses per class.** Right now each class has exactly one
      subclass; add the rest of the SRD 5.2 subclasses (and make the subclass
      picker show more than one option).
- [ ] **A real spell database.** The biggest gap. Spells are referenced (slots,
      prepared counts, save DCs) but you can't browse/search/pick them. Model the
      SRD spell list as data and add a spell picker + prepared-spell management.
- [ ] **Magic items & a proper inventory.** Beyond weapons/armor: attunement,
      consumables, currency, items that grant features/contributions.
- [ ] More **species** and **backgrounds** from the SRD.

### The Inspector / breakdowns
- [ ] **Show ability & feature *descriptions* in the side panel** — when you
      inspect a stat or a feature, show the rules text for the abilities feeding
      it, alongside the numeric contributions (and only when a description
      exists). Makes the breakdown teach the rules, not just show math.
- [ ] Inspect **resources and spell slots** (what sets the max, e.g. "2 ×
      proficiency bonus") the same way stats are inspectable.
- [ ] Surface **advantage/disadvantage sources** in the Inspector for d20 tests
      (the engine already tracks them; the UI doesn't show the "why" yet).

### Play & usability
- [ ] **Hit-dice spending during a short rest** (roll to heal), not just the
      pool display.
- [ ] **Weapon Mastery** rules text + the on-attack effects (Vex, Topple, …).
- [ ] Dice roller integrated with attacks/saves/skills.
- [ ] Keyboard shortcuts and better multiclass level-up UX.
- [ ] **Import / export** a character as a file to share with friends.

### Polish & distribution
- [ ] Theming (light mode, accent colors).
- [ ] In-app content editor (homebrew classes/feats without touching JSON).
- [ ] Code-sign the Windows/macOS builds so there's no "unknown publisher"
      warning (needs a paid certificate).
- [ ] Auto-update.

## 💡 Ideas / maybe

- Character sheet PDF export.
- Encounter/initiative tracker for running a table.
- Older editions (2014 5e) as an alternate content set — the engine is
  edition-agnostic; it's a matter of authoring the data.
- Sharing/syncing characters across devices.

---

*The architecture makes most "content" items (subclasses, spells, items,
species) additive — they're data files, not code. That's intentional: the hard
part (the engine) is done, so breadth is mostly authoring.*
