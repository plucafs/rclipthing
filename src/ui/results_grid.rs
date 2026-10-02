use std::path::Path;
use std::process::Command;

use egui::{Align2, FontId, Key, Pos2, Rect, RichText, Sense, Stroke, StrokeKind};
use crate::actions;
use crate::model::{AppState, ResultItem};

fn open_with_xdg(what: &Path) {
    let what = what.to_path_buf();
    std::thread::spawn(move || {
        if let Ok(mut c) = Command::new("xdg-open").arg(&what).spawn() {
            let _ = c.wait();
        }
    });
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

fn caption(ui: &mut egui::Ui, rect: Rect, item: &ResultItem) {
    let painter = ui.painter_at(rect);
    let band_h = 20.0;
    let band = Rect::from_min_max(
        Pos2::new(rect.left(), rect.bottom() - band_h),
        rect.right_bottom(),
    );
    painter.rect_filled(
        band,
        egui::CornerRadius {
            nw: 0,
            ne: 0,
            sw: 5,
            se: 5,
        },
        egui::Color32::from_black_alpha(170),
    );

    let font = FontId::proportional(11.0);
    let text_color = egui::Color32::from_gray(235);
    let score_str = format!("{:.3}", item.score);
    let score_w = painter
        .layout_no_wrap(score_str.clone(), font.clone(), text_color)
        .size()
        .x;

    let name = item
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| item.path.to_string_lossy().to_string());
    let room = (band.width() - score_w - 18.0).max(0.0);
    let max_chars = (room / 6.0) as usize;
    let name = truncate(&name, max_chars.max(4));

    painter.text(
        Pos2::new(band.left() + 6.0, band.bottom() - 4.0),
        Align2::LEFT_BOTTOM,
        name,
        font.clone(),
        text_color,
    );
    painter.text(
        Pos2::new(band.right() - 6.0, band.bottom() - 4.0),
        Align2::RIGHT_BOTTOM,
        score_str,
        font,
        text_color,
    );
}

fn cell(
    ui: &mut egui::Ui,
    state: &mut AppState,
    item: &ResultItem,
    index: usize,
    size: f32,
) -> Rect {
    let (id, rect) = ui.allocate_space(egui::vec2(size, size));
    let resp = ui.interact(rect, id, Sense::CLICK);

    let is_selected = state.selected == Some(index);
    let active = resp.hovered() || is_selected;

    let visuals = ui.visuals();
    let painter = ui.painter_at(rect);
    let corner = 6.0;
    let bg = if active {
        visuals.widgets.hovered.weak_bg_fill
    } else {
        visuals.extreme_bg_color
    };
    painter.rect_filled(rect, corner, bg);
    let stroke = if active {
        Stroke::new(1.5_f32, visuals.widgets.hovered.bg_stroke.color)
    } else {
        Stroke::new(1.0_f32, visuals.widgets.inactive.bg_stroke.color)
    };
    painter.rect_stroke(rect, corner, stroke, StrokeKind::Inside);

    if let Some(tex) = state.thumbnails.get(&item.path) {
        let ts = tex.size_vec2();
        let inner = rect.shrink(6.0);
        let scale = (inner.width() / ts.x).min(inner.height() / ts.y).min(1.0);
        let draw = ts * scale;
        let centered = Rect::from_center_size(rect.center(), draw);
        egui::Image::from_texture((tex.id(), tex.size_vec2()))
            .corner_radius(4.0)
            .paint_at(ui, centered);
    } else {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "...",
            FontId::proportional(size * 0.3),
            egui::Color32::from_gray(90),
        );
    }

    if active {
        caption(ui, rect, item);
    }

    let resp = resp.on_hover_text(item.path.to_string_lossy().to_string());

    let _ = resp.context_menu(|ui| {
        let path = item.path.clone();
        if ui.button("Open image").clicked() {
            open_with_xdg(&path);
            ui.close();
        }
        if ui.button("Copy path").clicked() {
            ui.ctx().copy_text(path.to_string_lossy().to_string());
            ui.close();
        }
        if ui.button("Open folder").clicked() {
            if let Some(parent) = path.parent() {
                open_with_xdg(parent);
            }
            ui.close();
        }
        if ui.button("Use as image query (+)").clicked() {
            state.add_terms.push(path.to_string_lossy().to_string());
            actions::start_search(state);
            ui.close();
        }
    });

    if resp.clicked() {
        state.selected = Some(index);
    }
    if resp.double_clicked() {
        state.selected = Some(index);
        open_with_xdg(&item.path);
    }

    rect
}

/// Returns true when the selection moved this frame (so the grid can scroll to it).
fn handle_keys(
    ui: &egui::Ui,
    state: &mut AppState,
    cols: usize,
    focus_before: Option<egui::Id>,
) -> bool {
    let n = state.results.len();
    if n == 0 {
        return false;
    }
    if focus_before.is_some() || ui.memory(|m| m.focused()).is_some() {
        return false;
    }

    let mut dx = 0isize;
    let mut dy = 0isize;
    let mut clear = false;
    let mut open = false;
    ui.input(|i| {
        if i.key_pressed(Key::ArrowRight) {
            dx = 1;
        }
        if i.key_pressed(Key::ArrowLeft) {
            dx = -1;
        }
        if i.key_pressed(Key::ArrowDown) {
            dy = 1;
        }
        if i.key_pressed(Key::ArrowUp) {
            dy = -1;
        }
        clear = i.key_pressed(Key::Escape);
        open = i.events.iter().any(|e| {
            matches!(
                e,
                egui::Event::Key {
                    key: Key::Enter,
                    pressed: true,
                    repeat: false,
                    ..
                }
            )
        });
    });

    if clear {
        if state.selected.is_some() {
            state.selected = None;
            return true;
        }
        return false;
    }
    if open {
        if let Some(idx) = state.selected
            && let Some(item) = state.results.get(idx)
        {
            let path = item.path.clone();
            open_with_xdg(&path);
        }
        return false;
    }
    if dx == 0 && dy == 0 {
        return false;
    }

    let cols = cols.max(1) as isize;
    let cur = state.selected.map(|s| s as isize).unwrap_or(0);
    let next = if dy != 0 {
        cur + dy * cols + dx
    } else {
        cur + dx
    };
    let next = next.clamp(0, n as isize - 1) as usize;
    if state.selected != Some(next) {
        state.selected = Some(next);
        return true;
    }
    false
}

pub fn results_grid(ui: &mut egui::Ui, state: &mut AppState, focus_before: Option<egui::Id>) {
    if state.results.is_empty() {
        if state.status_text.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("No search yet. Type a query above.")
                        .color(egui::Color32::GRAY),
                );
            });
        }
        return;
    }

    let size = state.settings.thumb_size;
    let spacing = 10.0;
    let avail_w = ui.available_width();
    let cols = ((avail_w + spacing) / (size + spacing)).floor().max(1.0) as usize;

    if state.selected.is_some_and(|s| s >= state.results.len()) {
        state.selected = None;
    }

    let moved = handle_keys(ui, state, cols, focus_before);

    let results = state.results.clone();
    let mut scroll_to: Option<Rect> = None;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let mut i = 0;
            while i < results.len() {
                let end = (i + cols).min(results.len());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = spacing;
                    for (k, item) in results[i..end].iter().enumerate() {
                        let index = i + k;
                        let rect = cell(ui, state, item, index, size);
                        if state.selected == Some(index) {
                            scroll_to = Some(rect);
                        }
                    }
                });
                ui.add_space(6.0);
                i = end;
            }
            if moved
                && let Some(r) = scroll_to
            {
                ui.scroll_to_rect(r, None);
            }
        });
}
