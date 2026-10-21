const bonusFields = {
  'Attack Bonus': ['attackBonus', 0, false],
  'PvE Attack': ['pveAttackPower', 0, false],
  'PvE Attack Power': ['pveAttackPower', 0, false],
  'Boss Attack': ['bossAttackPower', 0, false],
  'Boss Attack Power': ['bossAttackPower', 0, false],
  'Weapon Damage Boost': ['weaponDamageBoost', 100, true],
  'Damage Boost': ['damageBoost', 100, true],
  'PvE Damage Boost': ['pveDamageBoost', 0, true],
  'Boss Damage Boost': ['bossDamageBoost', 0, true],
  'Critical Damage Boost': ['criticalDamageBoost', 150, true],
  'Front Attack Damage Boost': ['frontBoost', 100, true],
  'Back Attack Damage Boost': ['backBoost', 100, true]
};

function equippedDetails(profile) {
  const equipped = profile?.equipment?.equipment?.equipmentList || [];
  const details = profile?.itemDetails || {};
  return equipped.filter(item => !String(item.slotPosName).startsWith('Arcana'))
    .map(item => ({ item, detail: details[String(item.slotPos)] || {} }));
}

function equipmentBonuses(profile) {
  const sums = {};
  for (const { detail } of equippedDetails(profile)) {
    for (const option of [...(detail.mainStats || []), ...(detail.subStats || [])]) {
      const mapping = bonusFields[option.name];
      if (!mapping) continue;
      const [field, baseline, percentage] = mapping;
      const text = String(option.value ?? '').trim();
      if (!(percentage ? /^\d+(?:\.\d+)?%$/ : /^\d+(?:\.\d+)?$/).test(text)) continue;
      sums[field] = (sums[field] ?? 0) + Number.parseFloat(text);
    }
  }
  const values = {};
  for (const [name, amount] of Object.entries(sums)) {
    const baseline = Object.values(bonusFields).find(([field]) => field === name)[1];
    values[name] = Math.round((baseline + amount) * 10000) / 10000;
  }
  return values;
}

const parseHp = typeof module !== 'undefined' ? require('./numeric-input').parseNumericInput : parseNumericInput;
function characterHp(data) {
  const entry = data?.info?.stat?.statList?.find(stat => stat.name === 'HP');
  const hp = parseHp(String(entry?.value ?? ''), 0);
  return hp > 0 ? hp : null;
}

function extractCharacter(data) {
  const profile = data?.info?.profile || {};
  const gear = equippedDetails(data);
  return { name: profile.characterName || '', server: profile.serverName || '',
    className: profile.className || '', level: profile.characterLevel || '',
    combatPower: profile.combatPower || '', race: profile.raceName || '',
    legion: profile.regionName || '', portrait: profile.profileImage || '',
    stats: (data?.info?.stat?.statList || []).filter(stat => stat.name && Number.isFinite(Number(stat.value))),
    gear,
    arcana: (data?.equipment?.equipment?.equipmentList || [])
      .filter(item => String(item.slotPosName).startsWith('Arcana'))
      .map(item => ({ item, detail: data?.itemDetails?.[String(item.slotPos)] || {} })),
    skills: (data?.equipment?.skill?.skillList || [])
      .filter(skill => skill.name && Number(skill.acquired) && Number(skill.skillLevel) > 0),
    titles: (data?.info?.title?.titleList || []).filter(title => title.name),
    boards: data?.info?.daevanion?.boardList || [],
    pet: data?.equipment?.petwing?.pet || null,
    wing: data?.equipment?.petwing?.wing || null };
}

const referenceRows = (typeof module !== 'undefined' ? require('./matchups').referenceGroups : referenceGroups)
  .flatMap(group => group.rows);
const referenceFields = Object.fromEntries(referenceRows.map(([label, field]) => [label, field]));
referenceFields['PvE Accuracy'] = 'pveAccuracy';
const referencePercentFields = new Set(referenceRows.filter(row => row[4]).map(row => row[1]));

