/// GraphQL queries for the Pulse Protocol MVP.
import { gql } from "@apollo/client";

export const VAULT_QUERY = gql`
  query Vault($id: ID!) {
    vault(id: $id) {
      id
      contractId
      owner
      status
      beneficiaries {
        address
        percentage
        claimed
        claimedAt
      }
      balance {
        token
        amount
      }
      escrowContract
      createdAt
    }
  }
`;

export const MY_VAULTS_QUERY = gql`
  query MyVaults {
    myVaults {
      id
      contractId
      owner
      status
      beneficiaries {
        address
        percentage
        claimed
        claimedAt
      }
      balance {
        token
        amount
      }
      escrowContract
      createdAt
    }
  }
`;

export const LIVENESS_SCORE_QUERY = gql`
  query LivenessScore($userId: ID!) {
    livenessScore(userId: $userId) {
      score
      lastVerified
      totalVerifications
    }
  }
`;

export const MY_LIVENESS_QUERY = gql`
  query MyLiveness {
    myLiveness {
      score
      lastVerified
      totalVerifications
    }
  }
`;

export const BENEFICIARIES_QUERY = gql`
  query Beneficiaries($vaultId: ID!) {
    beneficiaries(vaultId: $vaultId) {
      address
      percentage
      claimed
      claimedAt
    }
  }
`;
