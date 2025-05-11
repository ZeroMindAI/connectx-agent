use std::sync::Arc;

use game_lib::reducer::reducer;
use game_utils::print::print_public_state;
use sp1_sdk::{include_elf, ProverClient};
use turbo_sp1::zeromind::{zeromind_run_agent, zeromind_submit_agent, ZeromindAgentSubmission};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const GAME_ELF: &[u8] = include_elf!("game-program");
pub const AGENT_RANDOM_ELF: &[u8] = include_elf!("agent-random");
pub const AGENT_MINIMAX_ELF: &[u8] = include_elf!("agent-minimax");

fn main() {
    // ========= CONFIG YOUR AGENTS HERE =========

    let your_agent =
        ZeromindAgentSubmission::new(agent_minimax::agent, AGENT_MINIMAX_ELF, "Minimax");
    let opponent_agent =
        ZeromindAgentSubmission::new(agent_random::agent, AGENT_RANDOM_ELF, "Random");

    // ========= DO NOT TOUCH EVERYTHING BELOW HERE =========

    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    let client = Arc::new(ProverClient::from_env());

    let public_state = zeromind_submit_agent(
        client,
        reducer,
        Arc::new(GAME_ELF.to_vec()),
        your_agent,
        opponent_agent,
    )
    .unwrap();
    print_public_state(&public_state);

    println!("Moves: {:?}", public_state.moves);
}
