<script lang="ts">
  import Card from '../Card.svelte';
  import { signed, advGlyph } from '../../format';
  import { app } from '../../state.svelte';

  const skills = $derived(app.computed?.skills ?? []);

  // Three-letter ability tag for the governing ability.
  const ABBR: Record<string, string> = {
    str: 'STR',
    dex: 'DEX',
    con: 'CON',
    int: 'INT',
    wis: 'WIS',
    cha: 'CHA'
  };

  function label(skill: string): string {
    return skill.replace(/-/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function open(skill: string) {
    app.inspect({ kind: 'skill-bonus', of: skill });
  }

  function pinned(skill: string): boolean {
    const s = app.inspecting?.stat;
    return s?.kind === 'skill-bonus' && s?.of === skill;
  }
</script>

<Card title="Skills">
  <ul class="grid grid-cols-1 gap-px">
    {#each skills as sk}
      {@const adv = advGlyph(sk.test?.adv ?? 'flat')}
      <li>
        <button
          type="button"
          onclick={() => open(sk.skill)}
          class="w-full flex items-center gap-2 px-1.5 py-1 rounded text-left hover:bg-[var(--color-panel-2)]
            {pinned(sk.skill) ? 'bg-[var(--color-panel-2)] ring-1 ring-[var(--color-accent)]' : ''}"
        >
          <span
            class="w-1.5 h-1.5 rounded-full shrink-0 {sk.expertise
              ? 'bg-[var(--color-good)]'
              : sk.proficient
                ? 'bg-[var(--color-accent)]'
                : 'bg-[var(--color-border)]'}"
            title={sk.expertise ? 'Expertise' : sk.proficient ? 'Proficient' : 'Not proficient'}
          ></span>
          <span class="num w-8 text-right font-semibold">{signed(sk.bonus)}</span>
          {#if adv.glyph}<span class="text-[9px] {adv.cls}">{adv.glyph}</span>{/if}
          <span class="flex-1 text-[11px] truncate">{label(sk.skill)}</span>
          <span class="text-[9px] text-[var(--color-muted)]">{ABBR[sk.ability] ?? ''}</span>
        </button>
      </li>
    {/each}
  </ul>
</Card>
