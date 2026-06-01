<script lang="ts">
  import Card from '../Card.svelte';
  import { rechargeColor } from '../../format';
  import { app } from '../../state.svelte';

  const resources = $derived(app.computed?.resources ?? []);
  const hitDice = $derived(app.computed?.hit_dice ?? []);

  function rechargeLabel(r: string): string {
    return { 'short-rest': 'S', 'long-rest': 'L', dawn: '☀', special: '∗' }[r] ?? '?';
  }
</script>

<Card title="Resources & Dice">
  {#if resources.length === 0 && hitDice.length === 0}
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
            <span class="flex gap-0.5">
              {#each Array(r.max) as _, i}
                <span
                  class="w-2 h-2 rounded-full {i < r.current
                    ? 'bg-[var(--color-accent)]'
                    : 'border border-[var(--color-border)]'}"
                ></span>
              {/each}
            </span>
          {:else}
            <span class="num text-[11px]"
              >{r.current}<span class="text-[var(--color-muted)]">/{r.max}</span></span
            >
          {/if}
        </li>
      {/each}

      {#if hitDice.length}
        <li class="flex items-center gap-2 pt-1 border-t border-[var(--color-border)]/60">
          <span class="text-[9px] w-4 text-center text-[var(--color-muted)]">HD</span>
          <span class="flex-1 text-[11px]">Hit Dice</span>
          <span class="flex gap-1.5 num text-[11px]">
            {#each hitDice as hd}
              <span title="d{hd.sides}"
                >{hd.total - hd.spent}<span class="text-[var(--color-muted)]">/{hd.total}d{hd.sides}</span
                ></span
              >
            {/each}
          </span>
        </li>
      {/if}
    </ul>
  {/if}
</Card>
