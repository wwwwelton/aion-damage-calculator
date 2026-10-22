const test = require('node:test');
const assert = require('node:assert/strict');
const { getStatInfo, STAT_CATALOG } = require('./stat-tooltips.js');

test('provides detailed wiki explanation for primary hit detection stats', () => {
  const block = getStatInfo('block');
  assert.ok(block);
  assert.match(block.title, /Block/i);
  assert.match(block.desc, /Parry/i);
  assert.match(block.desc, /bypasses Block/i);

  const blockPen = getStatInfo('blockPenetration');
  assert.ok(blockPen);
  assert.match(blockPen.desc, /Block Chance/i);

  const evasion = getStatInfo('evasion');
  assert.ok(evasion);
  assert.match(evasion.desc, /Dodge/i);

  const accuracy = getStatInfo('accuracy');
  assert.ok(accuracy);
  assert.match(accuracy.desc, /Evasion and Block/i);
});

test('provides detailed wiki explanation for offensive and defensive mechanics', () => {
  const pen = getStatInfo('penetration');
  assert.ok(pen);
  assert.match(pen.desc, /Defense/i);

  const def = getStatInfo('defense');
  assert.ok(def);
  assert.match(def.desc, /reduction/i);

  const crit = getStatInfo('criticalDamageBoost');
  assert.ok(crit);
  assert.match(crit.desc, /150%/);

  const doubleProc = getStatInfo('double');
  assert.ok(doubleProc);
  assert.match(doubleProc.desc, /Double/i);
});

test('resolves genus-specific creature stats dynamically', () => {
  const cogniBlock = getStatInfo('cogniBlock');
  assert.ok(cogniBlock);
  assert.match(cogniBlock.title, /Cogni/);
  assert.match(cogniBlock.desc, /Cogni creature genus/);

  const feraDamage = getStatInfo('feraDamageBoost');
  assert.ok(feraDamage);
  assert.match(feraDamage.title, /Fera/);
});

test('localizes tooltip titles and descriptions with the selected UI language', () => {
  const { I18N } = require('./i18n');
  global.I18N = I18N;
  I18N.setLanguage('pt');
  const penetration = getStatInfo('penetration');
  assert.equal(penetration.title, 'Penetração');
  assert.match(penetration.desc, /Defesa efetiva/i);
  I18N.setLanguage('en');
  delete global.I18N;
});
