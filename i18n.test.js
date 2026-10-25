const { test } = require('node:test');
const assert = require('node:assert/strict');
const { I18N, TRANSLATIONS } = require('./i18n');

test('Portuguese and English expose the same UI translation contract', () => {
  assert.deepEqual(Object.keys(TRANSLATIONS.pt).sort(), Object.keys(TRANSLATIONS.en).sort());
});

test('language selection changes text and interpolates dynamic messages', () => {
  I18N.setLanguage('pt');
  assert.equal(I18N.t('appTitle'), 'Calculadora de Dano');
  assert.equal(I18N.format('charactersFound', { count: 2, extra: '' }),
    '2 personagem(ns) correspondente(s) encontrado(s).');
  assert.equal(I18N.t('genusBisTitle'), 'Linhas BIS globais de Genus');
  assert.equal(I18N.t('genusCritAccuracy'), 'Crítico, Precisão*');
  I18N.setLanguage('en');
  assert.equal(I18N.t('appTitle'), 'Damage Calculator');
  assert.equal(I18N.t('genusBisTitle'), 'Global BIS Genus Lines');
  assert.equal(I18N.t('genusCritAccuracy'), 'Crit, Accuracy*');
  assert.equal(I18N.format('aiReading', { progress: 42 }), 'Reading embedded model: 42%');
});

test('stat labels follow the selected UI language without changing game identifiers', () => {
  I18N.setLanguage('pt');
  assert.equal(I18N.statLabel('PvP Block Penetration'), 'PvP Penetração de Bloqueio');
  assert.equal(I18N.statLabel('Critical Hit Resist'), 'Resistência a Acerto Crítico');
  I18N.setLanguage('en');
  assert.equal(I18N.statLabel('PvP Block Penetration'), 'PvP Block Penetration');
});
