use crate::services::soroban::{
    address_to_scval, i128_to_scval, scval_to_i128, scval_to_u32, symbol_to_scval,
    u32_to_scval, SorobanClient,
};
use stellar_xdr::curr::ScVal;
use tracing::info;

/// Typed wrapper for the Vault smart contract on Soroban.
#[derive(Clone)]
pub struct VaultContractClient {
    contract_id: String,
    soroban: SorobanClient,
}

impl VaultContractClient {
    pub fn new(contract_id: &str, soroban: SorobanClient) -> Self {
        Self {
            contract_id: contract_id.to_string(),
            soroban,
        }
    }

    /// Create a new vault on-chain.
    /// Returns the vault_id (u32).
    pub async fn create_vault(
        &self,
        owner: &str,
        token: &str,
    ) -> Result<u32, String> {
        let args = vec![
            address_to_scval(owner)?,
            address_to_scval(token)?,
        ];

        let tx_hash = self
            .soroban
            .invoke_contract(&self.contract_id, "create_vault", args)
            .await?;

        info!("create_vault tx: {}", tx_hash);
        // The return value would be parsed from the tx result.
        // For MVP, we return 0 and track by contract_id in DB.
        Ok(0)
    }

    /// Deposit funds into a vault on-chain.
    pub async fn deposit(
        &self,
        vault_id: u32,
        depositor: &str,
        amount: i128,
        token: &str,
    ) -> Result<String, String> {
        let args = vec![
            u32_to_scval(vault_id),
            address_to_scval(depositor)?,
            i128_to_scval(amount),
            address_to_scval(token)?,
        ];

        self.soroban
            .invoke_contract(&self.contract_id, "deposit", args)
            .await
    }

    /// Withdraw funds from a vault.
    pub async fn withdraw(
        &self,
        vault_id: u32,
        caller: &str,
        amount: i128,
        token: &str,
    ) -> Result<String, String> {
        let args = vec![
            u32_to_scval(vault_id),
            address_to_scval(caller)?,
            i128_to_scval(amount),
            address_to_scval(token)?,
        ];

        self.soroban
            .invoke_contract(&self.contract_id, "withdraw", args)
            .await
    }

    /// Set beneficiaries for a vault.
    pub async fn set_beneficiaries(
        &self,
        vault_id: u32,
        caller: &str,
        beneficiary_contract: &str,
        addresses: Vec<String>,
        percentages: Vec<u32>,
    ) -> Result<String, String> {
        // Build Vec<ScVal> for addresses
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
            address_to_scval(caller)?,
            address_to_scval(beneficiary_contract)?,
            ScVal::Vec(Some(addr_vals?.try_into().map_err(|_| "addr vec")?)),
            ScVal::Vec(Some(pct_vals.try_into().map_err(|_| "pct vec")?)),
        ];

        self.soroban
            .invoke_contract(&self.contract_id, "set_beneficiaries", args)
            .await
    }

    /// Link a proof-of-life contract to the vault.
    pub async fn link_proof_of_life(
        &self,
        vault_id: u32,
        admin: &str,
        pol_contract: &str,
    ) -> Result<String, String> {
        let args = vec![
            u32_to_scval(vault_id),
            address_to_scval(admin)?,
            address_to_scval(pol_contract)?,
        ];

        self.soroban
            .invoke_contract(&self.contract_id, "link_proof_of_life", args)
            .await
    }

    /// Transition vault status (called by admin or linked PoL contract).
    pub async fn transition_status(
        &self,
        vault_id: u32,
        caller: &str,
        new_status: u32,
    ) -> Result<String, String> {
        let args = vec![
            u32_to_scval(vault_id),
            address_to_scval(caller)?,
            u32_to_scval(new_status),
        ];

        self.soroban
            .invoke_contract(&self.contract_id, "transition_status", args)
            .await
    }

    /// Get vault balance from on-chain storage.
    pub async fn get_balance(&self, vault_id: u32) -> Result<i128, String> {
        // Read the vault data from contract storage
        let key = ScVal::Vec(Some(
            vec![symbol_to_scval("Vault")?, u32_to_scval(vault_id)]
                .try_into()
                .map_err(|_| "key vec")?,
        ));

        match self.soroban.get_contract_data(&self.contract_id, key).await? {
            Some(val) => {
                // The vault struct is stored as a Map — extract the balance field
                scval_to_i128(&val).ok_or_else(|| "Cannot parse balance from on-chain data".into())
            }
            None => Ok(0),
        }
    }

    /// Get vault status from on-chain storage.
    pub async fn get_status(&self, vault_id: u32) -> Result<u32, String> {
        let key = ScVal::Vec(Some(
            vec![symbol_to_scval("Vault")?, u32_to_scval(vault_id)]
                .try_into()
                .map_err(|_| "key vec")?,
        ));

        match self.soroban.get_contract_data(&self.contract_id, key).await? {
            Some(val) => scval_to_u32(&val).ok_or_else(|| "Cannot parse status".into()),
            None => Err("Vault not found on-chain".into()),
        }
    }
}
