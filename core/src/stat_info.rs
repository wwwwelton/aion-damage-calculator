// Port of getStatInfo/localizedStatInfo from stat-tooltips.js. Catalog data is shared with the JS build.

use serde_json::Value;
use std::sync::LazyLock;

static CATALOG: LazyLock<Value> = LazyLock::new(|| {
    let root: Value = serde_json::from_str(include_str!("../../ui/data/stat-catalog.json"))
        .expect("ui/data/stat-catalog.json is valid JSON");
    root["catalog"].clone()
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatInfo {
    pub title: String,
    pub tag: String,
    pub desc: String,
}

/// Returns title, tag and description for a stat key; `pt` selects Portuguese descriptions.
/// Portuguese titles are not localized yet: the English catalog title is kept.
pub fn stat_info(name: &str, pt: bool) -> Option<StatInfo> {
    if name.is_empty() {
        return None;
    }
    if let Some(base) = catalog_entry(name) {
        return Some(localized(name, &base.0, &base.1, &base.2, pt, ""));
    }
    if let Some((genus, base_key)) = split_genus(name) {
        if let Some((title, tag, desc)) = catalog_entry(&base_key) {
            let short = strip_trailing_parens(&title);
            let genus_title = format!("{genus} {short}");
            let genus_tag = format!("Genus ({genus}) · {tag}");
            let genus_desc = format!("Specific to {genus} creature genus. {desc}");
            return Some(localized(&base_key, &genus_title, &genus_tag, &genus_desc, pt, &genus));
        }
    }
    let lower = name.to_lowercase();
    let key = CATALOG.as_object()?.keys().find(|k| k.to_lowercase() == lower)?;
    let (title, tag, desc) = catalog_entry(key)?;
    Some(localized(key, &title, &tag, &desc, pt, ""))
}

fn catalog_entry(key: &str) -> Option<(String, String, String)> {
    let entry = CATALOG.get(key)?;
    Some((
        entry["title"].as_str()?.to_string(),
        entry["tag"].as_str()?.to_string(),
        entry["desc"].as_str()?.to_string(),
    ))
}

/// `cogniBlock` -> ("Cogni", "block"), matching `^(cogni|fera|natura|varian)(.+)$` case-insensitively.
fn split_genus(name: &str) -> Option<(String, String)> {
    let lower = name.to_lowercase();
    for genus in ["cogni", "fera", "natura", "varian"] {
        if lower.starts_with(genus) && name.len() > genus.len() {
            let rest = &name[genus.len()..];
            let mut chars = rest.chars();
            let first = chars.next()?.to_lowercase().collect::<String>();
            let base = format!("{first}{}", chars.as_str());
            let mut g = genus.chars();
            let cap = g.next()?.to_uppercase().collect::<String>() + g.as_str();
            return Some((cap, base));
        }
    }
    None
}

/// Mirrors `title.replace(/\s*\(.*\)$/, '')`.
fn strip_trailing_parens(title: &str) -> String {
    match title.find('(') {
        Some(idx) if title.ends_with(')') => title[..idx].trim_end().to_string(),
        _ => title.to_string(),
    }
}

fn localized(key: &str, title: &str, tag: &str, desc: &str, pt: bool, genus: &str) -> StatInfo {
    if !pt {
        return StatInfo { title: title.into(), tag: tag.into(), desc: desc.into() };
    }
    let lower = title.to_lowercase();
    let defensive = ["tolerance", "resist", "defense", "evasion", "block", "endurance", "regeneration"]
        .iter()
        .any(|w| lower.contains(w));
    let base = PT_DESCRIPTIONS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, d)| d.to_string())
        .unwrap_or_else(|| {
            if defensive {
                "Reduz, evita ou neutraliza o efeito ofensivo correspondente conforme as regras de combate do AION 2.".into()
            } else {
                "Aumenta ou modifica o efeito ofensivo correspondente conforme as regras de combate do AION 2.".into()
            }
        });
    StatInfo {
        title: title.into(),
        tag: if defensive { "Defesa · atributo de combate".into() } else { "Ofensiva · atributo de combate".into() },
        desc: if genus.is_empty() { base } else { format!("Específico do gênero {genus}. {base}") },
    }
}

