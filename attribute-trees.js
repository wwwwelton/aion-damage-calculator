const attributeTree = document.querySelector('#attributes-view');
const treeInputs = document.querySelector('#calculator');
const treeFields = {
  pureAttack: { icon: 'attack', tip: 'treeTipPureAttack' },
  penetration: { icon: 'attack', tip: 'treeTipPenetration' },
  damageBoost: { icon: 'damage', tip: 'treeTipDamageBoost', percent: true },
  criticalDamageBoost: { icon: 'damage', tip: 'treeTipCriticalDamageBoost', percent: true },
  defense: { icon: 'defense', tip: 'treeTipDefense', profile: true },
  damageTolerance: { icon: 'defense', tip: 'treeTipDamageTolerance', profile: true, percent: true },
  evasion: { icon: 'chance', tip: 'treeTipEvasion', profile: true },
  playerHp: { icon: 'health', tip: 'treeTipPlayerHp' }
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
  if (stat) {
    const value = document.createElement('span');
    value.className = 'tree-value';
    node.after(value);
    const showTip = () => {
      selectedTreeStat = stat;
      selectedProfileStat = null;
      updateTreeTip();
    };
    bindTreeTip(circle, showTip);
  }
}
function profileStatDescription(stat) {
  if (stat.type === 'STR') return I18N.t('treeTipMight');
  const key = stat.name.replace(/[^a-zA-Z0-9]+([a-zA-Z0-9])/g, (_, letter) => letter.toUpperCase())
    .replace(/^./, letter => letter.toLowerCase());
  return (getStatInfo(stat.name) || getStatInfo(key))?.desc || I18N.t('treeProfileNoMechanics');
}

function updateTreeTip() {
  if (selectedProfileStat) {
    document.querySelector('#tree-tip-title').textContent = I18N.statLabel(selectedProfileStat.name);
    document.querySelector('#tree-tip-text').textContent = profileStatDescription(selectedProfileStat);
    return;
  }
  if (!selectedTreeStat) return;
  const field = treeFields[selectedTreeStat];
  const leaf = attributeTree.querySelector(`[data-tree-stat="${selectedTreeStat}"]`);
  const title = leaf.querySelector('.tree-caption').textContent.trim();
  document.querySelector('#tree-tip-title').textContent = title;
  document.querySelector('#tree-tip-text').textContent = I18N.t(field.tip);
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
    button.setAttribute('aria-label', `${title}: ${label}. ${I18N.t(field.tip)}`);
  }
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
    const label = I18N.statLabel(stat.name);
    const raw = String(stat.value).trim();
    const percent = raw.endsWith('%');
    const value = `${treeFormatter.format(Number(percent ? raw.slice(0, -1) : raw))}${percent ? '%' : ''}`;
    const description = profileStatDescription(stat);
    const item = document.createElement('li');
    const button = document.createElement('button');
    button.type = 'button';
    button.setAttribute('aria-label', `${label}: ${value}. ${description}`);
    button.append(iconElement(attributeIcon(stat.name)));
    const name = document.createElement('span');
    name.textContent = label;
    const amount = document.createElement('strong');
    amount.textContent = value;
    button.append(name, amount);
    bindTreeTip(button, () => {
      selectedProfileStat = stat;
      selectedTreeStat = '';
      updateTreeTip();
    });
    item.append(button);
    return item;
  }));
  if (activeTreeButton && !activeTreeButton.isConnected) closeTreeTip();
  if (!stats.length) selectedProfileStat = null;
  updateTreeTip();
}
treeInputs.addEventListener('input', renderAttributeTrees);
treeInputs.addEventListener('change', renderAttributeTrees);
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
});
renderAttributeTrees();
renderProfileStats();
