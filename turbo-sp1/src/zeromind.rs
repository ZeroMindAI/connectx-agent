use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use alloy_sol_types::SolValue;
use lazy_static::lazy_static;
use rand::thread_rng;
use sp1_sdk::{EnvProver, SP1ProvingKey, SP1Stdin, SP1VerifyingKey};
use substrate_bn::*;
use turbo_program::{
    context::TurboActionContext,
    crypto::bn_serialize::bn254_export_affine_g1_memcpy,
    metadata::{PlayerMetadata, ServerMetadata},
    program::TurboReducer,
    traits::{HasActions, HasTerminalState, TurboActionSerialization},
    zeromind::ZeroMindAgent,
};

lazy_static! {
    static ref SETUP_CACHE: Mutex<HashMap<Vec<u8>, Arc<(SP1ProvingKey, SP1VerifyingKey)>>> =
        Mutex::new(HashMap::new());
}

pub struct ZeromindAgentSubmission<PublicState> {
    agent: ZeroMindAgent<PublicState>,
    elf: Arc<Vec<u8>>,
    name: String,
}

impl<PublicState> ZeromindAgentSubmission<PublicState> {
    pub fn new(agent: ZeroMindAgent<PublicState>, elf: &[u8], name: &str) -> Self {
        Self {
            agent,
            elf: Arc::new(elf.to_vec()),
            name: name.to_string(),
        }
    }
}

fn setup_circuit(
    client: Arc<EnvProver>,
    elf: &[u8],
) -> Result<Arc<(SP1ProvingKey, SP1VerifyingKey)>, &'static str> {
    let mut cache = SETUP_CACHE.lock().map_err(|_| "Failed to lock cache")?;
    if let Some(arc) = cache.get(elf) {
        Ok(arc.clone())
    } else {
        let (pk, vk) = client.setup(elf);
        let arc = Arc::new((pk.clone(), vk.clone()));
        cache.insert(elf.to_vec(), arc.clone());
        Ok(arc)
    }
}

fn zeromind_submit_elf(
    client: Arc<EnvProver>,
    elf: &[u8],
    name: &str,
) -> Result<Arc<(SP1ProvingKey, SP1VerifyingKey)>, String> {
    let keys = setup_circuit(client, elf).map_err(|e| e.to_string())?;
    Ok(keys)
}

fn make_metadata() -> (ServerMetadata, PlayerMetadata, PlayerMetadata) {
    let mut rng = thread_rng();

    let server_random_seed = AffineG1::one() * Fr::random(&mut rng);
    let player_random_seed_0 = AffineG1::one() * Fr::random(&mut rng);
    let player_random_seed_1 = AffineG1::one() * Fr::random(&mut rng);

    (
        ServerMetadata {
            random_seed: bn254_export_affine_g1_memcpy(&server_random_seed),
        },
        PlayerMetadata {
            random_seed: bn254_export_affine_g1_memcpy(&player_random_seed_0),
        },
        PlayerMetadata {
            random_seed: bn254_export_affine_g1_memcpy(&player_random_seed_1),
        },
    )
}

fn zeromind_run_agent_inner<PublicState, PrivateState, GameAction>(
    reducer: TurboReducer<PublicState, PrivateState, GameAction>,
    agent1: ZeroMindAgent<PublicState>,
    agent2: ZeroMindAgent<PublicState>,
    server_metadata: ServerMetadata,
    player_metadata_0: PlayerMetadata,
    player_metadata_1: PlayerMetadata,
) -> PublicState
where
    PublicState: Default + HasTerminalState,
    PrivateState: Default,
    GameAction: TurboActionSerialization,
{
    let mut public_state = PublicState::default();
    let mut private_state = PrivateState::default();
    let mut current_player = 0;

    let mut player_contexts = Vec::new();
    let mut context_refs = Vec::new();

    // First create all the contexts
    player_contexts.push(TurboActionContext::new(
        &server_metadata,
        &player_metadata_0,
        0,
    ));

    player_contexts.push(TurboActionContext::new(
        &server_metadata,
        &player_metadata_1,
        1,
    ));

    // Then collect mutable references to them
    for context in &mut player_contexts {
        context_refs.push(context);
    }

    // Run the game
    while !public_state.is_terminal() {
        let action = if current_player == 0 {
            agent1(&public_state, &mut player_contexts[0])
        } else {
            agent2(&public_state, &mut player_contexts[1])
        };

        let action_parsed = GameAction::deserialize(&[action])
            .expect("Failed to deserialize action")
            .0;

        reducer(
            &mut public_state,
            &mut private_state,
            &action_parsed,
            &mut player_contexts[current_player],
        );
        current_player = 1 - current_player;
    }

    public_state
}

