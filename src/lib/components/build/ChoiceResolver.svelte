<script lang="ts">
  import Card from '../Card.svelte';
  import ChoiceCard from './ChoiceCard.svelte';
  import { app } from '../../state.svelte';

  const choices = $derived(app.pendingChoices ?? []);
  const count = $derived(choices.length);
</script>

{#if app.computed}
  <Card title="Choices">
    <div class="flex flex-col gap-2 max-h-full overflow-y-auto">
      <div class="text-[11px] font-semibold {count === 0 ? 'text-[var(--color-good)]' : 'text-[var(--color-muted)]'}">
        {#if count === 0}
          All choices resolved ✓
        {:else}
          <span class="num">{count}</span> {count === 1 ? 'choice' : 'choices'} to resolve
        {/if}
      </div>
      {#each choices as choice (choice.key)}
        <ChoiceCard {choice} />
      {/each}
    </div>
  </Card>
{/if}
