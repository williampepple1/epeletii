//! Bot AI module for single-player Ibani Scrabble.
//! Implements dictionary search heuristics and difficulity levels.

use crate::game::{Game, Player};
use crate::dictionary::{Dictionary, strip_tones};
use crate::tiles::Tile;
use crate::protocol::TilePlacement;
use std::collections::{HashMap, HashSet};
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Action returned by the Bot AI.
#[derive(Debug, Clone)]
pub enum BotMoveAction {
    PlaceTiles { placements: Vec<TilePlacement> },
    ExchangeTiles { letters: Vec<String> },
    PassTurn,
}

/// Profiles and chat lines for all 7 bots.
pub struct BotProfile {
    pub name: &'static str,
    pub rating: u32,
    pub role: &'static str,
    pub greetings: &'static [&'static str],
    pub high_score_phrases: &'static [&'static str],
    pub low_score_phrases: &'static [&'static str],
}

pub fn get_bot_profiles() -> Vec<BotProfile> {
    vec![
        BotProfile {
            name: "Curlew (Okolo)",
            rating: 600,
            role: "Symbol of the Founding Island (Okoloama)",
            greetings: &["Welcome to Okoloama! Let's play some words together! 🦛", "Hello! Let's have a friendly game! 🐦"],
            high_score_phrases: &["Wow, that was a good one!", "Look at that word! Nice! 🎉"],
            low_score_phrases: &["Just playing simple words.", "It's all about having fun! 😊"],
        },
        BotProfile {
            name: "Ottam tuwo",
            rating: 750,
            role: "Popular masquerade",
            greetings: &["Always keep your eyes up. 🎭", "Let's raise the roof! 🏠"],
            high_score_phrases: &["Unpredictable, isn't it?", "Unexpected overlays! 🎭"],
            low_score_phrases: &["Keep watching the board.", "Let's see what you do next. 👀"],
        },
        BotProfile {
            name: "King Jaja",
            rating: 1200,
            role: "Legendary Merchant Monarch & House Leader",
            greetings: &["Every turn is a trade deal. I intend to walk away with the profit. 👑", "Let's trade some tiles on the board."],
            high_score_phrases: &["A master tactician always secures the profit! 💰", "Pure economic efficiency! 📈"],
            low_score_phrases: &["Resource management is key.", "A minor trade, but strategic."],
        },
        BotProfile {
            name: "Queen Kambasa",
            rating: 1350,
            role: "First and Only Female Reigning Monarch of Grand Bonny",
            greetings: &["Do not underestimate a ruler who knows how to command the board. 👑", "A decisive monarch is here."],
            high_score_phrases: &["Targeting the multipliers is standard strategy for a queen. 🎯", "Bold and sharp! ⚡"],
            low_score_phrases: &["Securing the territory.", "A calculated maneuver."],
        },
        BotProfile {
            name: "Alagbarigha",
            rating: 1700,
            role: "High Priest & Founder (Okoloamakoromabo)",
            greetings: &["I see pathways on this board that you have yet to divine. 🔮", "Traditional wisdom will guide my tiles."],
            high_score_phrases: &["Foresight leads to great alignments.", "The spiritual path is fruitful! ✨"],
            low_score_phrases: &["Patience. Every tile has its place.", "Defending the closed board."],
        },
        BotProfile {
            name: "King Perekule I",
            rating: 1850,
            role: "Founder of the Perekule Dynasty & Economic Transformer",
            greetings: &["Structure and authority will always triumph over chaotic tiles. 👑", "Let us establish some order on this board."],
            high_score_phrases: &["Structure brings massive scoring! 🏛️", "A grand dynasty of words!"],
            low_score_phrases: &["Consolidating power.", "Organizing the letters."],
        },
        BotProfile {
            name: "Ikuba",
            rating: 2200,
            role: "Historical Deified Guardian Spirit of Grand Bonny",
            greetings: &["You challenge the sacred guardian of the realm? Prove your mastery! 🐉", "Prepare to face the ultimate spirit. ⚡"],
            high_score_phrases: &["Parallel placement mastery! You cannot match a deity. 🐉", "Perfect tile efficiency!"],
            low_score_phrases: &["Analyzing the endgame.", "A tactical transition."],
        },
    ]
}

