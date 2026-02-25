import { extractFeatures } from "../features";

describe("extractFeatures", () => {
  it("returns exactly 10 features", () => {
    const features = extractFeatures({
      fingerprintSuccess: true,
      lastVerificationTimestamp: Date.now(),
    });
    expect(features).toHaveLength(10);
  });

  it("all features are in [0, 1]", () => {
    const features = extractFeatures({
      fingerprintSuccess: true,
      lastVerificationTimestamp: Date.now() - 3 * 24 * 60 * 60 * 1000, // 3 days ago
    });
    for (const f of features) {
      expect(f).toBeGreaterThanOrEqual(0);
      expect(f).toBeLessThanOrEqual(1);
    }
  });

  it("fingerprint feature (x3) is high when auth succeeds", () => {
    const features = extractFeatures({
      fingerprintSuccess: true,
      lastVerificationTimestamp: null,
    });
    expect(features[2]).toBe(0.9);
  });

  it("fingerprint feature (x3) is low when auth fails", () => {
    const features = extractFeatures({
      fingerprintSuccess: false,
      lastVerificationTimestamp: null,
    });
    expect(features[2]).toBe(0.1);
  });

  it("days_since_last_verify (x9) is high for recent verification", () => {
    const features = extractFeatures({
      fingerprintSuccess: true,
      lastVerificationTimestamp: Date.now() - 1000, // 1 second ago
    });
    expect(features[8]).toBeGreaterThan(0.9);
  });

  it("days_since_last_verify (x9) is low for old verification", () => {
    const features = extractFeatures({
      fingerprintSuccess: true,
      lastVerificationTimestamp: Date.now() - 60 * 24 * 60 * 60 * 1000, // 60 days ago
    });
    expect(features[8]).toBeLessThan(0.01);
  });

  it("days_since_last_verify (x9) is low when never verified", () => {
    const features = extractFeatures({
      fingerprintSuccess: true,
      lastVerificationTimestamp: null,
    });
    expect(features[8]).toBe(0.1);
  });

  it("mock features vary slightly between calls", () => {
    const f1 = extractFeatures({
      fingerprintSuccess: true,
      lastVerificationTimestamp: Date.now(),
    });
    const f2 = extractFeatures({
      fingerprintSuccess: true,
      lastVerificationTimestamp: Date.now(),
    });

    // x1 (face_match_score) is mock — should be close to 0.85 but not always identical
    expect(f1[0]).toBeGreaterThan(0.75);
    expect(f1[0]).toBeLessThan(0.95);
    // At least one mock feature should differ (probabilistically near-certain)
    const allSame = f1.every((v, i) => v === f2[i]);
    // This is technically flaky but probability of all 8 randoms matching is ~0
    expect(allSame).toBe(false);
  });
});
