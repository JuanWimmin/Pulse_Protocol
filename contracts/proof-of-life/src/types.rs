use soroban_sdk::{contracttype, BytesN, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct LifeModel {
    pub weights: Vec<i128>,
    pub bias: i128,
    pub version: u32,
    pub last_updated: u64,
    pub calibration_complete: bool,
    pub total_verifications: u64,
    pub avg_confidence: u32,
    pub alert_threshold: u32,
    pub critical_threshold: u32,
    pub grace_period_days: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct VerificationRecord {
    pub timestamp: u64,
    pub liveness_score: u32,
    pub source: VerificationSource,
    pub oracle_signature: BytesN<64>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum VerificationSource {
    FacialRecognition,
    Fingerprint,
    BehaviorPattern,
    PerceptronAggregate,
    ManualCheckin,
    WitnessAttestation,
}
