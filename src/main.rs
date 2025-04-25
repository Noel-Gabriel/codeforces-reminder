//! This is a simple crate to fetch upcoming contests
//! from Codeforces using their API and automatically set
//! MacOS Reminders using osascript.
//!
//! The crate manages a contests.json and error_log.txt locally
//! in the path defined by dirs::data_local_dir().join("codeforces-reminder")
//! and get upcoming contests through 
//! Codeforces's API. Finished contests (i.e. contests saved locally but 
//! not present in the upcoming contests retrieved from Codeforces) 
//! will be removed, while new ones will set new
//! reminders and again be saved to the local contests.
//!
//! Also logs errors to error_log.txt in the same directory to facilitate monitoring 
//! when running this binary using cron or another scheduler.

mod contest;
use contest::{Contest, ContestResponse, Phase};
mod paths;

mod local;
use local::fetch_local_upcoming_contests;
use local::log_error;
use local::save_contests_locally;

use reqwest::blocking::{get, Response};
use std::process::Command;
use chrono::{Utc, TimeZone};
use std::collections::HashSet;



fn main() {
    let local_contests   = fetch_local_upcoming_contests();
    let current_upcoming = fetch_current_upcoming_contests(); 

    let new_contests = current_upcoming
        .iter()
        .filter(|contest| !local_contests.contains(contest))
        .cloned()
        .collect::<Vec<Contest>>();

    let mut local_upcoming = local_contests
        .into_iter()
        .filter(|contest| current_upcoming.contains(contest))
        .collect::<Vec<Contest>>();

    for mut contest in new_contests.into_iter() {
        if let Some(start) = contest.start_time_seconds.as_mut() {
            *start -= 1800; // Set reminder 30 minutes earlier
        }
        create_reminder(&contest);
        local_upcoming.push(contest);
    }

    if let Err(e) = save_contests_locally(&local_upcoming) {
        log_error(&format!("Failed to save local contests atomically. Error: {}", e));
    }
}

/// Retrieves upcoming contests 
/// using Codeforces's API as a HashSet.
///
/// Terminates and logs errors if it fails to retrieve the data 
/// or fails to deserialize the JSON.
fn fetch_current_upcoming_contests() -> HashSet<Contest> {
    let url = "https://codeforces.com/api/contest.list?gym=false";

    let response: Response = match get(url) {
        Ok(response) => response,
        Err(e) => {
            log_error(&format!("Could not retrieve online contest list. {}", e));
            std::process::exit(1); 
        }
    };

    let response: ContestResponse = match response.json() {
        Ok(response) => response,
        Err(e) => {
            log_error(&format!("Could not parse online contest JSON. {}", e));
            std::process::exit(1); 
        }
    };

    if response.status != "OK" {
        let comment = response.comment.unwrap_or_else(|| "No comment.".to_string());
        log_error(&format!("Codeforces response status FAILED. Comment: {}.", comment));
        std::process::exit(1);
    }

    response.result
        .into_iter()
        .filter(|contest| contest.phase == Phase::Before)
        .collect::<HashSet<Contest>>()
}

/// Creates a reminder using osascript run as a command.
///
/// This function ignores contests without a starting time 
/// (field start_time_seconds in struct Contest).
///
/// Will not terminate if it fails to set a reminder, but will log the failure.
fn create_reminder(contest: &Contest) {
    let Some(start) = contest.start_time_seconds else {
        log_error(&format!("Contest without start time: {}, {}", contest.id, contest.name));
        return
    };

    let time = Utc.timestamp_opt(start, 0)
        .unwrap()
        .with_timezone(&chrono::Local)
        .format("%d/%m/%Y %H:%M %Z")
        .to_string();

    let apple_script = format!(
        r#"tell application "Reminders"
        set newReminder to make new reminder with properties {{name:"{}, id: {}", body:"{:?}"}}
        set due date of newReminder to date "{}"
        end tell"#, contest.name, contest.id, contest.description, time);

    let status = Command::new("osascript")
        .arg("-e")
        .arg(apple_script)
        .status();

    if let Err(ref e) = status {
        log_error(&format!("Failed to run osascript for contest {}, id: {}. Error: {}", contest.name, contest.id, e));
        return;
    }

    if !status.unwrap().success() {
        log_error(&format!("Failed to add reminder for Contest {}, id: {}", contest.name, contest.id));
    }
}
