const test = require('node:test');
const assert = require('node:assert/strict');
const { existsSync, readFileSync } = require('node:fs');

// The Rust host embeds exactly the files listed in src/assets.rs. This test fails when a script or
// stylesheet the page loads is missing from disk or from that embed list, so the executable would serve a 404.
function embeddedPatterns() {
  const source = readFileSync('src/assets.rs', 'utf8');
  return [...source.matchAll(/#\[include = "([^"]+)"\]/g)].map(match => match[1]);
}

function isEmbedded(name, patterns) {
  return patterns.some(pattern => {
    if (!pattern.includes('*')) return pattern === name;
    const expression = pattern.split('**/*').map(part => part.replace(/\*/g, '[^/]*').replace(/[.]/g, '\\.'))
      .join('.*');
    return new RegExp(`^${expression}$`).test(name);
  });
}

test('every script and stylesheet the page loads is embedded in the executable', () => {
  const patterns = embeddedPatterns();
  const html = readFileSync('index.html', 'utf8');
  const references = [...html.matchAll(/<(?:script|link)[^>]*(?:src|href)="([^"]+)"/g)]
    .map(match => match[1])
    .filter(name => !/^(https?:|#)/.test(name));
  const missing = references.filter(name => !existsSync(name) || !isEmbedded(name, patterns));
  assert.deepEqual(missing, []);
});

test('the native host bridge is embedded and mentioned by the pages that use it', () => {
  const patterns = embeddedPatterns();
  assert.ok(isEmbedded('host-bridge.js', patterns));
  assert.match(readFileSync('characters.js', 'utf8'), /window\.aion2Host/);
  assert.match(readFileSync('ai-tips.js', 'utf8'), /window\.aion2Host/);
});
