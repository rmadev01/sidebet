// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Script.sol";
import {SideBet} from "../src/SideBet.sol";

contract DeploySideBet is Script {
    function run() external {
        // Base Sepolia UMA Oracle V3 address
        // Update to actual deployed address on Base
        address oracleAddress = vm.envAddress("UMA_ORACLE_ADDRESS");
        address bondCurrency = vm.envAddress("BOND_CURRENCY_ADDRESS");
        uint64 liveness = uint64(vm.envUint("ASSERTION_LIVENESS"));
        uint256 bondAmount = vm.envUint("ORACLE_BOND_AMOUNT");

        vm.startBroadcast();

        SideBet sideBet = new SideBet(
            oracleAddress,
            bondCurrency,
            liveness,
            bondAmount
        );

        console.log("SideBet deployed to:", address(sideBet));

        vm.stopBroadcast();
    }
}
