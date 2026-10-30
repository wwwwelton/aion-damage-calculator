pub use crate::controller::character_search::*;

pub fn init_characters() {
    init_character_search();
    crate::controller::enemy::init_enemy_search();
}
