<script lang="ts">
  import Card from '../Card.svelte';
  import { signed } from '../../format';
  import { app, toasts } from '../../state.svelte';
  import type { SpellDef } from '$bindings';

  // ---- attacks ----

  const weapons = $derived(app.computed?.weapons ?? []);

  /** Roll a d20 + attack bonus, then roll damage dice. Push a toast. */
  function rollAttack(w: any) {
    // d20 roll
    const d20 = Math.floor(Math.random() * 20) + 1;
    const isCrit = d20 === 20;
    const isFumble = d20 === 1;
    const toHit = d20 + (w.attack_bonus ?? 0);

    // Damage roll (count * sides, doubled on crit per 2024 rules)
    const count = isCrit ? (w.damage?.count ?? 1) * 2 : (w.damage?.count ?? 1);
    const sides = w.damage?.sides ?? 6;
    let dmgRoll = 0;
    for (let i = 0; i < count; i++) {
      dmgRoll += Math.floor(Math.random() * sides) + 1;
    }
    const dmgBonus = w.damage_bonus ?? 0;
    const dmgTotal = Math.max(0, dmgRoll + dmgBonus);

    if (isCrit) {
      toasts.push(`${w.name}: CRIT! ${toHit} to hit, ${dmgTotal} ${w.damage_type ?? 'damage'}`, 'good');
    } else if (isFumble) {
      toasts.push(`${w.name}: Nat 1 — miss`, 'warn');
    } else {
      toasts.push(`${w.name}: ${toHit} to hit, ${dmgTotal} ${w.damage_type ?? 'damage'}`, 'info');
    }
  }

  function dmgLabel(w: any): string {
    const d = `${w.damage?.count ?? 1}d${w.damage?.sides ?? 6}`;
    const bonus = (w.damage_bonus ?? 0) !== 0 ? ` ${signed(w.damage_bonus)}` : '';
    return `${d}${bonus} ${w.damage_type ?? ''}`.trim();
  }

  // ---- spells ----

  const sources = $derived(app.computed?.spellcasting ?? []);
  const hasCasting = $derived(sources.length > 0);
  const knownSpells = $derived<string[]>(app.sheet?.known_spells ?? []);
  const preparedSpells = $derived<string[]>(app.sheet?.prepared_spells ?? []);
  const charClassIds = $derived((app.sheet?.classes ?? []).map((c: { class: string }) => c.class));

  /** Spells ready to cast: known cantrips + prepared leveled spells. */
  const castableSpells = $derived<SpellDef[]>(
    app.catalog
      ? app.catalog.spells.filter((sp) => {
          if (sp.level === 0) return knownSpells.includes(sp.id);
          return preparedSpells.includes(sp.id);
        })
      : []
  );

  const ABILITY_ABBR: Record<string, string> = {
    str: 'STR', dex: 'DEX', con: 'CON', int: 'INT', wis: 'WIS', cha: 'CHA'
  };

  function combatLine(sp: SpellDef): string | null {
    const sc = sources[0];
    if (!sc) return null;
    const parts: string[] = [];
    if (sp.attack != null) {
      const bonus = sc.attack_bonus != null ? signed(sc.attack_bonus) : '?';
      parts.push(`spell atk ${bonus}`);
    } else if (sp.save != null) {
      const abbr = ABILITY_ABBR[sp.save.ability] ?? sp.save.ability.toUpperCase();
      parts.push(`${abbr} save DC ${sc.save_dc ?? '?'}`);
    }
    if (sp.damage != null) {
      parts.push(`${sp.damage.dice} ${sp.damage.damage_type}`);
    }
    return parts.length > 0 ? parts.join(' · ') : null;
  }

  // ---- resources ----

  const resources = $derived(app.computed?.resources ?? []);

  // ---- visibility ----

  const hasWeapons = $derived(weapons.length > 0);
  const hasSpells = $derived(hasCasting && castableSpells.length > 0);
  const hasResources = $derived(resources.length > 0);
  const hasAnything = $derived(hasWeapons || hasSpells || hasResources);

  // ---- collapsible groups ----

  let attacksOpen = $state(true);
  let spellsOpen = $state(true);
  let resourcesOpen = $state(true);
</script>

