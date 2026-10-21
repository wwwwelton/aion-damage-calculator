const { test } = require('node:test');
const assert = require('node:assert/strict');
const { rankDamageUpgrades } = require('./recommendations');

const base = {
  pureAttack: 1000, weaponDamageBoost: 1.5, multiHit: 1, powerShard: 1,
  attackBonus: 0, attackIncrease: 1, pveAttackPower: 0, bossAttackPower: 0,
  speciesAttackPower: 0, frontBackAttackPower: 0, skillCoefficient: 1,
  skillAdditionalDamage: 0, defense: 0, penetration: 0,
  damageBoost: 2, pveDamageBoost: 0, speciesDamageBoost: 0,
  criticalDamageBoost: 1, double: 1, frontBackDamageBoost: 1,
  skillSpecialtyEnhancement: 1
};

test('ranks equal one-point percentage changes using current damage, not a fixed tier list', () => {
  const upgrades = rankDamageUpgrades(base, { critical: false, position: 'none' });
  assert.deepEqual(upgrades.map(item => item.label), ['Weapon Damage Boost', 'Damage Boost']);
  assert.ok(Math.abs(upgrades[0].percent - (1 / 150) * 100) < 1e-9);
  assert.ok(Math.abs(upgrades[1].percent - 0.5) < 1e-9);
});

test('only includes critical and the selected positional boost when they apply', () => {
  const values = { ...base, criticalDamageBoost: 1.5, frontBackDamageBoost: 1.1 };
  const upgrades = rankDamageUpgrades(values, { critical: true, position: 'back' });
  assert.deepEqual(upgrades.map(item => item.label), [
    'Back Attack Damage Boost', 'Weapon Damage Boost', 'Critical Damage Boost', 'Damage Boost'
  ]);
  assert.ok(Math.abs(upgrades[0].percent - 100 / 110) < 1e-9);
});

test('does not claim percentage gains from a non-positive baseline', () => {
  assert.deepEqual(rankDamageUpgrades({ ...base, damageBoost: 0 }, { critical: false, position: 'none' }), []);
});
