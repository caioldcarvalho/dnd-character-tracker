<script lang="ts">
  import Card from '../Card.svelte';
  import Stat from '../Stat.svelte';
  import { signed, ABILITY_LABEL, type AbilityKey } from '../../format';
  import { app } from '../../state.svelte';

  const abilities = $derived(app.computed?.abilities ?? []);
  const saves = $derived(app.computed?.saves ?? []);

  function saveFor(ab: string) {
    return saves.find((s: any) => s.ability === ab);
  }
</script>

<Card title="Abilities">
  <div class="grid grid-cols-6 gap-1">
    {#each abilities as a}
      {@const save = saveFor(a.ability)}
      <div class="flex flex-col items-center gap-1 rounded bg-[var(--color-panel-2)] py-1.5">
        <span class="text-[9px] uppercase tracking-wide text-[var(--color-muted)]"
          >{ABILITY_LABEL[a.ability as AbilityKey]}</span
        >
        <Stat value={signed(a.modifier)} stat={{ kind: 'ability-modifier', of: a.ability }} big />
        <Stat label="score" value={a.score} stat={{ kind: 'ability-score', of: a.ability }} />
        {#if save}
          <div class="flex items-center gap-0.5 mt-0.5">
            <span
              class="w-1.5 h-1.5 rounded-full {save.proficient
                ? 'bg-[var(--color-accent)]'
                : 'bg-[var(--color-border)]'}"
              title={save.proficient ? 'Proficient save' : 'Not proficient'}
            ></span>
            <Stat
              label="save"
              value={signed(save.test.total)}
              stat={{ kind: 'saving-throw', of: a.ability }}
            />
          </div>
        {/if}
      </div>
    {/each}
  </div>
</Card>
