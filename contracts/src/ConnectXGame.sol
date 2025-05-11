// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";
import {FixedPointMathLib} from "@solmate/utils/FixedPointMathLib.sol";

struct GamePublicState {
    uint8[7][6] board; // 7 columns, 6 rows for Connect 4
    uint8 currentPlayer; // 1 for player 1, 2 for player 2
    uint8 winner; // 0 for no winner, 1 for player 1, 2 for player 2
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
    using FixedPointMathLib for uint256;

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
    uint256 public constant K_FACTOR = 32;

    /// @notice Default starting ELO
    uint256 public constant DEFAULT_ELO = 1200;

    /// @notice Scale factor for fixed point math (1e18)
    uint256 private constant SCALE = 1e18;

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
    /// @param _name The name of the agent
    function registerAgent(bytes32 _vkey, string memory _name) public {
        require(_vkey != bytes32(0), "Vkey not set");

        if (agentRegistry[_vkey].vkey == bytes32(0)) {
            agentRegistry[_vkey] = Agent({
                vkey: _vkey,
                owner: msg.sender,
                name: _name,
                elo: DEFAULT_ELO,
                gamesPlayed: 0
            });
            emit AgentRegistered(_vkey, msg.sender, _name);
        }
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

        uint256 elo1 = agent1.elo;
        uint256 elo2 = agent2.elo;

        // Calculate expected scores using fixed point math
        uint256 eloDiff1 = (elo2 > elo1) ? elo2 - elo1 : elo1 - elo2;
        uint256 expectedScore1 = SCALE.divWadDown(
            SCALE + uint256(10).rpow((eloDiff1 * SCALE) / 400, SCALE)
        );
        uint256 expectedScore2 = SCALE - expectedScore1;

        // Calculate actual scores (SCALE = 1.0, SCALE/2 = 0.5, 0 = 0.0)
        uint256 actualScore1;
        uint256 actualScore2;
        if (_winner == 1) {
            actualScore1 = SCALE;
            actualScore2 = 0;
        } else if (_winner == 2) {
            actualScore1 = 0;
            actualScore2 = SCALE;
        } else {
            actualScore1 = SCALE / 2;
            actualScore2 = SCALE / 2;
        }

        // Update ELO ratings
        uint256 delta1 = (K_FACTOR *
            (
                actualScore1 > expectedScore1
                    ? actualScore1 - expectedScore1
                    : expectedScore1 - actualScore1
            )) / SCALE;
        uint256 delta2 = (K_FACTOR *
            (
                actualScore2 > expectedScore2
                    ? actualScore2 - expectedScore2
                    : expectedScore2 - actualScore2
            )) / SCALE;

        if (actualScore1 > expectedScore1) {
            agent1.elo = agent1.elo + delta1;
            agent2.elo = agent2.elo > delta2 ? agent2.elo - delta2 : 0;
        } else {
            agent1.elo = agent1.elo > delta1 ? agent1.elo - delta1 : 0;
            agent2.elo = agent2.elo + delta2;
        }

        agent1.gamesPlayed++;
        agent2.gamesPlayed++;

        emit GameResult(_agent1, _agent2, _winner, agent1.elo, agent2.elo);
    }

    function toBytes(uint8[] memory arr) internal pure returns (bytes memory) {
        bytes memory b = new bytes(arr.length);
        for (uint i = 0; i < arr.length; i++) {
            b[i] = bytes1(arr[i]);
        }
        return b;
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

        bytes memory moves = toBytes(gamePublicState.moves);

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
