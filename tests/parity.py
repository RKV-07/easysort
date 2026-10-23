#!/usr/bin/env python3
"""Parity test: run Python organizer.py and Rust easysort on identical fixtures, compare trees."""
import os, shutil, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PY = os.path.join(ROOT, "organizer.py")
RUST = os.path.join(ROOT, "target", "debug", "easysort")

def make_fixture(d):
    os.makedirs(d, exist_ok=True)
    files = {
        "photo.jpg": b"",
        "report.PDF": b"",
        "data.csv": b"",
        "script.py": b"",
        "archive.tar.gz": b"",
        "notes": b"",
        "README": b"",
    }
    for name, content in files.items():
        with open(os.path.join(d, name), "wb") as f:
            f.write(content)
    # collision case
    os.makedirs(os.path.join(d, "Images"), exist_ok=True)
    with open(os.path.join(d, "Images", "photo.jpg"), "wb") as f:
        f.write(b"")
    # subfolder (should be untouched)
    os.makedirs(os.path.join(d, "sub"), exist_ok=True)
    with open(os.path.join(d, "sub", "inner.txt"), "wb") as f:
        f.write(b"")

def tree(d):
    out = []
    for root, _, files in os.walk(d):
        for fn in sorted(files):
            rel = os.path.relpath(os.path.join(root, fn), d)
            out.append(rel)
    return sorted(out)

def run_py(src, dst):
    shutil.copytree(src, dst)
    subprocess.run([sys.executable, PY, dst, "--run"], check=True, capture_output=True)

def run_rust(src, dst):
    shutil.copytree(src, dst)
    subprocess.run([RUST, "sort", dst, "--run"], check=True, capture_output=True)

def main():
    if not os.path.exists(RUST):
        print("Rust binary not found; run `cargo build -p easysort-cli` first", file=sys.stderr)
        sys.exit(1)
    src = tempfile.mkdtemp()
    make_fixture(src)
    py_dst = tempfile.mkdtemp() + "_py"
    rust_dst = tempfile.mkdtemp() + "_rust"
    run_py(src, py_dst)
    run_rust(src, rust_dst)
    py_tree = tree(py_dst)
    rust_tree = tree(rust_dst)
    # ignore manifest files in comparison
    py_tree = [p for p in py_tree if not p.startswith(".easysort")]
    rust_tree = [p for p in rust_tree if not p.startswith(".easysort")]
    if py_tree == rust_tree:
        print("PARITY OK: both produced identical file trees")
        print("\n".join(py_tree))
        return 0
    else:
        print("PARITY MISMATCH", file=sys.stderr)
        print("Python:", file=sys.stderr)
        for p in py_tree: print("  " + p, file=sys.stderr)
        print("Rust:", file=sys.stderr)
        for p in rust_tree: print("  " + p, file=sys.stderr)
        return 1

if __name__ == "__main__":
    sys.exit(main())
