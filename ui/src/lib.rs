pub mod calculator;
pub mod calculator_form;
pub mod header;
pub mod i18n;
pub mod state;

use calculator::CalculatorTab;
use header::Header;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use state::AppState;
use wasm_bindgen::prelude::*;

#[component]
fn App() -> impl IntoView {
    let state = AppState::new();
    view! {
        <Header state=state />
        <main>
            <CalculatorTab state=state />
        </main>
    }
}

#[wasm_bindgen]
pub fn mount_calculator() {
    mount_to_body(App);
}
