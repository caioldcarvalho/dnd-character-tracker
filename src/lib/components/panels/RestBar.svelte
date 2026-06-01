<script lang="ts">
  import { app } from '../../state.svelte';

  const ready = $derived(!!app.computed && !!app.sheet);
  const busy = $derived(!!app.busy);

  function shortRest() {
    if (busy) return;
    app.rest('short');
  }
  function longRest() {
    if (busy) return;
    app.rest('long');
  }
</script>

{#if ready}
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
      title="Long Rest — full HP, all slots, −1 exhaustion"
      class="group flex-1 flex flex-col items-start rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] px-2 py-1 text-left transition-colors hover:border-[var(--color-accent)] disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:border-[var(--color-border)]"
    >
      <span class="text-[11px] font-semibold text-[var(--color-accent)]">Long Rest</span>
      <span class="text-[9px] text-[var(--color-muted)] leading-tight"
        >full HP, all slots, −1 exhaustion</span
      >
    </button>
  </div>
{/if}
