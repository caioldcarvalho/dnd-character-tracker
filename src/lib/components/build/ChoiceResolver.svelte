<script lang="ts">
  import Card from '../Card.svelte';
  import ChoiceCard from './ChoiceCard.svelte';
  import { app } from '../../state.svelte';

  const prereqs = $derived(app.missingPrereqs);
  const choices = $derived(app.pendingChoices ?? []);
  const count = $derived(choices.length);
  const done = $derived(prereqs.length === 0 && count === 0);

  // ---- forward progress tracking ----
  //
  // As choices are resolved, they leave `pending_choices` and their picks land
  // in `sheet.choices`. We track each choice's prompt/source in a session-scoped
  // map so we can still render resolved summaries after the engine drops them
  // from the pending list.
  //
  // Total "steps" = resolved engine choices + pending engine choices
  // (prerequisites are not engine choices so we don't count them here).

  // Map<key, { prompt, source }>
  const seenChoices = new Map<string, { prompt: string; source: string }>();

  // Keep the map up to date whenever pending_choices changes.
  $effect(() => {
    for (const c of choices) {
      if (!seenChoices.has(c.key)) {
        seenChoices.set(c.key, { prompt: c.prompt, source: c.source });
      }
    }
  });

  // Resolved engine choices: entries in sheet.choices that we've seen.
  const resolvedChoices = $derived.by(() => {
    const sheetChoices: Array<{ key: string; picks: string[] }> = app.sheet?.choices ?? [];
    return sheetChoices
      .filter((rc) => rc.picks.length > 0 && seenChoices.has(rc.key))
      .map((rc) => ({
        key: rc.key,
        picks: rc.picks,
        prompt: seenChoices.get(rc.key)?.prompt ?? rc.key,
        source: seenChoices.get(rc.key)?.source ?? ''
      }));
  });

  // Total known choices for the progress bar (resolved + still pending).
  // We don't know what we don't know yet (sub-choices), so this is a
  // soft total that grows as new sub-choices appear.
  const totalKnown = $derived(resolvedChoices.length + count);
  const resolvedCount = $derived(resolvedChoices.length);

  // Progress fraction (0–1). Only show the bar once there's something to track.
  const progress = $derived(totalKnown > 0 ? resolvedCount / totalKnown : 0);

  // kebab/lowercase id → Title Case label (same helper as ChoiceCard).
  function title(id: string): string {
    return id.replace(/[-_]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  // Build a human-readable one-line summary for a resolved choice.
  function resolvedSummary(picks: string[]): string {
    return picks.map(title).join(', ');
  }
</script>

{#if app.computed}
  <Card title="Choices">
    <div class="flex flex-col gap-2 max-h-full overflow-y-auto">
      {#if done && resolvedCount === 0}
        <!-- Fresh character with no choices yet (no class/species/background) -->
        <div class="text-[11px] font-semibold text-[var(--color-good)]">All choices resolved ✓</div>
      {:else}
        <!-- Prerequisites first: guided "start here" path -->
        {#if prereqs.length > 0}
          <div class="text-[11px] font-semibold text-[var(--color-accent)]">Start here</div>
          <ol class="flex flex-col gap-1">
            {#each prereqs as p, i (p.key)}
              <li
                class="flex items-center gap-2 text-[11px] text-[var(--color-ink)] rounded border border-dashed border-[var(--color-accent)]/50 bg-[var(--color-accent)]/5 px-2 py-1"
              >
                <span
                  class="num shrink-0 w-4 h-4 rounded-full bg-[var(--color-accent)] text-[var(--color-bg)] text-[9px] font-bold flex items-center justify-center"
                  >{i + 1}</span
                >
                {p.label}
              </li>
            {/each}
          </ol>
        {/if}

        <!-- Progress bar — only shown once there are engine choices to track -->
        {#if totalKnown > 0}
          <div class="flex flex-col gap-1">
            <div class="flex items-center justify-between gap-2">
              <span class="text-[10px] text-[var(--color-muted)]">
                {#if done}
                  All choices resolved
                {:else}
                  Step <span class="num">{resolvedCount + 1}</span> of <span class="num">{totalKnown}</span>
                {/if}
              </span>
              <span class="num text-[10px] text-[var(--color-muted)]">{resolvedCount}/{totalKnown}</span>
            </div>
            <div class="h-1 rounded-full bg-[var(--color-panel-2)] overflow-hidden">
              <div
                class="h-full rounded-full transition-all duration-300"
                style="width: {Math.round(progress * 100)}%; background: var(--color-{done ? 'good' : 'accent'});"
              ></div>
            </div>
          </div>
        {/if}

        <!-- Resolved choices: compact summary of what was picked -->
        {#if resolvedChoices.length > 0}
          <div class="flex flex-col gap-1">
            {#each resolvedChoices as rc (rc.key)}
              <div
                class="flex items-start justify-between gap-2 rounded border border-[var(--color-border)] bg-[var(--color-panel-2)]/60 px-2 py-1"
              >
                <div class="flex flex-col gap-0.5 min-w-0">
                  <span class="text-[10px] font-semibold text-[var(--color-ink)] leading-tight"
                    >{rc.prompt}</span
                  >
                  <span class="text-[9px] text-[var(--color-good)] truncate"
                    >{resolvedSummary(rc.picks)}</span
                  >
                </div>
                <span class="shrink-0 text-[9px] font-semibold text-[var(--color-good)] mt-0.5">✓</span>
              </div>
            {/each}
          </div>
        {/if}

        <!-- Pending choices -->
        {#if count > 0}
          <div class="flex flex-col gap-2">
            {#each choices as choice (choice.key)}
              <ChoiceCard {choice} />
            {/each}
          </div>
        {/if}

        <!-- All done confirmation (only when there were choices to make) -->
        {#if done && resolvedCount > 0}
          <div class="text-[11px] font-semibold text-[var(--color-good)]">All choices resolved ✓</div>
        {/if}
      {/if}
    </div>
  </Card>
{/if}
