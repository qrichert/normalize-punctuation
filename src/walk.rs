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
