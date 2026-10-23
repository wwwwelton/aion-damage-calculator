const dossierText = key => I18N.t(key);
const dossierFormat = (key, values) => I18N.format(key, values);

function dossierNode(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = String(text);
  return node;
}

function dossierHeader(name, note) {
  const header = dossierNode('div', 'dossier-section-head');
  header.append(dossierNode('h3', '', name), dossierNode('small', '', note));
  return header;
}

function statList(entries, className = 'option-list') {
  const list = dossierNode('ul', className);
  for (const entry of entries || []) {
    if (!entry?.name) continue;
    const row = dossierNode('li');
    row.append(iconElement(attributeIcon(entry.name)), dossierNode('span', '', entry.name));
    const value = entry.value ?? entry.desc;
    if (value !== undefined && value !== null) row.append(dossierNode('strong', '', value));
    if (entry.extra && Number(entry.extra) !== 0) row.append(dossierNode('small', '', `+${entry.extra} ${dossierText('bonusSuffix')}`));
    list.append(row);
  }
  return list;
}

function gearCard({ item, detail }) {
  const card = dossierNode('article', 'gear-card');
  const head = dossierNode('div', 'gear-card-head');
  head.append(profileImage(item.icon || detail.icon, 'stat'));
  const title = dossierNode('div');
  title.append(dossierNode('strong', '', item.name || detail.name || dossierText('unknownItem')),
    dossierNode('small', '', `${item.slotPosName || dossierText('equipment')} · ${item.grade || detail.grade || dossierText('gradeUnknown')}${item.enchantLevel ? ` · +${item.enchantLevel}` : ''}${item.exceedLevel ? ` · exceed ${item.exceedLevel}` : ''}`));
  head.append(title);
  card.append(head);
  for (const [label, entries] of [[dossierText('baseOptions'), detail.mainStats],
    [dossierText('soulBind'), detail.subStats], [dossierText('manastones'), detail.magicStoneStat],
    [dossierText('theostones'), detail.godStoneStat]]) {
    if (!entries?.length) continue;
    const block = dossierNode('div', 'gear-options');
    block.append(dossierNode('h4', '', label), statList(entries));
    card.append(block);
  }
  if (detail.subSkills?.length) {
    const block = dossierNode('div', 'gear-options');
    block.append(dossierNode('h4', '', dossierText('soulSkills')));
    const skills = dossierNode('ul', 'option-list');
    for (const skill of detail.subSkills) {
      const row = dossierNode('li');
      row.append(profileImage(skill.icon, 'skill'), dossierNode('span', '', skill.name || dossierText('skill')),
        dossierNode('strong', '', `+${skill.level || 0}`));
      skills.append(row);
    }
    block.append(skills);
    card.append(block);
  }
  if (card.children.length === 1) card.append(dossierNode('p', 'reference-note', dossierText('detailsUnavailable')));
  return card;
}

function skillCard(skill) {
  const card = dossierNode('article', 'skill-card');
  card.append(profileImage(skill.icon, 'skill'));
  const content = dossierNode('div');
  content.append(dossierNode('strong', '', skill.name),
    dossierNode('small', '', `${dossierText('level')} ${skill.skillLevel}${Number(skill.equip) ? ` · ${dossierText('equipped')}` : ''}`));
  card.append(content);
  return card;
}

