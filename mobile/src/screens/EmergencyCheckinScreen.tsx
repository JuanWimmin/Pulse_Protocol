/// EmergencyCheckinScreen — Emergency proof-of-life check-in.
///
/// Allows the user to reset their vault back to ACTIVE when it's in
/// ALERT or GRACE_PERIOD. Requires biometric authentication.
///
/// Flow:
///   1. User taps "Emergency Check-In"
///   2. BiometricPrompt requests fingerprint
///   3. Verification runs locally (same perceptron flow)
///   4. Calls EMERGENCY_CHECKIN mutation on backend
///   5. Backend resets score to 10000 and vault status to ACTIVE
///   6. Shows confirmation with new status
///
/// Ready to be registered in MainNavigator by F1.

import React, { useState, useCallback } from "react";
import {
  View,
  Text,
  TouchableOpacity,
  ActivityIndicator,
  StyleSheet,
} from "react-native";
import { runEmergencyVerification } from "../services/verification";
import { VaultStatus } from "../types/verification";

// TODO(F1): Replace with Apollo Client mutation hook
// import { useMutation } from "@apollo/client";
// import { EMERGENCY_CHECKIN } from "../services/graphql/mutations";

type EmergencyCheckinScreenProps = {
  /// Current vault status — screen is only useful when ALERT or GRACE_PERIOD.
  currentVaultStatus: VaultStatus;
  /// Timestamp of the last successful verification.
  lastVerificationTimestamp: number | null;
  /// Callback after successful emergency check-in.
  onCheckinComplete?: (newStatus: VaultStatus) => void;
  /// Navigation back.
  onGoBack?: () => void;
};

type ScreenState = "idle" | "authenticating" | "submitting" | "done" | "error";

