const { test } = require('node:test');
const assert = require('node:assert/strict');
const { calculateDamage } = require('./calculator');

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

test('penetração acima da defesa mantém o sinal da subtração especificada', () => {
  assert.equal(calculateDamage({ ...base, defense: 50, penetration: 150 }), 1010);
});
