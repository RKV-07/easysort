use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovedPair {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub timestamp: i64,
    pub moves: Vec<MovedPair>,
}

#[derive(Debug, Default)]
pub struct UndoStack {
    pub batches: Vec<Batch>,
}

impl UndoStack {
    pub fn push(&mut self, batch: Batch) {
        self.batches.push(batch);
    }
    pub fn pop(&mut self) -> Option<Batch> {
        self.batches.pop()
    }
    pub fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }
}

pub fn undo_batch(batch: &Batch) -> Vec<(PathBuf, String)> {
    let mut failures = Vec::new();
    // reverse order
    for pair in batch.moves.iter().rev() {
        // pair: from=original src, to=current location
        if let Err(e) = std::fs::rename(&pair.to, &pair.from) {
            failures.push((pair.to.clone(), e.to_string()));
        }
    }
    failures
}

pub fn append_manifest(path: &Path, batch: &Batch) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(batch).map_err(std::io::Error::other)?;
    writeln!(f, "{}", line)?;
    Ok(())
}
