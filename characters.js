// Gear option ids are stable across client languages; the English names stay as a fallback for
// older payloads and fixtures. Only ids observed in the official API are listed.
const gearOptionNames = {
  WeaponFixingDamage: 'Attack', WeaponAccuracy: 'Accuracy', Critical: 'Critical Hit', Block: 'Block',
  ArmorDefense: 'Defense', ArmorEvasion: 'Evasion', CriticalResist: 'Critical Hit Resist',
  AmplifyAllDamage: 'Damage Boost', AmplifyWeaponDamage: 'Weapon Damage Boost',
  PvEAmplifyDamage: 'PvE Damage Boost', PvEDecreaseDamage: 'PvE Damage Tolerance',
  DefensePierce: 'Penetration', AdditionalHitRate: 'Multi-hit Chance', Perfect: 'Perfect Chance',
  CriticalAddDamage: 'Critical Attack', FrontAttackDefense: 'Front Defense', Restoration: 'Regeneration',
  HPMax: 'HP', MPMax: 'MP', SealStoneAddDamage: 'Power Shard Damage Bonus'
};
// Primary stats and divinities printed on gear are already included in the character-sheet totals,
// so they are never summed from equipment.
const primaryStatTypes = ['STR', 'DEX', 'CON', 'INT', 'AGI', 'WIS'];
const divinityTypes = ['Justice', 'Freedom', 'Illusion', 'Life', 'Time', 'Destruction', 'Death', 'Wisdom', 'Destiny', 'Space'];
const sheetOnlyOptionIds = new Set([...primaryStatTypes, ...divinityTypes]);

function optionName(option) {
  if (sheetOnlyOptionIds.has(option?.id)) return null;
  return gearOptionNames[option?.id] || option?.name;
}

const bonusFields = {
  'Attack Bonus': ['attackBonus', 0, false],
  'PvE Attack': ['pveAttackPower', 0, false],
  'PvE Attack Power': ['pveAttackPower', 0, false],
  'Boss Attack': ['bossAttackPower', 0, false],
  'Penetration': ['penetration', 0, false],
  'Boss Attack Power': ['bossAttackPower', 0, false],
  'Weapon Damage Boost': ['weaponDamageBoost', 100, true],
  'Damage Boost': ['damageBoost', 100, true],
  'PvE Damage Boost': ['pveDamageBoost', 0, true],
  'Boss Damage Boost': ['bossDamageBoost', 0, true],
  'Attack Increase': ['attackIncrease', 100, true],
  'Critical Damage Boost': ['criticalDamageBoost', 150, true],
  'Front Attack Damage Boost': ['frontBoost', 100, true],
  'Back Attack Damage Boost': ['backBoost', 100, true],
  'Skill Specialty Enhancement': ['skillSpecialtyEnhancement', 100, true],
};
// Sheet stats whose published "Attack increase" effect feeds Attack Increase (value / 10 = percent).
const attackIncreaseTypes = ['STR', 'Destruction'];

function equippedDetails(profile) {
  const equipped = profile?.equipment?.equipment?.equipmentList || [];
  const details = profile?.itemDetails || {};
  return equipped.filter(item => !String(item.slotPosName).startsWith('Arcana'))
    .map(item => ({ item, detail: details[String(item.slotPos)] || {} }));
}

function collectFormulaBonuses(profile) {
  const sums = {};
  const sources = {};
  for (const { detail } of equippedDetails(profile)) {
    for (const option of [...(detail.mainStats || []), ...(detail.subStats || [])]) {
      const mapping = bonusFields[optionName(option)];
      if (!mapping) continue;
      const [field, , percentage] = mapping;
      const text = String(option.value ?? '').trim();
      if (!(percentage ? /^\d+(?:\.\d+)?%$/ : /^\d+(?:\.\d+)?$/).test(text)) continue;
      sums[field] = (sums[field] ?? 0) + Number.parseFloat(text);
      sources[field] = 'gear';
    }
  }
  for (const stat of profile?.info?.stat?.statList || []) {
    if (!attackIncreaseTypes.includes(stat.type)) continue;
    const amount = Number(stat.value);
    if (!Number.isFinite(amount) || amount <= 0) continue;
    sums.attackIncrease = (sums.attackIncrease ?? 0) + amount / 10;
    sources.attackIncrease = sources.attackIncrease === 'gear' || sources.attackIncrease === 'sheet+gear' ? 'sheet+gear' : 'sheet';
  }
  const values = {};
  for (const [name, amount] of Object.entries(sums)) {
    const baseline = Object.values(bonusFields).find(([field]) => field === name)[1];
    values[name] = Math.round((baseline + amount) * 10000) / 10000;
  }
  return { values, sources };
}

