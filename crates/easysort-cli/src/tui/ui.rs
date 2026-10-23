use super::App;
use super::{Screen, SortKey};
use easysort_core::Mode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(2),
    ])
    .split(f.area());

    // header
    let header = Paragraph::new(format!(
        " easysort  {}  │  {} files  │  mode: {}  │  sort: {}",
        app.target.display(),
        app.entries.len(),
        match app.mode {
            Mode::ByType => "type",
            Mode::ByDate => "date",
        },
        match app.sort {
            SortKey::Name => "name",
            SortKey::Size => "size",
            SortKey::Date => "date",
            SortKey::Dest => "dest",
        },
    ))
    .block(Block::default().borders(Borders::ALL).title("easysort"));
    f.render_widget(header, chunks[0]);

    // filter line
    let filter_text = if app.filtering {
        format!(" filter: {}_", app.filter)
    } else if app.filter.is_empty() {
        " filter: (none)  — press / to filter".to_string()
    } else {
        format!(" filter: {}", app.filter)
    };
    f.render_widget(Paragraph::new(filter_text), chunks[1]);

    // table
    let vis = app.visible_indices();
    let rows: Vec<Row> = vis
        .iter()
        .map(|&i| {
            let m = &app.plan.moves[i];
            let name = m
                .src
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let size = std::fs::metadata(&m.src)
                .map(|md| format_size(md.len()))
                .unwrap_or_default();
            let dest = m
                .dest
                .parent()
                .unwrap_or(&app.target)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let check = if app.selected.contains(&i) {
                "☑"
            } else {
                "☐"
            };
            let cursor_marker = if i == app.cursor { "▸" } else { " " };
            Row::new(vec![
                Cell::from(format!("{} {}", cursor_marker, check)),
                Cell::from(name),
                Cell::from(size),
                Cell::from(format!("→ {}/", dest)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Length(24),
    ];
    let table = Table::new(rows, widths)
        .header(Row::new(vec!["", "Name", "Size", "Destination"]).style(Style::default().bold()))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{} shown", vis.len())),
        )
        .row_highlight_style(Style::default().reversed());

    let mut state = TableState::default();
    // find position of cursor in visible list
    if let Some(pos) = vis.iter().position(|&i| i == app.cursor) {
        state.select(Some(pos));
    }
    f.render_stateful_widget(table, chunks[2], &mut state);

    // status bar
    let status = format!(
        " {}  │  selected: {}  ({} bytes)  │  {}",
        app.status,
        app.selected.len(),
        app.total_selected_bytes(),
        "space toggle · a all · n none · / filter · s sort · tab mode · enter apply · u undo · ? help · q quit",
    );
    f.render_widget(Paragraph::new(status).wrap(Wrap { trim: true }), chunks[3]);

    // overlays
    match app.screen {
        Screen::Confirming => {
            let area = centered_rect(50, 30, f.area());
            f.render_widget(Clear, area);
            let text = format!(
                "Apply {} move(s)?\n\n{} bytes will be moved.\n\n[y] yes   [n] cancel",
                app.selected.len(),
                app.total_selected_bytes(),
            );
            f.render_widget(
                Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).title("Confirm"))
                    .alignment(Alignment::Center),
                area,
            );
        }
        Screen::Help => {
            let area = centered_rect(60, 70, f.area());
            f.render_widget(Clear, area);
            let help = "Navigation\n  j/k, ↑/↓     move cursor\n  g / G        top / bottom\n  Ctrl+d/u     page down/up\n\nSelection\n  Space        toggle current\n  a / n / i    all / none / invert\n\nView\n  /            filter (Esc clears)\n  s            cycle sort (name/size/date/dest)\n  Tab          toggle type ⇄ date mode\n\nActions\n  Enter        apply selected (confirm)\n  u            undo last batch\n  ?            this help\n  q            quit\n";
            f.render_widget(
                Paragraph::new(help).block(Block::default().borders(Borders::ALL).title("Help")),
                area,
            );
        }
        Screen::Result => {
            let area = centered_rect(60, 50, f.area());
            f.render_widget(Clear, area);
            let mut lines = vec![format!("Moved: {}", app.report_moved.len())];
            lines.push(format!("Failed: {}", app.report_failed.len()));
            for f2 in &app.report_failed {
                lines.push(format!(
                    "  ❌ {} — {}",
                    f2.src.file_name().unwrap_or_default().to_string_lossy(),
                    f2.reason
                ));
            }
            lines.push(String::new());
            lines.push("u: undo   Enter/Esc: back".into());
            f.render_widget(
                Paragraph::new(lines.join("\n"))
                    .block(Block::default().borders(Borders::ALL).title("Result")),
                area,
            );
        }
        _ => {}
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
