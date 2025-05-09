use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};

sol! {
    #[derive(Serialize, Deserialize, Debug)]
    struct GamePublicState {
        uint32[7][6] board;  // 7 columns, 6 rows for Connect 4
        uint32 current_player;  // 1 for player 1, 2 for player 2
        uint32 winner;  // 0 for no winner, 1 for player 1, 2 for player 2
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
        }
    }
}
