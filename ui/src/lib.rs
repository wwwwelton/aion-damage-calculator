pub mod boss;
#[cfg(target_arch = "wasm32")]
pub mod character;
#[cfg(target_arch = "wasm32")]
pub mod enemy;
#[cfg(target_arch = "wasm32")]
pub mod host;
pub mod calculator;
pub mod calculator_form;
pub mod header;
pub mod i18n;
pub mod state;

use boss::BossTab;
#[cfg(target_arch = "wasm32")]
use character::CharacterTab;
#[cfg(target_arch = "wasm32")]
use enemy::EnemyTab;
use calculator::CalculatorTab;
use header::Header;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use state::{ActiveTab, AppState};
use wasm_bindgen::prelude::*;

#[component]
fn App() -> impl IntoView {
    let state = AppState::new();
    // Panels stay mounted and toggle `hidden`, so inputs survive tab switches.
    view! {
        <Header state=state />
        <main>
            <div hidden=move || state.tab.get() != ActiveTab::Calculator>
                <CalculatorTab state=state />
            </div>
            <div hidden=move || state.tab.get() != ActiveTab::Boss>
                <BossTab state=state />
            </div>
            <div hidden=move || state.tab.get() != ActiveTab::Character>
                <CharacterTab state=state />
            </div>
            <div hidden=move || state.tab.get() != ActiveTab::Enemy>
                <EnemyTab state=state />
            </div>
        </main>
    }
}

#[wasm_bindgen]
pub fn mount_calculator() {
    mount_to_body(App);
}
