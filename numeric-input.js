function parseNumericInput(text, min = -Infinity) {
  const trimmed = text.trim();
  if (!/^[+-]?(?:\d+(?:[.,]\d+)?|[.,]\d+)$/.test(trimmed)) return null;
  const number = Number(trimmed.replace(',', '.'));
  return Number.isFinite(number) && number >= min ? number : null;
}

function formatNumericInput(text, number) {
  const normalized = String(number);
  if (normalized.includes('e')) return text.trim().replace(/([.,]\d*?[1-9])0+$/, '$1');
  return normalized.replace('.', text.includes(',') ? ',' : '.');
}

if (typeof module !== 'undefined') module.exports = { parseNumericInput, formatNumericInput };
