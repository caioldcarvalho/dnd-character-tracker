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
                  <span class="w-2 h-2 rounded-full bg-[var(--color-accent)]"></span>
                {/each}
              </span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </Card>
{/if}
