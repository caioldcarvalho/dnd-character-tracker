// A bundled sample CharacterSheet so the cockpit has something to render on first
// launch (and in browser preview, where the Tauri backend may be absent). Matches
// the engine's CharacterSheet serde shape.

export const SAMPLE_SHEET = {
  meta: { name: 'Aldric', player: '', id: 'sample-aldric' },
  abilities: { str: 16, dex: 14, con: 16, int: 10, wis: 12, cha: 13 },
  species: 'human',
  background: 'soldier',
  classes: [{ class: 'fighter', level: 11, subclass: 'battle-master' }],
  feats: [],
  choices: [
    { key: 'background-abilities', picks: ['str', 'str', 'con'] },
    { key: 'fighter-fighting-style', picks: ['defense'] },
    { key: 'battle-master-student-of-war', picks: ['history'] },
    { key: 'human-skill', picks: ['perception'] }
  ],
  hp: { current: 94, temp: 0, rolled: [] },
  resources: {},
  hit_dice_spent: {},
  conditions: [],
  exhaustion: 0,
  active_effects: [],
  equipment: {
    armor: { name: 'Chain Mail', base_ac: 16, kind: 'heavy', dex_cap: 0 },
    shield: true
  },
  weapons: [
    {
      name: 'Longsword',
      kind: 'melee',
      damage: { count: 1, sides: 8 },
      damage_type: 'slashing',
      finesse: false,
      two_handed: false,
      magic_bonus: 1,
      proficient: true,
      mastery: 'sap'
    }
  ],
  concentration: null,
  death_saves: { successes: 0, failures: 0 },
  inspiration: false,
  slots_expended: {},
  notes: ''
};