const enemyFields = {
  'Defense': 'defense',
  'Weapon Damage Tolerance': 'weaponDamageTolerance',
  'Damage Tolerance': 'damageTolerance',
  'PvE Damage Tolerance': 'pveDamageTolerance',
  'Species Damage Tolerance': 'speciesDamageTolerance',
  'Boss Damage Tolerance': 'bossDamageTolerance',
  'Critical Damage Tolerance': 'criticalDamageTolerance',
  'Front Attack Damage Tolerance': 'frontDamageTolerance',
  'Back Attack Damage Tolerance': 'backDamageTolerance',
  'Block': 'block',
  'Evasion': 'evasion',
  'Critical Hit Resist': 'criticalHitResist',
  'Double Resist': 'doubleResist',
  'Perfect Resist': 'perfectResist',
  'Multi-hit Resist': 'multiHitResist',
  'Endurance': 'endurance',
  'Elemental Resistance': 'elementalResistance',
  'Critical Damage Defense': 'criticalDamageDefense',
  'Front Defense': 'frontDefense',
  'Back Defense': 'backDefense'
};
Object.assign(enemyFields, Object.fromEntries(referenceRows.map(([, , label, field]) => [label, field])));
const enemyLabels = Object.fromEntries(Object.entries(enemyFields).map(([label, field]) => [field, label]));

const enemyPercentFields = new Set([
  'weaponDamageTolerance', 'damageTolerance', 'pveDamageTolerance', 'speciesDamageTolerance',
  'bossDamageTolerance', 'criticalDamageTolerance', 'frontDamageTolerance', 'backDamageTolerance',
  'doubleResist', 'perfectResist', 'multiHitResist', 'endurance'
]);
for (const [, , , field, percentage] of referenceRows) {
  if (percentage) enemyPercentFields.add(field);
}


function sheetFields(profile, mappings, percentFields) {
  const values = {};
  for (const stat of profile?.info?.stat?.statList || []) {
    const field = mappings[stat.name];
    if (!field) continue;
    const text = String(stat.value ?? '').trim();
    if (!(percentFields.has(field) ? /^\d+(?:\.\d+)?%?$/ : /^\d+(?:\.\d+)?$/).test(text)) continue;
    values[field] = Number.parseFloat(text);
  }
  return values;
}

function gearFields(profile, mappings, percentFields) {
  const values = {};
  for (const { detail } of equippedDetails(profile)) {
    for (const stat of [...(detail.mainStats || []), ...(detail.subStats || [])]) {
      const field = mappings[stat.name];
      if (!field) continue;
      const text = String(stat.value ?? '').trim();
      if (!(percentFields.has(field) ? /^\d+(?:\.\d+)?%$/ : /^\d+(?:\.\d+)?$/).test(text)) continue;
      values[field] = (values[field] ?? 0) + Number.parseFloat(text);
    }
  }
  return values;
}

function enemySheetDefenses(profile) {
  return sheetFields(profile, enemyFields, enemyPercentFields);
}

function enemyDefenses(profile) {
  return { ...gearFields(profile, enemyFields, enemyPercentFields), ...enemySheetDefenses(profile) };
}

function offensiveReferences(profile) {
  return { ...gearFields(profile, referenceFields, referencePercentFields),
    ...sheetFields(profile, referenceFields, referencePercentFields) };
}

if (typeof module !== 'undefined') module.exports = { extractCharacter, equipmentBonuses, enemyDefenses, offensiveReferences, characterHp };

