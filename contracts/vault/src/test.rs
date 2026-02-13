#![cfg(test)]

use super::*;
use contract::{VaultContract, VaultContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env, Vec};
use types::{Beneficiary, VaultStatus};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup_env() -> (Env, Address, VaultContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, admin, client)
}

fn create_token<'a>(env: &Env, admin: &Address) -> (Address, TokenClient<'a>, StellarAssetClient<'a>) {
    let contract_id = env.register_stellar_asset_contract_v2(admin.clone());
    let client = TokenClient::new(env, &contract_id.address());
    let admin_client = StellarAssetClient::new(env, &contract_id.address());
    (contract_id.address(), client, admin_client)
}

// ---------------------------------------------------------------------------
// 1. test_initialize
// ---------------------------------------------------------------------------

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);
}

// ---------------------------------------------------------------------------
// 2. test_initialize_twice_fails
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Second initialization should fail with AlreadyInitialized
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 3. test_create_vault
// ---------------------------------------------------------------------------

#[test]
fn test_create_vault() {
    let (env, _admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token = Address::generate(&env);

    let vault_id = client.create_vault(&owner, &token);
    assert_eq!(vault_id, 0);

    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.owner, owner);
    assert_eq!(vault.token, token);
    assert_eq!(vault.status, VaultStatus::Active);
    assert_eq!(vault.balance, 0);
}

// ---------------------------------------------------------------------------
// 4. test_create_multiple_vaults
// ---------------------------------------------------------------------------

#[test]
fn test_create_multiple_vaults() {
    let (env, _admin, client) = setup_env();

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let token = Address::generate(&env);

    let id0 = client.create_vault(&owner1, &token);
    let id1 = client.create_vault(&owner2, &token);
    let id2 = client.create_vault(&owner1, &token);

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);

    // Verify each vault has correct owner
    assert_eq!(client.get_vault(&id0).owner, owner1);
    assert_eq!(client.get_vault(&id1).owner, owner2);
    assert_eq!(client.get_vault(&id2).owner, owner1);
}

// ---------------------------------------------------------------------------
// 5. test_deposit
// ---------------------------------------------------------------------------

#[test]
fn test_deposit() {
    let (env, _admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_addr, _token_client, token_admin_client) = create_token(&env, &token_admin);

    let vault_id = client.create_vault(&owner, &token_addr);

    // Mint tokens to owner
    let mint_amount: i128 = 10_000;
    token_admin_client.mint(&owner, &mint_amount);

    // Deposit into vault
    let deposit_amount: i128 = 5_000;
    client.deposit(&vault_id, &owner, &deposit_amount);

    // Verify vault balance updated
    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.balance, deposit_amount);

    // Verify the balance from convenience getter
    let balance = client.get_balance(&vault_id);
    assert_eq!(balance, deposit_amount);
}

// ---------------------------------------------------------------------------
// 6. test_deposit_zero_amount
// ---------------------------------------------------------------------------

