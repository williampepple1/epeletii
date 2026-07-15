//! Game logic — state machine for a single Scrabble game.
//! Manages turns, scoring, tile drawing, and word formation.

use crate::board::{Board, Direction, Premium};
use crate::dictionary::Dictionary;
use crate::tiles::{standard_tile_bag, Tile, letter_points};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Score a word based on tiles and premium squares.
struct WordScore {
    score: u32,
    word_mult: u32,
}

/// Player in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub score: u32,
    pub rack: Vec<Tile>,
    pub ready: bool,
}

/// Current game state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GamePhase {
    Lobby,
    Playing,
    Finished,
}

/// The full game state.
#[derive(Debug, Clone)]
pub struct Game {
    pub board: Board,
    pub players: Vec<Player>,
    pub tile_bag: Vec<Tile>,
    pub current_turn: usize,
    pub phase: GamePhase,
    pub consecutive_passes: u32,
    pub dictionary: Dictionary,
    pub winner: Option<String>,
    pub tiles_placed_this_turn: Vec<(usize, usize)>,
}

impl Game {
    /// Create a new game with the given dictionary.
    pub fn new(dictionary: Dictionary) -> Self {
        Self {
            board: Board::new(),
            players: Vec::new(),
            tile_bag: standard_tile_bag(),
            current_turn: 0,
            phase: GamePhase::Lobby,
            consecutive_passes: 0,
            dictionary,
            winner: None,
            tiles_placed_this_turn: Vec::new(),
        }
    }

    /// Add a player to the game.
    pub fn add_player(&mut self, id: String, name: String) {
        self.players.push(Player {
            id,
            name,
            score: 0,
            rack: Vec::new(),
            ready: false,
        });
    }

    /// Draw N tiles from the bag.
    pub fn draw_tiles(&mut self, count: usize) -> Vec<Tile> {
        let available = self.tile_bag.len().min(count);
        if available == 0 {
            return Vec::new();
        }
        self.tile_bag.split_off(self.tile_bag.len() - available)
    }

    /// Shuffle and deal initial tiles to all players (7 each).
    pub fn deal_tiles(&mut self) {
        let mut rng = thread_rng();
        self.tile_bag.shuffle(&mut rng);
        let player_count = self.players.len();
        for i in 0..player_count {
            let drawn = self.draw_tiles(7);
            self.players[i].rack = drawn;
        }
    }

    /// Check if all players are ready.
    pub fn all_ready(&self) -> bool {
        self.players.len() >= 2 && self.players.iter().all(|p| p.ready)
    }

    /// Start the game with a specific first player.
    pub fn start(&mut self, first_player: usize) -> Result<(), String> {
        if self.players.len() < 2 {
            return Err("Need at least 2 players".to_string());
        }
        self.deal_tiles();
        self.phase = GamePhase::Playing;
        self.current_turn = first_player;
        self.consecutive_passes = 0;
        Ok(())
    }

    /// Each player draws one tile to determine who goes first.
    /// The player with the letter closest to 'A' wins (blank wins all).
    /// Returns (first_player_index, Vec<(player_id, letter)>)
    pub fn draw_for_first(&mut self) -> (usize, Vec<(String, String)>) {
        use std::cmp::Ordering;
        let mut rng = thread_rng();
        self.tile_bag.shuffle(&mut rng);

        let mut draws: Vec<(String, String)> = Vec::new();

        for player in &self.players {
            if let Some(tile) = self.tile_bag.pop() {
                draws.push((player.id.clone(), tile.letter.clone()));
            }
        }

        // Sort: blank tiles (space) go first, then alphabetically
        let winner_idx = draws
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let a_is_blank = a.1 == " ";
                let b_is_blank = b.1 == " ";
                match (a_is_blank, b_is_blank) {
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    _ => a.1.cmp(&b.1),
                }
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        // Put the tiles back
        for (_, letter) in &draws {
            self.tile_bag.push(crate::tiles::Tile {
                letter: letter.clone(),
                value: 0,
            });
        }

        (winner_idx, draws)
    }

