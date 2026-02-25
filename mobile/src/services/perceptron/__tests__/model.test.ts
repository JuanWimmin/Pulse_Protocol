import { Perceptron } from "../model";

describe("Perceptron", () => {
  const perceptron = new Perceptron();

  it("produces a high score (~0.85) with normal/active features", () => {
    // All features high — simulates an active, alive user
    const activeFeatures = [
      0.85, // face_match_score
      0.80, // face_liveness_score
      0.90, // fingerprint_frequency (real — just authenticated)
      0.70, // fingerprint_consistency
      0.75, // time_of_day_normality
      0.50, // typing_pattern_match
      0.50, // app_usage_match
      0.50, // movement_pattern_match
      0.90, // days_since_last_verify (normalized — recent)
      0.60, // session_behavior
    ];

    const score = perceptron.predict(activeFeatures);
    expect(score).toBeGreaterThan(0.7);
    expect(score).toBeLessThan(1.0);

    const onChain = Perceptron.toOnChainScore(score);
    expect(onChain).toBeGreaterThanOrEqual(7000);
    expect(onChain).toBeLessThanOrEqual(10000);
  });

  it("produces a low score (~0.15) with inactivity features", () => {
    // All features low — simulates an inactive/dead user
    const inactiveFeatures = [
      0.10, // face_match_score
      0.10, // face_liveness_score
      0.05, // fingerprint_frequency
      0.10, // fingerprint_consistency
      0.10, // time_of_day_normality
      0.05, // typing_pattern_match
      0.05, // app_usage_match
      0.05, // movement_pattern_match
      0.05, // days_since_last_verify (long time ago)
      0.05, // session_behavior
    ];

    const score = perceptron.predict(inactiveFeatures);
    expect(score).toBeGreaterThan(0.0);
    expect(score).toBeLessThan(0.4);

    const onChain = Perceptron.toOnChainScore(score);
    expect(onChain).toBeGreaterThanOrEqual(0);
    expect(onChain).toBeLessThanOrEqual(4000);
  });

  it("returns a score strictly between 0 and 1 (sigmoid bounds)", () => {
    const midFeatures = Array(10).fill(0.5);
    const score = perceptron.predict(midFeatures);
    expect(score).toBeGreaterThan(0);
    expect(score).toBeLessThan(1);
  });

  it("throws if feature vector has wrong length", () => {
    expect(() => perceptron.predict([0.5, 0.5])).toThrow(
      "Expected 10 features"
    );
  });

  it("throws if weights have wrong length", () => {
    expect(() => new Perceptron([0.5, 0.5])).toThrow("Expected 10 weights");
  });

  it("converts prediction to on-chain score correctly", () => {
    expect(Perceptron.toOnChainScore(0.0)).toBe(0);
    expect(Perceptron.toOnChainScore(1.0)).toBe(10000);
    expect(Perceptron.toOnChainScore(0.8523)).toBe(8523);
    expect(Perceptron.toOnChainScore(0.5)).toBe(5000);
  });
});
