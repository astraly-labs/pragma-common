#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize,))]
pub struct BaseEntry {
    pub timestamp: i64,
    pub source: String,
    pub publisher: String,
}
