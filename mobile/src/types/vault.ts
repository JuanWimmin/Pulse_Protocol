/// TypeScript types for vaults and beneficiaries, mirroring the GraphQL schema.
import { VaultStatus } from "./verification";

export interface Vault {
  id: string;
  contractId: string | null;
  owner: string;
  status: VaultStatus;
  beneficiaries: Beneficiary[];
  balance: TokenBalance[];
  escrowContract: string | null;
  createdAt: string;
}

export interface Beneficiary {
  address: string;
  percentage: number;
  claimed: boolean;
  claimedAt: string | null;
}

export interface TokenBalance {
  token: string;
  amount: string;
}

export interface TransactionResult {
  success: boolean;
  txHash: string | null;
  message: string;
}

export interface ClaimResult {
  success: boolean;
  amountReceived: string;
  txHash: string | null;
}

export interface CreateVaultInput {
  token: string;
  initialDeposit?: string;
}

export interface BeneficiaryInput {
  address: string;
  percentage: number;
}
