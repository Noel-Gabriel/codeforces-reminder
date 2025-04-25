use std::path::PathBuf;
use std::sync::OnceLock;
use std::fs::{self, File};
use std::io::{Write, BufReader, BufRead};

static CONTEST_FILE_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOG_FILE: OnceLock<File> = OnceLock::new();

const CONTEST_FILE_NAME: &str = "contests.json";
const LOG_FILE_NAME: &str = "error_log.txt";
const MAX_LOG_LINES: usize = 2000;

/// Returns the path to the contests.json
///
/// e.g. /Users/noel/Library/Application Support/codeforces-reminder/contests.json
/// on MacOS in my case.
pub fn contest_path() -> &'static PathBuf {
    CONTEST_FILE_PATH.get_or_init(build_contest_path) 
}

/// Creates the folder "codeforces-reminder" in
/// the path provided by data_local_dir if it does not exist.
///
/// Additionally tries to add the file contests.json as an empty json if
/// it does not exist, but does not panic if it fails.
fn build_contest_path() -> PathBuf {
    let data_dir = dirs::data_local_dir()
            .expect("Could not find data dir in .local")
            .join("codeforces-reminder");

    std::fs::create_dir_all(&data_dir).expect("Failed to create data dir");

    let contest_path = data_dir.join(CONTEST_FILE_NAME);

    if !contest_path.exists() {
        match File::create(&contest_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(b"[]") {
                    let _ = fs::remove_file(&contest_path);
                    crate::log_error(&format!("Could not initialize empty JSON file. {}", e));
                }
            }
            Err(e) => {
                crate::local::log_error(&format!("Failed to create initial contests file: {}", e));
            }
        }
    }

    contest_path
}

/// Returns the current open handle to the log file.
///
/// Log file is saved in same directory as contests.json.
pub fn log_file() -> &'static File {
    LOG_FILE.get_or_init(get_log_handle)
}

/// Creates the folder "codeforces-reminder" in
/// the path provided by data_local_dir if it does not exist.
///
/// Creates (or opens) the file error_log in said folder and returns the
/// file handle. 
fn get_log_handle() -> File {
    let state_dir = dirs::data_local_dir()
            .expect("Could not find data dir in .local")
            .join("codeforces-reminder");

    std::fs::create_dir_all(&state_dir).expect("Failed to create data dir");

    let log_path = state_dir.join(LOG_FILE_NAME);
   
    if log_path.exists() {
        // Check line number, delete logs if number reaches certain threshold
        // to not let errors pile up
        if let Ok(file) = fs::OpenOptions::new()
            .read(true)
            .open(&log_path) {

            let reader = BufReader::new(&file);
            let line_count = reader.lines().count();

            if line_count > MAX_LOG_LINES {
                return fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&log_path)
                    .expect("Failed to truncate log file");
            }
        }
    }   

    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Could not open or create log file")   
}
