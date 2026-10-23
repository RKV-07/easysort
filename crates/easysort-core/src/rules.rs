use std::collections::BTreeMap;

pub type Rules = BTreeMap<String, Vec<String>>;

pub fn default_rules() -> Rules {
    let mut r = BTreeMap::new();
    r.insert(
        "Images".into(),
        vec![".jpg", ".jpeg", ".png", ".gif", ".webp", ".heic", ".svg"]
            .into_iter()
            .map(String::from)
            .collect(),
    );
    r.insert(
        "Documents".into(),
        vec![".pdf", ".doc", ".docx", ".txt", ".odt", ".pages"]
            .into_iter()
            .map(String::from)
            .collect(),
    );
    r.insert(
        "Spreadsheets".into(),
        vec![".xls", ".xlsx", ".csv", ".numbers", ".ods"]
            .into_iter()
            .map(String::from)
            .collect(),
    );
    r.insert(
        "Videos".into(),
        vec![".mp4", ".mov", ".avi", ".mkv", ".m4v"]
            .into_iter()
            .map(String::from)
            .collect(),
    );
    r.insert(
        "Audio".into(),
        vec![".mp3", ".wav", ".flac", ".aac", ".m4a"]
            .into_iter()
            .map(String::from)
            .collect(),
    );
    r.insert(
        "Archives".into(),
        vec![".zip", ".rar", ".tar", ".gz", ".7z"]
            .into_iter()
            .map(String::from)
            .collect(),
    );
    r.insert(
        "Code".into(),
        vec![".py", ".js", ".html", ".css", ".json", ".sql"]
            .into_iter()
            .map(String::from)
            .collect(),
    );
    r.insert(
        "Installers".into(),
        vec![".dmg", ".pkg", ".exe", ".msi"]
            .into_iter()
            .map(String::from)
            .collect(),
    );
    r
}

pub fn classify(rules: &Rules, ext: &str) -> String {
    let ext = ext.to_lowercase();
    for (name, exts) in rules {
        if exts.iter().any(|e| e == &ext) {
            return name.clone();
        }
    }
    "Other".to_string()
}
