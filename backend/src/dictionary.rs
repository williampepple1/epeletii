//! Ibani dictionary word validator.
//! Loads words from the Ibani-dictionary SQLite database.

use rusqlite::Connection;
use std::collections::{HashSet, HashMap};
use std::path::Path;
use std::sync::Arc;
use crate::protocol::WordDetail;

/// Thread-safe word dictionary for validation.
#[derive(Debug, Clone)]
pub struct Dictionary {
    pub words: Arc<HashSet<String>>,
    word_map: Arc<HashMap<String, Vec<WordDetail>>>,
}

/// Strip tone marks (accents, macrons, graves, carons) from an Ibani word
/// while preserving underdots (ị, ẹ, ọ, ụ, ḅ).
pub fn strip_tones(s: &str) -> String {
    let lower = s.to_lowercase();
    lower.chars().map(|c| match c {
        'á' | 'ā' | 'à' | 'â' | 'ǎ' => 'a',
        'é' | 'ē' | 'è' | 'ê' | 'ě' | 'ë' | 'ẹ' => 'e', // ẹ -> e
        'í' | 'ī' | 'ì' | 'î' | 'ǐ' | 'ị' => 'i', // ị -> i
        'ó' | 'ō' | 'ò' | 'ô' | 'ǒ' | 'ọ' => 'o', // ọ -> o
        'ú' | 'ū' | 'ù' | 'û' | 'ǔ' | 'ụ' => 'u', // ụ -> u
        'ń' => 'n',
        'ḅ' => 'b', // ḅ -> b
        '\u{0301}' | '\u{0304}' | '\u{0300}' | '\u{030c}' | '\u{0302}' | '\u{0323}' => '\0', // combining accents and underdot
        other => other
    }).filter(|&c| c != '\0').collect()
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
                    Ok((words, word_map)) => {
                        log::info!("Loaded {} words ({} plain keys) from {}", words.len(), word_map.len(), path);
                        return Self {
                            words: Arc::new(words),
                            word_map: Arc::new(word_map),
                        };
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
                    Ok((words, word_map)) => {
                        log::info!("Loaded {} words from {}", words.len(), path);
                        return Self {
                            words: Arc::new(words),
                            word_map: Arc::new(word_map),
                        };
                    }
                    Err(e) => log::warn!("Failed to load {}: {}", path, e),
                }
            }
        }

        log::warn!("No dictionary found! Word validation disabled.");
        Self {
            words: Arc::new(HashSet::new()),
            word_map: Arc::new(HashMap::new()),
        }
    }

    /// Load Ibani words from the SQLite dictionary database.
    fn load_from_sqlite(path: &str) -> rusqlite::Result<(HashSet<String>, HashMap<String, Vec<WordDetail>>)> {
        let conn = Connection::open(path)?;
        let mut stmt = conn.prepare("SELECT Ibani_word, Pos, Meaning FROM Ibani_dictionary")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut words = HashSet::new();
        let mut word_map: HashMap<String, Vec<WordDetail>> = HashMap::new();
        for row in rows {
            if let Ok((word, pos, meaning)) = row {
                let word_trimmed = word.trim().to_string();
                let cleaned_lower = word_trimmed.to_lowercase();
                if !cleaned_lower.is_empty() {
                    words.insert(cleaned_lower.clone());
                    let plain = strip_tones(&cleaned_lower);
                    let detail = WordDetail {
                        word: word_trimmed,
                        pos: pos.trim().to_string(),
                        meaning: meaning.trim().to_string(),
                    };
                    word_map.entry(plain).or_insert_with(Vec::new).push(detail);
                }
            }
        }
        Ok((words, word_map))
    }

    /// Load words from a plain text file (one word per line).
    fn load_from_file(path: &str) -> std::io::Result<(HashSet<String>, HashMap<String, Vec<WordDetail>>)> {
        let content = std::fs::read_to_string(path)?;
        let mut words = HashSet::new();
        let mut word_map = HashMap::new();
        for line in content.lines() {
            let word = line.trim().to_lowercase();
            if !word.is_empty() {
                words.insert(word.clone());
                let plain = strip_tones(&word);
                let detail = WordDetail {
                    word: word.clone(),
                    pos: "".to_string(),
                    meaning: "".to_string(),
                };
                word_map.entry(plain).or_insert_with(Vec::new).push(detail);
            }
        }
        Ok((words, word_map))
    }

    /// Check if a word is valid in the Ibani dictionary.
    pub fn is_valid_word(&self, word: &str) -> bool {
        let cleaned = strip_tones(word).trim().to_lowercase();
        if cleaned.is_empty() || cleaned.len() < 2 {
            return false;
        }
        self.word_map.contains_key(&cleaned)
    }

    /// Lookup definitions for a word.
    pub fn lookup(&self, word: &str) -> Option<Vec<WordDetail>> {
        let cleaned = strip_tones(word).trim().to_lowercase();
        self.word_map.get(&cleaned).cloned()
    }

    /// Get total number of loaded words.
    pub fn size(&self) -> usize {
        self.words.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_tones() {
        assert_eq!(strip_tones("ábá"), "aba");
        assert_eq!(strip_tones("áa"), "aa");
        assert_eq!(strip_tones("abaji"), "abaji");
        assert_eq!(strip_tones("ẹ́mẹ́"), "eme");
        assert_eq!(strip_tones("bḅ"), "bb");
        assert_eq!(strip_tones("ịḅani"), "ibani");
    }

    #[test]
    fn test_real_dictionary_lookup() {
        let dict = Dictionary::load();
        assert!(dict.size() > 0, "Dictionary should load successfully");
        assert!(dict.is_valid_word("bo"), "bo should be a valid word");
        assert!(dict.is_valid_word("ibani"), "ibani should be a valid word");
        
        let defs = dict.lookup("bo").unwrap();
        assert!(!defs.is_empty(), "bo should have definitions");
        let original_words: Vec<&str> = defs.iter().map(|d| d.word.as_str()).collect();
        assert!(original_words.contains(&"bọ́"), "bọ́ should be in original words list");
    }
}
