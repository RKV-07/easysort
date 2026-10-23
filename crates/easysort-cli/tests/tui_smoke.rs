use easysort_cli::tui;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn tui_renders_without_panic() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("a.jpg"), b"").unwrap();
    std::fs::write(tmp.path().join("b.pdf"), b"").unwrap();
    let mut app = tui::App::new(tmp.path().to_path_buf());
    app.scan();
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| tui::ui::render(f, &app)).unwrap();
    // verify some content rendered
    let buf = terminal.backend().buffer();
    let text: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(text.contains("easysort"));
    assert!(text.contains("a.jpg") || text.contains("b.pdf"));
}

#[test]
fn tui_renders_confirm_overlay() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("a.jpg"), b"").unwrap();
    let mut app = tui::App::new(tmp.path().to_path_buf());
    app.scan();
    app.selected.insert(0);
    app.screen = tui::Screen::Confirming;
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| tui::ui::render(f, &app)).unwrap();
    let buf = terminal.backend().buffer();
    let text: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(text.contains("Apply"));
}

#[test]
fn tui_renders_help_overlay() {
    let tmp = TempDir::new().unwrap();
    let mut app = tui::App::new(tmp.path().to_path_buf());
    app.screen = tui::Screen::Help;
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| tui::ui::render(f, &app)).unwrap();
    let buf = terminal.backend().buffer();
    let text: String = buf.content().iter().map(|c| c.symbol()).collect();
    assert!(text.contains("Navigation"));
}
