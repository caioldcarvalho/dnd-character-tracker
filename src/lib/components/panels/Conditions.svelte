<script lang="ts">
  import Card from '../Card.svelte';
  import { app } from '../../state.svelte';

  const c = $derived(app.computed);
  const effects = $derived(c?.effects ?? []);
  const conditions = $derived(c?.conditions ?? app.sheet?.conditions ?? []);
  const exhaustion = $derived(app.sheet?.exhaustion ?? 0);
  const concentration = $derived(c?.concentration ?? app.sheet?.concentration ?? null);
</script>

<Card title="Status">
  <div class="flex flex-col gap-2">
    <!-- Active toggle effects (Rage, etc.) -->
    {#if effects.length}
      <div class="flex flex-wrap gap-1">
        {#each effects as e}
          <span
            class="text-[10px] px-1.5 py-0.5 rounded border {e.active
              ? 'border-[var(--color-accent)] text-[var(--color-accent)] bg-[var(--color-accent)]/10'
              : 'border-[var(--color-border)] text-[var(--color-muted)]'}"
          >
            {e.name}{e.active ? ' ●' : ''}
          </span>
        {/each}
      </div>
    {/if}

    <!-- Conditions -->
    <div class="flex flex-wrap gap-1 items-center">
      <span class="text-[9px] uppercase text-[var(--color-muted)] mr-1">Conditions</span>
      {#if conditions.length === 0}
        <span class="text-[10px] text-[var(--color-muted)]">none</span>
      {:else}
        {#each conditions as cond}
          <span
            class="text-[10px] px-1.5 py-0.5 rounded bg-[var(--color-bad)]/15 text-[var(--color-bad)] capitalize"
            >{cond}</span
          >
        {/each}
      {/if}
    </div>

    <!-- Exhaustion + concentration -->
    <div class="flex items-center gap-3 text-[10px]">
      <span class="flex items-center gap-1">
        <span class="uppercase text-[var(--color-muted)]">Exhaustion</span>
        <span class="num font-semibold {exhaustion > 0 ? 'text-[var(--color-warn)]' : ''}"
          >{exhaustion}/6</span
        >
      </span>
      {#if concentration}
        <span class="flex items-center gap-1">
          <span class="uppercase text-[var(--color-muted)]">Conc.</span>
          <span class="text-[var(--color-accent)]">{concentration}</span>
        </span>
      {/if}
    </div>
  </div>
</Card>
