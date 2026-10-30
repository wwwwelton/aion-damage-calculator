use crate::{host, i18n::{t,format_t}, state::AppState};
use aion2_core::character_extraction::extract_character;
use leptos::{prelude::*, task::spawn_local};
use serde_json::{json,Value};

fn key(item: &Value) -> String { format!("{}|{}|{}",item["globalRegion"].as_str().unwrap_or("naw"),item["serverId"],item["characterId"].as_str().unwrap_or("")) }

#[component]
pub fn Profiles(state: AppState, profile: RwSignal<Option<Value>>, selected: RwSignal<Option<Value>>, global_region: RwSignal<String>) -> impl IntoView {
    let saved = RwSignal::new(Vec::<Value>::new());
    let status = RwSignal::new(String::new());
    let enabled = RwSignal::new(true);
    let interval = RwSignal::new(600u64);
    let roster = RwSignal::new(None::<Value>);
    let roster_status = RwSignal::new(String::new());
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    let initial = storage.as_ref().and_then(|s| s.get_item("aion2Favorites").ok().flatten()).and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok()).unwrap_or_default();
    let favorites = RwSignal::new(initial.into_iter().take(20).collect::<Vec<_>>());
    let refresh_saved = move || spawn_local(async move {
        match host::request("saved_profiles","ww","",json!({})).await {
            Ok(data) => saved.set(data.as_array().cloned().unwrap_or_default()),
            Err(e) => status.set(e),
        }
    });
    refresh_saved();
    spawn_local(async move {
        match host::request("sync_settings","ww","",json!({})).await {
            Ok(settings) => { enabled.set(settings["enabled"].as_bool().unwrap_or(true)); interval.set(settings["interval"].as_u64().unwrap_or(600)); }
            Err(e) => status.set(e),
        }
    });
    window_event_listener(leptos::ev::Custom::<web_sys::CustomEvent>::new("aion2-profile-sync"),move |event| {
        let Ok(text) = js_sys::JSON::stringify(&event.detail()) else { return };
        let Some(text) = text.as_string() else { return };
        let Ok(update) = serde_json::from_str::<Value>(&text) else { return };
        if let Some(item) = selected.get_untracked() {
            if key(&item)==key(&update) {
                if let Some(error) = update["error"].as_str() { status.set(format_t(state.lang.get_untracked(),"profileSyncFailed",&[("error",error)])); }
                else if update["data"].is_object() { profile.set(Some(update["data"].clone())); }
            }
        }
        refresh_saved();
    });
    let favorite = move |_| {
        let Some(item) = selected.get_untracked() else { return };
        let id = key(&item);
        let mut next = favorites.get_untracked();
        if next.contains(&id) { next.retain(|value| value != &id); } else { next.push(id); if next.len()>20 { next.remove(0); } }
        match storage.as_ref().ok_or_else(|| "Browser storage unavailable".to_string()).and_then(|s| s.set_item("aion2Favorites", &serde_json::to_string(&next).unwrap()).map_err(|_| "Could not save favorites".to_string())) {
            Ok(()) => favorites.set(next), Err(e) => status.set(e),
        }
    };
    let refresh = move |_| {
        let Some(item) = selected.get_untracked() else { return };
        spawn_local(async move {
            let gr = item["globalRegion"].as_str().unwrap_or("naw");
            match host::request("refresh_profile","ww",gr,json!({"serverId":item["serverId"],"characterId":item["characterId"]})).await {
                Ok(data) => { profile.set(Some(data)); refresh_saved(); }, Err(e) => status.set(e),
            }
        });
    };
    let configure = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            match host::request("sync_settings","ww","",json!({"enabled":enabled.get_untracked(),"interval":interval.get_untracked()})).await {
                Ok(_) => status.set(t(state.lang.get_untracked(),"profileLoaded")), Err(e) => status.set(e),
            }
        });
    };
    let legion = move |_| {
        let Some(data) = profile.get_untracked() else { return };
        let view = extract_character(&data);
        let server_id = selected.get_untracked().map(|i| i["serverId"].clone()).unwrap_or(Value::Null);
        roster_status.set(format!("Legion: {}",view.legion));
        spawn_local(async move {
            match host::request("legion_roster","ww","",json!({"serverId":server_id,"serverName":view.server,"legionName":view.legion})).await {
                Ok(data) if data["members"].as_array().is_some_and(|a| !a.is_empty()) => roster.set(Some(data)),
                result => {
                    let members = saved.get_untracked().into_iter().filter_map(|entry| {
                        let member = extract_character(&entry["profile"]);
                        (member.legion.eq_ignore_ascii_case(&view.legion) && member.server==view.server).then(|| json!({"nickname":member.name,"job":member.class_name,"combat_power2":member.combat_power,"combat_power":member.gear_score,"server":member.server}))
                    }).collect::<Vec<_>>();
                    if let Err(e)=result { roster_status.set(format!("{e} · Membros locais: {}",members.len())); }
                    roster.set(Some(json!({"members":members})));
                }
            }
        });
    };
    view! {
        <section class="card">
            <form on:submit=configure>
                <label><input type="checkbox" prop:checked=move || enabled.get() on:change=move |ev| enabled.set(event_target_checked(&ev)) />{move || t(state.lang.get(),"syncEnabled")}</label>
                <label>{move || t(state.lang.get(),"syncInterval")}<input type="number" min="600" max="3600" prop:value=move || interval.get().to_string() on:input=move |ev| { if let Ok(value)=event_target_value(&ev).parse() { interval.set(value); } } /></label>
                <button type="submit">{move || t(state.lang.get(),"syncSave")}</button>
            </form>
            <p role="status">{move || status.get()}</p>
            <div id="saved-profiles">{move || {
                let mut entries = saved.get(); let favorites = favorites.get();
                entries.sort_by(|a,b| favorites.contains(&key(b)).cmp(&favorites.contains(&key(a))).then_with(|| b["fetchedAt"].as_u64().cmp(&a["fetchedAt"].as_u64())));
                entries.into_iter().map(|entry| {
                    let view = extract_character(&entry["profile"]);
                    let label = format!("{}{} · {}",if favorites.contains(&key(&entry)) { "★ " } else { "" },view.name,view.server);
                    view! { <button type="button" on:click=move |_| {
                        global_region.set(entry["globalRegion"].as_str().unwrap_or("naw").to_owned());
                        selected.set(Some(entry.clone())); profile.set(Some(entry["profile"].clone()));
                    }>{label}</button> }
                }).collect_view()
            }}</div>
            <div hidden=move || profile.get().is_none()>
                <button type="button" aria-pressed=move || selected.get().is_some_and(|i| favorites.get().contains(&key(&i))).to_string() on:click=favorite>"★"</button>
                <button type="button" on:click=refresh>{move || t(state.lang.get(),"refreshProfile")}</button>
                <button type="button" on:click=legion>{move || t(state.lang.get(),"legion")}</button>
            </div>
            <section hidden=move || roster.get().is_none()><h3>{move || roster_status.get()}</h3>{move || roster.get().map(|data| {
                let members = data["members"].as_array().cloned().unwrap_or_default();
                view! { <ul>{members.into_iter().map(|m| view! { <li>{format!("{} · {} · CP {} · GS {}",m["nickname"].as_str().unwrap_or(""),m["job"].as_str().unwrap_or(""),m["combat_power2"],m["combat_power"])}</li> }).collect_view()}</ul> }
            })}</section>
        </section>
    }
}
