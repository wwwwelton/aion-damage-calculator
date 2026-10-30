pub mod companion;
pub mod i18n;
pub mod header;
pub mod state;
#[cfg(target_arch = "wasm32")]
mod dom;
#[cfg(target_arch = "wasm32")]
pub use dom::mount_game_companion;

use aion2_core::calculator::{apply_target_mode, calculate_damage, TargetMode, Values};
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[component]
fn App() -> impl IntoView {
    let (pure_attack, set_pure_attack) = signal(1000.0_f64);
    let (damage_boost, set_damage_boost) = signal(1.0_f64);
    let (defense, set_defense) = signal(0.0_f64);
    let (mode, set_mode) = signal(TargetMode::Boss);

    let damage = move || {
        let values = Values {
            pure_attack: pure_attack.get(),
            damage_boost: damage_boost.get(),
            defense: defense.get(),
            ..Values::default()
        };
        calculate_damage(&apply_target_mode(&values, mode.get()))
    };

    view! {
        <main>
            <label>
                "Ataque puro "
                <input
                    type="number"
                    prop:value=move || pure_attack.get().to_string()
                    on:input=move |ev| set_pure_attack.set(event_target_value(&ev).parse().unwrap_or(0.0))
                />
            </label>
            <label>
                "Multiplicador de dano "
                <input
                    type="number"
                    step="0.01"
                    prop:value=move || damage_boost.get().to_string()
                    on:input=move |ev| set_damage_boost.set(event_target_value(&ev).parse().unwrap_or(0.0))
                />
            </label>
            <label>
                "Defesa "
                <input
                    type="number"
                    prop:value=move || defense.get().to_string()
                    on:input=move |ev| set_defense.set(event_target_value(&ev).parse().unwrap_or(0.0))
                />
            </label>
            <label>
                "Alvo "
                <select on:change=move |ev| set_mode.set(match event_target_value(&ev).as_str() {
                    "player" => TargetMode::Player,
                    "monster" => TargetMode::Monster,
                    _ => TargetMode::Boss,
                })>
                    <option value="boss">"Boss"</option>
                    <option value="monster">"Monstro"</option>
                    <option value="player">"Jogador"</option>
                </select>
            </label>
            <output>{move || format!("Dano: {:.1}", damage())}</output>
        </main>
    }
}

#[wasm_bindgen]
pub fn mount_calculator() {
    mount_to_body(App);
}