function equipmentBonuses(profile) {
  return collectFormulaBonuses(profile).values;
}

const parseHp = typeof module !== 'undefined' ? require('./numeric-input').parseNumericInput : parseNumericInput;
function characterHp(data) {
  const entry = data?.info?.stat?.statList?.find(stat => stat.name === 'HP');
  const hp = parseHp(String(entry?.value ?? ''), 0);
  return hp > 0 ? hp : null;
}
const importableFormulaFields = [
  'pureAttack', 'weaponDamageBoost', 'multiHit', 'powerShard', 'attackBonus',
  'attackIncrease', 'pveAttackPower', 'bossAttackPower', 'frontBackAttackPower',
  'skillCoefficient', 'skillAdditionalDamage', 'penetration', 'skillSpecialtyEnhancement',
  'damageBoost', 'pveDamageBoost', 'bossDamageBoost', 'criticalDamageBoost',
  'frontBoost', 'backBoost', 'hitsPerSecond'
];

function formulaImportCoverage(profile) {
  const { values, sources } = collectFormulaBonuses(profile);
  const hp = characterHp(profile);
  if (hp !== null) {
    values.playerHp = hp;
    sources.playerHp = 'sheet';
  }
  const imported = Object.keys(values);
  return {
    imported,
    manual: importableFormulaFields.filter(field => !(field in values)),
    sources
  };
}

function extractCharacter(data) {
  const profile = data?.info?.profile || {};
  const gear = equippedDetails(data);
  const stats = data?.info?.stat?.statList || [];
  const combatPower = profile.combatPower ? Number(profile.combatPower) : null;
  const itemLevel = Number(stats.find(stat => stat.type === 'ItemLevel')?.value) || null;
  return { name: profile.characterName || '', server: profile.serverName || '',
    className: profile.className || '', level: profile.characterLevel || '',
    combatPower, race: profile.raceName || '',
    gender: profile.genderName || '', titleName: profile.titleName || '',
    itemLevel, gearScore: itemLevel,
    titlesOwned: data?.info?.title?.ownedCount ?? null, titlesTotal: data?.info?.title?.totalCount ?? null,
    legion: profile.regionName || '', portrait: profile.profileImage || '',
    ranking: data?.externalRanking || null,
    stats: stats.filter(stat => {
      const value = String(stat.value ?? '').trim();
      return stat.name && value !== ''
        && Number.isFinite(Number(value.endsWith('%') ? value.slice(0, -1) : value));
    }),
    gear,
    arcana: (data?.equipment?.equipment?.equipmentList || [])
      .filter(item => String(item.slotPosName).startsWith('Arcana'))
      .map(item => ({ item, detail: data?.itemDetails?.[String(item.slotPos)] || {} })),
    skills: (data?.equipment?.skill?.skillList || [])
      .filter(skill => skill.name),
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
      const field = mappings[optionName(stat)];
      if (!field) continue;
      const text = String(stat.value ?? '').trim();
      if (!(percentFields.has(field) ? /^\d+(?:\.\d+)?%$/ : /^\d+(?:\.\d+)?$/).test(text)) continue;
      values[field] = (values[field] ?? 0) + Number.parseFloat(text);
    }
  }
  return values;
}

