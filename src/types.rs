use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Firefox preference types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrefType {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "locked")]
    Locked,
    #[serde(rename = "sticky")]
    Sticky,
}

/// Internal type for parser that always includes pref type
#[derive(Debug, Clone)]
pub struct PrefEntry {
    pub value: serde_json::Value,
    pub pref_type: PrefType,
}

/// Main output structure for the Firefox configuration
/// This is a type alias for the preferences HashMap to output at root level
pub type Config = HashMap<String, serde_json::Value>;

/// Representation for array output format
#[derive(Debug, Clone, Serialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pref_type: Option<PrefType>,
}
