#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _, Address, Bytes, BytesN, Env, String, Vec,
};
use soroban_sdk::token::{StellarAssetClient, TokenClient};

// ---------------------------------------------------------------------------
// Import contract structs (for registration) and clients (for calling)
// ---------------------------------------------------------------------------

use pulse_vault::contract::{VaultContract, VaultContractClient};
use pulse_vault::types::{
    Beneficiary as VaultBeneficiary, VaultId, VaultStatus,
};

use pulse_proof_of_life::contract::{ProofOfLifeContract, ProofOfLifeContractClient};
use pulse_proof_of_life::types::VerificationSource;

use pulse_beneficiary::contract::{BeneficiaryContract, BeneficiaryContractClient};
use pulse_beneficiary::types::Beneficiary;

use pulse_document_registry::contract::{DocumentRegistryContract, DocumentRegistryContractClient};
use pulse_document_registry::types::DocumentType;

// ===========================================================================
// Helpers
// ===========================================================================

/// Create and register a Stellar asset, returning its address plus both clients.
fn create_token<'a>(
    env: &Env,
    admin: &Address,
) -> (Address, TokenClient<'a>, StellarAssetClient<'a>) {
    let id = env.register_stellar_asset_contract_v2(admin.clone());
    (
        id.address(),
        TokenClient::new(env, &id.address()),
        StellarAssetClient::new(env, &id.address()),
    )
}

/// Build a Vec<i128> of 10 weights for the PoL perceptron model.
fn create_weights(env: &Env) -> Vec<i128> {
    let mut weights = Vec::new(env);
    for _ in 0..10 {
        weights.push_back(500_000i128); // 0.5 fixed-point
    }
    weights
}

/// Convenience struct that bundles the full test environment.
struct TestEnv<'a> {
    env: Env,
    admin: Address,
    #[allow(dead_code)]
    oracle: Address,

    vault_client: VaultContractClient<'a>,
    vault_addr: Address,

    pol_client: ProofOfLifeContractClient<'a>,
    pol_addr: Address,

    benef_client: BeneficiaryContractClient<'a>,
    #[allow(dead_code)]
    benef_addr: Address,

    doc_client: DocumentRegistryContractClient<'a>,
    #[allow(dead_code)]
    doc_addr: Address,
}

/// Register and initialise all four contracts with a shared admin.
fn setup_full_env<'a>() -> TestEnv<'a> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);

    // -- Vault --
    let vault_addr = env.register(VaultContract, ());
    let vault_client = VaultContractClient::new(&env, &vault_addr);
    vault_client.initialize(&admin);

    // -- ProofOfLife --
    let pol_addr = env.register(ProofOfLifeContract, ());
    let pol_client = ProofOfLifeContractClient::new(&env, &pol_addr);
    pol_client.initialize(&admin, &oracle);

    // -- Beneficiary --
    let benef_addr = env.register(BeneficiaryContract, ());
    let benef_client = BeneficiaryContractClient::new(&env, &benef_addr);
    benef_client.initialize(&admin);
    benef_client.set_vault_contract(&vault_addr);

    // -- DocumentRegistry --
    let doc_addr = env.register(DocumentRegistryContract, ());
    let doc_client = DocumentRegistryContractClient::new(&env, &doc_addr);
    doc_client.initialize(&admin);

    TestEnv {
        env,
        admin,
        oracle,
        vault_client,
        vault_addr,
        pol_client,
        pol_addr,
        benef_client,
        benef_addr,
        doc_client,
        doc_addr,
    }
}

// ===========================================================================
// Test 1 - Full Inheritance Lifecycle
// ===========================================================================

