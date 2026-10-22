// Wiki stat names, paired by the mechanic they affect. These are references, not new formula terms.
const referenceGroups = [
  { title: 'Hit, evasion and block', titlePt: 'Acerto, evasão e bloqueio',
    note: 'Accuracy counters both Evasion and Block. Block Penetration counters Block only; attacking from behind bypasses Block.',
    notePt: 'Precisão neutraliza Evasão e Bloqueio. Penetração de Bloqueio neutraliza apenas Bloqueio; atacar pelas costas ignora Bloqueio.', open: true, rows: [
    ['Accuracy', 'accuracy', 'Evasion', 'evasion'],
    ['Block Penetration', 'blockPenetration', 'Block', 'block'],
    ['PvP Accuracy', 'pvpAccuracy', 'PvP Evasion', 'pvpEvasion'],
    ['PvP Block Penetration', 'pvpBlockPenetration', 'PvP Block', 'pvpBlock']
  ] },
  { title: 'Critical hits and flat positional attack', titlePt: 'Críticos e ataque posicional fixo',
    note: 'Critical Hit affects chance; flat Critical/Front/Back Attack and their defenses are not terms in the supplied damage equation.',
    notePt: 'Acerto Crítico afeta a chance; Ataque Crítico/Frontal/Traseiro e suas defesas não fazem parte da equação de dano fornecida.', open: true, rows: [
    ['Critical Hit', 'criticalHit', 'Critical Hit Resist', 'criticalHitResist'],
    ['Front Attack Critical Hit', 'frontCriticalHit', 'Front Attack Critical Hit Resist', 'frontCriticalHitResist'],
    ['Back Attack Critical Hit', 'backCriticalHit', 'Back Attack Critical Hit Resist', 'backCriticalHitResist'],
    ['Critical Attack', 'criticalAttack', 'Critical Damage Defense', 'criticalDamageDefense'],
    ['Front Attack', 'frontAttack', 'Front Defense', 'frontDefense'],
    ['Back Attack', 'backAttack', 'Back Defense', 'backDefense'],
    ['PvP Critical Hit', 'pvpCriticalHit', 'PvP Critical Hit Resist', 'pvpCriticalHitResist']
  ] },
  { title: 'Procs and recovery', titlePt: 'Ativações e recuperação',
    note: 'Resists change activation chances, not the damage of a landed hit. Endurance halves a hit only if triggered; Regeneration recovers HP.',
    notePt: 'Resistências alteram chances de ativação, não o dano de um golpe acertado. Endurance reduz o golpe à metade quando ativa; Regeneration recupera HP.', open: true, rows: [
    ['Double Chance', 'doubleChance', 'Double Resist', 'doubleResist', true],
    ['Perfect Chance', 'perfectChance', 'Perfect Resist', 'perfectResist', true],
    ['Multi-hit Chance', 'multiHitChance', 'Multi-hit Resist', 'multiHitResist', true],
    ['Endurance Penetration', 'endurancePenetration', 'Endurance', 'endurance', true],
    ['Regeneration Penetration', 'regenerationPenetration', 'Regeneration', 'regeneration', true]
  ] },
  { title: 'PvP damage (reference only)', titlePt: 'Dano PvP (somente referência)', mode: 'player',
    note: 'Applies against players. PvP Attack, PvP Defense, PvP Damage Boost and PvP Damage Tolerance have no conversion in the supplied equation; the displayed hit is a shared-stat baseline, not final PvP damage.',
    notePt: 'Aplica-se contra jogadores. Ataque, Defesa, Aumento e Tolerância PvP não possuem conversão verificada na equação; o golpe exibido é uma base de atributos compartilhados.', open: false, rows: [
    ['PvP Damage Boost', 'pvpDamageBoost', 'PvP Damage Tolerance', 'pvpDamageTolerance', true],
    ['PvP Attack', 'pvpAttack', 'PvP Defense', 'pvpDefense']
  ] },
  { title: 'Elemental damage (reference only)', titlePt: 'Dano elemental (somente referência)',
    note: 'Choose the element of the actual skill. Fire, Water, Wind and Earth stats are shown independently; elemental boosts and tolerances are not included in the supplied damage equation.',
    notePt: 'Escolha o elemento da habilidade real. Fogo, Água, Vento e Terra são exibidos separadamente; aumentos e tolerâncias elementais não entram na equação fornecida.', open: false, rows: [
    ['Fire Boost', 'fireBoost', 'Fire Tolerance', 'fireTolerance', true],
    ['Water Boost', 'waterBoost', 'Water Tolerance', 'waterTolerance', true],
    ['Wind Boost', 'windBoost', 'Wind Tolerance', 'windTolerance', true],
    ['Earth Boost', 'earthBoost', 'Earth Tolerance', 'earthTolerance', true]
  ] },
  { title: 'Edit type-specific stats', titlePt: 'Editar atributos específicos do gênero', mode: 'pve',
    note: 'The selected type’s Damage Boost and Damage Tolerance replace generic Species terms in Final Damage. Attack, Defense, hit and block stats below are reference-only.',
    notePt: 'Aumento e Tolerância do gênero selecionado substituem os termos genéricos de Espécie no Dano Final. Ataque, Defesa, acerto e bloqueio abaixo são somente referência.', open: false, rows: ['Cogni', 'Fera', 'Natura', 'Varian'].flatMap((type) => [
    [`${type} Damage Boost`, `${type.toLowerCase()}DamageBoost`, `${type} Damage Tolerance`, `${type.toLowerCase()}DamageTolerance`, true],
    [`${type} Attack`, `${type.toLowerCase()}Attack`, `${type} Defense`, `${type.toLowerCase()}Defense`],
    [`${type} Accuracy`, `${type.toLowerCase()}Accuracy`, `${type} Evasion`, `${type.toLowerCase()}Evasion`],
    [`${type} Block Penetration`, `${type.toLowerCase()}BlockPenetration`, `${type} Block`, `${type.toLowerCase()}Block`],
    [`${type} Critical Hit`, `${type.toLowerCase()}CriticalHit`, `${type} Critical Hit Resist`, `${type.toLowerCase()}CriticalHitResist`]
  ]) }
];

