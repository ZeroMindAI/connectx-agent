use game_lib::state::GamePublicState;
use turbo_program::context::TurboActionContext;

pub fn agent(state: &GamePublicState, context: &mut TurboActionContext) -> u8 {
    // Check if a valid move is available
    if state.winner != 0 {
        return 0;
    }

    // Create a list of empty columns
    let mut empty_columns = Vec::new();
    for col in 0..7 {
        if state.board[0][col] == 0 {
            empty_columns.push(col);
        }
    }

    // Get a random move
    let idx = (context.rand_u32() % empty_columns.len() as u32) as usize;
    empty_columns[idx] as u8
}
