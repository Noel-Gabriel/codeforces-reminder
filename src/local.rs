use crate::contest::Contest;

use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::path::{PathBuf, Path};
use std::fs::{self, File};

const CONTEST_FILE_NAME: &str = "contests.json";
// const CONTEST_TEST_FILE_NAME: &str = "test.json";
const LOG_FILE_NAME: &str = "error_log.txt";
const LOG_MAX_LINE_NUMBER: u32 = 1000;

/// Looks for local JSON file named (default) "contests.json" containing
/// currently saved contests.
///
/// This function terminates the program if contests.json exists, but fails
/// to read it or parse it.
///
/// Logs all errors.
pub fn fetch_local_upcoming_contests() -> HashSet<Contest> {
    let path = PathBuf::from(CONTEST_FILE_NAME);

    if !path.exists() {
        let new_contests = HashSet::new();

        if let Err(e) = fs::write(&path, serde_json::to_string_pretty(&new_contests).unwrap()) {
            log_error(&format!("Failed to create and write initial contests file: {}", e));
        }

        return new_contests;
    }

    let contents = match fs::read_to_string(&path) {
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
/// Errors are saved locally in error_log.txt, which is emptied automatically
/// once it reaches (default) 1000 lines (i.e. 1000 errors, should not be happening soon).
/// Terminates the program if it fails to write or read.
pub fn log_error(msg: &str) {
    let path = PathBuf::from(LOG_FILE_NAME);

    let mut file = if path.exists() {
        let log = fs::read_to_string(&path).expect("Failed to read log");
        if log.lines().count() as u32 > LOG_MAX_LINE_NUMBER {
            fs::File::create(&path).expect("Could not create file");        
        }     

        fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("Could not open file for appending")

    } else {
        fs::File::create(&path).expect("Could not create file")
    };
    
    file.write_all(format!("{:?}: {}\n", chrono::offset::Local::now(), msg).as_bytes()).expect("Could not write to file");
}

/// Serializes the contests and tries to save them locally.
///
/// This function is guaranteed to either succeed in saving the new contests, or
/// keeping the old locally saved contests.
pub fn save_contests_locally(contests: &Vec<Contest>) -> std::io::Result<()> {
    let serialized = serde_json::to_string_pretty(&contests)?;
    save_contests_atomically(CONTEST_FILE_NAME, &serialized)
}

/// Function to save contests locally.
/// It saves contests by writing to a temporary file and then overwriting the
/// contests.json atomically (using the filesystem) to preserve old contests in case
/// of failure.
fn save_contests_atomically<P: AsRef<Path>>(path: P, data: &str) -> std::io::Result<()> {
    let temp_path = path.as_ref().with_extension("tmp");

    let file = File::create(&temp_path)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(data.as_bytes())?;
    writer.flush()?;
    writer.get_ref().sync_all()?; 

    fs::rename(&temp_path, &path)?; 
    Ok(())
}
