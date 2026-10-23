# easysort

**Turns your Downloads folder into a clean, typed archive — interactive TUI by default, cron-friendly CLI when you need it.**

[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/your-org/easysort/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/easysort/actions/workflows/ci.yml)
[![Dependencies: 0 (Python)](https://img.shields.io/badge/python%20deps-0-brightgreen.svg)](organizer.py)

One Rust binary. Zero runtime dependencies. A TUI for humans, a CLI for cron.

---

```
$ easysort ~/Downloads
┌ easysort ─ ~/Downloads ────────────────── 124 files · 3.1 GB ──┐
│ mode: type │ filter: (none) │ sort: size                       │
├─────────────────────────────────────────────────────────────────┤
│ ▸ ☐ photo.jpg              2.4 MB   → Images/                   │
│   ☑ report.pdf             840 KB   → Documents/                │
│   ☐ notes                          → Other/                    │
│   ☑ setup.exe              18.2 MB  → Installers/               │
├─────────────────────────────────────────────────────────────────┤
│ 2/124 selected · 19 MB · enter to apply                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Why this exists

Your Downloads folder is a graveyard. It's where every screenshot, installer, invoice and half-finished idea lands — and it never sorts itself out because sorting 300 files by hand is exactly the chore nobody volunteers for.

`easysort` does it in one command. You get a full plan before anything touches the disk, and you approve each file individually in the TUI — because "I moved my tax documents somewhere" is the kind of mistake you only want to make once.

Two ways to sort:

- **By type** → files land in `Images/`, `Documents/`, `Videos/`, and friends.
- **By date** → files land in `2024/05-May/` style folders based on when they were last modified.

## Highlights

- **Dry run is the default.** No flag means nothing moves. You have to type `--run` to touch the disk.
- **Interactive TUI.** Review, filter, sort, and toggle each file before committing. No more all-or-nothing.
- **Never deletes.** There is no delete code in this repo. Worst case, a file lands in the wrong folder.
- **Never overwrites.** Name collisions become `report_1.pdf`, `report_2.pdf` — the original is always intact.
- **Per-file error handling.** One unwritable file doesn't abort the run; it's reported and the rest proceed.
- **Undo.** Every applied batch is logged; `easysort undo` puts everything back.
- **Zero runtime dependencies.** Pure Rust standard library + serde/chrono. Static binary.
- **Cron/launchd friendly.** `easysort sort ~/Downloads --run` with JSON output and meaningful exit codes.

---

## Quick start

**Build from source:**

```bash
git clone https://github.com/your-org/easysort.git
cd easysort
cargo build --release
# binary at ./target/release/easysort
```

**Or install with cargo:**

```bash
cargo install --path crates/easysort-cli
```

**Launch the TUI:**

```bash
easysort ~/Downloads
```

**Sort non-interactively (cron):**

```bash
easysort sort ~/Downloads --run
```

**Sort by date:**

```bash
easysort sort ~/Downloads --run --by-date
```

---

## Usage

### TUI mode

```
easysort [PATH]
```

Defaults to `~/Downloads` if no path is given. Scan, review, filter, sort, toggle, and apply — all interactively.

| Key | Action |
|---|---|
| `j`/`k`, `↑`/`↓` | Navigate |
| `g` / `G` | Top / bottom |
| `Space` | Toggle current file |
| `a` / `n` / `i` | Select all / none / invert |
| `/` | Filter (substring) |
| `s` | Cycle sort: name → size → date → destination |
| `Tab` | Toggle type ⇄ date mode |
| `Enter` | Confirm and apply selected |
| `u` | Undo last batch |
| `?` | Help overlay |
| `q` | Quit |

### CLI mode

```
easysort sort PATH [--run] [--by-date] [--json]
easysort undo [--manifest PATH]
```

| Flag | Default | What it does |
|---|---|---|
| `PATH` | *(required)* | Folder to organize. |
| `--run` | off | Actually move files. Without it, dry-run plan only. |
| `--by-date` | off | Sort into `YYYY/MM-Month/` folders by last-modified time. |
| `--json` | off | Emit machine-readable JSON (for scripting). |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | Success (including dry run). |
| `1` | Bad usage, missing folder, or one or more move failures. |

---

## How it sorts

### By type (default)

| Category | Extensions |
|---|---|
| `Images/` | `.jpg` `.jpeg` `.png` `.gif` `.webp` `.heic` `.svg` |
| `Documents/` | `.pdf` `.doc` `.docx` `.txt` `.odt` `.pages` |
| `Spreadsheets/` | `.xls` `.xlsx` `.csv` `.numbers` `.ods` |
| `Videos/` | `.mp4` `.mov` `.avi` `.mkv` `.m4v` |
| `Audio/` | `.mp3` `.wav` `.flac` `.aac` `.m4a` |
| `Archives/` | `.zip` `.rar` `.tar` `.gz` `.7z` |
| `Code/` | `.py` `.js` `.html` `.css` `.json` `.sql` |
| `Installers/` | `.dmg` `.pkg` `.exe` `.msi` |
| `Other/` | *(anything else)* |

Matching is **case-insensitive**. `Other/` is the safety net for extensionless files.

### By date (`--by-date`)

```
Downloads/
├── 2024/
│   └── 05-May/
│       └── old-draft.pdf
└── 2026/
    └── 10-October/
        └── new-shot.png
```

### Rules that apply to both modes

- **Top level only.** Subfolders are skipped entirely.
- **Alphabetical order.** Stable, predictable output.
- **Collision-safe.** `Images/photo.jpg` exists → incoming becomes `Images/photo_1.jpg`.
- **Folders are never moved.** Only regular files.
- **Symlinks are skipped.** Avoids dangling-link crashes.

---

## Safety model

| Guarantee | How it's enforced |
|---|---|
| Nothing moves without `--run` (CLI) or `Enter`+`y` (TUI) | Move call is gated behind explicit confirmation. |
| Nothing is ever deleted | No delete/unlink/truncate call anywhere in the codebase. |
| Nothing is ever overwritten | Destination checked with `exists()` before every move. |
| Permissions and timestamps preserved | Uses `rename` with copy+remove fallback for cross-filesystem. |
| Per-file error handling | One failure is reported; the rest proceed. Exit code 1 if any failed. |
| Undo available | Every applied batch is logged to `.easysort-manifest.jsonl`. |

---

## Automating it

### Linux / macOS — cron

```cron
0 9 * * * /usr/local/bin/easysort sort /home/yourname/Downloads --run >> /tmp/organizer.log 2>&1
```

### macOS — launchd

See the previous README version's launchd section, or use `cron` via `crontab`. For macOS, prefer launchd (runs on wake). Create `~/Library/LaunchAgents/com.easysort.daily.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.easysort.daily</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/easysort</string>
        <string>sort</string>
        <string>/Users/yourname/Downloads</string>
        <string>--run</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key><integer>9</integer>
        <key>Minute</key><integer>0</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>/tmp/easysort.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/easysort.log</string>
</dict>
</plist>
```

```bash
launchctl load ~/Library/LaunchAgents/com.easysort.daily.plist
```

---

## Portable Python version

`organizer.py` (98 lines, stdlib only) is kept at the repo root for environments without Rust:

```bash
python3 organizer.py ~/Downloads          # dry run
python3 organizer.py ~/Downloads --run     # execute
python3 organizer.py ~/Downloads --by-date # by date
```

It has identical sorting logic and is kept in parity via `tests/parity.py`. Use it on locked-down machines, CI containers, or anywhere you can't install a binary.

---

## Project layout

```
easysort/
├── Cargo.toml
├── crates/
│   ├── easysort-core/       # pure logic, no TUI deps, fast tests
│   │   └── src/
│   │       ├── rules.rs     # extension → category
│   │       ├── plan.rs      # scan + plan building
│   │       ├── mover.rs     # execute + per-file errors + collision naming
│   │       ├── undo.rs      # undo stack + manifest
│   │       ├── mode.rs      # ByType / ByDate
│   │       └── error.rs
│   └── easysort-cli/        # binary: TUI + CLI
│       └── src/
│           ├── main.rs
│           ├── cli.rs       # clap
│           └── tui/         # ratatui app + rendering
├── organizer.py             # portable single-file version
├── tests/
│   └── parity.py            # Rust vs Python tree comparison
├── .github/workflows/ci.yml
├── LICENSE                  # Apache 2.0
└── README.md
```

## Development

```bash
cargo build                          # debug build
cargo build --release                 # release build
cargo test --workspace               # all tests
cargo clippy --workspace -- -D warnings  # lint
cargo fmt --check                    # format check
python3 tests/parity.py              # Rust vs Python parity (needs binary built)
```

## Known limitations

| Limitation | Status |
|---|---|
| Only top level scanned (no `--recursive`) | Roadmap |
| Hidden files not skipped | Roadmap |
| Extension-only detection (no content sniffing) | Roadmap |
| Date mode uses mtime only | Roadmap |
| Symlinks skipped (not moved) | By design in Rust; Python treats them as files |
| No config file (rules hardcoded) | Roadmap |
| Case-insensitive collisions on Linux | By design (matches Python) |

## Roadmap

- `--recursive` / `--depth N`
- Skip hidden files by default
- `--include` / `--exclude` globs
- Content-based detection for extensionless files
- Duplicate detection by hash
- Stats summary + `--json` (CLI done; TUI pending)
- Config file for custom rules
- Watch mode (`--watch`)
- More TUI tests (TestBackend snapshots)

## Contributing

Issues and PRs welcome. Keep it dependency-light, keep dry-run the default, keep the Python version in parity.

## License

Apache License 2.0 — see [LICENSE](LICENSE).
