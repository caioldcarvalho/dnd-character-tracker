# ▶ START HERE — next session pickup

**This file is the entry point for the next agent. Read it first.** Current as of
the `feat/play-loop` merge into `main`.

---

## Where we are
Two big efforts are **merged into `main`** and **pushed to `origin`**:
1. **UX overhaul + notes + spell system** (`feat/ux-overhaul`).
2. **Play-loop completion** (`feat/play-loop`) — the entire review backlog.

The full UX-review backlog is now **DONE**. Verified each time:
svelte-check 0 errors, `cargo test -p rpgman-engine` green, `… --example validate` clean.

Shipped:
- **Notes** — structured `Vec<Note>` cards (categories, pin, search, modal).
- **First-run welcome + Library hub** (cards, open/duplicate/rename/delete).
- **Class-aware ability assignment** (Manual / Standard Array / Point Buy; recommends the primary stat; consistent score-big display).
- **Build-flow guidance** — "Step X of Y" progress, resolved-choice summaries, option descriptions.
- **Rest/HP** — hit dice heal on spend; long rest regains ½ HD; rest/hit-die/level-up toasts. (Single hit-dice control in RestBar.)
- **Spell system** — `SpellDef` content, **122 curated SRD 5.2 spells**, `SpellsView` (browser, known/prepared, cast-from-slot, concentration banner), and **combat data** (attack/save/damage) shown on cast.
- **Conditions affect rolls** — poisoned/frightened/prone/restrained/blinded/invisible/etc. apply advantage/disadvantage via the engine d20 channels (traceable), with effect labels in the Conditions panel. (14 tests in `crates/engine/tests/conditions.rs`.)
- **Inspector** — demoted from a permanent rail to an on-demand overlay (Esc/✕ to dismiss); the cockpit is now full-width.
- **Actions-first surface** — `src/lib/components/panels/Actions.svelte` at the top of the play cockpit: **Roll** attacks (d20+atk, crit/nat-1 aware), **Cast** spells (spends slot + shows combat line), **Use** limited-use resources.

Background/methodology + the original prioritized plan: `.dev/review/IMPLEMENTATION_PLAN.md` (gitignored; on this machine only). Running log: the `rpgman-status` auto-memory.

---

## ▶ Suggested next work (extensions / polish — nothing is broken)
No urgent pickup. Reasonable next steps, roughly cheapest → priciest:
1. **Auto-fail saves** for paralyzed/stunned/unconscious/petrified — currently modeled as *disadvantage* on STR/DEX saves (noted as a simplification in `build.rs::conditions`). Make them true auto-fail. Small engine change.
2. **Apply condition advantage/disadvantage to the Actions "Roll"** — the engine already computes adv/dis from conditions; the frontend attack Roll in `Actions.svelte` rolls a flat d20. Surface the d20 adv/dis state to the roll. Small–medium.
3. **Spell upcasting / higher-level damage scaling** — `SpellDef.damage` is base only; cantrip-by-level and upcast scaling aren't modeled. Medium.
4. **More content** — only 1 subclass per class; spells only through level 2; 3 species / 4 backgrounds. Author more (per-file content agents worked well). Medium → large.
5. **Magic items, import/export, theming, code-signing** — larger product work.

---

## Run / verify
- Web build: `npm run build:wasm`, then `npm run dev` (Vite, **port 1420**, strictPort — `fuser -k 1420/tcp` to clear). Drive via the **chrome-devtools MCP** at `http://localhost:1420/`.
- Checks: `npm run check` (svelte-check) and `cargo test -p rpgman-engine`.
- After changing engine Rust `#[ts]` types, regenerate bindings: `cargo test -p rpgman-engine --features ts`.
- Content under `content/` is embedded at build time via `crates/engine/build.rs` (`kinds` array lists loaded subdirs incl. `spells`); `cargo run -p rpgman-engine --example validate` checks files parse.

### Gotchas
- `isolation:"worktree"` agents can branch from a slightly stale base — sequential agents on the same file conflict on merge-back; resolve by taking the superseding version (content: `--theirs`; code: hand-merge). For a sequential task that depends on just-merged changes, run the agent WITHOUT a worktree (directly on the branch).
- `/src/lib/wasm` is gitignored (dir + bare symlink) — never commit it.
- Commit messages: NO "co-authored by" trailer (user rule).
