// Small display helpers shared across the cockpit.

/** Signed number: 3 → "+3", -1 → "−1" (true minus sign), 0 → "+0". */
export function signed(n: number): string {
  return n >= 0 ? `+${n}` : `−${Math.abs(n)}`;
}

/** Ability abbreviations in canonical order. */
export const ABILITIES = ['str', 'dex', 'con', 'int', 'wis', 'cha'] as const;
export type AbilityKey = (typeof ABILITIES)[number];

export const ABILITY_LABEL: Record<AbilityKey, string> = {
  str: 'STR',
  dex: 'DEX',
  con: 'CON',
  int: 'INT',
  wis: 'WIS',
  cha: 'CHA'
};

/** Color token for a resource recharge kind. */
export function rechargeColor(recharge: string): string {
  switch (recharge) {
    case 'short-rest':
      return 'var(--color-accent)';
    case 'long-rest':
      return '#9b6bff';
    case 'dawn':
      return 'var(--color-warn)';
    default:
      return 'var(--color-muted)';
  }
}

/** Advantage glyph for a d20 test adv state. */
export function advGlyph(adv: string): { glyph: string; cls: string } {
  switch (adv) {
    case 'advantage':
      return { glyph: '▲', cls: 'text-[var(--color-good)]' };
    case 'disadvantage':
      return { glyph: '▼', cls: 'text-[var(--color-bad)]' };
    default:
      return { glyph: '', cls: '' };
  }
}
