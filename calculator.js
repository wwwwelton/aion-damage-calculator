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
  if (mode === 'player' && selected.criticalDamageBoost > 1) {
    selected.criticalDamageBoost = Math.max(1, selected.criticalDamageBoost - 0.25);
  }
  return selected;
}

function calculateDamage(values) {
  const v = values;
  const weaponMultiplier = Math.max(0, v.weaponDamageBoost - (v.weaponDamageTolerance ?? 0));
  const attack = (((v.pureAttack * weaponMultiplier * v.multiHit * v.powerShard)
    + v.attackBonus) * v.attackIncrease
    + v.pveAttackPower + v.bossAttackPower + v.speciesAttackPower + v.frontBackAttackPower);
  const skill = attack * v.skillCoefficient + v.skillAdditionalDamage;
  const mitigated = Math.max(0,
    skill - Math.max(0, v.defense - v.penetration) * (v.defenseScale ?? 0.1));
  const damageMultiplier = Math.max(0, v.damageBoost + v.pveDamageBoost + v.speciesDamageBoost
    + (v.bossDamageBoost ?? 0) - (v.damageTolerance ?? 0) - (v.pveDamageTolerance ?? 0)
    - (v.bossDamageTolerance ?? 0) - (v.speciesDamageTolerance ?? 0));
  const criticalMultiplier = Math.max(0, v.criticalDamageBoost - (v.criticalDamageTolerance ?? 0));
  const positionalMultiplier = Math.max(0, v.frontBackDamageBoost - (v.frontBackDamageTolerance ?? 0));
  return mitigated * damageMultiplier * criticalMultiplier * v.double
    * positionalMultiplier * v.skillSpecialtyEnhancement;
}

// Keys describing what the displayed hit and estimate take for granted. The hit itself is
// deterministic; everything probabilistic (crit, procs, position uptime, cadence) is assumed, not modelled.
function estimateAssumptions(values, { critical = false, double = false, position = 'none' } = {}) {
  const keys = [];
  if (critical) keys.push('assumeCritLands');
  if (double) keys.push('assumeDoubleProcs');
  keys.push(position === 'none' ? 'positionComparisonHint' : 'positionUptimeHint');
  if ((values?.hitsPerSecond ?? 0) > 0) keys.push('assumeConstantRate');
  keys.push('resistNotModelled', 'roundingNotModelled');
  return keys;
}

if (typeof module !== 'undefined') module.exports = { calculateDamage, applyTargetMode, estimateCombat, estimateAssumptions };
