use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Vec};

use crate::errors::BeneficiaryError;
use crate::storage;
use crate::types::Beneficiary;

#[contract]
pub struct BeneficiaryContract;

#[allow(deprecated)]
#[contractimpl]
impl BeneficiaryContract {
    /// Initialize the contract.
    pub fn initialize(env: Env, admin: Address) -> Result<(), BeneficiaryError> {
        if storage::is_initialized(&env) {
            return Err(BeneficiaryError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_initialized(&env);
        Ok(())
    }

    /// Set the vault contract address for cross-contract calls.
    pub fn set_vault_contract(env: Env, vault_contract: Address) -> Result<(), BeneficiaryError> {
        let admin = storage::get_admin(&env);
        admin.require_auth();
        storage::set_vault_contract(&env, &vault_contract);
        Ok(())
    }

    /// Add beneficiaries for a vault. Sum of percentages must equal 10000.
    pub fn set_beneficiaries(
        env: Env,
        vault_id: u64,
        beneficiaries: Vec<Beneficiary>,
    ) -> Result<(), BeneficiaryError> {
        let admin = storage::get_admin(&env);
        admin.require_auth();

        if beneficiaries.is_empty() {
            return Err(BeneficiaryError::NoBeneficiaries);
        }

        let mut sum: u32 = 0;
        for b in beneficiaries.iter() {
            sum += b.percentage;
        }
        if sum != 10_000 {
            return Err(BeneficiaryError::PercentageSumInvalid);
        }

        storage::set_beneficiaries(&env, vault_id, &beneficiaries);

        env.events()
            .publish((symbol_short!("benef"), symbol_short!("set")), vault_id);

        Ok(())
    }

    /// Get beneficiaries for a vault.
    pub fn get_beneficiaries(env: Env, vault_id: u64) -> Vec<Beneficiary> {
        storage::get_beneficiaries(&env, vault_id)
    }

    /// Check if an address can claim from a vault.
    pub fn can_claim(env: Env, vault_id: u64, claimer: Address) -> bool {
        if storage::has_claimed(&env, vault_id, &claimer) {
            return false;
        }
        let beneficiaries = storage::get_beneficiaries(&env, vault_id);
        for b in beneficiaries.iter() {
            if b.address == claimer {
                return true;
            }
        }
        false
    }

    /// Record that a beneficiary has claimed.
    pub fn record_claim(env: Env, vault_id: u64, claimer: Address) -> Result<u32, BeneficiaryError> {
        let admin = storage::get_admin(&env);
        admin.require_auth();

        if storage::has_claimed(&env, vault_id, &claimer) {
            return Err(BeneficiaryError::AlreadyClaimed);
        }

        let beneficiaries = storage::get_beneficiaries(&env, vault_id);
        let mut percentage: u32 = 0;
        let mut found = false;

        for b in beneficiaries.iter() {
            if b.address == claimer {
                percentage = b.percentage;
                found = true;
                break;
            }
        }

        if !found {
            return Err(BeneficiaryError::NotABeneficiary);
        }

        storage::set_claimed(&env, vault_id, &claimer);

        env.events()
            .publish((symbol_short!("benef"), symbol_short!("claim")), (&claimer, vault_id));

        Ok(percentage)
    }
}
