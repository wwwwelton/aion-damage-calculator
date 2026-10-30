use crate::i18n::{t, Lang};
use crate::state::{ActiveTab, AppState, ALL_TABS};
use leptos::prelude::*;

#[component]
pub fn Header(state: AppState) -> impl IntoView {
    let lang = state.lang;
    let active_tab = state.tab;

    view! {
        <header>
            <div class="header-row">
                <div>
                    <p class="eyebrow">{move || t(lang.get(), "appEyebrow")}</p>
                    <h1>{move || t(lang.get(), "appTitle")}</h1>
                </div>
                <label class="language-selector">
                    {move || t(lang.get(), "langLabel")}
                    " "
                    <select
                        id="lang-select"
                        aria-label="Language"
                        prop:value=move || lang.get().as_str()
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            let next = if val == "pt" { Lang::Pt } else { Lang::En };
                            #[cfg(target_arch = "wasm32")]
                            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                                let _ = storage.set_item("aion2_calculator_lang", next.as_str());
                            }
                            #[cfg(target_arch = "wasm32")]
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                doc.set_title(&t(next, "windowTitle"));
                            }
                            lang.set(next);
                        }
                    >
                        <option value="pt">"Português"</option>
                        <option value="en">"English"</option>
                    </select>
                </label>
            </div>
            <p>{move || t(lang.get(), "appSubtitle")}</p>
        </header>
        <nav class="tabs" role="tablist" aria-label="Application tabs">
            {ALL_TABS.into_iter().map(|tab: ActiveTab| {
                let is_active = move || active_tab.get() == tab;
                view! {
                    <button
                        type="button"
                        id=tab.tab_btn_id()
                        data-tab=tab.id()
                        role="tab"
                        aria-controls=tab.id()
                        class=move || if is_active() { "tab active" } else { "tab" }
                        aria-selected=move || if is_active() { "true" } else { "false" }
                        on:click=move |_| active_tab.set(tab)
                    >
                        {move || t(lang.get(), tab.i18n_key())}
                    </button>
                }
            }).collect_view()}
        </nav>
    }
}