    /// Index of current player.
    pub fn current_player_index(&self) -> usize {
        self.current_turn % self.players.len()
    }

    /// Advance to the next player's turn.
    pub fn next_turn(&mut self) {
        self.current_turn = (self.current_turn + 1) % self.players.len();
        self.tiles_placed_this_turn.clear();
    }

    /// Process a move: place tiles and calculate score.
    /// Returns the words formed and total score.
    pub fn place_tiles(
        &mut self,
        placements: &[(usize, usize, String)],
    ) -> Result<(Vec<String>, u32), String> {
        if !matches!(self.phase, GamePhase::Playing) {
            return Err("Game is not in playing phase".to_string());
        }
        if placements.is_empty() {
            return Err("No tiles placed".to_string());
        }

        let rows: HashSet<usize> = placements.iter().map(|(r, _, _)| *r).collect();
        let cols: HashSet<usize> = placements.iter().map(|(_, c, _)| *c).collect();

        let direction = if rows.len() == 1 {
            Direction::Across
        } else if cols.len() == 1 {
            Direction::Down
        } else {
            return Err("Tiles must be in a single row or column".to_string());
        };

        self.validate_placement(placements, direction)?;

        // Remove tiles from current player's rack (need player index first, then drop borrow)
        let player_idx = self.current_player_index();

        // Collect letters to remove separately to avoid double borrow
        let letters_to_place: Vec<String> = placements.iter().map(|(_, _, l)| l.to_lowercase()).collect();
        self.remove_tiles_from_rack(player_idx, &letters_to_place)?;

        // Place tiles on board
        let player_idx_u8 = player_idx as u8;
        for (row, col, letter) in placements {
            self.board
                .place_tile(*row, *col, letter, player_idx_u8)
                .map_err(|e| format!("Failed to place tile: {}", e))?;
        }

        self.tiles_placed_this_turn = placements.iter().map(|(r, c, _)| (*r, *c)).collect();

        // Calculate score
        let (words, score) = self.calculate_score(placements, direction);

        // Validate all formed words against the dictionary
        let words_valid = words.iter().all(|w| self.dictionary.is_valid_word(w));
        if !words_valid {
            // Undo: remove placed tiles from board
            for (row, col, _) in placements {
                self.board.squares[*row][*col].tile = None;
                self.board.squares[*row][*col].owner = None;
            }
            // Return tiles to player's rack
            for (_, _, letter) in placements {
                self.players[player_idx].rack.push(crate::tiles::Tile {
                    letter: letter.clone(),
                    value: 0,
                });
            }
            // Update letter points
            let pts = crate::tiles::letter_points();
            for t in &mut self.players[player_idx].rack {
                t.value = *pts.get(&t.letter).unwrap_or(&1);
            }
            let invalid_words: Vec<&str> = words.iter().filter(|w| !self.dictionary.is_valid_word(w)).map(|s| s.as_str()).collect();
            return Err(format!("Invalid word(s): {}", invalid_words.join(", ")));
        }

        // Add score to player
        self.players[player_idx].score += score;

        // Bonus for using all 7 tiles
        if self.players[player_idx].rack.is_empty() {
            self.players[player_idx].score += 50;
        }

        // Draw replacement tiles
        let drawn = self.draw_tiles(placements.len());
        self.players[player_idx].rack.extend(drawn);

        // Reset consecutive passes
        self.consecutive_passes = 0;

        Ok((words, score))
    }

