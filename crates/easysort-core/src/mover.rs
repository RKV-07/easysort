use crate::plan::PlannedMove;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct MovedFile {
    pub src: PathBuf,
    pub dest: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Failure {
    pub src: PathBuf,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct MoveReport {
    pub moved: Vec<MovedFile>,
    pub failures: Vec<Failure>,
}

pub fn unique_name(dir: &Path, name: &str) -> PathBuf {
    let dest = dir.join(name);
    if !dest.exists() {
        return dest;
    }
    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name);
    let ext = Path::new(name).extension().and_then(|s| s.to_str());
    let mut counter = 1;
    loop {
        let candidate = match ext {
            Some(e) => dir.join(format!("{}_{}.{}", stem, counter, e)),
            None => dir.join(format!("{}_{}", stem, counter)),
        };
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

fn move_one(src: &Path, dest: &Path) -> Result<(), std::io::Error> {
    // try rename first (fast, same filesystem)
    if fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    // cross-device fallback: copy then remove
    fs::copy(src, dest)?;
    fs::remove_file(src)?;
    Ok(())
}

pub fn execute(plan_moves: &[PlannedMove]) -> MoveReport {
    let mut report = MoveReport::default();
    for m in plan_moves {
        let src_meta = match std::fs::symlink_metadata(&m.src) {
            Ok(m) => m,
            Err(e) => {
                report.failures.push(Failure {
                    src: m.src.clone(),
                    reason: e.to_string(),
                });
                continue;
            }
        };
        if !src_meta.file_type().is_file() {
            report.failures.push(Failure {
                src: m.src.clone(),
                reason: "not a regular file".into(),
            });
            continue;
        }
        let dest_dir = match m.dest.parent() {
            Some(d) => d,
            None => {
                report.failures.push(Failure {
                    src: m.src.clone(),
                    reason: "no parent dir".into(),
                });
                continue;
            }
        };
        if let Err(e) = fs::create_dir_all(dest_dir) {
            report.failures.push(Failure {
                src: m.src.clone(),
                reason: e.to_string(),
            });
            continue;
        }
        let final_dest = unique_name(
            dest_dir,
            m.dest
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file"),
        );
        match move_one(&m.src, &final_dest) {
            Ok(()) => report.moved.push(MovedFile {
                src: m.src.clone(),
                dest: final_dest,
            }),
            Err(e) => report.failures.push(Failure {
                src: m.src.clone(),
                reason: e.to_string(),
            }),
        }
    }
    report
}
