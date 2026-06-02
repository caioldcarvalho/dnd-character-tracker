<script lang="ts">
  import Card from '../Card.svelte';
  import ChoiceCard from './ChoiceCard.svelte';
  import { app } from '../../state.svelte';

  const prereqs = $derived(app.missingPrereqs);
  const choices = $derived(app.pendingChoices ?? []);
  const count = $derived(choices.length);
  const done = $derived(prereqs.length === 0 && count === 0);
</script>

{#if app.computed}
  <Card title="Choices">
    <div class="flex flex-col gap-2 max-h-full overflow-y-auto">
      {#if done}
        <div class="text-[11px] font-semibold text-[var(--color-good)]">All choices resolved ✓</div>
      {:else}
        <!-- Prerequisites first: a blank character has no engine "choices" yet, so
             this is the guided "start here" path (set class/species/background in
             the panels above). -->
        {#if prereqs.length > 0}
          <div class="text-[11px] font-semibold text-[var(--color-accent)]">Start here</div>
          <ol class="flex flex-col gap-1">
            {#each prereqs as p, i (p.key)}
              <li
                class="flex items-center gap-2 text-[11px] text-[var(--color-ink)] rounded border border-dashed border-[var(--color-accent)]/50 bg-[var(--color-accent)]/5 px-2 py-1"
              >
                <span
                  class="num shrink-0 w-4 h-4 rounded-full bg-[var(--color-accent)] text-[var(--color-bg)] text-[9px] font-bold flex items-center justify-center"
                  >{i + 1}</span
                >
                {p.label}
              </li>
            {/each}
          </ol>
        {/if}

        {#if count > 0}
          <div class="text-[11px] font-semibold text-[var(--color-muted)]">
            <span class="num">{count}</span>
            {count === 1 ? 'choice' : 'choices'} to resolve
          </div>
          {#each choices as choice (choice.key)}
            <ChoiceCard {choice} />
          {/each}
        {/if}
      {/if}
    </div>
  </Card>
{/if}
