use soroban_sdk::{contract, contractimpl, symbol_short, token, Address, Env, Vec};

use crate::errors::VaultError;
use crate::storage;
use crate::types::{Beneficiary, VaultId, VaultInfo, VaultStatus};

#[contract]
pub struct VaultContract;

#[allow(deprecated)]
#[contractimpl]
impl VaultContract {
    /// Initialize the contract with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), VaultError> {
        if storage::is_initialized(&env) {
            return Err(VaultError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_initialized(&env);
        Ok(())
    }

    /// Create a new vault. Returns the vault ID.
    pub fn create_vault(env: Env, owner: Address, token: Address) -> Result<VaultId, VaultError> {
        owner.require_auth();

        let vault_id = storage::get_vault_count(&env);
        let vault = VaultInfo {
            id: vault_id,
            owner: owner.clone(),
            token,
            status: VaultStatus::Active,
            balance: 0,
            created_at: env.ledger().timestamp(),
            last_updated: env.ledger().timestamp(),
        };

        storage::set_vault(&env, vault_id, &vault);
        storage::set_vault_count(&env, vault_id + 1);

        env.events()
            .publish((symbol_short!("vault"), symbol_short!("created")), vault_id);

        Ok(vault_id)
    }

    /// Deposit tokens into a vault.
    pub fn deposit(env: Env, vault_id: VaultId, from: Address, amount: i128) -> Result<(), VaultError> {
        if amount <= 0 {
            return Err(VaultError::ZeroAmount);
        }

        from.require_auth();

        let mut vault = storage::get_vault(&env, vault_id)
            .ok_or(VaultError::VaultNotFound)?;

        if vault.owner != from {
            return Err(VaultError::NotAuthorized);
        }

        let token_client = token::Client::new(&env, &vault.token);
        token_client.transfer(&from, &env.current_contract_address(), &amount);

        vault.balance += amount;
        vault.last_updated = env.ledger().timestamp();
        storage::set_vault(&env, vault_id, &vault);

        env.events()
            .publish((symbol_short!("vault"), symbol_short!("deposit")), (vault_id, amount));

        Ok(())
    }

    /// Withdraw tokens from a vault. Only allowed when status is Active.
    pub fn withdraw(env: Env, vault_id: VaultId, to: Address, amount: i128) -> Result<(), VaultError> {
        if amount <= 0 {
            return Err(VaultError::ZeroAmount);
        }

        let mut vault = storage::get_vault(&env, vault_id)
            .ok_or(VaultError::VaultNotFound)?;

        vault.owner.require_auth();

        if vault.status != VaultStatus::Active {
            return Err(VaultError::InvalidStatus);
        }

        if vault.balance < amount {
            return Err(VaultError::InsufficientBalance);
        }

        let token_client = token::Client::new(&env, &vault.token);
        token_client.transfer(&env.current_contract_address(), &to, &amount);

        vault.balance -= amount;
        vault.last_updated = env.ledger().timestamp();
        storage::set_vault(&env, vault_id, &vault);

        env.events()
            .publish((symbol_short!("vault"), symbol_short!("withdraw")), (vault_id, amount));

        Ok(())
    }

    /// Set beneficiaries for a vault. Sum of percentages must equal 10000.
    pub fn set_beneficiaries(
        env: Env,
        vault_id: VaultId,
        beneficiaries: Vec<Beneficiary>,
    ) -> Result<(), VaultError> {
        let vault = storage::get_vault(&env, vault_id)
            .ok_or(VaultError::VaultNotFound)?;

        vault.owner.require_auth();

        if vault.status != VaultStatus::Active {
            return Err(VaultError::InvalidStatus);
        }

        if beneficiaries.is_empty() {
            return Err(VaultError::NoBeneficiaries);
        }

        let mut sum: u32 = 0;
        for b in beneficiaries.iter() {
            sum += b.percentage;
        }
        if sum != 10_000 {
            return Err(VaultError::PercentageSumInvalid);
        }

        storage::set_beneficiaries(&env, vault_id, &beneficiaries);

        env.events()
            .publish((symbol_short!("vault"), symbol_short!("benef")), vault_id);

        Ok(())
    }

    /// Get vault info.
    pub fn get_vault(env: Env, vault_id: VaultId) -> Result<VaultInfo, VaultError> {
        storage::get_vault(&env, vault_id).ok_or(VaultError::VaultNotFound)
    }

    /// Get vault status.
    pub fn get_status(env: Env, vault_id: VaultId) -> Result<VaultStatus, VaultError> {
        let vault = storage::get_vault(&env, vault_id)
            .ok_or(VaultError::VaultNotFound)?;
        Ok(vault.status)
    }

    /// Get vault balance.
    pub fn get_balance(env: Env, vault_id: VaultId) -> Result<i128, VaultError> {
        let vault = storage::get_vault(&env, vault_id)
            .ok_or(VaultError::VaultNotFound)?;
        Ok(vault.balance)
    }

    /// Get beneficiaries for a vault.
    pub fn get_beneficiaries(env: Env, vault_id: VaultId) -> Result<Vec<Beneficiary>, VaultError> {
        if storage::get_vault(&env, vault_id).is_none() {
            return Err(VaultError::VaultNotFound);
        }
        Ok(storage::get_beneficiaries(&env, vault_id))
    }

    /// Link a ProofOfLife contract to this vault.
    pub fn link_proof_of_life(
        env: Env,
        vault_id: VaultId,
        pol_contract: Address,
    ) -> Result<(), VaultError> {
        let vault = storage::get_vault(&env, vault_id)
            .ok_or(VaultError::VaultNotFound)?;

        vault.owner.require_auth();

        storage::set_pol_link(&env, vault_id, &pol_contract);

        Ok(())
    }

    /// Transition vault status. Only callable by the linked ProofOfLife contract or admin.
    pub fn transition_status(
        env: Env,
        vault_id: VaultId,
        caller: Address,
        new_status: VaultStatus,
    ) -> Result<(), VaultError> {
        caller.require_auth();

        let mut vault = storage::get_vault(&env, vault_id)
            .ok_or(VaultError::VaultNotFound)?;

        let admin = storage::get_admin(&env);
        let pol_link = storage::get_pol_link(&env, vault_id);

        let is_admin = caller == admin;
        let is_pol = pol_link.map_or(false, |pol| caller == pol);

        if !is_admin && !is_pol {
            return Err(VaultError::NotAuthorized);
        }

        vault.status = new_status.clone();
        vault.last_updated = env.ledger().timestamp();
        storage::set_vault(&env, vault_id, &vault);

        env.events()
            .publish((symbol_short!("vault"), symbol_short!("status")), (vault_id, new_status));

        Ok(())
    }
}
