use game_lib::reducer::reducer;
use game_utils::print::print_public_state;
use turbo_sp1::zeromind::zeromind_run_agent;

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    let public_state = zeromind_run_agent(reducer, agent_random::agent, agent_random::agent);
    print_public_state(&public_state);

    println!("Moves: {:?}", public_state.moves);
}