#[test]
fn test_full_inheritance_lifecycle() {
    let t = setup_full_env();

    // ── Actors ───────────────────────────────────────────────────────────
    let owner = Address::generate(&t.env);
    let beneficiary1 = Address::generate(&t.env);
    let beneficiary2 = Address::generate(&t.env);
    let token_admin = Address::generate(&t.env);

    // ── 1. Create token and mint to owner ────────────────────────────────
    let (token_addr, token_client, token_admin_client) =
        create_token(&t.env, &token_admin);
    let mint_amount: i128 = 1_000_000;
    token_admin_client.mint(&owner, &mint_amount);
    assert_eq!(token_client.balance(&owner), mint_amount);

    // ── 2. Create vault, deposit tokens ──────────────────────────────────
    let vault_id: VaultId = t.vault_client.create_vault(&owner, &token_addr);
    assert_eq!(vault_id, 0);

    let deposit_amount: i128 = 500_000;
    t.vault_client.deposit(&vault_id, &owner, &deposit_amount);
    assert_eq!(t.vault_client.get_balance(&vault_id), deposit_amount);

    // ── 3. Register PoL model for owner ──────────────────────────────────
    let weights = create_weights(&t.env);
    t.pol_client
        .register_model(&owner, &weights, &100_000i128);
    let model = t.pol_client.get_model(&owner);
    assert_eq!(model.version, 1);
    assert_eq!(model.weights.len(), 10);

    // ── 4. Link vault to PoL (both directions) ──────────────────────────
    t.vault_client
        .link_proof_of_life(&vault_id, &t.pol_addr);
    t.pol_client
        .link_vault(&owner, &t.vault_addr, &vault_id);

    // ── 5. Set beneficiaries on vault contract (60 / 40) ────────────────
    let vb1 = VaultBeneficiary {
        address: beneficiary1.clone(),
        percentage: 6_000,
        claimed: false,
    };
    let vb2 = VaultBeneficiary {
        address: beneficiary2.clone(),
        percentage: 4_000,
        claimed: false,
    };
    let mut vault_benefs = Vec::new(&t.env);
    vault_benefs.push_back(vb1);
    vault_benefs.push_back(vb2);
    t.vault_client
        .set_beneficiaries(&vault_id, &vault_benefs);

    let stored_vb = t.vault_client.get_beneficiaries(&vault_id);
    assert_eq!(stored_vb.len(), 2);
    assert_eq!(stored_vb.get(0).unwrap().percentage, 6_000);
    assert_eq!(stored_vb.get(1).unwrap().percentage, 4_000);

    // ── 6. Set beneficiaries on beneficiary contract ────────────────────
    let bb1 = Beneficiary {
        address: beneficiary1.clone(),
        percentage: 6_000,
        claimed: false,
    };
    let bb2 = Beneficiary {
        address: beneficiary2.clone(),
        percentage: 4_000,
        claimed: false,
    };
    let mut benef_list = Vec::new(&t.env);
    benef_list.push_back(bb1);
    benef_list.push_back(bb2);
    t.benef_client
        .set_beneficiaries(&vault_id, &benef_list);

    assert!(t.benef_client.can_claim(&vault_id, &beneficiary1));
    assert!(t.benef_client.can_claim(&vault_id, &beneficiary2));

    // ── 7. Register a document and link to vault ────────────────────────
    let ipfs_cid = String::from_str(&t.env, "QmTestDocumentCIDForInheritanceLifecycle");
    let doc_hash = BytesN::from_array(&t.env, &[1u8; 32]);
    let doc_id = t.doc_client.register_document(
        &owner,
        &ipfs_cid,
        &doc_hash,
        &DocumentType::Will,
        &false,
    );
    t.doc_client.link_to_vault(&doc_id, &vault_id);

    let vault_docs = t.doc_client.get_vault_documents(&vault_id);
    assert_eq!(vault_docs.len(), 1);
    assert_eq!(vault_docs.get(0).unwrap(), doc_id);

    // ── 8. Submit HIGH verification score (> 7000) => Active ────────────
    let sig = BytesN::from_array(&t.env, &[0u8; 64]);
    t.pol_client.submit_verification(
        &owner,
        &8_500u32,
        &VerificationSource::PerceptronAggregate,
        &sig,
    );
    assert_eq!(t.pol_client.get_liveness_score(&owner), 8_500);
    // Vault should still be Active
    assert_eq!(
        t.vault_client.get_status(&vault_id),
        VaultStatus::Active
    );

    // ── 9. Submit LOW score (< 7000 but > 3000) => transition to Alert ──
    let sig2 = BytesN::from_array(&t.env, &[2u8; 64]);
    t.pol_client.submit_verification(
        &owner,
        &5_000u32,
        &VerificationSource::BehaviorPattern,
        &sig2,
    );
    assert_eq!(t.pol_client.get_liveness_score(&owner), 5_000);

    // Admin transitions vault to Alert (simulating oracle-driven transition)
    t.vault_client
        .transition_status(&vault_id, &t.admin, &VaultStatus::Alert);
    assert_eq!(
        t.vault_client.get_status(&vault_id),
        VaultStatus::Alert
    );

    // ── 10. Submit VERY LOW score (< 3000) => GracePeriod ───────────────
    let sig3 = BytesN::from_array(&t.env, &[3u8; 64]);
    t.pol_client.submit_verification(
        &owner,
        &2_000u32,
        &VerificationSource::PerceptronAggregate,
        &sig3,
    );
    assert_eq!(t.pol_client.get_liveness_score(&owner), 2_000);

    // Admin transitions vault to GracePeriod
    t.vault_client
        .transition_status(&vault_id, &t.admin, &VaultStatus::GracePeriod);
    assert_eq!(
        t.vault_client.get_status(&vault_id),
        VaultStatus::GracePeriod
    );

    // ── 11. Grace period timeout => Triggered ───────────────────────────
    t.vault_client
        .transition_status(&vault_id, &t.admin, &VaultStatus::Triggered);
    assert_eq!(
        t.vault_client.get_status(&vault_id),
        VaultStatus::Triggered
    );

    // ── 12. Withdrawals should fail (vault is Triggered, not Active) ────
    let withdraw_result =
        t.vault_client
            .try_withdraw(&vault_id, &owner, &1_000);
    assert!(withdraw_result.is_err());

    // ── 13. Record claims on beneficiary contract ───────────────────────
    let pct1 = t
        .benef_client
        .record_claim(&vault_id, &beneficiary1);
    assert_eq!(pct1, 6_000);

    let pct2 = t
        .benef_client
        .record_claim(&vault_id, &beneficiary2);
    assert_eq!(pct2, 4_000);

    // Cannot claim twice
    let double_claim = t
        .benef_client
        .try_record_claim(&vault_id, &beneficiary1);
    assert!(double_claim.is_err());
    assert!(!t.benef_client.can_claim(&vault_id, &beneficiary1));

    // ── 14. Grant document access to beneficiaries ──────────────────────
    let access1 = t
        .doc_client
        .grant_access(&doc_id, &beneficiary1);
    assert_eq!(access1.doc_type, DocumentType::Will);
    assert!(!access1.is_encrypted);

    let access2 = t
        .doc_client
        .grant_access(&doc_id, &beneficiary2);
    assert_eq!(access2.doc_type, DocumentType::Will);

    // ── 15. Transition to Distributed ───────────────────────────────────
    t.vault_client
        .transition_status(&vault_id, &t.admin, &VaultStatus::Distributed);
    assert_eq!(
        t.vault_client.get_status(&vault_id),
        VaultStatus::Distributed
    );

    // ── Final assertions ────────────────────────────────────────────────
    // Vault balance is still tracked (actual distribution is external)
    assert_eq!(t.vault_client.get_balance(&vault_id), deposit_amount);
    // Document is verified
    let proof = t.doc_client.verify_document(&doc_id);
    assert!(proof.exists);
    assert_eq!(proof.doc_hash, BytesN::from_array(&t.env, &[1u8; 32]));
}

