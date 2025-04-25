use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};

/// Possible phases for a Codeforces contest.
/// Before is the only relevant phase for upcoming contests.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Phase {
    Before,
    Coding,
    PendingSystemTest,
    SystemTest,
    Finished,
}

/// Struct representing a contest.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Contest {
    /// Unique contest id.
    pub id: usize,
    /// Contest name.
    pub name: String,
    /// Contest phase. Phase::Before means upcoming contest.
    pub phase: Phase,
    /// Start time in seconds (Unix epoch).
    pub start_time_seconds: Option<i64>,
    /// Description of the contest.
    pub description: Option<String>,
}

/// Hashing based on id.
impl Hash for Contest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for Contest {}

/// Comparison based on id.
impl PartialEq for Contest {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Codeforces API response.
#[derive(Deserialize, Debug)]
pub struct ContestResponse {
    /// OK if everything worked, else FAILED.
    pub status: String,
    /// Optional comment if status == FAILED.
    pub comment: Option<String>,
    /// Retrieved contests.
    pub result: Vec<Contest>,
}
