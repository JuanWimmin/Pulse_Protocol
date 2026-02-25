/// VerifyScreen — Proof-of-life verification screen.
///
/// Flow:
///   1. User taps "Verify Life"
///   2. BiometricPrompt requests fingerprint
///   3. Processing animation while perceptron runs
///   4. Score sent to backend via SUBMIT_VERIFICATION mutation
///   5. Shows result: score + tx hash + vault status
///
/// Ready to be registered in MainNavigator by F1.

import React, { useState, useCallback } from "react";
import {
  View,
  Text,
  TouchableOpacity,
  ActivityIndicator,
  StyleSheet,
  Alert,
} from "react-native";
import { runVerification, VerificationResult } from "../services/verification";

// TODO(F1): Replace with Apollo Client mutation hook once client is set up
// import { useMutation } from "@apollo/client";
// import { SUBMIT_VERIFICATION } from "../services/graphql/mutations";

type VerifyScreenProps = {
  /// Timestamp of the last successful verification (from store/backend).
  lastVerificationTimestamp: number | null;
  /// Callback after successful verification submission.
  onVerificationComplete?: (score: number, txHash: string | null) => void;
};

type ScreenState = "idle" | "authenticating" | "submitting" | "done" | "error";

export default function VerifyScreen({
  lastVerificationTimestamp,
  onVerificationComplete,
}: VerifyScreenProps) {
  const [state, setState] = useState<ScreenState>("idle");
  const [result, setResult] = useState<VerificationResult | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const handleVerify = useCallback(async () => {
    setState("authenticating");
    setErrorMessage(null);

    try {
      // Step 1-4: Biometric + Perceptron (local)
      const verificationResult = await runVerification(lastVerificationTimestamp);
      setResult(verificationResult);

      if (!verificationResult.success) {
        setState("error");
        setErrorMessage(verificationResult.error ?? "Verification failed");
        return;
      }

      // Step 5: Submit to backend
      setState("submitting");

      // TODO(F1): Replace with actual Apollo mutation call
      // const { data } = await submitVerification({
      //   variables: {
      //     input: {
      //       perceptronOutput: verificationResult.score,
      //       source: verificationResult.source,
      //     },
      //   },
      // });
      // setTxHash(data.submitVerification.txHash);

      // Placeholder: simulate backend response
      const mockTxHash = "pending_backend_integration";
      setTxHash(mockTxHash);

      setState("done");
      onVerificationComplete?.(verificationResult.score, mockTxHash);
    } catch (err: unknown) {
      setState("error");
      const message = err instanceof Error ? err.message : "Unknown error";
      setErrorMessage(message);
    }
  }, [lastVerificationTimestamp, onVerificationComplete]);

  const scorePercentage = result ? (result.score / 100).toFixed(2) : "0.00";
  const scoreColor =
    result && result.score >= 7000
      ? "#4CAF50"
      : result && result.score >= 3000
        ? "#FF9800"
        : "#F44336";

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Proof of Life Verification</Text>
      <Text style={styles.subtitle}>
        Verify you are alive by authenticating with your fingerprint
      </Text>

      {state === "idle" && (
        <TouchableOpacity style={styles.verifyButton} onPress={handleVerify}>
          <Text style={styles.verifyButtonText}>Verify Life</Text>
        </TouchableOpacity>
      )}

      {state === "authenticating" && (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#6200EE" />
          <Text style={styles.loadingText}>Authenticating...</Text>
        </View>
      )}

      {state === "submitting" && (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#6200EE" />
          <Text style={styles.loadingText}>
            Submitting verification to oracle...
          </Text>
        </View>
      )}

      {state === "done" && result && (
        <View style={styles.resultContainer}>
          <Text style={styles.resultTitle}>Verification Complete</Text>
          <Text style={[styles.scoreText, { color: scoreColor }]}>
            {scorePercentage}%
          </Text>
          <Text style={styles.scoreLabel}>
            Liveness Score ({result.score} / 10000)
          </Text>
          {txHash && (
            <View style={styles.txContainer}>
              <Text style={styles.txLabel}>Transaction Hash:</Text>
              <Text style={styles.txHash} numberOfLines={1}>
                {txHash}
              </Text>
            </View>
          )}
          <TouchableOpacity
            style={[styles.verifyButton, styles.retryButton]}
            onPress={() => {
              setState("idle");
              setResult(null);
              setTxHash(null);
            }}
          >
            <Text style={styles.verifyButtonText}>Verify Again</Text>
          </TouchableOpacity>
        </View>
      )}

      {state === "error" && (
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>{errorMessage}</Text>
          <TouchableOpacity
            style={[styles.verifyButton, styles.retryButton]}
            onPress={() => {
              setState("idle");
              setErrorMessage(null);
            }}
          >
            <Text style={styles.verifyButtonText}>Try Again</Text>
          </TouchableOpacity>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    padding: 24,
    backgroundColor: "#FAFAFA",
    alignItems: "center",
    justifyContent: "center",
  },
  title: {
    fontSize: 24,
    fontWeight: "bold",
    color: "#212121",
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: "#757575",
    textAlign: "center",
    marginBottom: 40,
  },
  verifyButton: {
    backgroundColor: "#6200EE",
    paddingHorizontal: 48,
    paddingVertical: 16,
    borderRadius: 28,
    elevation: 2,
  },
  verifyButtonText: {
    color: "#FFFFFF",
    fontSize: 18,
    fontWeight: "600",
  },
  retryButton: {
    marginTop: 24,
    backgroundColor: "#757575",
  },
  loadingContainer: {
    alignItems: "center",
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: "#757575",
  },
  resultContainer: {
    alignItems: "center",
  },
  resultTitle: {
    fontSize: 20,
    fontWeight: "600",
    color: "#212121",
    marginBottom: 16,
  },
  scoreText: {
    fontSize: 48,
    fontWeight: "bold",
  },
  scoreLabel: {
    fontSize: 14,
    color: "#757575",
    marginTop: 4,
  },
  txContainer: {
    marginTop: 24,
    padding: 16,
    backgroundColor: "#FFFFFF",
    borderRadius: 8,
    width: "100%",
    elevation: 1,
  },
  txLabel: {
    fontSize: 12,
    color: "#757575",
    marginBottom: 4,
  },
  txHash: {
    fontSize: 12,
    fontFamily: "monospace",
    color: "#212121",
  },
  errorContainer: {
    alignItems: "center",
  },
  errorText: {
    fontSize: 16,
    color: "#F44336",
    textAlign: "center",
    marginBottom: 16,
  },
});
