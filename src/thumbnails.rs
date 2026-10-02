use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use eframe::egui::ColorImage;

use crate::model::WorkerMsg;

pub struct ThumbLoader {
    tx: Sender<WorkerMsg>,
    abort: Arc<AtomicBool>,
}

impl ThumbLoader {
    pub fn new(tx: Sender<WorkerMsg>) -> Self {
        Self {
            tx,
            abort: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&mut self) {
        self.abort.store(true, Ordering::Relaxed);
    }

    pub fn start(&mut self, paths: Vec<PathBuf>, max_px: u32) {
        if paths.is_empty() {
            return;
        }
        let abort = Arc::new(AtomicBool::new(false));
        self.abort = abort.clone();
        let next = Arc::new(AtomicUsize::new(0));
        let workers = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(8);

        for _ in 0..workers {
            let tx = self.tx.clone();
            let abort = abort.clone();
            let next = next.clone();
            let paths = paths.clone();
            std::thread::spawn(move || loop {
                if abort.load(Ordering::Relaxed) {
                    break;
                }
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= paths.len() {
                    break;
                }
                let p = &paths[i];
                if let Some(img) = load_thumb(p, max_px) {
                    let _ = tx.send(WorkerMsg::ThumbReady(p.clone(), img));
                }
            });
        }
    }
}

fn load_thumb(path: &PathBuf, max_px: u32) -> Option<ColorImage> {
    let img = image::open(path).ok()?;
    let thumb = img.thumbnail(max_px, max_px);
    let rgba = thumb.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels = rgba.into_raw();
    Some(ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        &pixels,
    ))
}