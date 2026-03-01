// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";
import {SideBet} from "../src/SideBet.sol";
import {IOptimisticOracleV3} from "../src/interfaces/IOptimisticOracleV3.sol";

/// @dev Mock ERC20 for oracle bond
contract MockERC20 is IERC20 {
    string public name = "Mock USDC";
    string public symbol = "USDC";
    uint8 public decimals = 6;
    uint256 public totalSupply;
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    function mint(address to, uint256 amount) external {
        balanceOf[to] += amount;
        totalSupply += amount;
    }

    function transfer(address to, uint256 amount) external returns (bool) {
        balanceOf[msg.sender] -= amount;
        balanceOf[to] += amount;
        return true;
    }

    function approve(address spender, uint256 amount) external returns (bool) {
        allowance[msg.sender][spender] = amount;
        return true;
    }

    function transferFrom(address from, address to, uint256 amount) external returns (bool) {
        allowance[from][msg.sender] -= amount;
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        return true;
    }
}

/// @dev Mock UMA Oracle V3
contract MockOracle is IOptimisticOracleV3 {
    bytes32 public lastAssertionId;
    uint256 private nonce;
    mapping(bytes32 => bool) public results;
    mapping(bytes32 => bool) public settled;

    function assertTruth(
        bytes calldata,
        address,
        address,
        address,
        uint64,
        IERC20,
        uint256,
        bytes32,
        bytes32
    ) external returns (bytes32 assertionId) {
        assertionId = keccak256(abi.encodePacked(nonce++, block.timestamp));
        lastAssertionId = assertionId;
        return assertionId;
    }

    function settleAssertion(bytes32) external {}

    function getAssertionResult(bytes32 assertionId) external view returns (bool) {
        return results[assertionId];
    }

    function getMinimumBond(address) external pure returns (uint256) {
        return 1e6; // 1 USDC
    }

    // Test helpers
    function resolveAssertion(bytes32 assertionId, bool truthfully, address callback) external {
        results[assertionId] = truthfully;
        settled[assertionId] = true;
        SideBet(payable(callback)).assertionResolvedCallback(assertionId, truthfully);
    }

    function disputeAssertion(bytes32 assertionId, address callback) external {
        SideBet(payable(callback)).assertionDisputedCallback(assertionId);
    }
}

