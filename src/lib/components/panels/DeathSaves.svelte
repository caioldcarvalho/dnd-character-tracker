<script lang="ts">
  import Card from '../Card.svelte';
  import { app } from '../../state.svelte';

  const dying = $derived(app.computed?.current_hp === 0);
  const ds = $derived(app.sheet?.death_saves ?? { successes: 0, failures: 0 });
  const successes = $derived(ds.successes ?? 0);
  const failures = $derived(ds.failures ?? 0);
  const stabilized = $derived(successes >= 3);
  const dead = $derived(failures >= 3);

  function mark(kind: 'success' | 'failure' | 'reset') {
    app.deathSave(kind);
  }
</script>

<Card title="Death Saves">
  {#if !app.computed || !app.sheet || !dying}
    <p class="text-[11px] text-[var(--color-muted)] py-2 text-center">—</p>
  {:else}
    <div class="flex flex-col gap-2">
      <!-- Pips -->
      <div class="flex flex-col gap-1.5">
        <div class="flex items-center gap-2">
          <span class="text-[9px] uppercase tracking-wide text-[var(--color-good)] w-12">Success</span>
          <span class="flex gap-1">
            {#each Array(3) as _, i}
              <span
                class="w-3 h-3 rounded-full {i < successes
                  ? 'bg-[var(--color-good)]'
                  : 'border border-[var(--color-border)]'}"
              ></span>
            {/each}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-[9px] uppercase tracking-wide text-[var(--color-bad)] w-12">Failure</span>
          <span class="flex gap-1">
            {#each Array(3) as _, i}
              <span
                class="w-3 h-3 rounded-full {i < failures
                  ? 'bg-[var(--color-bad)]'
                  : 'border border-[var(--color-border)]'}"
              ></span>
            {/each}
          </span>
        </div>
      </div>

      <!-- Outcome -->
      {#if stabilized}
        <p class="text-[11px] font-semibold text-[var(--color-good)] text-center">Stabilized</p>
      {:else if dead}
        <p class="text-[11px] font-semibold text-[var(--color-bad)] text-center">Dead</p>
      {/if}

      <!-- Controls -->
      <div class="flex gap-1.5">
        <button
          type="button"
          onclick={() => mark('success')}
          disabled={stabilized || dead}
          class="flex-1 text-[11px] py-1 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-good)] hover:border-[var(--color-good)] disabled:opacity-40 disabled:hover:border-[var(--color-border)] transition-colors"
        >
          Success
        </button>
        <button
          type="button"
          onclick={() => mark('failure')}
          disabled={stabilized || dead}
          class="flex-1 text-[11px] py-1 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-bad)] hover:border-[var(--color-bad)] disabled:opacity-40 disabled:hover:border-[var(--color-border)] transition-colors"
        >
          Failure
        </button>
        <button
          type="button"
          onclick={() => mark('reset')}
          class="text-[11px] px-2 py-1 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-muted)] hover:border-[var(--color-accent)] transition-colors"
        >
          Reset
        </button>
      </div>
    </div>
  {/if}
</Card>
