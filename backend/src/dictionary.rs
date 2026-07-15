//! Ibani dictionary word validator.
//! Loads words from the Ibani-dictionary SQLite database into a HashSet
//! for O(1) lookups during gameplay.

use rusqlite::Connection;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

/// Thread-safe word dictionary for validation.
#[derive(Debug, Clone)]
pub struct Dictionary {
    words: Arc<HashSet<String>>,
}

impl Dictionary {
    /// Load the dictionary from an Ibani-dictionary SQLite file.
    ///
    /// Environment variable `IBANI_DICT_PATH` can override the location.
    /// Default: looks for dictionary.db in the current or backend directory.
    pub fn load() -> Self {
        let paths = vec![
            std::env::var("IBANI_DICT_PATH").ok(),
            Some("./dictionary.db".to_string()),
            Some("dictionary.db".to_string()),
        ];

        for path in paths.into_iter().flatten() {
            if Path::new(&path).exists() {
                match Self::load_from_sqlite(&path) {
                    Ok(words) => {
                        log::info!("Loaded {} words from {}", words.len(), path);
                        return Self { words: Arc::new(words) };
                    }
                    Err(e) => {
                        log::warn!("Failed to load {}: {}", path, e);
                    }
                }
            }
        }

        // Fallback: try loading from flat file
        for path in &["words.txt", "../words.txt"] {
            if Path::new(path).exists() {
                match Self::load_from_file(path) {
                    Ok(words) => {
                        log::info!("Loaded {} words from {}", words.len(), path);
                        return Self { words: Arc::new(words) };
                    }
                    Err(e) => log::warn!("Failed to load {}: {}", path, e),
                }
            }
        }

        log::warn!("No dictionary found! Word validation disabled.");
        Self { words: Arc::new(HashSet::new()) }
    }

    /// Load Ibani words from the SQLite dictionary database.
    fn load_from_sqlite(path: &str) -> rusqlite::Result<HashSet<String>> {
        let conn = Connection::open(path)?;
        let mut stmt = conn.prepare("SELECT Ibani_word FROM Ibani_dictionary")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut words = HashSet::new();
        for row in rows {
            if let Ok(word) = row {
                // Normalize: lowercase, strip leading/trailing whitespace
                let cleaned = word.trim().to_lowercase();
                if !cleaned.is_empty() {
                    words.insert(cleaned);
                }
            }
        }
        Ok(words)
    }

    /// Load words from a plain text file (one word per line).
    fn load_from_file(path: &str) -> std::io::Result<HashSet<String>> {
        let content = std::fs::read_to_string(path)?;
        let words = content
            .lines()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| !l.is_empty())
            .collect();
        Ok(words)
    }

    /// Check if a word is valid in the Ibani dictionary.
    pub fn is_valid_word(&self, word: &str) -> bool {
        let cleaned = word.trim().to_lowercase();
        if cleaned.is_empty() || cleaned.len() < 2 {
            return false;
        }
        self.words.contains(&cleaned)
    }

    /// Get total number of loaded words.
    pub fn size(&self) -> usize {
        self.words.len()
    }
}
