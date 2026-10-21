const bossSearch = document.querySelector('#boss-search');
const bossGroup = document.querySelector('#boss-group');
const bossChoice = document.querySelector('#boss-choice');
const bossImport = document.querySelector('#import-boss');
const bossTypes = new Set(['Cogni', 'Fera', 'Natura', 'Varian']);
let visibleBosses = [];

function filterBosses() {
  const search = bossSearch.value.trim().toLocaleLowerCase();
  visibleBosses = bossCatalog.filter(boss =>
    (!bossGroup.value || boss.group === bossGroup.value)
    && `${boss.name} ${boss.location} ${boss.subtitle}`.toLocaleLowerCase().includes(search));
  bossChoice.replaceChildren(new Option('Select a boss', ''));
  for (const boss of visibleBosses) {
    bossChoice.add(new Option(`${boss.name} · Lv ${boss.level} · ${boss.type} · ${boss.group}`, boss.id));
  }
  document.querySelector('#boss-count').textContent = `${visibleBosses.length} of ${bossCatalog.length} bosses shown.`;
  showBoss();
}

function showBoss() {
  const boss = visibleBosses.find(item => item.id === bossChoice.value);
  bossImport.disabled = !boss;
  document.querySelector('#boss-title').textContent = boss?.name ?? 'Select a boss';
  document.querySelector('#boss-details').textContent = boss
    ? `Level ${boss.level} · ${boss.rank} · ${boss.type} · ${boss.group} · NPC ${boss.id}.${boss.location ? ` Location: ${boss.location}.` : ''}${boss.subtitle ? ` Classification: ${boss.subtitle}.` : ''} Defense and damage tolerances: not published in the source.`
    : '';
  const source = document.querySelector('#boss-source');
  source.hidden = !boss;
  if (boss) source.textContent = `Source: https://aion2data.com/monsters/${boss.id}-${boss.slug}`;
}

bossSearch.addEventListener('input', filterBosses);
bossGroup.addEventListener('change', filterBosses);
bossChoice.addEventListener('change', showBoss);
bossImport.addEventListener('click', () => {
  const boss = visibleBosses.find(item => item.id === bossChoice.value);
  if (!boss) return;
  const calculator = document.querySelector('#calculator');
  for (const input of calculator.querySelectorAll('.enemy input')) input.value = '0';
  document.querySelector('#creature-type').value = bossTypes.has(boss.type) ? boss.type.toLowerCase() : '';
  calculator.dispatchEvent(new Event('input', { bubbles: true }));
  const warning = document.querySelector('#import-warning');
  warning.hidden = false;
  warning.textContent = `${boss.name} (Lv ${boss.level}, ${boss.type}) selected. Previous enemy defenses cleared; boss Defense and tolerances remain unknown (0 until entered manually). ${bossTypes.has(boss.type) ? `${boss.type} creature bonuses now replace generic Species values.` : 'Special has no creature-specific matchup; generic Species values remain manual.'}`;
  showTab('calculator-view');
  document.querySelector('#result').scrollIntoView({ block: 'start' });
});
filterBosses();