function renderDossier(container, view) {
  const summary = dossierNode('div', 'dossier-summary');
  const portrait = profileImage(view.portrait, 'class', view.className);
  portrait.classList.add('dossier-portrait');
  summary.append(portrait);
  const summaryText = dossierNode('div');
  summaryText.append(dossierNode('strong', '', `${view.name} · ${view.className}`),
    dossierNode('small', '', [`${dossierText('level')} ${view.level || '—'}`, view.race || dossierText('raceUnknown'),
      view.gender, view.server || dossierText('serverUnknown')].filter(Boolean).join(' · ')),
    dossierNode('small', '', [`${dossierText('combatPower')} ${view.combatPower || '—'}`,
      view.itemLevel ? `${dossierText('itemLevel')} ${view.itemLevel}` : '',
      view.legion ? `${dossierText('legion')} ${view.legion}` : ''].filter(Boolean).join(' · ')));
  if (view.titleName || view.titlesOwned !== null) {
    summaryText.append(dossierNode('small', '', [view.titleName ? `${dossierText('equippedTitle')} ${view.titleName}` : '',
      view.titlesOwned !== null ? dossierFormat('titlesOwned', { owned: view.titlesOwned, total: view.titlesTotal ?? '?' }) : '']
      .filter(Boolean).join(' · ')));
  }
  summary.append(summaryText);
  const tabs = dossierNode('div', 'dossier-tabs');
  tabs.setAttribute('role', 'tablist');
  tabs.setAttribute('aria-label', dossierText('characterDetails'));
  const sections = [];
  for (const [label, render] of [
    [dossierText('tabEquipment'), section => {
      section.append(dossierHeader(dossierText('equippedItems'),
        dossierFormat('piecesPublished', { count: view.gear.length })));
      const grid = dossierNode('div', 'gear-grid');
      grid.append(...view.gear.map(gearCard));
      section.append(grid);
    }],
    [dossierText('tabSkills'), section => {
      for (const [category, key] of [['Active', 'activeSkills'], ['Passive', 'passiveSkills'], ['Dp', 'stigmaSkills']]) {
        const skills = view.skills.filter(skill => skill.category === category);
        section.append(dossierHeader(dossierText(key), dossierFormat('learnedCount', { count: skills.length })));
        const grid = dossierNode('div', 'skill-grid');
        grid.append(...skills.map(skillCard));
        section.append(grid);
      }
    }],
    [dossierText('tabAttributes'), section => {
      section.append(dossierHeader(dossierText('sheetAttributes'), dossierText('noInferredTotals')));
      const grid = dossierNode('div', 'attribute-grid');
      for (const stat of view.stats) {
        const card = dossierNode('article', 'attribute-card');
        card.append(iconElement(attributeIcon(stat.name)));
        const text = dossierNode('div');
        text.append(dossierNode('span', '', stat.type === 'ItemLevel' ? dossierText('itemLevel') : stat.name), dossierNode('strong', '', stat.value));
        for (const effect of stat.statSecondList || []) text.append(dossierNode('small', '', effect));
        card.append(text);
        grid.append(card);
      }
      section.append(grid);
    }],
    [dossierText('tabProgression'), section => {
      section.append(dossierHeader(dossierText('equippedTitles'),
        dossierFormat('publishedCount', { count: view.titles.length })));
      const titleGrid = dossierNode('div', 'progression-grid');
      for (const title of view.titles) {
        const card = dossierNode('article', 'progression-card');
        card.append(dossierNode('strong', '', title.name),
          dossierNode('small', '', [title.equipCategory || dossierText('title'), title.grade].filter(Boolean).join(' · ')));
        for (const stat of title.equipStatList || []) card.append(dossierNode('p', '', stat.desc));
        for (const stat of title.statList || []) card.append(dossierNode('p', 'title-owned-bonus', `${dossierText('titleOwnedBonus')}: ${stat.desc}`));
        titleGrid.append(card);
      }
      section.append(titleGrid);
      section.append(dossierHeader(dossierText('daevanionBoards'),
        dossierFormat('publishedCount', { count: view.boards.length })));
      const boards = dossierNode('div', 'progression-grid');
      for (const board of view.boards) {
        const card = dossierNode('article', 'progression-card');
        card.append(profileImage(board.icon, 'stat'), dossierNode('strong', '', board.name || dossierText('board')),
          dossierNode('small', '', dossierFormat('nodesOpen', {
            open: board.openNodeCount ?? 0, total: board.totalNodeCount ?? '?'
          })));
        boards.append(card);
      }
      section.append(boards);
      section.append(dossierHeader(dossierText('arcana'),
        dossierFormat('equippedCount', { count: view.arcana.length })));
      const arcana = dossierNode('div', 'gear-grid');
      arcana.append(...view.arcana.map(gearCard));
      section.append(arcana);
      const companions = dossierNode('div', 'progression-grid');
      for (const [label, value, detail] of [
        [dossierText('mount'), view.pet, view.pet?.level ? `${dossierText('level')} ${view.pet.level}` : ''],
        [dossierText('wings'), view.wing, [view.wing?.grade, view.wing?.enchantLevel ? `+${view.wing.enchantLevel}` : ''].filter(Boolean).join(' · ')]
      ]) {
        if (!value?.name) continue;
        const card = dossierNode('article', 'progression-card');
        card.append(profileImage(value.icon, 'stat'), dossierNode('strong', '', value.name),
          dossierNode('small', '', detail ? `${label} · ${detail}` : label));
        companions.append(card);
      }
      section.append(companions);
    }]
  ]) {
    const button = dossierNode('button', 'dossier-tab', label);
    button.type = 'button';
    button.setAttribute('role', 'tab');
    button.setAttribute('aria-selected', String(sections.length === 0));
    const section = dossierNode('section', 'dossier-pane');
    section.setAttribute('role', 'tabpanel');
    section.hidden = sections.length !== 0;
    render(section);
    button.addEventListener('click', () => {
      for (const [otherButton, otherSection] of sections) {
        otherButton.setAttribute('aria-selected', String(otherButton === button));
        otherSection.hidden = otherButton !== button;
      }
    });
    tabs.append(button);
    sections.push([button, section]);
  }
  container.replaceChildren(summary, tabs, ...sections.map(([, section]) => section));
}
