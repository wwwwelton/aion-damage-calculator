use crate::{i18n::{t, stat_label, Lang}, state::AppState};
use aion2_core::character_extraction::{extract_character, option_name};
use leptos::prelude::*;
use serde_json::Value;

fn text(v: &Value, key: &str) -> String {
    v.get(key).filter(|v| !v.is_null()).map(|v| v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string())).unwrap_or_default()
}
fn entries(v: &Value, key: &str) -> Vec<Value> { v[key].as_array().cloned().unwrap_or_default() }
fn image(url: String) -> impl IntoView {
    let safe = url.starts_with("https://") || url.starts_with("assets/");
    view! { <img class="profile-icon" src=if safe { url } else { String::new() } alt="" hidden=!safe loading="lazy" /> }
}
fn stats(items: Vec<Value>, lang: Lang) -> impl IntoView {
    view! { <ul class="option-list">{items.into_iter().filter(|v| !text(v,"name").is_empty()).map(|stat| {
        let sheet = option_name(&stat).is_none();
        let value = if stat["value"].is_null() { text(&stat,"desc") } else { text(&stat,"value") };
        let extra = text(&stat,"extra");
        view! { <li class:sheet-total=sheet><span>{stat_label(lang,&text(&stat,"name"))}</span><strong>{value}</strong>
            <small hidden=extra.is_empty()>{format!("+{extra} {}",t(lang,"bonusSuffix"))}</small>
            <small hidden=!sheet class="badge badge-display">{t(lang,"sheetTotalIncluded")}</small>
        </li> }
    }).collect_view()}</ul> }
}
fn gear(pair: Value, lang: Lang) -> impl IntoView {
    let item = &pair["item"];
    let detail = &pair["detail"];
    let name = if text(item,"name").is_empty() { text(detail,"name") } else { text(item,"name") };
    let icon = if text(item,"icon").is_empty() { text(detail,"icon") } else { text(item,"icon") };
    let blocks = [("baseOptions","mainStats"),("soulBind","subStats"),("manastones","magicStoneStat"),("theostones","godStoneStat")].into_iter().map(|(label,key)| {
        let items = entries(detail,key);
        let empty = items.is_empty();
        view! { <div hidden=empty class="gear-options"><h4>{t(lang,label)}</h4>{stats(items,lang)}</div> }
    }).collect_view();
    let skills = entries(detail,"subSkills").into_iter().map(|skill| view! { <li>{image(text(&skill,"icon"))}<span>{text(&skill,"name")}</span><strong>{format!("+{}",text(&skill,"level"))}</strong></li> }).collect_view();
    view! { <article class="gear-card"><div class="gear-card-head">{image(icon)}<div><strong>{name}</strong><small>{format!("{} · {} · +{} · exceed {}",text(item,"slotPosName"),text(item,"grade"),text(item,"enchantLevel"),text(item,"exceedLevel"))}</small></div></div>{blocks}<ul class="option-list">{skills}</ul></article> }
}

