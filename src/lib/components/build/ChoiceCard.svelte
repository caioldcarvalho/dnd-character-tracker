<script lang="ts">
  // Renders ONE pending choice and the control to resolve it. Every
  // ChoiceOptions kind is handled; unknown kinds fall back to a flat chip list.
  import { app } from '../../state.svelte';

  let { choice } = $props<{ choice: any }>();

  const ABILITIES = ['str', 'dex', 'con', 'int', 'wis', 'cha'];
  const ABILITY_LABEL: Record<string, string> = {
    str: 'STR',
    dex: 'DEX',
    con: 'CON',
    int: 'INT',
    wis: 'WIS',
    cha: 'CHA'
  };

  // kebab / lowercase id -> Title Case label.
  function title(id: string): string {
    return id.replace(/[-_]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  const kind = $derived(choice.options?.kind ?? 'unknown');
  const choose = $derived(choice.choose ?? 1);
  const picks = $derived<string[]>(app.picksFor(choice) ?? []);
  const resolved = $derived(picks.length === choose);

  // ---- generic multi-select (skills / expertise / named / weapon-mastery / spells / unknown) ----

  function toggleMulti(id: string) {
    const cur = app.picksFor(choice) ?? [];
    let next: string[];
    if (cur.includes(id)) next = cur.filter((p: string) => p !== id);
    else if (cur.length < choose) next = [...cur, id];
    else return; // at capacity; ignore (deselect still works above)
    app.resolveChoice(choice, next);
  }

  // ---- ability-score-increase (+2 one OR +1/+1 two-different) ----
  // Picks are ability ids; clicking adds an occurrence up to `choose`.
  // Clicking an already-present ability removes one occurrence (toggle-down).

  function asiClick(ab: string) {
    const cur = app.picksFor(choice) ?? [];
    const count = cur.filter((p: string) => p === ab).length;
    let next: string[];
    if (count > 0) {
      // remove one occurrence
      const idx = cur.indexOf(ab);
      next = [...cur.slice(0, idx), ...cur.slice(idx + 1)];
    } else if (cur.length < choose) {
      next = [...cur, ab];
    } else {
      return;
    }
    app.resolveChoice(choice, next);
  }

  function asiPlus(ab: string) {
    const cur = app.picksFor(choice) ?? [];
    if (cur.length >= choose) return;
    app.resolveChoice(choice, [...cur, ab]);
  }

  function asiCount(ab: string): number {
    return picks.filter((p) => p === ab).length;
  }

  // ---- ability-scores (background point spread) ----

  const points = $derived(choice.options?.points ?? 0);
  const maxEach = $derived(choice.options?.max_each ?? points);

  function asCount(ab: string): number {
    return picks.filter((p) => p === ab).length;
  }
  function asTotal(): number {
    return picks.length;
  }
  function asInc(ab: string) {
    const cur = app.picksFor(choice) ?? [];
    if (cur.length >= points) return;
    if (cur.filter((p: string) => p === ab).length >= maxEach) return;
    app.resolveChoice(choice, [...cur, ab]);
  }
  function asDec(ab: string) {
    const cur = app.picksFor(choice) ?? [];
    const idx = cur.indexOf(ab);
    if (idx < 0) return;
    app.resolveChoice(choice, [...cur.slice(0, idx), ...cur.slice(idx + 1)]);
  }

  // ---- feat picker ----

  const featOptions = $derived.by(() => {
    const feats = app.catalog?.feats ?? [];
    const cat = choice.options?.category ?? null;
    return cat == null ? feats : feats.filter((f: any) => f.category === cat);
  });

  // ---- subclass picker ----

  const subclassClassId = $derived(String(choice.key ?? '').replace(/-subclass$/, ''));
  const subclassOptions = $derived.by(() => {
    const subs = app.catalog?.subclasses ?? [];
    return subs.filter((s: any) => s.class === subclassClassId);
  });

  function pickOne(id: string) {
    app.resolveChoice(choice, id ? [id] : []);
  }
</script>

<div class="rounded border border-[var(--color-border)] bg-[var(--color-panel)] p-2 flex flex-col gap-1.5">
  <!-- header -->
  <div class="flex items-start justify-between gap-2">
    <div class="flex flex-col gap-0.5 min-w-0">
      <span class="text-[11px] font-semibold leading-tight">{choice.prompt}</span>
      <span class="text-[9px] uppercase tracking-wide text-[var(--color-muted)] truncate"
        >{choice.source}</span
      >
    </div>
    <div class="flex items-center gap-1.5 shrink-0">
      {#if resolved}
        <span class="text-[9px] font-semibold text-[var(--color-good)]">✓ resolved</span>
      {:else}
        <span class="text-[9px] text-[var(--color-muted)]">
          choose <span class="num">{choose}</span>
          <span class="text-[var(--color-muted)]">({picks.length}/{choose})</span>
        </span>
      {/if}
    </div>
  </div>

  <!-- control -->
  {#if kind === 'skills' || kind === 'expertise'}
    <div class="flex flex-wrap gap-1">
      {#each choice.options.from as id (id)}
        {@const on = picks.includes(id)}
        {@const full = picks.length >= choose && !on}
        <button
          type="button"
          onclick={() => toggleMulti(id)}
          disabled={full}
          class="px-2 py-0.5 rounded text-[10px] border transition-colors
            {on
            ? 'bg-[var(--color-accent)] text-[var(--color-panel)] border-[var(--color-accent)]'
            : full
              ? 'bg-[var(--color-panel-2)] border-[var(--color-border)] text-[var(--color-muted)] opacity-40 cursor-not-allowed'
              : 'bg-[var(--color-panel-2)] border-[var(--color-border)] hover:border-[var(--color-accent)]'}"
        >
          {title(id)}
        </button>
      {/each}
    </div>
  {:else if kind === 'ability-score-increase'}
    <div class="flex flex-wrap gap-1">
      {#each ABILITIES as ab (ab)}
        {@const c = asiCount(ab)}
        {@const full = picks.length >= choose && c === 0}
        <button
          type="button"
          onclick={() => asiClick(ab)}
          oncontextmenu={(e) => {
            e.preventDefault();
            asiPlus(ab);
          }}
          disabled={full}
          title="Click to add/remove; right-click to add a second +1 (for +2)"
          class="px-2 py-0.5 rounded text-[10px] border flex items-center gap-1 transition-colors
            {c > 0
            ? 'bg-[var(--color-accent)] text-[var(--color-panel)] border-[var(--color-accent)]'
            : full
              ? 'bg-[var(--color-panel-2)] border-[var(--color-border)] text-[var(--color-muted)] opacity-40 cursor-not-allowed'
              : 'bg-[var(--color-panel-2)] border-[var(--color-border)] hover:border-[var(--color-accent)]'}"
        >
          {ABILITY_LABEL[ab]}
          {#if c > 0}<span class="num font-semibold">+{c}</span>{/if}
        </button>
      {/each}
    </div>
    <span class="text-[9px] text-[var(--color-muted)]"
      >Click an ability to add +1; right-click to stack a second +1 on the same ability for +2.</span
    >
  {:else if kind === 'ability-scores'}
    {@const total = asTotal()}
    <div class="flex flex-col gap-1">
      {#each choice.options.from as ab (ab)}
        {@const c = asCount(ab)}
        <div class="flex items-center gap-2">
          <span class="text-[10px] w-10 text-[var(--color-muted)]">{ABILITY_LABEL[ab] ?? title(ab)}</span>
          <button
            type="button"
            onclick={() => asDec(ab)}
            disabled={c <= 0}
            class="w-5 h-5 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[11px] leading-none hover:border-[var(--color-accent)] disabled:opacity-30 disabled:cursor-not-allowed"
            >−</button
          >
          <span class="num w-6 text-center text-[11px] font-semibold">+{c}</span>
          <button
            type="button"
            onclick={() => asInc(ab)}
            disabled={total >= points || c >= maxEach}
            class="w-5 h-5 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[11px] leading-none hover:border-[var(--color-accent)] disabled:opacity-30 disabled:cursor-not-allowed"
            >+</button
          >
        </div>
      {/each}
      <span class="text-[9px] text-[var(--color-muted)]"
        >points used <span class="num">{total}</span>/<span class="num">{points}</span> · max
        <span class="num">{maxEach}</span> each</span
      >
    </div>
  {:else if kind === 'feat'}
    <select
      class="bg-[var(--color-panel-2)] border border-[var(--color-border)] rounded px-2 py-1 text-[11px] w-full"
      value={picks[0] ?? ''}
      onchange={(e) => pickOne((e.currentTarget as HTMLSelectElement).value)}
    >
      <option value="">— pick a feat —</option>
      {#if app.catalog == null}
        <option value="" disabled>loading…</option>
      {/if}
      {#each featOptions as f (f.id)}
        <option value={f.id}>{f.name}</option>
      {/each}
    </select>
  {:else if kind === 'subclass'}
    <select
      class="bg-[var(--color-panel-2)] border border-[var(--color-border)] rounded px-2 py-1 text-[11px] w-full"
      value={picks[0] ?? ''}
      onchange={(e) => pickOne((e.currentTarget as HTMLSelectElement).value)}
    >
      <option value="">— pick a subclass —</option>
      {#if app.catalog == null}
        <option value="" disabled>loading…</option>
      {/if}
      {#each subclassOptions as s (s.id)}
        <option value={s.id}>{s.name}</option>
      {/each}
    </select>
  {:else if kind === 'named'}
    <div class="flex flex-wrap gap-1">
      {#each choice.options.from as opt (opt.id)}
        {@const on = picks.includes(opt.id)}
        {@const full = picks.length >= choose && !on}
        <button
          type="button"
          onclick={() => toggleMulti(opt.id)}
          disabled={full}
          class="px-2 py-0.5 rounded text-[10px] border transition-colors
            {on
            ? 'bg-[var(--color-accent)] text-[var(--color-panel)] border-[var(--color-accent)]'
            : full
              ? 'bg-[var(--color-panel-2)] border-[var(--color-border)] text-[var(--color-muted)] opacity-40 cursor-not-allowed'
              : 'bg-[var(--color-panel-2)] border-[var(--color-border)] hover:border-[var(--color-accent)]'}"
        >
          {opt.name}
        </button>
      {/each}
    </div>
  {:else if kind === 'weapon-mastery' || kind === 'spells'}
    <div class="flex flex-wrap gap-1">
      {#each choice.options.from as id (id)}
        {@const on = picks.includes(id)}
        {@const full = picks.length >= choose && !on}
        <button
          type="button"
          onclick={() => toggleMulti(id)}
          disabled={full}
          class="px-2 py-0.5 rounded text-[10px] border transition-colors
            {on
            ? 'bg-[var(--color-accent)] text-[var(--color-panel)] border-[var(--color-accent)]'
            : full
              ? 'bg-[var(--color-panel-2)] border-[var(--color-border)] text-[var(--color-muted)] opacity-40 cursor-not-allowed'
              : 'bg-[var(--color-panel-2)] border-[var(--color-border)] hover:border-[var(--color-accent)]'}"
        >
          {title(id)}
        </button>
      {/each}
    </div>
  {:else}
    <!-- unknown kind: best-effort flat chip list from options.from if present -->
    <div class="flex flex-wrap gap-1">
      {#if Array.isArray(choice.options?.from)}
        {#each choice.options.from as raw (typeof raw === 'string' ? raw : raw.id)}
          {@const id = typeof raw === 'string' ? raw : raw.id}
          {@const lbl = typeof raw === 'string' ? title(raw) : (raw.name ?? title(raw.id))}
          {@const on = picks.includes(id)}
          {@const full = picks.length >= choose && !on}
          <button
            type="button"
            onclick={() => toggleMulti(id)}
            disabled={full}
            class="px-2 py-0.5 rounded text-[10px] border transition-colors
              {on
              ? 'bg-[var(--color-accent)] text-[var(--color-panel)] border-[var(--color-accent)]'
              : full
                ? 'bg-[var(--color-panel-2)] border-[var(--color-border)] text-[var(--color-muted)] opacity-40 cursor-not-allowed'
                : 'bg-[var(--color-panel-2)] border-[var(--color-border)] hover:border-[var(--color-accent)]'}"
          >
            {lbl}
          </button>
        {/each}
      {:else}
        <span class="text-[9px] text-[var(--color-muted)]"
          >Unsupported choice "{kind}" — choose <span class="num">{choose}</span>.</span
        >
      {/if}
    </div>
  {/if}
</div>
