use egui::{Key, Slider};
use std::path::Path;
use crate::model::{AppState, Theme};
use crate::persist;

pub fn settings_section(ui: &mut egui::Ui, state: &mut AppState) {
    let mut changed = false;

    egui::Grid::new("settings_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("Dark theme");
            let mut is_dark = state.settings.theme == Theme::Dark;
            if ui.checkbox(&mut is_dark, "").changed() {
                state.settings.theme = if is_dark { Theme::Dark } else { Theme::Light };
                changed = true;
            }
            ui.end_row();

            ui.label("UI scale");
            changed |= ui
                .add(Slider::new(&mut state.settings.ui_scale, 1.0..=2.0))
                .changed();
            ui.end_row();

            ui.label("Thumbnail size");
            changed |= ui
                .add(Slider::new(&mut state.settings.thumb_size, 80.0..=320.0))
                .changed();
            ui.end_row();
        });

    ui.add_space(6.0);
    ui.label("Search folders (rclip searches each one):");

    let mut remove = None;
    for i in 0..state.settings.folders.len() {
        let mut path_buf = state.settings.folders[i].clone();
        let ok = Path::new(&path_buf).is_dir();

        ui.horizontal(|ui| {
            if ok {
                ui.colored_label(egui::Color32::LIGHT_GREEN, "OK");
            } else {
                ui.colored_label(egui::Color32::LIGHT_RED, "not found");
            }
            let edited = ui.add(
                egui::TextEdit::singleline(&mut path_buf).id(crate::ui::folder_edit_id(i)),
            );
            if edited.changed() {
                state.settings.folders[i] = path_buf.trim().to_string();
                changed = true;
            }
            if ui.small_button("x").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        state.settings.folders.remove(i);
        changed = true;
    }

    // let input = ui.add(
    //         egui::TextEdit::singleline(&mut state.query)
    //             .id(crate::ui::search_input_id())
    //             .hint_text("Search photos semantically...")
    //             .font(egui::TextStyle::Body),
    //     );

    ui.horizontal(|ui| {
        let input = ui.add(
            egui::TextEdit::singleline(&mut state.new_folder)
                .id(crate::ui::folder_add_id())
                .hint_text("/path/to/folder"),
        );
        let enter = input.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if ui.button("Add").clicked() || enter {
            let f = state.new_folder.trim().to_string();
            if !f.is_empty() && !state.settings.folders.contains(&f) {
                state.settings.folders.push(f.clone());
                state.new_folder.clear();
                changed = true;

                if Path::new(&f).is_dir() {
                    if state.status == crate::model::WorkerStatus::Working {
                        let pending = state.pending_index.get_or_insert_with(Vec::new);
                        if !pending.contains(&f) {
                            pending.push(f);
                        }
                    } else {
                        crate::actions::start_index_folders(state, vec![f]);
                    }
                } else if state.status != crate::model::WorkerStatus::Working {
                    state.status = crate::model::WorkerStatus::Done;
                    state.status_text = format!("Folder not found: {f}");
                }
            }
        }
    });

    if changed {
        persist::save_settings(&persist::settings_path(), &state.settings);
    }
}