#[test]
fn test_deposit_zero_amount() {
    let (env, _admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token = Address::generate(&env);

    let vault_id = client.create_vault(&owner, &token);

    // Depositing zero should fail with ZeroAmount
    let result = client.try_deposit(&vault_id, &owner, &0);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 7. test_deposit_not_owner
// ---------------------------------------------------------------------------

#[test]
fn test_deposit_not_owner() {
    let (env, _admin, client) = setup_env();

    let owner = Address::generate(&env);
    let not_owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_addr, _token_client, token_admin_client) = create_token(&env, &token_admin);

    let vault_id = client.create_vault(&owner, &token_addr);

    // Mint tokens to not_owner so the transfer could theoretically work
    token_admin_client.mint(&not_owner, &10_000);

    // Deposit from not_owner should fail with NotAuthorized
    let result = client.try_deposit(&vault_id, &not_owner, &1_000);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 8. test_withdraw_active
// ---------------------------------------------------------------------------

#[test]
fn test_withdraw_active() {
    let (env, _admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_addr, token_client, token_admin_client) = create_token(&env, &token_admin);

    let vault_id = client.create_vault(&owner, &token_addr);

    // Mint and deposit
    token_admin_client.mint(&owner, &10_000);
    client.deposit(&vault_id, &owner, &10_000);

    // Withdraw partial amount
    client.withdraw(&vault_id, &owner, &3_000);

    // Vault balance should be reduced
    let vault = client.get_vault(&vault_id);
    assert_eq!(vault.balance, 7_000);

    // Owner token balance should reflect the withdrawal
    assert_eq!(token_client.balance(&owner), 3_000);
}

// ---------------------------------------------------------------------------
// 9. test_withdraw_not_active
// ---------------------------------------------------------------------------

#[test]
fn test_withdraw_not_active() {
    let (env, admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_addr, _token_client, token_admin_client) = create_token(&env, &token_admin);

    let vault_id = client.create_vault(&owner, &token_addr);

    // Mint and deposit
    token_admin_client.mint(&owner, &10_000);
    client.deposit(&vault_id, &owner, &10_000);

    // Transition vault to Alert (not Active)
    client.transition_status(&vault_id, &admin, &VaultStatus::Alert);

    // Withdrawal should fail with InvalidStatus
    let result = client.try_withdraw(&vault_id, &owner, &1_000);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 10. test_set_beneficiaries_valid
// ---------------------------------------------------------------------------

#[test]
fn test_set_beneficiaries_valid() {
    let (env, _admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let vault_id = client.create_vault(&owner, &token);

    let b1 = Beneficiary {
        address: Address::generate(&env),
        percentage: 6_000,
        claimed: false,
    };
    let b2 = Beneficiary {
        address: Address::generate(&env),
        percentage: 4_000,
        claimed: false,
    };

    let mut beneficiaries = Vec::new(&env);
    beneficiaries.push_back(b1.clone());
    beneficiaries.push_back(b2.clone());

    client.set_beneficiaries(&vault_id, &beneficiaries);

    // Retrieve and verify
    let stored = client.get_beneficiaries(&vault_id);
    assert_eq!(stored.len(), 2);
    assert_eq!(stored.get(0).unwrap().percentage, 6_000);
    assert_eq!(stored.get(1).unwrap().percentage, 4_000);
}

// ---------------------------------------------------------------------------
// 11. test_set_beneficiaries_invalid_sum
// ---------------------------------------------------------------------------

#[test]
fn test_set_beneficiaries_invalid_sum() {
    let (env, _admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let vault_id = client.create_vault(&owner, &token);

    let b1 = Beneficiary {
        address: Address::generate(&env),
        percentage: 5_000,
        claimed: false,
    };
    let b2 = Beneficiary {
        address: Address::generate(&env),
        percentage: 3_000,
        claimed: false,
    };

    let mut beneficiaries = Vec::new(&env);
    beneficiaries.push_back(b1);
    beneficiaries.push_back(b2);

    // Sum = 8000, not 10000 => PercentageSumInvalid
    let result = client.try_set_beneficiaries(&vault_id, &beneficiaries);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 12. test_set_beneficiaries_empty
// ---------------------------------------------------------------------------

#[test]
fn test_set_beneficiaries_empty() {
    let (env, _admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let vault_id = client.create_vault(&owner, &token);

    let beneficiaries: Vec<Beneficiary> = Vec::new(&env);

    // Empty list => NoBeneficiaries
    let result = client.try_set_beneficiaries(&vault_id, &beneficiaries);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 13. test_transition_status_by_admin
// ---------------------------------------------------------------------------

#[test]
fn test_transition_status_by_admin() {
    let (env, admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let vault_id = client.create_vault(&owner, &token);

    // Admin transitions from Active -> Alert
    client.transition_status(&vault_id, &admin, &VaultStatus::Alert);
    assert_eq!(client.get_status(&vault_id), VaultStatus::Alert);

    // Admin transitions from Alert -> GracePeriod
    client.transition_status(&vault_id, &admin, &VaultStatus::GracePeriod);
    assert_eq!(client.get_status(&vault_id), VaultStatus::GracePeriod);

    // Admin transitions from GracePeriod -> Triggered
    client.transition_status(&vault_id, &admin, &VaultStatus::Triggered);
    assert_eq!(client.get_status(&vault_id), VaultStatus::Triggered);

    // Admin transitions from Triggered -> Distributed
    client.transition_status(&vault_id, &admin, &VaultStatus::Distributed);
    assert_eq!(client.get_status(&vault_id), VaultStatus::Distributed);
}

// ---------------------------------------------------------------------------
// 14. test_transition_status_unauthorized
// ---------------------------------------------------------------------------

#[test]
fn test_transition_status_unauthorized() {
    let (env, _admin, client) = setup_env();

    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let vault_id = client.create_vault(&owner, &token);

    let random_user = Address::generate(&env);

    // Random user (not admin, not PoL) should fail with NotAuthorized
    let result = client.try_transition_status(&vault_id, &random_user, &VaultStatus::Alert);
    assert!(result.is_err());
}
