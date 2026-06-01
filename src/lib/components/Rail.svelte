<script lang="ts">
  import { app } from '../state.svelte';

  const sections = [
    { id: 'sheet', label: 'Sheet', icon: '▦' },
    { id: 'build', label: 'Build', icon: '⚒' },
    { id: 'spells', label: 'Spells', icon: '✦' },
    { id: 'gear', label: 'Gear', icon: '⚔' },
    { id: 'notes', label: 'Notes', icon: '✎' }
  ] as const;

  const pendingCount = $derived(app.computed?.pending_choices?.length ?? 0);

  // Return to the library. If there are unsaved changes, confirm first.
  function toLibrary() {
    if (app.dirty) {
      const ok = confirm('You have unsaved changes. Leave this character?\n(OK saves and leaves; Cancel stays.)');
      if (!ok) return;
      app.closeCharacter({ saveFirst: true });
    } else {
      app.closeCharacter();
    }
  }
</script>

<nav
  class="w-14 shrink-0 flex flex-col items-stretch border-r border-[var(--color-border)] bg-[var(--color-panel)] py-1"
>
  <!-- Library: leave the current character -->
  <button
    type="button"
    onclick={toLibrary}
    class="flex flex-col items-center gap-0.5 py-2 mb-1 border-b border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-accent)] transition-colors"
    title="Back to character library"
  >
    <span class="text-base leading-none">←</span>
    <span class="text-[9px] uppercase tracking-wide">Library</span>
  </button>

  {#each sections as s}
    <button
      type="button"
      onclick={() => (app.section = s.id)}
      class="relative flex flex-col items-center gap-0.5 py-2 transition-colors
        {app.section === s.id
          ? 'text-[var(--color-accent)] bg-[var(--color-panel-2)]'
          : 'text-[var(--color-muted)] hover:text-[var(--color-ink)]'}"
    >
      <span class="text-base leading-none">{s.icon}</span>
      <span class="text-[9px] uppercase tracking-wide">{s.label}</span>
      {#if s.id === 'build' && pendingCount > 0}
        <span
          class="absolute top-1 right-2 min-w-3.5 h-3.5 px-0.5 rounded-full bg-[var(--color-warn)] text-[var(--color-bg)] text-[8px] font-bold flex items-center justify-center num"
          title="{pendingCount} choices pending">{pendingCount}</span
        >
      {/if}
    </button>
  {/each}
</nav>
