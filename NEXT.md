# ▶ START HERE — next session pickup

**This file is the entry point for the next agent. Read it first, then begin with
the "DO THIS FIRST" task below.** Current as of the UX-overhaul merge into `main`.

---

## Where we are
The **UX overhaul + notes + spell system is merged into `main`** (merge commit
`Merge feat/ux-overhaul…`). It is **local only — NOT pushed** to `origin`, and the
merged `feat/ux-overhaul` branch is **not deleted**.

Done & verified (svelte-check 0 errors, `cargo test -p rpgman-engine` green):
- **Notes** — structured `Vec<Note>`: categories, pin, search, view/edit modal.
- **First-run welcome + Library hub** — cards with class/level/HP, open/duplicate/rename/delete.
- **Class-aware ability assignment** — Manual / Standard Array / Point Buy, recommends the class's primary stat; effective-score-big display in both build & play.
- **Build-flow guidance** — "Step X of Y" forward progress, resolved-choice summaries, option descriptions.
- **Rest/HP** — hit dice HEAL on spend; long rest regains ½ hit dice; toasts for rest / hit-die / level-up.
- **Spell system** — `SpellDef` content model, **122 curated SRD 5.2 spells** (35 cantrips + 45 L1 + 42 L2), catalog exposure, `known_spells`/`prepared_spells` on the sheet, and a full `SpellsView` (class-filtered searchable browser, known/prepared toggles, cast-from-slot, concentration banner).

Full methodology + backlog: `.dev/review/IMPLEMENTATION_PLAN.md` (gitignored; on this machine only).
Running project log: the `rpgman-status` auto-memory.

---

## ▶ DO THIS FIRST  — chosen as the LOWEST-TOKEN task
**Remove the duplicate hit-dice control.** There are now two hit-dice "spend"
controls:
- KEEP: `src/lib/components/panels/RestBar.svelte` — the "HIT DICE" row; spending **heals** (roll + CON) and is the intended control.
- REMOVE: the hit-dice entry in `src/lib/components/panels/Resources.svelte` (the "RESOURCES & DICE → Hit Dice / Spend d{n} hit die" block, around lines 7, 23, 88–100). Delete just the hit-dice part; leave the other resource pools intact.

Then verify: `npm run check` must report **0 errors**.

**Why this one:** it's the cheapest remaining item — a few lines in a single
file, no engine/content/bindings work — so it lands fastest and removes a real
inconsistency (two controls, one source of truth).

---

## Remaining backlog (after the quick win), roughly cheapest → priciest
1. ✅ next: duplicate hit-dice control — **tiny** (the DO-THIS-FIRST above).
2. Spell **cast** is currently qualitative (toast only). Wire damage / attack-vs-AC / save into `castSpell` + show it. **Medium** (needs spell combat fields in `SpellDef` + engine/UI).
3. Demote the **Inspector** from a permanent right rail to an on-demand popover so the action surface gets the space. **Medium**, cross-cutting layout.
4. **Conditions that actually modify rolls** (advantage/disadvantage; reflected in the Inspector). **Large** — engine work in the contribution / d20 graph.
5. **Actions-first PLAY surface** — attacks/cantrips/spells/limited-use features as tappable cards that resolve. **Largest**, design-led.

---

## Run / verify
- Web build: `npm run build:wasm`, then `npm run dev` (Vite, **port 1420**, strictPort — `fuser -k 1420/tcp` to clear). Drive via the **chrome-devtools MCP** at `http://localhost:1420/`.
- Checks: `npm run check` (svelte-check) and `cargo test -p rpgman-engine`.
- After changing engine Rust types, regenerate TS bindings: `cargo test -p rpgman-engine --features ts`.
- Content lives in `content/` (embedded at build time via `crates/engine/build.rs`; the `kinds` array lists the loaded subdirs, incl. `spells`). `cargo run -p rpgman-engine --example validate` checks files parse.

### Gotchas
- `isolation:"worktree"` agents can branch from a slightly stale base — sequential agents touching the same file conflict on merge-back; resolve by taking the superseding version (content: `--theirs`; code: hand-merge). For a sequential task that depends on just-merged changes, run the agent WITHOUT a worktree (directly on the branch).
- `/src/lib/wasm` is gitignored (both the dir and a bare symlink) — never commit it.
- Commit messages: NO "co-authored by" trailer (user rule).

---

## Open decisions for the human
- Push `main` to `origin`? (currently unpushed, ahead by many commits)
- Delete the merged `feat/ux-overhaul` branch?
