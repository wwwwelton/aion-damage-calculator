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
  const options = [new Option(I18N.t('selectMonster'), '')];
  for (const boss of visibleBosses) {
    options.push(new Option(`${boss.name} · Lv ${boss.level} · ${boss.type} · ${boss.group ?? boss.rank}`, boss.id));
  }
  bossChoice.replaceChildren(...options);
  document.querySelector('#boss-count').textContent = I18N.format('catalogCount', {
    shown: visibleBosses.length,
    total: catalog.length,
    kind: I18N.t(opponentKind.value === 'boss' ? 'bosses' : 'mobs')
  });
  showBoss();
}

function showBoss() {
  const boss = visibleBosses.find(item => item.id === bossChoice.value);
  bossImport.disabled = !boss;
  document.querySelector('#boss-title').textContent = boss?.name ?? I18N.t('selectMonster');
  const iconHolder = document.querySelector('#boss-icon');
  if (boss?.portrait) {
    iconHolder.replaceChildren(profileImage(`https://aion2data.com${boss.portrait}`, opponentKind.value === 'boss' ? 'boss' : 'mob'));
    iconHolder.hidden = false;
  } else {
    setDisplayIcon(iconHolder, opponentKind.value === 'boss' ? 'boss' : 'mob');
  }
  const facts = document.querySelector('#boss-facts');
  facts.hidden = !boss;
  facts.replaceChildren();
  if (boss) {
    const attack = boss.attack
      ? I18N.format('basicAttackSummary', {
        type: I18N.t(boss.attack.type === 'magic' ? 'attackMagic' : boss.attack.type === 'physical' ? 'attackPhysical' : 'attackUnknown'),
        reach: boss.attack.reach ?? '?', interval: boss.attack.interval ?? '?'
      })
      : null;
    for (const [label, value] of [[I18N.t('factLevel'), boss.level], [I18N.t('factRank'), boss.rank],
      [I18N.t('factCreatureType'), boss.type], [I18N.t('factEncounter'), boss.group ?? I18N.t('mobs')],
      [I18N.t('factHp'), I18N.t('unknownServerSide')], [I18N.t('factDefense'), I18N.t('unknownServerSide')],
      ...(boss.spawnPoints !== undefined ? [[I18N.t('factSpawnPoints'), boss.spawnPoints]] : []),
      ...(attack ? [[I18N.t('factBasicAttack'), attack]] : [])]) {
      const card = document.createElement('div');
      card.className = 'boss-fact';
      const caption = document.createElement('small');
      caption.textContent = label;
      const detail = document.createElement('strong');
      detail.textContent = value;
      card.append(caption, detail);
      facts.append(card);
    }
  }
  document.querySelector('#boss-details').textContent = boss
    ? `NPC ${boss.id}${boss.location ? ` · ${boss.location}` : ''}${boss.subtitle ? ` · ${boss.subtitle}` : ''}`
    : '';
  const skills = document.querySelector('#boss-skills');
  skills.hidden = !boss;
  skills.replaceChildren();
  if (boss) {
    const heading = document.createElement('h3');
    heading.textContent = I18N.format('bossSkillsTitle', { count: boss.skills?.length ?? 0 });
    skills.append(heading);
    if (boss.skills?.length) {
      const table = document.createElement('table');
      table.className = 'boss-skill-table';
      const head = table.createTHead().insertRow();
      for (const key of ['skillName', 'skillDamage', 'skillCast', 'skillCooldown', 'skillRange']) {
        const cell = document.createElement('th');
        cell.textContent = I18N.t(key);
        head.append(cell);
      }
      const body = table.createTBody();
      for (const skill of boss.skills) {
        const row = body.insertRow();
        const damage = I18N.t(skill.damage === 'Magic' ? 'attackMagic' : skill.damage === 'Physical' ? 'attackPhysical' : skill.damage);
        for (const value of [skill.name, damage, skill.cast, skill.cooldown, skill.range]) row.insertCell().textContent = value;
      }
      skills.append(table);
      const note = document.createElement('p');
      note.className = 'import-note';
      note.textContent = I18N.t('bossSkillsNote');
      skills.append(note);
    } else {
      const empty = document.createElement('p');
      empty.className = 'import-note';
      empty.textContent = I18N.t('noSkillData');
      skills.append(empty);
    }
  }
  const source = document.querySelector('#boss-source');
  source.hidden = !boss;
  if (boss) source.textContent = `${I18N.t('source')}: https://aion2data.com/monsters/${boss.id}-${boss.slug}`;
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
  warning.textContent = `${I18N.format('targetSelected', {
    name: boss.name, level: boss.level, type: boss.type, kind: opponentKind.value
  })} ${I18N.t(bossTypes.has(boss.type) ? 'creatureBonusesApplied' : 'specialCreatureManual')
    .replace('{type}', boss.type)}`;
  showTab('calculator-view');
  document.querySelector('.encounter').scrollIntoView({ block: 'start' });
});
filterBosses();
I18N.onLanguageChange(() => {
  const selected = bossChoice.value;
  filterBosses();
  bossChoice.value = selected;
  showBoss();
});
