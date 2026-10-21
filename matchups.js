// Wiki stat names, paired by the mechanic they affect. These are references, not new formula terms.
const referenceGroups = [
  { title: 'Hit, evasion and block', note: 'Accuracy counters both Evasion and Block. Block Penetration counters Block only; attacking from behind bypasses Block.', open: true, rows: [
    ['Accuracy', 'accuracy', 'Evasion', 'evasion'],
    ['Block Penetration', 'blockPenetration', 'Block', 'block'],
    ['PvP Accuracy', 'pvpAccuracy', 'PvP Evasion', 'pvpEvasion'],
    ['PvP Block Penetration', 'pvpBlockPenetration', 'PvP Block', 'pvpBlock']
  ] },
  { title: 'Critical hits and flat positional attack', note: 'Critical Hit affects chance; flat Critical/Front/Back Attack and their defenses are not terms in the supplied damage equation.', open: true, rows: [
    ['Critical Hit', 'criticalHit', 'Critical Hit Resist', 'criticalHitResist'],
    ['Front Attack Critical Hit', 'frontCriticalHit', 'Front Attack Critical Hit Resist', 'frontCriticalHitResist'],
    ['Back Attack Critical Hit', 'backCriticalHit', 'Back Attack Critical Hit Resist', 'backCriticalHitResist'],
    ['Critical Attack', 'criticalAttack', 'Critical Damage Defense', 'criticalDamageDefense'],
    ['Front Attack', 'frontAttack', 'Front Defense', 'frontDefense'],
    ['Back Attack', 'backAttack', 'Back Defense', 'backDefense'],
    ['PvP Critical Hit', 'pvpCriticalHit', 'PvP Critical Hit Resist', 'pvpCriticalHitResist']
  ] },
  { title: 'Procs and recovery', note: 'Resists change activation chances, not the damage of a landed hit. Endurance halves a hit only if triggered; Regeneration recovers HP.', open: true, rows: [
    ['Double Chance', 'doubleChance', 'Double Resist', 'doubleResist', true],
    ['Perfect Chance', 'perfectChance', 'Perfect Resist', 'perfectResist', true],
    ['Multi-hit Chance', 'multiHitChance', 'Multi-hit Resist', 'multiHitResist', true],
    ['Endurance Penetration', 'endurancePenetration', 'Endurance', 'endurance', true],
    ['Regeneration Penetration', 'regenerationPenetration', 'Regeneration', 'regeneration', true]
  ] },
  { title: 'Situational damage and elements', note: 'These stats require the matching target type or elemental skill. Boss, PvP and elemental terms are not in the supplied final-damage formula.', open: false, rows: [
    ['Boss Damage Boost', 'bossDamageBoost', 'Boss Damage Tolerance', 'bossDamageTolerance', true],
    ['PvP Damage Boost', 'pvpDamageBoost', 'PvP Damage Tolerance', 'pvpDamageTolerance', true],
    ['PvP Attack', 'pvpAttack', 'PvP Defense', 'pvpDefense'],
    ['Fire Boost', 'fireBoost', 'Fire Tolerance', 'fireTolerance', true],
    ['Water Boost', 'waterBoost', 'Water Tolerance', 'waterTolerance', true],
    ['Wind Boost', 'windBoost', 'Wind Tolerance', 'windTolerance', true],
    ['Earth Boost', 'earthBoost', 'Earth Tolerance', 'earthTolerance', true]
  ] },
  { title: 'Creature-type matchups', note: 'Cogni, Fera, Natura and Varian each have separate stats; the generic Species fields above are formula placeholders. These reference fields do not change the formula.', open: false, rows: ['Cogni', 'Fera', 'Natura', 'Varian'].flatMap((type) => [
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
    details.className = 'reference-group';
    details.open = group.open;
    const summary = document.createElement('summary');
    summary.textContent = group.title;
    const note = document.createElement('p');
    note.textContent = group.note;
    const grid = document.createElement('div');
    grid.className = 'matchup-grid';
    for (const [attackLabel, attackField, defenseLabel, defenseField, percentage] of group.rows) {
      const pair = document.createElement('div');
      pair.className = 'matchup';
      for (const [label, field, enemy] of [[attackLabel, attackField, false], [defenseLabel, defenseField, true]]) {
        const wrapper = document.createElement('label');
        if (enemy) wrapper.className = 'enemy';
        wrapper.textContent = `${enemy ? 'Enemy ' : ''}${label}${percentage ? ' (%)' : ''}`;
        const input = document.createElement('input');
        input.name = field;
        input.type = 'number';
        input.min = '0';
        input.step = 'any';
        input.value = '0';
        wrapper.append(input);
        pair.append(wrapper);
      }
      grid.append(pair);
    }
    details.append(summary, note, grid);
    container.append(details);
  }
}
