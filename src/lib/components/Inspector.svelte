<script lang="ts">
  // The breakdown side panel — the product's thesis made visible. Shows the full
  // contribution tree for the clicked stat: each line's resolved value, source,
  // note, and band. Non-applied lines (lost a Base max, non-stacking dupe) are
  // dimmed so you see what was considered AND rejected.
  import { app } from '../state.svelte';
  import { signed } from '../format';

  const data = $derived(app.inspecting);
  const breakdown = $derived(data?.breakdown);

  // Band → readable label + accent.
  const BAND: Record<string, { label: string; color: string }> = {
    base: { label: 'base', color: 'var(--color-muted)' },
    add: { label: 'add', color: 'var(--color-accent)' },
    multiply: { label: '×', color: 'var(--color-warn)' },
    floor: { label: 'floor', color: 'var(--color-muted)' },
    cap: { label: 'cap', color: 'var(--color-muted)' },
    override: { label: 'set', color: 'var(--color-bad)' },
    derived: { label: 'derived', color: 'var(--color-muted)' }
  };

  function bandOf(line: any) {
    return BAND[line.band] ?? { label: line.band, color: 'var(--color-muted)' };
  }
</script>

<aside
  class="h-full w-[320px] shrink-0 border-l border-[var(--color-border)] bg-[var(--color-panel)] flex flex-col"
>
  <header
    class="px-3 py-2 border-b border-[var(--color-border)] flex items-center justify-between bg-[var(--color-panel-2)]"
  >
    <span class="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-muted)]"
      >Inspector</span
    >
    {#if data}
      <button
        class="text-[var(--color-muted)] hover:text-[var(--color-ink)] text-xs"
        onclick={() => app.closeInspector()}
        title="Close">✕</button
      >
    {/if}
  </header>

  {#if !data || !breakdown}
    <div class="flex-1 flex items-center justify-center p-6 text-center">
      <p class="text-[11px] text-[var(--color-muted)] leading-relaxed">
        Click any stat to see <br />exactly what contributes to it.
      </p>
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto">
      <!-- Stat header + total -->
      <div class="px-3 py-3 border-b border-[var(--color-border)]">
        <div class="text-[10px] uppercase tracking-wide text-[var(--color-muted)]">
          {breakdown.label}
        </div>
        <div class="num text-3xl font-bold leading-tight mt-0.5">{breakdown.total}</div>
      </div>

      <!-- Contribution lines -->
      <ul class="divide-y divide-[var(--color-border)]/60">
        {#each breakdown.lines as line}
          {@const band = bandOf(line)}
          <li
            class="px-3 py-1.5 flex items-baseline gap-2 {line.applied
              ? ''
              : 'opacity-35 line-through decoration-[var(--color-muted)]/40'}"
          >
            <span
              class="num w-9 text-right text-sm font-semibold shrink-0"
              style="color: {line.applied ? 'var(--color-ink)' : 'var(--color-muted)'}"
            >
              {line.band === 'base' || line.band === 'override' ? line.value : signed(line.value)}
            </span>
            <span
              class="text-[8px] uppercase tracking-wide px-1 py-px rounded shrink-0 self-center"
              style="color: {band.color}; border: 1px solid {band.color}40;"
            >
              {band.label}
            </span>
            <span class="flex-1 min-w-0">
              <span class="block text-[11px] text-[var(--color-ink)] truncate">{line.source}</span>
              {#if line.note}
                <span class="block text-[10px] text-[var(--color-muted)] truncate">{line.note}</span
                >
              {/if}
            </span>
          </li>
        {/each}
      </ul>

      <!-- Resolved total footer -->
      <div
        class="px-3 py-2 border-t border-[var(--color-border)] flex items-center justify-between bg-[var(--color-panel-2)]"
      >
        <span class="text-[10px] uppercase tracking-wide text-[var(--color-muted)]">Total</span>
        <span class="num text-lg font-bold">{breakdown.total}</span>
      </div>
    </div>
  {/if}
</aside>
