/// TypeScript types for the verification flow, mirroring the GraphQL schema.

export interface VerificationInput {
  perceptronOutput: number; // 0-10000
  source: string;
}

export interface VerificationResultGql {
  score: number;
  txHash: string | null;
  timestamp: string;
}

export interface CheckinResultGql {
  score: number;
  txHash: string | null;
  vaultStatus: VaultStatus;
}

export enum VaultStatus {
  ACTIVE = "ACTIVE",
  ALERT = "ALERT",
  GRACE_PERIOD = "GRACE_PERIOD",
  TRIGGERED = "TRIGGERED",
  DISTRIBUTED = "DISTRIBUTED",
}