    /// Remove specific tiles from a player's rack.
    fn remove_tiles_from_rack(&mut self, player_idx: usize, letters: &[String]) -> Result<(), String> {
        for letter in letters {
            let pos = self.players[player_idx].rack.iter().position(|t| t.letter == *letter);
            match pos {
                Some(i) => { self.players[player_idx].rack.remove(i); }
                None => {
                    // Try blank tile
                    let blank_pos = self.players[player_idx].rack.iter().position(|t| t.letter == " ");
                    match blank_pos {
                        Some(i) => { self.players[player_idx].rack.remove(i); }
                        None => return Err(format!("Player doesn't have tile '{}'", letter)),
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate that the placement is legal (contiguous, connected, through center if first move).
    fn validate_placement(
        &self,
        placements: &[(usize, usize, String)],
        direction: Direction,
    ) -> Result<(), String> {
        if placements.is_empty() {
            return Err("No tiles placed".to_string());
        }

        let mut sorted: Vec<_> = placements.iter().collect();
        match direction {
            Direction::Across => sorted.sort_by_key(|(_, c, _)| *c),
            Direction::Down => sorted.sort_by_key(|(r, _, _)| *r),
        }

        let (first_row, first_col, _) = sorted[0];
        match direction {
            Direction::Across => {
                let row = *first_row;
                let start_col = *first_col;
                for (i, (r, c, _)) in sorted.iter().enumerate() {
                    if *r != row {
                        return Err("Tiles not in same row for across play".to_string());
                    }
                    if *c > start_col + i {
                        let expected = start_col + i;
                        if let Some(row_data) = self.board.squares.get(row) {
                            if let Some(sq) = row_data.get(expected) {
                                if sq.tile.is_none() {
                                    return Err(format!("Tiles must be contiguous (gap at {row},{expected})"));
                                }
                            }
                        }
                    }
                }
            }
            Direction::Down => {
                let col = *first_col;
                let start_row = *first_row;
                for (i, (r, c, _)) in sorted.iter().enumerate() {
                    if *c != col {
                        return Err("Tiles not in same column for down play".to_string());
                    }
                    if *r > start_row + i {
                        let expected = start_row + i;
                        if let Some(row_data) = self.board.squares.get(expected) {
                            if let Some(sq) = row_data.get(col) {
                                if sq.tile.is_none() {
                                    return Err(format!("Tiles must be contiguous (gap at {expected},{col})"));
                                }
                            }
                        }
                    }
                }
            }
        }

        // First move must go through center (7,7)
        let has_any_tiles = self.board.squares.iter().any(|row| {
            row.iter().any(|sq| sq.tile.is_some())
        });

        if !has_any_tiles {
            let covers_center = placements.iter().any(|(r, c, _)| *r == 7 && *c == 7);
            if !covers_center {
                return Err("First move must cover the center square (7,7)".to_string());
            }
        } else {
            // Must connect to existing tiles
            let connected = placements.iter().any(|(r, c, _)| {
                let (r, c) = (*r as i32, *c as i32);
                [(r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)]
                    .iter()
                    .any(|(nr, nc)| {
                        if *nr >= 0 && *nr < self.board.size as i32 && *nc >= 0 && *nc < self.board.size as i32 {
                            self.board.tile_at(*nr as usize, *nc as usize).is_some()
                        } else {
                            false
                        }
                    })
            });
            if !connected {
                return Err("Tiles must connect to existing tiles on the board".to_string());
            }
        }

        Ok(())
    }

    /// Calculate score for a move.
    /// Returns (words_formed, total_score).
    fn calculate_score(
        &self,
        placements: &[(usize, usize, String)],
        direction: Direction,
    ) -> (Vec<String>, u32) {
        let points = letter_points();
        let mut total_score = 0u32;
        let mut word_mult = 1u32;
        let mut all_words = Vec::new();

        // Main word
        let placed_set: HashSet<(usize, usize)> =
            placements.iter().map(|(r, c, _)| (*r, *c)).collect();

        let main_word = self.read_word_in_direction(
            placements[0].0,
            placements[0].1,
            direction,
            Some(&placed_set),
        );
        let main_score = self.score_word(&main_word, &placed_set, direction, &points);
        total_score += main_score.score;
        word_mult = word_mult.max(main_score.word_mult);
        all_words.push(main_word);

        // Cross words
        let cross_dir = match direction {
            Direction::Across => Direction::Down,
            Direction::Down => Direction::Across,
        };

        for (row, col, _) in placements {
            let has_prev = match cross_dir {
                Direction::Across => {
                    *col > 0
                        && self.board.tile_at(*row, *col - 1).is_some()
                }
                Direction::Down => {
                    *row > 0
                        && self.board.tile_at(*row - 1, *col).is_some()
                }
            };
            let has_next = match cross_dir {
                Direction::Across => {
                    *col + 1 < self.board.size
                        && self.board.tile_at(*row, *col + 1).is_some()
                }
                Direction::Down => {
                    *row + 1 < self.board.size
                        && self.board.tile_at(*row + 1, *col).is_some()
                }
            };

            if has_prev || has_next {
                let cross_word = self.read_word_in_direction(*row, *col, cross_dir, None);
                if cross_word.len() > 1 {
                    let single_placed: HashSet<(usize, usize)> =
                        [(Self::clone_pos(*row, *col))].into_iter().collect();
                    let cross_score = self.score_word(&cross_word, &single_placed, cross_dir, &points);
                    total_score += cross_score.score;
                    all_words.push(cross_word);
                }
            }
        }

        // Apply word multiplier from premium squares on new placements
        // Word multipliers stack multiplicatively
        for (row, col, _) in placements {
            let premium = self.board.squares[*row][*col].premium;
            match premium {
                Premium::DW => word_mult = word_mult.saturating_mul(2),
                Premium::TW => word_mult = word_mult.saturating_mul(3),
                _ => {}
            }
        }

        total_score = total_score.saturating_mul(word_mult);

        (all_words, total_score)
    }

    fn clone_pos(r: usize, c: usize) -> (usize, usize) {
        (r, c)
    }

    /// Read a word starting at (row, col) in the given direction.
    fn read_word_in_direction(
        &self,
        row: usize,
        col: usize,
        direction: Direction,
        _placed_set: Option<&HashSet<(usize, usize)>>,
    ) -> String {
        let mut chars = Vec::new();
        let mut r = row as i32;
        let mut c = col as i32;

        // Go to the start of the word
        match direction {
            Direction::Across => {
                while self.board.in_bounds(r, c - 1)
                    && self.board.tile_at(r as usize, (c - 1) as usize).is_some()
                {
                    c -= 1;
                }
            }
            Direction::Down => {
                while self.board.in_bounds(r - 1, c)
                    && self.board.tile_at((r - 1) as usize, c as usize).is_some()
                {
                    r -= 1;
                }
            }
        }

        // Read the word
        loop {
            if !self.board.in_bounds(r, c) {
                break;
            }
            match self.board.tile_at(r as usize, c as usize) {
                Some(tile) => chars.push(tile.to_string()),
                None => break,
            }
            match direction {
                Direction::Across => c += 1,
                Direction::Down => r += 1,
            }
        }

        chars.join("")
    }

    /// Score a word based on tiles and premium squares.
    fn score_word(
        &self,
        word: &str,
        new_placed: &HashSet<(usize, usize)>,
        direction: Direction,
        points: &HashMap<String, u8>,
    ) -> WordScore {
        let mut raw_score = 0u32;
        let mut word_mult = 1u32;

        // Find the starting position of this word on the board
        let start = self.find_word_start(word.chars().next().unwrap_or(' '), direction);
        let (base_row, base_col) = start;

        for (i, ch) in word.chars().enumerate() {
            let letter = ch.to_string();
            let letter_score = *points.get(&letter).unwrap_or(&1) as u32;

            let (r, c) = match direction {
                Direction::Across => (base_row, base_col + i),
                Direction::Down => (base_row + i, base_col),
            };

            let mut effective_score = letter_score;

            if new_placed.contains(&(r, c)) && r < self.board.size && c < self.board.size {
                match self.board.squares[r][c].premium {
                    Premium::DL => effective_score = letter_score.saturating_mul(2),
                    Premium::TL => effective_score = letter_score.saturating_mul(3),
                    Premium::DW => word_mult = word_mult.saturating_mul(2),
                    Premium::TW => word_mult = word_mult.saturating_mul(3),
                    Premium::Normal => {}
                }
            }

            raw_score += effective_score;
        }

        WordScore {
            score: raw_score,
            word_mult,
        }
    }

    /// Find the board position where a word starts (first character).
    fn find_word_start(&self, _first_char: char, direction: Direction) -> (usize, usize) {
        // Scan the board for a word matching the character
        // This is a simplification — in practice the caller knows the position
        // We look for any edge start
        for row in 0..self.board.size {
            for col in 0..self.board.size {
                if let Some(tile) = self.board.tile_at(row, col) {
                    if tile == _first_char.to_string().as_str() {
                        // Check if this is the start of a word
                        let is_start = match direction {
                            Direction::Across => col == 0 || self.board.tile_at(row, col - 1).is_none(),
                            Direction::Down => row == 0 || self.board.tile_at(row - 1, col).is_none(),
                        };
                        if is_start {
                            // Check if reading in this direction yields a multi-letter word
                            let word = self.read_word_in_direction(row, col, direction, None);
                            if word.len() > 1 && word.starts_with(_first_char) {
                                return (row, col);
                            }
                        }
                    }
                }
            }
        }
        (0, 0) // fallback
    }

    /// Pass the current player's turn.
    pub fn pass_turn(&mut self) {
        self.consecutive_passes += 1;
        if self.consecutive_passes >= self.players.len() as u32 {
            self.end_game(None, "All players passed".to_string());
        }
        self.next_turn();
    }

    /// Exchange tiles from current player's rack.
    pub fn exchange_tiles(&mut self, letters: &[String]) -> Result<(), String> {
        let player_idx = self.current_player_index();

        let mut exchanged = Vec::new();
        for letter in letters {
            let pos = self.players[player_idx].rack.iter().position(|t| t.letter == *letter);
            match pos {
                Some(i) => {
                    exchanged.push(self.players[player_idx].rack.remove(i));
                }
                None => return Err(format!("Tile '{}' not in rack", letter)),
            }
        }

        // Put exchanged tiles back in bag
        self.tile_bag.extend(exchanged);

        // Draw new tiles
        let drawn = self.draw_tiles(letters.len());
        self.players[player_idx].rack.extend(drawn);

        Ok(())
    }

    /// End the game.
    pub fn end_game(&mut self, winner: Option<String>, reason: String) {
        self.phase = GamePhase::Finished;
        self.winner = winner;
        let _ = reason; // suppress unused warning
    }

    /// Check if the game should end.
    pub fn check_game_end(&mut self) -> bool {
        let player = &self.players[self.current_player_index()];
        if player.rack.is_empty() && self.tile_bag.is_empty() {
            let winner = self
                .players
                .iter()
                .max_by_key(|p| p.score)
                .map(|p| p.id.clone());
            self.end_game(winner, "Player used all tiles".to_string());
            return true;
        }
        false
    }

    /// Get the winner if game is finished.
    pub fn get_winner(&self) -> Option<String> {
        self.winner.clone()
    }

    /// Number of tiles remaining in bag.
    pub fn tiles_remaining(&self) -> usize {
        self.tile_bag.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_game() -> Game {
        let dict = Dictionary::load();
        Game::new(dict)
    }

    #[test]
    fn test_add_players() {
        let mut game = make_game();
        game.add_player("p1".into(), "Alice".into());
        game.add_player("p2".into(), "Bob".into());
        assert_eq!(game.players.len(), 2);
    }

    #[test]
    fn test_draw_tiles() {
        let mut game = make_game();
        let initial_count = game.tile_bag.len();
        let drawn = game.draw_tiles(7);
        assert_eq!(drawn.len(), 7);
        assert_eq!(game.tile_bag.len(), initial_count - 7);
    }

    #[test]
    fn test_deal_tiles() {
        let mut game = make_game();
        game.add_player("p1".into(), "Alice".into());
        game.add_player("p2".into(), "Bob".into());
        game.deal_tiles();
        assert_eq!(game.players[0].rack.len(), 7);
        assert_eq!(game.players[1].rack.len(), 7);
    }
}
