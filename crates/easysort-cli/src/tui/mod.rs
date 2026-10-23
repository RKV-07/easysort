pub mod ui;

use chrono::Utc;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use easysort_core::{
    append_manifest, build_plan, default_rules, execute, scan, undo_batch, Batch, Failure,
    FileEntry, Mode, MovedFile, MovedPair, Plan,
};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Scanning,
    Ready,
    Confirming,
    Applying,
    Result,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Size,
    Date,
    Dest,
}

pub struct App {
    pub target: PathBuf,
    pub mode: Mode,
    pub rules: easysort_core::Rules,
    pub entries: Vec<FileEntry>,
    pub plan: Plan,
    pub selected: HashSet<usize>,
    pub filter: String,
    pub filtering: bool,
    pub sort: SortKey,
    pub cursor: usize,
    pub screen: Screen,
    pub report_moved: Vec<MovedFile>,
    pub report_failed: Vec<Failure>,
    pub undo_stack: Vec<Batch>,
    pub manifest_path: PathBuf,
    pub status: String,
    pub quit: bool,
}

impl App {
    pub fn new(target: PathBuf) -> Self {
        let manifest_path = target.join(".easysort-manifest.jsonl");
        Self {
            target,
            mode: Mode::ByType,
            rules: default_rules(),
            entries: Vec::new(),
            plan: Plan::default(),
            selected: HashSet::new(),
            filter: String::new(),
            filtering: false,
            sort: SortKey::Name,
            cursor: 0,
            screen: Screen::Scanning,
            report_moved: Vec::new(),
            report_failed: Vec::new(),
            undo_stack: Vec::new(),
            manifest_path,
            status: "scanning...".into(),
            quit: false,
        }
    }

    pub fn scan(&mut self) {
        match scan(&self.target) {
            Ok(entries) => {
                self.entries = entries;
                self.replan();
                self.screen = Screen::Ready;
                self.status = format!("{} files", self.entries.len());
            }
            Err(e) => {
                self.status = format!("scan error: {}", e);
                self.screen = Screen::Ready;
            }
        }
    }

    pub fn replan(&mut self) {
        self.plan = build_plan(&self.target, &self.entries, self.mode, &self.rules);
        // keep selection valid
        self.selected.retain(|&i| i < self.plan.moves.len());
        if self.cursor >= self.plan.moves.len() && !self.plan.moves.is_empty() {
            self.cursor = self.plan.moves.len() - 1;
        }
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.plan.moves.len())
            .filter(|&i| {
                if self.filter.is_empty() {
                    return true;
                }
                let name = self.plan.moves[i]
                    .src
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_lowercase();
                name.contains(&self.filter.to_lowercase())
            })
            .collect();
        // sort
        match self.sort {
            SortKey::Name => {
                idx.sort_by(|&a, &b| self.plan.moves[a].src.cmp(&self.plan.moves[b].src))
            }
            SortKey::Size => {
                // need size from entries; approximate by file metadata
                idx.sort_by(|&a, &b| {
                    let sa = std::fs::metadata(&self.plan.moves[a].src)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    let sb = std::fs::metadata(&self.plan.moves[b].src)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    sb.cmp(&sa)
                });
            }
            SortKey::Date => {
                idx.sort_by(|&a, &b| {
                    let da = std::fs::metadata(&self.plan.moves[a].src)
                        .and_then(|m| m.modified())
                        .ok();
                    let db = std::fs::metadata(&self.plan.moves[b].src)
                        .and_then(|m| m.modified())
                        .ok();
                    db.cmp(&da)
                });
            }
            SortKey::Dest => {
                idx.sort_by(|&a, &b| self.plan.moves[a].dest.cmp(&self.plan.moves[b].dest))
            }
        }
        idx
    }

    pub fn toggle(&mut self, i: usize) {
        if self.selected.contains(&i) {
            self.selected.remove(&i);
        } else {
            self.selected.insert(i);
        }
    }

    pub fn select_all_visible(&mut self) {
        for i in self.visible_indices() {
            self.selected.insert(i);
        }
    }

    pub fn select_none(&mut self) {
        self.selected.clear();
    }

    pub fn invert(&mut self) {
        let vis: HashSet<usize> = self.visible_indices().into_iter().collect();
        let new: HashSet<usize> = vis.difference(&self.selected).copied().collect();
        // remove visible from selected, add newly selected
        for i in &vis {
            self.selected.remove(i);
        }
        for i in new {
            self.selected.insert(i);
        }
    }

    pub fn apply(&mut self) {
        self.screen = Screen::Applying;
        // build a plan with only selected moves
        let selected_moves: Vec<_> = self
            .selected
            .iter()
            .map(|&i| self.plan.moves[i].clone())
            .collect();
        let report = execute(&selected_moves);
        self.report_moved = report.moved.clone();
        self.report_failed = report.failures.clone();

        if !report.moved.is_empty() {
            let batch = Batch {
                timestamp: Utc::now().timestamp(),
                moves: report
                    .moved
                    .iter()
                    .map(|m| MovedPair {
                        from: m.src.clone(),
                        to: m.dest.clone(),
                    })
                    .collect(),
            };
            self.undo_stack.push(batch.clone());
            let _ = append_manifest(&self.manifest_path, &batch);
        }

        // remove moved files from entries by rescanning
        self.scan();
        self.selected.clear();
        self.screen = Screen::Result;
        self.status = format!(
            "moved {} · failed {}",
            self.report_moved.len(),
            self.report_failed.len()
        );
    }

    pub fn undo_last(&mut self) {
        if let Some(batch) = self.undo_stack.pop() {
            let failures = undo_batch(&batch);
            if failures.is_empty() {
                self.status = format!("undone {} move(s)", batch.moves.len());
            } else {
                self.status = format!("undo had {} failure(s)", failures.len());
            }
            self.scan();
            self.selected.clear();
        } else {
            self.status = "nothing to undo".into();
        }
    }

    pub fn total_selected_bytes(&self) -> u64 {
        self.selected
            .iter()
            .map(|&i| {
                std::fs::metadata(&self.plan.moves[i].src)
                    .map(|m| m.len())
                    .unwrap_or(0)
            })
            .sum()
    }
}

