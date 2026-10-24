#!/usr/bin/env python3
"""
organizer.py — Auto File Organizer
Usage:
  python3 organizer.py /path/to/folder           # sort by type (dry run)
  python3 organizer.py /path/to/folder --run      # sort by type (execute)
  python3 organizer.py /path/to/folder --by-date  # sort by date (dry run)
"""
import sys
import shutil
import datetime
from pathlib import Path

FILE_RULES = {
    "Images":       [".jpg", ".jpeg", ".png", ".gif", ".webp", ".heic", ".svg"],
    "Documents":    [".pdf", ".doc", ".docx", ".txt", ".odt", ".pages"],
    "Spreadsheets": [".xls", ".xlsx", ".csv", ".numbers", ".ods"],
    "Videos":       [".mp4", ".mov", ".avi", ".mkv", ".m4v"],
    "Audio":         [".mp3", ".wav", ".flac", ".aac", ".m4a"],
    "Archives":     [".zip", ".rar", ".tar", ".gz", ".7z"],
    "Code":          [".py", ".js", ".html", ".css", ".json", ".sql"],
    "Installers":   [".dmg", ".pkg", ".exe", ".msi"],
}

def get_destination(extension: str) -> str:
    ext = extension.lower()
    for folder_name, exts in FILE_RULES.items():
        if ext in exts:
            return folder_name
    return "Other"

def safe_move(src: Path, dest_folder: Path) -> Path:
    """Move file; auto-rename if destination already exists."""
    dest = dest_folder / src.name
    counter = 1
    while dest.exists():
        dest = dest_folder / f"{src.stem}_{counter}{src.suffix}"
        counter += 1
    shutil.move(str(src), str(dest))
    return dest

def organize_by_type(folder: Path, dry_run: bool = True) -> None:
    moved = 0
    for file in sorted(folder.iterdir()):
        if file.is_dir():
            continue
        dest_folder = folder / get_destination(file.suffix)
        if dry_run:
            print(f"  [DRY RUN] {file.name:<40} →  {dest_folder.name}/")
        else:
            dest_folder.mkdir(exist_ok=True)
            result = safe_move(file, dest_folder)
            print(f"  ✅ {file.name:<40} →  {result.parent.name}/")
        moved += 1
    label = "Would move" if dry_run else "Moved"
    print(f"\n{label} {moved} file(s).")
    if dry_run:
        print("  Add --run to execute.")

def organize_by_date(folder: Path, dry_run: bool = True) -> None:
    moved = 0
    for file in sorted(folder.iterdir()):
        if file.is_dir():
            continue
        mtime      = datetime.datetime.fromtimestamp(file.stat().st_mtime)
        month_dir  = folder / str(mtime.year) / mtime.strftime("%m-%B")
        if dry_run:
            print(f"  [DRY RUN] {file.name:<40} →  {mtime.year}/{mtime.strftime('%m-%B')}/")
        else:
            month_dir.mkdir(parents=True, exist_ok=True)
            safe_move(file, month_dir)
            print(f"  📅 {file.name:<40} →  {mtime.year}/{mtime.strftime('%m-%B')}/")
        moved += 1
    label = "Would move" if dry_run else "Moved"
    print(f"\n{label} {moved} file(s).")

if __name__ == "__main__":
    args = sys.argv[1:]
    execute = "--run" in args
    by_date = "--by-date" in args
    paths = [a for a in args if not a.startswith("--")]

    if not paths:
        print("Usage: python3 organizer.py /path/to/folder [/path/to/folder2 ...] [--run] [--by-date]")
        sys.exit(1)

    for target_str in paths:
        target = Path(target_str)
        if not target.is_dir():
            print(f"Error: '{target}' is not a valid folder.")
            sys.exit(1)

        mode = "DATE" if by_date else "TYPE"
        run = "LIVE" if execute else "DRY RUN"
        print(f"📂 Organizing: {target}")
        print(f"   Mode: {mode}  |  {run}\n")

        if by_date:
            organize_by_date(target, dry_run=not execute)
        else:
            organize_by_type(target, dry_run=not execute)
