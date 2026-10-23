const attributeTree = document.querySelector('#attributes-view');
const treeInputs = document.querySelector('#calculator');
// `tip` keys carry a bespoke "why it matters" explanation; leaves without one
// use the stat catalog description. `profile` leaves read the imported
// character's defensive sheet instead of the calculator's enemy-side input.
const treeFields = {
  pureAttack: { icon: 'attack', tip: 'treeTipPureAttack', hint: 'treeHintPureAttack' },
  attackIncrease: { icon: 'attack', hint: 'treeHintAttackIncrease', percent: true },
  penetration: { icon: 'attack', tip: 'treeTipPenetration', hint: 'treeHintPenetration' },
  accuracy: { icon: 'chance', hint: 'treeHintAccuracy' },
  damageBoost: { icon: 'damage', tip: 'treeTipDamageBoost', hint: 'treeHintDamageBoost', percent: true },
  weaponDamageBoost: { icon: 'damage', hint: 'treeHintWeaponBoost', percent: true },
  criticalHit: { icon: 'chance', hint: 'treeHintCriticalHit' },
  criticalDamageBoost: { icon: 'damage', tip: 'treeTipCriticalDamageBoost', hint: 'treeHintCriticalDamageBoost', percent: true },
  defense: { icon: 'defense', tip: 'treeTipDefense', hint: 'treeHintDefense', profile: true },
  damageTolerance: { icon: 'defense', tip: 'treeTipDamageTolerance', hint: 'treeHintDamageTolerance', profile: true, percent: true },
  criticalHitResist: { icon: 'defense', hint: 'treeHintCriticalResist', profile: true },
  criticalDamageTolerance: { icon: 'defense', hint: 'treeHintCriticalTolerance', profile: true, percent: true },
  evasion: { icon: 'chance', tip: 'treeTipEvasion', hint: 'treeHintEvasion', profile: true },
  block: { icon: 'defense', hint: 'treeHintBlock', profile: true },
  playerHp: { icon: 'health', tip: 'treeTipPlayerHp', hint: 'treeHintPlayerHp' },
  endurance: { icon: 'health', hint: 'treeHintEndurance', profile: true, percent: true }
};
const treeFieldEntries = Object.entries(treeFields);
let treeFormatter = new Intl.NumberFormat(I18N.currentLang === 'pt' ? 'pt-BR' : 'en-US', { maximumFractionDigits: 2 });
let importedTreeCharacter = '';
let importedTreeClass = '';
let importedTreeDefenses = {};
let importedTreeStats = [];
let selectedProfileStat = null;
let selectedTreeStat = '';
const treeTip = document.querySelector('#tree-tip');
let activeTreeButton = null;
let treeTipPinned = false;

function closeTreeTip() {
  treeTip.hidden = true;
  activeTreeButton?.removeAttribute('aria-describedby');
  activeTreeButton = null;
  treeTipPinned = false;
}

function positionTreeTip(button) {
  const rect = button.getBoundingClientRect();
  const width = treeTip.offsetWidth;
  const height = treeTip.offsetHeight;
  const left = Math.max(12, Math.min(rect.left + rect.width / 2 - width / 2, window.innerWidth - width - 12));
  const above = rect.top - height - 10;
  const top = above >= 12 ? above : Math.min(rect.bottom + 10, window.innerHeight - height - 12);
  treeTip.style.left = `${left}px`;
  treeTip.style.top = `${Math.max(12, top)}px`;
}

function bindTreeTip(button, select) {
  const open = () => {
    activeTreeButton?.removeAttribute('aria-describedby');
    activeTreeButton = button;
    select();
    treeTip.hidden = false;
    button.setAttribute('aria-describedby', 'tree-tip');
    positionTreeTip(button);
  };
  button.addEventListener('mouseenter', () => { if (!treeTipPinned) open(); });
  button.addEventListener('focus', () => { if (!treeTipPinned) open(); });
  button.addEventListener('mouseleave', () => { if (!treeTipPinned && activeTreeButton === button) closeTreeTip(); });
  button.addEventListener('blur', () => { if (!treeTipPinned && activeTreeButton === button) closeTreeTip(); });
  button.addEventListener('click', () => {
    if (treeTipPinned && activeTreeButton === button) closeTreeTip();
    else {
      open();
      treeTipPinned = true;
    }
  });
}

