const { test } = require('node:test');
const assert = require('node:assert/strict');
const { extractCharacter, equipmentBonuses, enemyDefenses, offensiveReferences, characterHp,
  formulaImportCoverage } = require('./characters');

const profile = {
  info: { profile: { characterName: 'Kanon', serverName: 'Siel', className: 'Cleric', characterLevel: 45, combatPower: 90000 },
    stat: { statList: [{ name: 'Might', value: 12 }] } },
  equipment: { equipment: { equipmentList: [{ slotPos: 1, slotPosName: 'MainHand', name: 'Mace' }] },
    skill: { skillList: [{ name: 'Bolt', skillLevel: 12, acquired: 1, category: 'Active' },
      { name: 'Locked', skillLevel: 0, acquired: 0 }] } },
  itemDetails: { '1': { subStats: [{ name: 'Damage Boost', value: '5.6%' },
    { name: 'PvE Attack', value: '20' }, { name: 'Weapon Damage Boost', value: '3%' },
    { name: 'Attack', value: '251' }, { name: 'Critical Damage Boost', value: '2%' }],
    mainStats: [{ name: 'Attack', value: '323', minValue: '251', extra: '119' }] } }
};

test('groups real gear options, learned abilities and progression without inventing missing bonuses', () => {
  const data = structuredClone(profile);
  data.info.profile.titleName = 'Treading on Black Wings';
  data.info.title = { titleList: [{ name: 'Treading on Black Wings', equipCategory: 'Attack',
    equipStatList: [{ desc: 'PvP Attack +40' }] }] };
  data.info.daevanion = { boardList: [{ name: 'Nezekan', openNodeCount: 12, totalNodeCount: 20 }] };
  data.equipment.equipment.equipmentList[0].icon = 'https://assets.playnccdn.com/static-aion2-gamedata/resources/mace.png';
  data.itemDetails['1'].subSkills = [{ name: 'Bolt', level: 1 }];
  data.itemDetails['1'].magicStoneStat = [{ name: 'HP', value: '+100' }];
  data.info.stat.statList.push({ name: 'Damage Tolerance', value: '12.5%' },
    { name: 'Attack Modifier', value: '-2' }, { name: 'Unavailable', value: 'unknown' });
  data.info.profile.genderName = 'Female';
  data.info.profile.titleName = 'Treading on Black Wings';
  data.info.title.ownedCount = 170;
  data.info.title.totalCount = 297;
  data.info.stat.statList.push({ name: '아이템레벨', type: 'ItemLevel', value: 1805 });
  data.equipment.skill.skillList[0].icon = 'https://assets.playnccdn.com/static-aion2-gamedata/resources/bolt.png';
  const view = extractCharacter(data);
  assert.equal(view.name, 'Kanon');
  assert.deepEqual(view.stats.map(stat => stat.name), ['Might', 'Damage Tolerance', 'Attack Modifier', '아이템레벨']);
  assert.deepEqual([view.gender, view.titleName, view.itemLevel, view.titlesOwned, view.titlesTotal],
    ['Female', 'Treading on Black Wings', 1805, 170, 297]);
  assert.equal(view.gear[0].item.name, 'Mace');
  assert.equal(view.gear[0].detail.subStats[0].value, '5.6%');
  assert.equal(view.gear[0].detail.subSkills[0].name, 'Bolt');
  assert.equal(view.gear[0].detail.magicStoneStat[0].value, '+100');
  assert.deepEqual(view.skills.map(skill => skill.name), ['Bolt']);
  assert.equal(view.titles[0].equipStatList[0].desc, 'PvP Attack +40');
  assert.equal(view.boards[0].openNodeCount, 12);
  assert.deepEqual(view.arcana, []);
});

test('imports only explicit numeric gear bonuses with their appropriate percentage baselines', () => {
  assert.deepEqual(equipmentBonuses(profile), {
    damageBoost: 105.6, pveAttackPower: 20, weaponDamageBoost: 103, criticalDamageBoost: 152
  });
});

test('imports language-independent Might scaling and explicit formula stats without inventing Pure Attack', () => {
  const data = structuredClone(profile);
  data.info.stat.statList[0] = { name: '위력', type: 'STR', value: 56,
    statSecondList: ['공격력 증가 +5.6%'] };
  data.itemDetails['1'].subStats.push({ name: 'Penetration', value: '85' },
    { name: 'Skill Specialty Enhancement', value: '2%' });
  assert.deepEqual(equipmentBonuses(data), {
    damageBoost: 105.6, pveAttackPower: 20, weaponDamageBoost: 103,
    criticalDamageBoost: 152, penetration: 85, skillSpecialtyEnhancement: 102,
    attackIncrease: 105.6
  });
  const coverage = formulaImportCoverage(data);
  assert.ok(coverage.imported.includes('penetration'));
  assert.ok(coverage.imported.includes('attackIncrease'));
  assert.ok(coverage.manual.includes('pureAttack'));
  assert.ok(!coverage.imported.includes('pureAttack'));
});

