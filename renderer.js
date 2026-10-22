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
let formatter = new Intl.NumberFormat(I18N.currentLang === 'pt' ? 'pt-BR' : 'en-US', { maximumFractionDigits: 2 });
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
  document.querySelector('#result-label').textContent = I18N.t(
    mode === 'player' ? 'resultPlayer' : mode === 'boss' ? 'resultBoss' : 'resultMonster');
  document.querySelector('#enemy-hp-label').textContent = I18N.t(
    mode === 'player' ? 'enemyPlayerHp' : mode === 'boss' ? 'bossHp' : 'mobHp');
  const explanation = I18N.t(mode === 'player' ? 'modePlayerNote'
    : mode === 'boss' ? 'modeBossNote' : 'modeMonsterNote');
  updateActorIcons();
  document.querySelector('#mode-note').textContent = `${explanation} ${I18N.t('modeSwitchNote')}`;
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
    ? I18N.t('estimateNeedRate')
    : values.enemyHp <= 0
      ? I18N.t('estimateNeedHp')
      : I18N.t('estimateAssumptions');
  document.querySelector('#creature-summary').textContent = creatureType
    ? I18N.format('creatureSelected', {
      type: creatureChoice.selectedOptions[0].textContent,
      boost: formatter.format(values[`${creatureType}DamageBoost`]),
      tolerance: formatter.format(values[`${creatureType}DamageTolerance`])
    })
    : I18N.t('creatureGeneric');
  const upgrades = rankDamageUpgrades(scenarioValues, { critical, position });
  const upgradeKeys = {
    weaponDamageBoost: 'weaponDamageBoostLabel',
    damageBoost: 'damageBoostLabel',
    criticalDamageBoost: 'criticalDamageBoostLabel',
    frontBackDamageBoost: position === 'front' ? 'frontBoostLabel' : 'backBoostLabel'
  };
  upgradeList.replaceChildren(...upgrades.map(({ field, label, percent }) => {
    const item = document.createElement('li');
    item.dataset.field = field;
    item.dataset.label = I18N.t(upgradeKeys[field], label).replace(/\s*\(%\)$/, '');
    item.textContent = `${item.dataset.label}: ${I18N.format('damageGain', { percent: percent.toFixed(2) })}`;
    return item;
  }));
  const notes = [];
  if (!upgrades.length) notes.push(I18N.t('noPositiveBaseline'));
  if (!critical) notes.push(I18N.t('criticalComparisonHint'));
  if (position === 'none') notes.push(I18N.t('positionComparisonHint'));
  else notes.push(I18N.t('positionUptimeHint'));
  notes.push(I18N.t('procHint'));
  upgradeContext.textContent = notes.join(' ');
  status.textContent = targetMode.value === 'player'
    ? I18N.t('communityPvpStatus')
    : creatureType
      ? I18N.format('communityCreatureStatus', { type: creatureChoice.selectedOptions[0].textContent })
      : I18N.t('communityFormulaStatus');
  scheduleAiTips();
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

I18N.onLanguageChange(() => {
  formatter = new Intl.NumberFormat(I18N.currentLang === 'pt' ? 'pt-BR' : 'en-US',
    { maximumFractionDigits: 2 });
  updateMode();
  update();
});
