use std::path::Path;

use ignore::{self, DirEntry, WalkBuilder, WalkState};

pub fn find_files_recursively(
    root: impl AsRef<Path>,
    extensions: &[&str],
    f: impl Fn(&Path) + Sync,
) {
    find_files_recursively_many([root], extensions, f);
}

pub fn find_files_recursively_many(
    roots: impl IntoIterator<Item = impl AsRef<Path>>,
    extensions: &[&str],
    f: impl Fn(&Path) + Sync,
) {
    let does_entry_match = move |path: &Path| {
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return false;
        };
        extensions.contains(&extension)
    };

    let mut roots = roots.into_iter().peekable();
    let Some(root) = roots.next() else {
        return;
    };

    // Single file.
    if roots.peek().is_none() && root.as_ref().is_file() {
        if does_entry_match(root.as_ref()) {
            f(root.as_ref());
        }
        return;
    }

    let mut builder = WalkBuilder::new(root);
    for root in roots {
        builder.add(root);
    }
    builder.follow_links(true).hidden(true).max_depth(None);
    builder.build_parallel().run(|| {
        Box::new(|entry| {
            if let Ok(entry) = entry {
                if is_dir(&entry) {
                    return WalkState::Continue;
                }
                let path = entry.path();
                if does_entry_match(path) {
                    f(path);
                    return WalkState::Continue;
                }
            }
            WalkState::Skip
        })
    });
}

fn is_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_some_and(|entry| entry.is_dir())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

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
                "normalize-punctuation-walk-{}-{suffix}-{counter}",
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
    fn finds_matching_files_recursively() {
        let temp_dir = TempDir::new();
        let nested = temp_dir.0.join("nested");
        fs::create_dir(&nested).expect("nested directory should be created");
        let root_markdown = temp_dir.0.join("root.md");
        let nested_markdown = nested.join("nested.md");
        fs::write(&root_markdown, "root").expect("root fixture should be written");
        fs::write(&nested_markdown, "nested").expect("nested fixture should be written");
        fs::write(nested.join("other.txt"), "other").expect("text fixture should be written");
        fs::write(nested.join("no-extension"), "other")
            .expect("extensionless fixture should be written");

        let found = Mutex::new(BTreeSet::new());
        find_files_recursively(&temp_dir.0, &["md"], |path| {
            found
                .lock()
                .expect("found-files lock should not be poisoned")
                .insert(path.to_path_buf());
        });

        assert_eq!(
            found
                .into_inner()
                .expect("found-files lock should not be poisoned"),
            BTreeSet::from([root_markdown, nested_markdown])
        );
    }

    #[test]
    fn handles_single_files() {
        let temp_dir = TempDir::new();
        let markdown = temp_dir.0.join("file.md");
        let text = temp_dir.0.join("file.txt");
        fs::write(&markdown, "markdown").expect("Markdown fixture should be written");
        fs::write(&text, "text").expect("text fixture should be written");

        let found = Mutex::new(Vec::new());
        find_files_recursively(&markdown, &["md"], |path| {
            found
                .lock()
                .expect("found-files lock should not be poisoned")
                .push(path.to_path_buf());
        });
        find_files_recursively(&text, &["md"], |path| {
            found
                .lock()
                .expect("found-files lock should not be poisoned")
                .push(path.to_path_buf());
        });

        assert_eq!(
            found
                .into_inner()
                .expect("found-files lock should not be poisoned"),
            [markdown]
        );
    }

    #[test]
    fn handles_multiple_roots() {
        let temp_dir = TempDir::new();
        let first_root = temp_dir.0.join("first");
        let second_root = temp_dir.0.join("second");
        fs::create_dir(&first_root).expect("first root should be created");
        fs::create_dir(&second_root).expect("second root should be created");
        let first_markdown = first_root.join("first.md");
        let second_markdown = second_root.join("second.md");
        fs::write(&first_markdown, "first").expect("first fixture should be written");
        fs::write(&second_markdown, "second").expect("second fixture should be written");

        let found = Mutex::new(BTreeSet::new());
        find_files_recursively_many([first_root, second_root], &["md"], |path| {
            found
                .lock()
                .expect("found-files lock should not be poisoned")
                .insert(path.to_path_buf());
        });

        assert_eq!(
            found
                .into_inner()
                .expect("found-files lock should not be poisoned"),
            BTreeSet::from([first_markdown, second_markdown])
        );
    }

    #[test]
    fn handles_no_roots() {
        let called = AtomicBool::new(false);

        find_files_recursively_many(std::iter::empty::<PathBuf>(), &["md"], |_| {
            called.store(true, Ordering::Relaxed);
        });

        assert!(!called.load(Ordering::Relaxed));
    }
}
