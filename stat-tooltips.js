// AION 2 Stat Tooltips & Mechanics Encyclopedia
// Sourced from official AION 2 game files, client tooltips, and wiki documentation.

const STAT_CATALOG = {
  // Hit Detection & Defenses
  block: {
    title: 'Block (Shield Block & Parry)',
    tag: 'Defense · Hit Detection',
    desc: 'Increases Shield Block Chance and Parry Chance. When triggered, mitigates 50%–70% of incoming physical damage. Offset by attacker\'s Accuracy and Block Penetration. Attacking from behind bypasses Block completely.'
  },
  blockPenetration: {
    title: 'Block Penetration',
    tag: 'Offense · Hit Detection',
    desc: 'Directly ignores target\'s Block Chance. Prevents enemies and tanks from mitigating your physical damage with shields. Vital for physical DPS against high-block targets.'
  },
  evasion: {
    title: 'Evasion (Dodge)',
    tag: 'Defense · Hit Detection',
    desc: 'Increases Dodge Chance to completely avoid incoming physical and skill attacks (taking zero damage). Directly countered and offset by the attacker\'s Accuracy rating.'
  },
  accuracy: {
    title: 'Accuracy',
    tag: 'Offense · Hit Detection',
    desc: 'Prevents attacks from missing. Offsets both target\'s Evasion and Block. Missing deals zero damage; maintaining Accuracy above enemy Evasion is crucial for consistent DPS.'
  },
  pveAccuracy: {
    title: 'PvE Accuracy',
    tag: 'Offense · PvE Hit Detection',
    desc: 'Accuracy applied specifically against monsters and dungeon bosses. Counters high boss Evasion and Block in endgame encounters.'
  },
  pvpAccuracy: {
    title: 'PvP Accuracy',
    tag: 'PvP · Player Hit Detection',
    desc: 'Accuracy rating active exclusively in Player-vs-Player combat to counter enemy player Evasion.'
  },
  pvpEvasion: {
    title: 'PvP Evasion',
    tag: 'PvP · Player Dodge',
    desc: 'Dodge chance against incoming player attacks in PvP battles, offset by enemy PvP Accuracy.'
  },
  pvpBlock: {
    title: 'PvP Block',
    tag: 'PvP · Player Shield Block',
    desc: 'Shield Block and Parry chance against enemy player physical and skill attacks.'
  },
  pvpBlockPenetration: {
    title: 'PvP Block Penetration',
    tag: 'PvP · Anti-Shield',
    desc: 'Reduces enemy player Shield Block chance in PvP encounters.'
  },

  // Attack & Weapon Scaling
  attack: {
    title: 'Attack',
    tag: 'Offense · Flat Attack',
    desc: 'Flat attack rating from weapon, gear and buffs. Feeds Pure Attack before percentage multipliers.'
  },
  pureAttack: {
    title: 'Pure Attack',
    tag: 'Offense · Base Scaling',
    desc: 'Base physical/magical weapon and character attack power before percentage multipliers. Foundation for all damage formulas.'
  },
  weaponDamageBoost: {
    title: 'Weapon Damage Boost',
    tag: 'Offense · Multiplier (%)',
    desc: 'Multiplicative damage scaling determined by weapon mastery, weapon grade, and enhancement levels. Multiplies weapon attack output.'
  },
  weaponDamageTolerance: {
    title: 'Weapon Damage Tolerance',
    tag: 'Defense · Tolerance (%)',
    desc: 'Defensive tolerance attribute that directly reduces incoming Weapon Damage Boost percentage.'
  },
  multiHit: {
    title: 'Multi-Hit Multiplier',
    tag: 'Offense · Multiplier (%)',
    desc: 'Damage scaling multiplier for attacks that strike multiple times consecutively.'
  },
  multiHitChance: {
    title: 'Multi-Hit Chance',
    tag: 'Offense · Proc Chance',
    desc: 'Chance for a single attack action to strike up to 4 consecutive times, each hit dealing damage proportional to your Attack.'
  },
  multiHitResist: {
    title: 'Multi-Hit Resist',
    tag: 'Defense · Proc Resist',
    desc: 'Reduces the enemy\'s chance to trigger Multi-Hit attack chains against you.'
  },
  powerShard: {
    title: 'Power Shard',
    tag: 'Offense · Consumable Boost',
    desc: 'Consumable combat crystals equipped to amplify base weapon attack power on each landed strike.'
  },
  attackBonus: {
    title: 'Attack Bonus',
    tag: 'Offense · Flat Attack',
    desc: 'Flat numeric attack value added directly to your base attack power.'
  },
  attackIncrease: {
    title: 'Attack Increase',
    tag: 'Offense · Attack Multiplier (%)',
    desc: 'Percentage multiplier scaling your total attack rating.'
  },
  pveAttackPower: {
    title: 'PvE Attack Power',
    tag: 'Offense · PvE Flat Attack',
    desc: 'Flat attack bonus applied exclusively against open-world monsters and dungeon PvE targets.'
  },
  bossAttackPower: {
    title: 'Boss Attack Power',
    tag: 'Offense · Boss Specialist',
    desc: 'Flat attack bonus applied specifically when fighting Field Bosses, Dungeon Bosses, and World Bosses.'
  },
  speciesAttackPower: {
    title: 'Species Attack Power',
    tag: 'Offense · Genus Specialist',
    desc: 'Flat attack power applied against specific creature genera: Cogni, Fera, Natura, or Varian.'
  },
  frontBackAttackPower: {
    title: 'Front/Back Attack Power',
    tag: 'Offense · Positional Attack',
    desc: 'Flat attack bonus added whenever striking from a valid front or back flanking position.'
  },

  // Skills & Penetration
  skillCoefficient: {
    title: 'Skill Coefficient',
    tag: 'Offense · Skill Scaling (%)',
    desc: 'Primary damage percentage multiplier specified on the skill\'s tooltip (e.g. 150%, 220%, 350%). Scales with skill rank.'
  },
  skillAdditionalDamage: {
    title: 'Skill Additional Damage',
    tag: 'Offense · Flat Skill Damage',
    desc: 'Flat numeric bonus damage added directly into the skill execution formula, independent of weapon attack.'
  },
  penetration: {
    title: 'Penetration',
    tag: 'Offense · Defense Bypass',
    desc: 'Ignores a flat amount of target Defense. Critical against heavily armored bosses and tanks. Cannot reduce Defense below 0.'
  },
  defense: {
    title: 'Defense (Enemy Defense)',
    tag: 'Defense · Flat Mitigation',
    desc: 'Primary flat damage reduction stat from armor, shields, and buffs. Directly mitigates incoming physical and skill damage.'
  },
  skillSpecialtyEnhancement: {
    title: 'Skill Specialty Enhancement',
    tag: 'Offense · Specialization (%)',
    desc: 'Bonus damage multiplier unlocked through class Daevanion skill trees and Stigma enhancements.'
  },

  // Damage Boosts & Tolerances
  damageBoost: {
    title: 'Damage Boost',
    tag: 'Offense · Additive Bucket (%)',
    desc: 'Core additive percentage damage amplification bucket increasing all outgoing damage.'
  },
  damageTolerance: {
    title: 'Damage Tolerance',
    tag: 'Defense · Additive Mitigation (%)',
    desc: 'General percentage damage mitigation countering outgoing Damage Boost in the additive bucket.'
  },
  pveDamageBoost: {
    title: 'PvE Damage Boost',
    tag: 'Offense · PvE Multiplier (%)',
    desc: 'Additive percentage damage boost applied exclusively against non-player monsters and dungeon bosses.'
  },
  pveDamageTolerance: {
    title: 'PvE Damage Tolerance',
    tag: 'Defense · PvE Mitigation (%)',
    desc: 'Enemy percentage damage reduction offsetting player PvE Damage Boost.'
  },
  bossDamageBoost: {
    title: 'Boss Damage Boost',
    tag: 'Offense · Boss Multiplier (%)',
    desc: 'Crucial endgame stat providing additive percentage damage increase against Field, Dungeon, and World Bosses.'
  },
  bossDamageTolerance: {
    title: 'Boss Damage Tolerance',
    tag: 'Defense · Boss Mitigation (%)',
    desc: 'Enemy Boss percentage mitigation that directly offsets Boss Damage Boost.'
  },
  speciesDamageBoost: {
    title: 'Species Damage Boost',
    tag: 'Offense · Genus Multiplier (%)',
    desc: 'Percentage damage amplification against specific creature types: Cogni (Constructs), Fera (Beasts), Natura (Plants), Varian (Undead/Abyssal).'
  },
  speciesDamageTolerance: {
    title: 'Species Damage Tolerance',
    tag: 'Defense · Genus Mitigation (%)',
    desc: 'Mitigates incoming damage targeting specific creature genera.'
  },

  // Critical Hits
  criticalHit: {
    title: 'Critical Hit',
    tag: 'Offense · Critical Rate',
    desc: 'Increases chance to land a critical strike. Offset by target\'s Critical Hit Resist.'
  },
  criticalHitResist: {
    title: 'Critical Hit Resist',
    tag: 'Defense · Critical Prevention',
    desc: 'Directly subtracts from attacker\'s Critical Hit chance, preventing incoming critical strikes.'
  },
  criticalDamageBoost: {
    title: 'Critical Damage Boost',
    tag: 'Offense · Critical Multiplier (%)',
    desc: 'Damage multiplier applied when a critical hit lands (baseline 150%). Multiplies total damage exponentially when crit rate is high.'
  },
  criticalDamageTolerance: {
    title: 'Critical Damage Tolerance',
    tag: 'Defense · Critical Mitigation (%)',
    desc: 'Reduces incoming critical strike damage multiplier, dampening heavy burst spikes.'
  },
  critical: {
    title: 'Critical Hit State',
    tag: 'Toggle · Calculation State',
    desc: 'Toggle whether the calculated hit is considered a critical strike, applying Critical Damage Boost.'
  },
  criticalAttack: {
    title: 'Critical Attack',
    tag: 'Offense · Flat Crit Bonus',
    desc: 'Flat attack bonus added to damage only when a critical strike connects.'
  },
  criticalDamageDefense: {
    title: 'Critical Damage Defense',
    tag: 'Defense · Flat Crit Defense',
    desc: 'Flat defense value mitigating flat Critical Attack bonuses.'
  },
  pvpCriticalHit: {
    title: 'PvP Critical Hit',
    tag: 'PvP · Player Critical Rate',
    desc: 'Critical strike chance calculated specifically against other players in PvP.'
  },
  pvpCriticalHitResist: {
    title: 'PvP Critical Hit Resist',
    tag: 'PvP · Player Critical Defense',
    desc: 'Reduces enemy players\' critical strike chance in PvP combat.'
  },
  frontCriticalHit: {
    title: 'Front Attack Critical Hit',
    tag: 'Offense · Positional Critical Rate',
    desc: 'Additional critical strike chance applied only when attacking the target from the front.'
  },
  frontCriticalHitResist: {
    title: 'Front Attack Critical Hit Resist',
    tag: 'Defense · Positional Critical Defense',
    desc: 'Reduces the critical strike chance of attacks that land on your front.'
  },
  backCriticalHit: {
    title: 'Back Attack Critical Hit',
    tag: 'Offense · Positional Critical Rate',
    desc: 'Additional critical strike chance applied only when attacking the target from behind.'
  },
  backCriticalHitResist: {
    title: 'Back Attack Critical Hit Resist',
    tag: 'Defense · Positional Critical Defense',
    desc: 'Reduces the critical strike chance of attacks that land on your back.'
  },

  // Procs, Doubles & Recovery
  double: {
    title: 'Double Damage Multiplier',
    tag: 'Offense · Proc Multiplier (%)',
    desc: 'Multiplier applied when Double activates (200% in PvE, 150% in PvP). Deals massive extra burst damage.'
  },
  doubleEnabled: {
    title: 'Double Chance Proc',
    tag: 'Toggle · Calculation State',
    desc: 'Toggle whether the calculated hit triggers a Double damage proc.'
  },
  doubleChance: {
    title: 'Double Chance',
    tag: 'Offense · Proc Rate (%)',
    desc: 'Probability to trigger a Double hit on any attack, dealing greatly amplified Final Damage.'
  },
  doubleResist: {
    title: 'Double Resist',
    tag: 'Defense · Proc Resist (%)',
    desc: 'Reduces the enemy\'s Double Chance proc activation rate against you.'
  },
  perfectChance: {
    title: 'Perfect Chance',
    tag: 'Offense · Proc Rate (%)',
    desc: 'Chance to trigger a Perfect hit, maximizing damage variance roll and granting bonus impact.'
  },
  perfectResist: {
    title: 'Perfect Resist',
    tag: 'Defense · Proc Resist (%)',
    desc: 'Reduces enemy chance to land a Perfect hit.'
  },
  endurance: {
    title: 'Endurance',
    tag: 'Defense · Halving Proc (%)',
    desc: 'Defensive proc chance to activate Endurance, absorbing and halving (50% reduction) incoming strike damage.'
  },
  endurancePenetration: {
    title: 'Endurance Penetration',
    tag: 'Offense · Anti-Endurance (%)',
    desc: 'Directly lowers the target\'s Endurance activation chance.'
  },
  regeneration: {
    title: 'Regeneration',
    tag: 'Defense · HP Recovery',
    desc: 'Periodic health regeneration rate during active combat.'
  },
  regenerationPenetration: {
    title: 'Regeneration Penetration',
    tag: 'Offense · Healing Reduction',
    desc: 'Reduces effectiveness of enemy HP regeneration during combat.'
  },

  // Positional Angles
  position: {
    title: 'Attack Position',
    tag: 'Combat · Spatial Orientation',
    desc: 'Attacking from Front triggers Front Damage Boost. Attacking from Back triggers Back Damage Boost and bypasses Shield Block entirely!'
  },
  frontBoost: {
    title: 'Front Attack Damage Boost',
    tag: 'Offense · Positional (%)',
    desc: 'Positional percentage damage multiplier when striking an enemy head-on.'
  },
  frontDamageTolerance: {
    title: 'Enemy Front Damage Tolerance',
    tag: 'Defense · Positional (%)',
    desc: 'Reduces incoming damage from attacks landing on the front flank.'
  },
  backBoost: {
    title: 'Back Attack Damage Boost',
    tag: 'Offense · Positional (%)',
    desc: 'Substantial damage multiplier when striking from behind. Back attacks also bypass Shield Block!'
  },
  backDamageTolerance: {
    title: 'Enemy Back Damage Tolerance',
    tag: 'Defense · Positional (%)',
    desc: 'Reduces incoming back-attack flanking damage.'
  },
  frontAttack: {
    title: 'Front Attack',
    tag: 'Offense · Flat Front Attack',
    desc: 'Flat attack bonus added when striking an enemy from the front.'
  },
  frontDefense: {
    title: 'Front Defense',
    tag: 'Defense · Flat Front Defense',
    desc: 'Flat defense mitigating incoming frontal attacks.'
  },
  backAttack: {
    title: 'Back Attack',
    tag: 'Offense · Flat Back Attack',
    desc: 'Flat attack bonus added when striking an enemy from behind.'
  },
  backDefense: {
    title: 'Back Defense',
    tag: 'Defense · Flat Back Defense',
    desc: 'Flat defense mitigating incoming rear flanking attacks.'
  },

  // Elemental
  elementalResistance: {
    title: 'Elemental Resistance',
    tag: 'Defense · Magic Resistance',
    desc: 'Mitigates incoming non-physical magic damage across all elemental spell categories.'
  },
  fireBoost: { title: 'Fire Damage Boost', tag: 'Elemental · Fire', desc: 'Increases damage dealt by Fire-affinity skills.' },
  fireTolerance: { title: 'Fire Tolerance', tag: 'Elemental · Fire Defense', desc: 'Reduces damage received from Fire skills.' },
  waterBoost: { title: 'Water Damage Boost', tag: 'Elemental · Water', desc: 'Increases damage dealt by Water/Ice-affinity skills.' },
  waterTolerance: { title: 'Water Tolerance', tag: 'Elemental · Water Defense', desc: 'Reduces damage received from Water skills.' },
  windBoost: { title: 'Wind Damage Boost', tag: 'Elemental · Wind', desc: 'Increases damage dealt by Wind/Lightning skills.' },
  windTolerance: { title: 'Wind Tolerance', tag: 'Elemental · Wind Defense', desc: 'Reduces damage received from Wind skills.' },
  earthBoost: { title: 'Earth Damage Boost', tag: 'Elemental · Earth', desc: 'Increases damage dealt by Earth/Nature skills.' },
  earthTolerance: { title: 'Earth Tolerance', tag: 'Elemental · Earth Defense', desc: 'Reduces damage received from Earth skills.' },

  // PvP General
  pvpAttack: { title: 'PvP Attack', tag: 'PvP · Flat Attack', desc: 'Flat attack power applied only in Player-vs-Player combat.' },
  pvpDefense: { title: 'PvP Defense', tag: 'PvP · Flat Defense', desc: 'PvP defense mitigates damage received from enemy players (scales at 1% per point).' },
  pvpDamageBoost: { title: 'PvP Damage Boost', tag: 'PvP · Damage (%)', desc: 'Percentage damage amplification against enemy player characters.' },
  pvpDamageTolerance: { title: 'PvP Damage Tolerance', tag: 'PvP · Defense (%)', desc: 'Percentage damage mitigation against enemy player damage.' },

  // HP & Cadence
  playerHp: {
    title: 'Player HP',
    tag: 'Health · Survivability',
    desc: 'Your character\'s total health pool. Does not alter outgoing hit damage, but defines defensive durability.'
  },
  enemyHp: {
    title: 'Enemy / Boss HP',
    tag: 'Health · Target Capacity',
    desc: 'Total health pool of the target. Used to compute sustained Time-to-Kill (defeat time).'
  },
  hitsPerSecond: {
    title: 'Landed Hits Per Second',
    tag: 'Cadence · Sustained DPS',
    desc: 'Attacking rate per second. Multiplied by estimated damage per hit to determine sustained Damage Per Second (DPS).'
  }
};