if (typeof module !== 'undefined') module.exports = { referenceGroups };

if (typeof document !== 'undefined') {
  const container = document.querySelector('#reference-matchups');
  for (const group of referenceGroups) {
    const details = document.createElement('details');
    details.className = `reference-group${group.mode ? ` ${group.mode === 'pve' ? 'pve-only' : group.mode + '-only'}` : ''}`;
    details.open = group.open;
    const summary = document.createElement('summary');
    summary.textContent = I18N.currentLang === 'pt' ? group.titlePt : group.title;
    const note = document.createElement('p');
    note.textContent = I18N.currentLang === 'pt' ? group.notePt : group.note;
    const grid = document.createElement('div');
    grid.className = 'matchup-grid';
    for (const [attackLabel, attackField, defenseLabel, defenseField, percentage] of group.rows) {
      const pair = document.createElement('div');
      pair.className = 'matchup';
      for (const [label, field, enemy] of [[attackLabel, attackField, false], [defenseLabel, defenseField, true]]) {
        const wrapper = document.createElement('label');
        if (enemy) wrapper.className = 'enemy';
        wrapper.textContent = `${enemy ? (I18N.currentLang === 'pt' ? 'Inimigo ' : 'Enemy ') : ''}${I18N.statLabel(label)}${percentage ? ' (%)' : ''}`;
        const input = document.createElement('input');
        input.name = field;
        input.type = 'number';
        input.min = '0';
        input.step = 'any';
        input.value = '0';
        wrapper.append(input);
        wrapper.dataset.statLabel = label;
        wrapper.dataset.enemy = String(enemy);
        wrapper.dataset.percentage = String(Boolean(percentage));
        pair.append(wrapper);
      }
      grid.append(pair);
    }
    details.append(summary, note, grid);
    (group.mode === 'pve' ? document.querySelector('#creature-matchups') : container).append(details);
  }
  if (typeof attachStatTooltips === 'function') attachStatTooltips();
  I18N.onLanguageChange(() => {
    document.querySelectorAll('.reference-group').forEach((details, index) => {
      const group = referenceGroups[index];
      details.querySelector('summary').textContent = I18N.currentLang === 'pt' ? group.titlePt : group.title;
      details.querySelector('p').textContent = I18N.currentLang === 'pt' ? group.notePt : group.note;
      details.querySelectorAll('label[data-stat-label]').forEach(label => {
        const prefix = label.dataset.enemy === 'true' ? (I18N.currentLang === 'pt' ? 'Inimigo ' : 'Enemy ') : '';
        const suffix = label.dataset.percentage === 'true' ? ' (%)' : '';
        label.firstChild.textContent = `${prefix}${I18N.statLabel(label.dataset.statLabel)}${suffix}`;
      });
    });
  });
}
