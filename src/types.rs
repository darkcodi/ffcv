use std::collections::HashMap;

/// Main output structure for the Firefox configuration
/// This is a type alias for the preferences HashMap to output at root level
pub type Config = HashMap<String, serde_json::Value>;