// Published secondary effects of sheet stats, keyed by the language-independent stat type.
const typedOffensiveEffects = {
  Wisdom: 'doubleChance',
  Illusion: 'endurancePenetration',
  Death: 'regenerationPenetration',
  Justice: 'perfectChance'
};
const typedDefensiveEffects = {
  Time: 'doubleResist',
  Destruction: 'perfectResist',
  Destiny: 'endurance',
  Life: 'regeneration'
};
// Percent-of-base increases: the sheet publishes no base totals, so these are shown, never imported.
const sheetModifierEffects = {
  AGI: ['accuracyIncrease', 'criticalHitIncrease'],
  DEX: ['evasionIncrease', 'blockIncrease', 'criticalHitResistIncrease'],
  CON: ['hpIncrease'],
  Justice: ['defenseIncrease'],
  Freedom: ['accuracyIncrease', 'evasionIncrease'],
  Life: ['hpIncrease'],
  Death: ['criticalHitIncrease'],
  Space: ['blockIncrease']
};
// Effect labels (as printed in statSecondList) that map onto a calculator field.
const formulaEffectLabels = ['Attack increase', 'Double Chance', 'Double Resist', 'Perfect Chance', 'Perfect Resist',
  'Endurance Penetration', 'Endurance', 'Regeneration Penetration', 'Regeneration'];
function effectRole(text) {
  return formulaEffectLabels.some(label => String(text).startsWith(label)) ? 'formula' : 'display';
}
function statGroup(stat) {
  if (primaryStatTypes.includes(stat?.type)) return 'primary';
  if (divinityTypes.includes(stat?.type)) return 'divinity';
  return 'other';
}

function typedPercentEffects(profile, mappings) {
  const values = {};
  for (const stat of profile?.info?.stat?.statList || []) {
    const field = mappings[stat.type];
    const amount = Number(stat.value) / 10;
    if (field && Number.isFinite(amount) && amount > 0) values[field] = amount;
  }
  return values;
}

function sheetModifiers(profile) {
  const values = {};
  for (const stat of profile?.info?.stat?.statList || []) {
    const amount = Number(stat.value) / 10;
    if (!Number.isFinite(amount) || amount <= 0) continue;
    for (const field of sheetModifierEffects[stat.type] || []) {
      values[field] = Math.round(((values[field] ?? 0) + amount) * 10000) / 10000;
    }
  }
  return values;
}

function addPartialValues(...sources) {
  const values = {};
  for (const source of sources) {
    for (const [field, value] of Object.entries(source)) {
      values[field] = Math.round(((values[field] ?? 0) + value) * 10000) / 10000;
    }
  }
  return values;
}

function enemySheetDefenses(profile) {
  return { ...typedPercentEffects(profile, typedDefensiveEffects),
    ...sheetFields(profile, enemyFields, enemyPercentFields) };
}

function enemyDefenses(profile) {
  const directSheet = sheetFields(profile, enemyFields, enemyPercentFields);
  return { ...addPartialValues(
    gearFields(profile, enemyFields, enemyPercentFields),
    typedPercentEffects(profile, typedDefensiveEffects)
  ), ...directSheet };
}

function offensiveReferences(profile) {
  const directSheet = sheetFields(profile, referenceFields, referencePercentFields);
  return { ...addPartialValues(
    gearFields(profile, referenceFields, referencePercentFields),
    typedPercentEffects(profile, typedOffensiveEffects)
  ), ...directSheet };
}

if (typeof module !== 'undefined') module.exports = { extractCharacter, equipmentBonuses, enemyDefenses,
  offensiveReferences, characterHp, formulaImportCoverage, sheetModifiers, effectRole, statGroup, optionName };

