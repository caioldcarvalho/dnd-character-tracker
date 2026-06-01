<script lang="ts">
  import Card from '../Card.svelte';
  import { signed } from '../../format';
  import { app } from '../../state.svelte';

  // Built-in 2024 SRD armor list. dex_cap: light=null (full DEX), medium=2, heavy=0.
  const ARMORS = [
    { name: 'Padded', base_ac: 11, kind: 'light', dex_cap: null },
    { name: 'Leather', base_ac: 11, kind: 'light', dex_cap: null },
    { name: 'Studded Leather', base_ac: 12, kind: 'light', dex_cap: null },
    { name: 'Hide', base_ac: 12, kind: 'medium', dex_cap: 2 },
    { name: 'Chain Shirt', base_ac: 13, kind: 'medium', dex_cap: 2 },
    { name: 'Scale Mail', base_ac: 14, kind: 'medium', dex_cap: 2 },
    { name: 'Breastplate', base_ac: 14, kind: 'medium', dex_cap: 2 },
    { name: 'Half Plate', base_ac: 15, kind: 'medium', dex_cap: 2 },
    { name: 'Ring Mail', base_ac: 14, kind: 'heavy', dex_cap: 0 },
    { name: 'Chain Mail', base_ac: 16, kind: 'heavy', dex_cap: 0 },
    { name: 'Splint', base_ac: 17, kind: 'heavy', dex_cap: 0 },
    { name: 'Plate', base_ac: 18, kind: 'heavy', dex_cap: 0 }
  ] as const;

  const UNARMORED = '__unarmored__';

  const sheet = $derived(app.sheet);
  const equipment = $derived(sheet?.equipment ?? null);
  const armor = $derived(equipment?.armor ?? null);
  const shield = $derived(equipment?.shield ?? false);
  const weapons = $derived(sheet?.weapons ?? []);
  const computedWeapons = $derived(app.computed?.weapons ?? []);
  const ac = $derived(app.computed?.armor_class?.total ?? null);

  function onPickArmor(e: Event) {
    const v = (e.currentTarget as HTMLSelectElement).value;
    if (v === UNARMORED) {
      app.setArmor(null);
      return;
    }
    const found = ARMORS.find((a) => a.name === v);
    if (found) app.setArmor({ ...found });
  }

  function compDmg(c: any): string {
    if (!c) return '';
    const d = `${c.damage?.count ?? 0}d${c.damage?.sides ?? 0}`;
    const bonus = c.damage_bonus ? ` ${signed(c.damage_bonus)}` : '';
    return `${d}${bonus}`;
  }

  const inputCls =
    'bg-[var(--color-panel-2)] border border-[var(--color-border)] rounded px-2 py-1 text-[11px] focus:border-[var(--color-accent)] focus:outline-none';
</script>

