<script lang="ts">
  import { signed } from '../format';
  import { app } from '../state.svelte';

  const c = $derived(app.computed);
  const sheet = $derived(app.sheet);

  const classChips = $derived(
    (sheet?.classes ?? []).map((e: any) => {
      const sub = e.subclass ? ` (${e.subclass})` : '';
      return `${e.class} ${e.level}${sub}`;
    })
  );
</script>

<header
  class="h-12 shrink-0 flex items-center gap-3 px-3 border-b border-[var(--color-border)] bg-[var(--color-panel)]"
>
  <!-- Identity -->
  <div class="flex flex-col min-w-0">
    <span class="text-sm font-semibold truncate leading-tight">{sheet?.meta?.name || 'rpgman'}</span>
    <span class="text-[10px] text-[var(--color-muted)] truncate leading-tight">
      {#if sheet?.species}<span class="capitalize">{sheet.species}</span>{/if}
      {#if sheet?.background}· <span class="capitalize">{sheet.background}</span>{/if}
      {#if classChips.length}· <span class="capitalize">{classChips.join(' / ')}</span>{/if}
    </span>
  </div>

  <div class="flex-1"></div>

  <!-- The four glance numbers -->
  {#if c}
    <div class="flex items-stretch gap-2">
      <div class="flex flex-col items-center justify-center px-2 rounded bg-[var(--color-panel-2)]">
        <span class="text-[9px] uppercase text-[var(--color-muted)] leading-none">Lvl</span>
        <span class="num text-base font-bold leading-tight">{c.level}</span>
      </div>
      <div class="flex flex-col items-center justify-center px-2 rounded bg-[var(--color-panel-2)] min-w-[64px]">
        <span class="text-[9px] uppercase text-[var(--color-muted)] leading-none">HP</span>
        <span class="num text-base font-bold leading-tight">
          {c.current_hp}<span class="text-[var(--color-muted)] text-xs">/{c.max_hp?.total}</span>
          {#if c.temp_hp > 0}<span class="text-[var(--color-good)] text-xs"> +{c.temp_hp}</span>{/if}
        </span>
      </div>
      <div class="flex flex-col items-center justify-center px-2 rounded bg-[var(--color-panel-2)]">
        <span class="text-[9px] uppercase text-[var(--color-muted)] leading-none">AC</span>
        <span class="num text-base font-bold leading-tight">{c.armor_class?.total}</span>
      </div>
      <div class="flex flex-col items-center justify-center px-2 rounded bg-[var(--color-panel-2)]">
        <span class="text-[9px] uppercase text-[var(--color-muted)] leading-none">Init</span>
        <span class="num text-base font-bold leading-tight">{signed(c.initiative?.total ?? 0)}</span>
      </div>
    </div>
  {/if}

  <!-- Status indicator -->
  <div class="ml-2 flex items-center gap-2">
    {#if app.busy}
      <span class="text-[10px] text-[var(--color-muted)]">computing…</span>
    {/if}
    {#if app.error}
      <span class="text-[10px] text-[var(--color-bad)]" title={app.error}>⚠ error</span>
    {/if}
  </div>
</header>
