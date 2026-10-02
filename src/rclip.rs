use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

use crate::model::{ResultItem, WorkerMsg};

const SEARCH_TIMEOUT_SECS: u64 = 90;
const INDEX_TIMEOUT_SECS: u64 = 900;

pub struct RunRequest {
    pub folders: Vec<String>,
    pub query: String,
    pub add_terms: Vec<String>,
    pub sub_terms: Vec<String>,
    pub top_n: usize,
    pub index: bool,
}

pub fn spawn_run(rq: RunRequest, tx: Sender<WorkerMsg>) {
    std::thread::spawn(move || run(&rq, &tx));
}

fn run(rq: &RunRequest, tx: &Sender<WorkerMsg>) {
    if rq.index {
        for folder in &rq.folders {
            if !Path::new(folder).is_dir() {
                let _ = tx.send(WorkerMsg::SearchError(format!("Folder not found: {folder}")));
                continue;
            }
            if let Err(e) = invoke(folder, rq, true) {
                let _ = tx.send(WorkerMsg::SearchError(format!("Indexing {folder} failed: {e}")));
            }
        }
        let _ = tx.send(WorkerMsg::IndexDone);
        return;
    }

    let mut merged: Vec<ResultItem> = Vec::new();
    let mut error: Option<String> = None;
    for folder in &rq.folders {
        if !Path::new(folder).is_dir() {
            error = Some(format!("Folder not found: {folder}"));
            continue;
        }
        match invoke(folder, rq, false) {
            Ok(mut items) => {
                for it in items.drain(..) {
                    if !merged.iter().any(|m| m.path == it.path) {
                        merged.push(it);
                    }
                }
            }
            Err(e) => error = Some(e),
        }
    }

    if merged.is_empty() {
        if let Some(e) = error {
            let _ = tx.send(WorkerMsg::SearchError(e));
        } else {
            let _ = tx.send(WorkerMsg::SearchDone(Vec::new()));
        }
        return;
    }
    let _ = tx.send(WorkerMsg::SearchDone(merged));
}

fn effective_query(query: &str) -> &str {
    let q = query.trim();
    if q.is_empty() {
        "."
    } else {
        q
    }
}

fn invoke(folder: &str, rq: &RunRequest, index: bool) -> Result<Vec<ResultItem>, String> {
    let query = effective_query(&rq.query);

    let mut cmd = Command::new("rclip");
    if !index {
        cmd.arg("--no-indexing");
    }
    cmd.arg("-t");
    let top = if index { "1".to_string() } else { rq.top_n.to_string() };
    cmd.arg(&top);
    for a in &rq.add_terms {
        cmd.arg("-a").arg(a);
    }
    for s in &rq.sub_terms {
        cmd.arg("-s").arg(s);
    }
    cmd.arg(query);
    cmd.current_dir(folder);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to run rclip: {e}"))?;

    let timeout = if index {
        INDEX_TIMEOUT_SECS
    } else {
        SEARCH_TIMEOUT_SECS
    };
    let started = std::time::Instant::now();
    loop {
        match child.try_wait().map_err(|e| format!("rclip: {e}"))? {
            Some(status) => {
                let mut out = String::new();
                let mut err = String::new();
                if let Some(mut o) = child.stdout.take() {
                    let _ = o.read_to_string(&mut out);
                }
                if let Some(mut e) = child.stderr.take() {
                    let _ = e.read_to_string(&mut err);
                }
                return finish(&out, &err, status.success());
            }
            None => {
                if started.elapsed().as_secs() > timeout {
                    let _ = child.kill();
                    return Err("rclip: timeout".to_string());
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
}

fn finish(out: &str, err: &str, success: bool) -> Result<Vec<ResultItem>, String> {
    if !success {
        return Err(format!("rclip exited with an error: {}", err.trim()));
    }
    let mut items = Vec::new();
    for (i, line) in out.lines().enumerate() {
        if i == 0 {
            continue;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (score_s, path_s) = line
            .split_once('\t')
            .ok_or_else(|| format!("rclip: unexpected output line: {line}"))?;
        let score: f32 = score_s
            .trim()
            .parse()
            .map_err(|_| format!("rclip: invalid score: {score_s}"))?;
        let path = strip_quotes(path_s.trim());
        items.push(ResultItem {
            path: PathBuf::from(path),
            score,
        });
    }
    Ok(items)
}

fn strip_quotes(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() >= 2 && b[0] == b'"' && b[b.len() - 1] == b'"' {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output() {
        let out = "score\tfilepath\n0.264\t\"/a/b.jpg\"\n0.236\t\"/x\\ y/c.png\"\n";
        let items = finish(out, "", true).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].path, PathBuf::from("/a/b.jpg"));
        assert_eq!(items[0].score, 0.264);
        assert_eq!(items[1].path, PathBuf::from("/x\\ y/c.png"));
    }

    #[test]
    fn test_parse_header_only() {
        let out = "score\tfilepath\n";
        let items = finish(out, "", true).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("\"/a\""), "/a");
        assert_eq!(strip_quotes("/a"), "/a");
    }

    #[test]
    fn test_effective_query_rejects_blank() {
        assert_eq!(effective_query(""), ".");
        assert_eq!(effective_query("   "), ".");
        assert_eq!(effective_query("  cat  "), "cat");
        assert_eq!(effective_query("."), ".");
    }
}