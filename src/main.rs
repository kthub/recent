use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::process;
use clap::Parser;
use walkdir::WalkDir;
use chrono::{DateTime, Local};
use rayon::prelude::*;

#[derive(Parser)]
#[command(version, about = "This program shows recently modified files/directories", long_about = None)]
struct Cli {
    #[arg(value_name = "Target Directory", help = "File search root. Default: current directory")]
    tdir: Option<String>,

    #[arg(short, long, help = "Verbose output (show top-level entry with last modified file)")]
    verbose: bool,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    let target_dir = args.tdir
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    if !target_dir.exists() {
        eprintln!("Error: Directory '{}' does not exist.", target_dir.display());
        process::exit(1);
    }

    if !target_dir.is_dir() {
        eprintln!("Error: '{}' is not a directory.", target_dir.display());
        process::exit(1);
    }

    let entries: Vec<PathBuf> = fs::read_dir(&target_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.file_name().map_or(true, |n| n != ".DS_Store"))
        .collect();

    // (modified_time, top_level_entry, last_modified_file_within_dir)
    let mut result: Vec<(DateTime<Local>, PathBuf, Option<PathBuf>)> = entries
        .par_iter()
        .filter_map(|path| {
            if path.is_file() {
                let datetime = file_modified_time(path)?;
                Some((datetime, path.clone(), None))
            } else if path.is_dir() {
                let (lmfp, lmft) = get_last_modified_timestamp(path)?;
                Some((lmft, path.clone(), Some(lmfp)))
            } else {
                None
            }
        })
        .collect();

    result.sort_by(|a, b| b.0.cmp(&a.0));

    for (modified_time, entry_path, lmfp) in result {
        if args.verbose {
            let time_str = modified_time.format("%Y/%m/%d %H:%M:%S %Z");
            match lmfp {
                Some(ref inner) => println!("{} {} [last: {}]", time_str, entry_path.display(), inner.display()),
                None => println!("{} {}", time_str, entry_path.display()),
            }
        } else {
            println!("{} {}", modified_time.format("%Y/%m/%d %H:%M:%S"), entry_path.display());
        }
    }

    Ok(())
}

fn file_modified_time(path: &Path) -> Option<DateTime<Local>> {
    fs::metadata(path).ok()?.modified().ok().map(|t| t.into())
}

const SKIP_DIRS: &[&str] = &[
    ".git", ".svn", ".hg",
    "node_modules",
    "target",
    "vendor",
    "__pycache__", ".venv", "venv",
    "dist", "build", ".next", ".nuxt",
];

fn get_last_modified_timestamp(path: &Path) -> Option<(PathBuf, DateTime<Local>)> {
    WalkDir::new(path)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            // depth==0 はルート自身なので常に通す
            if e.depth() == 0 {
                return true;
            }
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                return !SKIP_DIRS.contains(&name.as_ref());
            }
            true
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.file_name() != ".DS_Store")
        .filter_map(|e| {
            let datetime = file_modified_time(e.path())?;
            Some((e.path().to_path_buf(), datetime))
        })
        .max_by_key(|(_, dt)| *dt)
}
