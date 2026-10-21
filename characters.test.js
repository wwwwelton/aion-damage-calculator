const { test } = require('node:test');
const assert = require('node:assert/strict');
const { extractCharacter, equipmentBonuses, enemyDefenses, offensiveReferences, characterHp } = require('./characters');

const profile = {
  info: { profile: { characterName: 'Kanon', serverName: 'Siel', className: 'Cleric', characterLevel: 45, combatPower: 90000 },
    stat: { statList: [{ name: 'Might', value: 12 }] } },
  equipment: { equipment: { equipmentList: [{ slotPos: 1, slotPosName: 'MainHand', name: 'Mace' }] },
    skill: { skillList: [{ name: 'Bolt', skillLevel: 12, acquired: 1, category: 'Active' },
      { name: 'Locked', skillLevel: 0, acquired: 0 }] } },
  itemDetails: { '1': { subStats: [{ name: 'Damage Boost', value: '5.6%' },
    { name: 'PvE Attack', value: '20' }, { name: 'Weapon Damage Boost', value: '3%' },
    { name: 'Attack', value: '251' }, { name: 'Critical Damage Boost', value: '2%' }],
    mainStats: [{ name: 'Attack', value: '323', minValue: '251', extra: '119' }] } }
};

test('shows public attributes, equipped items and learned skills without treating item attack as Pure Attack', () => {
  const data = extractCharacter(profile);
  assert.equal(data.name, 'Kanon');
  assert.deepEqual(data.stats, ['Might: 12']);
  assert.deepEqual(data.skills, ['Bolt · Lv 12 · Active']);
  assert.equal(data.gear.length, 1);
  assert.match(data.gear[0], /Damage Boost \+5.6%/);
});

test('imports only explicit numeric gear bonuses with their appropriate percentage baselines', () => {
  assert.deepEqual(equipmentBonuses(profile), {
    damageBoost: 105.6, pveAttackPower: 20, weaponDamageBoost: 103, criticalDamageBoost: 152
  });
});

test('ignores incomplete item details and invalid bonus units rather than inventing missing totals', () => {
  assert.deepEqual(equipmentBonuses({ equipment: { equipment: { equipmentList: [] } }, itemDetails: { '1': profile.itemDetails['1'] } }), {});
  const bad = structuredClone(profile);
  bad.itemDetails['1'].subStats = [{ name: 'Damage Boost', value: '560' }, { name: 'Attack Bonus', value: 'unknown' }];
  assert.deepEqual(equipmentBonuses(bad), {});
});

test('imports explicit enemy sheet defenses without substituting offensive or inferred gear stats', () => {
  const enemy = structuredClone(profile);
  enemy.info.stat.statList = [
    { name: 'Defense', value: '825' },
    { name: 'Damage Tolerance', value: '12.5%' },
    { name: 'PvE Damage Tolerance', value: '4%' },
    { name: 'Weapon Damage Tolerance', value: '3%' },
    { name: 'Critical Damage Tolerance', value: '5%' },
    { name: 'Back Attack Damage Tolerance', value: '7%' },
    { name: 'Block', value: '162' },
    { name: 'Evasion', value: '22' },
    { name: 'Double Resist', value: '8%' },
    { name: 'Endurance', value: '3%' },
    { name: 'Attack', value: '500' },
    { name: 'Boss Damage Tolerance', value: '?' }
  ];
  assert.deepEqual(enemyDefenses(enemy), {
    defense: 825, damageTolerance: 12.5, pveDamageTolerance: 4,
    weaponDamageTolerance: 3, criticalDamageTolerance: 5,
    backDamageTolerance: 7, block: 162, evasion: 22, doubleResist: 8, endurance: 3
  });
});

test('uses explicit equipped defenses as partial values when public sheet omits them', () => {
  const enemy = structuredClone(profile);
  enemy.itemDetails['1'].subStats.push({ name: 'Defense', value: '230' },
    { name: 'Block', value: '15' }, { name: 'Damage Tolerance', value: '2.5%' },
    { name: 'Weapon Damage Tolerance', value: '4%' });
  enemy.itemDetails['1'].mainStats.push({ name: 'Defense', value: '120' });
  assert.deepEqual(enemyDefenses(enemy), {
    defense: 350, block: 15, damageTolerance: 2.5, weaponDamageTolerance: 4
  });
  enemy.info.stat.statList.push({ name: 'Defense', value: '890' });
  assert.equal(enemyDefenses(enemy).defense, 890);
});

test('first character supplies comparable offensive stats from sheet, then explicit gear only when absent', () => {
  const player = structuredClone(profile);
  player.info.stat.statList.push({ name: 'Accuracy', value: '1200' },
    { name: 'Critical Hit', value: '475' });
  player.itemDetails['1'].subStats.push({ name: 'Accuracy', value: '150' },
    { name: 'Block Penetration', value: '28' },
    { name: 'Double Chance', value: '3.5%' },
    { name: 'Cogni Damage Boost', value: '4%' },
    { name: 'Fire Boost', value: '2.5%' },
    { name: 'Critical Hit', value: '30' },
    { name: 'Defense', value: '150' });
  assert.deepEqual(offensiveReferences(player), {
    accuracy: 1200, criticalHit: 475, blockPenetration: 28,
    doubleChance: 3.5, cogniDamageBoost: 4, fireBoost: 2.5
  });
});

test('imports only explicit positive character-sheet HP, never item or missing HP', () => {
  assert.equal(characterHp(profile), null);
  const player = structuredClone(profile);
  player.info.stat.statList.push({ name: 'HP', value: 12500 });
  assert.equal(characterHp(player), 12500);
  player.info.stat.statList.at(-1).value = 'unknown';
  assert.equal(characterHp(player), null);
});
