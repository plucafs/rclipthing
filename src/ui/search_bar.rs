use egui::Key;
use crate::actions;
use crate::model::AppState;

pub fn search_bar(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        
        let input = ui.add(
            egui::TextEdit::singleline(&mut state.query)
                .id(crate::ui::search_input_id())
                .hint_text("Search photos semantically...")
                .font(egui::TextStyle::Body),
        );

        let enter_pressed = input.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if ui.button("Search").clicked() || enter_pressed {
            actions::start_search(state);
        }
        if ui.button("Clear").clicked() {
            actions::clear_query(state);
        }
    });
}
