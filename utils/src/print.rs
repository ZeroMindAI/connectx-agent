use game_lib::state::GamePublicState;

pub fn print_board(board: &[[u32; 7]; 6]) {
    for row in board {
        for col in row {
            if *col == 0 {
                print!("- ");
            } else if *col == 1 {
                print!("X ");
            } else if *col == 2 {
                print!("O ");
            }
        }
        println!();
    }
}

pub fn print_public_state(public_state: &GamePublicState) {
    if public_state.current_player == 1 {
        println!("Current player: X");
    } else if public_state.current_player == 2 {
        println!("Current player: O");
    } else {
        println!("Current player: -");
    }

    if public_state.winner == 1 {
        println!("Winner: X");
    } else if public_state.winner == 2 {
        println!("Winner: O");
    } else {
        println!("Winner: Draw");
    }

    println!();

    print_board(&public_state.board);
}
