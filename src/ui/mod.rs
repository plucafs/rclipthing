pub mod options_bar;
pub mod results_grid;
pub mod search_bar;
pub mod settings_section;
pub mod status_bar;

use egui::Id;

pub fn search_input_id() -> Id {
    Id::new("search_input")
}

pub fn add_term_input_id() -> Id {
    Id::new("new_add_term")
}

pub fn sub_term_input_id() -> Id {
    Id::new("new_sub_term")
}

pub fn folder_edit_id(i: usize) -> Id {
    Id::new(("folder_edit", i))
}

pub fn folder_add_id() -> Id {
    Id::new("folder_add")
}
