use crate::contest::Contest;
use crate::paths::contest_path;
use crate::paths::log_file;

use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::fs::{self, File};

/// Deserializes and returns the locally saved contests in contests.json.
///
/// This function panics if contests.json exists, but fails
/// to read it or parse it.
pub fn fetch_local_upcoming_contests() -> HashSet<Contest> {
    let path = contest_path();

    if !path.exists() { return HashSet::new() }

    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log_error(&format!("Failed to read local contests file: {}", e));
            std::process::exit(1);
        }
    };

    match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(e) => {
            log_error(&format!("Failed to parse contests JSON: {}", e));
            std::process::exit(1);
        }
    }
}

/// Function used to log errors.
///
/// Panics if it fails to write or read.
pub fn log_error(msg: &str) {
    let mut file = log_file(); 

    file.write_all(format!("{:?}: {}\n", chrono::offset::Local::now(), msg).as_bytes()).expect("Could not write to file");
}

/// Serializes the contests and tries to save them locally.
///
/// This function is guaranteed to either succeed in saving the new contests, or
/// keeping the old locally saved contests.
pub fn save_contests_locally(contests: &Vec<Contest>) -> std::io::Result<()> {
    let serialized = serde_json::to_string_pretty(&contests)?;
    save_contests_atomically(&serialized)
}

/// Function to save contests locally.
/// It saves contests by writing to a temporary file and then overwriting the
/// contests.json atomically (using the filesystem) to preserve old contests in case
/// of failure.
fn save_contests_atomically(data: &str) -> std::io::Result<()> {
    let path = contest_path();
    let temp_path = path.with_extension("tmp");

    let file = File::create(&temp_path)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(data.as_bytes())?;
    writer.flush()?;
    writer.get_ref().sync_all()?; 

    fs::rename(&temp_path, path)?; 
    Ok(())
}
