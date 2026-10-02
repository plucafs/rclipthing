use egui::{Key, Slider};
use crate::actions;
use crate::model::AppState;
use crate::persist;

const SETTINGS_PATH: &str = "settings.json";

fn term_chips(ui: &mut egui::Ui, terms: &mut Vec<String>, symbol: &str, color: egui::Color32) {
    let mut remove = None;
    for (i, term) in terms.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.colored_label(color, format!("{symbol} {term}"));
            if ui.small_button("x").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        terms.remove(i);
    }
}

pub fn options_bar(ui: &mut egui::Ui, state: &mut AppState) {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label("Results:");
        changed |= ui
            .add(Slider::new(&mut state.settings.top_n, 10..=200).text(""))
            .changed();

        ui.separator();
        ui.label("+ (add):");
        let add_input = ui.add_sized(
            [110.0, 22.0],
            egui::TextEdit::singleline(&mut state.new_add_term)
                .id(crate::ui::add_term_input_id())
                .hint_text("concept"),
        );
        let add_enter = add_input.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if add_enter && !state.new_add_term.trim().is_empty() {
            state
                .add_terms
                .push(state.new_add_term.trim().to_string());
            state.new_add_term.clear();
        }

        ui.separator();
        ui.label("- (exclude):");
        let sub_input = ui.add_sized(
            [110.0, 22.0],
            egui::TextEdit::singleline(&mut state.new_sub_term)
                .id(crate::ui::sub_term_input_id())
                .hint_text("concept"),
        );
        let sub_enter = sub_input.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if sub_enter && !state.new_sub_term.trim().is_empty() {
            state
                .sub_terms
                .push(state.new_sub_term.trim().to_string());
            state.new_sub_term.clear();
        }

        ui.separator();
        if ui.button("Reindex").clicked() {
            actions::start_index(state);
        }
    });

    ui.horizontal(|ui| {
        if !state.add_terms.is_empty() {
            term_chips(ui, &mut state.add_terms, "+", egui::Color32::LIGHT_GREEN);
        }
        if !state.sub_terms.is_empty() {
            if !state.add_terms.is_empty() {
                ui.separator();
            }
            term_chips(ui, &mut state.sub_terms, "-", egui::Color32::LIGHT_RED);
        }
    });

    if changed {
        persist::save_settings(SETTINGS_PATH, &state.settings);
    }
}