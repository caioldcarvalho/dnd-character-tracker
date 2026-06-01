<script lang="ts">
  // Build-mode main view. Composes the identity, ability, class and choice
  // editors into the cockpit's 2-column grid. Ensures the content catalog is
  // loaded so the pickers below have their options.
  import { onMount } from 'svelte';
  import { app } from '../../state.svelte';
  import IdentityPanel from './IdentityPanel.svelte';
  import AbilityScores from './AbilityScores.svelte';
  import ClassList from './ClassList.svelte';
  import ChoiceResolver from './ChoiceResolver.svelte';

  onMount(() => {
    app.ensureCatalog();
  });
</script>

{#if app.sheet}
  <div class="grid grid-cols-2 gap-2">
    <IdentityPanel />
    <AbilityScores />
    <div class="col-span-2">
      <ClassList />
    </div>
    <div class="col-span-2">
      <ChoiceResolver />
    </div>
  </div>
{:else}
  <div class="flex flex-col items-center justify-center gap-3 py-16 text-center">
    <p class="text-[11px] text-[var(--color-muted)]">No character — click New</p>
    <button
      type="button"
      onclick={() => app.newCharacter()}
      class="rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink)] hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
    >
      New
    </button>
  </div>
{/if}
