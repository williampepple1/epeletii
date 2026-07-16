//! Game board logic for Ibani Scrabble.
//! Standard 15x15 Scrabble board with premium squares.

use serde::{Deserialize, Serialize};

/// Direction a word is played.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Direction {
    Across,
    Down,
}

/// Premium square types on the board.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Premium {
    /// Double Letter Score
    DL,
    /// Triple Letter Score
    TL,
    /// Double Word Score
    DW,
    /// Triple Word Score
    TW,
    /// Normal square
    Normal,
}

/// A single square on the board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Square {
    pub row: usize,
    pub col: usize,
    pub premium: Premium,
    pub tile: Option<String>,   // placed tile letter
    pub owner: Option<u8>,      // player index who placed it
}

/// The 15x15 game board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub squares: Vec<Vec<Square>>,
    pub size: usize,
}

impl Board {
    /// Create a standard 15x15 Scrabble board with premium squares.
    pub fn new() -> Self {
        let size = 15;
        let mut squares = Vec::with_capacity(size);

        for row in 0..size {
            let mut row_squares = Vec::with_capacity(size);
            for col in 0..size {
                let premium = Self::premium_at(row, col);
                row_squares.push(Square {
                    row,
                    col,
                    premium,
                    tile: None,
                    owner: None,
                });
            }
            squares.push(row_squares);
        }

        Self { squares, size }
    }

    /// Get premium square type for standard Scrabble layout.
    fn premium_at(row: usize, col: usize) -> Premium {
        // Standard Scrabble board premium square layout
        // Board positions (0-indexed):
        //   TW at (0,0), (0,7), (0,14), (7,0), (7,14), (14,0), (14,7), (14,14)
        //   DW at (1,1), (2,2), (3,3), (4,4), (7,7), (10,10), (11,11), (12,12), (13,13), and symmetric
        //       plus (1,13), (2,12), (3,11), (4,10), (10,4), (11,3), (12,2), (13,1)
        //   TL at specific positions
        //   DL at remaining premium positions

        // Check for center star (7,7) — it's a DW in standard Scrabble
        if (row, col) == (7, 7) {
            return Premium::DW;
        }

        let r = row.min(14 - row); // reflect to top-left quadrant
        let c = col.min(14 - col);

        // TW positions
        if (r == 0 || r == 14) && (c == 0 || c == 7 || c == 14) {
            return Premium::TW;
        }
        // Edge-mid TW: (7,0) and (7,14) — don't reflect to (0,*) corner
        if r == 7 && (c == 0 || c == 14) {
            return Premium::TW;
        }

        // DW positions
        if r == c && (r == 1 || r == 2 || r == 3 || r == 4 || r == 10 || r == 11 || r == 12 || r == 13) {
            return Premium::DW;
        }

        // TL positions
        let tl_positions = [
            (1, 5), (1, 9),
            (5, 1), (5, 5), (5, 9), (5, 13),
            (9, 1), (9, 5), (9, 9), (9, 13),
            (13, 5), (13, 9),
        ];
        if tl_positions.contains(&(r, c)) || tl_positions.contains(&(c, r)) {
            return Premium::TL;
        }

        // DL positions
        let dl_positions = [
            (0, 3), (0, 11),
            (2, 6), (2, 8),
            (3, 0), (3, 7), (3, 14),
            (6, 2), (6, 6), (6, 8), (6, 12),
            (7, 3), (7, 11),
            (8, 2), (8, 6), (8, 8), (8, 12),
            (11, 0), (11, 7), (11, 14),
            (12, 6), (12, 8),
            (14, 3), (14, 11),
        ];
        if dl_positions.contains(&(r, c)) || dl_positions.contains(&(c, r)) {
            return Premium::DL;
        }

        Premium::Normal
    }

    /// Try to place a tile on the board.
    /// Returns an error if the square is already occupied.
    pub fn place_tile(&mut self, row: usize, col: usize, letter: &str, player: u8) -> Result<(), String> {
        if row >= self.size || col >= self.size {
            return Err(format!("Position ({}, {}) is out of bounds", row, col));
        }
        if self.squares[row][col].tile.is_some() {
            return Err(format!("Square ({}, {}) is already occupied", row, col));
        }
        self.squares[row][col].tile = Some(letter.to_string());
        self.squares[row][col].owner = Some(player);
        Ok(())
    }

    /// Get a tile at a position.
    pub fn tile_at(&self, row: usize, col: usize) -> Option<&str> {
        self.squares[row][col].tile.as_deref()
    }

    /// Check if a position is within bounds.
    pub fn in_bounds(&self, row: i32, col: i32) -> bool {
        row >= 0 && row < self.size as i32 && col >= 0 && col < self.size as i32
    }

    /// Reset the board (clear all tiles).
    pub fn reset(&mut self) {
        for row in 0..self.size {
            for col in 0..self.size {
                self.squares[row][col].tile = None;
                self.squares[row][col].owner = None;
            }
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_creation() {
        let board = Board::new();
        assert_eq!(board.squares.len(), 15);
        assert_eq!(board.squares[0].len(), 15);
    }

    #[test]
    fn test_center_is_dw() {
        let board = Board::new();
        assert_eq!(board.squares[7][7].premium, Premium::DW);
    }

    #[test]
    fn test_corners_are_tw() {
        let board = Board::new();
        assert_eq!(board.squares[0][0].premium, Premium::TW);
        assert_eq!(board.squares[0][14].premium, Premium::TW);
        assert_eq!(board.squares[14][0].premium, Premium::TW);
        assert_eq!(board.squares[14][14].premium, Premium::TW);
    }

    #[test]
    fn test_edge_mid_are_tw() {
        let board = Board::new();
        assert_eq!(board.squares[0][7].premium, Premium::TW, "(0,7) should be TW");
        assert_eq!(board.squares[7][0].premium, Premium::TW, "(7,0) should be TW");
        assert_eq!(board.squares[7][14].premium, Premium::TW, "(7,14) should be TW");
        assert_eq!(board.squares[14][7].premium, Premium::TW, "(14,7) should be TW");
    }

    #[test]
    fn test_place_and_get_tile() {
        let mut board = Board::new();
        assert!(board.place_tile(7, 7, "a", 0).is_ok());
        assert_eq!(board.tile_at(7, 7), Some("a"));
        assert!(board.place_tile(7, 7, "b", 1).is_err()); // occupied
    }
}
