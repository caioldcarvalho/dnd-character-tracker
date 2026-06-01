<script lang="ts">
  import Card from '../Card.svelte';
  import { app } from '../../state.svelte';

  const c = $derived(app.computed);
  const current = $derived(c?.current_hp ?? 0);
  const max = $derived(c?.max_hp?.total ?? 0);
  const temp = $derived(c?.temp_hp ?? 0);

  const pct = $derived(max > 0 ? Math.max(0, Math.min(100, (current / max) * 100)) : 0);
  const barColor = $derived(
    pct > 50
      ? 'var(--color-good)'
      : pct > 25
        ? 'var(--color-warn)'
        : 'var(--color-bad)'
  );

  let amount = $state<number | null>(null);
  let tempAmount = $state<number | null>(null);

  function damage() {
    const n = Number(amount);
    if (n > 0) app.applyDamage(n);
    amount = null;
  }
  function heal() {
    const n = Number(amount);
    if (n > 0) app.heal(n);
    amount = null;
  }
  function setTemp() {
    const n = Number(tempAmount);
    if (n >= 0) app.setTempHp(n);
    tempAmount = null;
  }
  function quick(n: number) {
    if (n < 0) app.applyDamage(-n);
    else app.heal(n);
  }
</script>

<Card title="Hit Points">
  {#if !c}
    <p class="text-[11px] text-[var(--color-muted)] py-2 text-center">No character.</p>
  {:else}
    <div class="flex flex-col gap-2">
      <!-- Big current / max -->
      <div class="flex items-baseline gap-1.5">
        <span
          class="num font-semibold text-2xl leading-none"
          style="color: {current === 0 ? 'var(--color-bad)' : 'var(--color-ink)'};"
          >{current}</span
        >
        <span class="num text-sm text-[var(--color-muted)] leading-none">/ {max}</span>
        {#if temp > 0}
          <span class="num text-[11px] text-[var(--color-good)] leading-none font-semibold"
            >+{temp}</span
          >
        {/if}
        {#if current === 0}
          <span
            class="ml-auto text-[9px] uppercase tracking-wide px-1.5 py-0.5 rounded bg-[var(--color-bad)]/15 text-[var(--color-bad)]"
            >Unconscious</span
          >
        {/if}
      </div>

      <!-- HP bar -->
      <div class="h-1.5 w-full rounded-full bg-[var(--color-panel-2)] overflow-hidden">
        <div
          class="h-full rounded-full transition-all"
          style="width: {pct}%; background: {barColor};"
        ></div>
      </div>

      <!-- Quick adjust -->
      <div class="flex gap-1">
        <button
          type="button"
          onclick={() => quick(-5)}
          class="flex-1 num text-[11px] rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] py-1 hover:border-[var(--color-accent)]"
          >−5</button
        >
        <button
          type="button"
          onclick={() => quick(-1)}
          class="flex-1 num text-[11px] rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] py-1 hover:border-[var(--color-accent)]"
          >−1</button
        >
        <button
          type="button"
          onclick={() => quick(1)}
          class="flex-1 num text-[11px] rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] py-1 hover:border-[var(--color-accent)]"
          >+1</button
        >
        <button
          type="button"
          onclick={() => quick(5)}
          class="flex-1 num text-[11px] rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] py-1 hover:border-[var(--color-accent)]"
          >+5</button
        >
      </div>

      <!-- Damage / Heal -->
      <div class="flex gap-1">
        <input
          type="number"
          min="0"
          bind:value={amount}
          placeholder="—"
          class="num w-12 text-[11px] text-center rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] py-1 focus:border-[var(--color-accent)] focus:outline-none"
        />
        <button
          type="button"
          onclick={damage}
          class="flex-1 text-[11px] rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] py-1 hover:border-[var(--color-bad)] hover:text-[var(--color-bad)]"
          >Damage</button
        >
        <button
          type="button"
          onclick={heal}
          class="flex-1 text-[11px] rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] py-1 hover:border-[var(--color-good)] hover:text-[var(--color-good)]"
          >Heal</button
        >
      </div>

      <!-- Temp HP -->
      <div class="flex gap-1 items-center">
        <span class="text-[9px] uppercase text-[var(--color-muted)]">Temp</span>
        <input
          type="number"
          min="0"
          bind:value={tempAmount}
          placeholder="—"
          class="num w-12 text-[11px] text-center rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] py-1 focus:border-[var(--color-accent)] focus:outline-none"
        />
        <button
          type="button"
          onclick={setTemp}
          class="flex-1 text-[11px] rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] py-1 hover:border-[var(--color-accent)]"
          >Set</button
        >
      </div>
    </div>
  {/if}
</Card>
