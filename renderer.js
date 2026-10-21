const form = document.querySelector('#calculator');
const result = document.querySelector('#result');
const status = document.querySelector('#status');
const upgradeList = document.querySelector('#upgrade-list');
const upgradeContext = document.querySelector('#upgrade-context');
const percentFields = new Set([
  'weaponDamageBoost', 'multiHit', 'powerShard', 'attackIncrease', 'skillCoefficient',
  'damageBoost', 'pveDamageBoost', 'bossDamageBoost', 'bossDamageTolerance', 'speciesDamageBoost', 'criticalDamageBoost',
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
for (const label of form.querySelectorAll('label')) {
  const input = label.querySelector('input');
  if (input) label.append(iconElement(attributeIcon(input.name)));
}
const attackerClass = document.querySelector('#attacker-class');
const targetClass = document.querySelector('#target-class');
function updateActorIcons() {
  setDisplayIcon(document.querySelector('#attacker-icon'), 'class', attackerClass.value);
  setDisplayIcon(document.querySelector('#target-icon'),
    targetMode.value === 'player' ? 'class' : targetMode.value === 'boss' ? 'boss' : 'mob',
    targetClass.value);
}

const targetMode = document.querySelector('#target-mode');
const creatureHeader = document.querySelector('#creature-type-header');
const creatureChoice = document.querySelector('#creature-type');
function updateMode() {
  const mode = targetMode.value;
  document.querySelector('#calculator-view').dataset.mode = mode;
  document.querySelector('#target-kind-label').hidden = mode === 'player';
  document.querySelector('#target-class-label').hidden = mode !== 'player';
  document.querySelector('#result-label').textContent = `${mode === 'player' ? 'PVP · PLAYER' : mode === 'boss' ? 'PVE · BOSS' : 'PVE · MONSTER'} · ESTIMATED HIT`;
  document.querySelector('#enemy-hp-label').textContent = mode === 'player' ? 'Enemy player HP' : mode === 'boss' ? 'Boss HP' : 'Mob HP';
  const explanation = mode === 'player'
    ? 'PvE, Boss and Species bonuses are excluded. PvP Defense uses 1% per point instead of 10%; a Double proc uses half its PvE bonus. PvP-specific Attack, Boost and Tolerance remain reference-only because their exact stacking is unverified.'
    : mode === 'boss'
      ? 'PvE and Boss attack and damage bonuses apply. Boss Damage Tolerance offsets Boss Damage Boost in the additive bucket.'
      : 'PvE and Species terms apply; Boss attack and damage bonuses are excluded against non-boss monsters.';
  updateActorIcons();
  document.querySelector('#mode-note').textContent = `${explanation} Switching target modes clears enemy defenses; offensive values stay entered.`;
}
let previousMode = targetMode.value;
targetMode.addEventListener('change', () => {
  if (targetMode.value !== previousMode) {
    for (const input of form.querySelectorAll('.enemy input')) input.value = '0';
    document.querySelector('#target-name').value = '';
    document.querySelector('#target-class').value = '';
    creatureHeader.value = '';
    creatureChoice.value = '';
    previousMode = targetMode.value;
  }
  updateMode();
  update();
});
creatureHeader.addEventListener('change', () => {
  creatureChoice.value = creatureHeader.value === 'special' ? '' : creatureHeader.value;
  update();
});
creatureChoice.addEventListener('change', () => {
  creatureHeader.value = creatureChoice.value;
});
attackerClass.addEventListener('input', updateActorIcons);
targetClass.addEventListener('input', updateActorIcons);
updateMode();

function update() {
  updateActorIcons();
  const values = {};
  for (const input of form.querySelectorAll('input')) {
    const number = parseNumericInput(input.value, input.min === '' ? -Infinity : Number(input.min)) ?? 0;
    values[input.name] = percentFields.has(input.name) ? number / 100 : number;
  }
  const creatureType = targetMode.value === 'player' ? '' : creatureChoice.value;
  if (creatureType) {
    values.speciesDamageBoost = values[`${creatureType}DamageBoost`] / 100;
    values.speciesDamageTolerance = values[`${creatureType}DamageTolerance`] / 100;
  }
  const critical = document.querySelector('#critical').value === 'yes';
  values.criticalDamageBoost = critical ? values.criticalDamageBoost : 1;
  if (!critical) values.criticalDamageTolerance = 0;
  values.double = document.querySelector('#doubleEnabled').value === 'yes' ? values.double : 1;
  const position = document.querySelector('#position').value;
  values.frontBackDamageBoost = position === 'front' ? values.frontBoost
    : position === 'back' ? values.backBoost : 1;
  values.frontBackDamageTolerance = position === 'front' ? values.frontDamageTolerance
    : position === 'back' ? values.backDamageTolerance : 0;
  const scenarioValues = applyTargetMode(values, targetMode.value);
  const damage = calculateDamage(scenarioValues);
  result.textContent = formatter.format(damage);
  const estimate = estimateCombat(damage, values.hitsPerSecond, values.enemyHp);
  document.querySelector('#estimated-dps').textContent = estimate.dps === null ? '—' : formatter.format(estimate.dps);
  document.querySelector('#estimated-time').textContent = estimate.secondsToDefeat === null
    ? '—' : `${formatter.format(estimate.secondsToDefeat)} s`;
  document.querySelector('#estimate-note').textContent = values.hitsPerSecond <= 0
    ? 'Enter a positive landed-hit rate to estimate DPS. Zero means unknown, not zero damage per second.'
    : values.enemyHp <= 0
      ? 'Enter target HP to estimate time to defeat; zero means unknown. NPC HP is not published in the client data.'
      : 'Assumes constant landed hits, no misses, procs, healing, downtime, phase changes or in-game rounding.';
  document.querySelector('#creature-summary').textContent = creatureType
    ? `${creatureChoice.selectedOptions[0].textContent}: ${formatter.format(values[`${creatureType}DamageBoost`])}% Damage Boost versus ${formatter.format(values[`${creatureType}DamageTolerance`])}% enemy Damage Tolerance applied instead of generic Species values.`
    : 'Generic Species values are used until a creature type is selected.';
  const upgrades = rankDamageUpgrades(scenarioValues, { critical, position });
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
  status.textContent = targetMode.value === 'player'
    ? 'Community PvP estimate: 1% Defense per point; PvP-specific Attack/Boost/Tolerance not included.'
    : creatureType
      ? `Community formula estimate; ${creatureType} creature matchup replaces generic Species values. In-game rounding is not included.`
      : 'Community formula estimate; in-game rounding is not included.';
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
