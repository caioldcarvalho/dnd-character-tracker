<script lang="ts">
  // A compact, searchable single-select dropdown for the build flow.
  // Pure leaf: no store import, all behavior driven through props.
  type Option = { id: string; name: string; hint?: string };

  let {
    label,
    value,
    options,
    placeholder = 'Select…',
    allowNone = false,
    onpick
  }: {
    label: string;
    value: string | null;
    options: Option[];
    placeholder?: string;
    allowNone?: boolean;
    onpick: (id: string | null) => void;
  } = $props();

  let open = $state(false);
  let query = $state('');
  let active = $state(0);
  let root = $state<HTMLDivElement | null>(null);
  let input = $state<HTMLInputElement | null>(null);

  const selected = $derived((options ?? []).find((o) => o.id === value) ?? null);

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    const list = options ?? [];
    if (!q) return list;
    return list.filter(
      (o) => o.name.toLowerCase().includes(q) || (o.hint?.toLowerCase().includes(q) ?? false)
    );
  });

  // Rows include the optional "none" entry at index 0.
  const rows = $derived.by(() => {
    const r: Array<Option | null> = [];
    if (allowNone) r.push(null);
    for (const o of filtered) r.push(o);
    return r;
  });

  function openMenu() {
    if (open) return;
    open = true;
    query = '';
    active = Math.max(
      0,
      rows.findIndex((r) => (r ? r.id === value : value === null))
    );
    queueMicrotask(() => input?.focus());
  }

  function close() {
    open = false;
    query = '';
  }

  function choose(opt: Option | null) {
    onpick(opt ? opt.id : null);
    close();
  }

  function onkeydown(e: KeyboardEvent) {
    if (!open) {
      if (e.key === 'Enter' || e.key === ' ' || e.key === 'ArrowDown') {
        e.preventDefault();
        openMenu();
      }
      return;
    }
    if (e.key === 'Escape') {
      e.preventDefault();
      close();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      active = Math.min(rows.length - 1, active + 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      active = Math.max(0, active - 1);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (rows.length) choose(rows[Math.min(active, rows.length - 1)]);
    }
  }

  // Close on outside click.
  $effect(() => {
    if (!open) return;
    function onDoc(e: MouseEvent) {
      if (root && !root.contains(e.target as Node)) close();
    }
    document.addEventListener('mousedown', onDoc);
    return () => document.removeEventListener('mousedown', onDoc);
  });
</script>

<div class="flex flex-col gap-0.5" bind:this={root}>
  <span class="text-[9px] uppercase tracking-wide text-[var(--color-muted)]">{label}</span>

  <div class="relative">
    <button
      type="button"
      onclick={() => (open ? close() : openMenu())}
      {onkeydown}
      class="w-full flex items-center gap-2 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] px-2 py-1 text-left text-[11px]
        hover:border-[var(--color-accent)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]
        {open ? 'ring-1 ring-[var(--color-accent)] border-[var(--color-accent)]' : ''}"
      aria-haspopup="listbox"
      aria-expanded={open}
    >
      {#if selected}
        <span class="flex-1 truncate text-[var(--color-ink)]">{selected.name}</span>
        {#if selected.hint}
          <span class="text-[9px] text-[var(--color-muted)] truncate">{selected.hint}</span>
        {/if}
      {:else}
        <span class="flex-1 truncate text-[var(--color-muted)]">{placeholder}</span>
      {/if}
      <span class="text-[9px] text-[var(--color-muted)] shrink-0">{open ? '▴' : '▾'}</span>
    </button>

    {#if open}
      <div
        class="absolute left-0 right-0 top-full z-20 mt-0.5 rounded border border-[var(--color-border)] bg-[var(--color-panel)] shadow-lg"
        role="listbox"
      >
        <div class="p-1 border-b border-[var(--color-border)]">
          <input
            bind:this={input}
            bind:value={query}
            {onkeydown}
            oninput={() => (active = 0)}
            type="text"
            placeholder="Search…"
            class="w-full rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] px-2 py-1 text-[11px] text-[var(--color-ink)]
              placeholder:text-[var(--color-muted)] focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
          />
        </div>

        <ul class="max-h-56 overflow-y-auto py-0.5">
          {#if rows.length === 0}
            <li class="px-2 py-1.5 text-[10px] text-[var(--color-muted)]">No matches</li>
          {/if}
          {#each rows as opt, i}
            {@const isSel = opt ? opt.id === value : value === null}
            <li>
              <button
                type="button"
                onclick={() => choose(opt)}
                onmousemove={() => (active = i)}
                class="w-full flex items-center gap-2 px-2 py-1 text-left text-[11px]
                  {i === active ? 'bg-[var(--color-panel-2)]' : ''}
                  {isSel ? 'ring-1 ring-inset ring-[var(--color-accent)]' : ''}"
                role="option"
                aria-selected={isSel}
              >
                <span
                  class="w-1.5 h-1.5 rounded-full shrink-0 {isSel
                    ? 'bg-[var(--color-accent)]'
                    : 'bg-[var(--color-border)]'}"
                ></span>
                {#if opt}
                  <span class="flex-1 truncate text-[var(--color-ink)]">{opt.name}</span>
                  {#if opt.hint}
                    <span class="text-[9px] text-[var(--color-muted)] truncate">{opt.hint}</span>
                  {/if}
                {:else}
                  <span class="flex-1 truncate text-[var(--color-muted)] italic">— none —</span>
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>
</div>
