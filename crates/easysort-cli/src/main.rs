mod cli;
pub mod tui;

use clap::Parser;
use cli::{Cli, Command};
use easysort_core::{
    append_manifest, build_plan, default_rules, execute, scan, undo_batch, Batch, Mode, MovedPair,
};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("easysort=info".parse()?))
        .init();

    let cli = Cli::parse();
    match cli.command {
        Some(Command::Sort {
            path,
            run,
            by_date,
            json,
        }) => cmd_sort(path, run, by_date, json),
        Some(Command::Undo { manifest }) => cmd_undo(manifest),
        None => {
            // default: TUI
            let path = cli.path.unwrap_or_else(default_downloads);
            tui::run(path)
        }
    }
}

fn default_downloads() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|u| u.home_dir().join("Downloads").into())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn cmd_sort(path: PathBuf, run: bool, by_date: bool, json: bool) -> anyhow::Result<()> {
    if !path.is_dir() {
        eprintln!("Error: '{}' is not a valid folder.", path.display());
        std::process::exit(1);
    }
    let mode = if by_date { Mode::ByDate } else { Mode::ByType };
    let rules = default_rules();
    let entries = scan(&path)?;
    let plan = build_plan(&path, &entries, mode, &rules);

    if !run {
        // dry run
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &plan
                        .moves
                        .iter()
                        .map(|m| serde_json::json!({
                            "src": m.src, "dest": m.dest
                        }))
                        .collect::<Vec<_>>()
                )?
            );
        } else {
            println!("📂 Organizing: {}", path.display());
            println!(
                "   Mode: {}  |  DRY RUN\n",
                if by_date { "DATE" } else { "TYPE" }
            );
            for m in &plan.moves {
                println!(
                    "  [DRY RUN] {:<40} →  {}/",
                    m.src.file_name().unwrap_or_default().to_string_lossy(),
                    m.dest
                        .parent()
                        .unwrap_or(&path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                );
            }
            println!("\nWould move {} file(s).", plan.moves.len());
            println!("  Add --run to execute.");
        }
        return Ok(());
    }

    let report = execute(&plan.moves);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "moved": report.moved.len(),
                "failed": report.failures.len(),
                "moves": report.moved.iter().map(|m| serde_json::json!({"src": m.src, "dest": m.dest})).collect::<Vec<_>>(),
                "failures": report.failures.iter().map(|f| serde_json::json!({"src": f.src, "reason": f.reason})).collect::<Vec<_>>(),
            }))?
        );
    } else {
        println!("📂 Organizing: {}", path.display());
        println!(
            "   Mode: {}  |  LIVE\n",
            if by_date { "DATE" } else { "TYPE" }
        );
        for m in &report.moved {
            println!(
                "  ✅ {:<40} →  {}/",
                m.src.file_name().unwrap_or_default().to_string_lossy(),
                m.dest
                    .parent()
                    .unwrap_or(&path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
        }
        for f in &report.failures {
            println!(
                "  ❌ {:<40} →  {}",
                f.src.file_name().unwrap_or_default().to_string_lossy(),
                f.reason
            );
        }
        println!("\nMoved {} file(s).", report.moved.len());
        if !report.failures.is_empty() {
            println!("{} failure(s).", report.failures.len());
        }
    }

    // write manifest
    if !report.moved.is_empty() {
        let manifest = path.join(".easysort-manifest.jsonl");
        let batch = Batch {
            timestamp: chrono::Utc::now().timestamp(),
            moves: report
                .moved
                .iter()
                .map(|m| MovedPair {
                    from: m.src.clone(),
                    to: m.dest.clone(),
                })
                .collect(),
        };
        let _ = append_manifest(&manifest, &batch);
    }

    if !report.failures.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_undo(manifest: Option<PathBuf>) -> anyhow::Result<()> {
    let m = manifest.unwrap_or_else(|| PathBuf::from(".easysort-manifest.jsonl"));
    if !m.exists() {
        eprintln!("No manifest found at {}", m.display());
        std::process::exit(1);
    }
    let content = std::fs::read_to_string(&m)?;
    // undo last batch (last line)
    let last = content.lines().rfind(|l| !l.trim().is_empty());
    match last {
        Some(line) => {
            let batch: Batch = serde_json::from_str(line)?;
            let failures = undo_batch(&batch);
            if failures.is_empty() {
                println!("Undone {} move(s).", batch.moves.len());
            } else {
                for (p, r) in &failures {
                    eprintln!("  ❌ {} → {}", p.display(), r);
                }
                std::process::exit(1);
            }
        }
        None => {
            eprintln!("Manifest is empty.");
            std::process::exit(1);
        }
    }
    Ok(())
}
