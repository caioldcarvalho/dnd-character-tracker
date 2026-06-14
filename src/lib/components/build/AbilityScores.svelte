<script lang="ts">
  import Card from '../Card.svelte';
  import { signed, ABILITIES, ABILITY_LABEL, type AbilityKey } from '../../format';
  import { app } from '../../state.svelte';

  // ----- derived helpers -----

  const computed = $derived(app.computed?.abilities ?? []);
  const sheet = $derived(app.sheet);

  // Standard array values.
  const STANDARD_ARRAY = [15, 14, 13, 12, 10, 8];

  // Point-buy cost table (2024 rules: 8–15 only, costs above 13 increase).
  const PB_COST: Record<number, number> = { 8: 0, 9: 1, 10: 2, 11: 3, 12: 4, 13: 5, 14: 7, 15: 9 };
  const PB_BUDGET = 27;
  const PB_MIN = 8;
  const PB_MAX = 15;

  // Modes: 'manual' | 'standard-array' | 'point-buy'
  let mode = $state<'manual' | 'standard-array' | 'point-buy'>('manual');

  // Standard array assignment: maps each ABILITIES slot to a chosen ability key (or null).
  // Index = position in STANDARD_ARRAY; value = the ability assigned to that slot.
  let saAssignment = $state<(AbilityKey | null)[]>(ABILITIES.map(() => null));

  // Point-buy current scores (stored locally until applied).
  let pbScores = $state<Record<AbilityKey, number>>({
    str: 8, dex: 8, con: 8, int: 8, wis: 8, cha: 8
  });

  // --- class-aware primary ability suggestion ---

  // Static fallback: which ability is most important for each class (for non-casters
  // or before computed.spellcasting is available).
  const CLASS_PRIMARY: Record<string, AbilityKey> = {
    barbarian: 'str',
    fighter: 'str',
    paladin: 'str',
    ranger: 'dex',
    rogue: 'dex',
    monk: 'dex',
    bard: 'cha',
    cleric: 'wis',
    druid: 'wis',
    sorcerer: 'cha',
    warlock: 'cha',
    wizard: 'int'
  };

  /** The recommended primary ability for the current character, or null if unknown. */
  const primaryAbility = $derived.by<AbilityKey | null>(() => {
    // Prefer the engine-computed spellcasting ability (most accurate).
    const castingAbility = app.computed?.spellcasting?.[0]?.ability as AbilityKey | undefined;
    if (castingAbility) return castingAbility;
    // Fall back to static mapping for the first class.
    const firstClass = sheet?.classes?.[0]?.class as string | undefined;
    if (firstClass && firstClass in CLASS_PRIMARY) return CLASS_PRIMARY[firstClass];
    return null;
  });

  // --- standard array helpers ---

  /** Abilities already assigned in the SA mode. */
  const saAssigned = $derived(new Set(saAssignment.filter((a): a is AbilityKey => a !== null)));

  function saAssign(slotIdx: number, ability: AbilityKey | null) {
    // If this ability is already assigned elsewhere, clear that slot first.
    if (ability !== null) {
      saAssignment = saAssignment.map((a, i) => (a === ability && i !== slotIdx ? null : a));
    }
    saAssignment[slotIdx] = ability;
  }

  function saApply() {
    const scores: Record<string, number> = {};
    // Fill already-placed abilities.
    for (let i = 0; i < STANDARD_ARRAY.length; i++) {
      const ab = saAssignment[i];
      if (ab) scores[ab] = STANDARD_ARRAY[i];
    }
    // Unassigned abilities keep the current sheet score (or 10).
    for (const ab of ABILITIES) {
      if (!(ab in scores)) scores[ab] = base(ab);
    }
    app.setAbilities(scores);
  }

  function saReset() {
    saAssignment = ABILITIES.map(() => null);
  }

  const saComplete = $derived(saAssigned.size === 6);

  // --- point-buy helpers ---

  function pbCost(score: number): number {
    return PB_COST[score] ?? 99;
  }

  const pbSpent = $derived.by<number>(() =>
    (Object.values(pbScores) as number[]).reduce((sum, s) => sum + pbCost(s), 0)
  );
  const pbRemaining = $derived(PB_BUDGET - pbSpent);

  function pbStep(ability: AbilityKey, delta: number) {
    const cur = pbScores[ability];
    const next = cur + delta;
    if (next < PB_MIN || next > PB_MAX) return;
    const costDelta = pbCost(next) - pbCost(cur);
    if (delta > 0 && pbRemaining < costDelta) return; // would overspend
    pbScores = { ...pbScores, [ability]: next };
  }

  function pbApply() {
    const scores: Record<string, number> = { ...pbScores };
    app.setAbilities(scores);
  }

  function pbReset() {
    pbScores = { str: 8, dex: 8, con: 8, int: 8, wis: 8, cha: 8 };
  }

  // --- shared score/stepper helpers (manual mode) ---

  function base(ability: AbilityKey): number {
    return sheet?.abilities?.[ability] ?? 10;
  }

  function finalFor(ability: AbilityKey): any {
    return computed.find((a: any) => a.ability === ability);
  }

  function clamp(n: number): number {
    return Math.max(1, Math.min(30, n));
  }

  function step(ability: AbilityKey, delta: number) {
    app.setAbility(ability, clamp(base(ability) + delta));
  }

  // When switching away from point-buy or SA, seed values from the live sheet.
  function switchMode(m: typeof mode) {
    if (m === 'point-buy') {
      for (const ab of ABILITIES) {
        const cur = base(ab);
        pbScores[ab] = Math.max(PB_MIN, Math.min(PB_MAX, cur));
      }
    }
    if (m === 'standard-array') {
      saReset();
    }
    mode = m;
  }
