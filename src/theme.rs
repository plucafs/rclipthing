use egui::{Color32, Stroke, Visuals};
use crate::model::Theme;

fn accent(theme: &Theme) -> Color32 {
    match theme {
        Theme::Dark => Color32::from_rgb(62, 130, 220),
        Theme::Light => Color32::from_rgb(38, 100, 190),
    }
}

pub fn apply_theme(ctx: &egui::Context, theme: &Theme) {
    let mut visuals = match theme {
        Theme::Dark => Visuals::dark(),
        Theme::Light => Visuals::light(),
    };
    let accent = accent(theme);

    visuals.hyperlink_color = accent;
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = Stroke::new(1.0_f32, accent);
    visuals.widgets.hovered.bg_stroke.color = accent;
    visuals.widgets.active.bg_stroke.color = accent;

    ctx.set_visuals(visuals);
}