/// Helper: check if a word can be spelled with the given rack and board letters.
fn can_spell(word: &str, rack: &[String], board_letters: &HashSet<String>) -> bool {
    let mut rack_counts = HashMap::new();
    let mut blanks = 0;
    for letter in rack {
        if letter == " " {
            blanks += 1;
        } else {
            *rack_counts.entry(letter.to_lowercase()).or_insert(0) += 1;
        }
    }

    for c in word.chars() {
        let char_str = c.to_string().to_lowercase();
        let count = rack_counts.entry(char_str.clone()).or_insert(0);
        if *count > 0 {
            *count -= 1;
        } else if board_letters.contains(&char_str) {
            // Assume we can hook/intersect this letter from the board
        } else if blanks > 0 {
            blanks -= 1;
        } else {
            return false;
        }
    }
    true
}

/// A candidate word placement on the board.
#[derive(Debug, Clone)]
struct CandidatePlacement {
    placements: Vec<TilePlacement>,
    score: u32,
    word: String,
}

/// Calculate the bot's turn action.
pub fn make_bot_move(game: &Game, bot_player_idx: usize, bot_level: &str) -> BotMoveAction {
    let player = &game.players[bot_player_idx];
    let rack_letters: Vec<String> = player.rack.iter().map(|t| t.letter.clone()).collect();

    // 1. Gather all letters currently on the board
    let mut board_letters = HashSet::new();
    let mut board_has_tiles = false;
    for row in 0..15 {
        for col in 0..15 {
            if let Some(ref tile) = game.board.squares[row][col].tile {
                board_letters.insert(tile.to_lowercase());
                board_has_tiles = true;
            }
        }
    }

    // 2. Filter the dictionary to find candidate words we can spell
    let max_word_len = match bot_level {
        "Curlew (Okolo)" => 3,
        "Ottam tuwo" => 5,
        "King Jaja" => 6,
        "Queen Kambasa" => 7,
        "Alagbarigha" => 8,
        _ => 10, // King Perekule and Ikuba search up to 10 letters
    };

    let mut candidate_words = Vec::new();
    for word in game.dictionary.words.iter() {
        if word.len() >= 2 && word.len() <= max_word_len {
            if can_spell(word, &rack_letters, &board_letters) || !board_has_tiles {
                candidate_words.push(word.clone());
            }
        }
    }

    // If no candidate words at all, pass or exchange
    if candidate_words.is_empty() {
        return choose_fallback_move(&rack_letters, game.tile_bag.len());
    }

    // 3. Scan the board for valid placements of these candidate words
    let mut valid_moves = Vec::new();

    if !board_has_tiles {
        // First move must cross (7,7)
        let row = 7;
        for word in &candidate_words {
            let len = word.len();
            for c_start in (7 - len + 1)..=7 {
                if c_start + len <= 15 {
                    // Try Across
                    let mut placements = Vec::new();
                    for (i, c) in word.chars().enumerate() {
                        placements.push(TilePlacement {
                            row,
                            col: c_start + i,
                            letter: c.to_string(),
                        });
                    }
                    if let Some(mv) = test_placement(game, placements, word) {
                        valid_moves.push(mv);
                    }

                    // Try Down
                    let mut placements_down = Vec::new();
                    for (i, c) in word.chars().enumerate() {
                        placements_down.push(TilePlacement {
                            row: c_start + i,
                            col: 7,
                            letter: c.to_string(),
                        });
                    }
                    if let Some(mv) = test_placement(game, placements_down, word) {
                        valid_moves.push(mv);
                    }
                }
            }
        }
    } else {
        // Try placing adjacent to / overlapping existing tiles
        // Find anchor points: cells that contain tiles
        let mut anchors = Vec::new();
        for r in 0..15 {
            for c in 0..15 {
                if game.board.squares[r][c].tile.is_some() {
                    anchors.push((r, c));
                }
            }
        }

        // Shuffle anchors to avoid looking at the same board area first every time (improves randomness)
        let mut rng = thread_rng();
        anchors.shuffle(&mut rng);

        // Limit the number of anchors we check to avoid performance spikes
        let anchors_to_check = match bot_level {
            "Curlew (Okolo)" | "Ottam tuwo" => anchors.into_iter().take(5).collect::<Vec<_>>(),
            "King Jaja" | "Queen Kambasa" => anchors.into_iter().take(15).collect::<Vec<_>>(),
            _ => anchors, // Advanced and GM search all anchors
        };

        for &(r, c) in &anchors_to_check {
            let existing_letter = game.board.squares[r][c].tile.as_ref().unwrap().to_lowercase();
            
            // Find all candidates that contain this letter
            for word in &candidate_words {
                let len = word.len();
                // Find all indices of the existing letter in the word
                let indices: Vec<usize> = word.char_indices()
                    .filter(|(_, ch)| ch.to_string().to_lowercase() == existing_letter)
                    .map(|(idx, _)| idx)
                    .collect();

                for &idx in &indices {
                    // Try Across: word starts at col = c - idx
                    let c_start = c as i32 - idx as i32;
                    if c_start >= 0 && c_start + len as i32 <= 15 {
                        let c_start_usize = c_start as usize;
                        let mut placements = Vec::new();
                        let mut possible = true;
                        
                        for (i, ch) in word.chars().enumerate() {
                            let curr_col = c_start_usize + i;
                            let sq = &game.board.squares[r][curr_col];
                            if let Some(ref tile) = sq.tile {
                                if tile.to_lowercase() != ch.to_string().to_lowercase() {
                                    possible = false;
                                    break;
                                }
                            } else {
                                placements.push(TilePlacement {
                                    row: r,
                                    col: curr_col,
                                    letter: ch.to_string(),
                                });
                            }
                        }

                        if possible && !placements.is_empty() {
                            if let Some(mv) = test_placement(game, placements, word) {
                                valid_moves.push(mv);
                            }
                        }
                    }

                    // Try Down: word starts at row = r - idx
                    let r_start = r as i32 - idx as i32;
                    if r_start >= 0 && r_start + len as i32 <= 15 {
                        let r_start_usize = r_start as usize;
                        let mut placements = Vec::new();
                        let mut possible = true;

                        for (i, ch) in word.chars().enumerate() {
                            let curr_row = r_start_usize + i;
                            let sq = &game.board.squares[curr_row][c];
                            if let Some(ref tile) = sq.tile {
                                if tile.to_lowercase() != ch.to_string().to_lowercase() {
                                    possible = false;
                                    break;
                                }
                            } else {
                                placements.push(TilePlacement {
                                    row: curr_row,
                                    col: c,
                                    letter: ch.to_string(),
                                });
                            }
                        }

                        if possible && !placements.is_empty() {
                            if let Some(mv) = test_placement(game, placements, word) {
                                valid_moves.push(mv);
                            }
                        }
                    }
                }
            }
        }
    }

    if valid_moves.is_empty() {
        return choose_fallback_move(&rack_letters, game.tile_bag.len());
    }

    // 4. Select the best move according to the bot level
    let mut rng = thread_rng();
    match bot_level {
        "Curlew (Okolo)" => {
            // Beginner: Plays short, low-scoring words. Randomly choose from lowest scoring half of moves.
            valid_moves.sort_by_key(|m| m.score);
            let limit = (valid_moves.len() / 2).max(1);
            let selected = valid_moves[..limit].choose(&mut rng).unwrap();
            BotMoveAction::PlaceTiles { placements: selected.placements.clone() }
        }
        "Ottam tuwo" => {
            // Unpredictable: plays unexpected moves. Pick a random valid placement.
            let selected = valid_moves.choose(&mut rng).unwrap();
            BotMoveAction::PlaceTiles { placements: selected.placements.clone() }
        }
        "King Jaja" => {
            // Intermediate (tactical/multiplier focus). Filter for moves using high-value letters, sort by score.
            // Pick from the top 3 moves.
            valid_moves.sort_by_key(|m| std::cmp::Reverse(m.score));
            let limit = valid_moves.len().min(3);
            let selected = valid_moves[..limit].choose(&mut rng).unwrap();
            BotMoveAction::PlaceTiles { placements: selected.placements.clone() }
        }
        "Queen Kambasa" => {
            // Intermediate aggressive. Prioritizes premium tile targets. Sort by score and pick the best.
            valid_moves.sort_by_key(|m| std::cmp::Reverse(m.score));
            BotMoveAction::PlaceTiles { placements: valid_moves[0].placements.clone() }
        }
        "Alagbarigha" => {
            // Advanced defensive. Choose a good scoring move (top 5) that doesn't place open multipliers.
            // For simplicity, pick from top 3.
            valid_moves.sort_by_key(|m| std::cmp::Reverse(m.score));
            let limit = valid_moves.len().min(3);
            let selected = valid_moves[..limit].choose(&mut rng).unwrap();
            BotMoveAction::PlaceTiles { placements: selected.placements.clone() }
        }
        "King Perekule I" => {
            // Advanced bingo hunter. Prioritizes using maximum tiles.
            valid_moves.sort_by(|a, b| {
                let len_cmp = b.placements.len().cmp(&a.placements.len());
                if len_cmp == std::cmp::Ordering::Equal {
                    b.score.cmp(&a.score)
                } else {
                    len_cmp
                }
            });
            BotMoveAction::PlaceTiles { placements: valid_moves[0].placements.clone() }
        }
        _ => {
            // Grandmaster (Ikuba): Play the absolute highest-scoring valid placement.
            valid_moves.sort_by_key(|m| std::cmp::Reverse(m.score));
            BotMoveAction::PlaceTiles { placements: valid_moves[0].placements.clone() }
        }
    }
}

