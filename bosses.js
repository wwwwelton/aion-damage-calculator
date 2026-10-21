const bossSearch = document.querySelector('#boss-search');
const bossGroup = document.querySelector('#boss-group');
const opponentKind = document.querySelector('#opponent-kind');
const bossChoice = document.querySelector('#boss-choice');
const bossImport = document.querySelector('#import-boss');
const bossTypes = new Set(['Cogni', 'Fera', 'Natura', 'Varian']);
let visibleBosses = [];

function filterBosses() {
  const search = bossSearch.value.trim().toLocaleLowerCase();
  const catalog = opponentKind.value === 'boss' ? bossCatalog : mobCatalog;
  visibleBosses = catalog.filter(boss =>
    (opponentKind.value === 'mob' || !bossGroup.value || boss.group === bossGroup.value)
    && `${boss.name} ${boss.location ?? ''} ${boss.subtitle ?? ''}`.toLocaleLowerCase().includes(search));
  const options = [new Option('Select a monster', '')];
  for (const boss of visibleBosses) {
    options.push(new Option(`${boss.name} · Lv ${boss.level} · ${boss.type} · ${boss.group ?? boss.rank}`, boss.id));
  }
  bossChoice.replaceChildren(...options);
  document.querySelector('#boss-count').textContent = `${visibleBosses.length} of ${catalog.length} ${opponentKind.value === 'boss' ? 'bosses' : 'mobs'} shown.`;
  showBoss();
}

function showBoss() {
  const boss = visibleBosses.find(item => item.id === bossChoice.value);
  bossImport.disabled = !boss;
  document.querySelector('#boss-title').textContent = boss?.name ?? 'Select a monster';
  setDisplayIcon(document.querySelector('#boss-icon'), opponentKind.value === 'boss' ? 'boss' : 'mob');
  document.querySelector('#boss-details').textContent = boss
    ? `Level ${boss.level} · ${boss.rank} · ${boss.type} · ${boss.group ?? 'Mob'} · NPC ${boss.id}.${boss.location ? ` Location: ${boss.location}.` : ''}${boss.subtitle ? ` Classification: ${boss.subtitle}.` : ''} Defense and damage tolerances: not published in the source.`
    : '';
  const source = document.querySelector('#boss-source');
  source.hidden = !boss;
  if (boss) source.textContent = `Source: https://aion2data.com/monsters/${boss.id}-${boss.slug}`;
}

opponentKind.addEventListener('change', () => {
  document.querySelector('#boss-group-label').hidden = opponentKind.value === 'mob';
  filterBosses();
});
bossSearch.addEventListener('input', filterBosses);
bossGroup.addEventListener('change', filterBosses);
bossChoice.addEventListener('change', showBoss);
bossImport.addEventListener('click', () => {
  const boss = visibleBosses.find(item => item.id === bossChoice.value);
  if (!boss) return;
  const calculator = document.querySelector('#calculator');
  for (const input of calculator.querySelectorAll('.enemy input')) input.value = '0';
  document.querySelector('#target-mode').value = opponentKind.value === 'boss' ? 'boss' : 'monster';
  document.querySelector('#target-mode').dispatchEvent(new Event('change'));
  const type = bossTypes.has(boss.type) ? boss.type.toLowerCase() : 'special';
  document.querySelector('#creature-type').value = type === 'special' ? '' : type;
  document.querySelector('#creature-type-header').value = type;
  document.querySelector('#target-name').value = boss.name;
  calculator.dispatchEvent(new Event('input', { bubbles: true }));
  const warning = document.querySelector('#import-warning');
  warning.hidden = false;
  warning.textContent = `${boss.name} (Lv ${boss.level}, ${boss.type}, ${opponentKind.value}) selected. Previous enemy defenses cleared; target Defense and tolerances remain unknown (0 until entered manually). ${bossTypes.has(boss.type) ? `${boss.type} creature bonuses now replace generic Species values.` : 'Special has no creature-specific matchup; generic Species values remain manual.'}`;
  showTab('calculator-view');
  document.querySelector('.encounter').scrollIntoView({ block: 'start' });
});
filterBosses();
