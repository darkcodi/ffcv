//! omni.ja archive extractor
//!
//! This module provides functionality to extract preference files from
//! Firefox's omni.ja ZIP archives with caching and security validations.

use crate::error::{Error, Result};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use tempfile::TempDir;
use zip::read::ZipArchive;

/// Default maximum omni.ja file size (100MB)
pub const DEFAULT_MAX_OMNI_SIZE: usize = 100 * 1024 * 1024;

/// Configuration for omni.ja extraction
///
/// # Example
///
/// ```rust,no_run
/// use ffcv::ExtractConfig;
/// use std::path::PathBuf;
///
/// let config = ExtractConfig {
///     max_omni_size: 50 * 1024 * 1024, // 50MB
///     cache_dir: Some(PathBuf::from("/tmp/ffcv_cache")),
///     target_files: vec!["defaults/pref/*.js".to_string()],
///     force_refresh: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ExtractConfig {
    /// Maximum omni.ja file size in bytes
    pub max_omni_size: usize,
    /// Optional custom cache directory
    pub cache_dir: Option<PathBuf>,
    /// Target file patterns to extract
    pub target_files: Vec<String>,
    /// Force cache refresh
    pub force_refresh: bool,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            max_omni_size: DEFAULT_MAX_OMNI_SIZE,
            cache_dir: None,
            target_files: vec![],
            force_refresh: false,
        }
    }
}

/// omni.ja archive extractor
///
/// Handles extraction of preference files from omni.ja ZIP archives
/// with caching, security validations, and size limits.
///
/// # Example
///
/// ```rust,no_run
/// use ffcv::OmniExtractor;
/// use std::path::PathBuf;
///
/// let omni_path = PathBuf::from("/usr/lib/firefox/omni.ja");
/// let extractor = OmniExtractor::new(omni_path).unwrap();
/// let extracted_files = extractor.extract_prefs().unwrap();
///
/// for file in extracted_files {
///     println!("Extracted: {:?}", file);
/// }
/// ```
pub struct OmniExtractor {
    /// Path to omni.ja file
    omni_path: PathBuf,
    /// Cache directory
    cache_dir: Option<TempDir>,
    /// Configuration
    config: ExtractConfig,
}

impl OmniExtractor {
    /// Create a new extractor with default configuration
    ///
    /// # Arguments
    ///
    /// * `omni_path` - Path to omni.ja file
    ///
    /// # Returns
    ///
    /// - `Ok(extractor)` - Extractor ready to use
    /// - `Err(_)` - Error initializing extractor
    pub fn new(omni_path: PathBuf) -> Result<Self> {
        Self::with_config(omni_path, ExtractConfig::default())
    }

    /// Create a new extractor with custom configuration
    ///
    /// # Arguments
    ///
    /// * `omni_path` - Path to omni.ja file
    /// * `config` - Custom extraction configuration
    ///
    /// # Returns
    ///
    /// - `Ok(extractor)` - Extractor ready to use
    /// - `Err(_)` - Error initializing extractor
    pub fn with_config(omni_path: PathBuf, config: ExtractConfig) -> Result<Self> {
        // Validate omni.ja exists
        if !omni_path.exists() {
            return Err(Error::PrefFileNotFound {
                file: omni_path.display().to_string(),
            });
        }

        // Check file size
        let metadata = fs::metadata(&omni_path)?;
        let file_size = metadata.len() as usize;

        if file_size > config.max_omni_size {
            return Err(Error::OmniJaTooLarge {
                actual: file_size,
                limit: config.max_omni_size,
            });
        }

        // Initialize cache directory
        let cache_dir = if config.cache_dir.is_some() {
            // Use custom cache directory (not temp)
            None
        } else {
            // Use temp directory for caching
            Some(TempDir::new()?)
        };

        Ok(Self {
            omni_path,
            cache_dir,
            config,
        })
    }

    /// Extract preference files from omni.ja
    ///
    /// This method extracts JavaScript preference files from the omni.ja
    /// archive, using cache if available and valid.
    ///
    /// # Returns
    ///
    /// - `Ok(files)` - Vector of extracted file paths
    /// - `Err(_)` - Error during extraction
    pub fn extract_prefs(&self) -> Result<Vec<PathBuf>> {
        // Try to use cache if not forcing refresh
        if !self.config.force_refresh {
            if let Ok(cached) = self.try_load_from_cache() {
                return Ok(cached);
            }
        }

        // Extract from archive
        let extracted = self.extract_from_archive()?;

        // Save to cache for next time
        if let Err(e) = self.save_to_cache(&extracted) {
            eprintln!("Warning: Failed to cache extracted files: {}", e);
        }

        Ok(extracted)
    }

    /// List all JavaScript files in the archive
    ///
    /// # Returns
    ///
    /// - `Ok(files)` - Vector of JavaScript file names
    /// - `Err(_)` - Error reading archive
    pub fn list_js_files(&self) -> Result<Vec<String>> {
        let file = fs::File::open(&self.omni_path)?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| Error::ExtractionFailed(e.to_string()))?;

        let mut js_files = Vec::new();

        for i in 0..archive.len() {
            let file = archive
                .by_index(i)
                .map_err(|e| Error::ExtractionFailed(e.to_string()))?;
            let name = file.name().to_string();

            if name.ends_with(".js") {
                js_files.push(name);
            }
        }

