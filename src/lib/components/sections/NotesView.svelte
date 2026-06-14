<script lang="ts">
  import Card from '../Card.svelte';
  import { app, noteId } from '../../state.svelte';
  import type { Note, NoteCategory } from '$bindings';

  type Filter = NoteCategory | 'all';

  const CATEGORIES: { value: NoteCategory; label: string; color: string }[] = [
    { value: 'general', label: 'General', color: '#8b94a7' },
    { value: 'quest', label: 'Quest', color: '#e0b34a' },
    { value: 'npc', label: 'NPC', color: '#5b8cff' },
    { value: 'location', label: 'Location', color: '#4caf80' },
    { value: 'lore', label: 'Lore', color: '#a87ffb' },
    { value: 'combat', label: 'Combat', color: '#e0574a' }
  ];
  const catMeta = (c: NoteCategory) => CATEGORIES.find((x) => x.value === c) ?? CATEGORIES[0];

  const MAX_LEN = 5000;

  let search = $state('');
  let filter = $state<Filter>('all');

  // Shared editor buffer for both the inline "new note" form and modal edits.
  let form = $state<{ title: string; content: string; category: NoteCategory }>({
    title: '',
    content: '',
    category: 'general'
  });
  let creating = $state(false);

  // Modal (view / edit a single note).
  let viewing = $state<Note | null>(null);
  let editingInModal = $state(false);
  let confirmingDelete = $state(false);

  const notes = $derived<Note[]>(app.sheet?.notes ?? []);

  const filtered = $derived(
    notes
      .filter((n) => {
        const q = search.trim().toLowerCase();
        const matchesSearch =
          !q || n.title.toLowerCase().includes(q) || n.content.toLowerCase().includes(q);
        const matchesCat = filter === 'all' || n.category === filter;
        return matchesSearch && matchesCat;
      })
      .sort((a, b) => {
        if (a.pinned !== b.pinned) return a.pinned ? -1 : 1;
        return (b.updated_at ?? '').localeCompare(a.updated_at ?? '');
      })
  );
  const pinned = $derived(filtered.filter((n) => n.pinned));
  const others = $derived(filtered.filter((n) => !n.pinned));

  function fmtDate(iso: string): string {
    if (!iso) return '—';
    const d = new Date(iso);
    if (isNaN(d.getTime())) return '—';
    return d.toLocaleDateString(undefined, { day: '2-digit', month: 'short', year: 'numeric' });
  }

  function nowIso() {
    return new Date().toISOString();
  }

  // ---- create ----
  function startNew() {
    form = { title: '', content: '', category: 'general' };
    creating = true;
  }
  function cancelNew() {
    creating = false;
  }
  function saveNew() {
    const now = nowIso();
    app.addNote({
      id: noteId(),
      title: form.title.trim() || 'Untitled',
      content: form.content.slice(0, MAX_LEN),
      category: form.category,
      pinned: false,
      created_at: now,
      updated_at: now
    });
    creating = false;
  }

  // ---- view / edit modal ----
  function openNote(n: Note) {
    viewing = n;
    editingInModal = false;
    confirmingDelete = false;
  }
  function closeModal() {
    viewing = null;
    editingInModal = false;
    confirmingDelete = false;
  }
  function startEditInModal() {
    if (!viewing) return;
    form = { title: viewing.title, content: viewing.content, category: viewing.category };
    editingInModal = true;
  }
  function saveModal() {
    if (!viewing) return;
    const updated: Note = {
      ...viewing,
      title: form.title.trim() || 'Untitled',
      content: form.content.slice(0, MAX_LEN),
      category: form.category,
      updated_at: nowIso()
    };
    app.updateNote(viewing.id, updated);
    viewing = updated;
    editingInModal = false;
  }
  function togglePin() {
    if (!viewing) return;
    app.toggleNotePin(viewing.id);
    viewing = { ...viewing, pinned: !viewing.pinned };
  }
  function deleteNote() {
    if (!viewing) return;
    app.deleteNote(viewing.id);
    closeModal();
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && viewing) closeModal();
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#snippet pin(size: number, filled: boolean)}
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill={filled ? 'currentColor' : 'none'}
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path d="M12 17v5M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z" />
  </svg>
{/snippet}