if (typeof document !== 'undefined') {
  const ui = key => I18N.t(key);
  const formatUi = (key, values) => I18N.format(key, values);
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
  let loadedItem = null;
  let savedProfiles = [];
  let favorites = [];
  try {
    const stored = JSON.parse(localStorage.getItem('aion2Favorites') || '[]');
    if (Array.isArray(stored)) favorites = stored.filter(key => typeof key === 'string').slice(0, 20);
  } catch { /* Invalid preference data must not prevent profile loading. */ }
  const profileKey = item => `${item.queryGlobalRegion}|${item.serverId}|${item.characterId}`;
  let recentSearches = [];
  try {
    const stored = JSON.parse(localStorage.getItem('aion2RecentSearches') || '[]');
    if (Array.isArray(stored)) recentSearches = stored.filter(value => typeof value === 'string' && value.length <= 20).slice(0, 10);
  } catch { /* A broken history must not block searching. */ }
  const recentNames = document.createElement('datalist');
  recentNames.id = 'recent-character-searches';
  document.querySelector('#character-name').setAttribute('list', recentNames.id);
  searchForm.append(recentNames);
  function renderRecentSearches() {
    recentNames.replaceChildren(...recentSearches.map(name => new Option(name, name)));
  }
  renderRecentSearches();
  const profileDate = value => new Date(Number(value) * 1000).toLocaleString(I18N.currentLang === 'pt' ? 'pt-BR' : 'en-US');
  function updateProfileStatus(data) {
    const fetchedAt = data?.cacheMeta?.fetchedAt;
    profileStatus.textContent = `${ui(data.complete === false ? 'partialProfile' : 'profileLoaded')}${fetchedAt ? ` · ${formatUi(data.cacheMeta.cached ? 'cachedProfileDate' : 'profileDate', { date: profileDate(fetchedAt) })}` : ''}`;
  }
  function showProfile(data) {
    loaded = data;
    const view = extractCharacter(data);
    document.querySelector('#profile-title').textContent = `${view.name} · ${view.server}`;
    const cpDisplay = view.combatPower ? `CP ${view.combatPower.toLocaleString()}` : '';
    const gsDisplay = view.gearScore ? `GS ${view.gearScore.toLocaleString()}` : '';
    const scores = [cpDisplay, gsDisplay].filter(Boolean).join(' · ');
    document.querySelector('#profile-meta').textContent = `${view.className} · ${ui('level')} ${view.level} · ${view.race}${scores ? ` · ${scores}` : ''}`;
    setDisplayIcon(document.querySelector('#profile-class-icon'), 'class', view.className);
    renderDossier(document.querySelector('#profile-dossier'), view);
    importButton.disabled = !Object.keys(equipmentBonuses(data)).length
      && !Object.keys(offensiveReferences(data)).length && characterHp(data) === null;
    updateProfileStatus(data);
  }
  function renderSavedProfiles() {
    const container = document.querySelector('#saved-profiles');
    const entries = [...savedProfiles].sort((a, b) => {
      const key = entry => `${entry.globalRegion}|${entry.serverId}|${entry.characterId}`;
      return Number(favorites.includes(key(b))) - Number(favorites.includes(key(a))) || Number(b.fetchedAt) - Number(a.fetchedAt);
    });
    container.replaceChildren(...entries.map(entry => {
      const view = extractCharacter(entry.profile);
      const item = { serverId: entry.serverId, characterId: entry.characterId, name: view.name,
        serverName: view.server, className: view.className, level: view.level, raceName: view.race,
        queryRegion: 'ww', queryGlobalRegion: entry.globalRegion };
      const button = document.createElement('button');
      button.type = 'button';
      button.textContent = `${favorites.includes(profileKey(item)) ? '★ ' : ''}${view.name} · ${view.server} · ${profileDate(entry.fetchedAt)}`;
      button.addEventListener('click', () => { results = [item]; void loadCharacter(0); });
      return button;
    }));
  }
  async function refreshSavedProfiles() {
    if (!window.aion2Host) return;
    try {
      const entries = await request('saved_profiles', {});
      savedProfiles = Array.isArray(entries) ? entries : [];
      renderSavedProfiles();
    } catch (error) { searchStatus.textContent = error.message; }
  }
  document.querySelector('#favorite-profile').addEventListener('click', () => {
    if (!loadedItem) return;
    const key = profileKey(loadedItem);
    const next = favorites.includes(key) ? favorites.filter(value => value !== key) : [...favorites, key].slice(-20);
    try {
      localStorage.setItem('aion2Favorites', JSON.stringify(next));
      favorites = next;
      document.querySelector('#favorite-profile').setAttribute('aria-pressed', String(favorites.includes(key)));
      renderSavedProfiles();
    } catch (error) { profileStatus.textContent = formatUi('profileStorageFailed', { error: error.message }); }
  });
  document.querySelector('#refresh-profile').addEventListener('click', () => {
    if (!loadedItem) return;
    results = [loadedItem];
    void loadCharacter(0, true);
  });
  document.addEventListener('click', event => {
    const btn = event.target.closest('.legion-link-btn');
    if (!btn) return;
    const legionName = btn.dataset.legionName;
    const serverName = btn.dataset.serverName;
    openLegionDialog(legionName, serverName);
  });

  async function openLegionDialog(legionName, serverName) {
    const dialog = document.querySelector('#legion-dialog');
    const title = document.querySelector('#legion-dialog-title');
    const summary = document.querySelector('#legion-dialog-summary');
    const list = document.querySelector('#legion-members-list');
    if (!dialog || !list) return;
    title.textContent = `Legion: ${legionName}`;
    summary.textContent = `Consultando integrantes da legião no servidor ${serverName || 'ativo'}…`;
    list.replaceChildren();
    dialog.showModal();

    // Try live Atool legion roster via native host request first
    try {
      const serverId = loadedItem?.serverId || 1401;
      const data = await request('legion_roster', { serverName: serverName || 'Siel', legionName, serverId });
      if (data && data.members && data.members.length > 0) {
        summary.textContent = `Membros registrados: ${data.total_members} · Média CP: ${Math.round(Number(data.avg_combat_power2 || 0)).toLocaleString()} · Média GS: ${Math.round(Number(data.avg_combat_power || 0)).toLocaleString()} · Rank do Servidor: #${data.server_rank || '—'}`;
        list.replaceChildren(...data.members.map(m => {
          const card = document.createElement('article');
          card.className = 'progression-card';
          const strong = document.createElement('strong');
          strong.textContent = `${m.nickname} · ${m.job || ''}`;
          const small = document.createElement('small');
          const cpVal = m.combat_power2 ? `CP ${Number(m.combat_power2).toLocaleString()}` : '';
          const gsVal = m.combat_power ? `GS ${Number(m.combat_power).toLocaleString()}` : '';
          small.textContent = [m.race, cpVal, gsVal, m.server].filter(Boolean).join(' · ');
          card.append(strong, small);
          return card;
        }));
        return;
      }
    } catch (err) {
      // Fallback to local saved members if network/Atool fails
    }

    const savedMembers = savedProfiles.filter(entry => {
      const v = extractCharacter(entry.profile);
      return v.legion && v.legion.toLowerCase() === legionName.toLowerCase();
    });
    summary.textContent = `Membros registrados localmente (${savedMembers.length}) no servidor ${serverName || 'ativo'}.`;
    if (savedMembers.length > 0) {
      list.replaceChildren(...savedMembers.map(entry => {
        const v = extractCharacter(entry.profile);
        const card = document.createElement('article');
        card.className = 'progression-card';
        const strong = document.createElement('strong');
        strong.textContent = `${v.name} · ${v.className}`;
        const small = document.createElement('small');
        small.textContent = `Nível ${v.level} · CP ${v.combatPower ? v.combatPower.toLocaleString() : '—'} · GS ${v.gearScore ? v.gearScore.toLocaleString() : '—'}`;
        card.append(strong, small);
        return card;
      }));
    } else {
      const p = document.createElement('p');
      p.className = 'reference-note';
      p.textContent = 'Não foi possível carregar a lista de integrantes remota e nenhum outro membro desta legião foi consultado localmente ainda.';
      list.replaceChildren(p);
    }
  }

  if (window.aion2Host) {
    window.aion2Host.onMessage(event => {
      if (event.data?.type === 'profile_sync') {
        const update = event.data;
        if (loadedItem && update.serverId === loadedItem.serverId && update.characterId === loadedItem.characterId
            && update.globalRegion === loadedItem.queryGlobalRegion) {
          if (update.error) profileStatus.textContent = formatUi('profileSyncFailed', { error: update.error });
          else if (update.data) showProfile(update.data);
        }
        if (enemyLoadedItem && update.serverId === enemyLoadedItem.serverId && update.characterId === enemyLoadedItem.characterId
            && update.globalRegion === enemyLoadedItem.queryGlobalRegion) {
          if (update.error) enemyProfileStatus.textContent = formatUi('profileSyncFailed', { error: update.error });
          else if (update.data) showEnemyProfile(update.data);
        }
        void refreshSavedProfiles();
        return;
      }
      const { id, data, error } = event.data || {};
      const request = requests.get(id);
      if (!request) return;
      requests.delete(id);
      if (error) request.reject(new Error(error));
      else request.resolve(data);
    });
  }
  function request(kind, parameters, queryRegion = region.value, queryGlobalRegion = globalRegion.value) {
    if (!window.aion2Host) return Promise.reject(new Error(ui('requiresWindows')));
    const id = ++nextRequest;
    return new Promise((resolve, reject) => {
      requests.set(id, { resolve, reject });
      window.aion2Host.post({ id, kind, region: queryRegion,
        globalRegion: queryGlobalRegion, ...parameters });
    });
  }
  function resetSearch() {
    ++currentSearch;
    ++currentProfile;
    results = [];
    loaded = null;
    loadedItem = null;
    choiceLabel.hidden = true;
    panel.hidden = true;
    searchStatus.textContent = ui('searchAgain');
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
    loadedItem = null;
    choiceLabel.hidden = true;
    panel.hidden = true;
    importButton.disabled = true;
    searchStatus.textContent = ui('searchingProfiles');
    try {
      const queryRegion = region.value;
      const queryGlobalRegion = globalRegion.value;
      const data = await request('search', { name });
      if (searchId !== currentSearch) return;
      results = Array.isArray(data.results) ? data.results.map(item => ({
        ...item, queryRegion, queryGlobalRegion
      })) : [];
      if (!results.length) {
        searchStatus.textContent = data.complete === false ? ui('searchStillProcessing') : ui('noCharacters');
        return;
      }
      recentSearches = [name, ...recentSearches.filter(value => value !== name)].slice(0, 10);
      renderRecentSearches();
      try { localStorage.setItem('aion2RecentSearches', JSON.stringify(recentSearches)); }
      catch (error) { profileStatus.textContent = formatUi('profileStorageFailed', { error: error.message }); }
      searchStatus.textContent = formatUi('charactersFound', { count: results.length,
        extra: data.complete === false ? ui('moreMayAppear') : '' });
      choice.replaceChildren(new Option(ui('selectCharacter'), ''));
      results.forEach((item, index) => {
        const classLabel = item.className && item.className !== 'Unknown' ? item.className : '';
        const cpLabel = item.combatPower ? `CP ${item.combatPower.toLocaleString()}` : '';
        const details = [item.serverName, classLabel, cpLabel].filter(Boolean).join(' · ');
        choice.add(new Option(`${item.name}${details ? ` · ${details}` : ''}`, String(index)));
      });
      if (results.length === 1) await loadCharacter(0);
      else choiceLabel.hidden = false;
    } catch (error) {
      if (searchId === currentSearch) searchStatus.textContent = formatUi('searchFailed', { error: error.message });
    }
  });
  choice.addEventListener('change', () => {
    if (choice.value !== '') void loadCharacter(Number(choice.value));
  });
  async function loadCharacter(index, force = false) {
    const item = results[index];
    if (!item) return;
    loadedItem = item;
    document.querySelector('#favorite-profile').setAttribute('aria-pressed', String(favorites.includes(profileKey(item))));
    const profileId = ++currentProfile;
    panel.hidden = false;
    loaded = null;
    importButton.disabled = true;
    document.querySelector('#profile-title').textContent = `${item.name} · ${item.serverName}`;
    document.querySelector('#profile-meta').textContent = `${item.className} · ${ui('level')} ${item.level} · ${item.raceName || ''} · CP ${item.combatPower ?? '—'}`;
    setDisplayIcon(document.querySelector('#profile-class-icon'), 'class', item.className);
    profileStatus.textContent = ui('loadingProfile');
    document.querySelector('#profile-dossier').replaceChildren();
    try {
      const data = await request(force ? 'refresh_profile' : 'profile', { serverId: item.serverId, characterId: item.characterId,
        region: item.queryRegion, globalRegion: item.queryGlobalRegion });
      if (profileId !== currentProfile) return;
      showProfile(data);
      void refreshSavedProfiles();
    } catch (error) {
      if (profileId === currentProfile) profileStatus.textContent = formatUi('profileFailed', { error: error.message });
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
    document.dispatchEvent(new CustomEvent('character-imported', {
      detail: { name: attacker.name, defenses: enemyDefenses(loaded), stats: attacker.stats,
        modifiers: sheetModifiers(loaded) }
    }));
    const coverage = formulaImportCoverage(loaded);
    const manual = coverage.manual.map(field => {
      const input = form.elements.namedItem(field);
      const text = input?.closest('label')?.childNodes;
      const label = text && [...text].find(node => node.nodeType === 3 && node.textContent.trim());
      return label?.textContent.trim() || field;
    }).join(', ') || ui('importManualFallback');
    const sourceCounts = Object.values(coverage.sources);
    const coverageText = formatUi('importCoverage', {
      formula: coverage.imported.length,
      sheet: sourceCounts.filter(source => source !== 'gear').length,
      gear: sourceCounts.filter(source => source === 'gear').length,
      references: Object.keys(references).length,
      manual
    });
    profileStatus.textContent = coverageText;
    const warning = document.querySelector('#import-warning');
    warning.hidden = false;
    warning.textContent = coverageText;
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
  let enemyLoadedItem = null;
  let enemySearchId = 0;
  let enemyProfileId = 0;

  function resetEnemySearch() {
    ++enemySearchId;
    ++enemyProfileId;
    enemyResults = [];
    enemyLoaded = null;
    enemyLoadedItem = null;
    enemyChoiceLabel.hidden = true;
    enemyPanel.hidden = true;
    enemyImportButton.disabled = true;
    enemySearchStatus.textContent = ui('searchAgain');
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
    enemyLoadedItem = null;
    enemyChoiceLabel.hidden = true;
    enemyPanel.hidden = true;
    enemyImportButton.disabled = true;
    enemySearchStatus.textContent = ui('searchingProfiles');
    try {
      const queryRegion = enemyRegion.value;
      const queryGlobalRegion = enemyGlobalRegion.value;
      const data = await request('search', { name }, queryRegion, queryGlobalRegion);
      if (searchId !== enemySearchId) return;
      enemyResults = Array.isArray(data.results) ? data.results.map(item => ({
        ...item, queryRegion, queryGlobalRegion
      })) : [];
      if (!enemyResults.length) {
        enemySearchStatus.textContent = data.complete === false ? ui('searchStillProcessing') : ui('noCharacters');
        return;
      }
      enemySearchStatus.textContent = formatUi('charactersFound', { count: enemyResults.length, extra: '' });
      enemyChoice.replaceChildren(new Option(ui('selectCharacter'), ''));
      enemyResults.forEach((item, index) => {
        const classLabel = item.className && item.className !== 'Unknown' ? item.className : '';
        const cpLabel = item.combatPower ? `CP ${item.combatPower.toLocaleString()}` : '';
        const details = [item.serverName, classLabel, cpLabel].filter(Boolean).join(' · ');
        enemyChoice.add(new Option(`${item.name}${details ? ` · ${details}` : ''}`, String(index)));
      });
      if (enemyResults.length === 1) await loadEnemy(0);
      else enemyChoiceLabel.hidden = false;
    } catch (error) {
      if (searchId === enemySearchId) enemySearchStatus.textContent = formatUi('searchFailed', { error: error.message });
    }
  });
  enemyChoice.addEventListener('change', () => {
    if (enemyChoice.value !== '') void loadEnemy(Number(enemyChoice.value));
  });
  async function loadEnemy(index) {
    const item = enemyResults[index];
    if (!item) return;
    enemyLoadedItem = item;
    const profileId = ++enemyProfileId;
    enemyLoaded = null;
    enemyPanel.hidden = false;
    enemyImportButton.disabled = true;
    const initialClass = item.className && item.className !== 'Unknown' ? item.className : '';
    const initialCp = item.combatPower ? `CP ${item.combatPower.toLocaleString()}` : '';
    const initialDetails = [initialClass, `${ui('level')} ${item.level}`, initialCp].filter(Boolean).join(' · ');
    document.querySelector('#enemy-title').textContent = `${item.name} · ${item.serverName}`;
    document.querySelector('#enemy-meta').textContent = initialDetails;
    if (initialClass) setDisplayIcon(document.querySelector('#enemy-class-icon'), 'class', initialClass);
    enemyProfileStatus.textContent = ui('loadingDefenses');
    document.querySelector('#enemy-dossier').replaceChildren();
    document.querySelector('#enemy-stats').replaceChildren();
    try {
      const data = await request('profile', { serverId: item.serverId, characterId: item.characterId },
        item.queryRegion, item.queryGlobalRegion);
      if (profileId !== enemyProfileId) return;
      showEnemyProfile(data);
    } catch (error) {
      if (profileId === enemyProfileId) enemyProfileStatus.textContent = formatUi('profileFailed', { error: error.message });
    }
  }
  function showEnemyProfile(data) {
    enemyLoaded = data;
    renderDossier(document.querySelector('#enemy-dossier'), extractCharacter(data));
    const enemyChar = extractCharacter(data);
    document.querySelector('#enemy-title').textContent = `${enemyChar.name} · ${enemyChar.server}`;
    const enemyCp = enemyChar.combatPower ? `CP ${enemyChar.combatPower.toLocaleString()}` : '';
    const enemyGs = enemyChar.gearScore ? `GS ${enemyChar.gearScore.toLocaleString()}` : '';
    const enemyScores = [enemyCp, enemyGs].filter(Boolean).join(' · ');
    document.querySelector('#enemy-meta').textContent = `${enemyChar.className} · ${ui('level')} ${enemyChar.level} · ${enemyChar.race}${enemyScores ? ` · ${enemyScores}` : ''}`;
    setDisplayIcon(document.querySelector('#enemy-class-icon'), 'class', enemyChar.className);
    const defenses = enemyDefenses(data);
    const sheet = enemySheetDefenses(data);
    const entries = Object.entries(defenses);
    const hp = characterHp(data);
    if (hp !== null) entries.push(['enemyHp', hp]);
    document.querySelector('#enemy-stats').replaceChildren(...(entries.length
      ? entries.map(([field, value]) => `${field === 'enemyHp' ? 'HP' : enemyLabels[field]}: ${value} (${field === 'enemyHp' || field in sheet ? ui('characterSheetSource') : ui('equipmentPartialSource')})`)
      : [ui('noDefenses')]).map(text => {
      const row = document.createElement('li');
      row.append(iconElement(attributeIcon(text.split(':')[0])), document.createTextNode(text));
      return row;
    }));
    enemyImportButton.disabled = !entries.length;
    const date = data.cacheMeta?.fetchedAt;
    enemyProfileStatus.textContent = `${ui(data.complete === false ? 'enemyPartialProfile' : 'enemyProfileLoaded')}${date ? ` · ${formatUi(data.cacheMeta.cached ? 'cachedProfileDate' : 'profileDate', { date: profileDate(date) })}` : ''}`;
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
    document.querySelector('#target-class').value = extractCharacter(enemyLoaded).className || document.querySelector('#enemy-meta').textContent.split(' · ')[0];
    calculator.dispatchEvent(new Event('input', { bubbles: true }));
    const warning = document.querySelector('#import-warning');
    warning.hidden = false;
    warning.textContent = formatUi('enemyImportResult', {
      count: Object.keys(defenses).length,
      name: extractCharacter(enemyLoaded).name,
      hp: hp === null ? ui('enemyHpUnknown') : formatUi('enemyHpKnown', { hp })
    });
    showTab('calculator-view');
    document.querySelector('.encounter').scrollIntoView({ block: 'start' });
  });
  I18N.onLanguageChange(() => {
    if (loaded) {
      renderDossier(document.querySelector('#profile-dossier'), extractCharacter(loaded));
      updateProfileStatus(loaded);
    }
    if (enemyLoaded) {
      renderDossier(document.querySelector('#enemy-dossier'), extractCharacter(enemyLoaded));
      enemyProfileStatus.textContent = enemyLoaded.complete === false
        ? ui('enemyPartialProfile') : ui('enemyProfileLoaded');
    }
  });
  async function syncSettings(save = false) {
    if (!window.aion2Host) return;
    const enabled = document.querySelector('#sync-enabled');
    const interval = document.querySelector('#sync-interval');
    if (save && !interval.reportValidity()) return;
    try {
      const settings = await request('sync_settings', save ? { enabled: enabled.checked, interval: Number(interval.value) } : {});
      enabled.checked = settings.enabled;
      interval.value = String(settings.interval);
    } catch (error) { searchStatus.textContent = error.message; }
  }
  document.querySelector('#sync-enabled').addEventListener('change', () => { void syncSettings(true); });
  document.querySelector('#sync-interval').addEventListener('change', () => { void syncSettings(true); });
  void syncSettings();
  void refreshSavedProfiles();
}
