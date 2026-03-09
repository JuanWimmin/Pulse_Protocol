/// GraphQL mutations for the Pulse Protocol MVP.
import { gql } from "@apollo/client";

export const CREATE_VAULT = gql`
  mutation CreateVault($input: CreateVaultInput!) {
    createVault(input: $input) {
      id
      contractId
      owner
      status
      escrowContract
      createdAt
    }
  }
`;

export const DEPOSIT = gql`
  mutation Deposit($vaultId: ID!, $amount: String!, $token: String!) {
    deposit(vaultId: $vaultId, amount: $amount, token: $token) {
      success
      txHash
      message
    }
  }
`;

export const SET_BENEFICIARIES = gql`
  mutation SetBeneficiaries($vaultId: ID!, $beneficiaries: [BeneficiaryInput!]!) {
    setBeneficiaries(vaultId: $vaultId, beneficiaries: $beneficiaries) {
      address
      percentage
      claimed
      claimedAt
    }
  }
`;

export const SUBMIT_VERIFICATION = gql`
  mutation SubmitVerification($input: VerificationInput!) {
    submitVerification(input: $input) {
      score
      txHash
      vaultStatus
    }
  }
`;

export const EMERGENCY_CHECKIN = gql`
  mutation EmergencyCheckin {
    emergencyCheckin {
      success
      newScore
      txHash
    }
  }
`;

export const CLAIM_INHERITANCE = gql`
  mutation ClaimInheritance($vaultId: ID!) {
    claimInheritance(vaultId: $vaultId) {
      success
      amountReceived
      txHash
    }
  }
`;

export const FORCE_TRANSITION = gql`
  mutation ForceTransition($vaultId: ID!, $newStatus: VaultStatus!) {
    forceTransition(vaultId: $vaultId, newStatus: $newStatus) {
      id
      status
    }
  }
`;
