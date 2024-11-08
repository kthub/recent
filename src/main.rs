use std::path::PathBuf;
use std::fs;
use std::io;
use std::process;
use clap::Parser;
use walkdir::WalkDir;
use chrono::{DateTime, Utc, Duration};

#[derive(Parser)]
#[command(version, about = "This program shows recently modified files/directories", long_about = None)]
struct Cli {

    // Target directory
    #[arg(value_name = "Target Directory", help = "File search root. Default: current directory")]
    tdir: Option<String>,

    // Verbose option
    #[arg(short, long, help = "Verbose output")]
    verbose: bool
}

fn main() -> io::Result<()> {
    // parse command line arguments
    let args = Cli::parse();

    let mut target_dir = PathBuf::from(".");
    if let Some(tdir) = args.tdir {
        target_dir = PathBuf::from(tdir);
    }

    // check if the path exists
    if !target_dir.exists() {
        println!("Error: Directory '{}' does not exist.", target_dir.display());
        process::exit(1);
    }
    
    // check if the path is directory
    if !target_dir.is_dir() {
        println!("Error: '{}' is not a directory.", target_dir.display());
        process::exit(1);
    }

    // result vector
    // - PathBuf : Directory/File name
    // - DateTime<Utc> : last modified timestamp
    // - PathBuf : last modified file path in the directory
    let mut result_vector: Vec<(PathBuf, DateTime<Utc>, PathBuf)> = Vec::new();

    // target directory loop
    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check if entry is file or directory
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified_time) = metadata.modified() {
                    let datetime: DateTime<Utc> = modified_time.into();
                    result_vector.push((path.clone(), datetime, path.clone()));
                } else {
                    eprintln!("Can't get updated time of the file.");
                }
            } else {
                eprintln!("Can't get metadata of the file.");
            }
        } else if path.is_dir() {
            let (lmfp, lmft) = get_last_modified_timestamp(&path);
            result_vector.push((path.clone(), lmft, lmfp));
        }
    }

    // sort vector with DateTime<Utc>
    result_vector.sort_by(|a: &(PathBuf, DateTime<Utc>, PathBuf), b| b.1.cmp(&a.1));

    // show result
    for (_path, modified_time, lmfp) in result_vector {
        println!("{} {}", modified_time.format("%Y/%m/%d %H:%M:%S"), lmfp.display());
    }

    Ok(())
}

// get last modified timestamp
fn get_last_modified_timestamp(path: &PathBuf) -> (PathBuf, DateTime<Utc>) {
    let mut last_modified_timestamp: DateTime<Utc> = Utc::now() - Duration::days(365 * 100);
    let mut last_modified_file_path: PathBuf = PathBuf::new();

    for entry in WalkDir::new(path).min_depth(1).max_depth(100) {
        let entry = entry.unwrap();
        let file_path = entry.path();

        if file_path.is_file() {
            if let Ok(metadata) = fs::metadata(file_path) {
                if let Ok(modified_time) = metadata.modified() {
                    let datetime: DateTime<Utc> = modified_time.into();
                    if datetime > last_modified_timestamp {
                        last_modified_timestamp = datetime;
                        last_modified_file_path = file_path.to_path_buf();
                    }
                }
            }
        }
    }

    (last_modified_file_path, last_modified_timestamp)
}
