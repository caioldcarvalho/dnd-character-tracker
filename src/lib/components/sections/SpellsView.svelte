<script lang="ts">
  import Card from '../Card.svelte';
  import { signed } from '../../format';
  import { app } from '../../state.svelte';
  import type { SpellDef } from '$bindings';

  // ---- derived state ----

  const sources = $derived(app.computed?.spellcasting ?? []);
  const slots = $derived(app.computed?.spell_slots ?? []);
  const hasCasting = $derived(sources.length > 0);
  const concentration = $derived(app.sheet?.concentration ?? null);

  // Character class ids (e.g. ["wizard", "cleric"])
  const charClassIds = $derived((app.sheet?.classes ?? []).map((c: { class: string }) => c.class));

  const knownSpells = $derived<string[]>(app.sheet?.known_spells ?? []);
  const preparedSpells = $derived<string[]>(app.sheet?.prepared_spells ?? []);

  // ---- catalog ----

  let catalogLoading = $state(false);

  $effect(() => {
    if (!app.catalog && !catalogLoading && hasCasting) {
      catalogLoading = true;
      app.ensureCatalog().finally(() => { catalogLoading = false; });
    }
  });

  // All spells available to this character's classes
  const availableSpells = $derived<SpellDef[]>(
    app.catalog
      ? app.catalog.spells.filter((sp) =>
          sp.classes.some((cls) => charClassIds.includes(cls))
        )
      : []
  );

  // ---- browser / filter state ----

  let searchQuery = $state('');
  let levelFilter = $state<'all' | 'cantrip' | number>('all');
  let browserOpen = $state(true);
  let castListOpen = $state(true);

  // Filtered + grouped spells for the browser
  const filteredSpells = $derived<SpellDef[]>(
    availableSpells.filter((sp) => {
      const q = searchQuery.trim().toLowerCase();
      const nameMatch = !q || sp.name.toLowerCase().includes(q);
      let levelMatch = true;
      if (levelFilter === 'cantrip') levelMatch = sp.level === 0;
      else if (levelFilter !== 'all') levelMatch = sp.level === levelFilter;
      return nameMatch && levelMatch;
    })
  );

  // Group by level (0 = cantrips, 1-9)
  const spellsByLevel = $derived<Map<number, SpellDef[]>>(
    filteredSpells.reduce((map, sp) => {
      const list = map.get(sp.level) ?? [];
      list.push(sp);
      map.set(sp.level, list);
      return map;
    }, new Map<number, SpellDef[]>())
  );

  const sortedLevels = $derived([...spellsByLevel.keys()].sort((a, b) => a - b));

  // ---- cast list: known cantrips + prepared leveled spells ----

  const castableSpells = $derived<SpellDef[]>(
    app.catalog
      ? app.catalog.spells.filter((sp) => {
          if (sp.level === 0) return knownSpells.includes(sp.id);
          return preparedSpells.includes(sp.id);
        })
      : []
  );

  // ---- combat line helpers ----

  const ABILITY_ABBR: Record<string, string> = {
    str: 'STR', dex: 'DEX', con: 'CON', int: 'INT', wis: 'WIS', cha: 'CHA'
  };

  /** Build a compact combat summary string for a spell, e.g. "spell atk +5 • 1d10 fire" */
  function combatLine(sp: SpellDef): string | null {
    const sc = sources[0];
    if (!sc) return null;
    const parts: string[] = [];
    if (sp.attack != null) {
      const bonus = sc.attack_bonus != null ? signed(sc.attack_bonus) : '?';
      parts.push(`spell atk ${bonus}`);
    } else if (sp.save != null) {
      const abilAbbr = ABILITY_ABBR[sp.save.ability] ?? sp.save.ability.toUpperCase();
      parts.push(`${abilAbbr} save DC ${sc.save_dc ?? '?'}`);
    }
    if (sp.damage != null) {
      parts.push(`${sp.damage.dice} ${sp.damage.damage_type}`);
    }
    return parts.length > 0 ? parts.join(' • ') : null;
  }

  // ---- school abbreviations ----
  const SCHOOL_ABBR: Record<string, string> = {
    abjuration: 'Abj',
    conjuration: 'Conj',
    divination: 'Div',
    enchantment: 'Enc',
    evocation: 'Evo',
    illusion: 'Ill',
    necromancy: 'Nec',
    transmutation: 'Trs'
  };

  // ---- unique level filter options ----
  const uniqueLevels = $derived(
    [...new Set(availableSpells.map((sp) => sp.level))].sort((a, b) => a - b)
  );
