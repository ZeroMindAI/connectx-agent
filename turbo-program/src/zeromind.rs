use crate::{
    context::TurboActionContext,
    metadata::{PlayerMetadata, ServerMetadata},
    program::TurboReducer,
    traits::TurboActionSerialization,
};

pub type ZeroMindAgent<PublicState> = fn(&PublicState, &mut TurboActionContext) -> u8;

// Currently limited to 2 players turn based games
pub fn zeromind_agent_program<PublicState, PrivateState, GameAction>(
    reducer: TurboReducer<PublicState, PrivateState, GameAction>,
    agent: ZeroMindAgent<PublicState>,
) where
    PublicState: Default,
    PrivateState: Default,
    GameAction: TurboActionSerialization,
{
    let server_metadata = sp1_zkvm::io::read::<ServerMetadata>();
    let player_metadata = sp1_zkvm::io::read::<Vec<PlayerMetadata>>();
    let actions = sp1_zkvm::io::read::<Vec<u8>>();
    let player_id = sp1_zkvm::io::read::<u8>();

    if player_id != 0 && player_id != 1 {
        panic!("Invalid player id");
    }

    let mut public_state = PublicState::default();
    let mut private_state = PrivateState::default();
    let mut current_player = 0;

    // Create contexts for all players and set them
    let mut player_contexts = Vec::new();
    let mut context_refs = Vec::new();

    // First create all the contexts
    for (i, metadata) in player_metadata.iter().enumerate() {
        player_contexts.push(TurboActionContext::new(&server_metadata, metadata, i));
    }

    // Then collect mutable references to them
    for context in &mut player_contexts {
        context_refs.push(context);
    }

    let actions_clone = actions.clone();

    // Iterate over the actions and apply them
    for action in actions {
        let context = &mut context_refs[current_player as usize];

        if current_player == player_id {
            let real_action = agent(&public_state, context);
            if action != real_action {
                panic!("Invalid action");
            }
        }

        // Process the action
        let action_parsed = GameAction::deserialize(&[action])
            .expect("Failed to deserialize action")
            .0;
        reducer(
            &mut public_state,
            &mut private_state,
            &action_parsed,
            context,
        );

        current_player = 1 - current_player;
    }

    // Return the actions result
    sp1_zkvm::io::commit_slice(&actions_clone);
}