<div class="flex flex-col gap-2">
  <Card title="Armor">
    {#if !sheet}
      <p class="text-[11px] text-[var(--color-muted)]">No character loaded.</p>
    {:else}
      <div class="flex flex-col gap-2">
        <div class="flex items-center gap-2 flex-wrap">
          <label class="text-[10px] uppercase tracking-wider text-[var(--color-muted)]" for="gear-armor">
            Armor
          </label>
          <select id="gear-armor" class="{inputCls} flex-1 min-w-[140px]" value={armor?.name ?? UNARMORED} onchange={onPickArmor}>
            <option value={UNARMORED}>— Unarmored —</option>
            {#each ARMORS as a}
              <option value={a.name}>{a.name} ({a.base_ac}, {a.kind})</option>
            {/each}
          </select>
        </div>

        <div class="flex items-center justify-between gap-2 flex-wrap">
          <label class="flex items-center gap-1.5 text-[11px] cursor-pointer select-none">
            <input
              type="checkbox"
              checked={shield}
              onchange={() => app.toggleShield()}
              class="accent-[var(--color-accent)]"
            />
            <span class={shield ? 'text-[var(--color-ink)]' : 'text-[var(--color-muted)]'}>Shield (+2 AC)</span>
          </label>

          <button
            type="button"
            onclick={() => app.inspect({ kind: 'armor-class' })}
            title="Show AC breakdown"
            class="flex items-center gap-1.5 rounded bg-[var(--color-panel-2)] border border-[var(--color-border)] px-2 py-1 hover:border-[var(--color-accent)] transition-colors"
          >
            <span class="text-[9px] uppercase tracking-wider text-[var(--color-muted)]">AC</span>
            <span class="num text-[13px] font-semibold text-[var(--color-ink)]">{ac ?? '—'}</span>
            <span class="text-[10px] text-[var(--color-accent)]">ⓘ</span>
          </button>
        </div>

        {#if armor}
          <p class="text-[9px] text-[var(--color-muted)]">
            {armor.name}: base <span class="num">{armor.base_ac}</span> · {armor.kind}{#if armor.dex_cap !== null} · DEX cap <span class="num">{armor.dex_cap}</span>{/if}
          </p>
        {:else}
          <p class="text-[9px] text-[var(--color-muted)]">Unarmored: <span class="num">10</span> + full DEX</p>
        {/if}
      </div>
    {/if}
  </Card>

  <Card title="Weapons">
    {#if !sheet}
      <p class="text-[11px] text-[var(--color-muted)]">No character loaded.</p>
    {:else}
      <div class="flex flex-col gap-1.5">
        {#if weapons.length === 0}
          <p class="text-[11px] text-[var(--color-muted)] py-1">No weapons. Add one below.</p>
        {/if}

        {#each weapons as w, i (i)}
          {@const c = computedWeapons[i]}
          <div class="rounded border border-[var(--color-border)] bg-[var(--color-panel-2)]/40 p-1.5 flex flex-col gap-1.5">
            <!-- row 1: name + kind + remove -->
            <div class="flex items-center gap-1.5">
              <input
                class="{inputCls} flex-1 min-w-0 font-semibold"
                value={w.name}
                placeholder="Weapon name"
                oninput={(e) => app.updateWeapon(i, { name: (e.currentTarget as HTMLInputElement).value })}
              />
              <select
                class={inputCls}
                value={w.kind}
                onchange={(e) => app.updateWeapon(i, { kind: (e.currentTarget as HTMLSelectElement).value })}
              >
                <option value="melee">melee</option>
                <option value="ranged">ranged</option>
              </select>
              <button
                type="button"
                onclick={() => app.removeWeapon(i)}
                title="Remove weapon"
                aria-label="Remove weapon"
                class="w-6 h-6 shrink-0 rounded border border-[var(--color-border)] text-[var(--color-muted)] hover:text-[var(--color-bad)] hover:border-[var(--color-bad)] transition-colors leading-none"
              >×</button>
            </div>

            <!-- row 2: damage + type + magic bonus -->
            <div class="flex items-center gap-1.5 flex-wrap">
              <div class="flex items-center gap-1">
                <input
                  type="number"
                  min="1"
                  class="{inputCls} w-12 num text-right"
                  value={w.damage?.count ?? 1}
                  oninput={(e) =>
                    app.updateWeapon(i, {
                      damage: { count: Number((e.currentTarget as HTMLInputElement).value) || 0, sides: w.damage?.sides ?? 0 }
                    })}
                />
                <span class="text-[11px] text-[var(--color-muted)]">d</span>
                <input
                  type="number"
                  min="1"
                  class="{inputCls} w-12 num text-right"
                  value={w.damage?.sides ?? 6}
                  oninput={(e) =>
                    app.updateWeapon(i, {
                      damage: { count: w.damage?.count ?? 0, sides: Number((e.currentTarget as HTMLInputElement).value) || 0 }
                    })}
                />
              </div>
              <input
                class="{inputCls} w-20 min-w-0 flex-1"
                value={w.damage_type}
                placeholder="type"
                oninput={(e) => app.updateWeapon(i, { damage_type: (e.currentTarget as HTMLInputElement).value })}
              />
              <div class="flex items-center gap-1" title="Magic bonus">
                <span class="text-[10px] text-[var(--color-muted)]">magic</span>
                <input
                  type="number"
                  class="{inputCls} w-12 num text-right"
                  value={w.magic_bonus ?? 0}
                  oninput={(e) => app.updateWeapon(i, { magic_bonus: Number((e.currentTarget as HTMLInputElement).value) || 0 })}
                />
              </div>
            </div>

            <!-- row 3: flags + mastery -->
            <div class="flex items-center gap-2.5 flex-wrap">
              <label class="flex items-center gap-1 text-[10px] cursor-pointer select-none text-[var(--color-muted)]">
                <input
                  type="checkbox"
                  checked={w.finesse}
                  onchange={(e) => app.updateWeapon(i, { finesse: (e.currentTarget as HTMLInputElement).checked })}
                  class="accent-[var(--color-accent)]"
                />finesse
              </label>
              <label class="flex items-center gap-1 text-[10px] cursor-pointer select-none text-[var(--color-muted)]">
                <input
                  type="checkbox"
                  checked={w.two_handed}
                  onchange={(e) => app.updateWeapon(i, { two_handed: (e.currentTarget as HTMLInputElement).checked })}
                  class="accent-[var(--color-accent)]"
                />2H
              </label>
              <label class="flex items-center gap-1 text-[10px] cursor-pointer select-none text-[var(--color-muted)]">
                <input
                  type="checkbox"
                  checked={w.proficient}
                  onchange={(e) => app.updateWeapon(i, { proficient: (e.currentTarget as HTMLInputElement).checked })}
                  class="accent-[var(--color-accent)]"
                />prof
              </label>
              <input
                class="{inputCls} w-24 min-w-0 flex-1"
                value={w.mastery ?? ''}
                placeholder="mastery"
                oninput={(e) => {
                  const v = (e.currentTarget as HTMLInputElement).value;
                  app.updateWeapon(i, { mastery: v === '' ? null : v });
                }}
              />
            </div>

            <!-- computed result -->
            <div class="flex items-center gap-3 pt-1 border-t border-[var(--color-border)]/50 text-[10px]">
              <span class="uppercase tracking-wider text-[var(--color-muted)] text-[9px]">live</span>
              <span class="flex items-center gap-1">
                <span class="text-[var(--color-muted)] text-[9px]">atk</span>
                <span class="num font-semibold text-[var(--color-good)]">{c ? signed(c.attack_bonus) : '—'}</span>
              </span>
              <span class="flex items-center gap-1">
                <span class="text-[var(--color-muted)] text-[9px]">dmg</span>
                <span class="num text-[var(--color-ink)]">{c ? compDmg(c) : '—'}</span>
                {#if c?.damage_type}<span class="text-[var(--color-muted)] text-[9px]">{c.damage_type}</span>{/if}
              </span>
              {#if c?.mastery}
                <span class="text-[9px] text-[var(--color-accent)] capitalize">{c.mastery}</span>
              {/if}
            </div>
          </div>
        {/each}

        <button
          type="button"
          onclick={() => app.addWeapon()}
          class="mt-0.5 self-start rounded bg-[var(--color-panel-2)] border border-[var(--color-border)] px-2.5 py-1 text-[11px] text-[var(--color-muted)] hover:text-[var(--color-ink)] hover:border-[var(--color-accent)] transition-colors"
        >+ Add weapon</button>
      </div>
    {/if}
  </Card>
</div>
