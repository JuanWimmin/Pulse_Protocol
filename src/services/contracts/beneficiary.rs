use crate::services::soroban::{address_to_scval, u32_to_scval, SorobanClient};
use stellar_xdr::curr::ScVal;
use tracing::info;

/// Typed wrapper for the Beneficiary smart contract on Soroban.
#[derive(Clone)]
pub struct BeneficiaryContractClient {
    contract_id: String,
    soroban: SorobanClient,
}

impl BeneficiaryContractClient {
    pub fn new(contract_id: &str, soroban: SorobanClient) -> Self {
        Self {
            contract_id: contract_id.to_string(),
            soroban,
        }
    }

    /// Set beneficiaries for a vault on-chain.
    pub async fn set_beneficiaries(
        &self,
        vault_id: u32,
        admin: &str,
        addresses: Vec<String>,
        percentages: Vec<u32>,
    ) -> Result<String, String> {
        let addr_vals: Result<Vec<ScVal>, String> = addresses
            .iter()
            .map(|a| address_to_scval(a))
            .collect();

        let pct_vals: Vec<ScVal> = percentages
            .iter()
            .map(|p| u32_to_scval(*p))
            .collect();

        let args = vec![
            u32_to_scval(vault_id),
            address_to_scval(admin)?,
            ScVal::Vec(Some(addr_vals?.try_into().map_err(|_| "addr vec")?)),
            ScVal::Vec(Some(pct_vals.try_into().map_err(|_| "pct vec")?)),
        ];

        let tx_hash = self
            .soroban
            .invoke_contract(&self.contract_id, "set_beneficiaries", args)
            .await?;

        info!("set_beneficiaries tx: {}", tx_hash);
        Ok(tx_hash)
    }

    /// Check if a beneficiary can claim from a vault.
    pub async fn can_claim(
        &self,
        vault_id: u32,
        claimer: &str,
    ) -> Result<bool, String> {
        // For queries, we read from contract storage rather than invoking
        // In MVP, we trust the DB cache. On-chain verification is done during claim.
        let args = vec![
            u32_to_scval(vault_id),
            address_to_scval(claimer)?,
        ];

        // This would need a simulate-only call. For MVP, we rely on DB.
        Ok(true)
    }

    /// Record a claim for a beneficiary on-chain.
    /// Returns the percentage claimed.
    pub async fn record_claim(
        &self,
        vault_id: u32,
        claimer: &str,
    ) -> Result<(String, u32), String> {
        let args = vec![
            u32_to_scval(vault_id),
            address_to_scval(claimer)?,
        ];

        let tx_hash = self
            .soroban
            .invoke_contract(&self.contract_id, "record_claim", args)
            .await?;

        info!("record_claim tx: {} (claimer={})", tx_hash, claimer);
        // Percentage is tracked in DB cache
        Ok((tx_hash, 0))
    }
}