test('ignores incomplete item details and invalid bonus units rather than inventing missing totals', () => {
  assert.deepEqual(equipmentBonuses({ equipment: { equipment: { equipmentList: [] } }, itemDetails: { '1': profile.itemDetails['1'] } }), {});
  const bad = structuredClone(profile);
  bad.itemDetails['1'].subStats = [{ name: 'Damage Boost', value: '560' }, { name: 'Attack Bonus', value: 'unknown' }];
  assert.deepEqual(equipmentBonuses(bad), {});
});

test('imports explicit enemy sheet defenses without substituting offensive or inferred gear stats', () => {
  const enemy = structuredClone(profile);
  enemy.info.stat.statList = [
    { name: 'Defense', value: '825' },
    { name: 'Damage Tolerance', value: '12.5%' },
    { name: 'PvE Damage Tolerance', value: '4%' },
    { name: 'Weapon Damage Tolerance', value: '3%' },
    { name: 'Critical Damage Tolerance', value: '5%' },
    { name: 'Back Attack Damage Tolerance', value: '7%' },
    { name: 'Block', value: '162' },
    { name: 'Evasion', value: '22' },
    { name: 'Double Resist', value: '8%' },
    { name: 'Endurance', value: '3%' },
    { name: 'Attack', value: '500' },
    { name: 'Boss Damage Tolerance', value: '?' }
  ];
  assert.deepEqual(enemyDefenses(enemy), {
    defense: 825, damageTolerance: 12.5, pveDamageTolerance: 4,
    weaponDamageTolerance: 3, criticalDamageTolerance: 5,
    backDamageTolerance: 7, block: 162, evasion: 22, doubleResist: 8, endurance: 3
  });
});

test('uses explicit equipped defenses as partial values when public sheet omits them', () => {
  const enemy = structuredClone(profile);
  enemy.itemDetails['1'].subStats.push({ name: 'Defense', value: '230' },
    { name: 'Block', value: '15' }, { name: 'Damage Tolerance', value: '2.5%' },
    { name: 'Weapon Damage Tolerance', value: '4%' });
  enemy.itemDetails['1'].mainStats.push({ name: 'Defense', value: '120' });
  assert.deepEqual(enemyDefenses(enemy), {
    defense: 350, block: 15, damageTolerance: 2.5, weaponDamageTolerance: 4
  });
  enemy.info.stat.statList.push({ name: 'Defense', value: '890' });
  assert.equal(enemyDefenses(enemy).defense, 890);
});

test('first character supplies comparable offensive stats from sheet, then explicit gear only when absent', () => {
  const player = structuredClone(profile);
  player.info.stat.statList.push({ name: 'Accuracy', value: '1200' },
    { name: 'Critical Hit', value: '475' });
  player.itemDetails['1'].subStats.push({ name: 'Accuracy', value: '150' },
    { name: 'Block Penetration', value: '28' },
    { name: 'Double Chance', value: '3.5%' },
    { name: 'Cogni Damage Boost', value: '4%' },
    { name: 'Fire Boost', value: '2.5%' },
    { name: 'Critical Hit', value: '30' },
    { name: 'Defense', value: '150' });
  assert.deepEqual(offensiveReferences(player), {
    accuracy: 1200, criticalHit: 475, blockPenetration: 28,
    doubleChance: 3.5, cogniDamageBoost: 4, fireBoost: 2.5
  });
});

test('imports language-independent Pantheon effects by stable stat type', () => {
  const data = structuredClone(profile);
  data.info.stat.statList = [
    { name: '지혜', type: 'Wisdom', value: 21 },
    { name: '환상', type: 'Illusion', value: 33 },
    { name: '죽음', type: 'Death', value: 42 },
    { name: '시간', type: 'Time', value: 53 },
    { name: '파괴', type: 'Destruction', value: 14 },
    { name: '운명', type: 'Destiny', value: 39 },
    { name: '생명', type: 'Life', value: 19 }
  ];
  assert.deepEqual(offensiveReferences(data), {
    doubleChance: 2.1, endurancePenetration: 3.3, regenerationPenetration: 4.2
  });
  assert.deepEqual(enemyDefenses(data), {
    doubleResist: 5.3, perfectResist: 1.4, endurance: 3.9, regeneration: 1.9
  });
});