// ===========================================================================
// Test 2 - Emergency Recovery
// ===========================================================================

#[test]
fn test_emergency_recovery() {
    let t = setup_full_env();

    let owner = Address::generate(&t.env);
    let token_admin = Address::generate(&t.env);

    // ── 1. Setup token + vault + PoL ─────────────────────────────────────
    let (token_addr, token_client, token_admin_client) =
        create_token(&t.env, &token_admin);
    token_admin_client.mint(&owner, &100_000);

    let vault_id = t.vault_client.create_vault(&owner, &token_addr);
    t.vault_client.deposit(&vault_id, &owner, &50_000);

    // Register PoL model and link vault
    let weights = create_weights(&t.env);
    t.pol_client
        .register_model(&owner, &weights, &100_000i128);
    t.vault_client
        .link_proof_of_life(&vault_id, &t.pol_addr);
    t.pol_client
        .link_vault(&owner, &t.vault_addr, &vault_id);

    // Default score should be max (10000)
    assert_eq!(t.pol_client.get_liveness_score(&owner), 10_000);
    assert_eq!(
        t.vault_client.get_status(&vault_id),
        VaultStatus::Active
    );

    // ── 2. Submit low score => transition to Alert ──────────────────────
    let sig = BytesN::from_array(&t.env, &[0u8; 64]);
    t.pol_client.submit_verification(
        &owner,
        &5_000u32,
        &VerificationSource::BehaviorPattern,
        &sig,
    );
    assert_eq!(t.pol_client.get_liveness_score(&owner), 5_000);

    // Admin transitions to Alert
    t.vault_client
        .transition_status(&vault_id, &t.admin, &VaultStatus::Alert);
    assert_eq!(
        t.vault_client.get_status(&vault_id),
        VaultStatus::Alert
    );

    // Withdrawal should fail while in Alert
    let withdraw_attempt =
        t.vault_client.try_withdraw(&vault_id, &owner, &1_000);
    assert!(withdraw_attempt.is_err());

    // ── 3. Emergency check-in => score back to 10000 ────────────────────
    t.pol_client.emergency_checkin(&owner);
    assert_eq!(t.pol_client.get_liveness_score(&owner), 10_000);

    // ── 4. Admin transitions back to Active ─────────────────────────────
    t.vault_client
        .transition_status(&vault_id, &t.admin, &VaultStatus::Active);
    assert_eq!(
        t.vault_client.get_status(&vault_id),
        VaultStatus::Active
    );

    // ── 5. Verify vault is Active, deposits can be withdrawn ────────────
    t.vault_client.withdraw(&vault_id, &owner, &20_000);
    assert_eq!(t.vault_client.get_balance(&vault_id), 30_000);
    assert_eq!(token_client.balance(&owner), 70_000); // 100k - 50k deposited + 20k withdrawn

    // Additional deposit should also work
    t.vault_client.deposit(&vault_id, &owner, &10_000);
    assert_eq!(t.vault_client.get_balance(&vault_id), 40_000);
}

