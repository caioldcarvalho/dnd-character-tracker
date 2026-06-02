<script lang="ts">
  // Character open/new menu. Lists saved characters (Tauri only) and offers a
  // "new character" action. Shown when nothing is loaded, and reachable from the
  // rail. Pure store + ipc; no overlap with the build editors.
  import { app } from '../../state.svelte';
  import * as ipc from '../../ipc';
  import { onMount } from 'svelte';

  let characters = $state<{ name: string; path: string }[]>([]);
  let loading = $state(false);
  let err = $state<string | null>(null);

  async function refresh() {
    loading = true;
    err = null;
    try {
      characters = await ipc.listCharacters();
    } catch (e) {
      err = String(e);
    } finally {
      loading = false;
    }
  }

  async function open(path: string) {
    try {
      const sheet = await ipc.loadCharacter(path);
      await app.setSheet(sheet, path);
      app.section = 'sheet';
    } catch (e) {
      err = String(e);
    }
  }

  onMount(refresh);
</script>

<div class="h-full flex items-center justify-center p-6">
  <div class="w-[360px] rounded border border-[var(--color-border)] bg-[var(--color-panel)] overflow-hidden">
    <header
      class="px-3 py-2 border-b border-[var(--color-border)] bg-[var(--color-panel-2)] flex items-center justify-between"
    >
      <span class="text-[11px] font-semibold uppercase tracking-wider text-[var(--color-muted)]"
        >Characters</span
      >
      <button
        type="button"
        class="text-[11px] px-2 py-0.5 rounded border border-[var(--color-accent)] text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10"
        onclick={() => app.newCharacter()}>+ New</button
      >
    </header>

    <div class="p-2 max-h-[60vh] overflow-y-auto">
      {#if err}
        <p class="text-[11px] text-[var(--color-bad)] px-1 py-2">{err}</p>
      {/if}
      {#if loading}
        <p class="text-[11px] text-[var(--color-muted)] px-1 py-2">Loading…</p>
      {:else if characters.length === 0}
        <p class="text-[11px] text-[var(--color-muted)] px-1 py-3 text-center">
          No saved characters yet.<br />Click <span class="text-[var(--color-accent)]">+ New</span> to
          begin.
        </p>
      {:else}
        <ul class="flex flex-col gap-0.5">
          {#each characters as c}
            <li>
              <button
                type="button"
                onclick={() => open(c.path)}
                class="w-full text-left px-2 py-1.5 rounded hover:bg-[var(--color-panel-2)] flex items-center justify-between group"
              >
                <span class="text-[12px]">{c.name}</span>
                <span
                  class="text-[9px] text-[var(--color-muted)] opacity-0 group-hover:opacity-100"
                  >open →</span
                >
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
</div>
