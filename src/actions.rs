use crate::model::{AppState, WorkerStatus};
use crate::rclip::{spawn_run, RunRequest};

pub fn start_search(state: &mut AppState) {
    if state.status == WorkerStatus::Working {
        return;
    }
    let query = state.query.trim().to_string();
    if query.is_empty() && state.add_terms.is_empty() && state.sub_terms.is_empty() {
        state.status_text = "Type a query".to_string();
        return;
    }
    let Some(tx) = state.worker_tx.clone() else {
        return;
    };
    let n = state.settings.folders.len();
    let rq = RunRequest {
        folders: state.settings.folders.clone(),
        query,
        add_terms: state.add_terms.clone(),
        sub_terms: state.sub_terms.clone(),
        top_n: state.settings.top_n,
        index: false,
    };
    state.status = WorkerStatus::Working;
    state.status_text = if n > 1 {
        format!("Searching in {n} folders...")
    } else {
        "Searching...".to_string()
    };
    spawn_run(rq, tx);
}

pub fn start_index(state: &mut AppState) {
    let folders = state.settings.folders.clone();
    start_index_folders(state, folders);
}

pub fn start_index_folders(state: &mut AppState, folders: Vec<String>) {
    if state.status == WorkerStatus::Working || folders.is_empty() {
        return;
    }
    let Some(tx) = state.worker_tx.clone() else {
        return;
    };
    let rq = RunRequest {
        folders,
        query: String::new(),
        add_terms: Vec::new(),
        sub_terms: Vec::new(),
        top_n: state.settings.top_n,
        index: true,
    };
    state.status = WorkerStatus::Working;
    state.status_text = if rq.folders.len() == 1 {
        format!("Indexing {}...", rq.folders[0])
    } else {
        "Indexing in progress...".to_string()
    };
    spawn_run(rq, tx);
}

pub fn clear_query(state: &mut AppState) {
    state.query.clear();
    state.results.clear();
    state.selected = None;
    state.status_text.clear();
    state.status = WorkerStatus::Idle;
}