const PT_STAT_DESCRIPTIONS = {
  pureAttack: 'Soma de Ataque de arma, guarder, acessórios, linhas de atributo, pedras e aparências antes dos multiplicadores. Não é um atributo exibido pelo jogo.',
  weaponDamageBoost: 'Multiplica o Ataque vindo de equipamentos e é reduzido pela Tolerância a Dano de Arma do alvo.',
  multiHit: 'Fator efetivo de dano usado pela fórmula para representar o resultado de Golpe Múltiplo; não é a chance exibida na ficha.',
  multiHitChance: 'Chance de gerar até quatro golpes extras. Cada golpe extra causa uma fração do dano final.',
  multiHitResist: 'Reduz a chance de o inimigo ativar Golpe Múltiplo.',
  powerShard: 'Multiplicador da munição Power Shard aplicado ao ataque de equipamento.',
  attackBonus: 'Ataque fixo somado depois do Aumento de Dano de Arma e antes do Aumento de Ataque.',
  attackIncrease: 'Multiplica Ataque Puro e Bônus de Ataque. Might contribui diretamente para este percentual.',
  skillCoefficient: 'Parcela do Ataque usada pela habilidade. O coeficiente depende da habilidade escolhida.',
  skillAdditionalDamage: 'Dano fixo da habilidade somado antes da redução por Defesa.',
  penetration: 'Subtrai Defesa do alvo na proporção 1:1, sem permitir Defesa efetiva abaixo de zero.',
  defense: 'Redução fixa: 10% do valor no PvE e 1% no PvP, depois de descontar Penetração.',
  damageTolerance: 'Reduz o efeito dos bônus de dano do atacante sobre golpes recebidos. Quanto maior o bônus adversário, mais relevante é essa mitigação; não aumenta seu dano causado.',
  evasion: 'Pode evitar completamente um golpe recebido; a Precisão do atacante combate essa chance. Importa para sobreviver, não para aumentar o dano de um golpe acertado.',
  block: 'Pode bloquear parte do dano físico recebido, mas a Precisão e a Penetração de Bloqueio do atacante combatem essa defesa. Ataques pelas costas ignoram Bloqueio.',
  criticalHitResist: 'Combate a chance de crítico do atacante, reduzindo a frequência de golpes críticos recebidos; não altera o dano de golpes normais.',
  damageBoost: 'Entra no grupo aditivo de aumentos de dano e é reduzido pelas tolerâncias correspondentes.',
  criticalDamageBoost: 'Multiplicador de um golpe crítico. A base é 1,5× no PvE e 1,25× no PvP.',
  criticalDamageTolerance: 'Reduz o multiplicador de dano crítico do atacante.',
  double: 'Multiplicador de um proc de Chance Dupla: 2× no PvE e 1,5× no PvP.',
  doubleChance: 'Chance de ativar um golpe Duplo, reduzida pela Resistência Dupla do alvo.',
  attack: 'Ataque fixo de arma, equipamentos e buffs. Alimenta o Ataque Puro antes dos multiplicadores percentuais.',
  frontBoost: 'Multiplica o dano quando o golpe acerta pela frente.',
  backBoost: 'Multiplica o dano quando o golpe acerta pelas costas; ataques traseiros também ignoram Bloqueio.',
  skillSpecialtyEnhancement: 'Multiplicador final fornecido pela especialização da habilidade.',
  playerHp: 'HP total do atacante. Não altera o dano causado.',
  enemyHp: 'HP total do alvo, usado para estimar o tempo de derrota.',
  hitsPerSecond: 'Taxa de golpes realmente acertados, multiplicada pelo dano estimado para calcular DPS.',
  blockPenetration: 'Ignora parte da chance de Bloqueio do alvo. Importa para dano físico contra alvos com escudo ou alto Bloqueio.',
  accuracy: 'Evita que golpes errem: compensa a Evasão e o Bloqueio do alvo. Um golpe errado causa zero dano, então manter Precisão acima da Evasão inimiga sustenta o DPS.',
  pveAccuracy: 'Precisão aplicada apenas contra monstros e chefes; combate a Evasão e o Bloqueio altos de chefes de fim de jogo.',
  pvpAccuracy: 'Precisão ativa apenas contra jogadores, para compensar a Evasão PvP do adversário.',
  pvpEvasion: 'Chance de esquivar de ataques de jogadores no PvP; combatida pela Precisão PvP do atacante.',
  pvpBlock: 'Chance de bloquear ou aparar ataques físicos e de habilidade de jogadores inimigos.',
  pvpBlockPenetration: 'Reduz a chance de Bloqueio de jogadores inimigos no PvP.',
  weaponDamageTolerance: 'Reduz diretamente o percentual de Aumento de Dano de Arma do atacante.',
  pveAttackPower: 'Ataque fixo aplicado apenas contra monstros e chefes de masmorra.',
  bossAttackPower: 'Ataque fixo aplicado apenas contra chefes de campo, de masmorra e mundiais.',
  speciesAttackPower: 'Ataque fixo aplicado contra o gênero específico do alvo: Cogni, Fera, Natura ou Varian.',
  frontBackAttackPower: 'Ataque fixo somado quando o golpe parte de uma posição frontal ou traseira válida.',
  pveDamageBoost: 'Aumento percentual aditivo aplicado apenas contra monstros e chefes.',
  pveDamageTolerance: 'Redução percentual do inimigo que compensa seu Aumento de Dano PvE.',
  bossDamageBoost: 'Aumento percentual aditivo contra chefes; atributo importante no fim de jogo.',
  bossDamageTolerance: 'Redução percentual de chefes que compensa diretamente o Aumento de Dano contra Chefes.',
  speciesDamageBoost: 'Aumento percentual contra um gênero de criatura: Cogni, Fera, Natura ou Varian.',
  speciesDamageTolerance: 'Redução percentual aplicada a dano direcionado a um gênero de criatura.',
  criticalHit: 'Aumenta a chance de acertar um golpe crítico; é compensada pela Resistência a Crítico do alvo.',
  criticalAttack: 'Ataque fixo somado apenas quando o golpe é crítico.',
  criticalDamageDefense: 'Defesa fixa que reduz o bônus de Ataque Crítico recebido.',
  pvpCriticalHit: 'Chance de crítico calculada apenas contra jogadores.',
  pvpCriticalHitResist: 'Reduz a chance de crítico de jogadores inimigos.',
  frontCriticalHit: 'Chance de crítico adicional apenas em ataques pela frente do alvo.',
  frontCriticalHitResist: 'Reduz a chance de crítico de golpes recebidos pela frente.',
  backCriticalHit: 'Chance de crítico adicional apenas em ataques pelas costas do alvo.',
  backCriticalHitResist: 'Reduz a chance de crítico de golpes recebidos pelas costas.',
  doubleResist: 'Reduz a chance de o inimigo ativar um golpe Duplo contra você.',
  perfectChance: 'Chance de ativar um golpe Perfeito, que maximiza a variação de dano.',
  perfectResist: 'Reduz a chance de o inimigo acertar um golpe Perfeito.',
  endurance: 'Chance de ativar Endurance e reduzir o golpe recebido à metade.',
  endurancePenetration: 'Reduz a chance de o alvo ativar Endurance.',
  regeneration: 'Recuperação periódica de HP durante o combate.',
  regenerationPenetration: 'Reduz a eficácia da regeneração de HP do inimigo.',
  frontDamageTolerance: 'Reduz o dano de golpes que acertam pela frente.',
  backDamageTolerance: 'Reduz o dano de golpes que acertam pelas costas.',
  frontAttack: 'Ataque fixo somado ao atacar o alvo pela frente.',
  frontDefense: 'Defesa fixa contra ataques frontais recebidos.',
  backAttack: 'Ataque fixo somado ao atacar o alvo pelas costas.',
  backDefense: 'Defesa fixa contra ataques recebidos pelas costas.',
  elementalResistance: 'Reduz o dano mágico recebido de todas as categorias elementais.',
  fireBoost: 'Aumenta o dano de habilidades de Fogo.',
  fireTolerance: 'Reduz o dano recebido de habilidades de Fogo.',
  waterBoost: 'Aumenta o dano de habilidades de Água/Gelo.',
  waterTolerance: 'Reduz o dano recebido de habilidades de Água.',
  windBoost: 'Aumenta o dano de habilidades de Vento/Raio.',
  windTolerance: 'Reduz o dano recebido de habilidades de Vento.',
  earthBoost: 'Aumenta o dano de habilidades de Terra/Natureza.',
  earthTolerance: 'Reduz o dano recebido de habilidades de Terra.',
  pvpAttack: 'Ataque fixo aplicado apenas contra jogadores.',
  pvpDefense: 'Defesa aplicada ao dano de jogadores inimigos: 1% de redução por ponto.',
  pvpDamageBoost: 'Aumento percentual de dano contra jogadores inimigos.',
  pvpDamageTolerance: 'Redução percentual do dano causado por jogadores inimigos.'
};

