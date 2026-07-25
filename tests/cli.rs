use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the Unix epoch")
            .as_nanos();
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "normalize-punctuation-{}-{suffix}-{counter}",
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
fn help_flags_print_help() {
    for flag in ["--help", "-h"] {
        let output = Command::new(env!("CARGO_BIN_EXE_normalize-punctuation"))
            .arg(flag)
            .output()
            .expect("normalize-punctuation should run");

        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .expect("stdout should be UTF-8")
                .contains("Usage:")
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn version_flags_print_package_version() {
    for flag in ["--version", "-V"] {
        let output = Command::new(env!("CARGO_BIN_EXE_normalize-punctuation"))
            .arg(flag)
            .output()
            .expect("normalize-punctuation should run");

        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .expect("stdout should be UTF-8")
                .contains(env!("CARGO_PKG_VERSION"))
        );
        assert!(output.stderr.is_empty());
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

#[test]
fn normalizes_every_supported_replacement_recursively() {
    const REPLACEMENTS: &[(&str, &str)] = &[
        ("‘", "'"),
        ("’", "'"),
        ("“", "\""),
        ("”", "\""),
        ("ˋ", "`"),
        ("‚", "'"),
        ("„", "\""),
        ("…", "..."),
        ("« ", "\""),
        ("«\u{a0}", "\""),
        ("«\u{202f}", "\""),
        ("«\u{2009}", "\""),
        (" »", "\""),
        ("\u{a0}»", "\""),
        ("\u{202f}»", "\""),
        ("\u{2009}»", "\""),
        ("\u{a0}", "&nbsp;"),
        ("\u{202f}", "&#8239;"),
        ("\u{2009};", "&#8239;;"),
        ("\u{2009}?", "&#8239;?"),
        ("\u{2009}!", "&#8239;!"),
        ("\u{2009}:", "&nbsp;:"),
        ("\u{2009}", "&thinsp;"),
        ("«", "\""),
        ("»", "\""),
        ("‐", "-"),
        ("﹘", "-"),
        ("−", "-"),
        ("–", "-"),
    ];

    let temp_dir = TempDir::new();
    let nested = temp_dir.0.join("nested");
    fs::create_dir(&nested).expect("nested directory should be created");
    let fixtures = REPLACEMENTS
        .iter()
        .enumerate()
        .map(|(index, (input, expected))| {
            let path = nested.join(format!("{index}.md"));
            fs::write(&path, input).expect("replacement fixture should be written");
            (path, *expected)
        })
        .collect::<Vec<_>>();
    let ignored = nested.join("ignored.txt");
    fs::write(&ignored, "“Ignored.”").expect("ignored fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_normalize-punctuation"))
        .arg(&temp_dir.0)
        .output()
        .expect("normalize-punctuation should run");

    assert!(!output.status.success());
    for (path, expected) in fixtures {
        assert_eq!(
            fs::read_to_string(path).expect("normalized fixture should be readable"),
            expected
        );
    }
    assert_eq!(
        fs::read_to_string(ignored).expect("ignored fixture should be readable"),
        "“Ignored.”"
    );
    let expected_summary = format!(
        "Scanned {} files, modified {}. (",
        REPLACEMENTS.len(),
        REPLACEMENTS.len()
    );
    assert!(
        String::from_utf8(output.stdout)
            .expect("stdout should be UTF-8")
            .starts_with(&expected_summary)
    );
}

#[test]
fn defaults_to_current_directory_and_succeeds_when_clean() {
    let temp_dir = TempDir::new();
    let clean = temp_dir.0.join("clean.md");
    fs::write(&clean, "Already normalized.\n").expect("clean fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_normalize-punctuation"))
        .current_dir(&temp_dir.0)
        .output()
        .expect("normalize-punctuation should run");

    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string(clean).expect("clean fixture should be readable"),
        "Already normalized.\n"
    );
    assert!(
        String::from_utf8(output.stdout)
            .expect("stdout should be UTF-8")
            .starts_with("Scanned 1 file, modified 0. (")
    );
}

#[test]
fn fails_without_corrupting_invalid_utf8() {
    let temp_dir = TempDir::new();
    let invalid = temp_dir.0.join("invalid.md");
    let contents = [0xff, 0xfe];
    fs::write(&invalid, contents).expect("invalid UTF-8 fixture should be written");

    let output = Command::new(env!("CARGO_BIN_EXE_normalize-punctuation"))
        .arg(&invalid)
        .output()
        .expect("normalize-punctuation should run");

    assert!(!output.status.success());
    assert_eq!(
        fs::read(invalid).expect("invalid UTF-8 fixture should be readable"),
        contents
    );
}