if (typeof document !== 'undefined') {
  const tabs = document.querySelectorAll('.tab');
  function showTab(id) {
    for (const tab of tabs) {
      const active = tab.dataset.tab === id;
      tab.classList.toggle('active', active);
      tab.setAttribute('aria-selected', String(active));
      document.getElementById(tab.dataset.tab).hidden = !active;
    }
  }
  tabs.forEach(tab => tab.addEventListener('click', () => showTab(tab.dataset.tab)));

  const searchForm = document.querySelector('#character-search');
  const region = document.querySelector('#character-region');
  const globalRegion = document.querySelector('#global-region');
  const choiceLabel = document.querySelector('#character-choice-label');
  const choice = document.querySelector('#character-choice');
  const panel = document.querySelector('#profile-panel');
  const searchStatus = document.querySelector('#search-status');
  const profileStatus = document.querySelector('#profile-status');
  const importButton = document.querySelector('#import-character');
  const requests = new Map();
  let nextRequest = 0;
  let currentSearch = 0;
  let currentProfile = 0;
  let results = [];
  let loaded = null;
  let previousImport = {};
  let previousReferenceImport = {};

  if (window.chrome?.webview) {
    window.chrome.webview.addEventListener('message', event => {
      const { id, data, error } = event.data || {};
      const request = requests.get(id);
      if (!request) return;
      requests.delete(id);
      if (error) request.reject(new Error(error));
      else request.resolve(data);
    });
  }
  function request(kind, parameters, queryRegion = region.value, queryGlobalRegion = globalRegion.value) {
    if (!window.chrome?.webview) return Promise.reject(new Error('Character search requires the Windows app.'));
    const id = ++nextRequest;
    return new Promise((resolve, reject) => {
      requests.set(id, { resolve, reject });
      window.chrome.webview.postMessage({ id, kind, region: queryRegion,
        globalRegion: queryGlobalRegion, ...parameters });
    });
  }
  function resetSearch() {
    ++currentSearch;
    ++currentProfile;
    results = [];
    loaded = null;
    choiceLabel.hidden = true;
    panel.hidden = true;
    searchStatus.textContent = 'Search again after changing the region.';
  }
  region.addEventListener('change', () => {
    document.querySelector('#global-region-label').hidden = region.value !== 'ww';
    resetSearch();
  });
  globalRegion.addEventListener('change', resetSearch);
  searchForm.addEventListener('submit', async event => {
    event.preventDefault();
    const name = document.querySelector('#character-name').value.trim();
    if (!name) return;
    const searchId = ++currentSearch;
    ++currentProfile;
    results = [];
    loaded = null;
    choiceLabel.hidden = true;
    panel.hidden = true;
    importButton.disabled = true;
    searchStatus.textContent = 'Searching public NotMeter profiles…';
    try {
      const queryRegion = region.value;
      const queryGlobalRegion = globalRegion.value;
      const data = await request('search', { name });
      if (searchId !== currentSearch) return;
      results = Array.isArray(data.results) ? data.results.map(item => ({
        ...item, queryRegion, queryGlobalRegion
      })) : [];
      if (!results.length) {
        searchStatus.textContent = data.complete === false ? 'Search is still processing. Try again shortly.' : 'No characters found.';
        return;
      }
      searchStatus.textContent = `${results.length} matching character${results.length === 1 ? '' : 's'} found${data.complete === false ? ' (more may still appear)' : ''}.`;
      choice.replaceChildren(new Option('Select a character', ''));
      results.forEach((item, index) => choice.add(new Option(
        `${item.name} · ${item.serverName} · ${item.className} · CP ${item.combatPower ?? '—'}`, String(index))));
      if (results.length === 1) await loadCharacter(0);
      else choiceLabel.hidden = false;
    } catch (error) {
      if (searchId === currentSearch) searchStatus.textContent = `Search failed: ${error.message}`;
    }
  });
  choice.addEventListener('change', () => {
    if (choice.value !== '') void loadCharacter(Number(choice.value));
  });
  async function loadCharacter(index) {
    const item = results[index];
    if (!item) return;
    const profileId = ++currentProfile;
    panel.hidden = false;
    loaded = null;
    importButton.disabled = true;
    document.querySelector('#profile-title').textContent = `${item.name} · ${item.serverName}`;
    document.querySelector('#profile-meta').textContent = `${item.className} · Level ${item.level} · ${item.raceName || ''} · CP ${item.combatPower ?? '—'}`;
    setDisplayIcon(document.querySelector('#profile-class-icon'), 'class', item.className);
    profileStatus.textContent = 'Loading attributes, gear and skills…';
    document.querySelector('#profile-dossier').replaceChildren();
    try {
      const data = await request('profile', { serverId: item.serverId, characterId: item.characterId,
        region: item.queryRegion, globalRegion: item.queryGlobalRegion });
      if (profileId !== currentProfile) return;
      loaded = data;
      const view = extractCharacter(data);
      renderDossier(document.querySelector('#profile-dossier'), view);
      importButton.disabled = !Object.keys(equipmentBonuses(data)).length
        && !Object.keys(offensiveReferences(data)).length && characterHp(data) === null;
      profileStatus.textContent = data.complete === false
        ? 'Partial public profile: some equipment details may still be loading.'
        : 'Public profile loaded. Stats shown here may lag behind the game.';
    } catch (error) {
      if (profileId === currentProfile) profileStatus.textContent = `Profile failed: ${error.message}`;
    }
  }
  importButton.addEventListener('click', () => {
    if (!loaded) return;
    const bonuses = equipmentBonuses(loaded);
    const references = offensiveReferences(loaded);
    const form = document.querySelector('#calculator');
    for (const [field, previous] of Object.entries(previousImport)) {
      if (field in bonuses) continue;
      const input = form.elements.namedItem(field);
      if (input && Number(input.value) === previous) {
        input.value = String(Object.values(bonusFields).find(([name]) => name === field)[1]);
      }
    }
    for (const [field, previous] of Object.entries(previousReferenceImport)) {
      if (field in references) continue;
      const input = form.elements.namedItem(field);
      if (input && parseNumericInput(input.value) === previous) input.value = '0';
    }
    previousReferenceImport = {};
    for (const [field, value] of Object.entries(references)) {
      const input = form.elements.namedItem(field);
      if (input) {
        input.value = String(value);
        previousReferenceImport[field] = value;
      }
    }
    previousImport = {};
    for (const [field, value] of Object.entries(bonuses)) {
      const input = form.elements.namedItem(field);
      if (input) {
        input.value = String(value);
        previousImport[field] = value;
      }
    }
    const attacker = extractCharacter(loaded);
    document.querySelector('#attacker-name').value = attacker.name || '';
    document.querySelector('#attacker-class').value = document.querySelector('#profile-meta').textContent.split(' · ')[0];
    form.elements.namedItem('playerHp').value = String(characterHp(loaded) ?? 0);
    form.dispatchEvent(new Event('input', { bubbles: true }));
    profileStatus.textContent = `Loaded ${Object.keys(bonuses).length} formula bonuses and ${Object.keys(references).length} matchup stats. Equipment-only values are partial, not complete character totals.`;
    const warning = document.querySelector('#import-warning');
    warning.hidden = false;
    warning.textContent = `Imported ${Object.keys(bonuses).length} formula bonuses and ${Object.keys(references).length} offensive matchup stats for ${attacker.name}. ${characterHp(loaded) === null ? 'Character-sheet HP unavailable; Player HP is unknown.' : `Character-sheet HP: ${characterHp(loaded)}.`} Pure Attack and unavailable totals remain manual; reference-only matchups do not alter Final Damage.`;
    showTab('calculator-view');
    document.querySelector('.encounter').scrollIntoView({ block: 'start' });
  });
  const enemyRegion = document.querySelector('#enemy-region');
  const enemyGlobalRegion = document.querySelector('#enemy-global-region');
  const enemyChoice = document.querySelector('#enemy-choice');
  const enemyChoiceLabel = document.querySelector('#enemy-choice-label');
  const enemyPanel = document.querySelector('#enemy-panel');
  const enemySearchStatus = document.querySelector('#enemy-search-status');
  const enemyProfileStatus = document.querySelector('#enemy-profile-status');
  const enemyImportButton = document.querySelector('#import-enemy');
  let enemyResults = [];
  let enemyLoaded = null;
  let enemySearchId = 0;
  let enemyProfileId = 0;

  function resetEnemySearch() {
    ++enemySearchId;
    ++enemyProfileId;
    enemyResults = [];
    enemyLoaded = null;
    enemyChoiceLabel.hidden = true;
    enemyPanel.hidden = true;
    enemyImportButton.disabled = true;
    enemySearchStatus.textContent = 'Search again after changing the region.';
  }
  enemyRegion.addEventListener('change', () => {
    document.querySelector('#enemy-global-region-label').hidden = enemyRegion.value !== 'ww';
    resetEnemySearch();
  });
  enemyGlobalRegion.addEventListener('change', resetEnemySearch);
  document.querySelector('#enemy-search').addEventListener('submit', async event => {
    event.preventDefault();
    const name = document.querySelector('#enemy-name').value.trim();
    if (!name) return;
    const searchId = ++enemySearchId;
    ++enemyProfileId;
    enemyResults = [];
    enemyLoaded = null;
    enemyChoiceLabel.hidden = true;
    enemyPanel.hidden = true;
    enemyImportButton.disabled = true;
    enemySearchStatus.textContent = 'Searching public character profiles…';
    try {
      const queryRegion = enemyRegion.value;
      const queryGlobalRegion = enemyGlobalRegion.value;
      const data = await request('search', { name }, queryRegion, queryGlobalRegion);
      if (searchId !== enemySearchId) return;
      enemyResults = Array.isArray(data.results) ? data.results.map(item => ({
        ...item, queryRegion, queryGlobalRegion
      })) : [];
      if (!enemyResults.length) {
        enemySearchStatus.textContent = data.complete === false ? 'Search is still processing. Try again shortly.' : 'No characters found.';
        return;
      }
      enemySearchStatus.textContent = `${enemyResults.length} matching character${enemyResults.length === 1 ? '' : 's'} found.`;
      enemyChoice.replaceChildren(new Option('Select a character', ''));
      enemyResults.forEach((item, index) => enemyChoice.add(new Option(
        `${item.name} · ${item.serverName} · ${item.className} · CP ${item.combatPower ?? '—'}`, String(index))));
      if (enemyResults.length === 1) await loadEnemy(0);
      else enemyChoiceLabel.hidden = false;
    } catch (error) {
      if (searchId === enemySearchId) enemySearchStatus.textContent = `Search failed: ${error.message}`;
    }
  });
  enemyChoice.addEventListener('change', () => {
    if (enemyChoice.value !== '') void loadEnemy(Number(enemyChoice.value));
  });
  async function loadEnemy(index) {
    const item = enemyResults[index];
    if (!item) return;
    const profileId = ++enemyProfileId;
    enemyLoaded = null;
    enemyPanel.hidden = false;
    enemyImportButton.disabled = true;
    document.querySelector('#enemy-title').textContent = `${item.name} · ${item.serverName}`;
    document.querySelector('#enemy-meta').textContent = `${item.className} · Level ${item.level} · CP ${item.combatPower ?? '—'}`;
    setDisplayIcon(document.querySelector('#enemy-class-icon'), 'class', item.className);
    enemyProfileStatus.textContent = 'Loading defensive attributes…';
    document.querySelector('#enemy-dossier').replaceChildren();
    document.querySelector('#enemy-stats').replaceChildren();
    try {
      const data = await request('profile', { serverId: item.serverId, characterId: item.characterId },
        item.queryRegion, item.queryGlobalRegion);
      if (profileId !== enemyProfileId) return;
      enemyLoaded = data;
      renderDossier(document.querySelector('#enemy-dossier'), extractCharacter(data));
      const defenses = enemyDefenses(data);
      const sheet = enemySheetDefenses(data);
      const entries = Object.entries(defenses);
      const hp = characterHp(data);
      if (hp !== null) entries.push(['enemyHp', hp]);
      document.querySelector('#enemy-stats').replaceChildren(...(entries.length
        ? entries.map(([field, value]) => `${field === 'enemyHp' ? 'HP' : enemyLabels[field]}: ${value}${field === 'enemyHp' || field in sheet ? ' (character sheet)' : ' (equipment only; partial)'}`)
        : ['No matching defensive attributes in this public profile.']).map(text => {
        const row = document.createElement('li');
        row.append(iconElement(attributeIcon(text.split(':')[0])), document.createTextNode(text));
        return row;
      }));
      enemyImportButton.disabled = !entries.length;
      enemyProfileStatus.textContent = data.complete === false
        ? 'Partial profile: equipment-only values may be incomplete.'
        : 'Public profile loaded. Equipment-only values are partial; NPC and boss defenses differ.';
    } catch (error) {
      if (profileId === enemyProfileId) enemyProfileStatus.textContent = `Profile failed: ${error.message}`;
    }
  }
  enemyImportButton.addEventListener('click', () => {
    if (!enemyLoaded) return;
    const defenses = enemyDefenses(enemyLoaded);
    const calculator = document.querySelector('#calculator');
    document.querySelector('#target-mode').value = 'player';
    document.querySelector('#target-mode').dispatchEvent(new Event('change'));
    for (const input of calculator.querySelectorAll('.enemy input')) input.value = '0';
    for (const [field, value] of Object.entries(defenses)) {
      const input = calculator.elements.namedItem(field);
      if (input) input.value = String(value);
    }
    const hp = characterHp(enemyLoaded);
    if (hp !== null) calculator.elements.namedItem('enemyHp').value = String(hp);
    document.querySelector('#target-name').value = extractCharacter(enemyLoaded).name || '';
    document.querySelector('#target-class').value = document.querySelector('#enemy-meta').textContent.split(' · ')[0];
    calculator.dispatchEvent(new Event('input', { bubbles: true }));
    const warning = document.querySelector('#import-warning');
    warning.hidden = false;
    warning.textContent = `Imported ${Object.keys(defenses).length} available defenses from ${extractCharacter(enemyLoaded).name}. ${hp === null ? 'Enemy HP is unknown.' : `Enemy character-sheet HP: ${hp}.`} Equipment-only numbers are partial; missing values reset to zero (unknown). This character is not an NPC or boss profile.`;
    showTab('calculator-view');
    document.querySelector('.encounter').scrollIntoView({ block: 'start' });
  });
}