contract SideBetTest is Test {
    SideBet public sideBet;
    MockOracle public oracle;
    MockERC20 public bondToken;

    address creator = makeAddr("creator");
    address opponent = makeAddr("opponent");
    address stranger = makeAddr("stranger");

    bytes constant CLAIM = "The Los Angeles Lakers defeated the Boston Celtics on March 5, 2026.";

    function setUp() public {
        oracle = new MockOracle();
        bondToken = new MockERC20();

        sideBet = new SideBet(
            address(oracle),
            address(bondToken),
            7200,       // 2 hour liveness
            1e6         // 1 USDC bond
        );

        // Fund bond currency to the contract for oracle settlement
        bondToken.mint(address(sideBet), 100e6);

        // Fund test accounts
        vm.deal(creator, 10 ether);
        vm.deal(opponent, 10 ether);
        vm.deal(stranger, 10 ether);
    }

    // ── Test 1: Full Happy Path ──

    function test_fullHappyPath() public {
        // Create bet
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent,
            CLAIM,
            uint64(block.timestamp + 1 days)
        );
        assertEq(betId, 0);

        SideBet.Bet memory bet = sideBet.getBet(betId);
        assertEq(bet.creator, creator);
        assertEq(bet.opponent, opponent);
        assertEq(bet.creatorDeposit, 1 ether);
        assertEq(uint(bet.status), uint(SideBet.BetStatus.Proposed));

        // Accept bet
        vm.prank(opponent);
        sideBet.acceptBet{value: 1 ether}(betId);

        bet = sideBet.getBet(betId);
        assertEq(bet.opponentDeposit, 1 ether);
        assertEq(uint(bet.status), uint(SideBet.BetStatus.Active));

        // Settle — trigger UMA assertion
        sideBet.settleBet(betId);
        bet = sideBet.getBet(betId);
        assertEq(uint(bet.status), uint(SideBet.BetStatus.Settling));
        assertTrue(bet.assertionId != bytes32(0));

        // Resolve — creator wins (assertion was truthful)
        uint256 creatorBalBefore = creator.balance;
        oracle.resolveAssertion(bet.assertionId, true, address(sideBet));

        bet = sideBet.getBet(betId);
        assertEq(uint(bet.status), uint(SideBet.BetStatus.Settled));
        assertEq(bet.winner, creator);
        assertEq(creator.balance, creatorBalBefore + 2 ether);
    }

    // ── Test 2: Bet Cancellation Before Acceptance ──

    function test_cancelBeforeAcceptance() public {
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent, CLAIM, uint64(block.timestamp + 1 days)
        );

        uint256 balBefore = creator.balance;

        vm.prank(creator);
        sideBet.cancelBet(betId);

        SideBet.Bet memory bet = sideBet.getBet(betId);
        assertEq(uint(bet.status), uint(SideBet.BetStatus.Cancelled));
        assertEq(creator.balance, balBefore + 1 ether);
    }

    // ── Test 3: Bet Expiry ──

    function test_claimExpired() public {
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent, CLAIM, uint64(block.timestamp + 1 hours)
        );

        // Warp past expiry
        vm.warp(block.timestamp + 2 hours);

        uint256 balBefore = creator.balance;
        sideBet.claimExpired(betId);

        assertEq(creator.balance, balBefore + 1 ether);
        SideBet.Bet memory bet = sideBet.getBet(betId);
        assertEq(uint(bet.status), uint(SideBet.BetStatus.Cancelled));
    }

    // ── Test 4: Disputed Assertion Flow ──

    function test_disputedAssertion() public {
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent, CLAIM, uint64(block.timestamp + 1 days)
        );

        vm.prank(opponent);
        sideBet.acceptBet{value: 1 ether}(betId);

        sideBet.settleBet(betId);
        SideBet.Bet memory bet = sideBet.getBet(betId);

        // Dispute — status should stay Settling
        oracle.disputeAssertion(bet.assertionId, address(sideBet));
        bet = sideBet.getBet(betId);
        assertEq(uint(bet.status), uint(SideBet.BetStatus.Settling));

        // Eventually DVM resolves — opponent wins (assertion was false)
        uint256 opponentBalBefore = opponent.balance;
        oracle.resolveAssertion(bet.assertionId, false, address(sideBet));

        bet = sideBet.getBet(betId);
        assertEq(bet.winner, opponent);
        assertEq(opponent.balance, opponentBalBefore + 2 ether);
    }

    // ── Test 5: Edge Cases ──

    function test_revert_doubleAccept() public {
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent, CLAIM, uint64(block.timestamp + 1 days)
        );

        vm.prank(opponent);
        sideBet.acceptBet{value: 1 ether}(betId);

        vm.prank(opponent);
        vm.expectRevert(SideBet.InvalidStatus.selector);
        sideBet.acceptBet{value: 1 ether}(betId);
    }

    function test_revert_settleBeforeActive() public {
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent, CLAIM, uint64(block.timestamp + 1 days)
        );

        vm.expectRevert(SideBet.InvalidStatus.selector);
        sideBet.settleBet(betId);
    }

    function test_revert_wrongOpponent() public {
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent, CLAIM, uint64(block.timestamp + 1 days)
        );

        vm.prank(stranger);
        vm.expectRevert(SideBet.NotOpponent.selector);
        sideBet.acceptBet{value: 1 ether}(betId);
    }

    function test_revert_cancelAfterAccepted() public {
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent, CLAIM, uint64(block.timestamp + 1 days)
        );

        vm.prank(opponent);
        sideBet.acceptBet{value: 1 ether}(betId);

        vm.prank(creator);
        vm.expectRevert(SideBet.InvalidStatus.selector);
        sideBet.cancelBet(betId);
    }

    function test_revert_claimExpiredTooEarly() public {
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent, CLAIM, uint64(block.timestamp + 1 days)
        );

        vm.expectRevert(SideBet.BetNotExpired.selector);
        sideBet.claimExpired(betId);
    }

    function test_revert_zeroDeposit() public {
        vm.prank(creator);
        vm.expectRevert(SideBet.InsufficientDeposit.selector);
        sideBet.createBet{value: 0}(
            opponent, CLAIM, uint64(block.timestamp + 1 days)
        );
    }

    function test_revert_acceptAfterExpiry() public {
        vm.prank(creator);
        uint256 betId = sideBet.createBet{value: 1 ether}(
            opponent, CLAIM, uint64(block.timestamp + 1 hours)
        );

        vm.warp(block.timestamp + 2 hours);

        vm.prank(opponent);
        vm.expectRevert(SideBet.BetExpired.selector);
        sideBet.acceptBet{value: 1 ether}(betId);
    }
}
