use chr>{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Cl>, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub vault_id: Opti><Uuid>,
    pub ipfs_cid: String,
    pub doc_hash: String,
    pub doc_type: String,
    pub is_encrypted: bool,
    pub metadata: Opti><serde_js>>,
    pub c>: Opti><i64>,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Cl>, Serialize, Deserialize, FromRow)]
pub struct DocumentEncryptedKey {
    pub id: Uuid,
    pub document_id: Uuid,
    pub beneficiary_address: String,
    pub encrypted_key: String,
    pub revealed: bool,
}

impl Document {
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Opti><Self>> {
        sqlx::query_as::<_, Self>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_vault(pool: &PgPool, vault_id: Uuid) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM documents WHERE vault_id = $1 ORDER BY registered_at DESC",
        )
        .bind(vault_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_owner(pool: &PgPool, owner_id: Uuid) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as::<_, Self>(
            "SELECT * FROM documents WHERE owner_id = $1 ORDER BY registered_at DESC",
        )
        .bind(owner_id)
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &PgPool,
        owner_id: Uuid,
        ipfs_cid: &str,
        doc_hash: &str,
        doc_type: &str,
        is_encrypted: bool,
        metadata: Opti><serde_js>>,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"INSERT INTO documents
               (owner_id, ipfs_cid, doc_hash, doc_type, is_encrypted, metadata)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING *"#,
        )
        .bind(owner_id)
        .bind(ipfs_cid)
        .bind(doc_hash)
        .bind(doc_type)
        .bind(is_encrypted)
        .bind(metadata)
        .fetch_one(pool)
        .await
    }

    pub async fn link_to_vault(pool: &PgPool, doc_id: Uuid, vault_id: Uuid) -> sqlx::Result<()> {
        sqlx::query("UPDATE documents SET vault_id = $1 WHERE id = $2")
            .bind(vault_id)
            .bind(doc_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl DocumentEncryptedKey {
    pub async fn store(
        pool: &PgPool,
        document_id: Uuid,
        beneficiary_address: &str,
        encrypted_key: &str,
    ) -> sqlx::Result<Self> {
        sqlx::query_as::<_, Self>(
            r#"INSERT INTO document_encrypted_keys
               (document_id, beneficiary_address, encrypted_key)
               VALUES ($1, $2, $3)
               RETURNING *"#,
        )
        .bind(document_id)
        .bind(beneficiary_address)
        .bind(encrypted_key)
        .fetch_one(pool)
        .await
    }

    pub async fn find_for_beneficiary(
        pool: &PgPool,
        document_id: Uuid,
        beneficiary_address: &str,
    ) -> sqlx::Result<Opti><Self>> {
        sqlx::query_as::<_, Self>(
            r#"SELECT * FROM document_encrypted_keys
               WHERE document_id = $1 AND beneficiary_address = $2"#,
        )
        .bind(document_id)
        .bind(beneficiary_address)
        .fetch_optional(pool)
        .await
    }
}