{#if hasAnything}
  <Card title="Actions">
    <div class="flex flex-col gap-3">

      <!-- ATTACKS -->
      {#if hasWeapons}
        <div class="flex flex-col gap-1">
          <button
            type="button"
            onclick={() => (attacksOpen = !attacksOpen)}
            class="flex items-center gap-1.5 w-full text-left text-[9px] uppercase tracking-wider font-semibold text-[var(--color-muted)] hover:text-[var(--color-ink)] transition-colors"
          >
            <span class="text-[8px]">{attacksOpen ? '▾' : '▸'}</span>
            <span>Attacks</span>
            <span class="font-normal normal-case tracking-normal ml-1 opacity-60">{weapons.length}</span>
          </button>
          {#if attacksOpen}
            {#each weapons as w, i (i)}
              <div class="flex items-center gap-2 rounded px-1.5 py-1.5 hover:bg-[var(--color-panel-2)] transition-colors group">
                <div class="flex-1 min-w-0">
                  <div class="text-[11px] font-medium text-[var(--color-ink)] truncate">{w.name}</div>
                  <div class="text-[9px] text-[var(--color-muted)] num">
                    {signed(w.attack_bonus ?? 0)} to hit · {dmgLabel(w)}
                  </div>
                </div>
                <button
                  type="button"
                  onclick={() => rollAttack(w)}
                  class="shrink-0 rounded bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
                >Roll</button>
              </div>
            {/each}
          {/if}
        </div>
      {/if}

      <!-- SPELLS -->
      {#if hasSpells}
        {#if hasWeapons}
          <div class="border-t border-[var(--color-border)]/60"></div>
        {/if}
        <div class="flex flex-col gap-1">
          <button
            type="button"
            onclick={() => (spellsOpen = !spellsOpen)}
            class="flex items-center gap-1.5 w-full text-left text-[9px] uppercase tracking-wider font-semibold text-[var(--color-muted)] hover:text-[var(--color-ink)] transition-colors"
          >
            <span class="text-[8px]">{spellsOpen ? '▾' : '▸'}</span>
            <span>Spells</span>
            <span class="font-normal normal-case tracking-normal ml-1 opacity-60">{castableSpells.length}</span>
          </button>
          {#if spellsOpen}
            {#each castableSpells as sp (sp.id)}
              <div class="flex items-center gap-2 rounded px-1.5 py-1.5 hover:bg-[var(--color-panel-2)] transition-colors group">
                <!-- Level badge -->
                <span class="shrink-0 w-5 text-center">
                  {#if sp.level === 0}
                    <span class="text-[9px] font-bold text-[var(--color-accent)]">C</span>
                  {:else}
                    <span class="num text-[9px] font-semibold text-[var(--color-muted)]">{sp.level}</span>
                  {/if}
                </span>

                <!-- Name + combat line -->
                <div class="flex-1 min-w-0">
                  <div class="text-[11px] font-medium text-[var(--color-ink)] truncate">
                    {sp.name}
                    {#if sp.concentration}<span class="text-[var(--color-warn)] ml-1 text-[9px]" title="Concentration">C</span>{/if}
                  </div>
                  {#if combatLine(sp)}
                    <div class="text-[9px] text-[var(--color-accent)] num">{combatLine(sp)}</div>
                  {:else}
                    <div class="text-[9px] text-[var(--color-muted)]">{sp.casting_time} · {sp.range}</div>
                  {/if}
                </div>

                <!-- Cast button -->
                <button
                  type="button"
                  onclick={() => app.castSpell({
                    id: sp.id,
                    name: sp.name,
                    level: sp.level,
                    concentration: sp.concentration,
                    attack: sp.attack,
                    save: sp.save,
                    damage: sp.damage
                  })}
                  class="shrink-0 rounded bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
                >Cast</button>
              </div>
            {/each}
          {/if}
        </div>
      {/if}

      <!-- LIMITED-USE RESOURCES -->
      {#if hasResources}
        {#if hasWeapons || hasSpells}
          <div class="border-t border-[var(--color-border)]/60"></div>
        {/if}
        <div class="flex flex-col gap-1">
          <button
            type="button"
            onclick={() => (resourcesOpen = !resourcesOpen)}
            class="flex items-center gap-1.5 w-full text-left text-[9px] uppercase tracking-wider font-semibold text-[var(--color-muted)] hover:text-[var(--color-ink)] transition-colors"
          >
            <span class="text-[8px]">{resourcesOpen ? '▾' : '▸'}</span>
            <span>Limited Use</span>
            <span class="font-normal normal-case tracking-normal ml-1 opacity-60">{resources.length}</span>
          </button>
          {#if resourcesOpen}
            {#each resources as r (r.id)}
              {@const cur = r.current ?? r.max}
              <div class="flex items-center gap-2 rounded px-1.5 py-1.5 hover:bg-[var(--color-panel-2)] transition-colors">
                <div class="flex-1 min-w-0">
                  <div class="text-[11px] font-medium text-[var(--color-ink)] truncate">
                    {r.name}
                    {#if r.die}<span class="text-[var(--color-muted)] text-[9px] ml-1">d{r.die}</span>{/if}
                  </div>
                  <div class="text-[9px] text-[var(--color-muted)] num">{cur}/{r.max} remaining</div>
                </div>
                <button
                  type="button"
                  onclick={() => app.spendResource(r.id, r.max, 1)}
                  disabled={cur <= 0}
                  class="shrink-0 rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)] hover:border-[var(--color-accent)] transition-colors disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:border-[var(--color-border)] disabled:hover:text-[var(--color-muted)]"
                >Use</button>
              </div>
            {/each}
          {/if}
        </div>
      {/if}

    </div>
  </Card>
{/if}
