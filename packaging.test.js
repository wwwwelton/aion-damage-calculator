const test = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync, readdirSync } = require('node:fs');

// Resolves the csproj EmbeddedResource globs the same way MSBuild names them
// (LogicalName with %(Filename)%(Extension)), so the test fails when a file the
// WebView2 host serves is no longer packaged into the executable.
function embeddedLogicalNames() {
  const csproj = readFileSync('Aion2Calculator.csproj', 'utf8');
  const names = new Set();
  for (const [, include, logical] of csproj.matchAll(/<EmbeddedResource Include="([^"]+)" LogicalName="([^"]+)"/g)) {
    if (!include.includes('*')) { names.add(logical); continue; }
    const dir = include.slice(0, include.lastIndexOf('/'));
    const pattern = new RegExp(`^${include.slice(dir.length + 1).replace(/\./g, '\\.').replace(/\*/g, '.*')}$`);
    for (const file of readdirSync(dir, { withFileTypes: true })) {
      if (!file.isFile() || !pattern.test(file.name)) continue;
      const dot = file.name.lastIndexOf('.');
      names.add(logical.replace('%(Filename)', dot > 0 ? file.name.slice(0, dot) : file.name)
        .replace('%(Extension)', dot > 0 ? file.name.slice(dot) : ''));
    }
  }
  return names;
}

test('every resource the Windows host serves is embedded in the executable', () => {
  const embedded = embeddedLogicalNames();
  const program = readFileSync('Program.cs', 'utf8');
  const served = [...program.matchAll(/\["([^"]+)"\] = "/g)].map(match => match[1]);
  const classIcons = [...program.matchAll(/"(gladiator|templar|assassin|ranger|sorcerer|spiritmaster|cleric|chanter)"/g)]
    .map(match => `assets/classes/${match[1]}.webp`);
  const html = readFileSync('index.html', 'utf8');
  const scripts = [...html.matchAll(/<script defer src="([^"]+)"/g)].map(match => match[1]);
  const missing = [...new Set([...served, ...classIcons, ...scripts, 'style.css'])].filter(name => !embedded.has(name));
  assert.deepEqual(missing, []);
});
