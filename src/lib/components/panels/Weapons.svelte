<script lang="ts">
  import Card from '../Card.svelte';
  import { signed } from '../../format';
  import { app } from '../../state.svelte';

  const weapons = $derived(app.computed?.weapons ?? []);

  function dmg(w: any): string {
    const d = `${w.damage.count}d${w.damage.sides}`;
    const bonus = w.damage_bonus ? ` ${signed(w.damage_bonus)}` : '';
    return `${d}${bonus}`;
  }
</script>

{#if weapons.length}
  <Card title="Attacks">
    <table class="w-full text-[11px]">
      <thead>
        <tr class="text-[9px] uppercase text-[var(--color-muted)] text-left">
          <th class="font-normal pb-1">Weapon</th>
          <th class="font-normal pb-1 text-right">Atk</th>
          <th class="font-normal pb-1 text-right">Damage</th>
          <th class="font-normal pb-1 text-right">Mastery</th>
        </tr>
      </thead>
      <tbody>
        {#each weapons as w, i}
          <tr class="border-t border-[var(--color-border)]/50">
            <td class="py-1 truncate">{w.name}</td>
            <td class="py-1 text-right">
              <button
                type="button"
                onclick={() => app.inspectWeapon(i, 'attack')}
                class="num font-semibold hover:text-[var(--color-accent)] cursor-pointer"
                title="What makes up this attack bonus?">{signed(w.attack_bonus)} ⓘ</button
              >
            </td>
            <td class="py-1 text-right num">
              <button
                type="button"
                onclick={() => app.inspectWeapon(i, 'damage')}
                class="hover:text-[var(--color-accent)] cursor-pointer"
                title="What makes up this damage?">{dmg(w)}</button
              ><span class="text-[var(--color-muted)] text-[9px]"> {w.damage_type}</span>
            </td>
            <td class="py-1 text-right">
              {#if w.mastery}<span class="text-[9px] text-[var(--color-accent)] capitalize">{w.mastery}</span>{/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </Card>
{/if}