</script>

<Card title="Ability Scores">
  {#if sheet}
    <div class="flex flex-col gap-2">

      <!-- Mode selector -->
      <div class="flex gap-1 rounded bg-[var(--color-panel-2)] p-0.5">
        {#each [['manual','Manual'] as const, ['standard-array','Standard Array'] as const, ['point-buy','Point Buy'] as const] as [m, label]}
          <button
            type="button"
            onclick={() => switchMode(m)}
            class="flex-1 rounded px-2 py-1 text-[10px] font-semibold transition-colors
              {mode === m
                ? 'bg-[var(--color-accent)] text-[var(--color-bg)]'
                : 'text-[var(--color-muted)] hover:text-[var(--color-ink)]'}"
          >{label}</button>
        {/each}
      </div>

      <!-- ===================== MANUAL MODE ===================== -->
      {#if mode === 'manual'}
        <div class="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
          {#each ABILITIES as ability}
            {@const final = finalFor(ability)}
            {@const effective = final?.score ?? base(ability)}
            {@const baseScore = base(ability)}
            {@const isPrimary = ability === primaryAbility}
            <div class="flex flex-col gap-1 rounded bg-[var(--color-panel-2)] p-1.5
              {isPrimary ? 'ring-1 ring-[var(--color-accent)]' : ''}">
              <div class="flex items-center justify-between">
                <span class="text-[9px] font-semibold uppercase tracking-wide text-[var(--color-muted)] flex items-center gap-0.5">
                  {ABILITY_LABEL[ability]}
                  {#if isPrimary}
                    <span class="text-[8px] text-[var(--color-accent)]" title="Primary ability for your class">★</span>
                  {/if}
                </span>
                {#if final && final.score !== baseScore}
                  <span class="text-[9px] text-[var(--color-muted)]" title="Base score">
                    base <span class="num">{baseScore}</span>
                  </span>
                {/if}
              </div>

              <!-- Big number = effective (base + bonuses) -->
              <div class="flex items-center gap-1">
                <button
                  type="button"
                  onclick={() => step(ability, -1)}
                  disabled={baseScore <= 1}
                  aria-label={`Decrease ${ABILITY_LABEL[ability]}`}
                  class="flex h-6 w-6 shrink-0 items-center justify-center rounded border border-[var(--color-border)] bg-[var(--color-panel)] text-[12px] leading-none text-[var(--color-ink)] hover:border-[var(--color-accent)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
                >−</button>
                <div class="flex flex-1 flex-col items-center">
                  <span class="num text-[18px] font-semibold tabular-nums leading-none">{effective}</span>
                  <span class="num text-[10px] text-[var(--color-accent)] leading-none">{signed(final?.modifier ?? Math.floor((effective - 10) / 2))}</span>
                </div>
                <button
                  type="button"
                  onclick={() => step(ability, 1)}
                  disabled={baseScore >= 30}
                  aria-label={`Increase ${ABILITY_LABEL[ability]}`}
                  class="flex h-6 w-6 shrink-0 items-center justify-center rounded border border-[var(--color-border)] bg-[var(--color-panel)] text-[12px] leading-none text-[var(--color-ink)] hover:border-[var(--color-accent)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
                >+</button>
              </div>
            </div>
          {/each}
        </div>

      <!-- ===================== STANDARD ARRAY MODE ===================== -->
      {:else if mode === 'standard-array'}
        {#if primaryAbility}
          <p class="text-[10px] text-[var(--color-accent)]">
            Recommended: place <span class="num font-semibold">15</span> in <span class="font-semibold">{ABILITY_LABEL[primaryAbility]}</span> (primary for your class).
          </p>
        {/if}

        <p class="text-[9px] text-[var(--color-muted)]">Assign each value to an ability. Each value can only be used once.</p>

        <div class="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
          {#each STANDARD_ARRAY as value, i}
            {@const isPrimarySlot = i === 0 && primaryAbility !== null}
            <div class="flex flex-col gap-1 rounded bg-[var(--color-panel-2)] p-1.5
              {isPrimarySlot ? 'ring-1 ring-[var(--color-accent)]' : ''}">
              <div class="flex items-center justify-between mb-0.5">
                <span class="num text-[18px] font-semibold leading-none">{value}</span>
                {#if saAssignment[i]}
                  <span class="text-[9px] text-[var(--color-accent)] font-semibold">{ABILITY_LABEL[saAssignment[i]!]}</span>
                {:else}
                  <span class="text-[9px] text-[var(--color-muted)]">unassigned</span>
                {/if}
              </div>
              <select
                value={saAssignment[i] ?? ''}
                onchange={(e) => saAssign(i, (e.currentTarget as HTMLSelectElement).value as AbilityKey || null)}
                class="w-full rounded border border-[var(--color-border)] bg-[var(--color-panel)] px-1 py-0.5 text-[10px] text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
              >
                <option value="">— pick ability —</option>
                {#each ABILITIES as ab}
                  <option
                    value={ab}
                    disabled={saAssigned.has(ab) && saAssignment[i] !== ab}
                  >{ABILITY_LABEL[ab]}</option>
                {/each}
              </select>
            </div>
          {/each}
        </div>

        <div class="flex items-center justify-between gap-2 border-t border-[var(--color-border)] pt-1.5">
          <button
            type="button"
            onclick={saReset}
            class="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
          >Reset</button>
          <button
            type="button"
            onclick={saApply}
            disabled={!saComplete}
            class="rounded bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90 disabled:opacity-40"
            title={saComplete ? 'Apply these scores to your sheet' : 'Assign all 6 values first'}
          >Apply array</button>
        </div>

      <!-- ===================== POINT BUY MODE ===================== -->
      {:else if mode === 'point-buy'}
        <!-- Budget readout -->
        <div class="flex items-center justify-between rounded bg-[var(--color-panel-2)] px-2 py-1">
          <span class="text-[10px] text-[var(--color-muted)]">Budget</span>
          <span class="num text-[13px] font-semibold {pbRemaining < 0 ? 'text-[var(--color-bad)]' : pbRemaining === 0 ? 'text-[var(--color-good)]' : 'text-[var(--color-ink)]'}">
            {pbRemaining} / {PB_BUDGET} pts remaining
          </span>
        </div>

        {#if primaryAbility}
          <p class="text-[10px] text-[var(--color-accent)]">
            Recommended: maximize <span class="font-semibold">{ABILITY_LABEL[primaryAbility]}</span> (primary for your class).
          </p>
        {/if}

        <div class="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
          {#each ABILITIES as ability}
            {@const score = pbScores[ability]}
            {@const mod = Math.floor((score - 10) / 2)}
            {@const isPrimary = ability === primaryAbility}
            {@const canInc = score < PB_MAX && pbRemaining >= (pbCost(score + 1) - pbCost(score))}
            {@const canDec = score > PB_MIN}
            <div class="flex flex-col gap-1 rounded bg-[var(--color-panel-2)] p-1.5
              {isPrimary ? 'ring-1 ring-[var(--color-accent)]' : ''}">
              <div class="flex items-center justify-between">
                <span class="text-[9px] font-semibold uppercase tracking-wide text-[var(--color-muted)] flex items-center gap-0.5">
                  {ABILITY_LABEL[ability]}
                  {#if isPrimary}
                    <span class="text-[8px] text-[var(--color-accent)]" title="Primary ability for your class">★</span>
                  {/if}
                </span>
                <span class="text-[9px] text-[var(--color-muted)]">cost <span class="num">{pbCost(score)}</span></span>
              </div>
              <div class="flex items-center gap-1">
                <button
                  type="button"
                  onclick={() => pbStep(ability, -1)}
                  disabled={!canDec}
                  aria-label={`Decrease ${ABILITY_LABEL[ability]}`}
                  class="flex h-6 w-6 shrink-0 items-center justify-center rounded border border-[var(--color-border)] bg-[var(--color-panel)] text-[12px] leading-none text-[var(--color-ink)] hover:border-[var(--color-accent)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
                >−</button>
                <div class="flex flex-1 flex-col items-center">
                  <span class="num text-[18px] font-semibold tabular-nums leading-none">{score}</span>
                  <span class="num text-[10px] text-[var(--color-accent)] leading-none">{signed(mod)}</span>
                </div>
                <button
                  type="button"
                  onclick={() => pbStep(ability, 1)}
                  disabled={!canInc}
                  aria-label={`Increase ${ABILITY_LABEL[ability]}`}
                  class="flex h-6 w-6 shrink-0 items-center justify-center rounded border border-[var(--color-border)] bg-[var(--color-panel)] text-[12px] leading-none text-[var(--color-ink)] hover:border-[var(--color-accent)] disabled:opacity-30 disabled:hover:border-[var(--color-border)]"
                >+</button>
              </div>
            </div>
          {/each}
        </div>

        <div class="flex items-center justify-between gap-2 border-t border-[var(--color-border)] pt-1.5">
          <button
            type="button"
            onclick={pbReset}
            class="rounded border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
          >Reset</button>
          <button
            type="button"
            onclick={pbApply}
            disabled={pbRemaining < 0}
            class="rounded bg-[var(--color-accent)] px-3 py-1.5 text-[11px] font-semibold text-[var(--color-bg)] hover:opacity-90 disabled:opacity-40"
          >Apply scores</button>
        </div>
      {/if}

    </div>
  {:else}
    <p class="text-[11px] text-[var(--color-muted)]">No character loaded.</p>
  {/if}
</Card>
