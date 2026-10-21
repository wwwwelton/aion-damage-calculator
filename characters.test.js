const { test } = require('node:test');
const assert = require('node:assert/strict');
const { extractCharacter, equipmentBonuses } = require('./characters');

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
