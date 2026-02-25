/// Verification flow orchestrator for proof-of-life.
///
/// Coordinates the full verification pipeline:
///   1. Request fingerprint via BiometricPrompt
///   2. Extract features (real + mock)
///   3. Run perceptron locally → raw score
///   4. Convert to on-chain score [0, 10000]
///   5. Return result ready for backend submission
///
/// The backend receives the score via the `submitVerification` GraphQL mutation.

import {
  authenticateWithBiometric,
  checkBiometricAvailability,
} from "./biometrics/fingerprint";
import { extractFeatures, FeatureContext } from "./perceptron/features";
import { Perceptron } from "./perceptron/model";

export interface VerificationResult {
  success: boolean;
  /// On-chain score [0, 10000] representing 0.00% - 100.00%
  score: number;
  /// Raw perceptron output [0, 1]
  rawScore: number;
  /// Feature vector used for the prediction
  features: number[];
  /// Source identifier for the backend
  source: string;
  /// Error message if verification failed
  error?: string;
}

const perceptron = new Perceptron();

/// Run the full proof-of-life verification flow.
///
/// @param lastVerificationTimestamp - timestamp (ms) of the last successful
///        verification, or null if never verified. Used for feature extraction.
export async function runVerification(
  lastVerificationTimestamp: number | null
): Promise<VerificationResult> {
  // Step 1: Check biometric availability
  const availability = await checkBiometricAvailability();
  if (!availability.available) {
    return {
      success: false,
      score: 0,
      rawScore: 0,
      features: [],
      source: "biometric_unavailable",
      error: "Biometric authentication is not available on this device",
    };
  }

  // Step 2: Request fingerprint authentication
  const authResult = await authenticateWithBiometric(
    "Pulse Protocol - Verify you are alive"
  );

  if (!authResult.success) {
    return {
      success: false,
      score: 0,
      rawScore: 0,
      features: [],
      source: "biometric_failed",
      error: authResult.error ?? "Biometric authentication was cancelled or failed",
    };
  }

  // Step 3: Extract features
  const context: FeatureContext = {
    fingerprintSuccess: true,
    lastVerificationTimestamp,
  };
  const features = extractFeatures(context);

  // Step 4: Run perceptron
  const rawScore = perceptron.predict(features);
  const score = Perceptron.toOnChainScore(rawScore);

  // Step 5: Return result
  return {
    success: true,
    score,
    rawScore,
    features,
    source: "mobile_biometric",
  };
}

/// Run an emergency check-in verification.
/// Same as regular verification but the backend treats it differently
/// (resets score to 10000 and vault to ACTIVE).
export async function runEmergencyVerification(
  lastVerificationTimestamp: number | null
): Promise<VerificationResult> {
  const result = await runVerification(lastVerificationTimestamp);
  if (result.success) {
    result.source = "emergency_checkin";
  }
  return result;
}
