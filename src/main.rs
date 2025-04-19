use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::{env, fs};

use normalize_punctuation::{utils, walk};

macro_rules! plural {
    ($count:expr) => {
        if $count == 1 { "" } else { "s" }
    };
}

fn main() -> ExitCode {
    let start = Instant::now();

    let Some(path) = get_path_from_args(env::args()).or_else(get_cwd) else {
        eprintln!(
            "\
Could not determine current working directory.
Please provide a directory or a file as argument.
"
        );
        return ExitCode::FAILURE;
    };

    let nb_files = AtomicUsize::new(0);
    let nb_modified = AtomicUsize::new(0);

    walk::find_files_recursively(path, &["md"], |p| {
        thread_local! {
            static BUFFER: RefCell<String> = RefCell::new(String::with_capacity(100_000))
        }

        nb_files.fetch_add(1, Ordering::Relaxed);

        if BUFFER
            .with_borrow_mut(|buffer| normalize_file(buffer, p))
            .is_err()
        {
            eprintln!("{}", utils::path_relative_to_cwd(p).display());
            nb_modified.fetch_add(1, Ordering::Relaxed);
        }
    });

    let nb_files = nb_files.into_inner();
    let nb_modified = nb_modified.into_inner();

    println!(
        "Scanned {nb_files} file{}, modified {nb_modified}. ({:.3}s)",
        plural!(nb_files),
        start.elapsed().as_secs_f64()
    );

    if nb_modified == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn get_path_from_args(mut args: impl Iterator<Item = String>) -> Option<PathBuf> {
    args.nth(1).map(PathBuf::from)
}

fn get_cwd() -> Option<PathBuf> {
    env::current_dir().ok()
}

fn normalize_file(buffer: &mut String, path: &Path) -> Result<(), ()> {
    const REPLACEMENTS: [(&str, &str); 17] = [
        ("‘", "'"),
        ("’", "'"),
        ("“", "\""),
        ("”", "\""),
        ("ˋ", "`"), // Grave accent.
        ("‚", "'"),
        ("„", "\""),
        ("…", "..."),
        ("\u{a0}", " "), // NBSP
        // ("\u{202f}", ""), // NNBSP (narrow).
        ("« ", "\""),
        ("«", "\""),
        (" »", "\""),
        ("»", "\""),
        ("‐", "-"),
        ("﹘", "-"),
        ("−", "-"),
        ("–", "-"), // en-dash.
    ];

    if utils::read_to_string_buffer(buffer, path).is_err() {
        return Err(());
    }

    // `replace()` allocates a new `String`. Leave `normalized` empty
    // (i.e., don't allocate) unless we _know_ we need to (replacement).
    let mut normalized = String::new();
    let mut modified = false;

    for (pattern, replacement) in REPLACEMENTS {
        if modified {
            // Use already-normalized version.
            if normalized.contains(pattern) {
                normalized = normalized.replace(pattern, replacement);
            }
        } else {
            // Reuse unchanged buffer.
            if buffer.contains(pattern) {
                // Delegate to `normalized`.
                normalized = buffer.replace(pattern, replacement);
                modified = true;
            }
        }
    }

    if modified {
        _ = fs::write(path, normalized);
        Err(())
    } else {
        Ok(())
    }
}
