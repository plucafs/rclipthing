use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui::{ColorImage, TextureHandle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "light")]
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: Theme,
    pub ui_scale: f32,
    pub folders: Vec<String>,
    pub thumb_size: f32,
    pub top_n: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            ui_scale: 1.30,
            folders: vec!["./images".to_string()],
            thumb_size: 200.0,
            top_n: 50,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResultItem {
    pub path: PathBuf,
    pub score: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkerStatus {
    Idle,
    Working,
    Done,
}

pub struct AppState {
    pub settings: Settings,
    pub query: String,
    pub add_terms: Vec<String>,
    pub sub_terms: Vec<String>,
    pub results: Vec<ResultItem>,
    pub thumbnails: HashMap<PathBuf, TextureHandle>,
    pub status: WorkerStatus,
    pub status_text: String,
    pub selected: Option<usize>,
    pub pending_index: Option<Vec<String>>,
    pub new_add_term: String,
    pub new_sub_term: String,
    pub new_folder: String,
    pub worker_tx: Option<std::sync::mpsc::Sender<WorkerMsg>>,
}

#[derive(Debug)]
pub enum WorkerMsg {
    SearchDone(Vec<ResultItem>),
    SearchError(String),
    IndexDone,
    ThumbReady(PathBuf, ColorImage),
}