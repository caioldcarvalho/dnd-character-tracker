<script lang="ts">
  import Card from '../Card.svelte';
  import Stat from '../Stat.svelte';
  import { signed } from '../../format';
  import { app } from '../../state.svelte';

  const sources = $derived(app.computed?.spellcasting ?? []);
  const slots = $derived(app.computed?.spell_slots ?? []);
  const hasCasting = $derived(sources.length > 0 || slots.length > 0);
</script>

{#if hasCasting}
  <Card title="Spellcasting">
    <div class="flex flex-col gap-2">
      {#each sources as sc}
        <div class="flex items-center gap-2">
          <span class="text-[11px] font-semibold capitalize flex-1">{sc.source}</span>
          <Stat label="DC" value={sc.save_dc} stat={{ kind: 'spell-save-dc', of: sc.source }} />
          <Stat
            label="atk"
            value={signed(sc.attack_bonus)}
            stat={{ kind: 'spell-attack-bonus', of: sc.source }}
          />
          {#if sc.prepared !== null && sc.prepared !== undefined}
            <Stat label="prep" value={sc.prepared} />
          {/if}
        </div>
      {/each}

      {#if slots.length}
        <div class="flex flex-wrap gap-1.5 pt-1 border-t border-[var(--color-border)]/60">
          {#each slots as s}
            <div class="flex flex-col items-center rounded bg-[var(--color-panel-2)] px-1.5 py-1">
              <span class="text-[9px] text-[var(--color-muted)]">L{s.level}</span>
              <span class="flex gap-0.5 mt-0.5">
                {#each Array(s.max) as _, i}
                  <button
                    type="button"
                    onclick={() => (i < s.current ? app.expendSlot(s.level) : app.restoreSlot(s.level))}
                    title={i < s.current ? `Expend L${s.level} slot` : `Restore L${s.level} slot`}
                    class="w-2.5 h-2.5 rounded-full transition-colors hover:ring-1 hover:ring-[var(--color-accent)] {i <
                    s.current
                      ? 'bg-[var(--color-accent)]'
                      : 'border border-[var(--color-border)] bg-[var(--color-panel)]'}"
                    aria-label={i < s.current ? `Expend level ${s.level} slot` : `Restore level ${s.level} slot`}
                  ></button>
                {/each}
              </span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </Card>
{/if}
