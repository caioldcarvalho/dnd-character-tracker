<script lang="ts">
  import Card from '../Card.svelte';
  import { signed, ABILITIES, ABILITY_LABEL, type AbilityKey } from '../../format';
  import { app } from '../../state.svelte';

  // Engine-computed final scores (base + species/background bonuses applied).
  const computed = $derived(app.computed?.abilities ?? []);

  // Standard array, descending — assigned to abilities in canonical order.
  const STANDARD_ARRAY = [15, 14, 13, 12, 10, 8];

  function base(ability: AbilityKey): number {
    return app.sheet?.abilities?.[ability] ?? 10;
  }

  function finalFor(ability: AbilityKey): any {
    return computed.find((a: any) => a.ability === ability);
  }

  function clamp(n: number): number {
    return Math.max(1, Math.min(20, n));
  }

  function step(ability: AbilityKey, delta: number) {
    app.setAbility(ability, clamp(base(ability) + delta));
  }

  function standardArray() {
    const scores: Record<string, number> = {};
    ABILITIES.forEach((ab, i) => {
      scores[ab] = STANDARD_ARRAY[i] ?? 10;
    });
    app.setAbilities(scores);
  }
</script>

<Card title="Ability Scores">
  {#if app.sheet}
    <div class="flex flex-col gap-1.5">
      <div class="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
        {#each ABILITIES as ability}
          {@const final = finalFor(ability)}
          {@const score = base(ability)}
          <div class="flex flex-col gap-1 rounded bg-[var(--color-panel-2)] p-1.5">
            <div class="flex items-center justify-between">
              <span class="text-[9px] font-semibold uppercase tracking-wide text-[var(--color-muted)]"
                >{ABILITY_LABEL[ability]}</span
              >
              {#if final}
                <span class="text-[9px] text-[var(--color-muted)]" title="Final score / modifier">
                  <span class="num">{final.score}</span>
                  <span class="num ml-0.5 text-[var(--color-accent)]">{signed(final.modifier)}</span>
                </span>
              {/if}
            </div>
            <div class="flex items-center gap-1">
              <button
                type="button"
                onclick={() => step(ability, -1)}
                disabled={score <= 1}
                aria-label={`Decrease ${ABILITY_LABEL[ability]}`}
                class="flex h-6 w-6 shrink-0 items-center justify-center rounded border border-[var(--color-border)] bg-[var(--color-panel)] text-[12px] leading-none text-[var(--color-ink)] hover:border-[var(--color-accent)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
                >−</button
              >
              <span class="num flex-1 text-center text-[15px] font-semibold tabular-nums">{score}</span>
              <button
                type="button"
                onclick={() => step(ability, 1)}
                disabled={score >= 20}
                aria-label={`Increase ${ABILITY_LABEL[ability]}`}
                class="flex h-6 w-6 shrink-0 items-center justify-center rounded border border-[var(--color-border)] bg-[var(--color-panel)] text-[12px] leading-none text-[var(--color-ink)] hover:border-[var(--color-accent)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
                >+</button
              >
            </div>
          </div>
        {/each}
      </div>

      <div class="flex items-center justify-between border-t border-[var(--color-border)] pt-1.5">
        <span class="text-[9px] uppercase tracking-wide text-[var(--color-muted)]">Quick fill</span>
        <button
          type="button"
          onclick={standardArray}
          class="rounded border border-[var(--color-border)] bg-[var(--color-panel-2)] px-2 py-1 text-[10px] text-[var(--color-ink)] hover:border-[var(--color-accent)]"
          title="Assign standard array (15 14 13 12 10 8) in STR→CHA order"
        >
          Standard array
          <span class="num ml-1 text-[var(--color-muted)]">15 14 13 12 10 8</span>
        </button>
      </div>
    </div>
  {:else}
    <p class="text-[11px] text-[var(--color-muted)]">No character loaded.</p>
  {/if}
</Card>