</script>

<div class="flex flex-col gap-2">
  {#if !app.sheet}
    <Card title="Spellcasting">
      <p class="text-[11px] text-[var(--color-muted)]">No character loaded.</p>
    </Card>
  {:else if !hasCasting}
    <Card title="Spellcasting">
      <p class="text-[11px] text-[var(--color-muted)]">This character has no spellcasting.</p>
    </Card>
  {:else}

    <!-- Concentration banner -->
    {#if concentration}
      <div class="flex items-center gap-2 rounded border border-[var(--color-accent)]/40 bg-[var(--color-accent)]/10 px-2.5 py-1.5">
        <span class="text-[9px] uppercase tracking-wider text-[var(--color-accent)] font-semibold shrink-0">Concentrating</span>
        <span class="text-[11px] text-[var(--color-ink)] flex-1 truncate">{concentration}</span>
        <button
          type="button"
          onclick={() => app.setConcentration(null)}
          title="Drop concentration"
          class="rounded border border-[var(--color-border)] px-2 py-0.5 text-[10px] text-[var(--color-muted)] hover:text-[var(--color-bad)] hover:border-[var(--color-bad)] transition-colors"
        >Drop</button>
      </div>
    {/if}

    <!-- DC / Atk / Prep header + slot pips -->
    <Card title="Spellcasting">
      <div class="flex flex-col gap-3">
        <!-- Per-source stats -->
        <div class="flex flex-col gap-1.5">
          {#each sources as sc}
            <div class="flex items-center gap-2 text-[11px]">
              <span class="flex-1 font-semibold capitalize truncate">{sc.source}</span>
              <span class="flex items-center gap-1 text-[var(--color-muted)]">
                <span class="text-[9px] uppercase tracking-wider">DC</span>
                <span class="num font-semibold text-[var(--color-ink)]">{sc.save_dc ?? '—'}</span>
              </span>
              <span class="flex items-center gap-1 text-[var(--color-muted)]">
                <span class="text-[9px] uppercase tracking-wider">Atk</span>
                <span class="num font-semibold text-[var(--color-ink)]">
                  {sc.attack_bonus != null ? signed(sc.attack_bonus) : '—'}
                </span>
              </span>
              {#if sc.prepared != null}
                <span class="flex items-center gap-1 text-[var(--color-muted)]">
                  <span class="text-[9px] uppercase tracking-wider">Prep</span>
                  <span class="num font-semibold text-[var(--color-ink)]">{sc.prepared}</span>
                </span>
              {/if}
            </div>
          {/each}
        </div>

        <!-- Slot pips -->
        {#if slots.length}
          <div class="flex flex-col gap-1 pt-2 border-t border-[var(--color-border)]/60">
            {#each slots as s}
              <div class="flex items-center gap-2 text-[11px]">
                <span class="text-[var(--color-muted)] w-14">Level {s.level}</span>
                <span class="num text-[var(--color-ink)] w-12">{s.current}/{s.max}</span>
                <span class="flex flex-wrap gap-0.5">
                  {#each Array(s.max ?? 0) as _, i}
                    <button
                      type="button"
                      onclick={() =>
                        i < s.current ? app.expendSlot(s.level) : app.restoreSlot(s.level)}
                      title={i < s.current ? `Expend L${s.level} slot` : `Restore L${s.level} slot`}
                      aria-label={i < s.current
                        ? `Expend level ${s.level} slot`
                        : `Restore level ${s.level} slot`}
                      class="w-2.5 h-2.5 rounded-full transition-colors hover:ring-1 hover:ring-[var(--color-accent)] {i <
                      s.current
                        ? 'bg-[var(--color-accent)]'
                        : 'border border-[var(--color-border)] bg-[var(--color-panel)]'}"
                    ></button>
                  {/each}
                </span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </Card>

    <!-- Cast list: known cantrips + prepared spells -->
    <Card title="Prepared & Cantrips">
      <div class="flex flex-col gap-1">
        <!-- Collapsible header -->
        <button
          type="button"
          onclick={() => (castListOpen = !castListOpen)}
          class="flex items-center gap-1.5 w-full text-left text-[10px] text-[var(--color-muted)] hover:text-[var(--color-ink)] transition-colors -mt-0.5 mb-0.5"
        >
          <span class="text-[9px]">{castListOpen ? '▾' : '▸'}</span>
          <span>{castableSpells.length} spell{castableSpells.length !== 1 ? 's' : ''} ready to cast</span>
        </button>

        {#if castListOpen}
          {#if catalogLoading}
            <p class="text-[10px] text-[var(--color-muted)] py-1">Loading spells…</p>
          {:else if castableSpells.length === 0}
            <p class="text-[10px] text-[var(--color-muted)] py-1">
              No spells prepared. Use the browser below to mark spells as Known or Prepared.
            </p>
          {:else}
            {#each castableSpells as sp (sp.id)}
              <div class="flex items-center gap-1.5 rounded px-1.5 py-1 hover:bg-[var(--color-panel-2)] transition-colors group">
                <!-- Level badge -->
                {#if sp.level === 0}
                  <span class="text-[9px] font-semibold text-[var(--color-accent)] w-5 shrink-0 text-center">C</span>
                {:else}
                  <span class="num text-[9px] font-semibold text-[var(--color-muted)] w-5 shrink-0 text-center">{sp.level}</span>
                {/if}

                <!-- Name + meta -->
                <span class="flex-1 min-w-0">
                  <span class="text-[11px] font-medium text-[var(--color-ink)] truncate block">{sp.name}</span>
                  <span class="text-[9px] text-[var(--color-muted)] block">
                    {sp.casting_time} · {sp.range}{sp.concentration ? ' · Conc' : ''}{sp.ritual ? ' · Ritual' : ''}
                  </span>
                  {#if combatLine(sp)}
                    <span class="text-[9px] text-[var(--color-accent)] block num">{combatLine(sp)}</span>
                  {/if}
                </span>

                <!-- Cast button (primary) -->
                <button
                  type="button"
                  onclick={() => app.castSpell({ id: sp.id, name: sp.name, level: sp.level, concentration: sp.concentration, attack: sp.attack, save: sp.save, damage: sp.damage })}
                  class="shrink-0 rounded bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
                >Cast</button>
              </div>
            {/each}
          {/if}
        {/if}
      </div>
    </Card>

    <!-- Spell browser -->
    <Card title="Spell Browser">
      <!-- Toggle header -->
      <div class="flex flex-col gap-2">
        <button
          type="button"
          onclick={() => (browserOpen = !browserOpen)}
          class="flex items-center gap-1.5 w-full text-left text-[10px] text-[var(--color-muted)] hover:text-[var(--color-ink)] transition-colors -mt-0.5"
        >
          <span class="text-[9px]">{browserOpen ? '▾' : '▸'}</span>
          <span>{availableSpells.length} spells available for your class{charClassIds.length !== 1 ? 'es' : ''}</span>
        </button>

        {#if browserOpen}
          <!-- Search + filter controls -->
          <div class="flex gap-1.5 flex-wrap">
            <input
              type="search"
              placeholder="Search spells…"
              bind:value={searchQuery}
              class="flex-1 min-w-[120px] bg-[var(--color-panel-2)] border border-[var(--color-border)] rounded px-2 py-1 text-[11px] focus:border-[var(--color-accent)] focus:outline-none placeholder:text-[var(--color-muted)]/60"
            />
            <select
              bind:value={levelFilter}
              class="bg-[var(--color-panel-2)] border border-[var(--color-border)] rounded px-2 py-1 text-[11px] focus:border-[var(--color-accent)] focus:outline-none"
            >
              <option value="all">All levels</option>
              {#if uniqueLevels.includes(0)}
                <option value="cantrip">Cantrips</option>
              {/if}
              {#each uniqueLevels.filter((l) => l > 0) as lvl}
                <option value={lvl}>Level {lvl}</option>
              {/each}
            </select>
          </div>

          {#if catalogLoading}
            <p class="text-[10px] text-[var(--color-muted)] py-2">Loading catalog…</p>
          {:else if availableSpells.length === 0}
            <p class="text-[10px] text-[var(--color-muted)] py-2">No spells match your class selection.</p>
          {:else if filteredSpells.length === 0}
            <p class="text-[10px] text-[var(--color-muted)] py-2">No spells match your search.</p>
          {:else}
            <!-- Grouped by level -->
            {#each sortedLevels as lvl}
              {@const spellsForLevel = spellsByLevel.get(lvl) ?? []}
              <div class="flex flex-col gap-0.5">
                <!-- Level heading -->
                <div class="flex items-center gap-1.5 pt-1.5 pb-0.5 border-t border-[var(--color-border)]/60 first:border-t-0 first:pt-0">
                  <span class="text-[9px] uppercase tracking-wider font-semibold text-[var(--color-muted)]">
                    {lvl === 0 ? 'Cantrips' : `Level ${lvl}`}
                  </span>
                  <span class="text-[9px] text-[var(--color-muted)]/60">({spellsForLevel.length})</span>
                </div>

                {#each spellsForLevel as sp (sp.id)}
                  {@const isKnown = knownSpells.includes(sp.id)}
                  {@const isPrepared = preparedSpells.includes(sp.id)}
                  <div class="flex items-center gap-1.5 rounded px-1.5 py-1 hover:bg-[var(--color-panel-2)] transition-colors">
                    <!-- Name + school + markers -->
                    <span class="flex-1 min-w-0 flex items-baseline gap-1.5">
                      <span class="text-[11px] text-[var(--color-ink)] font-medium truncate">{sp.name}</span>
                      <span class="text-[9px] text-[var(--color-muted)] shrink-0">{SCHOOL_ABBR[sp.school] ?? sp.school}</span>
                      {#if sp.concentration}
                        <span class="text-[8px] text-[var(--color-warn)] shrink-0" title="Concentration">C</span>
                      {/if}
                      {#if sp.ritual}
                        <span class="text-[8px] text-[var(--color-good)] shrink-0" title="Ritual">R</span>
                      {/if}
                    </span>

                    <!-- Known toggle -->
                    <button
                      type="button"
                      onclick={() => app.toggleKnownSpell(sp.id)}
                      title={isKnown ? 'Remove from known' : 'Mark as known'}
                      class="shrink-0 rounded border px-1.5 py-0.5 text-[9px] font-semibold transition-colors {isKnown
                        ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/15 text-[var(--color-accent)]'
                        : 'border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-ink)] hover:border-[var(--color-border)]'}"
                    >Kn</button>

                    <!-- Prepared toggle (only for leveled spells) -->
                    {#if sp.level > 0}
                      <button
                        type="button"
                        onclick={() => app.togglePreparedSpell(sp.id)}
                        title={isPrepared ? 'Remove from prepared' : 'Mark as prepared'}
                        class="shrink-0 rounded border px-1.5 py-0.5 text-[9px] font-semibold transition-colors {isPrepared
                          ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/15 text-[var(--color-accent)]'
                          : 'border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-ink)] hover:border-[var(--color-border)]'}"
                      >Pr</button>
                    {:else}
                      <!-- Placeholder to keep alignment -->
                      <span class="w-[30px] shrink-0"></span>
                    {/if}
                  </div>
                {/each}
              </div>
            {/each}
          {/if}
        {/if}
      </div>
    </Card>

  {/if}
</div>
