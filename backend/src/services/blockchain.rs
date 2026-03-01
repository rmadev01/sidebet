/// Blockchain service — contract interaction via JSON-RPC.
/// In a full implementation this would use ethers-rs to:
/// - Call createBet / acceptBet / settleBet on SideBet.sol
/// - Listen for assertionResolvedCallback events
/// - Read bet state from the contract
///
/// For now, this provides the interface stubs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct BlockchainService {
    pub rpc_url: String,
    pub contract_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnChainBet {
    pub bet_id: u64,
    pub creator: String,
    pub opponent: String,
    pub creator_deposit: String,
    pub opponent_deposit: String,
    pub status: u8,
}

impl BlockchainService {
    pub fn new(rpc_url: String, contract_address: String) -> Self {
        Self {
            rpc_url,
            contract_address,
        }
    }

    /// Read a bet from the contract (stub)
    pub async fn get_bet(&self, bet_id: u64) -> Result<OnChainBet, String> {
        // TODO: implement actual contract call via ethers-rs
        tracing::info!("Reading bet {} from contract {}", bet_id, self.contract_address);
        Err("Not implemented — wire up ethers-rs contract bindings".into())
    }

    /// Get the next bet ID from the contract (stub)
    pub async fn next_bet_id(&self) -> Result<u64, String> {
        tracing::info!("Reading nextBetId from {}", self.contract_address);
        Err("Not implemented".into())
    }
}
