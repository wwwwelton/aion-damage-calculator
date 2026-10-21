// Multipliers are ratios: 1 = 100%, 1.25 = 125%. Attack values are absolute.
function estimateCombat(damage, hitsPerSecond, targetHp) {
  const dps = damage > 0 && hitsPerSecond > 0 ? damage * hitsPerSecond : null;
  return {
    dps: dps !== null && Number.isFinite(dps) ? dps : null,
    secondsToDefeat: dps > 0 && targetHp > 0 ? targetHp / dps : null
  };
}

function applyTargetMode(values, mode) {
  const selected = { ...values };
  if (mode === 'player') {
    selected.pveAttackPower = 0;
    selected.pveDamageBoost = 0;
    selected.pveDamageTolerance = 0;
  }
  if (mode !== 'boss') {
    selected.bossAttackPower = 0;
    selected.bossDamageBoost = 0;
    selected.bossDamageTolerance = 0;
  }
  if (mode === 'player') {
    selected.speciesAttackPower = 0;
    selected.speciesDamageBoost = 0;
    selected.speciesDamageTolerance = 0;
  }
  selected.defenseScale = mode === 'player' ? 0.01 : 0.1;
  if (mode === 'player' && selected.double > 1) selected.double = 1 + (selected.double - 1) * 0.5;
  return selected;
}

function calculateDamage(values) {
  const v = values;
  const attack = (((v.pureAttack * (v.weaponDamageBoost - (v.weaponDamageTolerance ?? 0)) * v.multiHit * v.powerShard)
    + v.attackBonus) * v.attackIncrease
    + v.pveAttackPower + v.bossAttackPower + v.speciesAttackPower + v.frontBackAttackPower);
  const skill = attack * v.skillCoefficient + v.skillAdditionalDamage;
  const mitigated = skill - Math.max(0, v.defense - v.penetration) * (v.defenseScale ?? 0.1);
  return mitigated * (v.damageBoost + v.pveDamageBoost + v.speciesDamageBoost
    + (v.bossDamageBoost ?? 0) - (v.damageTolerance ?? 0) - (v.pveDamageTolerance ?? 0)
    - (v.bossDamageTolerance ?? 0) - (v.speciesDamageTolerance ?? 0))
    * (v.criticalDamageBoost - (v.criticalDamageTolerance ?? 0)) * v.double
    * (v.frontBackDamageBoost - (v.frontBackDamageTolerance ?? 0)) * v.skillSpecialtyEnhancement;
}

if (typeof module !== 'undefined') module.exports = { calculateDamage, applyTargetMode, estimateCombat };
