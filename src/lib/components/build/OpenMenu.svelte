<script lang="ts">
  // Character library hub. Shown when no character is loaded.
  // First-run (empty library): welcome with two CTAs.
  // Returning (saved chars): informative cards with open/dup/rename/delete.
  import { app } from '../../state.svelte';
  import * as ipc from '../../ipc';
  import type { CharacterSummary } from '../../ipc';
  import { SAMPLE_SHEET } from '../../sample';
  import { onMount } from 'svelte';

  let characters = $state<CharacterSummary[]>([]);
  let loading = $state(false);
  let err = $state<string | null>(null);

  // Per-card inline state: keyed by path
  type CardMode = 'idle' | 'confirm-delete' | 'rename';
  let cardModes = $state<Record<string, CardMode>>({});
  let renameValues = $state<Record<string, string>>({});

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
    err = null;
    try {
      const sheet = await ipc.loadCharacter(path);
      await app.setSheet(sheet, path);
      app.section = 'sheet';
    } catch (e) {
      err = String(e);
    }
  }

  async function loadSample() {
    await app.setSheet(structuredClone(SAMPLE_SHEET));
    app.section = 'sheet';
  }

  function startRename(c: CharacterSummary) {
    renameValues[c.path] = c.name;
    cardModes[c.path] = 'rename';
  }

  async function commitRename(c: CharacterSummary) {
    const newName = (renameValues[c.path] ?? c.name).trim();
    if (newName && newName !== c.name) {
      err = null;
      try {
        await app.renameCharacter(c.path, newName);
        await refresh();
      } catch (e) {
        err = String(e);
      }
    }
    cardModes[c.path] = 'idle';
  }

  function cancelRename(path: string) {
    cardModes[path] = 'idle';
  }

  function confirmDelete(path: string) {
    cardModes[path] = 'confirm-delete';
  }

  async function doDelete(path: string) {
    err = null;
    try {
      await app.deleteCharacter(path);
      await refresh();
    } catch (e) {
      err = String(e);
    }
    cardModes[path] = 'idle';
  }

  function cancelDelete(path: string) {
    cardModes[path] = 'idle';
  }

  async function duplicate(c: CharacterSummary) {
    err = null;
    try {
      await app.duplicateCharacter(c.path);
      await refresh();
    } catch (e) {
      err = String(e);
    }
  }

  onMount(refresh);
</script>

