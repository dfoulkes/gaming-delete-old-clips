use clap::Parser;
use std::error::Error;
use std::fs::metadata;
use std::time::SystemTime;
use walkdir::WalkDir;
use chrono::{Duration, Utc};
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the directory to be scanned.
    #[arg(short, long)]
    path: String,
    /// The file extension to look for. Defaults to "mp4" if not provided.
    #[arg(short, long, default_value = "mp4")]
    file_extension: Option<String>,
    /// The minimum number of files that must be present in the directory to trigger the deletion.
    #[arg(short, long, default_value = "10")]
    min_files: u16,
    /// The number of days after which the files should be deleted.
    #[arg(short, long, default_value = "14")]
    keep_days: i64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.keep_days < 1 {
        return Err("keep_days must be greater than 0".into());
    }

    let files = find_mp4s_that_are_older_max_age(args.path, args.file_extension, args.min_files, args.keep_days)?;
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

fn get_files_with_extension(path: String, file_extension: String) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(move |entry| entry.file_type().is_file())
        .filter(move |entry| entry.path().extension()
            .and_then(|ext| Some(ext.to_string_lossy() == file_extension))
            .unwrap_or(false))
}

fn get_count_of_files(path: String, file_extension: String) -> Result<u16, Box<dyn Error>> {
    let count = get_files_with_extension(path, file_extension).count() as u16;
    Ok(count)
}

fn find_mp4s_that_are_older_max_age(path: String, file_extension: Option<String>, min_files: u16, keep_days: i64) -> Result<Vec<String>, Box<dyn Error>> {
    let file_extension = file_extension.unwrap_or("mp4".to_string());
    let epoch_of_the_oldest_date = (Utc::now() - Duration::days(keep_days)).timestamp() as u64;
    let count :u16 = get_count_of_files(path.clone(), file_extension.clone())?;
    let mut files = vec![];
    if count > min_files {
        let mut new_files: Vec<(String, u64)> = get_files_with_extension(path.clone(), file_extension.clone())
            .filter_map(|entry| metadata(entry.path()).ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|modified_time| modified_time.duration_since(SystemTime::UNIX_EPOCH).ok())
                .and_then(|duration| Some((entry.path().to_string_lossy().into_owned(), duration.as_secs()))))
            .collect();

        // Sort files by modification time in descending order
        new_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Skip the first min_files number of files
        let old_files = new_files.into_iter().skip(min_files as usize);

        // Filter files that are older than fourteen_days_ago
        let old_files = old_files.filter(|(_, time)| *time < epoch_of_the_oldest_date)
            .map(|(file, _)| file)
            .collect::<Vec<String>>();

        files.extend(old_files);
    }
    Ok(files)
}