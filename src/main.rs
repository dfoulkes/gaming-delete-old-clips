use clap::Parser;
use std::error::Error;
use std::fs;
use std::fs::metadata;
use std::time::SystemTime;
use walkdir::WalkDir;
use chrono::{Duration, Utc};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the directory to be scanned.
    #[arg(short, long)]
    path: String,
    /// The file extension to look for. Defaults to "mp4" if not provided.
    #[arg(short, long, default_value = "mp4")]
    file_extension: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let files = find_mp4s_that_are_older_then_14_days(args.path, args.file_extension)?;
    delete_files(files)?;
    Ok(())
}

fn delete_files(files: Vec<String>) -> Result<(), Box<dyn Error>> {
    for file in files {
      //  fs::remove_file(file)?;
        println!("deleting file: {}", file); // remove the file
        fs::remove_file(file)?; // remove the file
    }
    Ok(())
}

fn find_mp4s_that_are_older_then_14_days(path: String, file_extension: Option<String>) -> Result<Vec<String>, Box<dyn Error>> {
    let file_extension = file_extension.unwrap_or("mp4".to_string());
    let fourteen_days_ago = (Utc::now() - Duration::days(14)).timestamp() as u64;

    let files: Vec<String> = WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension()
            .and_then(|ext| Some(ext.to_string_lossy() == file_extension))
            .unwrap_or(false))
        .filter_map(|entry| metadata(entry.path()).ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified_time| modified_time.duration_since(SystemTime::UNIX_EPOCH).ok())
            .and_then(|duration| Some(duration.as_secs() < fourteen_days_ago))
            .and_then(|is_older_than_14_days| if is_older_than_14_days { Some(entry.path().to_string_lossy().into_owned()) } else { None }))
        .collect();

    Ok(files)
}