pub fn run(target: PathBuf) -> anyhow::Result<()> {
    // panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        original_hook(info);
    }));

    let mut terminal = ratatui::init();
    let mut app = App::new(target);
    app.scan();

    let res = run_loop(&mut terminal, &mut app);

    ratatui::restore();
    res
}

fn run_loop(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        return Ok(());
                    }
                    match app.screen {
                        Screen::Ready => handle_ready(app, key),
                        Screen::Confirming => handle_confirming(app, key),
                        Screen::Result => handle_result(app, key),
                        Screen::Help => {
                            if matches!(
                                key.code,
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')
                            ) {
                                app.screen = Screen::Ready;
                            }
                        }
                        Screen::Applying => {}
                        Screen::Scanning => {}
                    }
                    if app.filtering {
                        handle_filter_input(app, key);
                    }
                    if app.quit {
                        return Ok(());
                    }
                }
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        if app.screen == Screen::Applying {
            // apply is synchronous in v1; this branch won't be hit mid-apply
        }

        // trigger apply when confirming accepted
        // (handled in handle_confirming)
    }
}

fn handle_ready(app: &mut App, key: KeyEvent) {
    if app.filtering {
        return;
    }
    let vis = app.visible_indices();
    match key.code {
        KeyCode::Char('q') => {
            app.quit = true;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if !vis.is_empty() {
                let next = match vis.iter().position(|&i| i == app.cursor) {
                    Some(pos) => (pos + 1).min(vis.len() - 1),
                    None => 0,
                };
                app.cursor = vis[next];
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if !vis.is_empty() {
                let prev = match vis.iter().position(|&i| i == app.cursor) {
                    Some(pos) => pos.saturating_sub(1),
                    None => vis.len() - 1,
                };
                app.cursor = vis[prev];
            }
        }
        KeyCode::Char('g') => {
            if !vis.is_empty() {
                app.cursor = vis[0];
            }
        }
        KeyCode::Char('G') => {
            if !vis.is_empty() {
                app.cursor = *vis.last().unwrap();
            }
        }
        KeyCode::Char(' ') => {
            app.toggle(app.cursor);
        }
        KeyCode::Char('a') => {
            app.select_all_visible();
        }
        KeyCode::Char('n') => {
            app.select_none();
        }
        KeyCode::Char('i') => {
            app.invert();
        }
        KeyCode::Char('/') => {
            app.filtering = true;
            app.filter.clear();
        }
        KeyCode::Char('s') => {
            app.sort = match app.sort {
                SortKey::Name => SortKey::Size,
                SortKey::Size => SortKey::Date,
                SortKey::Date => SortKey::Dest,
                SortKey::Dest => SortKey::Name,
            };
        }
        KeyCode::Tab => {
            app.mode = match app.mode {
                Mode::ByType => Mode::ByDate,
                Mode::ByDate => Mode::ByType,
            };
            app.replan();
        }
        KeyCode::Enter => {
            if !app.selected.is_empty() {
                app.screen = Screen::Confirming;
            }
        }
        KeyCode::Char('u') => {
            app.undo_last();
        }
        KeyCode::Char('?') => {
            app.screen = Screen::Help;
        }
        KeyCode::Esc => {
            app.filter.clear();
        }
        _ => {}
    }
}

fn handle_confirming(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.apply();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.screen = Screen::Ready;
        }
        _ => {}
    }
}

fn handle_result(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('u') => {
            app.undo_last();
            app.screen = Screen::Ready;
        }
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => {
            app.screen = Screen::Ready;
        }
        _ => {}
    }
}

fn handle_filter_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.filtering = false;
            app.filter.clear();
        }
        KeyCode::Enter => {
            app.filtering = false;
        }
        KeyCode::Backspace => {
            app.filter.pop();
        }
        KeyCode::Char(c) => {
            app.filter.push(c);
        }
        _ => {}
    }
}