document.addEventListener('pointerdown', event => {
  if (activeTreeButton && !activeTreeButton.contains(event.target)) closeTreeTip();
});
document.addEventListener('keydown', event => {
  if (event.key === 'Escape') closeTreeTip();
});
function repositionTreeTip() {
  if (activeTreeButton) positionTreeTip(activeTreeButton);
}
window.addEventListener('scroll', repositionTreeTip, true);
window.addEventListener('resize', repositionTreeTip);

for (const node of attributeTree.querySelectorAll('.tree-node')) {
  const stat = node.parentElement.dataset.treeStat;
  const circle = document.createElement(stat ? 'button' : 'span');
  circle.className = 'tree-circle';
  if (stat) {
    circle.type = 'button';
    circle.dataset.treeStat = stat;
  }
  circle.append(iconElement(stat ? treeFields[stat].icon : node.dataset.treeIcon));
  node.before(circle);
  node.classList.remove('tree-node');
  node.classList.add('tree-caption');
  const hint = document.createElement('small');
  hint.className = 'tree-hint';
  hint.dataset.hintKey = stat ? treeFields[stat].hint : node.dataset.treeHint;
  if (stat) {
    const value = document.createElement('span');
    value.className = 'tree-value';
    node.after(value, hint);
    bindTreeTip(circle, () => {
      selectedTreeStat = stat;
      selectedProfileStat = null;
      updateTreeTip();
    });
  } else {
    node.after(hint);
  }
}
// Narrow screens scroll the trees horizontally; start centred on the root node.
document.querySelector('#attributes-tab').addEventListener('click', () => {
  for (const tree of attributeTree.querySelectorAll('.attribute-tree')) {
    tree.scrollLeft = (tree.scrollWidth - tree.clientWidth) / 2;
  }
});

function profileStatDescription(stat) {
  if (stat.type === 'STR') return I18N.t('treeTipMight');
  const key = stat.name.replace(/[^a-zA-Z0-9]+([a-zA-Z0-9])/g, (_, letter) => letter.toUpperCase())
    .replace(/^./, letter => letter.toLowerCase());
  return (getStatInfo(stat.name) || getStatInfo(key))?.desc || I18N.t('treeProfileNoMechanics');
}

function updateTreeTip() {
  if (selectedProfileStat) {
    document.querySelector('#tree-tip-title').textContent = selectedProfileStat.title ?? I18N.statLabel(selectedProfileStat.name);
    document.querySelector('#tree-tip-text').textContent = selectedProfileStat.description ?? profileStatDescription(selectedProfileStat);
    return;
  }
  if (!selectedTreeStat) return;
  const leaf = attributeTree.querySelector(`[data-tree-stat="${selectedTreeStat}"]`);
  document.querySelector('#tree-tip-title').textContent = leaf.querySelector('.tree-caption').textContent.trim();
  document.querySelector('#tree-tip-text').textContent = treeTipText(selectedTreeStat);
}
function treeTipText(stat) {
  const field = treeFields[stat];
  return field.tip ? I18N.t(field.tip) : getStatInfo(stat).desc;
}

function renderAttributeTrees() {
  const characterName = document.querySelector('#attacker-name').value;
  const characterClass = document.querySelector('#attacker-class').value;
  const sameProfile = importedTreeCharacter && characterName === importedTreeCharacter
    && characterClass === importedTreeClass;
  for (const [stat, field] of treeFieldEntries) {
    const leaf = attributeTree.querySelector(`li[data-tree-stat="${stat}"]`);
    const input = treeInputs.elements.namedItem(stat);
    const raw = field.profile ? (sameProfile ? importedTreeDefenses[stat] : undefined) : input?.value;
    const value = raw === undefined || raw === null || raw === '' ? null : Number(String(raw).replace(',', '.'));
    const available = value !== null && Number.isFinite(value) && (stat !== 'playerHp' || value > 0);
    const label = available ? `${treeFormatter.format(value)}${field.percent ? '%' : ''}` : I18N.t('treeUnknown');
    leaf.querySelector('.tree-value').textContent = label;
    const title = leaf.querySelector('.tree-caption').textContent.trim();
    const button = leaf.querySelector('button');
    button.setAttribute('aria-label', `${title}: ${label}. ${treeTipText(stat)}`);
  }
  for (const hint of attributeTree.querySelectorAll('.tree-hint')) hint.textContent = I18N.t(hint.dataset.hintKey);
  const note = document.querySelector('#tree-value-note');
  note.textContent = sameProfile
    ? I18N.format('treeImportedFrom', { name: importedTreeCharacter })
    : I18N.t('treeValueNote');
  updateTreeTip();
}

