/// Feature extractor for the Perceptron MVP.
///
/// Produces a 10-dimensional feature vector:
///   - 2 real features: fingerprint success + days since last verification
///   - 8 mock features with slight random variation (+/- 0.05)
///     so the score isn't always identical across runs.
///
/// Feature order must match the weight order in model.ts.

const FEATURE_COUNT = 10;
const MOCK_VARIATION = 0.05;

/// Mock feature defaults — reasonable values for an "alive" user.
const MOCK_DEFAULTS = {
  faceMatchScore: 0.85,
  faceLivenessScore: 0.80,
  fingerprintConsistency: 0.70,
  timeOfDayNormality: 0.75,
  typingPatternMatch: 0.50,
  appUsageMatch: 0.50,
  movementPatternMatch: 0.50,
  sessionBehavior: 0.60,
};

export interface FeatureContext {
  /// Whether the fingerprint authentication succeeded (real feature).
  fingerprintSuccess: boolean;
  /// Timestamp (ms) of the last successful verification, or null if never verified.
  lastVerificationTimestamp: number | null;
}

/// Add slight random variation to a mock value, clamped to [0, 1].
function vary(base: number): number {
  const offset = (Math.random() * 2 - 1) * MOCK_VARIATION;
  return Math.max(0, Math.min(1, base + offset));
}

/// Normalize "days since last verification" to [0, 1].
/// 0 days → 1.0 (just verified), 30+ days → ~0.0 (long time).
function normalizeDaysSinceVerify(
  lastTimestamp: number | null,
  now: number = Date.now()
): number {
  if (lastTimestamp === null) {
    return 0.1; // Never verified — low but not zero
  }
  const daysSince = (now - lastTimestamp) / (1000 * 60 * 60 * 24);
  // Exponential decay: score halves every 7 days
  return Math.max(0, Math.min(1, Math.exp(-daysSince / 7)));
}

/// Extract a 10-dimensional feature vector for the perceptron.
///
/// Real features:
///   - x3 (fingerprint_frequency): 0.9 if auth succeeded, 0.1 if not
///   - x9 (days_since_last_verify): exponential decay from last verification
///
/// Mock features (x1, x2, x4-x8, x10): hardcoded defaults with +/- 0.05 variation.
export function extractFeatures(context: FeatureContext): number[] {
  const features: number[] = [
    vary(MOCK_DEFAULTS.faceMatchScore),           // x1: face_match_score (mock)
    vary(MOCK_DEFAULTS.faceLivenessScore),         // x2: face_liveness_score (mock)
    context.fingerprintSuccess ? 0.9 : 0.1,       // x3: fingerprint_frequency (REAL)
    vary(MOCK_DEFAULTS.fingerprintConsistency),    // x4: fingerprint_consistency (mock)
    vary(MOCK_DEFAULTS.timeOfDayNormality),        // x5: time_of_day_normality (mock)
    vary(MOCK_DEFAULTS.typingPatternMatch),        // x6: typing_pattern_match (mock)
    vary(MOCK_DEFAULTS.appUsageMatch),             // x7: app_usage_match (mock)
    vary(MOCK_DEFAULTS.movementPatternMatch),      // x8: movement_pattern_match (mock)
    normalizeDaysSinceVerify(                      // x9: days_since_last_verify (REAL)
      context.lastVerificationTimestamp
    ),
    vary(MOCK_DEFAULTS.sessionBehavior),           // x10: session_behavior (mock)
  ];

  if (features.length !== FEATURE_COUNT) {
    throw new Error(`Feature vector has ${features.length} elements, expected ${FEATURE_COUNT}`);
  }

  return features;
}