{#snippet catDot(c: NoteCategory)}
  <span class="inline-flex items-center gap-1.5">
    <span class="h-2 w-2 rounded-full" style="background:{catMeta(c).color}"></span>
    <span class="text-[10px] uppercase tracking-wider" style="color:{catMeta(c).color}"
      >{catMeta(c).label}</span
    >
  </span>
{/snippet}

{#snippet editor(onsave: () => void, oncancel: () => void, saveLabel: string)}
  <div class="flex flex-col gap-2">
    <input
      bind:value={form.title}
      placeholder="Title"
      class="w-full rounded bg-[var(--color-panel-2)] border border-[var(--color-border)] px-2 py-1.5 text-[13px] text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
    />
    <div class="flex flex-wrap gap-1">
      {#each CATEGORIES as c (c.value)}
        <button
          type="button"
          onclick={() => (form.category = c.value)}
          class="rounded border px-2 py-0.5 text-[10px] uppercase tracking-wider transition-colors"
          style={form.category === c.value
            ? `border-color:${c.color};color:${c.color};background:${c.color}1a`
            : 'border-color:var(--color-border);color:var(--color-muted)'}
        >
          {c.label}
        </button>
      {/each}
    </div>
    <textarea
      bind:value={form.content}
      placeholder="Write your note…"
      maxlength={MAX_LEN}
      rows="8"
      class="w-full resize-y rounded bg-[var(--color-panel-2)] border border-[var(--color-border)] px-2 py-1.5 text-[12px] leading-relaxed text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
    ></textarea>
    <div class="flex items-center justify-between">
      <span class="text-[10px] text-[var(--color-muted)]"
        ><span class="num">{form.content.length}</span>/{MAX_LEN}</span
      >
      <div class="flex gap-1.5">
        <button
          type="button"
          onclick={oncancel}
          class="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
          >Cancel</button
        >
        <button
          type="button"
          onclick={onsave}
          class="rounded bg-[var(--color-accent)] px-2.5 py-1 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
          >{saveLabel}</button
        >
      </div>
    </div>
  </div>
{/snippet}

{#snippet noteCard(n: Note)}
  <button
    type="button"
    onclick={() => openNote(n)}
    class="flex w-full flex-col gap-1.5 rounded border border-[var(--color-border)] bg-[var(--color-panel)] p-2.5 text-left transition-colors hover:border-[var(--color-accent)]"
    style="border-left:3px solid {catMeta(n.category).color}"
  >
    <div class="flex items-start justify-between gap-2">
      {@render catDot(n.category)}
      {#if n.pinned}
        <span class="text-[var(--color-accent)]">{@render pin(13, true)}</span>
      {/if}
    </div>
    <h4 class="text-[13px] font-semibold text-[var(--color-ink)] truncate">
      {n.title || 'Untitled'}
    </h4>
    <p class="line-clamp-3 text-[11px] leading-relaxed text-[var(--color-muted)]">
      {n.content || 'Empty note'}
    </p>
    <span class="text-[10px] text-[var(--color-muted)]">{fmtDate(n.updated_at)}</span>
  </button>
{/snippet}

{#if app.sheet}
  <div class="flex flex-col gap-2">
    <!-- Toolbar -->
    <Card title="Notes">
      <div class="flex flex-col gap-2">
        <div class="flex flex-wrap items-center gap-2">
          <input
            bind:value={search}
            placeholder="Search notes…"
            class="min-w-40 flex-1 rounded bg-[var(--color-panel-2)] border border-[var(--color-border)] px-2 py-1.5 text-[12px] text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
          />
          <button
            type="button"
            onclick={startNew}
            class="rounded bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
            >+ New note</button
          >
        </div>
        <div class="flex flex-wrap gap-1">
          <button
            type="button"
            onclick={() => (filter = 'all')}
            class="rounded border px-2 py-0.5 text-[10px] uppercase tracking-wider transition-colors"
            style={filter === 'all'
              ? 'border-color:var(--color-accent);color:var(--color-accent);background:#5b8cff1a'
              : 'border-color:var(--color-border);color:var(--color-muted)'}>All</button
          >
          {#each CATEGORIES as c (c.value)}
            <button
              type="button"
              onclick={() => (filter = c.value)}
              class="rounded border px-2 py-0.5 text-[10px] uppercase tracking-wider transition-colors"
              style={filter === c.value
                ? `border-color:${c.color};color:${c.color};background:${c.color}1a`
                : 'border-color:var(--color-border);color:var(--color-muted)'}>{c.label}</button
            >
          {/each}
        </div>
      </div>
    </Card>

    <!-- Inline new-note editor -->
    {#if creating}
      <Card title="New note">
        {@render editor(saveNew, cancelNew, 'Save')}
      </Card>
    {/if}

    <!-- Pinned -->
    {#if pinned.length > 0}
      <div class="flex flex-col gap-1.5">
        <h3
          class="flex items-center gap-1.5 px-0.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-muted)]"
        >
          <span class="text-[var(--color-accent)]">{@render pin(13, true)}</span> Pinned
        </h3>
        <div class="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
          {#each pinned as n (n.id)}
            {@render noteCard(n)}
          {/each}
        </div>
      </div>
    {/if}

    <!-- Others -->
    {#if others.length > 0}
      <div class="flex flex-col gap-1.5">
        {#if pinned.length > 0}
          <h3
            class="px-0.5 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-muted)]"
          >
            All notes
          </h3>
        {/if}
        <div class="grid grid-cols-1 gap-2 md:grid-cols-2 xl:grid-cols-3">
          {#each others as n (n.id)}
            {@render noteCard(n)}
          {/each}
        </div>
      </div>
    {/if}

    <!-- Empty state -->
    {#if filtered.length === 0}
      <Card>
        <div class="flex flex-col items-center gap-2 py-8 text-center">
          <p class="text-[12px] text-[var(--color-muted)]">
            {search || filter !== 'all'
              ? 'No notes match your filters.'
              : 'No notes yet. Capture backstory, quests, NPCs, locations and combat reminders.'}
          </p>
          {#if !search && filter === 'all'}
            <button
              type="button"
              onclick={startNew}
              class="rounded bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
              >+ Create your first note</button
            >
          {/if}
        </div>
      </Card>
    {/if}
  </div>

  <!-- View / edit modal -->
  {#if viewing}
    <div class="fixed inset-0 z-50">
      <button
        type="button"
        aria-label="Close"
        onclick={closeModal}
        class="absolute inset-0 bg-black/60"
      ></button>
      <div class="pointer-events-none absolute inset-0 flex items-start justify-center overflow-y-auto p-4">
        <div
          class="pointer-events-auto mt-[6vh] w-full max-w-lg rounded border border-[var(--color-border)] bg-[var(--color-panel)] shadow-2xl"
          role="dialog"
          aria-modal="true"
          aria-label={viewing.title || 'Note'}
        >
          <header
            class="flex items-center justify-between gap-2 rounded-t border-b border-[var(--color-border)] bg-[var(--color-panel-2)] px-3 py-2"
          >
            <h2 class="truncate text-[13px] font-semibold text-[var(--color-ink)]">
              {editingInModal ? 'Edit note' : viewing.title || 'Untitled'}
            </h2>
            <button
              type="button"
              onclick={closeModal}
              aria-label="Close"
              class="text-[var(--color-muted)] hover:text-[var(--color-ink)]">✕</button
            >
          </header>

          <div class="p-3">
            {#if editingInModal}
              {@render editor(saveModal, () => (editingInModal = false), 'Save')}
            {:else}
              <div class="flex flex-col gap-3">
                <div class="flex items-center gap-2">
                  {@render catDot(viewing.category)}
                  {#if viewing.pinned}
                    <span class="text-[var(--color-accent)]">{@render pin(13, true)}</span>
                  {/if}
                </div>
                <div class="whitespace-pre-wrap text-[12px] leading-relaxed text-[var(--color-ink)]">
                  {viewing.content || 'Empty note'}
                </div>
                <div
                  class="flex flex-col gap-0.5 border-t border-[var(--color-border)] pt-2 text-[10px] text-[var(--color-muted)]"
                >
                  <span>Created {fmtDate(viewing.created_at)}</span>
                  <span>Updated {fmtDate(viewing.updated_at)}</span>
                </div>
                <div class="flex flex-wrap items-center gap-1.5 pt-1">
                  <button
                    type="button"
                    onclick={startEditInModal}
                    class="rounded bg-[var(--color-accent)] px-2.5 py-1 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
                    >Edit</button
                  >
                  <button
                    type="button"
                    onclick={togglePin}
                    class="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
                    >{viewing.pinned ? 'Unpin' : 'Pin'}</button
                  >
                  <div class="ml-auto">
                    {#if confirmingDelete}
                      <span class="inline-flex items-center gap-1.5">
                        <span class="text-[11px] text-[var(--color-bad)]">Delete?</span>
                        <button
                          type="button"
                          onclick={deleteNote}
                          class="rounded bg-[var(--color-bad)] px-2 py-1 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90"
                          >Yes</button
                        >
                        <button
                          type="button"
                          onclick={() => (confirmingDelete = false)}
                          class="rounded border border-[var(--color-border)] px-2 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
                          >No</button
                        >
                      </span>
                    {:else}
                      <button
                        type="button"
                        onclick={() => (confirmingDelete = true)}
                        class="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-bad)] hover:bg-[var(--color-bad)]/10"
                        >Delete</button
                      >
                    {/if}
                  </div>
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}
{:else}
  <Card title="Notes">
    <p class="text-[11px] text-[var(--color-muted)]">No character.</p>
  </Card>
{/if}
