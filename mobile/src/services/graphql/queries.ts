/// GraphQL queries relevant to the verification flow (F2 scope).
/// These are ready to be used with Apollo Client once F1 sets up the client.

/// Fetch liveness score for the current user.
export const LIVENESS_SCORE_QUERY = `
  query LivenessScore($userId: ID!) {
    livenessScore(userId: $userId) {
      score
      lastVerified
      totalVerifications
    }
  }
`;
