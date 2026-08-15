//! Ibani Scrabble tile definitions.
//! Based on frequency analysis of the Ibani training corpus and dictionary.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single letter tile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Tile {
    pub letter: String,      // the Ibani letter (e.g. "a", "ị", "ḅ")
    pub value: u8,           // point value
}

/// The full tile bag for the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileBag {
    pub tiles: Vec<Tile>,
    pub remaining: usize,
}

/// Build the standard Ibani Scrabble tile distribution.
///
/// Letters and their frequencies derived from corpus analysis.
/// Tone marks are NOT separate tiles — they're diacritics on base vowels.
/// A blank/wild tile is included (2 tiles).
pub fn standard_tile_bag() -> Vec<Tile> {
    // Distribution: letter -> (count, points)
    // Frequencies normalized from ~900K character corpus.
    // Underdotted letters are merged with plain base letters for ease of play.
    let dist: Vec<(&str, u32, u8)> = vec![
        // Vowels
        ("a", 12, 1),
        ("i", 20, 1), // merged i (10) + ị (10)
        ("e", 16, 1), // merged e (8) + ẹ (8)
        ("o", 16, 1), // merged o (8) + ọ (8)
        ("u", 12, 1), // merged u (6) + ụ (6)
        // Common consonants
        ("n", 10, 1),
        ("m", 9, 1),
        ("r", 8, 1),
        ("g", 6, 2),
        ("s", 6, 2),
        ("p", 6, 2),
        ("b", 10, 2), // merged b (5) + ḅ (5)
        ("h", 5, 2),
        ("k", 5, 2),
        ("d", 4, 3),
        ("t", 4, 3),
        ("w", 4, 3),
        ("y", 4, 3),
        ("l", 3, 4),
        ("f", 3, 4),
        // Less common consonants
        ("j", 2, 6),
        ("z", 2, 6),
        ("v", 1, 8),
        // Blank (wild)
        (" ", 2, 0),
    ];

    let mut tiles = Vec::new();
    for &(letter, count, value) in &dist {
        for _ in 0..count {
            tiles.push(Tile {
                letter: letter.to_string(),
                value,
            });
        }
    }
    tiles
}

/// Shorthand map: letter -> point value
pub fn letter_points() -> HashMap<String, u8> {
    let bag = standard_tile_bag();
    let mut map = HashMap::new();
    for tile in bag {
        map.entry(tile.letter.clone()).or_insert(tile.value);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_bag_size() {
        let bag = standard_tile_bag();
        assert_eq!(bag.len(), 170, "Standard bag should have 170 tiles");
    }

    #[test]
    fn test_letter_points_consistency() {
        let pts = letter_points();
        assert_eq!(*pts.get("a").unwrap(), 1);
        assert_eq!(*pts.get("i").unwrap(), 1);
        assert_eq!(*pts.get("b").unwrap(), 2);
        assert_eq!(*pts.get(" ").unwrap(), 0);
    }
}
