const { test } = require('node:test');
const assert = require('node:assert/strict');
const { parseNumericInput, formatNumericInput } = require('./numeric-input');

test('accepts comma and dot decimals without changing their magnitude', () => {
  for (const text of ['50,0', '50.0', ' 50,0 ']) assert.equal(parseNumericInput(text, 0), 50);
  assert.equal(parseNumericInput('0,5', 0), 0.5);
  assert.equal(parseNumericInput('.25', 0), 0.25);
  assert.equal(parseNumericInput('-12,5'), -12.5);
});

test('rejects incomplete, ambiguous, nonfinite and below-minimum values', () => {
  for (const text of ['', '50,', '1,2,3', '1.000,5', 'Infinity', '1e309', 'abc']) {
    assert.equal(parseNumericInput(text, 0), null, text);
  }
  assert.equal(parseNumericInput('-1', 0), null);
});

test('normalizes numeric fields on blur while retaining the entered decimal separator', () => {
  assert.equal(formatNumericInput('50,0', 50), '50');
  assert.equal(formatNumericInput('1,250', 1.25), '1,25');
  assert.equal(formatNumericInput('1.250', 1.25), '1.25');
});