<div class="h-full flex items-start justify-center pt-16 p-6">
  <div class="w-full max-w-[520px] flex flex-col gap-4">

    <!-- Header -->
    <div class="flex items-center justify-between">
      <h1 class="text-[13px] font-semibold text-[var(--color-ink)] tracking-wide">
        Characters
      </h1>
      <button
        type="button"
        class="rounded bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
        onclick={() => app.newCharacter()}
      >
        + Create a character
      </button>
    </div>

    {#if err}
      <p class="text-[11px] text-[var(--color-bad)] px-1">{err}</p>
    {/if}

    {#if loading}
      <p class="text-[11px] text-[var(--color-muted)] px-1">Loading…</p>

    {:else if characters.length === 0}
      <!-- First-run welcome -->
      <div
        class="rounded border border-[var(--color-border)] bg-[var(--color-panel)] px-5 py-8 flex flex-col items-center gap-4 text-center"
      >
        <p class="text-[12px] text-[var(--color-ink)] font-medium">Welcome to rpgman</p>
        <p class="text-[11px] text-[var(--color-muted)] max-w-[320px] leading-relaxed">
          Track every stat, skill, and choice for your D&D 2024 characters — with full engine
          calculations.
        </p>
        <div class="flex flex-col gap-2 w-full max-w-[240px]">
          <button
            type="button"
            class="rounded bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
            onclick={() => app.newCharacter()}
          >
            Create a character
          </button>
          <button
            type="button"
            class="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
            onclick={loadSample}
          >
            Load example character (Aldric)
          </button>
        </div>
      </div>

    {:else}
      <!-- Character cards -->
      <ul class="flex flex-col gap-2">
        {#each characters as c (c.path)}
          {@const mode = cardModes[c.path] ?? 'idle'}
          <li
            class="rounded border border-[var(--color-border)] bg-[var(--color-panel)] overflow-hidden"
          >
            {#if mode === 'rename'}
              <!-- Inline rename -->
              <div class="px-3 py-2 flex items-center gap-2">
                <input
                  type="text"
                  class="flex-1 bg-[var(--color-panel-2)] border border-[var(--color-border)] rounded px-2 py-0.5 text-[12px] text-[var(--color-ink)] outline-none focus:border-[var(--color-accent)]"
                  bind:value={renameValues[c.path]}
                  onkeydown={(e) => {
                    if (e.key === 'Enter') commitRename(c);
                    if (e.key === 'Escape') cancelRename(c.path);
                  }}
                />
                <button
                  type="button"
                  class="rounded bg-[var(--color-accent)] px-2.5 py-1 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
                  onclick={() => commitRename(c)}
                >
                  Save
                </button>
                <button
                  type="button"
                  class="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
                  onclick={() => cancelRename(c.path)}
                >
                  Cancel
                </button>
              </div>

            {:else if mode === 'confirm-delete'}
              <!-- Inline delete confirm -->
              <div class="px-3 py-2 flex items-center justify-between gap-2">
                <span class="text-[11px] text-[var(--color-warn)]">
                  Delete <span class="font-semibold">{c.name}</span>? This cannot be undone.
                </span>
                <div class="flex gap-1.5 shrink-0">
                  <button
                    type="button"
                    class="rounded bg-[var(--color-bad)] px-2.5 py-1 text-[11px] font-semibold text-white hover:opacity-90"
                    onclick={() => doDelete(c.path)}
                  >
                    Delete
                  </button>
                  <button
                    type="button"
                    class="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
                    onclick={() => cancelDelete(c.path)}
                  >
                    Cancel
                  </button>
                </div>
              </div>

            {:else}
              <!-- Normal card -->
              <div class="flex items-stretch">
                <!-- Main clickable area: open the character -->
                <button
                  type="button"
                  class="flex-1 text-left px-3 py-2.5 hover:bg-[var(--color-panel-2)] transition-colors"
                  onclick={() => open(c.path)}
                >
                  <div class="text-[12px] font-medium text-[var(--color-ink)]">{c.name}</div>
                  {#if c.classLabel || c.species || c.level != null}
                    <div class="mt-0.5 flex items-center gap-1.5 text-[10px] text-[var(--color-muted)]">
                      {#if c.species}
                        <span>{c.species}</span>
                      {/if}
                      {#if c.species && c.classLabel}
                        <span class="opacity-40">·</span>
                      {/if}
                      {#if c.classLabel}
                        <span>{c.classLabel}</span>
                      {/if}
                      {#if c.level != null}
                        <span class="opacity-40">·</span>
                        <span>Level {c.level}</span>
                      {/if}
                      {#if c.hp}
                        <span class="opacity-40">·</span>
                        <span class="num text-[var(--color-good)]">{c.hp.current} HP</span>
                      {/if}
                    </div>
                  {/if}
                </button>

                <!-- Card action row -->
                <div
                  class="flex items-center gap-0.5 px-2 border-l border-[var(--color-border)] bg-[var(--color-panel-2)]"
                >
                  <button
                    type="button"
                    title="Open"
                    class="rounded px-2 py-1 text-[10px] text-[var(--color-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-panel)]"
                    onclick={() => open(c.path)}
                  >
                    Open
                  </button>
                  <button
                    type="button"
                    title="Duplicate"
                    class="rounded px-2 py-1 text-[10px] text-[var(--color-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-panel)]"
                    onclick={() => duplicate(c)}
                  >
                    Dup
                  </button>
                  <button
                    type="button"
                    title="Rename"
                    class="rounded px-2 py-1 text-[10px] text-[var(--color-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-panel)]"
                    onclick={() => startRename(c)}
                  >
                    Rename
                  </button>
                  <button
                    type="button"
                    title="Delete"
                    class="rounded px-2 py-1 text-[10px] text-[var(--color-bad)] hover:text-white hover:bg-[var(--color-bad)]/20"
                    onclick={() => confirmDelete(c.path)}
                  >
                    Del
                  </button>
                </div>
              </div>
            {/if}
          </li>
        {/each}
      </ul>

      <!-- Also offer sample load when library has chars (secondary, at bottom) -->
      <div class="pt-1">
        <button
          type="button"
          class="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
          onclick={loadSample}
        >
          Load example character (Aldric)
        </button>
      </div>
    {/if}
  </div>
</div>