function renderProfileStats() {
  const sameProfile = importedTreeCharacter
    && document.querySelector('#attacker-name').value === importedTreeCharacter
    && document.querySelector('#attacker-class').value === importedTreeClass;
  const list = document.querySelector('#tree-profile-list');
  const stats = sameProfile ? importedTreeStats : [];
  document.querySelector('#tree-profile-empty').hidden = stats.length > 0;
  list.replaceChildren(...stats.map(stat => {
    const raw = String(stat.value).trim();
    const percent = raw.endsWith('%');
    const value = `${treeFormatter.format(Number(percent ? raw.slice(0, -1) : raw))}${percent ? '%' : ''}`;
    const item = document.createElement('li');
    item.append(statButton(I18N.statLabel(stat.name), value, profileStatDescription(stat), attributeIcon(stat.name), stat));
    return item;
  }));
  if (activeTreeButton && !activeTreeButton.isConnected) closeTreeTip();
  if (!stats.length) selectedProfileStat = null;
  updateTreeTip();
}

function statButton(label, value, description, icon, selection) {
  const button = document.createElement('button');
  button.type = 'button';
  button.setAttribute('aria-label', `${label}: ${value}. ${description}`);
  button.append(iconElement(icon));
  const name = document.createElement('span');
  name.textContent = label;
  const amount = document.createElement('strong');
  amount.textContent = value;
  button.append(name, amount);
  bindTreeTip(button, () => {
    selectedProfileStat = selection;
    selectedTreeStat = '';
    updateTreeTip();
  });
  return button;
}

const genusFields = ['DamageBoost', 'DamageTolerance', 'Attack', 'Defense', 'Accuracy', 'Evasion',
  'BlockPenetration', 'Block', 'CriticalHit', 'CriticalHitResist'];
const catalogGroups = [
  ['treeGroupHit', ['accuracy', 'evasion', 'blockPenetration', 'block', 'pveAccuracy', 'pvpAccuracy', 'pvpEvasion', 'pvpBlockPenetration', 'pvpBlock']],
  ['treeGroupAttack', ['pureAttack', 'attackBonus', 'attackIncrease', 'weaponDamageBoost', 'weaponDamageTolerance', 'powerShard', 'multiHit', 'pveAttackPower', 'bossAttackPower', 'speciesAttackPower', 'frontBackAttackPower', 'pvpAttack']],
  ['treeGroupSkill', ['skillCoefficient', 'skillAdditionalDamage', 'skillSpecialtyEnhancement', 'penetration', 'defense', 'pvpDefense']],
  ['treeGroupBoost', ['damageBoost', 'damageTolerance', 'pveDamageBoost', 'pveDamageTolerance', 'bossDamageBoost', 'bossDamageTolerance', 'speciesDamageBoost', 'speciesDamageTolerance', 'pvpDamageBoost', 'pvpDamageTolerance']],
  ['treeGroupCritical', ['criticalHit', 'criticalHitResist', 'criticalDamageBoost', 'criticalDamageTolerance', 'criticalAttack', 'criticalDamageDefense', 'pvpCriticalHit', 'pvpCriticalHitResist']],
  ['treeGroupProc', ['doubleChance', 'doubleResist', 'double', 'perfectChance', 'perfectResist', 'multiHitChance', 'multiHitResist', 'endurancePenetration', 'endurance', 'regenerationPenetration', 'regeneration']],
  ['treeGroupPosition', ['frontBoost', 'frontDamageTolerance', 'backBoost', 'backDamageTolerance', 'frontAttack', 'frontDefense', 'backAttack', 'backDefense', 'frontCriticalHit', 'frontCriticalHitResist', 'backCriticalHit', 'backCriticalHitResist']],
  ['treeGroupElemental', ['elementalResistance', 'fireBoost', 'fireTolerance', 'waterBoost', 'waterTolerance', 'windBoost', 'windTolerance', 'earthBoost', 'earthTolerance']],
  ['treeGroupGenus', ['cogni', 'fera', 'natura', 'varian'].flatMap(genus => genusFields.map(field => genus + field))],
  ['treeGroupHp', ['playerHp', 'enemyHp', 'hitsPerSecond']]
];

