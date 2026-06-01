<script lang="ts">
  import Card from '../Card.svelte';
  import { signed } from '../../format';
  import { app } from '../../state.svelte';

  const sources = $derived(app.computed?.spellcasting ?? []);
  const slots = $derived(app.computed?.spell_slots ?? []);
  const hasCasting = $derived(sources.length > 0);
</script>

<Card title="Spellcasting">
  {#if !hasCasting}
    <p class="text-[11px] text-[var(--color-muted)]">This character has no spellcasting.</p>
  {:else}
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1.5">
        {#each sources as sc}
          <div class="flex items-center gap-2 text-[11px]">
            <span class="flex-1 font-semibold capitalize truncate">{sc.source}</span>
            <span class="flex items-center gap-1 text-[var(--color-muted)]">
              <span class="text-[9px] uppercase tracking-wider">DC</span>
              <span class="num font-semibold text-[var(--color-ink)]">{sc.save_dc ?? '—'}</span>
            </span>
            <span class="flex items-center gap-1 text-[var(--color-muted)]">
              <span class="text-[9px] uppercase tracking-wider">Atk</span>
              <span class="num font-semibold text-[var(--color-ink)]">
                {sc.attack_bonus != null ? signed(sc.attack_bonus) : '—'}
              </span>
            </span>
            {#if sc.prepared != null}
              <span class="flex items-center gap-1 text-[var(--color-muted)]">
                <span class="text-[9px] uppercase tracking-wider">Prep</span>
                <span class="num font-semibold text-[var(--color-ink)]">{sc.prepared}</span>
              </span>
            {/if}
          </div>
        {/each}
      </div>

      {#if slots.length}
        <div class="flex flex-col gap-1 pt-2 border-t border-[var(--color-border)]/60">
          {#each slots as s}
            <div class="flex items-center gap-2 text-[11px]">
              <span class="text-[var(--color-muted)] w-14">Level {s.level}</span>
              <span class="num text-[var(--color-ink)] w-12">{s.current}/{s.max}</span>
              <span class="flex flex-wrap gap-0.5">
                {#each Array(s.max ?? 0) as _, i}
                  <button
                    type="button"
                    onclick={() =>
                      i < s.current ? app.expendSlot(s.level) : app.restoreSlot(s.level)}
                    title={i < s.current ? `Expend L${s.level} slot` : `Restore L${s.level} slot`}
                    aria-label={i < s.current
                      ? `Expend level ${s.level} slot`
                      : `Restore level ${s.level} slot`}
                    class="w-2.5 h-2.5 rounded-full transition-colors hover:ring-1 hover:ring-[var(--color-accent)] {i <
                    s.current
                      ? 'bg-[var(--color-accent)]'
                      : 'border border-[var(--color-border)] bg-[var(--color-panel)]'}"
                  ></button>
                {/each}
              </span>
            </div>
          {/each}
        </div>
      {/if}

      <p class="text-[10px] text-[var(--color-muted)] pt-1 border-t border-[var(--color-border)]/60">
        A searchable spell list is coming — for now track slots and prepared count here.
      </p>
    </div>
  {/if}
</Card>
