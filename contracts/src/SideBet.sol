// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {IOptimisticOracleV3} from "./interfaces/IOptimisticOracleV3.sol";

/// @title SideBet
/// @notice Peer-to-peer betting contract settled via UMA Optimistic Oracle V3
/// @dev Bets use native ETH for wagers, USDC/WETH for oracle bonds
contract SideBet {
    // ── Types ──

    enum BetStatus {
        Proposed,   // Creator deposited, waiting for opponent
        Active,     // Both sides deposited
        Settling,   // UMA assertion in progress
        Settled,    // Resolved, funds distributed
        Cancelled   // Creator cancelled before acceptance
    }

    struct Bet {
        address creator;
        address opponent;
        uint256 creatorDeposit;
        uint256 opponentDeposit;
        bytes assertionClaim;       // UTF-8 assertion for UMA
        uint64 expiresAt;           // Deadline for opponent to accept
        bytes32 assertionId;        // Set when settlement triggered
        BetStatus status;
        address winner;             // Set after resolution
    }

    // ── State ──

    mapping(uint256 => Bet) public bets;
    uint256 public nextBetId;

    IOptimisticOracleV3 public oracle;
    IERC20 public bondCurrency;         // e.g. USDC for oracle bond
    uint64 public assertionLiveness;    // Default liveness in seconds
    uint256 public oracleBondAmount;    // Bond for UMA assertions

    // Map UMA assertionId → our betId
    mapping(bytes32 => uint256) public assertionToBet;

    address public owner;

    // ── Events ──

    event BetCreated(uint256 indexed betId, address indexed creator, address indexed opponent, uint256 amount);
    event BetAccepted(uint256 indexed betId, address indexed opponent, uint256 amount);
    event BetCancelled(uint256 indexed betId);
    event BetSettling(uint256 indexed betId, bytes32 assertionId);
    event BetSettled(uint256 indexed betId, address indexed winner, uint256 payout);
    event BetExpiryClaimed(uint256 indexed betId);

    // ── Errors ──

    error NotCreator();
    error NotOpponent();
    error InvalidStatus();
    error InsufficientDeposit();
    error BetExpired();
    error BetNotExpired();
    error TransferFailed();
    error OnlyOracle();
    error ZeroAddress();

    // ── Constructor ──

    constructor(
        address _oracle,
        address _bondCurrency,
        uint64 _assertionLiveness,
        uint256 _oracleBondAmount
    ) {
        if (_oracle == address(0) || _bondCurrency == address(0)) revert ZeroAddress();
        oracle = IOptimisticOracleV3(_oracle);
        bondCurrency = IERC20(_bondCurrency);
        assertionLiveness = _assertionLiveness;
        oracleBondAmount = _oracleBondAmount;
        owner = msg.sender;
    }

    // ── Core Functions ──

    /// @notice Create a new bet. Creator sends ETH as their wager.
    /// @param opponent Address of the other party
    /// @param assertionClaim UTF-8 encoded claim for UMA resolution
    /// @param expiresAt Timestamp deadline for opponent to accept
    function createBet(
        address opponent,
        bytes calldata assertionClaim,
        uint64 expiresAt
    ) external payable returns (uint256 betId) {
        if (opponent == address(0) || opponent == msg.sender) revert ZeroAddress();
        if (msg.value == 0) revert InsufficientDeposit();
        if (expiresAt <= block.timestamp) revert BetExpired();

        betId = nextBetId++;
        bets[betId] = Bet({
            creator: msg.sender,
            opponent: opponent,
            creatorDeposit: msg.value,
            opponentDeposit: 0,
            assertionClaim: assertionClaim,
            expiresAt: expiresAt,
            assertionId: bytes32(0),
            status: BetStatus.Proposed,
            winner: address(0)
        });

        emit BetCreated(betId, msg.sender, opponent, msg.value);
    }

    /// @notice Opponent accepts and deposits their side of the bet
    function acceptBet(uint256 betId) external payable {
        Bet storage bet = bets[betId];
        if (msg.sender != bet.opponent) revert NotOpponent();
        if (bet.status != BetStatus.Proposed) revert InvalidStatus();
        if (block.timestamp > bet.expiresAt) revert BetExpired();
        if (msg.value == 0) revert InsufficientDeposit();

        bet.opponentDeposit = msg.value;
        bet.status = BetStatus.Active;

        emit BetAccepted(betId, msg.sender, msg.value);
    }

    /// @notice Creator cancels bet before it's accepted. Refunds deposit.
    function cancelBet(uint256 betId) external {
        Bet storage bet = bets[betId];
        if (msg.sender != bet.creator) revert NotCreator();
        if (bet.status != BetStatus.Proposed) revert InvalidStatus();

        bet.status = BetStatus.Cancelled;
        uint256 refund = bet.creatorDeposit;
        bet.creatorDeposit = 0;

        (bool ok, ) = payable(msg.sender).call{value: refund}("");
        if (!ok) revert TransferFailed();

        emit BetCancelled(betId);
    }

    /// @notice Trigger settlement via UMA OOv3 assertion
    /// @dev Caller must have approved bondCurrency to the Oracle
    function settleBet(uint256 betId) external {
        Bet storage bet = bets[betId];
        if (bet.status != BetStatus.Active) revert InvalidStatus();

        // Approve bond currency to oracle
        bondCurrency.approve(address(oracle), oracleBondAmount);

        // Assert the claim via UMA
        bytes32 assertionId = oracle.assertTruth(
            bet.assertionClaim,
            address(this),          // asserter
            address(this),          // callbackRecipient
            address(0),             // escalationManager (default)
            assertionLiveness,
            bondCurrency,
            oracleBondAmount,
            bytes32("ASSERT_TRUTH"),
            bytes32(0)              // domainId
        );

        bet.assertionId = assertionId;
        bet.status = BetStatus.Settling;
        assertionToBet[assertionId] = betId;

        emit BetSettling(betId, assertionId);
    }

    /// @notice Callback from UMA Oracle when assertion resolves
    /// @dev Only callable by the Oracle contract
    function assertionResolvedCallback(
        bytes32 assertionId,
        bool assertedTruthfully
    ) external {
        if (msg.sender != address(oracle)) revert OnlyOracle();

        uint256 betId = assertionToBet[assertionId];
        Bet storage bet = bets[betId];

        if (bet.status != BetStatus.Settling) revert InvalidStatus();

        uint256 total = bet.creatorDeposit + bet.opponentDeposit;

        if (assertedTruthfully) {
            // Assertion was true → creator wins
            bet.winner = bet.creator;
        } else {
            // Assertion was false → opponent wins
            bet.winner = bet.opponent;
        }

        bet.status = BetStatus.Settled;

        // Transfer winnings
        (bool ok, ) = payable(bet.winner).call{value: total}("");
        if (!ok) revert TransferFailed();

        emit BetSettled(betId, bet.winner, total);
    }

    /// @notice Callback from UMA if assertion is disputed
    /// @dev Status stays Settling — wait for DVM resolution
    function assertionDisputedCallback(bytes32 assertionId) external {
        if (msg.sender != address(oracle)) revert OnlyOracle();
        // No state change needed. Status remains Settling.
        // DVM will eventually call assertionResolvedCallback.
    }

    /// @notice Reclaim deposit if opponent never accepted before deadline
    function claimExpired(uint256 betId) external {
        Bet storage bet = bets[betId];
        if (bet.status != BetStatus.Proposed) revert InvalidStatus();
        if (block.timestamp <= bet.expiresAt) revert BetNotExpired();

        bet.status = BetStatus.Cancelled;
        uint256 refund = bet.creatorDeposit;
        bet.creatorDeposit = 0;

        (bool ok, ) = payable(bet.creator).call{value: refund}("");
        if (!ok) revert TransferFailed();

        emit BetExpiryClaimed(betId);
    }

    // ── View Functions ──

    function getBet(uint256 betId) external view returns (Bet memory) {
        return bets[betId];
    }

    // ── Admin ──

    function updateLiveness(uint64 newLiveness) external {
        require(msg.sender == owner, "Not owner");
        assertionLiveness = newLiveness;
    }

    function updateBondAmount(uint256 newBond) external {
        require(msg.sender == owner, "Not owner");
        oracleBondAmount = newBond;
    }

    receive() external payable {}
}
