// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import {IERC20} from "forge-std/interfaces/IERC20.sol";

/// @title IOptimisticOracleV3
/// @notice Minimal interface for UMA's Optimistic Oracle V3
interface IOptimisticOracleV3 {
    struct EscalationManagerSettings {
        bool arbitrateViaEscalationManager;
        bool discardOracle;
        bool validateDisputers;
        address assertingCaller;
        address escalationManager;
    }

    /// @notice Assert a truth claim
    /// @param claim The truth claim being asserted (UTF-8 encoded)
    /// @param asserter The account making the assertion
    /// @param callbackRecipient Contract to receive assertion resolution callbacks
    /// @param escalationManager Escalation manager (0x0 for default)
    /// @param liveness Assertion liveness period in seconds
    /// @param currency ERC20 token used for the bond
    /// @param bond Assertion bond amount
    /// @param identifier Price identifier (e.g. ASSERT_TRUTH)
    /// @param domainId Domain identifier for the assertion
    /// @return assertionId Unique identifier for the assertion
    function assertTruth(
        bytes calldata claim,
        address asserter,
        address callbackRecipient,
        address escalationManager,
        uint64 liveness,
        IERC20 currency,
        uint256 bond,
        bytes32 identifier,
        bytes32 domainId
    ) external returns (bytes32 assertionId);

    /// @notice Settle an assertion after liveness period
    function settleAssertion(bytes32 assertionId) external;

    /// @notice Get the assertion result after settlement
    function getAssertionResult(bytes32 assertionId) external view returns (bool);

    /// @notice Get minimum bond amount for a currency
    function getMinimumBond(address currency) external view returns (uint256);
}
