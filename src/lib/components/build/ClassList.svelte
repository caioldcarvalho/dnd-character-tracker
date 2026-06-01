<script lang="ts">
  // Manage the character's classes (multiclass-capable). Each entry gets a level
  // stepper, an optional subclass picker (once the class's subclass level is
  // reached), and a remove button. Below the list, an "Add class" picker offers
  // the classes not yet taken.
  import Card from '../Card.svelte';
  import Picker from './Picker.svelte';
  import { app } from '../../state.svelte';

  const catalog = $derived(app.catalog);
  const classes = $derived(app.sheet?.classes ?? []);

  /** Catalog ClassSummary for a class id. */
  function classInfo(id: string) {
    return catalog?.classes.find((c: any) => c.id === id) ?? null;
  }

  /** Subclasses belonging to a given class id. */
  function subclassesFor(classId: string) {
    return (catalog?.subclasses ?? []).filter((s: any) => s.class === classId);
  }

  /** Classes not already on the sheet, shaped for the Add picker. */
  const addable = $derived.by(() => {
    if (!catalog) return [];
    const taken = new Set(classes.map((c: any) => c.class));
    return catalog.classes
      .filter((c: any) => !taken.has(c.id))
      .map((c: any) => ({
        id: c.id,
        name: c.name,
        hint: 'd' + c.hit_die + (c.caster ? ' · ' + c.caster + ' caster' : '')
      }));
  });

  function step(i: number, lvl: number, delta: number) {
    app.setClassLevel(i, Math.max(1, Math.min(20, lvl + delta)));
  }
</script>

<Card title="Classes">
  {#if !catalog}
    <div class="text-[10px] text-[var(--color-muted)] px-1 py-2">Loading catalog…</div>
  {:else}
    <div class="flex flex-col gap-1.5">
      {#each classes as entry, i (i)}
        {@const info = classInfo(entry.class)}
        {@const subs = subclassesFor(entry.class)}
        {@const showSub = !!info && entry.level >= info.subclass_level}
        <div class="rounded bg-[var(--color-panel-2)] border border-[var(--color-border)] p-1.5 flex flex-col gap-1.5">
          <div class="flex items-center gap-1.5">
            <span class="flex-1 text-[11px] font-semibold truncate">{info?.name ?? entry.class}</span>
            {#if info}
              <span class="text-[9px] text-[var(--color-muted)]">d{info.hit_die}</span>
            {/if}

            <!-- level stepper -->
            <div class="flex items-center gap-0.5">
              <button
                type="button"
                onclick={() => step(i, entry.level, -1)}
                disabled={entry.level <= 1}
                class="w-5 h-5 grid place-items-center rounded bg-[var(--color-panel)] border border-[var(--color-border)] text-[11px] leading-none hover:bg-[var(--color-panel-2)] disabled:opacity-40 disabled:cursor-not-allowed"
                aria-label="Decrease level"
              >
                −
              </button>
              <span class="num w-6 text-center text-[11px] font-semibold">{entry.level}</span>
              <button
                type="button"
                onclick={() => step(i, entry.level, 1)}
                disabled={entry.level >= 20}
                class="w-5 h-5 grid place-items-center rounded bg-[var(--color-panel)] border border-[var(--color-border)] text-[11px] leading-none hover:bg-[var(--color-panel-2)] disabled:opacity-40 disabled:cursor-not-allowed"
                aria-label="Increase level"
              >
                +
              </button>
            </div>

            <!-- remove -->
            <button
              type="button"
              onclick={() => app.removeClass(i)}
              class="w-5 h-5 grid place-items-center rounded bg-[var(--color-panel)] border border-[var(--color-border)] text-[11px] leading-none text-[var(--color-muted)] hover:text-[var(--color-bad)] hover:border-[var(--color-bad)]"
              aria-label="Remove class"
              title="Remove class"
            >
              ×
            </button>
          </div>

          {#if showSub}
            <div class="flex items-center gap-1.5">
              <span class="text-[9px] uppercase tracking-wide text-[var(--color-muted)] w-12">Subclass</span>
              <div class="flex-1">
                <Picker
                  label=""
                  options={subs.map((s: any) => ({ id: s.id, name: s.name }))}
                  value={entry.subclass}
                  allowNone={true}
                  placeholder="Choose subclass…"
                  onpick={(id: string | null) => app.setSubclass(i, id)}
                />
              </div>
            </div>
          {/if}
        </div>
      {/each}

      {#if classes.length === 0}
        <div class="text-[10px] text-[var(--color-muted)] px-1 py-1">No classes yet.</div>
      {/if}

      <!-- add class -->
      {#if addable.length > 0}
        <div class="flex items-center gap-1.5 pt-0.5">
          <span class="text-[9px] uppercase tracking-wide text-[var(--color-muted)] w-12">Add</span>
          <div class="flex-1">
            <Picker
              label=""
              options={addable}
              value={null}
              placeholder="Add class…"
              onpick={(id: string | null) => id && app.addClass(id)}
            />
          </div>
        </div>
      {/if}
    </div>
  {/if}
</Card>
