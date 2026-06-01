<script lang="ts">
  // A clickable stat value. Clicking opens its contribution breakdown in the
  // inspector — the signature interaction. Pass the StatId the engine uses.
  import { app } from '../state.svelte';

  let {
    label,
    value,
    stat = null,
    sub = '',
    glyph = '',
    glyphClass = '',
    active = false,
    big = false
  } = $props<{
    label?: string;
    value: string | number;
    stat?: any;
    sub?: string;
    glyph?: string;
    glyphClass?: string;
    active?: boolean;
    big?: boolean;
  }>();

  const clickable = $derived(stat !== null);
  const pinned = $derived(
    clickable &&
      app.inspecting &&
      JSON.stringify(app.inspecting.stat) === JSON.stringify(stat)
  );

  function open() {
    if (clickable) app.inspect(stat);
  }
</script>

<button
  type="button"
  onclick={open}
  disabled={!clickable}
  class="group flex flex-col items-center justify-center rounded px-1.5 py-1 text-center transition-colors
    {clickable ? 'cursor-pointer hover:bg-[var(--color-panel-2)]' : 'cursor-default'}
    {pinned ? 'bg-[var(--color-panel-2)] ring-1 ring-[var(--color-accent)]' : ''}
    {active ? 'opacity-100' : ''}"
  title={clickable ? 'Click for breakdown' : ''}
>
  {#if label}
    <span class="text-[9px] uppercase tracking-wide text-[var(--color-muted)] leading-none mb-0.5"
      >{label}</span
    >
  {/if}
  <span class="num font-semibold leading-none {big ? 'text-xl' : 'text-base'} flex items-center gap-0.5">
    {value}{#if glyph}<span class="text-[10px] {glyphClass}">{glyph}</span>{/if}
  </span>
  {#if sub}
    <span class="text-[9px] text-[var(--color-muted)] leading-none mt-0.5">{sub}</span>
  {/if}
  {#if clickable}
    <span
      class="text-[8px] text-[var(--color-muted)] opacity-0 group-hover:opacity-60 leading-none mt-0.5"
      >ⓘ</span
    >
  {/if}
</button>
