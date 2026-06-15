<script lang="ts">
  import Card from '../Card.svelte';
  import { rechargeColor } from '../../format';
  import { app } from '../../state.svelte';

  const resources = $derived(app.computed?.resources ?? []);

  function rechargeLabel(r: string): string {
    return { 'short-rest': 'S', 'long-rest': 'L', dawn: '☀', special: '∗' }[r] ?? '?';
  }

  function spend(r: any, n = 1) {
    if (r.current <= 0) return;
    app.spendResource(r.id, r.max, n);
  }
  function restore(r: any, n = 1) {
    if (r.current >= r.max) return;
    app.restoreResource(r.id, r.max, n);
  }
</script>

<Card title="Resources & Dice">
  {#if !app.computed || resources.length === 0}
    <p class="text-[11px] text-[var(--color-muted)] py-2 text-center">No resource pools.</p>
  {:else}
    <ul class="flex flex-col gap-1.5">
      {#each resources as r}
        <li class="flex items-center gap-2">
          <span
            class="text-[9px] w-4 h-4 rounded-full flex items-center justify-center shrink-0 font-bold"
            style="background: {rechargeColor(r.recharge)}22; color: {rechargeColor(r.recharge)};"
            title="Recharges on {r.recharge}"
          >
            {rechargeLabel(r.recharge)}
          </span>
          <span class="flex-1 text-[11px] truncate">
            {r.name}{#if r.die}<span class="text-[var(--color-muted)]"> d{r.die}</span>{/if}
          </span>
          <!-- pips for small pools, counter for large -->
          {#if r.max <= 8}
            <span class="flex gap-0.5 items-center">
              {#each Array(r.max) as _, i}
                <button
                  type="button"
                  onclick={() => (i < r.current ? spend(r) : restore(r))}
                  title={i < r.current ? 'Spend one' : 'Restore one'}
                  aria-label={i < r.current ? `Spend ${r.name}` : `Restore ${r.name}`}
                  class="w-2.5 h-2.5 rounded-full transition-colors hover:scale-110 {i < r.current
                    ? 'bg-[var(--color-accent)] hover:bg-[var(--color-bad)]'
                    : 'border border-[var(--color-border)] hover:border-[var(--color-accent)]'}"
                ></button>
              {/each}
            </span>
          {:else}
            <span class="flex items-center gap-1">
              <button
                type="button"
                onclick={() => spend(r)}
                disabled={r.current <= 0}
                title="Spend one"
                aria-label="Spend {r.name}"
                class="w-4 h-4 leading-none rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-muted)] flex items-center justify-center hover:border-[var(--color-accent)] hover:text-[var(--color-ink)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
                >−</button
              >
              <span class="num text-[11px] min-w-[2.2rem] text-center"
                >{r.current}<span class="text-[var(--color-muted)]">/{r.max}</span></span
              >
              <button
                type="button"
                onclick={() => restore(r)}
                disabled={r.current >= r.max}
                title="Restore one"
                aria-label="Restore {r.name}"
                class="w-4 h-4 leading-none rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-muted)] flex items-center justify-center hover:border-[var(--color-accent)] hover:text-[var(--color-ink)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
                >+</button
              >
            </span>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</Card>
