use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseEntry {
    pub timestamp: u64,
    pub source: String,
    pub publisher: String,
}