#[component]
pub fn Dossier(state: AppState, data: Value) -> impl IntoView {
    let tab = RwSignal::new(0usize);
    view! { <div class="profile-dossier">{move || {
        let lang = state.lang.get();
        let view = extract_character(&data);
        let equipment = view.gear.into_iter().map(|item| gear(item,lang)).collect_view();
        let arcana = view.arcana.into_iter().map(|item| gear(item,lang)).collect_view();
        let skills = [ ("Active","activeSkills"), ("Passive","passiveSkills"), ("Dp","stigmaSkills") ].into_iter().map(|(category,key)| {
            let rows = view.skills.iter().filter(|s| text(s,"category")==category).map(|skill| {
                let acquired = text(skill,"acquired").parse::<f64>().unwrap_or(0.0)>0.0 && text(skill,"skillLevel").parse::<f64>().unwrap_or(0.0)>0.0;
                let equipped = text(skill,"equip").parse::<f64>().unwrap_or(0.0)>0.0;
                let label = t(lang, if !acquired { "skillNotAcquired" } else if equipped { "equipped" } else { "skillAcquired" });
                view! { <article class="skill-card">{image(text(skill,"icon"))}<strong>{text(skill,"name")}</strong><small>{format!("{label} · {} {}",t(lang,"level"),text(skill,"skillLevel"))}</small></article> }
            }).collect_view();
            view! { <h3>{t(lang,key)}</h3><div class="skill-grid">{rows}</div> }
        }).collect_view();
        let attributes = view.stats.into_iter().map(|stat| {
            let effects = entries(&stat,"statSecondList").into_iter().map(|effect| view! { <small>{stat_label(lang,effect.as_str().unwrap_or(""))}</small> }).collect_view();
            view! { <article class="attribute-card"><span>{stat_label(lang,&text(&stat,"name"))}</span><strong>{text(&stat,"value")}</strong>{effects}</article> }
        }).collect_view();
        let titles = view.titles.into_iter().map(|title| {
            let effects = entries(&title,"equipStatList").into_iter().chain(entries(&title,"statList")).map(|stat| view! { <p>{text(&stat,"desc")}</p> }).collect_view();
            view! { <article class="progression-card"><strong>{text(&title,"name")}</strong><small>{format!("{} · {}",text(&title,"equipCategory"),text(&title,"grade"))}</small>{effects}</article> }
        }).collect_view();
        let boards = view.boards.into_iter().map(|board| view! { <article class="progression-card">{image(text(&board,"icon"))}<strong>{text(&board,"name")}</strong><small>{format!("{} / {}",text(&board,"openNodeCount"),text(&board,"totalNodeCount"))}</small></article> }).collect_view();
        let companions = [view.pet, view.wing].into_iter().flatten().map(|item| view! { <article class="progression-card">{image(text(&item,"icon"))}<strong>{text(&item,"name")}</strong><small>{format!("{} · {} · +{}",text(&item,"level"),text(&item,"grade"),text(&item,"enchantLevel"))}</small></article> }).collect_view();
        let ranking = view.ranking.into_iter().flat_map(|rank| [("rankOverall","overall_rank","overall_total","overall_percentile"),("rankClass","job_rank","job_total","job_percentile"),("rankServer","server_rank","server_total","server_percentile")].into_iter().filter_map(move |(scope,r,total,p)| {
            let percentile = text(&rank,p).parse::<f64>().ok().filter(|v| v.is_finite())?;
            let r = text(&rank,r); let total = text(&rank,total);
            if r.is_empty() || total.is_empty() { return None; }
            Some(format!("{}: {r} / {total} · {percentile:.2}%",t(lang,scope)))
        })).map(|line| view! { <small>{line}</small> }).collect_view();
        view! {
            <div class="dossier-summary">{image(view.portrait)}<div><strong>{format!("{} · {}",view.name,view.class_name)}</strong><small>{format!("{} · {} · {} · {}",view.level,view.race,view.gender,view.server)}</small><small>{format!("CP {} · GS {}",view.combat_power.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),view.gear_score.map(|v| v.to_string()).unwrap_or_else(|| "—".into()))}</small><small>{view.title_name}</small>{ranking}</div></div>
            <div class="dossier-tabs" role="tablist">{["tabEquipment","tabSkills","tabAttributes","tabProgression"].into_iter().enumerate().map(|(i,key)| view! { <button type="button" role="tab" aria-selected=move || (tab.get()==i).to_string() on:click=move |_| tab.set(i)>{t(lang,key)}</button> }).collect_view()}</div>
            <section role="tabpanel" hidden=move || tab.get()!=0><div class="gear-grid">{equipment}</div><div class="progression-grid">{companions}</div></section>
            <section role="tabpanel" hidden=move || tab.get()!=1>{skills}</section>
            <section role="tabpanel" hidden=move || tab.get()!=2><p>{t(lang,"noInferredTotals")}</p><div class="attribute-grid">{attributes}</div></section>
            <section role="tabpanel" hidden=move || tab.get()!=3><h3>{t(lang,"equippedTitles")}</h3><div class="progression-grid">{titles}</div><h3>{t(lang,"daevanionBoards")}</h3><div class="progression-grid">{boards}</div><h3>{t(lang,"arcana")}</h3><div class="gear-grid">{arcana}</div></section>
        }
    }}</div> }
}
