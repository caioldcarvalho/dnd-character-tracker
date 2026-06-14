<script lang="ts">
  import Topbar from '$lib/components/Topbar.svelte';
  import Rail from '$lib/components/Rail.svelte';
  import Inspector from '$lib/components/Inspector.svelte';
  import Abilities from '$lib/components/panels/Abilities.svelte';
  import Skills from '$lib/components/panels/Skills.svelte';
  import Combat from '$lib/components/panels/Combat.svelte';
  import Weapons from '$lib/components/panels/Weapons.svelte';
  import HPPanel from '$lib/components/panels/HPPanel.svelte';
  import RestBar from '$lib/components/panels/RestBar.svelte';
  import DeathSaves from '$lib/components/panels/DeathSaves.svelte';
  import Resources from '$lib/components/panels/Resources.svelte';
  import Spellcasting from '$lib/components/panels/Spellcasting.svelte';
  import Features from '$lib/components/panels/Features.svelte';
  import Conditions from '$lib/components/panels/Conditions.svelte';
  import BuildPanel from '$lib/components/build/BuildPanel.svelte';
  import OpenMenu from '$lib/components/build/OpenMenu.svelte';
  import GearPanel from '$lib/components/sections/GearPanel.svelte';
  import SpellsView from '$lib/components/sections/SpellsView.svelte';
  import NotesView from '$lib/components/sections/NotesView.svelte';
  import { app, inTauri } from '$lib/state.svelte';
  import * as ipc from '$lib/ipc';
  import { onMount } from 'svelte';

  let loadError = $state<string | null>(null);

  onMount(async () => {
    // Preload the content catalog for build pickers.
    app.ensureCatalog();
    // Do NOT auto-load the sample. First-run users see the welcome screen (OpenMenu)
    // with explicit CTAs so they choose what to do.
    if (!inTauri()) {
      try {
        // Validate localStorage is accessible (catches private-mode Safari etc.).
        await ipc.listCharacters();
      } catch (e) {
        loadError = String(e);
      }
    }
  });
</script>

<!-- One snippet per cockpit panel, so the masonry can lay them out uniformly. -->
{#snippet hpPanel()}<HPPanel />{/snippet}
{#snippet combat()}<Combat />{/snippet}
{#snippet conditions()}<Conditions />{/snippet}
{#snippet skills()}<Skills />{/snippet}
{#snippet weapons()}<Weapons />{/snippet}
{#snippet deathSaves()}<DeathSaves />{/snippet}
{#snippet resources()}<Resources />{/snippet}
{#snippet spellcasting()}<Spellcasting />{/snippet}
{#snippet features()}<Features />{/snippet}

<div class="h-screen flex flex-col overflow-hidden">
  <Topbar />

  <div class="flex-1 flex overflow-hidden">
    {#if app.sheet}
      <Rail />
    {/if}

    <!-- Main grid -->
    <main class="flex-1 overflow-y-auto p-2">
      {#if !app.sheet}
        <!-- No character loaded → open/new menu -->
        <OpenMenu />
      {:else if app.computed?.errors?.length}
        <div
          class="mb-2 px-3 py-2 rounded border border-[var(--color-bad)] bg-[var(--color-bad)]/10 text-[var(--color-bad)] text-[11px]"
        >
          ⚠ Engine reported {app.computed.errors.length} error(s) — likely a content cycle.
        </div>
      {/if}

      {#if app.sheet && app.section === 'build'}
        <!-- Build mode: identity, abilities, classes, choice resolver -->
        <BuildPanel />
      {:else if app.sheet && app.section === 'gear'}
        <GearPanel />
      {:else if app.sheet && app.section === 'spells'}
        <SpellsView />
      {:else if app.sheet && app.section === 'notes'}
        <NotesView />
      {:else if app.computed}
        <!-- Sheet mode: dense cockpit. Full-width header strips, then a multi-column
             masonry so variable-height panels pack with no holes and reflow. -->
        <div class="flex flex-col gap-2">
          <Abilities />
          <RestBar />
        </div>
        <div class="mt-2 columns-1 md:columns-2 xl:columns-3 gap-2 [column-fill:balance]">
          {#snippet item(c: import('svelte').Snippet)}
            <div class="mb-2 break-inside-avoid">{@render c()}</div>
          {/snippet}
          {@render item(hpPanel)}
          {@render item(combat)}
          {@render item(conditions)}
          {@render item(skills)}
          {@render item(weapons)}
          {#if app.computed.current_hp === 0 && (app.computed.max_hp?.total ?? 0) > 0}
            {@render item(deathSaves)}
          {/if}
          {@render item(resources)}
          {@render item(spellcasting)}
          {@render item(features)}
        </div>
      {/if}
    </main>

    <!-- Inspector only where stat breakdowns are relevant; on Notes/Spells it
         would just show a stale, unrelated breakdown and waste the width. -->
    {#if app.sheet && (app.section === 'sheet' || app.section === 'build' || app.section === 'gear')}
      <Inspector />
    {/if}
  </div>
</div>