static PT_DESCRIPTIONS: &[(&str, &str)] = &[
    ("pureAttack", "Soma de Ataque de arma, guarder, acessórios, linhas de atributo, pedras e aparências antes dos multiplicadores. Não é um atributo exibido pelo jogo."),
    ("weaponDamageBoost", "Multiplica o Ataque vindo de equipamentos e é reduzido pela Tolerância a Dano de Arma do alvo."),
    ("multiHit", "Fator efetivo de dano usado pela fórmula para representar o resultado de Golpe Múltiplo; não é a chance exibida na ficha."),
    ("multiHitChance", "Chance de gerar até quatro golpes extras. Cada golpe extra causa uma fração do dano final."),
    ("multiHitResist", "Reduz a chance de o inimigo ativar Golpe Múltiplo."),
    ("powerShard", "Multiplicador da munição Power Shard aplicado ao ataque de equipamento."),
    ("attackBonus", "Ataque fixo somado depois do Aumento de Dano de Arma e antes do Aumento de Ataque."),
    ("attackIncrease", "Multiplica Ataque Puro e Bônus de Ataque. Might contribui diretamente para este percentual."),
    ("skillCoefficient", "Parcela do Ataque usada pela habilidade. O coeficiente depende da habilidade escolhida."),
    ("skillAdditionalDamage", "Dano fixo da habilidade somado antes da redução por Defesa."),
    ("penetration", "Subtrai Defesa do alvo na proporção 1:1, sem permitir Defesa efetiva abaixo de zero."),
    ("defense", "Redução fixa: 10% do valor no PvE e 1% no PvP, depois de descontar Penetração."),
    ("damageTolerance", "Reduz o efeito dos bônus de dano do atacante sobre golpes recebidos. Quanto maior o bônus adversário, mais relevante é essa mitigação; não aumenta seu dano causado."),
    ("evasion", "Pode evitar completamente um golpe recebido; a Precisão do atacante combate essa chance. Importa para sobreviver, não para aumentar o dano de um golpe acertado."),
    ("block", "Pode bloquear parte do dano físico recebido, mas a Precisão e a Penetração de Bloqueio do atacante combatem essa defesa. Ataques pelas costas ignoram Bloqueio."),
    ("criticalHitResist", "Combate a chance de crítico do atacante, reduzindo a frequência de golpes críticos recebidos; não altera o dano de golpes normais."),
    ("damageBoost", "Entra no grupo aditivo de aumentos de dano e é reduzido pelas tolerâncias correspondentes."),
    ("criticalDamageBoost", "Multiplicador de um golpe crítico. A base é 1,5× no PvE e 1,25× no PvP."),
    ("criticalDamageTolerance", "Reduz o multiplicador de dano crítico do atacante."),
    ("double", "Multiplicador de um proc de Chance Dupla: 2× no PvE e 1,5× no PvP."),
    ("doubleChance", "Chance de ativar um golpe Duplo, reduzida pela Resistência Dupla do alvo."),
    ("attack", "Ataque fixo de arma, equipamentos e buffs. Alimenta o Ataque Puro antes dos multiplicadores percentuais."),
    ("frontBoost", "Multiplica o dano quando o golpe acerta pela frente."),
    ("backBoost", "Multiplica o dano quando o golpe acerta pelas costas; ataques traseiros também ignoram Bloqueio."),
    ("skillSpecialtyEnhancement", "Multiplicador final fornecido pela especialização da habilidade."),
    ("playerHp", "HP total do atacante. Não altera o dano causado."),
    ("enemyHp", "HP total do alvo, usado para estimar o tempo de derrota."),
    ("hitsPerSecond", "Taxa de golpes realmente acertados, multiplicada pelo dano estimado para calcular DPS."),
    ("blockPenetration", "Ignora parte da chance de Bloqueio do alvo. Importa para dano físico contra alvos com escudo ou alto Bloqueio."),
    ("accuracy", "Evita que golpes errem: compensa a Evasão e o Bloqueio do alvo. Um golpe errado causa zero dano, então manter Precisão acima da Evasão inimiga sustenta o DPS."),
    ("pveAccuracy", "Precisão aplicada apenas contra monstros e chefes; combate a Evasão e o Bloqueio altos de chefes de fim de jogo."),
    ("pvpAccuracy", "Precisão ativa apenas contra jogadores, para compensar a Evasão PvP do adversário."),
    ("pvpEvasion", "Chance de esquivar de ataques de jogadores no PvP; combatida pela Precisão PvP do atacante."),
    ("pvpBlock", "Chance de bloquear ou aparar ataques físicos e de habilidade de jogadores inimigos."),
    ("pvpBlockPenetration", "Reduz a chance de Bloqueio de jogadores inimigos no PvP."),
    ("weaponDamageTolerance", "Reduz diretamente o percentual de Aumento de Dano de Arma do atacante."),
    ("pveAttackPower", "Ataque fixo aplicado apenas contra monstros e chefes de masmorra."),
    ("bossAttackPower", "Ataque fixo aplicado apenas contra chefes de campo, de masmorra e mundiais."),
    ("speciesAttackPower", "Ataque fixo aplicado contra o gênero específico do alvo: Cogni, Fera, Natura ou Varian."),
    ("frontBackAttackPower", "Ataque fixo somado quando o golpe parte de uma posição frontal ou traseira válida."),
    ("pveDamageBoost", "Aumento percentual aditivo aplicado apenas contra monstros e chefes."),
    ("pveDamageTolerance", "Redução percentual do inimigo que compensa seu Aumento de Dano PvE."),
    ("bossDamageBoost", "Aumento percentual aditivo contra chefes; atributo importante no fim de jogo."),
    ("bossDamageTolerance", "Redução percentual de chefes que compensa diretamente o Aumento de Dano contra Chefes."),
    ("speciesDamageBoost", "Aumento percentual contra um gênero de criatura: Cogni, Fera, Natura ou Varian."),
    ("speciesDamageTolerance", "Redução percentual aplicada a dano direcionado a um gênero de criatura."),
    ("criticalHit", "Aumenta a chance de acertar um golpe crítico; é compensada pela Resistência a Crítico do alvo."),
    ("criticalAttack", "Ataque fixo somado apenas quando o golpe é crítico."),
    ("criticalDamageDefense", "Defesa fixa que reduz o bônus de Ataque Crítico recebido."),
    ("pvpCriticalHit", "Chance de crítico calculada apenas contra jogadores."),
    ("pvpCriticalHitResist", "Reduz a chance de crítico de jogadores inimigos."),
    ("frontCriticalHit", "Chance de crítico adicional apenas em ataques pela frente do alvo."),
    ("frontCriticalHitResist", "Reduz a chance de crítico de golpes recebidos pela frente."),
    ("backCriticalHit", "Chance de crítico adicional apenas em ataques pelas costas do alvo."),
    ("backCriticalHitResist", "Reduz a chance de crítico de golpes recebidos pelas costas."),
    ("doubleResist", "Reduz a chance de o inimigo ativar um golpe Duplo contra você."),
    ("perfectChance", "Chance de ativar um golpe Perfeito, que maximiza a variação de dano."),
    ("perfectResist", "Reduz a chance de o inimigo acertar um golpe Perfeito."),
    ("endurance", "Chance de ativar Endurance e reduzir o golpe recebido à metade."),
    ("endurancePenetration", "Reduz a chance de o alvo ativar Endurance."),
    ("regeneration", "Recuperação periódica de HP durante o combate."),
    ("regenerationPenetration", "Reduz a eficácia da regeneração de HP do inimigo."),
    ("frontDamageTolerance", "Reduz o dano de golpes que acertam pela frente."),
    ("backDamageTolerance", "Reduz o dano de golpes que acertam pelas costas."),
    ("frontAttack", "Ataque fixo somado ao atacar o alvo pela frente."),
    ("frontDefense", "Defesa fixa contra ataques frontais recebidos."),
    ("backAttack", "Ataque fixo somado ao atacar o alvo pelas costas."),
    ("backDefense", "Defesa fixa contra ataques recebidos pelas costas."),
    ("elementalResistance", "Reduz o dano mágico recebido de todas as categorias elementais."),
    ("fireBoost", "Aumenta o dano de habilidades de Fogo."),
    ("fireTolerance", "Reduz o dano recebido de habilidades de Fogo."),
    ("waterBoost", "Aumenta o dano de habilidades de Água/Gelo."),
    ("waterTolerance", "Reduz o dano recebido de habilidades de Água."),
    ("windBoost", "Aumenta o dano de habilidades de Vento/Raio."),
    ("windTolerance", "Reduz o dano recebido de habilidades de Vento."),
    ("earthBoost", "Aumenta o dano de habilidades de Terra/Natureza."),
    ("earthTolerance", "Reduz o dano recebido de habilidades de Terra."),
    ("pvpAttack", "Ataque fixo aplicado apenas contra jogadores."),
    ("pvpDefense", "Defesa aplicada ao dano de jogadores inimigos: 1% de redução por ponto."),
    ("pvpDamageBoost", "Aumento percentual de dano contra jogadores inimigos."),
    ("pvpDamageTolerance", "Redução percentual do dano causado por jogadores inimigos."),
];

#[cfg(test)]
mod tests {
    use super::stat_info;

    #[test]
    fn english_catalog_and_genus_variant() {
        let block = stat_info("block", false).unwrap();
        assert!(block.desc.contains("Parry"));
        assert_eq!(stat_info("cogniBlock", false).unwrap().title, "Cogni Block");
        assert!(stat_info("cogniBlock", false).unwrap().desc.starts_with("Specific to Cogni"));
        assert!(stat_info("penetration", false).unwrap().desc.contains("Defense"));
    }

    #[test]
    fn portuguese_description_and_defensive_tag() {
        let block = stat_info("block", true).unwrap();
        assert!(block.desc.starts_with("Pode bloquear"));
        assert_eq!(block.tag, "Defesa · atributo de combate");
        let unknown_defensive = stat_info("cogniBlock", true).unwrap();
        assert!(unknown_defensive.desc.starts_with("Específico do gênero Cogni. Pode bloquear"));
    }

    #[test]
    fn unknown_and_empty_names_return_none() {
        assert!(stat_info("", false).is_none());
        assert!(stat_info("notAStat", false).is_none());
    }
}
