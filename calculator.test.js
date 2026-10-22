const { test } = require('node:test');
const assert = require('node:assert/strict');
const { calculateDamage, applyTargetMode, estimateCombat } = require('./calculator');

const base = {
  pureAttack: 1000, weaponDamageBoost: 1, multiHit: 1, powerShard: 1,
  attackBonus: 0, attackIncrease: 1, pveAttackPower: 0, bossAttackPower: 0,
  speciesAttackPower: 0, frontBackAttackPower: 0, skillCoefficient: 1,
  skillAdditionalDamage: 0, defense: 0, penetration: 0,
  damageBoost: 1, pveDamageBoost: 0, speciesDamageBoost: 0,
  criticalDamageBoost: 1, double: 1, frontBackDamageBoost: 1,
  skillSpecialtyEnhancement: 1
};

test('combina ataque, bônus aditivos, habilidade, mitigação e multiplicadores na ordem fornecida', () => {
  const values = {
    ...base, pureAttack: 200, weaponDamageBoost: 1.5, multiHit: 2,
    powerShard: 1.25, attackBonus: 50, attackIncrease: 1.1,
    pveAttackPower: 10, bossAttackPower: 20, speciesAttackPower: 30,
    frontBackAttackPower: 40, skillCoefficient: 2, skillAdditionalDamage: 15,
    defense: 300, penetration: 100, damageBoost: 1.1,
    pveDamageBoost: 0.2, speciesDamageBoost: 0.3,
    criticalDamageBoost: 1.5, double: 2, frontBackDamageBoost: 1.2,
    skillSpecialtyEnhancement: 1.1
  };
  // ((200*1.5*2*1.25+50)*1.1+10+20+30+40)*2+15-(300-100)*0.1 = 1955
  assert.ok(Math.abs(calculateDamage(values) - 1955 * 1.6 * 1.5 * 2 * 1.2 * 1.1) < 1e-9);
});

test('penetration cannot reduce effective defense below zero', () => {
  assert.equal(calculateDamage({ ...base, defense: 50, penetration: 150 }), 1000);
});

test('enemy tolerances reduce their matching damage stages without changing neutral defaults', () => {
  const v = { ...base, weaponDamageBoost: 1.5, criticalDamageBoost: 1.5,
    frontBackDamageBoost: 1.2, pveDamageBoost: 0.3, speciesDamageBoost: 0.2,
    weaponDamageTolerance: 0.1, damageTolerance: 0.1, pveDamageTolerance: 0.05,
    speciesDamageTolerance: 0.05, criticalDamageTolerance: 0.2,
    frontBackDamageTolerance: 0.1 };
  assert.ok(Math.abs(calculateDamage(v) - 1400 * 1.3 * 1.3 * 1.1) < 1e-9);
  assert.equal(calculateDamage(base), 1000);
});

test('hit and proc resistances do not pretend to change a landed hit', () => {
  assert.equal(calculateDamage({ ...base, block: 1000, evasion: 1000,
    criticalHitResist: 50, doubleResist: 0.5, endurance: 0.5 }), 1000);
});

test('switching between boss, monster and player excludes inapplicable damage terms without losing inputs', () => {
  const values = { ...base, pveAttackPower: 100, bossAttackPower: 200,
    speciesAttackPower: 300, pveDamageBoost: 0.1, speciesDamageBoost: 0.2,
    pveDamageTolerance: 0.05, speciesDamageTolerance: 0.02 };
  assert.ok(Math.abs(calculateDamage(applyTargetMode(values, 'boss')) - 1600 * 1.23) < 1e-9);
  assert.ok(Math.abs(calculateDamage(applyTargetMode(values, 'monster')) - 1400 * 1.23) < 1e-9);
  assert.equal(calculateDamage(applyTargetMode(values, 'player')), 1000);
  assert.ok(Math.abs(calculateDamage(applyTargetMode(values, 'boss')) - 1600 * 1.23) < 1e-9);
});

test('boss boost and tolerance only apply to bosses and share the additive bucket', () => {
  const values = { ...base, bossDamageBoost: 0.3, bossDamageTolerance: 0.1 };
  assert.equal(calculateDamage(applyTargetMode(values, 'boss')), 1200);
  assert.equal(calculateDamage(applyTargetMode(values, 'monster')), 1000);
  assert.equal(calculateDamage(applyTargetMode(values, 'player')), 1000);
});

test('PvP defense scales by 1% and a Double proc has half the PvE bonus', () => {
  const values = { ...base, defense: 100, double: 2 };
  assert.equal(calculateDamage(applyTargetMode(values, 'boss')), 1980);
  assert.equal(calculateDamage(applyTargetMode(values, 'player')), 1498.5);
  assert.equal(calculateDamage(applyTargetMode({ ...values, penetration: 200 }, 'player')), 1500);
});

test('PvP critical hits use the 1.25 base multiplier instead of the PvE 1.5 base', () => {
  const values = { ...base, criticalDamageBoost: 1.5 };
  assert.equal(calculateDamage(applyTargetMode(values, 'boss')), 1500);
  assert.equal(calculateDamage(applyTargetMode(values, 'player')), 1250);
});

test('damage stages cannot turn a landed hit into negative damage', () => {
  assert.equal(calculateDamage({ ...base, defense: 20000 }), 0);
  assert.equal(calculateDamage({ ...base, damageTolerance: 2 }), 0);
  assert.equal(calculateDamage({ ...base, criticalDamageTolerance: 2 }), 0);
});

test('DPS and defeat time require a positive hit rate and actual target HP', () => {
  assert.deepEqual(estimateCombat(1200, 2.5, 9000), { dps: 3000, secondsToDefeat: 3 });
  assert.deepEqual(estimateCombat(1200, 2.5, 0), { dps: 3000, secondsToDefeat: null });
  assert.deepEqual(estimateCombat(1200, 0, 9000), { dps: null, secondsToDefeat: null });
  assert.deepEqual(estimateCombat(-100, 2, 9000), { dps: null, secondsToDefeat: null });
});
