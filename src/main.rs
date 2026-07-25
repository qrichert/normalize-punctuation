use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::{env, fs};

use normalize_punctuation::{normalize, utils, walk};

macro_rules! plural {
    ($count:expr) => {
        if $count == 1 { "" } else { "s" }
    };
}

fn main() -> ExitCode {
    let start = Instant::now();

    let mut paths = get_paths_from_args(env::args());
    if paths.is_empty() {
        let Some(path) = get_cwd() else {
            eprintln!(
                "\
Could not determine current working directory.
Please provide a directory or a file as argument.
"
            );
            return ExitCode::FAILURE;
        };
        paths.push(path);
    }

    let nb_files = AtomicUsize::new(0);
    let nb_modified = AtomicUsize::new(0);

    walk::find_files_recursively_many(paths, &["md"], |p| {
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

fn get_paths_from_args(args: impl Iterator<Item = String>) -> Vec<PathBuf> {
    args.skip(1).map(PathBuf::from).collect()
}

fn get_cwd() -> Option<PathBuf> {
    env::current_dir().ok()
}

fn normalize_file(buffer: &mut String, path: &Path) -> Result<(), ()> {
    if utils::read_to_string_buffer(buffer, path).is_err() {
        return Err(());
    }

    match normalize::normalize_str(buffer) {
        Some(normalized) => {
            _ = fs::write(path, normalized);
            Err(())
        }
        None => Ok(()),
    }
}