function localizedStatInfo(name, info, genus = '') {
  if (typeof I18N === 'undefined' || I18N.currentLang !== 'pt') return info;
  const defensive = /Tolerance|Resist|Defense|Evasion|Block|Endurance|Regeneration/i.test(info.title);
  const description = PT_STAT_DESCRIPTIONS[name]
    || (defensive
      ? 'Reduz, evita ou neutraliza o efeito ofensivo correspondente conforme as regras de combate do AION 2.'
      : 'Aumenta ou modifica o efeito ofensivo correspondente conforme as regras de combate do AION 2.');
  return {
    title: I18N.statLabel(info.title),
    tag: defensive ? 'Defesa · atributo de combate' : 'Ofensiva · atributo de combate',
    desc: genus ? `Específico do gênero ${genus}. ${description}` : description
  };
}

function getStatInfo(name) {
  if (!name) return null;
  if (STAT_CATALOG[name]) return localizedStatInfo(name, STAT_CATALOG[name]);

  // Check genus prefixes: cogni, fera, natura, varian
  const genusMatch = name.match(/^(cogni|fera|natura|varian)(.+)$/i);
  if (genusMatch) {
    const genus = genusMatch[1].charAt(0).toUpperCase() + genusMatch[1].slice(1).toLowerCase();
    const baseName = genusMatch[2].charAt(0).toLowerCase() + genusMatch[2].slice(1);
    const baseInfo = STAT_CATALOG[baseName] || STAT_CATALOG[baseName.replace(/^[a-z]/, c => c.toLowerCase())];
    if (baseInfo) {
      return localizedStatInfo(baseName, {
        title: `${genus} ${baseInfo.title.replace(/\s*\(.*\)$/, '')}`,
        tag: `Genus (${genus}) · ${baseInfo.tag}`,
        desc: `Specific to ${genus} creature genus. ${baseInfo.desc}`
      }, genus);
    }
  }

  // Fallback pattern matching
  for (const key of Object.keys(STAT_CATALOG)) {
    if (name.toLowerCase() === key.toLowerCase()) return localizedStatInfo(key, STAT_CATALOG[key]);
  }
  return null;
}