// ===========================================================================
// Test 3 - Document Inheritance Flow
// ===========================================================================

#[test]
fn test_document_inheritance_flow() {
    let t = setup_full_env();

    let owner = Address::generate(&t.env);
    let beneficiary1 = Address::generate(&t.env);
    let token_admin = Address::generate(&t.env);

    // ── 1. Setup vault ──────────────────────────────────────────────────
    let (token_addr, _token_client, token_admin_client) =
        create_token(&t.env, &token_admin);
    token_admin_client.mint(&owner, &100_000);

    let vault_id = t.vault_client.create_vault(&owner, &token_addr);
    t.vault_client.deposit(&vault_id, &owner, &50_000);

    // ── 2. Register public document ─────────────────────────────────────
    let public_cid =
        String::from_str(&t.env, "QmPublicManifestCIDForInheritanceFlow12345");
    let public_hash = BytesN::from_array(&t.env, &[1u8; 32]);
    let public_doc_id = t.doc_client.register_document(
        &owner,
        &public_cid,
        &public_hash,
        &DocumentType::PublicManifest,
        &false,
    );

    // ── 3. Register encrypted document ──────────────────────────────────
    let encrypted_cid =
        String::from_str(&t.env, "QmEncryptedWillCIDForInheritanceFlowTest");
    let encrypted_hash = BytesN::from_array(&t.env, &[2u8; 32]);
    let encrypted_doc_id = t.doc_client.register_document(
        &owner,
        &encrypted_cid,
        &encrypted_hash,
        &DocumentType::Will,
        &true,
    );

    // ── 4. Link both documents to vault ─────────────────────────────────
    t.doc_client.link_to_vault(&public_doc_id, &vault_id);
    t.doc_client
        .link_to_vault(&encrypted_doc_id, &vault_id);

    let vault_docs = t.doc_client.get_vault_documents(&vault_id);
    assert_eq!(vault_docs.len(), 2);
    assert_eq!(vault_docs.get(0).unwrap(), public_doc_id);
    assert_eq!(vault_docs.get(1).unwrap(), encrypted_doc_id);

    // ── 5. Store encrypted AES key for beneficiary ──────────────────────
    let encrypted_key = Bytes::from_slice(&t.env, &[42u8; 32]);
    t.doc_client.store_encrypted_key(
        &encrypted_doc_id,
        &beneficiary1,
        &encrypted_key,
    );

    // ── 6. Verify document metadata ─────────────────────────────────────
    let public_doc_info = t.doc_client.get_document(&public_doc_id);
    assert!(!public_doc_info.is_encrypted);
    assert!(public_doc_info.linked);
    assert_eq!(public_doc_info.vault_id, vault_id);
    assert_eq!(public_doc_info.doc_type, DocumentType::PublicManifest);

    let encrypted_doc_info = t.doc_client.get_document(&encrypted_doc_id);
    assert!(encrypted_doc_info.is_encrypted);
    assert!(encrypted_doc_info.linked);
    assert_eq!(encrypted_doc_info.vault_id, vault_id);
    assert_eq!(encrypted_doc_info.doc_type, DocumentType::Will);

    // ── 7. Simulate death: transition vault through to Triggered ────────
    t.vault_client
        .transition_status(&vault_id, &t.admin, &VaultStatus::Alert);
    t.vault_client
        .transition_status(&vault_id, &t.admin, &VaultStatus::GracePeriod);
    t.vault_client
        .transition_status(&vault_id, &t.admin, &VaultStatus::Triggered);
    assert_eq!(
        t.vault_client.get_status(&vault_id),
        VaultStatus::Triggered
    );

    // ── 8. Grant access to beneficiary for public document ──────────────
    let public_access = t
        .doc_client
        .grant_access(&public_doc_id, &beneficiary1);

    assert_eq!(public_access.doc_type, DocumentType::PublicManifest);
    assert!(!public_access.is_encrypted);
    // Public documents have no encrypted key
    assert_eq!(public_access.encrypted_key.len(), 0);
    assert_eq!(
        public_access.ipfs_cid,
        String::from_str(&t.env, "QmPublicManifestCIDForInheritanceFlow12345")
    );

    // ── 9. Grant access to beneficiary for encrypted document ───────────
    let encrypted_access = t
        .doc_client
        .grant_access(&encrypted_doc_id, &beneficiary1);

    assert_eq!(encrypted_access.doc_type, DocumentType::Will);
    assert!(encrypted_access.is_encrypted);
    // Encrypted document should have the encrypted key we stored
    assert_eq!(encrypted_access.encrypted_key.len(), 32);
    assert_eq!(
        encrypted_access.encrypted_key,
        Bytes::from_slice(&t.env, &[42u8; 32])
    );
    assert_eq!(
        encrypted_access.ipfs_cid,
        String::from_str(&t.env, "QmEncryptedWillCIDForInheritanceFlowTest")
    );

    // ── 10. Verify document proof for both documents ────────────────────
    let public_proof = t.doc_client.verify_document(&public_doc_id);
    assert!(public_proof.exists);
    assert_eq!(
        public_proof.doc_hash,
        BytesN::from_array(&t.env, &[1u8; 32])
    );

    let encrypted_proof = t.doc_client.verify_document(&encrypted_doc_id);
    assert!(encrypted_proof.exists);
    assert_eq!(
        encrypted_proof.doc_hash,
        BytesN::from_array(&t.env, &[2u8; 32])
    );
}
