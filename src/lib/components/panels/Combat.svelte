<script lang="ts">
  import Card from '../Card.svelte';
  import Stat from '../Stat.svelte';
  import { signed } from '../../format';
  import { app } from '../../state.svelte';

  const c = $derived(app.computed);
  const speeds = $derived(c?.speeds ?? []);
</script>

<Card title="Combat">
  <div class="grid grid-cols-4 gap-1">
    <div class="flex flex-col items-center rounded bg-[var(--color-panel-2)] py-1.5">
      <Stat label="AC" value={c?.armor_class?.total ?? '—'} stat={{ kind: 'armor-class' }} big />
    </div>
    <div class="flex flex-col items-center rounded bg-[var(--color-panel-2)] py-1.5">
      <Stat label="Init" value={signed(c?.initiative?.total ?? 0)} stat={{ kind: 'initiative' }} big />
    </div>
    <div class="flex flex-col items-center rounded bg-[var(--color-panel-2)] py-1.5">
      <Stat label="Prof" value={signed(c?.proficiency_bonus ?? 0)} />
    </div>
    <div class="flex flex-col items-center rounded bg-[var(--color-panel-2)] py-1.5">
      <Stat label="Masteries" value={c?.masteries_known ?? 0} stat={{ kind: 'weapon-masteries-known' }} />
    </div>
  </div>

  <div class="grid grid-cols-4 gap-1 mt-1">
    {#each speeds as sp}
      <div class="flex flex-col items-center rounded bg-[var(--color-panel-2)] py-1.5">
        <Stat label={sp.kind} value={`${sp.value}`} sub="ft" stat={{ kind: 'speed', of: sp.kind }} />
      </div>
    {/each}
    <div class="flex flex-col items-center rounded bg-[var(--color-panel-2)] py-1.5">
      <Stat label="Carry" value={`${c?.carrying_capacity ?? 0}`} sub="lb" stat={{ kind: 'carrying-capacity' }} />
    </div>
  </div>
</Card>