test('imports only explicit positive character-sheet HP, never item or missing HP', () => {
  assert.equal(characterHp(profile), null);
  const player = structuredClone(profile);
  player.info.stat.statList.push({ name: 'HP', value: 12500 });
  assert.equal(characterHp(player), 12500);
  player.info.stat.statList.at(-1).value = 'unknown';
  assert.equal(characterHp(player), null);
});

const { sheetModifiers, effectRole, statGroup } = require('./characters');

// Trimmed from Avatarnoia's public sheet (Cleric 45, Siel): only stat types and divinities are published.
const avatarnoia = {
  info: { profile: { characterName: 'Avatarnoia', serverName: 'Siel', className: 'Cleric', characterLevel: 45 },
    stat: { statList: [
      { type: 'STR', name: 'Might', value: 4, statSecondList: ['Attack increase +0.4%'] },
      { type: 'AGI', name: 'Precision', value: 23, statSecondList: ['Accuracy increase +2.3%', 'Critical Hit increase +2.3%'] },
      { type: 'CON', name: 'Constitution', value: 16, statSecondList: ['HP increase +1.6%'] },
      { type: 'Justice', name: 'Justice [Nezekan]', value: 58, statSecondList: ['Defense increase +5.8%', 'Perfect Chance +5.8%'] },
      { type: 'Freedom', name: 'Freedom [Vaizel]', value: 52, statSecondList: ['Accuracy increase +5.2%', 'Evasion increase +5.2%'] },
      { type: 'Destruction', name: 'Destruction [Zikel]', value: 6, statSecondList: ['Attack increase +0.6%', 'Perfect Resist +0.6%'] },
      { type: 'ItemLevel', name: '아이템레벨', value: 2507 }
    ] } },
  equipment: { equipment: { equipmentList: [{ slotPos: 1, slotPosName: 'MainHand', name: 'Aulamus Mace' },
    { slotPos: 5, slotPosName: 'Torso', name: 'Vakron Breastplate' }] } },
  itemDetails: {
    '1': { mainStats: [{ id: 'WeaponFixingDamage', name: 'Attack', value: '323' }],
      subStats: [{ id: 'AmplifyAllDamage', name: '피해 증폭', value: '5.6%' },
        { id: 'DefensePierce', name: '관통', value: '700' },
        { id: 'AGI', name: 'Precision', value: '17' },
        { id: 'UnknownOption', name: 'Mystery', value: '9' }] },
    '5': { mainStats: [{ id: 'Freedom', name: 'Freedom [Vaizel]', value: '12' },
      { id: 'PvEAmplifyDamage', name: 'PvE Damage Boost', value: '0.5%' },
      { id: 'AdditionalHitRate', name: 'Multi-hit Chance', value: '1%' }] }
  }
};

test('sums Might and Destruction attack increase from the sheet and reports where each field came from', () => {
  assert.deepEqual(equipmentBonuses(avatarnoia), {
    damageBoost: 105.6, penetration: 700, pveDamageBoost: 0.5, attackIncrease: 101
  });
  const coverage = formulaImportCoverage(avatarnoia);
  assert.deepEqual(coverage.sources, {
    damageBoost: 'gear', penetration: 'gear', pveDamageBoost: 'gear', attackIncrease: 'sheet'
  });
  const mixed = structuredClone(avatarnoia);
  mixed.itemDetails['1'].subStats.push({ name: 'Attack Increase', value: '2%' });
  assert.equal(equipmentBonuses(mixed).attackIncrease, 103);
  assert.equal(formulaImportCoverage(mixed).sources.attackIncrease, 'sheet+gear');
});

test('imports gear options by stable id and never re-counts primary stats or divinities printed on gear', () => {
  assert.deepEqual(offensiveReferences(avatarnoia), { multiHitChance: 1, perfectChance: 5.8 });
  assert.deepEqual(enemyDefenses(avatarnoia), { perfectResist: 0.6 });
  assert.equal(characterHp(avatarnoia), null);
});

test('exposes percent-of-base sheet modifiers for display without turning them into flat values', () => {
  assert.deepEqual(sheetModifiers(avatarnoia), {
    accuracyIncrease: 7.5, criticalHitIncrease: 2.3, hpIncrease: 1.6,
    defenseIncrease: 5.8, evasionIncrease: 5.2
  });
  assert.equal(effectRole('Attack increase +0.6%'), 'formula');
  assert.equal(effectRole('Perfect Chance +5.8%'), 'formula');
  assert.equal(effectRole('Accuracy increase +2.3%'), 'display');
  assert.equal(effectRole('Cooldown -1.1%'), 'display');
  assert.deepEqual(avatarnoia.info.stat.statList.map(statGroup),
    ['primary', 'primary', 'primary', 'divinity', 'divinity', 'divinity', 'other']);
});
