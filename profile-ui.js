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
    if (entry.extra && Number(entry.extra) !== 0) row.append(dossierNode('small', '', `+${entry.extra} bonus`));
    list.append(row);
  }
  return list;
}

function gearCard({ item, detail }) {
  const card = dossierNode('article', 'gear-card');
  const head = dossierNode('div', 'gear-card-head');
  head.append(profileImage(item.icon || detail.icon, 'stat'));
  const title = dossierNode('div');
  title.append(dossierNode('strong', '', item.name || detail.name || 'Unknown item'),
    dossierNode('small', '', `${item.slotPosName || 'Equipment'} · ${item.grade || detail.grade || 'Grade unknown'}${item.enchantLevel ? ` · +${item.enchantLevel}` : ''}${item.exceedLevel ? ` · exceed ${item.exceedLevel}` : ''}`));
  head.append(title);
  card.append(head);
  for (const [label, entries] of [['Base & bonus options', detail.mainStats], ['Soul Bind', detail.subStats],
    ['Manastones', detail.magicStoneStat], ['Theostones', detail.godStoneStat]]) {
    if (!entries?.length) continue;
    const block = dossierNode('div', 'gear-options');
    block.append(dossierNode('h4', '', label), statList(entries));
    card.append(block);
  }
  if (detail.subSkills?.length) {
    const block = dossierNode('div', 'gear-options');
    block.append(dossierNode('h4', '', 'Soul Bind skills'));
    const skills = dossierNode('ul', 'option-list');
    for (const skill of detail.subSkills) {
      const row = dossierNode('li');
      row.append(profileImage(skill.icon, 'skill'), dossierNode('span', '', skill.name || 'Skill'),
        dossierNode('strong', '', `+${skill.level || 0}`));
      skills.append(row);
    }
    block.append(skills);
    card.append(block);
  }
  if (card.children.length === 1) card.append(dossierNode('p', 'reference-note', 'Detailed options not available in this profile.'));
  return card;
}

function skillCard(skill) {
  const card = dossierNode('article', 'skill-card');
  card.append(profileImage(skill.icon, 'skill'));
  const content = dossierNode('div');
  content.append(dossierNode('strong', '', skill.name),
    dossierNode('small', '', `Level ${skill.skillLevel}${Number(skill.equip) ? ' · Equipped' : ''}`));
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
    dossierNode('small', '', `Level ${view.level || '—'} · ${view.race || 'Race unknown'} · ${view.server || 'Server unknown'}`),
    dossierNode('small', '', `Combat Power ${view.combatPower || '—'}${view.legion ? ` · Legion ${view.legion}` : ''}`));
  summary.append(summaryText);
  const tabs = dossierNode('div', 'dossier-tabs');
  tabs.setAttribute('role', 'tablist');
  tabs.setAttribute('aria-label', 'Character details');
  const sections = [];
  for (const [label, render] of [
    ['Equipment', section => {
      section.append(dossierHeader('Equipped items', `${view.gear.length} pieces · explicit options only`));
      const grid = dossierNode('div', 'gear-grid');
      grid.append(...view.gear.map(gearCard));
      section.append(grid);
    }],
    ['Skills', section => {
      for (const [category, name] of [['Active', 'Active skills'], ['Passive', 'Passive skills'], ['Dp', 'Stigma skills']]) {
        const skills = view.skills.filter(skill => skill.category === category);
        section.append(dossierHeader(name, `${skills.length} learned`));
        const grid = dossierNode('div', 'skill-grid');
        grid.append(...skills.map(skillCard));
        section.append(grid);
      }
    }],
    ['Attributes', section => {
      section.append(dossierHeader('Character-sheet attributes', 'No equipment totals inferred'));
      const grid = dossierNode('div', 'attribute-grid');
      for (const stat of view.stats) {
        const card = dossierNode('article', 'attribute-card');
        card.append(iconElement(attributeIcon(stat.name)));
        const text = dossierNode('div');
        text.append(dossierNode('span', '', stat.name), dossierNode('strong', '', stat.value));
        for (const effect of stat.statSecondList || []) text.append(dossierNode('small', '', effect));
        card.append(text);
        grid.append(card);
      }
      section.append(grid);
    }],
    ['Progression', section => {
      section.append(dossierHeader('Equipped titles', `${view.titles.length} published`));
      const titleGrid = dossierNode('div', 'progression-grid');
      for (const title of view.titles) {
        const card = dossierNode('article', 'progression-card');
        card.append(dossierNode('strong', '', title.name), dossierNode('small', '', title.equipCategory || 'Title'));
        for (const stat of title.equipStatList || []) card.append(dossierNode('p', '', stat.desc));
        titleGrid.append(card);
      }
      section.append(titleGrid);
      section.append(dossierHeader('Daevanion boards', `${view.boards.length} published`));
      const boards = dossierNode('div', 'progression-grid');
      for (const board of view.boards) {
        const card = dossierNode('article', 'progression-card');
        card.append(profileImage(board.icon, 'stat'), dossierNode('strong', '', board.name || 'Board'),
          dossierNode('small', '', `${board.openNodeCount ?? 0} / ${board.totalNodeCount ?? '?'} nodes open`));
        boards.append(card);
      }
      section.append(boards);
      section.append(dossierHeader('Arcana', `${view.arcana.length} equipped`));
      const arcana = dossierNode('div', 'gear-grid');
      arcana.append(...view.arcana.map(gearCard));
      section.append(arcana);
      const companions = dossierNode('div', 'progression-grid');
      for (const [label, value] of [['Mount', view.pet], ['Wings', view.wing]]) {
        if (!value?.name) continue;
        const card = dossierNode('article', 'progression-card');
        card.append(profileImage(value.icon, 'stat'), dossierNode('strong', '', value.name),
          dossierNode('small', '', label));
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
