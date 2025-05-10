//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use agent_minimax::agent;
use game_lib::reducer::reducer;
use turbo_program::zeromind::zeromind_agent_program;

pub fn main() {
    zeromind_agent_program(reducer, agent);
}
