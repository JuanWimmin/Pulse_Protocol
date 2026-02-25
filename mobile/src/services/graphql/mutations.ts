/// GraphQL mutations for the verification flow (F2 scope).
/// These are ready to be used with Apollo Client once F1 sets up the client.

/// Submit a proof-of-life verification score to the backend oracle.
export const SUBMIT_VERIFICATION = `
  mutation SubmitVerification($input: VerificationInput!) {
    submitVerification(input: $input) {
      score
      txHash
      timestamp
    }
  }
`;

/// Emergency check-in — resets liveness score to 10000 and vault to ACTIVE.
/// Only available when vault is in ALERT or GRACE_PERIOD.
export const EMERGENCY_CHECKIN = `
  mutation EmergencyCheckin {
    emergencyCheckin {
      score
      txHash
      vaultStatus
    }
  }
`;
