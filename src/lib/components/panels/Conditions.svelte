<script lang="ts">
  import Card from '../Card.svelte';
  import { app } from '../../state.svelte';

  // The 2024 condition list as toggle chips. Exhaustion is intentionally
  // omitted here — it has its own 0–6 stepper below.
  const CONDITIONS: { id: string; name: string }[] = [
    { id: 'blinded', name: 'Blinded' },
    { id: 'charmed', name: 'Charmed' },
    { id: 'deafened', name: 'Deafened' },
    { id: 'frightened', name: 'Frightened' },
    { id: 'grappled', name: 'Grappled' },
    { id: 'incapacitated', name: 'Incapacitated' },
    { id: 'invisible', name: 'Invisible' },
    { id: 'paralyzed', name: 'Paralyzed' },
    { id: 'petrified', name: 'Petrified' },
    { id: 'poisoned', name: 'Poisoned' },
    { id: 'prone', name: 'Prone' },
    { id: 'restrained', name: 'Restrained' },
    { id: 'stunned', name: 'Stunned' },
    { id: 'unconscious', name: 'Unconscious' }
  ];

  const c = $derived(app.computed);
  const effects = $derived(c?.effects ?? []);
  const activeConditions = $derived<string[]>(c?.conditions ?? app.sheet?.conditions ?? []);
  const exhaustion = $derived(app.sheet?.exhaustion ?? 0);
  const concentration = $derived(c?.concentration ?? app.sheet?.concentration ?? null);
  const inspiration = $derived(app.sheet?.inspiration ?? false);

  function hasCondition(id: string): boolean {
    return activeConditions.some((x) => String(x).toLowerCase() === id);
  }
</script>

{#if app.computed && app.sheet}
  <Card title="Status">
    <div class="flex flex-col gap-2">
      <!-- Active toggle effects (Rage, etc.) — click to toggle -->
      {#if effects.length}
        <div class="flex flex-wrap gap-1">
          {#each effects as e}
            <button
              type="button"
              onclick={() => app.toggleEffect(e.id)}
              class="text-[10px] px-1.5 py-0.5 rounded border transition-colors {e.active
                ? 'border-[var(--color-accent)] text-[var(--color-accent)] bg-[var(--color-accent)]/15'
                : 'border-[var(--color-border)] text-[var(--color-muted)] bg-[var(--color-panel-2)] hover:border-[var(--color-accent)]'}"
            >
              {e.name}{e.active ? ' ●' : ''}
            </button>
          {/each}
        </div>
      {/if}

      <!-- Conditions — toggle chips for the 2024 list -->
      <div class="flex flex-wrap gap-1 items-center">
        <span class="text-[9px] uppercase text-[var(--color-muted)] mr-0.5 w-full sm:w-auto"
          >Conditions</span
        >
        {#each CONDITIONS as cond}
          {@const on = hasCondition(cond.id)}
          <button
            type="button"
            onclick={() => app.toggleCondition(cond.id)}
            class="text-[10px] px-1.5 py-0.5 rounded border transition-colors {on
              ? 'border-[var(--color-bad)] text-[var(--color-bad)] bg-[var(--color-bad)]/15'
              : 'border-[var(--color-border)] text-[var(--color-muted)] bg-[var(--color-panel-2)] hover:border-[var(--color-bad)]'}"
          >
            {cond.name}
          </button>
        {/each}
      </div>

      <!-- Exhaustion stepper + Inspiration toggle -->
      <div class="flex items-center gap-3 text-[10px] flex-wrap">
        <span class="flex items-center gap-1">
          <span class="uppercase text-[var(--color-muted)]">Exhaustion</span>
          <span class="flex items-center gap-0.5">
            <button
              type="button"
              onclick={() => app.setExhaustion(Math.max(0, exhaustion - 1))}
              disabled={exhaustion <= 0}
              class="w-5 h-5 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-muted)] leading-none hover:border-[var(--color-accent)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
              aria-label="Decrease exhaustion">−</button
            >
            <span
              class="num font-semibold w-8 text-center {exhaustion > 0
                ? 'text-[var(--color-warn)]'
                : ''}">{exhaustion}/6</span
            >
            <button
              type="button"
              onclick={() => app.setExhaustion(Math.min(6, exhaustion + 1))}
              disabled={exhaustion >= 6}
              class="w-5 h-5 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-muted)] leading-none hover:border-[var(--color-accent)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
              aria-label="Increase exhaustion">+</button
            >
          </span>
        </span>

        <button
          type="button"
          onclick={() => app.toggleInspiration()}
          class="flex items-center gap-1 px-1.5 py-0.5 rounded border transition-colors {inspiration
            ? 'border-[var(--color-good)] text-[var(--color-good)] bg-[var(--color-good)]/15'
            : 'border-[var(--color-border)] text-[var(--color-muted)] bg-[var(--color-panel-2)] hover:border-[var(--color-good)]'}"
        >
          <span class="uppercase">Inspiration</span>
          <span>{inspiration ? '●' : '○'}</span>
        </button>
      </div>

      <!-- Concentration -->
      {#if concentration}
        <div class="flex items-center gap-2 text-[10px]">
          <span class="uppercase text-[var(--color-muted)]">Conc.</span>
          <span class="text-[var(--color-accent)] truncate flex-1">{concentration}</span>
          <button
            type="button"
            onclick={() => app.setConcentration(null)}
            class="text-[10px] px-1.5 py-0.5 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-muted)] hover:border-[var(--color-bad)] hover:text-[var(--color-bad)]"
          >
            drop
          </button>
        </div>
      {/if}
    </div>
  </Card>
{/if}