/// Helper: test if a placement is valid and get its score.
fn test_placement(game: &Game, placements: Vec<TilePlacement>, word: &str) -> Option<CandidatePlacement> {
    let tuples: Vec<(usize, usize, String)> = placements.iter().map(|p| (p.row, p.col, p.letter.clone())).collect();
    
    // We clone the game state to test the move safely
    let mut temp_game = game.clone();
    
    // Apply placement
    match temp_game.place_tiles(&tuples) {
        Ok((_, score)) => Some(CandidatePlacement {
            placements,
            score,
            word: word.to_string(),
        }),
        Err(_) => None,
    }
}

/// Helper: choose whether to exchange or pass if no words are playable.
fn choose_fallback_move(rack: &[String], bag_len: usize) -> BotMoveAction {
    if bag_len > 0 && !rack.is_empty() {
        // Exchange 3 tiles (or all if rack is smaller)
        let count = rack.len().min(3);
        let mut rng = thread_rng();
        let mut shuffled = rack.to_vec();
        shuffled.shuffle(&mut rng);
        let letters = shuffled[..count].to_vec();
        BotMoveAction::ExchangeTiles { letters }
    } else {
        BotMoveAction::PassTurn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::Dictionary;
    use crate::game::Game;

    #[test]
    fn test_bot_can_find_first_move() {
        let dict = Dictionary::load();
        let mut game = Game::new(dict);
        game.add_player("player1".to_string(), "Human".to_string());
        game.add_player("bot1".to_string(), "Bot".to_string());
        game.phase = crate::game::GamePhase::Playing;
        
        // Give the bot some tiles
        game.players[0].rack = vec![
            Tile { letter: "b".to_string(), value: 2 },
            Tile { letter: "o".to_string(), value: 1 },
            Tile { letter: "a".to_string(), value: 1 },
            Tile { letter: "d".to_string(), value: 3 },
            Tile { letter: "e".to_string(), value: 1 },
            Tile { letter: "i".to_string(), value: 1 },
        ];

        let action = make_bot_move(&game, 0, "Curlew (Okolo)");
        match action {
            BotMoveAction::PlaceTiles { placements } => {
                assert!(!placements.is_empty(), "Bot should place some tiles");
                // The first placement must cover (7,7)
                let covers_center = placements.iter().any(|p| p.row == 7 && p.col == 7);
                assert!(covers_center, "First placement must cover center square (7,7)");
            }
            _ => panic!("Bot should have played a word on an empty board"),
        }
    }
}
