use std::path::Path;

use ignore::{self, DirEntry, WalkBuilder, WalkState};

pub fn find_files_recursively(
    root: impl AsRef<Path>,
    extensions: &[&str],
    f: impl Fn(&Path) + Sync,
) {
    let does_entry_match = move |path: &Path| {
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            return false;
        };
        extensions.contains(&extension)
    };

    let root = root.as_ref();

    // Single file.
    if root.is_file() && does_entry_match(root) {
        f(root);
        return;
    }

    WalkBuilder::new(root)
        .follow_links(true)
        .hidden(true)
        .max_depth(None)
        .build_parallel()
        .run(|| {
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
