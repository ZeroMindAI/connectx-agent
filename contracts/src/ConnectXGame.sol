// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

struct GamePublicState {
    uint32[7][6] board; // 7 columns, 6 rows for Connect 4
    uint32 currentPlayer; // 1 for player 1, 2 for player 2
    uint32 winner; // 0 for no winner, 1 for player 1, 2 for player 2
    uint8[] moves;
}

struct Agent {
    bytes32 vkey;
    address owner;
    string name;
    uint256 elo; // ELO rating score
    uint256 gamesPlayed;
}

/// @title ConnectXGame.
/// @author ZeroMind
/// @notice This contract implements an AI game arena for the ConnectX game.
contract ConnectXGame {
    /// @notice The address of the SP1 verifier contract.
    /// @dev This can either be a specific SP1Verifier for a specific version, or the
    ///      SP1VerifierGateway which can be used to verify proofs for any version of SP1.
    ///      For the list of supported verifiers on each chain, see:
    ///      https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address public verifier;

    /// @notice The verification key for the game program.
    bytes32 public gameVKey;

    /// @notice Agent registry
    mapping(bytes32 => Agent) public agentRegistry;

    /// @notice K-factor for ELO calculation
    uint32 public constant K_FACTOR = 32;

    /// @notice Default starting ELO
    uint32 public constant DEFAULT_ELO = 1200;

    event AgentRegistered(
        bytes32 indexed vkey,
        address indexed owner,
        string name
    );

    event GameResult(
        bytes32 indexed agent1,
        bytes32 indexed agent2,
        uint32 winner,
        uint256 agent1NewElo,
        uint256 agent2NewElo
    );

    constructor(address _verifier, bytes32 _gameVKey) {
        verifier = _verifier;
        gameVKey = _gameVKey;
    }

    /// @notice Register an agent
    /// @param _vkey The vkey of the agent
    /// @param _gameVKey The vkey of the game
    /// @param _name The name of the agent
    function registerAgent(
        bytes32 _vkey,
        bytes32 _gameVKey,
        string memory _name
    ) public {
        require(_vkey != bytes32(0), "Vkey not set");
        require(_gameVKey != bytes32(0), "Game vkey not set");
        require(
            agentRegistry[_vkey].vkey == bytes32(0),
            "Agent already registered"
        );

        agentRegistry[_vkey] = Agent({
            vkey: _vkey,
            owner: msg.sender,
            name: _name,
            elo: DEFAULT_ELO,
            gamesPlayed: 0
        });
        emit AgentRegistered(_vkey, msg.sender, _name);
    }

    /// @notice Get an agent
    /// @param _vkey The vkey of the agent
    /// @return Agent
    function getAgent(bytes32 _vkey) public view returns (Agent memory) {
        return agentRegistry[_vkey];
    }

    /// @notice Update ELO ratings after a game
    /// @param _agent1 First agent's vkey
    /// @param _agent2 Second agent's vkey
    /// @param _winner Winner (1 for agent1, 2 for agent2, 3 for draw)
    function updateElo(
        bytes32 _agent1,
        bytes32 _agent2,
        uint32 _winner
    ) internal {
        require(
            _winner == 1 || _winner == 2 || _winner == 3,
            "Game is in progress"
        );

        Agent storage agent1 = agentRegistry[_agent1];
        Agent storage agent2 = agentRegistry[_agent2];

        // Calculate expected scores
        uint256 expectedScore1 = 1000 /
            (1 + 10 ** ((agent2.elo - agent1.elo) / 400));
        uint256 expectedScore2 = 1000 /
            (1 + 10 ** ((agent1.elo - agent2.elo) / 400));

        // Calculate actual scores (1000 = 1.0, 500 = 0.5, 0 = 0.0)
        uint32 actualScore1;
        uint32 actualScore2;
        if (_winner == 1) {
            actualScore1 = 1000;
            actualScore2 = 0;
        } else if (_winner == 2) {
            actualScore1 = 0;
            actualScore2 = 1000;
        } else {
            actualScore1 = 500;
            actualScore2 = 500;
        }

        // Update ELO ratings
        agent1.elo += (K_FACTOR * (actualScore1 - expectedScore1)) / 1000;
        agent2.elo += (K_FACTOR * (actualScore2 - expectedScore2)) / 1000;

        agent1.gamesPlayed++;
        agent2.gamesPlayed++;

        emit GameResult(_agent1, _agent2, _winner, agent1.elo, agent2.elo);
    }

    function playGame(
        bytes32 _agent1,
        bytes32 _agent2,
        bytes calldata _agent1proof,
        bytes calldata _agent2proof,
        bytes calldata _gameProof,
        bytes calldata _gamePublicValues
    ) public {
        require(
            agentRegistry[_agent1].vkey != bytes32(0),
            "Agent 1 not registered"
        );
        require(
            agentRegistry[_agent2].vkey != bytes32(0),
            "Agent 2 not registered"
        );

        // Verify moves in game public values and moves
        GamePublicState memory gamePublicState = abi.decode(
            _gamePublicValues,
            (GamePublicState)
        );

        // Verify the game proof
        ISP1Verifier(verifier).verifyProof(
            gameVKey,
            _gamePublicValues,
            _gameProof
        );

        bytes memory moves = abi.encodePacked(gamePublicState.moves);

        // Verify agent1 proof
        ISP1Verifier(verifier).verifyProof(
            agentRegistry[_agent1].vkey,
            moves,
            _agent1proof
        );

        // Verify agent2 proof
        ISP1Verifier(verifier).verifyProof(
            agentRegistry[_agent2].vkey,
            moves,
            _agent2proof
        );

        // Update ELO ratings
        updateElo(_agent1, _agent2, gamePublicState.winner);
    }
}
