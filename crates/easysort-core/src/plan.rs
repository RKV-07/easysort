use crate::mode::Mode;
use crate::rules::Rules;
use chrono::{Datelike, Local};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: i64,
}

#[derive(Debug, Clone)]
pub struct PlannedMove {
    pub src: PathBuf,
    pub dest: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct Plan {
    pub moves: Vec<PlannedMove>,
    pub skipped: Vec<PathBuf>,
}

pub fn scan(folder: &Path) -> Result<Vec<FileEntry>, std::io::Error> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        let ft = entry.file_type()?;
        if !ft.is_file() {
            continue;
        }
        if entry.file_name().to_string_lossy() == ".easysort-manifest.jsonl" {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.is_file() {
            continue;
        }
        out.push(FileEntry {
            path: entry.path(),
            size: meta.len(),
            mtime: meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

pub fn build_plan(folder: &Path, entries: &[FileEntry], mode: Mode, rules: &Rules) -> Plan {
    let mut plan = Plan::default();
    for e in entries {
        let dest_folder = match mode {
            Mode::ByType => {
                let full_ext = e
                    .path
                    .extension()
                    .and_then(|x| x.to_str())
                    .map(|s| format!(".{}", s))
                    .unwrap_or_default();
                folder.join(crate::rules::classify(rules, &full_ext))
            }
            Mode::ByDate => {
                let dt = chrono::DateTime::from_timestamp(e.mtime, 0)
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
                    .with_timezone(&Local);
                folder.join(dt.year().to_string()).join(format!(
                    "{:02}-{}",
                    dt.month(),
                    month_name(dt.month())
                ))
            }
        };
        let name = e.path.file_name().unwrap_or_default();
        plan.moves.push(PlannedMove {
            src: e.path.clone(),
            dest: dest_folder.join(name),
        });
    }
    plan
}

fn month_name(m: u32) -> &'static str {
    match m {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}
