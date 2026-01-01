//! Firefox preference explanations module
//!
//! This module contains a database of explanations for Firefox preferences.
//! Explanations are stored in a static HashMap for efficient lookup.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Static lookup table for Firefox preference explanations
///
/// Add new explanations here to include them in JSON array output.
///
/// # Writing Good Explanations
/// - Describe what the preference controls
/// - Mention the effects of different values (especially for booleans and enums)
/// - Note any potential side effects or breaking changes
/// - Keep explanations concise but informative
/// - Use clear, non-technical language when possible
///
/// # Example
/// ```rust
/// HashMap::from([
///     ("javascript.enabled", "Master switch to enable or disable JavaScript..."),
///     ("browser.startup.homepage", "The default homepage that Firefox opens with..."),
/// ])
/// ```
static PREF_EXPLANATIONS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

/// Get explanation for a preference key
///
/// Returns `Some` with the explanation text if available, otherwise returns `None`.
///
/// # Arguments
/// * `key` - The preference key to look up (e.g., "javascript.enabled")
///
/// # Returns
/// `Some(String)` containing the explanation, or `None` if not found
///
/// # Example
/// ```rust
/// if let Some(explanation) = get_preference_explanation("javascript.enabled") {
///     assert!(explanation.contains("JavaScript"));
/// }
/// ```
pub fn get_preference_explanation(key: &str) -> Option<String> {
    PREF_EXPLANATIONS
        .get_or_init(|| {
            // Add more explanations here following the pattern above:
            // ("preference.name", "Explanation of what this preference does..."),
            //
            // Example format for different preference types:
            // - Boolean: "When true, X happens. When false, Y happens."
            // - Integer: "Sets the value for X. Valid values are 0-5 where..."
            // - String: "Specifies the X. Use Y format for..."
            HashMap::from([
                (
                    "javascript.enabled",
                    "Master switch to enable or disable JavaScript execution in Firefox. \
              When set to true, JavaScript can run in web pages. When false, JavaScript is \
              completely disabled, which may break many modern websites that rely on JavaScript \
              for functionality.",
                ),
                (
                    "privacy.trackingprotection.enabled",
                    "Enables Firefox's built-in tracking protection feature to block online trackers. \
              When set to true, Firefox blocks known tracking scripts and cookies from third-party \
              trackers, enhancing privacy while browsing. When false, tracking protection is disabled \
              and trackers may monitor your browsing activity across websites.",
                ),
            ])
        })
        .get(key)
        .map(|s| s.to_string())
}
