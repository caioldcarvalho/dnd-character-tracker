<script lang="ts">
  import Card from '../Card.svelte';
  import Picker from './Picker.svelte';
  import { app } from '../../state.svelte';

  // Three-letter uppercase ability abbreviation.
  function abbr(a: string): string {
    return (a ?? '').slice(0, 3).toUpperCase();
  }

  // Current sheet values to bind the controls to.
  const name = $derived(app.sheet?.meta?.name ?? '');
  const species = $derived(app.sheet?.species ?? null);
  const background = $derived(app.sheet?.background ?? null);

  // Picker options derived from the loaded catalog.
  const speciesOptions = $derived(
    (app.catalog?.species ?? []).map((s: any) => ({
      id: s.id,
      name: s.name,
      hint: `${s.size} · ${s.speed}ft`
    }))
  );

  const backgroundOptions = $derived(
    (app.catalog?.backgrounds ?? []).map((b: any) => ({
      id: b.id,
      name: b.name,
      hint: (b.abilities ?? []).map(abbr).join('/')
    }))
  );
</script>

<Card title="Identity">
  <div class="flex flex-col gap-2">
    <!-- Character name -->
    <label class="flex flex-col gap-1">
      <span class="text-[9px] uppercase tracking-wide text-[var(--color-muted)]">Name</span>
      <input
        type="text"
        value={name}
        oninput={(e) => app.setName((e.currentTarget as HTMLInputElement).value)}
        placeholder="Unnamed"
        class="w-full rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] px-2 py-1 text-[11px] text-[var(--color-ink)] placeholder:text-[var(--color-muted)] focus:outline-none focus:border-[var(--color-accent)]"
      />
    </label>

    {#if app.catalog}
      <!-- Species -->
      <Picker
        label="Species"
        options={speciesOptions}
        value={species}
        allowNone={true}
        placeholder="Choose species…"
        onpick={(id: string | null) => app.setSpecies(id)}
      />

      <!-- Background -->
      <Picker
        label="Background"
        options={backgroundOptions}
        value={background}
        allowNone={true}
        placeholder="Choose background…"
        onpick={(id: string | null) => app.setBackground(id)}
      />
    {:else}
      <div class="text-[11px] text-[var(--color-muted)] py-1">Loading…</div>
    {/if}
  </div>
</Card>
