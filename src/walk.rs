use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

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

    let roots = roots
        .into_iter()
        .map(|root| root.as_ref().to_path_buf())
        .collect::<Vec<_>>();
    let Some((root, other_roots)) = roots.split_first() else {
        return;
    };

    // Single file.
    if other_roots.is_empty() && root.is_file() {
        if does_entry_match(root) {
            f(root);
        }
        return;
    }

    let mut builder = WalkBuilder::new(root);
    for root in other_roots {
        builder.add(root);
    }
    builder.follow_links(true).hidden(true).max_depth(None);
    // Only guard against duplicate visits when the roots can actually reach the
    // same file, so the common case (a single root, or disjoint roots) stays
    // lock-free. Accepted gaps: hard links (distinct canonical paths, so not
    // deduplicated) and a single directory root whose tree contains an internal
    // symlink to another file within it (the gate is off for a single root).
    // Both are rare and, given `f` writes files, at worst cause redundant work.
    let visited_files = do_roots_overlap(&roots).then(|| Mutex::new(HashSet::new()));
    builder.build_parallel().run(|| {
        Box::new(|entry| {
            if let Ok(entry) = entry {
                if is_dir(&entry) {
                    return WalkState::Continue;
                }
                let path = entry.path();
                if does_entry_match(path) {
                    if is_first_visit(visited_files.as_ref(), path) {
                        f(path);
                    }
                    return WalkState::Continue;
                }
            }
            WalkState::Continue
        })
    });
}

fn do_roots_overlap(roots: &[PathBuf]) -> bool {
    if roots.len() < 2 {
        return false;
    }

    let roots = roots
        .iter()
        .map(|path| canonical_path(path))
        .collect::<Vec<_>>();

    roots.iter().enumerate().any(|(index, root)| {
        roots[index + 1..]
            .iter()
            .any(|other| root.starts_with(other) || other.starts_with(root))
    })
}

fn is_dir(entry: &DirEntry) -> bool {
    entry.file_type().is_some_and(|entry| entry.is_dir())
}

/// Records `path` as visited and reports whether this was the first time it was
/// seen, keying on canonical path so a file reached through several roots is
/// only handled once. When `visited` is `None` (the roots cannot overlap) every
/// visit is treated as the first. Canonicalization happens outside the lock so
/// workers only contend on the `HashSet` insert.
fn is_first_visit(visited: Option<&Mutex<HashSet<PathBuf>>>, path: &Path) -> bool {
    let Some(visited) = visited else {
        return true;
    };
    let key = canonical_path(path);
    visited
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(key)
}

/// Returns the canonical form of `path`, falling back to `path` itself when it
/// cannot be resolved (e.g. it no longer exists).
fn canonical_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
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

        assert!(!do_roots_overlap(&[
            first_root.clone(),
            second_root.clone()
        ]));

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
    fn visits_files_only_once_for_overlapping_roots() {
        let temp_dir = TempDir::new();
        let nested = temp_dir.0.join("nested");
        fs::create_dir(&nested).expect("nested directory should be created");
        let markdown = nested.join("file.md");
        fs::write(&markdown, "markdown").expect("Markdown fixture should be written");

        assert!(do_roots_overlap(&[temp_dir.0.clone(), markdown.clone()]));

        let visits = AtomicUsize::new(0);
        find_files_recursively_many(
            [
                temp_dir.0.clone(),
                nested,
                markdown.clone(),
                temp_dir.0.clone(),
            ],
            &["md"],
            |_| {
                visits.fetch_add(1, Ordering::Relaxed);
            },
        );

        assert_eq!(visits.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn handles_no_roots() {
        let called = AtomicBool::new(false);

        find_files_recursively_many(std::iter::empty::<PathBuf>(), &["md"], |_| {
            called.store(true, Ordering::Relaxed);
        });

        assert!(!called.load(Ordering::Relaxed));
    }

    #[test]
    fn canonical_path_resolves_indirect_paths() {
        let temp_dir = TempDir::new();
        let nested = temp_dir.0.join("nested");
        fs::create_dir(&nested).expect("nested directory should be created");
        let markdown = nested.join("file.md");
        fs::write(&markdown, "markdown").expect("Markdown fixture should be written");

        // Same file, reached through a redundant `..` component.
        let indirect = nested.join("..").join("nested").join("file.md");

        assert_eq!(canonical_path(&indirect), canonical_path(&markdown));
    }

    #[test]
    fn canonical_path_falls_back_for_unresolvable_paths() {
        let temp_dir = TempDir::new();
        let missing = temp_dir.0.join("does-not-exist.md");

        // Canonicalization fails, so the input path is returned unchanged.
        assert_eq!(canonical_path(&missing), missing);
    }

    #[test]
    fn is_first_visit_without_tracking_is_always_true() {
        let path = PathBuf::from("any.md");

        assert!(is_first_visit(None, &path));
        assert!(is_first_visit(None, &path));
    }

    #[test]
    fn is_first_visit_deduplicates_by_canonical_path() {
        let temp_dir = TempDir::new();
        let nested = temp_dir.0.join("nested");
        fs::create_dir(&nested).expect("nested directory should be created");
        let markdown = nested.join("file.md");
        fs::write(&markdown, "markdown").expect("Markdown fixture should be written");
        let alias = nested.join("..").join("nested").join("file.md");

        let visited = Mutex::new(HashSet::<PathBuf>::new());

        assert!(is_first_visit(Some(&visited), &markdown));
        assert!(!is_first_visit(Some(&visited), &markdown)); // Exact repeat.
        assert!(!is_first_visit(Some(&visited), &alias)); // Same canonical path.
    }
}
