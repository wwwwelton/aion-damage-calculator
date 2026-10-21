const form = document.querySelector('#calculator');
const result = document.querySelector('#result');
const status = document.querySelector('#status');
const upgradeList = document.querySelector('#upgrade-list');
const upgradeContext = document.querySelector('#upgrade-context');
const percentFields = new Set([
  'weaponDamageBoost', 'multiHit', 'powerShard', 'attackIncrease', 'skillCoefficient',
  'damageBoost', 'pveDamageBoost', 'speciesDamageBoost', 'criticalDamageBoost',
  'weaponDamageTolerance', 'damageTolerance', 'pveDamageTolerance',
  'speciesDamageTolerance', 'criticalDamageTolerance', 'frontDamageTolerance',
  'backDamageTolerance', 'doubleChance', 'doubleResist', 'perfectChance',
  'perfectResist', 'multiHitChance', 'multiHitResist', 'endurancePenetration', 'endurance',
  'double', 'frontBoost', 'backBoost', 'skillSpecialtyEnhancement'
]);
const formatter = new Intl.NumberFormat('en-US', { maximumFractionDigits: 2 });
const numericInputs = form.querySelectorAll('input[type="number"]');
for (const input of numericInputs) {
  input.type = 'text';
  input.inputMode = 'decimal';
  input.autocomplete = 'off';
}

function update() {
  const values = {};
  for (const input of form.querySelectorAll('input')) {
    const number = parseNumericInput(input.value, input.min === '' ? -Infinity : Number(input.min)) ?? 0;
    values[input.name] = percentFields.has(input.name) ? number / 100 : number;
  }
  const critical = document.querySelector('#critical').value === 'yes';
  values.criticalDamageBoost = critical ? values.criticalDamageBoost : 1;
  values.double = document.querySelector('#doubleEnabled').value === 'yes' ? values.double : 1;
  const position = document.querySelector('#position').value;
  values.frontBackDamageBoost = position === 'front' ? values.frontBoost
    : position === 'back' ? values.backBoost : 1;
  values.frontBackDamageTolerance = position === 'front' ? values.frontDamageTolerance
    : position === 'back' ? values.backDamageTolerance : 0;
  const damage = calculateDamage(values);
  result.textContent = formatter.format(damage);
  const upgrades = rankDamageUpgrades(values, { critical, position });
  upgradeList.replaceChildren(...upgrades.map(({ label, percent }) => {
    const item = document.createElement('li');
    item.textContent = `${label}: +${percent.toFixed(2)}% damage`;
    return item;
  }));
  const notes = [];
  if (!upgrades.length) notes.push('A positive damage baseline is required to compare percentage gains.');
  if (!critical) notes.push('Critical Damage Boost applies only to critical hits; select Critical Hit to compare it.');
  if (position === 'none') notes.push('Select Front or Back to compare a positional boost.');
  else notes.push('Positional gains apply only when attacks land from the selected side.');
  notes.push('Double Chance is a proc rate, not a guaranteed multiplier; proc chance and resistance are not modeled.');
  upgradeContext.textContent = notes.join(' ');
  status.textContent = 'Result of the supplied formula; in-game rounding is not included.';
}
form.addEventListener('input', update);
form.addEventListener('focusout', (event) => {
  const input = event.target;
  if (!input.matches('input')) return;
  const number = parseNumericInput(input.value, input.min === '' ? -Infinity : Number(input.min));
  input.value = number === null ? '0' : formatNumericInput(input.value, number);
  update();
});
form.addEventListener('change', update);
update();
