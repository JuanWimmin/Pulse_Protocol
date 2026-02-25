/// Perceptron MVP for Pulse Protocol proof-of-life verification.
///
/// Hardcoded weights produce:
///   - score ~0.85 with normal/active features
///   - score ~0.15 with inactivity features
///
/// Scores are returned as float [0, 1] and converted to u32 [0, 10000]
/// before sending to the backend (0.00% - 100.00%).

const FEATURE_COUNT = 10;

/// Default weights tuned for demo — scaled so the sigmoid produces:
///   ~0.85 with typical active features, ~0.05 with inactivity features.
/// Proportional importance preserved: face > fingerprint > time > patterns.
const DEFAULT_WEIGHTS: readonly number[] = [
  1.75, // x1: face_match_score
  1.40, // x2: face_liveness_score
  0.70, // x3: fingerprint_frequency
  0.70, // x4: fingerprint_consistency
  0.56, // x5: time_of_day_normality
  0.49, // x6: typing_pattern_match
  0.35, // x7: app_usage_match
  0.35, // x8: movement_pattern_match
  0.35, // x9: days_since_last_verify (normalized)
  0.35, // x10: session_behavior
];

const DEFAULT_BIAS = -3.5;

export class Perceptron {
  private readonly weights: readonly number[];
  private readonly bias: number;

  constructor(
    weights: readonly number[] = DEFAULT_WEIGHTS,
    bias: number = DEFAULT_BIAS
  ) {
    if (weights.length !== FEATURE_COUNT) {
      throw new Error(
        `Expected ${FEATURE_COUNT} weights, got ${weights.length}`
      );
    }
    this.weights = weights;
    this.bias = bias;
  }

  /// Run prediction on a feature vector. Returns a float in [0, 1].
  predict(features: number[]): number {
    if (features.length !== FEATURE_COUNT) {
      throw new Error(
        `Expected ${FEATURE_COUNT} features, got ${features.length}`
      );
    }

    const z =
      this.weights.reduce((sum, w, i) => sum + w * features[i], 0) + this.bias;
    return this.sigmoid(z);
  }

  /// Convert a raw prediction [0,1] to a score in [0, 10000] for on-chain storage.
  static toOnChainScore(prediction: number): number {
    return Math.round(prediction * 10000);
  }

  private sigmoid(z: number): number {
    return 1 / (1 + Math.exp(-z));
  }
}