export default function EmergencyCheckinScreen({
  currentVaultStatus,
  lastVerificationTimestamp,
  onCheckinComplete,
  onGoBack,
}: EmergencyCheckinScreenProps) {
  const [state, setState] = useState<ScreenState>("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const isEligible =
    currentVaultStatus === VaultStatus.ALERT ||
    currentVaultStatus === VaultStatus.GRACE_PERIOD;

  const handleEmergencyCheckin = useCallback(async () => {
    if (!isEligible) return;

    setState("authenticating");
    setErrorMessage(null);

    try {
      // Step 1-4: Biometric + Perceptron (local) — tagged as emergency
      const result = await runEmergencyVerification(lastVerificationTimestamp);

      if (!result.success) {
        setState("error");
        setErrorMessage(result.error ?? "Biometric verification failed");
        return;
      }

      // Step 5: Submit emergency check-in to backend
      setState("submitting");

      // TODO(F1): Replace with actual Apollo mutation call
      // const { data } = await emergencyCheckin();
      // const newStatus = data.emergencyCheckin.vaultStatus;

      // Placeholder: simulate backend response
      const newStatus = VaultStatus.ACTIVE;

      setState("done");
      onCheckinComplete?.(newStatus);
    } catch (err: unknown) {
      setState("error");
      const message = err instanceof Error ? err.message : "Unknown error";
      setErrorMessage(message);
    }
  }, [isEligible, lastVerificationTimestamp, onCheckinComplete]);

  const statusLabel: Record<VaultStatus, string> = {
    [VaultStatus.ACTIVE]: "Active",
    [VaultStatus.ALERT]: "Alert",
    [VaultStatus.GRACE_PERIOD]: "Grace Period",
    [VaultStatus.TRIGGERED]: "Triggered",
    [VaultStatus.DISTRIBUTED]: "Distributed",
  };

  const statusColor: Record<VaultStatus, string> = {
    [VaultStatus.ACTIVE]: "#4CAF50",
    [VaultStatus.ALERT]: "#FF9800",
    [VaultStatus.GRACE_PERIOD]: "#F44336",
    [VaultStatus.TRIGGERED]: "#9E9E9E",
    [VaultStatus.DISTRIBUTED]: "#9E9E9E",
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Emergency Check-In</Text>

      <View style={styles.statusCard}>
        <Text style={styles.statusLabel}>Current Vault Status</Text>
        <Text
          style={[
            styles.statusValue,
            { color: statusColor[currentVaultStatus] },
          ]}
        >
          {statusLabel[currentVaultStatus]}
        </Text>
      </View>

      {!isEligible && (
        <View style={styles.infoContainer}>
          <Text style={styles.infoText}>
            Emergency check-in is only available when your vault is in Alert or
            Grace Period status.
          </Text>
          {onGoBack && (
            <TouchableOpacity style={styles.backButton} onPress={onGoBack}>
              <Text style={styles.backButtonText}>Go Back</Text>
            </TouchableOpacity>
          )}
        </View>
      )}

      {isEligible && state === "idle" && (
        <View style={styles.actionContainer}>
          <Text style={styles.warningText}>
            Authenticate with your fingerprint to prove you are alive. Your
            vault status will be reset to Active and your liveness score will be
            restored to 100%.
          </Text>
          <TouchableOpacity
            style={styles.emergencyButton}
            onPress={handleEmergencyCheckin}
          >
            <Text style={styles.emergencyButtonText}>Emergency Check-In</Text>
          </TouchableOpacity>
        </View>
      )}

      {(state === "authenticating" || state === "submitting") && (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#F44336" />
          <Text style={styles.loadingText}>
            {state === "authenticating"
              ? "Verifying identity..."
              : "Submitting emergency check-in..."}
          </Text>
        </View>
      )}

      {state === "done" && (
        <View style={styles.successContainer}>
          <Text style={styles.successTitle}>Check-In Successful</Text>
          <Text style={styles.successScore}>Score: 10000 (100.00%)</Text>
          <Text style={styles.successStatus}>
            Vault Status: Active
          </Text>
          {onGoBack && (
            <TouchableOpacity style={styles.backButton} onPress={onGoBack}>
              <Text style={styles.backButtonText}>Return to Dashboard</Text>
            </TouchableOpacity>
          )}
        </View>
      )}

      {state === "error" && (
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>{errorMessage}</Text>
          <TouchableOpacity
            style={styles.retryButton}
            onPress={() => {
              setState("idle");
              setErrorMessage(null);
            }}
          >
            <Text style={styles.retryButtonText}>Try Again</Text>
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
    marginBottom: 24,
  },
  statusCard: {
    backgroundColor: "#FFFFFF",
    padding: 20,
    borderRadius: 12,
    width: "100%",
    alignItems: "center",
    elevation: 2,
    marginBottom: 32,
  },
  statusLabel: {
    fontSize: 12,
    color: "#757575",
    marginBottom: 4,
  },
  statusValue: {
    fontSize: 24,
    fontWeight: "bold",
  },
  infoContainer: {
    alignItems: "center",
  },
  infoText: {
    fontSize: 14,
    color: "#757575",
    textAlign: "center",
    marginBottom: 24,
  },
  actionContainer: {
    alignItems: "center",
  },
  warningText: {
    fontSize: 14,
    color: "#757575",
    textAlign: "center",
    marginBottom: 32,
    lineHeight: 22,
  },
  emergencyButton: {
    backgroundColor: "#F44336",
    paddingHorizontal: 48,
    paddingVertical: 16,
    borderRadius: 28,
    elevation: 2,
  },
  emergencyButtonText: {
    color: "#FFFFFF",
    fontSize: 18,
    fontWeight: "600",
  },
  loadingContainer: {
    alignItems: "center",
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: "#757575",
  },
  successContainer: {
    alignItems: "center",
  },
  successTitle: {
    fontSize: 22,
    fontWeight: "bold",
    color: "#4CAF50",
    marginBottom: 12,
  },
  successScore: {
    fontSize: 18,
    color: "#212121",
    marginBottom: 4,
  },
  successStatus: {
    fontSize: 16,
    color: "#4CAF50",
    fontWeight: "600",
    marginBottom: 32,
  },
  backButton: {
    backgroundColor: "#757575",
    paddingHorizontal: 36,
    paddingVertical: 14,
    borderRadius: 28,
  },
  backButtonText: {
    color: "#FFFFFF",
    fontSize: 16,
    fontWeight: "600",
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
  retryButton: {
    backgroundColor: "#F44336",
    paddingHorizontal: 36,
    paddingVertical: 14,
    borderRadius: 28,
  },
  retryButtonText: {
    color: "#FFFFFF",
    fontSize: 16,
    fontWeight: "600",
  },
});
