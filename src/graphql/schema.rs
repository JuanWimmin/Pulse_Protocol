use async_graphql::{Context, EmptySubscription, Object, Result, Schema};
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use super::types::*;
use crate::auth::AuthenticatedUser;
use crate::config::Config;
use crate::models;
use crate::services::aggregator;
use crate::services::contracts::proof_of_life::ProofOfLifeContractClient;
use crate::services::contracts::vault::VaultContractClient;
use crate::services::publisher::Publisher;
use crate::services::soroban::SorobanClient;
use crate::services::trustless_work::TrustlessWorkOrchestrator;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Helper: convert DB beneficiaries to GQL type.
fn to_gql_beneficiaries(db: Vec<models::beneficiary::Beneficiary>) -> Vec<Beneficiary> {
    db.into_iter()
        .map(|b| Beneficiary {
            address: b.stellar_address,
            percentage: b.percentage,
            claimed: b.claimed,
            claimed_at: b.claimed_at,
        })
        .collect()
}

/// Helper: build GQL Vault from DB models.
async fn build_vault(
    pool: &PgPool,
    v: models::vault::Vault,
) -> Result<Vault> {
    let owner = models::user::User::find_by_id(pool, v.owner_id)
        .await?
        .map(|u| u.stellar_address)
        .unwrap_or_default();
    let beneficiaries = models::beneficiary::Beneficiary::find_by_vault(pool, v.id).await?;

    Ok(Vault {
        id: v.id,
        contract_id: v.contract_id,
        owner,
        status: VaultStatus::from_db(&v.status),
        beneficiaries: to_gql_beneficiaries(beneficiaries),
        balance: vec![], // Sprint 2: fetch from on-chain via wrappers
        escrow_contract: v.escrow_contract_id,
        created_at: v.created_at,
    })
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get vault by ID.
    async fn vault(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<Vault>> {
        let pool = ctx.data::<PgPool>()?;
        match models::vault::Vault::find_by_id(pool, id).await? {
            Some(v) => Ok(Some(build_vault(pool, v).await?)),
            None => Ok(None),
        }
    }

    /// Get all vaults for the authenticated user.
    async fn my_vaults(&self, ctx: &Context<'_>) -> Result<Vec<Vault>> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let vaults = models::vault::Vault::find_by_owner(pool, auth.user_id).await?;
        let mut result = Vec::new();
        for v in vaults {
            result.push(build_vault(pool, v).await?);
        }
        Ok(result)
    }

    /// Get liveness score for a user.
    async fn liveness_score(
        &self,
        ctx: &Context<'_>,
        user_id: Uuid,
    ) -> Result<Option<LivenessData>> {
        let pool = ctx.data::<PgPool>()?;
        match models::verification::Verification::latest(pool, user_id).await? {
            Some(v) => {
                let count = models::verification::Verification::count(pool, user_id).await?;
                Ok(Some(LivenessData {
                    score: v.score,
                    last_verified: v.created_at,
                    total_verifications: count as i32,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get liveness score for authenticated user.
    async fn my_liveness(&self, ctx: &Context<'_>) -> Result<Option<LivenessData>> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        match models::verification::Verification::latest(pool, auth.user_id).await? {
            Some(v) => {
                let count = models::verification::Verification::count(pool, auth.user_id).await?;
                Ok(Some(LivenessData {
                    score: v.score,
                    last_verified: v.created_at,
                    total_verifications: count as i32,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get beneficiaries for a vault.
    async fn beneficiaries(
        &self,
        ctx: &Context<'_>,
        vault_id: Uuid,
    ) -> Result<Vec<Beneficiary>> {
        let pool = ctx.data::<PgPool>()?;
        let db = models::beneficiary::Beneficiary::find_by_vault(pool, vault_id).await?;
        Ok(to_gql_beneficiaries(db))
    }

    /// Get assets for the authenticated user.
    async fn my_assets(&self, ctx: &Context<'_>) -> Result<Vec<Asset>> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let items = models::asset::Asset::find_by_owner(pool, auth.user_id).await?;
        Ok(items
            .into_iter()
            .map(|a| Asset {
                id: a.id,
                name: a.name,
                symbol: a.symbol,
                amount: a.amount,
                value_usd: a.value_usd,
                custody: a.custody,
                status: a.status,
                created_at: a.created_at,
            })
            .collect())
    }

    /// Activity feed for the authenticated user.
    async fn activity_feed(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
    ) -> Result<Vec<ActivityEvent>> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let limit = limit.unwrap_or(20).max(1) as i64;
        let items = models::activity::ActivityEvent::find_by_user(pool, auth.user_id, limit).await?;
        Ok(items
            .into_iter()
            .map(|a| ActivityEvent {
                id: a.id,
                title: a.title,
                detail: a.detail,
                kind: a.kind,
                created_at: a.created_at,
            })
            .collect())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new vault. If VAULT_CONTRACT_ID is set, also creates on-chain.
    async fn create_vault(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "input")] input: CreateVaultInput,
    ) -> Result<Vault> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let config = ctx.data::<Config>()?;
        let soroban = ctx.data::<SorobanClient>()?;

        // Create in DB first
        let vault = models::vault::Vault::create(pool, auth.user_id).await?;

        // If contracts are deployed, create on-chain too
        if let Some(ref contract_id) = config.vault_contract_id {
            let client = VaultContractClient::new(contract_id, soroban.clone());
            match client.create_vault(&auth.stellar_address, &input.token).await {
                Ok(_vault_id) => {
                    models::vault::Vault::set_contract_id(pool, vault.id, contract_id).await?;
                    info!("Vault {} created on-chain", vault.id);
                }
                Err(e) => {
                    warn!("On-chain create_vault failed (DB record kept): {}", e);
                }
            }
        }

        Ok(Vault {
            id: vault.id,
            contract_id: vault.contract_id,
            owner: auth.stellar_address.clone(),
            status: VaultStatus::from_db(&vault.status),
            beneficiaries: vec![],
            balance: vec![],
            escrow_contract: vault.escrow_contract_id,
            created_at: vault.created_at,
        })
    }

    /// Add an asset to the user's inventory.
    async fn add_asset(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "input")] input: AddAssetInput,
    ) -> Result<Asset> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;

        let status = input.status.unwrap_or_else(|| "active".to_string());
        let asset = models::asset::Asset::create(
            pool,
            auth.user_id,
            &input.name,
            &input.symbol,
            input.amount,
            input.value_usd,
            input.custody,
            &status,
        )
        .await?;

        Ok(Asset {
            id: asset.id,
            name: asset.name,
            symbol: asset.symbol,
            amount: asset.amount,
            value_usd: asset.value_usd,
            custody: asset.custody,
            status: asset.status,
            created_at: asset.created_at,
        })
    }

    /// Remove an asset owned by the user.
    async fn remove_asset(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let deleted = models::asset::Asset::delete(pool, id, auth.user_id).await?;
        Ok(deleted)
    }

    /// Deposit funds into a vault. If TW is configured and vault has an escrow,
    /// funds the escrow on-chain via backend-orquesta.
    async fn deposit(
        &self,
        ctx: &Context<'_>,
        vault_id: Uuid,
        amount: String,
        token: String,
    ) -> Result<TransactionResult> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let tw = ctx.data::<TrustlessWorkOrchestrator>()?;

        let vault = models::vault::Vault::find_by_id(pool, vault_id)
            .await?
            .ok_or("Vault not found")?;

        if vault.owner_id != auth.user_id {
            return Err("Not authorized".into());
        }

        // If TW is configured and vault has an escrow, fund it on-chain
        if tw.is_available() {
            if let Some(ref escrow_id) = vault.escrow_contract_id {
                let parsed_amount: i128 = amount
                    .parse()
                    .map_err(|_| "Invalid amount (expected integer stroops)")?;

                match tw
                    .fund_escrow(escrow_id, &auth.stellar_address, parsed_amount, &token)
                    .await
                {
                    Ok(tx_hash) => {
                        info!("Escrow funded: tx={}", tx_hash);
                        return Ok(TransactionResult {
                            success: true,
                            tx_hash: Some(tx_hash),
                            message: format!("Deposited {} {} into escrow", amount, token),
                        });
                    }
                    Err(e) => {
                        warn!("TW fund_escrow failed: {}", e);
                    }
                }
            }
        }

        // Fallback: DB-only record
        Ok(TransactionResult {
            success: true,
            tx_hash: None,
            message: format!(
                "Deposit of {} {} recorded (escrow not configured)",
                amount, token
            ),
        })
    }

    /// Set beneficiaries for a vault (replaces existing).
    async fn set_beneficiaries(
        &self,
        ctx: &Context<'_>,
        vault_id: Uuid,
        beneficiaries: Vec<BeneficiaryInput>,
    ) -> Result<Vec<Beneficiary>> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;

        let vault = models::vault::Vault::find_by_id(pool, vault_id)
            .await?
            .ok_or("Vault not found")?;

        if vault.owner_id != auth.user_id {
            return Err("Not authorized".into());
        }

        let total: i32 = beneficiaries.iter().map(|b| b.percentage).sum();
        if total != 10000 {
            return Err(format!("Percentages must sum to 10000, got {}", total).into());
        }

        let pairs: Vec<(String, i32)> = beneficiaries
            .into_iter()
            .map(|b| (b.address, b.percentage))
            .collect();

        // Sprint 1: DB only. Sprint 2 calls set_beneficiaries() on-chain.
        let db = models::beneficiary::Beneficiary::set_for_vault(pool, vault_id, &pairs).await?;
        Ok(to_gql_beneficiaries(db))
    }

    /// Submit a proof-of-life verification.
    /// Validates score, optionally publishes on-chain via oracle, persists to DB.
    async fn submit_verification(
        &self,
        ctx: &Context<'_>,
        input: VerificationInput,
    ) -> Result<VerificationResult> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let config = ctx.data::<Config>()?;
        let soroban = ctx.data::<SorobanClient>()?;

        // Validate via aggregator
        let validated = aggregator::validate_score(
            &auth.stellar_address,
            input.perceptron_output,
            &input.source,
        )
        .map_err(|e| async_graphql::Error::new(e))?;

        // Try to publish on-chain if PoL contract and oracle key are configured
        let tx_hash = if let (Some(ref pol_id), Some(ref secret)) =
            (&config.proof_of_life_contract_id, &config.oracle_secret_key)
        {
            let pol_client = ProofOfLifeContractClient::new(pol_id, soroban.clone());
            match Publisher::new(pol_client, secret) {
                Ok(publisher) => match publisher.publish_score(&validated).await {
                    Ok(hash) => {
                        info!("Score published on-chain: tx={}", hash);
                        Some(hash)
                    }
                    Err(e) => {
                        warn!("On-chain publish failed (saving to DB anyway): {}", e);
                        None
                    }
                },
                Err(e) => {
                    warn!("Publisher init failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Persist to DB
        let v = models::verification::Verification::create(
            pool,
            auth.user_id,
            input.perceptron_output,
            &input.source,
            Some(input.perceptron_output),
            tx_hash.as_deref(),
        )
        .await?;

        Ok(VerificationResult {
            score: v.score,
            tx_hash: v.on_chain_tx_hash,
            vault_status: None,
        })
    }

    /// Log a manual activity event.
    async fn log_activity(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "input")] input: ActivityInput,
    ) -> Result<ActivityEvent> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;

        let event = models::activity::ActivityEvent::create(
            pool,
            auth.user_id,
            &input.title,
            input.detail.as_deref(),
            &input.kind,
        )
        .await?;

        Ok(ActivityEvent {
            id: event.id,
            title: event.title,
            detail: event.detail,
            kind: event.kind,
            created_at: event.created_at,
        })
    }

    /// Remove an activity event.
    async fn remove_activity(&self, ctx: &Context<'_>, id: Uuid) -> Result<bool> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let deleted = models::activity::ActivityEvent::delete(pool, id, auth.user_id).await?;
        Ok(deleted)
    }

    /// Emergency check-in to reset liveness score.
    /// Optionally calls emergency_checkin() on-chain.
    async fn emergency_checkin(&self, ctx: &Context<'_>) -> Result<CheckinResult> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let config = ctx.data::<Config>()?;
        let soroban = ctx.data::<SorobanClient>()?;

        // Try on-chain emergency checkin
        let tx_hash = if let Some(ref pol_id) = config.proof_of_life_contract_id {
            let pol_client = ProofOfLifeContractClient::new(pol_id, soroban.clone());
            match pol_client.emergency_checkin(&auth.stellar_address).await {
                Ok(hash) => {
                    info!("Emergency checkin on-chain: tx={}", hash);
                    Some(hash)
                }
                Err(e) => {
                    warn!("On-chain emergency checkin failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Persist to DB with max score
        let v = models::verification::Verification::create(
            pool,
            auth.user_id,
            10000,
            "emergency",
            None,
            tx_hash.as_deref(),
        )
        .await?;

        Ok(CheckinResult {
            success: true,
            new_score: v.score,
            tx_hash: v.on_chain_tx_hash,
        })
    }

    /// Claim inheritance from a triggered vault.
    /// If TW is configured, releases funds via escrow to the beneficiary.
    async fn claim_inheritance(
        &self,
        ctx: &Context<'_>,
        vault_id: Uuid,
    ) -> Result<ClaimResult> {
        let pool = ctx.data::<PgPool>()?;
        let auth = ctx.data::<AuthenticatedUser>()?;
        let tw = ctx.data::<TrustlessWorkOrchestrator>()?;
        let soroban = ctx.data::<SorobanClient>()?;

        let vault = models::vault::Vault::find_by_id(pool, vault_id)
            .await?
            .ok_or("Vault not found")?;

        if vault.status != "triggered" {
            return Err("Vault must be in TRIGGERED status to claim".into());
        }

        let can_claim = models::beneficiary::Beneficiary::can_claim(
            pool,
            vault_id,
            &auth.stellar_address,
        )
        .await?;

        if !can_claim {
            return Err("Not a valid beneficiary or already claimed".into());
        }

        // Record claim in DB
        let b = models::beneficiary::Beneficiary::record_claim(
            pool,
            vault_id,
            &auth.stellar_address,
        )
        .await?;

        // Release funds via TW escrow if configured
        let tx_hash = if tw.is_available() {
            if let (Some(ref escrow_id), Some(oracle_addr)) =
                (&vault.escrow_contract_id, soroban.oracle_address())
            {
                match tw
                    .release_to_beneficiary(escrow_id, &oracle_addr, &auth.stellar_address)
                    .await
                {
                    Ok(hash) => {
                        info!("TW release to {}: tx={}", auth.stellar_address, hash);
                        Some(hash)
                    }
                    Err(e) => {
                        warn!("TW release failed: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(ClaimResult {
            success: true,
            amount_received: format!("{}%", b.percentage as f64 / 100.0),
            tx_hash,
        })
    }

    /// Force vault state transition (admin/demo).
    /// Optionally calls transition_status() on-chain.
    /// When transitioning to TRIGGERED, approves TW escrow milestones.
    async fn force_transition(
        &self,
        ctx: &Context<'_>,
        vault_id: Uuid,
        new_status: VaultStatus,
    ) -> Result<Vault> {
        let pool = ctx.data::<PgPool>()?;
        let config = ctx.data::<Config>()?;
        let soroban = ctx.data::<SorobanClient>()?;
        let tw = ctx.data::<TrustlessWorkOrchestrator>()?;

        // Try on-chain transition if vault contract is configured
        if let Some(ref contract_id) = config.vault_contract_id {
            if let Some(addr) = soroban.oracle_address() {
                let client = VaultContractClient::new(contract_id, soroban.clone());
                let status_u32 = match new_status {
                    VaultStatus::Active => 0,
                    VaultStatus::Alert => 1,
                    VaultStatus::GracePeriod => 2,
                    VaultStatus::Triggered => 3,
                    VaultStatus::Distributed => 4,
                };
                match client.transition_status(0, &addr, status_u32).await {
                    Ok(hash) => info!("On-chain transition: tx={}", hash),
                    Err(e) => warn!("On-chain transition failed: {}", e),
                }
            }
        }

        // When transitioning to TRIGGERED, approve TW escrow milestones
        if matches!(new_status, VaultStatus::Triggered) && tw.is_available() {
            let vault = models::vault::Vault::find_by_id(pool, vault_id)
                .await?
                .ok_or("Vault not found")?;

            if let (Some(ref escrow_id), Some(oracle_addr)) =
                (&vault.escrow_contract_id, soroban.oracle_address())
            {
                match tw.approve_milestones(escrow_id, &oracle_addr).await {
                    Ok(hash) => info!("TW milestones approved: tx={}", hash),
                    Err(e) => warn!("TW approve_milestones failed: {}", e),
                }
            }
        }

        // Update DB
        let vault =
            models::vault::Vault::update_status(pool, vault_id, new_status.as_str()).await?;

        build_vault(pool, vault).await
    }
}
