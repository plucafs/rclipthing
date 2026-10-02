mod actions;
mod model;
mod persist;
mod rclip;
mod theme;
mod thumbnails;
mod ui;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

use eframe::egui::{self, TextureOptions};
use model::{AppState, WorkerMsg, WorkerStatus};
use theme::apply_theme;
use thumbnails::ThumbLoader;
use ui::options_bar::options_bar;
use ui::results_grid::results_grid;
use ui::search_bar::search_bar;
use ui::settings_section::settings_section;
use ui::status_bar::status_bar;

struct RclipApp {
    state: AppState,
    loader: ThumbLoader,
    rx: Receiver<WorkerMsg>,
    last_thumb_size: f32,
    last_max_px: u32,
    focus_search: bool,
}

impl RclipApp {
    fn handle_messages(&mut self, ctx: &egui::Context) {
        let mut got = false;
        while let Ok(msg) = self.rx.try_recv() {
            got = true;
            match msg {
                WorkerMsg::SearchDone(items) => {
                    self.state.results = items;
                    self.state.selected = None;
                    let n = self.state.results.len();
                    self.state.status = WorkerStatus::Done;
                    self.state.status_text = if n == 0 {
                        "No results".to_string()
                    } else {
                        format!("{n} results")
                    };
                    self.prepare_thumbnails();
                }
                WorkerMsg::SearchError(e) => {
                    self.state.results.clear();
                    self.state.selected = None;
                    self.state.status = WorkerStatus::Done;
                    self.state.status_text = e;
                }
                WorkerMsg::IndexDone => {
                    self.state.status = WorkerStatus::Done;
                    self.state.status_text =
                        "Indexing complete. Try the search again.".to_string();
                }
                WorkerMsg::ThumbReady(path, img) => {
                    let tex = ctx.load_texture(path.to_string_lossy(), img, TextureOptions::LINEAR);
                    self.state.thumbnails.insert(path, tex);
                }
            }
        }
        if got {
            ctx.request_repaint();
        }
        if self.state.status != WorkerStatus::Working
            && let Some(folders) = self.state.pending_index.take()
        {
            actions::start_index_folders(&mut self.state, folders);
            ctx.request_repaint();
        }
    }

    fn prepare_thumbnails(&mut self) {
        let keep: HashSet<&PathBuf> = self.state.results.iter().map(|r| &r.path).collect();
        self.state.thumbnails.retain(|p, _| keep.contains(p));

        let max_px = 256.max(self.state.settings.thumb_size as u32);
        let stale = max_px > self.last_max_px;
        let need: Vec<PathBuf> = self
            .state
            .results
            .iter()
            .filter(|r| stale || !self.state.thumbnails.contains_key(&r.path))
            .map(|r| r.path.clone())
            .collect();
        self.last_max_px = max_px;
        self.loader.cancel();
        self.loader.start(need, max_px);
    }
}

impl eframe::App for RclipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let focus_before = ctx.memory(|m| m.focused());
        apply_theme(ctx, &self.state.settings.theme);
        ctx.set_pixels_per_point(self.state.settings.ui_scale);

        self.handle_messages(ctx);

        if (self.state.settings.thumb_size - self.last_thumb_size).abs() > f32::EPSILON {
            self.last_thumb_size = self.state.settings.thumb_size;
            self.prepare_thumbnails();
        }

        egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close();
                    }
                });
            });
        });
        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            status_bar(ui, &mut self.state);
        });
        egui::TopBottomPanel::top("search_panel").show(ctx, |ui| {
            search_bar(ui, &mut self.state);
        });
        egui::TopBottomPanel::top("options_panel").show(ctx, |ui| {
            options_bar(ui, &mut self.state);
            egui::CollapsingHeader::new("Settings")
                .default_open(false)
                .show(ui, |ui| settings_section(ui, &mut self.state));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            results_grid(ui, &mut self.state, focus_before);
        });

        if self.state.status == WorkerStatus::Working {
            ctx.request_repaint();
        }

        if self.focus_search {
            self.focus_search = false;
            ctx.memory_mut(|m| m.request_focus(ui::search_input_id()));
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let settings = persist::load_settings(&persist::settings_path());

    let (tx, rx): (Sender<WorkerMsg>, Receiver<WorkerMsg>) = channel();

    let state = AppState {
        settings,
        query: String::new(),
        add_terms: Vec::new(),
        sub_terms: Vec::new(),
        results: Vec::new(),
        thumbnails: Default::default(),
        status: WorkerStatus::Idle,
        status_text: String::new(),
        selected: None,
        pending_index: None,
        new_add_term: String::new(),
        new_sub_term: String::new(),
        new_folder: String::new(),
        worker_tx: Some(tx.clone()),
    };

    let loader = ThumbLoader::new(tx);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 720.0]),
        ..Default::default()
    };

    let last_thumb_size = state.settings.thumb_size;
    let last_max_px = 256.max(state.settings.thumb_size as u32);

    eframe::run_native(
        "rclipthing",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(RclipApp {
                state,
                loader,
                rx,
                last_thumb_size,
                last_max_px,
                focus_search: true,
            }))
        }),
    )
}