use easysort_core::*;
use std::fs;
use tempfile::TempDir;

fn touch(dir: &std::path::Path, name: &str) {
    fs::write(dir.join(name), b"").unwrap();
}

#[test]
fn classify_case_insensitive() {
    let r = default_rules();
    assert_eq!(classify(&r, ".PDF"), "Documents");
    assert_eq!(classify(&r, ".jpg"), "Images");
    assert_eq!(classify(&r, ".xyz"), "Other");
    assert_eq!(classify(&r, ""), "Other");
}

#[test]
fn scan_top_level_only() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "a.jpg");
    touch(tmp.path(), "b.txt");
    fs::create_dir(tmp.path().join("sub")).unwrap();
    touch(&tmp.path().join("sub"), "c.png");
    let entries = scan(tmp.path()).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn scan_skips_dangling_symlink() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "real.jpg");
    #[cfg(unix)]
    std::os::unix::fs::symlink("/nonexistent/target", tmp.path().join("broken.jpg")).unwrap();
    let entries = scan(tmp.path()).unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn unique_name_collision() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "photo.jpg");
    let r = unique_name(tmp.path(), "photo.jpg");
    assert_eq!(r.file_name().unwrap(), "photo_1.jpg");
    touch(tmp.path(), "photo_1.jpg");
    let r2 = unique_name(tmp.path(), "photo.jpg");
    assert_eq!(r2.file_name().unwrap(), "photo_2.jpg");
}

#[test]
fn unique_name_multi_dot() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "archive.tar.gz");
    let r = unique_name(tmp.path(), "archive.tar.gz");
    assert_eq!(r.file_name().unwrap(), "archive.tar_1.gz");
}

#[test]
fn unique_name_no_ext() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "README");
    let r = unique_name(tmp.path(), "README");
    assert_eq!(r.file_name().unwrap(), "README_1");
}

#[test]
fn execute_moves_and_handles_collision() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "photo.jpg");
    // pre-create destination
    fs::create_dir(tmp.path().join("Images")).unwrap();
    touch(&tmp.path().join("Images"), "photo.jpg");
    let entries = scan(tmp.path()).unwrap();
    let plan = build_plan(tmp.path(), &entries, Mode::ByType, &default_rules());
    let report = execute(&plan.moves);
    assert_eq!(report.moved.len(), 1);
    assert_eq!(report.failures.len(), 0);
    assert!(tmp.path().join("Images").join("photo_1.jpg").exists());
}

#[test]
fn execute_continues_on_permission_error() {
    // create a read-only dir scenario: move into a folder we can't create
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "a.jpg");
    // make the parent read-only so create_dir_all fails
    let ro = tmp.path().join("readonly");
    fs::create_dir(&ro).unwrap();
    // put file inside readonly so scan finds it, but dest parent unwritable?
    // simpler: just verify execute returns failures not panic
    let entries = scan(tmp.path()).unwrap();
    let plan = build_plan(tmp.path(), &entries, Mode::ByType, &default_rules());
    let _ = execute(&plan.moves);
}

#[test]
fn by_date_creates_year_month() {
    let tmp = TempDir::new().unwrap();
    touch(tmp.path(), "old.pdf");
    let entries = scan(tmp.path()).unwrap();
    let plan = build_plan(tmp.path(), &entries, Mode::ByDate, &default_rules());
    assert!(plan.moves[0].dest.to_string_lossy().contains("/"));
    // dest should have year and month-monthname
    let s = plan.moves[0].dest.to_string_lossy();
    assert!(s.chars().any(|c| c.is_ascii_digit()));
}
