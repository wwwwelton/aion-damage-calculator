const bonusFields = {
  'Attack Bonus': ['attackBonus', 0, false],
  'PvE Attack': ['pveAttackPower', 0, false],
  'PvE Attack Power': ['pveAttackPower', 0, false],
  'Boss Attack': ['bossAttackPower', 0, false],
  'Boss Attack Power': ['bossAttackPower', 0, false],
  'Weapon Damage Boost': ['weaponDamageBoost', 100, true],
  'Damage Boost': ['damageBoost', 100, true],
  'PvE Damage Boost': ['pveDamageBoost', 0, true],
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

function extractCharacter(data) {
  const profile = data?.info?.profile || {};
  const stats = (data?.info?.stat?.statList || [])
    .filter(item => item.name && Number.isFinite(Number(item.value)))
    .map(item => `${item.name}: ${item.value}`);
  const gear = equippedDetails(data).map(({ item, detail }) => {
    const options = [...(detail.mainStats || []), ...(detail.subStats || [])]
      .filter(option => option.name && option.value != null)
      .map(option => `${option.name} +${option.value}`);
    return `${item.slotPosName || 'Gear'} · ${item.name || detail.name || 'Unknown'}${options.length ? ' — ' + options.join(', ') : ''}`;
  });
  const skills = (data?.equipment?.skill?.skillList || [])
    .filter(skill => skill.name && Number(skill.acquired) && Number(skill.skillLevel) > 0)
    .map(skill => `${skill.name} · Lv ${skill.skillLevel} · ${skill.category || 'Skill'}`);
  return { name: profile.characterName || '', server: profile.serverName || '',
    className: profile.className || '', level: profile.characterLevel || '',
    combatPower: profile.combatPower || '', stats, gear, skills };
}

if (typeof module !== 'undefined') module.exports = { extractCharacter, equipmentBonuses };

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
  function request(kind, parameters) {
    if (!window.chrome?.webview) return Promise.reject(new Error('Character search requires the Windows app.'));
    const id = ++nextRequest;
    return new Promise((resolve, reject) => {
      requests.set(id, { resolve, reject });
      window.chrome.webview.postMessage({ id, kind, region: region.value,
        globalRegion: globalRegion.value, ...parameters });
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
    profileStatus.textContent = 'Loading attributes, gear and skills…';
    for (const id of ['profile-stats', 'profile-gear', 'profile-skills']) document.getElementById(id).replaceChildren();
    try {
      const data = await request('profile', { serverId: item.serverId, characterId: item.characterId,
        region: item.queryRegion, globalRegion: item.queryGlobalRegion });
      if (profileId !== currentProfile) return;
      loaded = data;
      const view = extractCharacter(data);
      for (const [id, entries] of [['profile-stats', view.stats], ['profile-gear', view.gear], ['profile-skills', view.skills]]) {
        const list = document.getElementById(id);
        list.replaceChildren(...(entries.length ? entries : ['Not available in this public profile.']).map(text => {
          const row = document.createElement('li');
          row.textContent = text;
          return row;
        }));
      }
      importButton.disabled = !Object.keys(equipmentBonuses(data)).length;
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
    const form = document.querySelector('#calculator');
    for (const [field, previous] of Object.entries(previousImport)) {
      if (field in bonuses) continue;
      const input = form.elements.namedItem(field);
      if (input && Number(input.value) === previous) {
        input.value = String(Object.values(bonusFields).find(([name]) => name === field)[1]);
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
    form.dispatchEvent(new Event('input', { bubbles: true }));
    profileStatus.textContent = `Loaded ${Object.keys(bonuses).length} equipment bonus fields. All other values remain manual; these are not complete character totals.`;
    const warning = document.querySelector('#import-warning');
    warning.hidden = false;
    warning.textContent = `Imported ${Object.keys(bonuses).length} equipment bonuses for ${extractCharacter(loaded).name}. Pure Attack, target Defense and other missing inputs are still manual or example values; verify them before relying on Final Damage.`;
    showTab('calculator-view');
    document.querySelector('#result').scrollIntoView({ block: 'start' });
  });
}
