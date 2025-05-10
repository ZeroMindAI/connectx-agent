// Connect‑4 AI agent for Turbo environment
// Fits into:
//     pub fn agent(state: &GamePublicState, context: &mut TurboActionContext) -> u8
// The AI uses a depth‑limited minimax search with alpha–beta pruning.
// `state.board` is a 6×7 array of u8 where 0 = empty, 1 = player 1 disc, 2 = player 2 disc.
// `state.current_player` indicates whose turn it is (1 or 2).
// We ignore `context` as requested.

use game_lib::state::GamePublicState;
use turbo_program::context::TurboActionContext;

const ROWS: usize = 6;
const COLS: usize = 7;
const MAX_DEPTH: usize = 6; // adjust for stronger/slower play

/// Returns the column (0‑indexed) the agent chooses to drop a disc in.
#[allow(clippy::needless_range_loop)]
pub fn agent(state: &GamePublicState, _context: &mut TurboActionContext) -> u8 {
    // ===== Determine which piece is ours and which is the opponent's =====
    let my_piece: u8 = state.current_player;
    let opp_piece: u8 = if my_piece == 1 { 2 } else { 1 };

    // ===== Helper board wrapper with search / evaluation logic =====
    #[derive(Clone)]
    struct Board([[u8; COLS]; ROWS]);

    impl Board {
        /// Columns that are not full.
        fn valid_moves(&self) -> Vec<usize> {
            (0..COLS).filter(|&c| self.0[0][c] == 0).collect()
        }

        /// Returns a new board with `piece` dropped in `col`, or `None` if the column is full.
        fn drop(&self, col: usize, piece: u8) -> Option<Self> {
            if col >= COLS || self.0[0][col] != 0 {
                return None;
            }
            let mut next = self.clone();
            for r in (0..ROWS).rev() {
                if next.0[r][col] == 0 {
                    next.0[r][col] = piece;
                    return Some(next);
                }
            }
            None
        }

        /// Four‑in‑a‑row check for `piece`.
        fn is_win(&self, piece: u8) -> bool {
            // Horizontal
            for r in 0..ROWS {
                for c in 0..COLS - 3 {
                    if (0..4).all(|i| self.0[r][c + i] == piece) {
                        return true;
                    }
                }
            }
            // Vertical
            for c in 0..COLS {
                for r in 0..ROWS - 3 {
                    if (0..4).all(|i| self.0[r + i][c] == piece) {
                        return true;
                    }
                }
            }
            // Diagonal ↘
            for r in 0..ROWS - 3 {
                for c in 0..COLS - 3 {
                    if (0..4).all(|i| self.0[r + i][c + i] == piece) {
                        return true;
                    }
                }
            }
            // Diagonal ↗
            for r in 3..ROWS {
                for c in 0..COLS - 3 {
                    if (0..4).all(|i| self.0[r - i][c + i] == piece) {
                        return true;
                    }
                }
            }
            false
        }

        /// Scores a 4‑cell window for `piece` (heuristic).
        fn eval_window(window: &[u8; 4], piece: u8) -> i32 {
            let empty = window.iter().filter(|&&v| v == 0).count();
            let count_piece = window.iter().filter(|&&v| v == piece).count();
            match (count_piece, empty) {
                (4, _) => 1_000,
                (3, 1) => 5,
                (2, 2) => 2,
                _ => 0,
            }
        }

        /// Heuristic board evaluation from `my_piece` POV (positive is good).
        fn evaluate(&self, my_piece: u8, opp_piece: u8) -> i32 {
            let mut score = 0;
            // Center control bonus.
            let center_col = COLS / 2;
            let center_count = (0..ROWS)
                .filter(|&r| self.0[r][center_col] == my_piece)
                .count();
            score += (center_count as i32) * 6;

            // Score all 4‑cell windows in every direction.
            // Horizontal
            for r in 0..ROWS {
                for c in 0..COLS - 3 {
                    let w = [
                        self.0[r][c],
                        self.0[r][c + 1],
                        self.0[r][c + 2],
                        self.0[r][c + 3],
                    ];
                    score += Self::eval_window(&w, my_piece);
                    score -= Self::eval_window(&w, opp_piece);
                }
            }
            // Vertical
            for c in 0..COLS {
                for r in 0..ROWS - 3 {
                    let w = [
                        self.0[r][c],
                        self.0[r + 1][c],
                        self.0[r + 2][c],
                        self.0[r + 3][c],
                    ];
                    score += Self::eval_window(&w, my_piece);
                    score -= Self::eval_window(&w, opp_piece);
                }
            }
            // Diagonal ↘
            for r in 0..ROWS - 3 {
                for c in 0..COLS - 3 {
                    let w = [
                        self.0[r][c],
                        self.0[r + 1][c + 1],
                        self.0[r + 2][c + 2],
                        self.0[r + 3][c + 3],
                    ];
                    score += Self::eval_window(&w, my_piece);
                    score -= Self::eval_window(&w, opp_piece);
                }
            }
            // Diagonal ↗
            for r in 3..ROWS {
                for c in 0..COLS - 3 {
                    let w = [
                        self.0[r][c],
                        self.0[r - 1][c + 1],
                        self.0[r - 2][c + 2],
                        self.0[r - 3][c + 3],
                    ];
                    score += Self::eval_window(&w, my_piece);
                    score -= Self::eval_window(&w, opp_piece);
                }
            }
            score
        }

        /// Minimax + alpha–beta. Returns (score, best_col).
        fn minimax(
            &self,
            depth: usize,
            mut alpha: i32,
            mut beta: i32,
            maximizing: bool,
            my_piece: u8,
            opp_piece: u8,
        ) -> (i32, Option<usize>) {
            let moves = self.valid_moves();
            let terminal = self.is_win(my_piece) || self.is_win(opp_piece) || moves.is_empty();
            if depth == 0 || terminal {
                let val = if terminal {
                    if self.is_win(my_piece) {
                        1_000_000
                    } else if self.is_win(opp_piece) {
                        -1_000_000
                    } else {
                        0 // draw or full board
                    }
                } else {
                    self.evaluate(my_piece, opp_piece)
                };
                return (val, None);
            }

            let mut best_col = None;
            if maximizing {
                let mut value = i32::MIN;
                for col in moves {
                    if let Some(next) = self.drop(col, my_piece) {
                        let (score, _) =
                            next.minimax(depth - 1, alpha, beta, false, my_piece, opp_piece);
                        if score > value {
                            value = score;
                            best_col = Some(col);
                        }
                        alpha = alpha.max(value);
                        if alpha >= beta {
                            break; // β cut‑off
                        }
                    }
                }
                (value, best_col)
            } else {
                let mut value = i32::MAX;
                for col in moves {
                    if let Some(next) = self.drop(col, opp_piece) {
                        let (score, _) =
                            next.minimax(depth - 1, alpha, beta, true, my_piece, opp_piece);
                        if score < value {
                            value = score;
                            best_col = Some(col);
                        }
                        beta = beta.min(value);
                        if alpha >= beta {
                            break; // α cut‑off
                        }
                    }
                }
                (value, best_col)
            }
        }

        /// Top‑level helper that picks the best column for `my_piece`.
        fn best_move(&self, my_piece: u8, opp_piece: u8) -> usize {
            let (_, col) = self.minimax(MAX_DEPTH, i32::MIN, i32::MAX, true, my_piece, opp_piece);
            col.unwrap_or_else(|| *self.valid_moves().first().unwrap_or(&0))
        }
    }

    // Run the search from the current position and return the column.
    let root = Board(state.board);
    root.best_move(my_piece, opp_piece) as u8
}
