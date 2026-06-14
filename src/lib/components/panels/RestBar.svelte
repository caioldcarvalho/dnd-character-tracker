<script lang="ts">
  import { app } from '../../state.svelte';

  const ready = $derived(!!app.computed && !!app.sheet);
  const busy = $derived(!!app.busy);

  /** Hit dice pools from the computed character. */
  const hitDicePools = $derived(
    (app.computed?.hit_dice ?? []) as Array<{ sides: number; total: number; spent: number }>
  );
  const hasAvailableDice = $derived(hitDicePools.some((p) => p.spent < p.total));
  const atFullHp = $derived(
    (app.computed?.current_hp ?? 0) >= (app.computed?.max_hp?.total ?? 0)
  );

  function shortRest() {
    if (busy) return;
    app.rest('short');
  }
  function longRest() {
    if (busy) return;
    app.rest('long');
  }
  function spendDie(sides: number, total: number) {
    if (busy || atFullHp) return;
    app.spendHitDie(sides, total);
  }
</script>

{#if ready}
  <div class="flex flex-col gap-1.5">
    <!-- Hit Dice spending row -->
    {#if hitDicePools.length > 0}
      <div
        class="flex flex-wrap items-center gap-1 rounded border border-[var(--color-border)] bg-[var(--color-panel)] px-2 py-1.5"
      >
        <span class="text-[9px] uppercase text-[var(--color-muted)] shrink-0 mr-0.5">Hit Dice</span>
        {#each hitDicePools as pool (pool.sides)}
          {@const available = pool.total - pool.spent}
          <button
            type="button"
            disabled={busy || available <= 0 || atFullHp}
            onclick={() => spendDie(pool.sides, pool.total)}
            title={available > 0 && !atFullHp
              ? `Spend 1d${pool.sides} to heal (roll + CON mod)`
              : available <= 0
                ? `No d${pool.sides} remaining`
                : 'Already at full HP'}
            class="num rounded border px-1.5 py-0.5 text-[10px] transition-colors
              {available > 0 && !atFullHp
              ? 'border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-ink)] hover:border-[var(--color-good)] hover:text-[var(--color-good)]'
              : 'border-[var(--color-border)] bg-[var(--color-panel-2)] text-[var(--color-muted)] opacity-40 cursor-not-allowed'}"
          >
            d{pool.sides}
            <span class="text-[9px]">{available}/{pool.total}</span>
          </button>
        {/each}
        {#if atFullHp && hasAvailableDice}
          <span class="text-[9px] text-[var(--color-muted)] ml-1">full HP</span>
        {/if}
      </div>
    {/if}

    <!-- Short / Long Rest buttons -->
    <div
      class="flex items-stretch gap-1.5 rounded border border-[var(--color-border)] bg-[var(--color-panel)] p-1.5"
    >
      <button
        type="button"
        disabled={busy}
        onclick={shortRest}
        title="Short Rest — recharge short-rest pools, spend hit dice"
        class="group flex-1 flex flex-col items-start rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] px-2 py-1 text-left transition-colors hover:border-[#4ea1ff] disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:border-[var(--color-border)]"
      >
        <span class="text-[11px] font-semibold text-[#4ea1ff]">Short Rest</span>
        <span class="text-[9px] text-[var(--color-muted)] leading-tight"
          >recharge short-rest pools</span
        >
      </button>

      <button
        type="button"
        disabled={busy}
        onclick={longRest}
        title="Long Rest — full HP, all slots, −1 exhaustion, regain ½ hit dice"
        class="group flex-1 flex flex-col items-start rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] px-2 py-1 text-left transition-colors hover:border-[var(--color-accent)] disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:border-[var(--color-border)]"
      >
        <span class="text-[11px] font-semibold text-[var(--color-accent)]">Long Rest</span>
        <span class="text-[9px] text-[var(--color-muted)] leading-tight"
          >full HP, all slots, −1 exhaustion</span
        >
      </button>
    </div>
  </div>
{/if}
