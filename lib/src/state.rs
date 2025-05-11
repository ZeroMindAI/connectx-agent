use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};
use turbo_program::traits::{HasActions, HasTerminalState};

sol! {
    #[derive(Serialize, Deserialize, Debug)]
    struct GamePublicState {
        uint8[7][6] board;  // 7 columns, 6 rows for Connect 4
        uint8 current_player;  // 1 for player 1, 2 for player 2
        uint8 winner;  // 0 for no winner, 1 for player 1, 2 for player 2
        uint8[] moves;
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GamePrivateState {
    pub moves: u32,
}

impl Default for GamePublicState {
    fn default() -> Self {
        Self {
            board: [[0; 7]; 6],
            current_player: 1,
            winner: 0,
            moves: vec![],
        }
    }
}

impl HasTerminalState for GamePublicState {
    fn is_terminal(&self) -> bool {
        self.winner != 0
    }
}

impl HasActions for GamePublicState {
    fn actions(&self) -> Vec<u8> {
        self.moves.clone()
    }
}
