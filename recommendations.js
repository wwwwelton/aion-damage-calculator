const damageFormula = typeof module !== 'undefined' ? require('./calculator').calculateDamage : calculateDamage;
// Compare equal +1 percentage-point changes to stats the supplied formula can model.
function rankDamageUpgrades(values, conditions) {
  const baseline = damageFormula(values);
  if (!Number.isFinite(baseline) || baseline <= 0) return [];

  const candidates = [
    ['Weapon Damage Boost', 'weaponDamageBoost'],
    ['Damage Boost', 'damageBoost']
  ];
  if (conditions.critical) candidates.push(['Critical Damage Boost', 'criticalDamageBoost']);
  if (conditions.position !== 'none') {
    candidates.push([
      conditions.position === 'front' ? 'Front Attack Damage Boost' : 'Back Attack Damage Boost',
      'frontBackDamageBoost'
    ]);
  }

  return candidates.map(([label, key]) => {
    const upgraded = damageFormula({ ...values, [key]: values[key] + 0.01 });
    return { label, percent: (upgraded - baseline) / baseline * 100 };
  }).filter(item => Number.isFinite(item.percent) && item.percent > 0)
    .sort((a, b) => b.percent - a.percent);
}

if (typeof module !== 'undefined') module.exports = { rankDamageUpgrades };
