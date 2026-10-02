use egui::{Align, Layout};
use crate::model::{AppState, WorkerStatus};
use crate::persist;

fn bump_thumb_size(state: &mut AppState, delta: f32) {
    let v = (state.settings.thumb_size + delta).clamp(80.0, 320.0);
    if (v - state.settings.thumb_size).abs() > f32::EPSILON {
        state.settings.thumb_size = v;
        persist::save_settings(&persist::settings_path(), &state.settings);
    }
}

pub fn status_bar(ui: &mut egui::Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        match state.status {
            WorkerStatus::Working => {
                ui.spinner();
                ui.label(&state.status_text);
            }
            WorkerStatus::Done => {
                let color = if state.results.is_empty() {
                    egui::Color32::GRAY
                } else {
                    egui::Color32::LIGHT_GREEN
                };
                ui.colored_label(color, &state.status_text);
            }
            WorkerStatus::Idle => {
                if !state.status_text.is_empty() {
                    ui.colored_label(egui::Color32::YELLOW, &state.status_text);
                }
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.small_button("+").clicked() {
                bump_thumb_size(state, 20.0);
            }
            if ui.small_button("-").clicked() {
                bump_thumb_size(state, -20.0);
            }
            ui.label(format!("{} px", state.settings.thumb_size.round()));
            if !state.results.is_empty() {
                ui.separator();
                ui.label(format!("{} results", state.results.len()));
            }
        });
    });
}
