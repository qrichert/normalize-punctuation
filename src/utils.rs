//! Miscellaneous helper functions.

use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use pathdiff;

/// Return path relative to current working directory.
///
/// ```no_run
/// # use std::path::Path;
/// # use normalize_punctuation::utils::path_relative_to_cwd;
/// // CWD = `/home/quentin/`
/// let file = Path::new("/home/quentin/docs/index.md");
/// assert_eq!(
///     path_relative_to_cwd(file),
///     Path::new("docs/index.md")
/// );
#[must_use]
pub fn path_relative_to_cwd(path: &Path) -> Cow<Path> {
    // Better some sync overhead once, than to call `env::current_dir()`
    // repeatedly.
    static CWD: OnceLock<Option<PathBuf>> = OnceLock::new();
    let Some(base_dir) = CWD.get_or_init(|| {
        #[cfg(test)]
        {
            return Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        }
        #[allow(unreachable_code)]
        env::current_dir().ok()
    }) else {
        // Unreachable in tests.
        #[cfg(not(tarpaulin_include))]
        return Cow::Borrowed(path);
    };

    if let Some(diffed) = pathdiff::diff_paths(path, base_dir) {
        if diffed == Path::new("") {
            Cow::Borrowed(path)
        } else {
            Cow::Owned(diffed)
        }
    } else {
        Cow::Borrowed(path)
    }
}

/// Read to pre-allocated `String` buffer.
///
/// Same as [`std::fs::read_to_string()`], but doesn't allocate if it
/// doesn't need to increase the buffer size (and bypasses the metadata
/// check for size-hint, since the buffer is supposed to be big enough).
///
/// # Errors
///
/// Errors if file cannot be read.
pub fn read_to_string_buffer(buffer: &mut String, path: &Path) -> io::Result<usize> {
    let mut file = File::open(path)?;
    buffer.clear();
    file.read_to_string(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CWD: &str = env!("CARGO_MANIFEST_DIR");

    #[test]
    fn path_relative_to_cwd_regular() {
        let file = Path::new(CWD).join("foo/bar.txt");

        assert_eq!(path_relative_to_cwd(&file), Path::new("foo/bar.txt"));
    }

    #[test]
    fn path_relative_to_cwd_equal() {
        let file = Path::new(CWD);

        assert_eq!(path_relative_to_cwd(file), file);
    }

    #[test]
    fn path_relative_to_cwd_parent() {
        let file = Path::new(CWD).parent().unwrap().join("foo/bar.txt");

        assert_eq!(path_relative_to_cwd(&file), Path::new("../foo/bar.txt"));
    }

    #[test]
    fn path_relative_to_cwd_relative_path() {
        let file = Path::new("foo/bar.txt");

        // Can't make a relative path relative.
        assert_eq!(path_relative_to_cwd(file), file);
    }
}
