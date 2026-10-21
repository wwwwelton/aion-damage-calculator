// Multipliers are ratios: 1 = 100%, 1.25 = 125%. Attack values are absolute.
function calculateDamage(values) {
  const v = values;
  const attack = (((v.pureAttack * (v.weaponDamageBoost - (v.weaponDamageTolerance ?? 0)) * v.multiHit * v.powerShard)
    + v.attackBonus) * v.attackIncrease
    + v.pveAttackPower + v.bossAttackPower + v.speciesAttackPower + v.frontBackAttackPower);
  const skill = attack * v.skillCoefficient + v.skillAdditionalDamage;
  const mitigated = skill - (v.defense - v.penetration) * 0.1;
  return mitigated * (v.damageBoost + v.pveDamageBoost + v.speciesDamageBoost
    - (v.damageTolerance ?? 0) - (v.pveDamageTolerance ?? 0) - (v.speciesDamageTolerance ?? 0))
    * (v.criticalDamageBoost - (v.criticalDamageTolerance ?? 0)) * v.double
    * (v.frontBackDamageBoost - (v.frontBackDamageTolerance ?? 0)) * v.skillSpecialtyEnhancement;
}

if (typeof module !== 'undefined') module.exports = { calculateDamage };
