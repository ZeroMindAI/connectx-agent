use serde_json::json;
use turbo_program::context::TurboActionContext;

use crate::{action::GameAction, state::GamePrivateState, state::GamePublicState};

/// Check if there's a winner in the current board state
fn check_winner(board: &[[u32; 7]; 6], row: usize, col: usize, player: u32) -> bool {
    // Check horizontal
    let mut count = 0;
    for c in 0..7 {
        if board[row][c] == player {
            count += 1;
            if count >= 4 {
                return true;
            }
        } else {
            count = 0;
        }
    }

    // Check vertical
    count = 0;
    for r in 0..6 {
        if board[r][col] == player {
            count += 1;
            if count >= 4 {
                return true;
            }
        } else {
            count = 0;
        }
    }

    // Check diagonal (top-left to bottom-right)
    let mut r = row as i32;
    let mut c = col as i32;
    while r > 0 && c > 0 {
        r -= 1;
        c -= 1;
    }
    count = 0;
    while r < 6 && c < 7 {
        if board[r as usize][c as usize] == player {
            count += 1;
            if count >= 4 {
                return true;
            }
        } else {
            count = 0;
        }
        r += 1;
        c += 1;
    }

    // Check diagonal (top-right to bottom-left)
    r = row as i32;
    c = col as i32;
    while r > 0 && c < 6 {
        r -= 1;
        c += 1;
    }
    count = 0;
    while r < 6 && c >= 0 {
        if board[r as usize][c as usize] == player {
            count += 1;
            if count >= 4 {
                return true;
            }
        } else {
            count = 0;
        }
        r += 1;
        c -= 1;
    }

    false
}

/// Check if the board is full
fn is_board_full(board: &[[u32; 7]; 6]) -> bool {
    for col in 0..7 {
        if board[0][col] == 0 {
            return false;
        }
    }
    true
}

pub fn reducer(
    public_state: &mut GamePublicState,
    private_state: &mut GamePrivateState,
    action: &GameAction,
    context: &mut TurboActionContext,
) {
    match action {
        GameAction::DropPiece(column) => {
            #[cfg(not(target_os = "zkvm"))]
            {
                *context.client_response() = None;
            }

            // Skip if the game is already won
            if public_state.winner != 0 {
                return;
            }

            // Validate column
            if *column >= 7 {
                return;
            }

            // Find the lowest empty row in the selected column
            let mut row = 5;
            while row > 0 && public_state.board[row][*column as usize] != 0 {
                row -= 1;
            }

            // If column is full, return
            if public_state.board[row][*column as usize] != 0 {
                return;
            }

            // Place the piece
            public_state.board[row][*column as usize] = public_state.current_player;
            private_state.moves += 1;

            println!(
                "Current player: {}, Colum: {}",
                public_state.current_player, *column
            );

            // Check for winner
            if check_winner(
                &public_state.board,
                row,
                *column as usize,
                public_state.current_player,
            ) {
                public_state.winner = public_state.current_player;
            } else if is_board_full(&public_state.board) {
                // Game is a draw
                public_state.winner = 3; // 3 represents a draw
            } else {
                // Switch players
                public_state.current_player = if public_state.current_player == 1 {
                    2
                } else {
                    1
                };
            }

            public_state.moves.push(*column);

            #[cfg(not(target_os = "zkvm"))]
            {
                *context.client_response() = Some(json!({
                    "row": row,
                    "column": column,
                    "player": public_state.current_player,
                    "winner": public_state.winner
                }));
            }
        }
    }
}
