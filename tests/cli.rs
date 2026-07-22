use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "normalize-punctuation-{}-{suffix}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("temporary directory should be created");
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("temporary directory should be removed");
    }
}

#[test]
fn normalizes_every_path_argument() {
    let temp_dir = TempDir::new();
    let clean = temp_dir.0.join("clean.md");
    let unnormalized = temp_dir.0.join("unnormalized.md");
    fs::write(&clean, "Already normalized.\n").expect("clean fixture should be written");
    fs::write(&unnormalized, "“Needs normalization.”\n")
        .expect("unnormalized fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_normalize-punctuation"))
        .args([clean, unnormalized.clone()])
        .output()
        .expect("normalize-punctuation should run");

    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(unnormalized).expect("normalized fixture should be readable"),
        "\"Needs normalization.\"\n"
    );
    assert!(
        String::from_utf8(output.stdout)
            .expect("stdout should be UTF-8")
            .starts_with("Scanned 2 files, modified 1. (")
    );
}
