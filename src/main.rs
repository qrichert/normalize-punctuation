use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use normalize_punctuation::{utils, walk};

const PATH: &str = "/Users/Quentin/Developer/knowledge";

macro_rules! plural {
    ($count:expr) => {
        if $count == 1 { "" } else { "s" }
    };
}

fn main() -> ExitCode {
    let start = Instant::now();

    let nb_files = AtomicUsize::new(0);
    let nb_modified = AtomicUsize::new(0);

    walk::find_files_recursively(PATH, &["md"], |p| {
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

fn normalize_file(buffer: &mut String, path: &Path) -> Result<(), ()> {
    const REPLACEMENTS: [(&str, &str); 16] = [
        ("‘", "'"),
        ("’", "'"),
        ("“", "\""),
        ("”", "\""),
        ("‚", "'"),
        ("„", "\""),
        ("…", "..."),
        ("\u{a0}", " "),
        // ("\u{202f}", ""),
        ("« ", "\""),
        ("«", "\""),
        (" »", "\""),
        ("»", "\""),
        ("‐", "-"),
        ("﹘", "-"),
        ("−", "-"),
        ("–", "—"),
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