function createStatTooltipElement(info) {
  const trigger = document.createElement('span');
  trigger.className = 'stat-tip-trigger';
  trigger.setAttribute('tabindex', '0');
  trigger.setAttribute('role', 'button');
  trigger.setAttribute('aria-label', `${typeof I18N !== 'undefined' && I18N.currentLang === 'pt' ? 'Informações sobre' : 'Information about'} ${info.title}`);

  const icon = document.createElement('span');
  icon.className = 'stat-tip-icon';
  icon.textContent = 'ⓘ';
  trigger.appendChild(icon);

  const box = document.createElement('div');
  box.className = 'stat-tooltip-box';
  box.setAttribute('role', 'tooltip');

  const title = document.createElement('div');
  title.className = 'stat-tooltip-title';
  title.textContent = info.title;

  const tag = document.createElement('span');
  tag.className = 'stat-tooltip-tag';
  tag.textContent = info.tag;

  const desc = document.createElement('p');
  desc.className = 'stat-tooltip-desc';
  desc.textContent = info.desc;

  box.append(title, tag, desc);
  trigger.appendChild(box);

  // Toggle on click/tap for accessibility
  trigger.addEventListener('click', (e) => {
    e.stopPropagation();
    const isOpen = trigger.classList.contains('open');
    closeAllTooltips();
    if (!isOpen) trigger.classList.add('open');
  });

  return trigger;
}

function closeAllTooltips() {
  if (typeof document === 'undefined') return;
  document.querySelectorAll('.stat-tip-trigger.open').forEach(el => el.classList.remove('open'));
}

function attachStatTooltips(root) {
  if (typeof document === 'undefined') return;
  const container = root || document;
  const labels = container.querySelectorAll('label');
  for (const label of labels) {
    if (label.querySelector('.stat-tip-trigger')) continue;
    const input = label.querySelector('input, select');
    if (!input || (!input.name && !input.id)) continue;
    const key = input.name || input.id;
    const info = getStatInfo(key);
    if (info) {
      label.appendChild(createStatTooltipElement(info));
    }
  }
}

if (typeof document !== 'undefined') {
  document.addEventListener('click', closeAllTooltips);
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') closeAllTooltips();
  });

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => attachStatTooltips());
  } else {
    attachStatTooltips();
  }
  if (typeof I18N !== 'undefined') {
    I18N.onLanguageChange(() => {
      document.querySelectorAll('.stat-tip-trigger').forEach(node => node.remove());
      attachStatTooltips();
    });
  }
}

if (typeof module !== 'undefined') {
  module.exports = { STAT_CATALOG, getStatInfo, attachStatTooltips };
}
