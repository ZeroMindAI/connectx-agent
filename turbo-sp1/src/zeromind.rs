use rand::thread_rng;
use substrate_bn::*;
use turbo_program::{
    context::TurboActionContext,
    crypto::bn_serialize::bn254_export_affine_g1_memcpy,
    metadata::{PlayerMetadata, ServerMetadata},
    program::TurboReducer,
    traits::{HasTerminalState, TurboActionSerialization},
    zeromind::ZeroMindAgent,
};

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
    let mut public_state = PublicState::default();
    let mut private_state = PrivateState::default();
    let mut current_player = 0;

    let mut player_contexts = Vec::new();
    let mut context_refs = Vec::new();

    let mut rng = thread_rng();

    let server_random_seed = AffineG1::one() * Fr::random(&mut rng);
    let player_random_seed_0 = AffineG1::one() * Fr::random(&mut rng);
    let player_random_seed_1 = AffineG1::one() * Fr::random(&mut rng);

    let server_metadata = ServerMetadata {
        random_seed: bn254_export_affine_g1_memcpy(&server_random_seed),
    };

    let player_metadata_0 = PlayerMetadata {
        random_seed: bn254_export_affine_g1_memcpy(&player_random_seed_0),
    };

    let player_metadata_1 = PlayerMetadata {
        random_seed: bn254_export_affine_g1_memcpy(&player_random_seed_1),
    };

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
