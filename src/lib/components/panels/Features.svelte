<script lang="ts">
  import Card from '../Card.svelte';
  import { app } from '../../state.svelte';

  const features = $derived(app.computed?.features ?? []);

  // Group by the feature 'kind' label for collapsible sections.
  const groups = $derived.by(() => {
    const m = new Map<string, any[]>();
    for (const f of features) {
      const k = f.kind ?? 'Feature';
      if (!m.has(k)) m.set(k, []);
      m.get(k)!.push(f);
    }
    return [...m.entries()];
  });

  let open = $state<Record<string, boolean>>({});
  function toggle(k: string) {
    open[k] = !(open[k] ?? true);
  }
</script>

<Card title="Features ({features.length})">
  {#if features.length === 0}
    <p class="text-[11px] text-[var(--color-muted)] py-2 text-center">No features yet.</p>
  {:else}
    <div class="flex flex-col gap-1">
      {#each groups as [kind, items]}
        {@const isOpen = open[kind] ?? true}
        <div>
          <button
            type="button"
            onclick={() => toggle(kind)}
            class="w-full flex items-center gap-1.5 text-left text-[10px] uppercase tracking-wide text-[var(--color-muted)] py-0.5"
          >
            <span class="text-[8px]">{isOpen ? '▾' : '▸'}</span>
            <span>{kind}</span>
            <span class="text-[var(--color-border)]">({items.length})</span>
          </button>
          {#if isOpen}
            <ul class="flex flex-col gap-0.5 pl-3">
              {#each items as f}
                <li class="text-[11px] leading-tight">
                  <button
                    type="button"
                    onclick={() => app.inspectFeature(f)}
                    class="text-left w-full group hover:text-[var(--color-accent)] cursor-pointer"
                    title={f.description ? 'Click to read what this does' : f.name}
                  >
                    <span class="text-[var(--color-ink)] group-hover:text-[var(--color-accent)]"
                      >{f.name}</span
                    >{#if f.description}<span
                        class="text-[8px] text-[var(--color-muted)] group-hover:text-[var(--color-accent)]"
                      > ⓘ</span
                      >{/if}<span class="text-[9px] text-[var(--color-muted)]"> · {f.source}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</Card>