function catalogValue(name, percent) {
  const input = treeInputs.elements.namedItem(name);
  const number = parseNumericInput(input.value, input.min === '' ? -Infinity : Number(input.min));
  return number === null ? I18N.t('treeUnknown') : `${treeFormatter.format(number)}${percent ? '%' : ''}`;
}

const catalogValues = new Map();
// Built once per language; values are refreshed in place so open groups stay open.
function renderCatalog() {
  catalogValues.clear();
  const container = document.querySelector('#tree-catalog');
  container.replaceChildren(...catalogGroups.map(([titleKey, names]) => {
    const group = document.createElement('details');
    group.className = 'tree-catalog-group';
    group.open = titleKey === 'treeGroupHit';
    const summary = document.createElement('summary');
    summary.textContent = `${I18N.t(titleKey)} (${names.length})`;
    const list = document.createElement('ul');
    list.className = 'tree-profile-list';
    list.append(...names.map(name => {
      const info = getStatInfo(name);
      const label = treeInputs.elements.namedItem(name).closest('label');
      const percent = /\(%\)/.test(label.textContent) || /\(%\)/.test(info.tag);
      const title = `${label.classList.contains('enemy') ? `${I18N.t('treeCatalogEnemy')} · ` : ''}${info.title.replace(/\s*\(.*\)$/, '')}`;
      const item = document.createElement('li');
      const button = statButton(title, catalogValue(name, percent), info.desc, attributeIcon(name.replace(/([a-z])([A-Z])/g, '$1 $2')), { title, description: info.desc });
      catalogValues.set(name, { button, percent, title, description: info.desc });
      item.append(button);
      return item;
    }));
    group.append(summary, list);
    return group;
  }));
  if (activeTreeButton && !activeTreeButton.isConnected) closeTreeTip();
}

function updateCatalogValues() {
  for (const [name, { button, percent, title, description }] of catalogValues) {
    const value = catalogValue(name, percent);
    button.querySelector('strong').textContent = value;
    button.setAttribute('aria-label', `${title}: ${value}. ${description}`);
  }
}
treeInputs.addEventListener('input', renderAttributeTrees);
treeInputs.addEventListener('change', renderAttributeTrees);
treeInputs.addEventListener('input', updateCatalogValues);
treeInputs.addEventListener('change', updateCatalogValues);
for (const selector of ['#attacker-name', '#attacker-class']) {
  const input = document.querySelector(selector);
  input.addEventListener('input', renderAttributeTrees);
  input.addEventListener('input', renderProfileStats);
}
document.addEventListener('character-imported', ({ detail }) => {
  closeTreeTip();
  selectedProfileStat = null;
  selectedTreeStat = '';
  importedTreeCharacter = detail.name;
  importedTreeClass = document.querySelector('#attacker-class').value;
  importedTreeDefenses = detail.defenses;
  importedTreeStats = detail.stats;
  renderAttributeTrees();
  renderProfileStats();
});
I18N.onLanguageChange(() => {
  closeTreeTip();
  treeFormatter = new Intl.NumberFormat(I18N.currentLang === 'pt' ? 'pt-BR' : 'en-US', { maximumFractionDigits: 2 });
  renderAttributeTrees();
  renderProfileStats();
  renderCatalog();
});
renderAttributeTrees();
renderProfileStats();
renderCatalog();