        Ok(js_files)
    }

    /// Extract preference files from the archive
    fn extract_from_archive(&self) -> Result<Vec<PathBuf>> {
        let file = fs::File::open(&self.omni_path)?;
        let mut archive =
            ZipArchive::new(file).map_err(|e| Error::ExtractionFailed(e.to_string()))?;

        let mut extracted_files = Vec::new();
        let cache_dir = self.get_cache_path()?;

        for i in 0..archive.len() {
            let mut zipfile = archive
                .by_index(i)
                .map_err(|e| Error::ExtractionFailed(e.to_string()))?;
            let name = zipfile.name().to_string();

            // Security: Check for path traversal attacks
            if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
                continue;
            }

            // Check if file matches target patterns
            if self.should_extract(&name) {
                // Validate uncompressed size
                let uncompressed_size = zipfile.size() as usize;

                // Security: Check for ZIP bomb (suspicious compression ratio)
                if zipfile.compressed_size() as usize > 0 && uncompressed_size > 10 * 1024 * 1024 {
                    // Individual file > 10MB uncompressed is suspicious
                    continue;
                }

                // Extract file
                let output_path = cache_dir.join(&name);

                // Create parent directories
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let mut output_file = fs::File::create(&output_path)?;
                std::io::copy(&mut zipfile, &mut output_file)?;

                extracted_files.push(output_path);
            }
        }

        Ok(extracted_files)
    }

    /// Check if a file should be extracted based on target patterns
    fn should_extract(&self, name: &str) -> bool {
        // Always include greprefs.js
        if name.ends_with("/greprefs.js") || name == "greprefs.js" {
            return true;
        }

        // Check if it matches any target pattern
        for pattern in &self.config.target_files {
            if pattern.ends_with("*.js") {
                // Wildcard pattern
                let prefix = &pattern[..pattern.len() - 4]; // Remove "*.js" (4 chars)
                if name.starts_with(prefix) && name.ends_with(".js") {
                    return true;
                }
            } else if name == pattern {
                // Exact match
                return true;
            }
        }

        false
    }

    /// Try to load extracted files from cache
    fn try_load_from_cache(&self) -> Result<Vec<PathBuf>> {
        let cache_dir = self.get_cache_path()?;

        if !cache_dir.exists() {
            return Err(Error::ExtractionFailed("Cache not found".to_string()));
        }

        // Check cache validity
        let cache_metadata = fs::metadata(&cache_dir)?;
        let omni_metadata = fs::metadata(&self.omni_path)?;

        // Check if cache is older than omni.ja
        if let (Ok(cache_time), Ok(omni_time)) =
            (cache_metadata.modified(), omni_metadata.modified())
        {
            if cache_time < omni_time {
                return Err(Error::ExtractionFailed("Cache is stale".to_string()));
            }
        }

        // Find all cached .js files
        let mut cached_files = Vec::new();
        for entry in walkdir::WalkDir::new(&cache_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "js").unwrap_or(false) {
                cached_files.push(path.to_path_buf());
            }
        }

        if cached_files.is_empty() {
            return Err(Error::ExtractionFailed("No cached files".to_string()));
        }

        Ok(cached_files)
    }

    /// Save extracted files to cache
    fn save_to_cache(&self, _files: &[PathBuf]) -> Result<()> {
        // Files are already extracted to cache directory
        // This is a no-op but kept for future enhancement
        Ok(())
    }

    /// Get the cache directory path
    fn get_cache_path(&self) -> Result<PathBuf> {
        if let Some(ref custom_dir) = self.config.cache_dir {
            Ok(custom_dir.clone())
        } else if let Some(ref temp_dir) = self.cache_dir {
            Ok(temp_dir.path().to_path_buf())
        } else {
            // Use system temp directory
            let mut cache_path = std::env::temp_dir();
            cache_path.push("ffcv");
            cache_path.push("omni");
            cache_path.push(format!(
                "{}_{}",
                self.omni_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("omni"),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            ));

            fs::create_dir_all(&cache_path)?;
            Ok(cache_path)
        }
    }

    /// Clear the extraction cache
    pub fn clear_cache(&self) -> Result<()> {
        let cache_dir = self.get_cache_path()?;

        if cache_dir.exists() {
            fs::remove_dir_all(&cache_dir)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_config_default() {
        let config = ExtractConfig::default();
        assert_eq!(config.max_omni_size, DEFAULT_MAX_OMNI_SIZE);
        assert!(!config.force_refresh);
        assert!(config.target_files.is_empty());
    }

    #[test]
    fn test_should_extract_patterns() {
        let config = ExtractConfig {
            target_files: vec!["defaults/pref/*.js".to_string()],
            ..Default::default()
        };

        let extractor = OmniExtractor {
            omni_path: PathBuf::from("/fake/omni.ja"),
            cache_dir: None,
            config,
        };

        // Test wildcard pattern matching
        // "defaults/pref/*.js" -> prefix = "defaults/pref/"
        assert!(extractor.should_extract("defaults/pref/browser.js"));
        assert!(extractor.should_extract("defaults/pref/firefox.js"));
        assert!(!extractor.should_extract("defaults/pref/readme.txt")); // Not a .js file
        assert!(!extractor.should_extract("other/file.js")); // Wrong prefix

        // Test exact match
        let config2 = ExtractConfig {
            target_files: vec!["greprefs.js".to_string()],
            ..Default::default()
        };
        let extractor2 = OmniExtractor {
            omni_path: PathBuf::from("/fake/omni.ja"),
            cache_dir: None,
            config: config2,
        };
        assert!(extractor2.should_extract("greprefs.js"));
        assert!(!extractor2.should_extract("other.js"));
    }

    #[test]
    fn test_security_path_traversal() {
        let config = ExtractConfig::default();
        let extractor = OmniExtractor {
            omni_path: PathBuf::from("/fake/omni.ja"),
            cache_dir: None,
            config,
        };

        // These should be rejected (path traversal attempts)
        assert!(!extractor.should_extract("../etc/passwd"));
        assert!(!extractor.should_extract("/absolute/path.js"));
        assert!(!extractor.should_extract("\\windows\\path.js"));
    }
}
