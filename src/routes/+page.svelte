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
  import { app, inTauri } from '$lib/state.svelte';
  import { SAMPLE_SHEET } from '$lib/sample';
  import { onMount } from 'svelte';

  let loadError = $state<string | null>(null);

  onMount(async () => {
    // Preload the content catalog for build pickers.
    app.ensureCatalog();
    // In browser preview (no backend), load the bundled sample so the cockpit
    // has something to show. In the app, the open/new menu drives loading.
    if (!inTauri()) {
      try {
        await app.setSheet(structuredClone(SAMPLE_SHEET));
      } catch (e) {
        loadError = String(e);
      }
    }
  });
</script>

<div class="h-screen flex flex-col overflow-hidden">
  <Topbar />

  <div class="flex-1 flex overflow-hidden">
    <Rail />

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
      {:else if app.computed}
        <!-- Dense cockpit grid (Sheet / play mode) -->
        <div class="grid grid-cols-12 gap-2 auto-rows-min">
          <div class="col-span-12"><Abilities /></div>
          <!-- Play controls row: HP + rests prominent -->
          <div class="col-span-5"><HPPanel /></div>
          <div class="col-span-7"><RestBar /></div>
          <div class="col-span-5"><Combat /></div>
          <div class="col-span-7"><Conditions /></div>
          <div class="col-span-4 row-span-3"><Skills /></div>
          <div class="col-span-8"><Weapons /></div>
          {#if app.computed.current_hp === 0}
            <div class="col-span-8"><DeathSaves /></div>
          {/if}
          <div class="col-span-8"><Resources /></div>
          <div class="col-span-8"><Spellcasting /></div>
          <div class="col-span-8"><Features /></div>
        </div>
      {/if}
    </main>

    <Inspector />
  </div>
</div>