pub fn zeromind_run_agent<PublicState, PrivateState, GameAction>(
    reducer: TurboReducer<PublicState, PrivateState, GameAction>,
    agent1: ZeroMindAgent<PublicState>,
    agent2: ZeroMindAgent<PublicState>,
) -> PublicState
where
    PublicState: Default + HasTerminalState,
    PrivateState: Default,
    GameAction: TurboActionSerialization,
{
    let (server_metadata, player_metadata_0, player_metadata_1) = make_metadata();

    zeromind_run_agent_inner(
        reducer,
        agent1,
        agent2,
        server_metadata,
        player_metadata_0,
        player_metadata_1,
    )
}

pub fn zeromind_submit_agent<PublicState, PrivateState, GameAction>(
    client: Arc<EnvProver>,
    reducer: TurboReducer<PublicState, PrivateState, GameAction>,
    game_elf: Arc<Vec<u8>>,
    agent1: ZeromindAgentSubmission<PublicState>,
    agent2: ZeromindAgentSubmission<PublicState>,
) -> Result<PublicState, String>
where
    PublicState: Default
        + HasTerminalState
        + HasActions
        + SolValue
        + From<<<PublicState as SolValue>::SolType as alloy_sol_types::SolType>::RustType>,
    PrivateState: Default,
    GameAction: TurboActionSerialization,
{
    let keysGame = setup_circuit(client.clone(), game_elf.as_ref())?;
    let keys1 = zeromind_submit_elf(client.clone(), agent1.elf.as_ref(), &agent1.name)?;
    let keys2 = zeromind_submit_elf(client.clone(), agent2.elf.as_ref(), &agent2.name)?;

    let (server_metadata, player_metadata_0, player_metadata_1) = make_metadata();

    let result = zeromind_run_agent_inner(
        reducer,
        agent1.agent,
        agent2.agent,
        server_metadata.clone(),
        player_metadata_0.clone(),
        player_metadata_1.clone(),
    );

    let actions = result.actions();

    let mut stdin = SP1Stdin::new();
    stdin.write(&server_metadata);
    stdin.write(&vec![player_metadata_0, player_metadata_1]);

    let mut stdin_game = stdin.clone();
    let mut actions_game: Vec<u8> = Vec::new();
    for (i, action) in actions.iter().enumerate() {
        actions_game.push((i % 2) as u8); // player turn
        actions_game.push(*action);
    }
    stdin_game.write(&actions_game);

    let mut stdin0 = stdin.clone();
    stdin0.write(&actions);
    stdin0.write(&0);

    let mut stdin1 = stdin.clone();
    stdin1.write(&actions);
    stdin1.write(&1);

    // Verify game execution
    {
        let (public_values, report) = client
            .execute(game_elf.as_ref(), &stdin_game)
            .run()
            .map_err(|_| "Failed to execute circuit")?;

        let game_state: PublicState = PublicState::abi_decode(public_values.as_slice())
            .map_err(|_| "Failed to decode game state")?;

        // Check if moves match
        if game_state.actions() != actions {
            return Err("Game moves do not match expected moves".to_string());
        }

        // Check if there is a winner (not 0) and matches result
        if !game_state.is_terminal() {
            return Err("Game did not reach terminal state".to_string());
        }

        // Check if the result matches
        if PublicState::abi_encode(&result) != public_values.as_slice() {
            return Err("Game result does not match expected result".to_string());
        }

        println!(
            "Game result verified ({} cycles)",
            report.total_instruction_count()
        );
    }

    // Verify agent 1 moves
    {
        // Try executing the circuit first
        let (public_values, report) = client
            .execute(agent1.elf.as_ref(), &stdin0)
            .run()
            .map_err(|_| "Failed to execute circuit")?;

        // Verify the public values match actions
        let actions_public_values: sp1_sdk::SP1PublicValues = public_values;
        if actions_public_values.as_slice() != actions {
            return Err("Actions do not match".to_string());
        }

        println!(
            "Agent 1 moves verified ({} cycles)",
            report.total_instruction_count()
        );
    }

    // Verify agent 2 moves
    {
        // Try executing the circuit first
        let (public_values, report) = client
            .execute(agent2.elf.as_ref(), &stdin1)
            .run()
            .map_err(|_| "Failed to execute circuit")?;

        // Verify the public values match actions
        let actions_public_values: sp1_sdk::SP1PublicValues = public_values;
        if actions_public_values.as_slice() != actions {
            return Err("Actions do not match".to_string());
        }

        println!(
            "Agent 2 moves verified ({} cycles)",
            report.total_instruction_count()
        );
    }

    // Generate game proof
    let gameProof = client
        .prove(&keysGame.0, &stdin_game)
        .groth16()
        .run()
        .map_err(|_| "Failed to generate proof")?;

    println!("Game proof generated");

    // Generate agent 1 proof
    let agent1Proof = client
        .prove(&keys1.0, &stdin0)
        .groth16()
        .run()
        .map_err(|_| "Failed to generate proof")?;

    println!("Agent 1 proof generated");

    // Generate agent 2 proof
    let agent2Proof = client
        .prove(&keys2.0, &stdin1)
        .groth16()
        .run()
        .map_err(|_| "Failed to generate proof")?;

    println!("Agent 2 proof generated");

    Ok(result)
}
